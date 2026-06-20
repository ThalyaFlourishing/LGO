//! Slot resolver — name → canonical Slot lookup from `data/lgo_items.json`,
//! plus end-to-end `.toml` rewrite.
//!
//! Model guidance: high borrow-checker/algorithmic friction — see `docs/MODEL_GUIDANCE.md` before non-trivial edits.
//!
//! Two layers:
//!
//! 1. `ItemsDb` — in-memory name → Slot index.
//! 2. `resolve_stats_file` / `resolve_toml_str` — read a bookmarklet-produced
//!    `.toml`, look up each item by name, rewrite the `slot` field to the
//!    canonical Display form, regroup items by slot family in canonical
//!    order with divider comments, and write the result to a new
//!    `*_resolved.toml`. Comments and per-item warnings from the input are
//!    preserved via `toml_edit`.
//!
//! See `docs/RESOLVER_DESIGN.md` for the overall design.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use toml_edit::{value, ArrayOfTables, DocumentMut, Table};

use crate::gear::Slot;
use crate::stat::TRACKED_STATS;

/// Default path to the offline items DB, relative to the working directory.
pub const DEFAULT_ITEMS_DB_PATH: &str = "data/lgo_items.json";

// =============================================================================
// ItemsDb — name → Slot index
// =============================================================================

/// In-memory name → canonical Slot index, built once from
/// `data/lgo_items.json` at startup.
///
/// Lookups are O(1). The internal `Vec<DbItem>` per name is a deliberate
/// shape choice (see RESOLVER_DESIGN.md §8) leaving room for future
/// disambiguation by tier / item-level / quality if that ever becomes
/// necessary; today, JSON object keys are unique by construction so each
/// Vec has length 1.
#[derive(Debug)]
pub struct ItemsDb {
    by_name: HashMap<String, Vec<DbItem>>,
}

/// A single resolved entry: an item name and its canonical equipment Slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbItem {
    pub name: String,
    pub slot: Slot,
}

/// Errors that can occur when loading or parsing the items DB.
#[derive(Debug)]
pub enum ItemsDbError {
    /// I/O failure reading the file.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// File contents are not valid JSON, or do not match the expected schema.
    ParseJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    /// A DB entry's `slot` field is not a recognised JSON-form Slot variant
    /// (see `Slot::from_json_variant`). Indicates the JSON file's schema has
    /// drifted from what `gear.rs` knows about.
    UnknownSlot {
        item_name: String,
        slot_string: String,
    },
}

impl std::fmt::Display for ItemsDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemsDbError::Io { path, source } => {
                write!(f, "Cannot read items DB '{}': {}", path.display(), source)
            }
            ItemsDbError::ParseJson { path, source } => {
                write!(f, "Cannot parse items DB '{}': {}", path.display(), source)
            }
            ItemsDbError::UnknownSlot {
                item_name,
                slot_string,
            } => write!(
                f,
                "Items DB entry for item '{}' has unrecognised slot '{}'",
                item_name, slot_string
            ),
        }
    }
}

impl std::error::Error for ItemsDbError {}

/// Raw shape of an entry in `data/lgo_items.json`.
///
/// The actual file also has a `stats` field, but the resolver doesn't care
/// about stats — only slots. Serde silently ignores unknown fields by
/// default, so the `stats` object is skipped without ceremony.
#[derive(Debug, Deserialize)]
struct RawEntry {
    name: String,
    slot: String,
}

impl ItemsDb {
    /// Load from the default path (`data/lgo_items.json`).
    pub fn load_default() -> Result<Self, ItemsDbError> {
        Self::load(Path::new(DEFAULT_ITEMS_DB_PATH))
    }

    /// Load from an explicit path. Useful in tests against synthetic fixtures.
    pub fn load(path: &Path) -> Result<Self, ItemsDbError> {
        let json = fs::read_to_string(path).map_err(|e| ItemsDbError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Self::from_json_str(&json, path)
    }

    /// Parse a JSON string. Split out from `load` so tests can avoid disk I/O.
    pub fn from_json_str(json: &str, path_for_errors: &Path) -> Result<Self, ItemsDbError> {
        // Top-level shape: { "<item name>": { name, slot, stats }, ... }
        let raw: HashMap<String, RawEntry> =
            serde_json::from_str(json).map_err(|e| ItemsDbError::ParseJson {
                path: path_for_errors.to_path_buf(),
                source: e,
            })?;

        let mut by_name: HashMap<String, Vec<DbItem>> = HashMap::new();
        for (_key, entry) in raw {
            // Trust the inner `name` field; the outer key is identical in the
            // real file but the inner field is what `db_build` was guaranteed
            // to write and is thus the authoritative copy.
            let slot =
                Slot::from_json_variant(&entry.slot).ok_or_else(|| ItemsDbError::UnknownSlot {
                    item_name: entry.name.clone(),
                    slot_string: entry.slot.clone(),
                })?;
            by_name.entry(entry.name.clone()).or_default().push(DbItem {
                name: entry.name,
                slot,
            });
        }

        Ok(ItemsDb { by_name })
    }

    /// Number of unique item names in the index.
    #[allow(dead_code)] // Public API; exercised by tests only — `main.rs` doesn't need it yet.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// True if no items were loaded.
    #[allow(dead_code)] // Public API; exercised by tests only — `main.rs` doesn't need it yet.
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// Return the canonical equipment Slot for an item name, if known.
    ///
    /// Returns `None` for unknown names — typically legendary / player-renamed
    /// items that the bookmarklet user must annotate by hand, or genuinely
    /// new items added to the game after `data/lgo_items.json` was last
    /// rebuilt.
    ///
    /// Resolution policy is "first match wins" (RESOLVER_DESIGN.md §9). In
    /// practice every Vec has length 1 because JSON object keys are unique;
    /// the Vec is structural future-proofing only.
    pub fn lookup(&self, name: &str) -> Option<Slot> {
        self.by_name
            .get(name)
            .and_then(|v| v.first())
            .map(|item| item.slot)
    }
}

// =============================================================================
// Resolution outcomes / report
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionOutcome {
    /// The item's name was found in the DB; its slot was rewritten.
    Resolved {
        name: String,
        /// The original `slot = "..."` string from the input file, if any.
        from_slot: Option<String>,
        /// The canonical Slot the resolver wrote.
        to_slot: Slot,
    },
    /// The item's name was not found in the DB; its slot was left as-is.
    Unknown {
        name: String,
        original_slot: Option<String>,
        reason: UnknownReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnknownReason {
    /// Not present in `data/lgo_items.json`. Typically: legendary / renamed
    /// items, or items added to the game after the most recent DB build.
    NotInDb,
}

#[derive(Debug)]
pub struct Report {
    /// Per-merge outcome (added/preserved/overwritten/removed/unknown).
    pub outcome: MergeOutcome,
    /// The bookmarklet output that drove this merge, if any. `None` means
    /// no new export was found and the canonical file is unchanged.
    pub bookmarklet_path: Option<PathBuf>,
    /// The canonical merged file (always reported, written when an
    /// export was processed).
    pub canonical_path: PathBuf,
    /// True if a canonical file existed before this run.
    pub previous_existed: bool,
    /// True if there was no bookmarklet output and the canonical file was
    /// left untouched. When set, `outcome` is empty.
    pub no_new_export: bool,
}

#[derive(Debug)]
pub enum ResolveError {
    IoRead {
        path: PathBuf,
        source: std::io::Error,
    },
    IoWrite {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseToml {
        path: PathBuf,
        source: toml_edit::TomlError,
    },
    NoItemsArray {
        path: PathBuf,
    },
    /// Neither `lgo_<character>_stats.toml` nor `lgo_<character>_gear.toml`
    /// exists in the AllServers directory.
    NoInputFiles {
        dir: PathBuf,
        character: String,
    },
    /// Two or more files match a single character query case-insensitively.
    /// Cannot occur on Windows; on Linux this surfaces as a clean error.
    AmbiguousFiles {
        message: String,
    },
    /// `--force` was passed but stdin is not a terminal. Auto-accepting
    /// destructive changes is exactly the failure mode `--force` is meant
    /// to guard against, so we refuse.
    ForceRequiresTty,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::IoRead { path, source } => {
                write!(f, "Cannot read '{}': {}", path.display(), source)
            }
            ResolveError::IoWrite { path, source } => {
                write!(f, "Cannot write '{}': {}", path.display(), source)
            }
            ResolveError::ParseToml { path, source } => {
                write!(f, "Cannot parse '{}': {}", path.display(), source)
            }
            ResolveError::NoItemsArray { path } => {
                write!(f, "No [[item]] entries in '{}'", path.display())
            }
            ResolveError::NoInputFiles { dir, character } => write!(
                f,
                "No lgo_{}_stats.toml or lgo_{}_gear.toml found in {}",
                character,
                character,
                dir.display()
            ),
            ResolveError::AmbiguousFiles { message } => write!(f, "Error: {}", message),
            ResolveError::ForceRequiresTty => {
                write!(f, "--force requires interactive stdin for prompts.")
            }
        }
    }
}

impl std::error::Error for ResolveError {}

// =============================================================================
// Slot family (paired-slot grouping for output)
// =============================================================================

/// Collapse paired slots into a single "family" for grouping purposes.
/// Wrist1/Wrist2 → Wrist1; Finger1/Finger2 → Finger1; Ear1/Ear2 → Ear1.
/// Other slots are returned unchanged.
fn slot_family(s: Slot) -> Slot {
    match s {
        Slot::Wrist2 => Slot::Wrist1,
        Slot::Finger2 => Slot::Finger1,
        Slot::Ear2 => Slot::Ear1,
        other => other,
    }
}

/// Human-readable label for a slot family used in output divider comments.
fn slot_family_label(family: Slot) -> &'static str {
    match family {
        Slot::Wrist1 => "Wrist",
        Slot::Finger1 => "Finger",
        Slot::Ear1 => "Ear",
        Slot::Head => "Head",
        Slot::Chest => "Chest",
        Slot::Legs => "Legs",
        Slot::Hands => "Hands",
        Slot::Feet => "Feet",
        Slot::Shoulders => "Shoulders",
        Slot::Back => "Back",
        Slot::Neck => "Neck",
        Slot::Pocket => "Pocket",
        Slot::MainHand => "Main-hand",
        Slot::OffHand => "Off-hand",
        Slot::Ranged => "Ranged",
        Slot::ClassItem => "Class Item",
        // Slot2 variants never reach here because slot_family collapses them.
        Slot::Wrist2 | Slot::Finger2 | Slot::Ear2 => "Unknown",
    }
}

/// Order in which slot families should appear in the output, matching the
/// canonical Slot::ALL traversal (paired slots collapsed to their first
/// representative).
fn slot_family_order() -> Vec<Slot> {
    let mut seen: Vec<Slot> = Vec::new();
    for &slot in Slot::ALL {
        let family = slot_family(slot);
        if !seen.contains(&family) {
            seen.push(family);
        }
    }
    seen
}

// =============================================================================
// resolve_toml_str — pure function (no I/O)
// =============================================================================

/// Pure: parse `src`, resolve slots via `db`, return new TOML text plus
/// per-item outcomes. No file I/O — the file-level wrapper
/// `resolve_stats_file` is the I/O caller.
pub fn resolve_toml_str(
    src: &str,
    db: &ItemsDb,
) -> Result<(String, Vec<ResolutionOutcome>), ResolveError> {
    let mut doc: DocumentMut = src.parse().map_err(|e| ResolveError::ParseToml {
        path: PathBuf::from("<in-memory>"),
        source: e,
    })?;

    {
        let items_arr = doc
            .get_mut("item")
            .and_then(|i| i.as_array_of_tables_mut())
            .ok_or_else(|| ResolveError::NoItemsArray {
                path: PathBuf::from("<in-memory>"),
            })?;

        // Take ownership of the existing entries by replacing the array with
        // an empty one, then rebuild from scratch in canonical order.
        let taken = std::mem::replace(items_arr, ArrayOfTables::new());
        let original_tables: Vec<Table> = taken.iter().cloned().collect();

        let outcomes_and_buckets = bucket_items(original_tables, db);
        let (mut buckets, unknowns, outcomes_local) = outcomes_and_buckets;

        // Rebuild the array in canonical family order with divider comments.
        // Dividers use plain ASCII so the resolved file renders cleanly in
        // any terminal/editor (no Unicode box-drawing dependency).
        //
        // `next_pos` is threaded through push_group so each table receives a
        // monotonically increasing `position`, overriding the original-source
        // positions that toml_edit attached at parse time. Without this, the
        // renderer emits tables in original-source order regardless of the
        // order we push them into the new ArrayOfTables.
        let mut new_arr = ArrayOfTables::new();
        let mut next_pos: usize = 0;
        for family in slot_family_order() {
            if let Some(group_items) = buckets.remove(&family) {
                if group_items.is_empty() {
                    continue;
                }
                push_group(
                    &mut new_arr,
                    group_items,
                    slot_family_label(family),
                    &mut next_pos,
                );
            }
        }
        if !unknowns.is_empty() {
            push_group(
                &mut new_arr,
                unknowns,
                "Unknown (not in items DB)",
                &mut next_pos,
            );
        }

        *items_arr = new_arr;

        // Stash outcomes in an outer-scope binding by returning early.
        return Ok((doc.to_string(), outcomes_local));
    }
}

/// Helper: bucket the input tables by canonical slot family (resolved) /
/// unknown bucket (not in DB), and emit a `ResolutionOutcome` per item.
#[allow(clippy::type_complexity)]
fn bucket_items(
    tables: Vec<Table>,
    db: &ItemsDb,
) -> (
    HashMap<Slot, Vec<Table>>,
    Vec<Table>,
    Vec<ResolutionOutcome>,
) {
    let mut buckets: HashMap<Slot, Vec<Table>> = HashMap::new();
    let mut unknowns: Vec<Table> = Vec::new();
    let mut outcomes: Vec<ResolutionOutcome> = Vec::new();

    for mut table in tables {
        let name = table
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let original_slot = table
            .get("slot")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match db.lookup(&name) {
            Some(slot) => {
                // Rewrite slot field to canonical Display form. Existing key
                // decor (whitespace alignment) is preserved because we're
                // replacing an existing key, not creating a new one.
                table.insert("slot", value(slot.to_string()));
                outcomes.push(ResolutionOutcome::Resolved {
                    name,
                    from_slot: original_slot,
                    to_slot: slot,
                });
                buckets.entry(slot_family(slot)).or_default().push(table);
            }
            None => {
                outcomes.push(ResolutionOutcome::Unknown {
                    name,
                    original_slot,
                    reason: UnknownReason::NotInDb,
                });
                unknowns.push(table);
            }
        }
    }

    (buckets, unknowns, outcomes)
}

/// Push a slot group onto the new array of tables, inserting a divider
/// comment on the first table after any pre-existing prefix decor.
/// Each table is also assigned a fresh sequential
/// `position` (via `next_pos`) so the renderer emits the rebuilt array in
/// the order we constructed it, ignoring the original-source positions.
fn push_group(arr: &mut ArrayOfTables, items: Vec<Table>, label: &str, next_pos: &mut usize) {
    for (i, mut table) in items.into_iter().enumerate() {
        if i == 0 {
            let existing = table
                .decor()
                .prefix()
                .and_then(|s| s.as_str())
                .unwrap_or("");
            let base = existing.trim_end_matches('\n');
            let new_prefix = if base.is_empty() {
                format!("\n# --- {} ---\n\n", label)
            } else {
                format!("{}\n\n# --- {} ---\n\n", base, label)
            };
            table.decor_mut().set_prefix(new_prefix);
        }
        table.set_position(*next_pos);
        *next_pos += 1;
        arr.push(table);
    }
}

// =============================================================================
// Merge layer — preserve hand-edits across re-runs
// =============================================================================
//
// Behaviour on iteration:
//   * Items present in both `previous` and `incoming` (matched by name)
//     are kept verbatim from `previous` by default.
//     When the same name appears multiple times (duplicate owned instances):
//       1. Exact-equal instances are paired first (same name+slot+stats).
//       2. Remaining same-name instances are paired in stable occurrence order.
//       3. Previous-only leftovers are removal candidates.
//       4. Incoming-only leftovers are additions.
//   * Items present only in `incoming` are added.
//   * Items present only in `previous` are removed (disappeared from export).
//   * `--force` opts into prompting the user per item before destructive
//     changes; identical-data items remain a no-op even under `--force`.
//
// See `docs/merge-brief.md` and `docs/AGENT_CONTEXT.md` §10.

/// Per-item user prompt categories under `--force`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptCategory {
    /// The item exists in both files and the incoming data differs from
    /// the previous data; ask whether to overwrite the previous block.
    Overwrite,
    /// The item exists in `previous` but is missing from `incoming`; ask
    /// whether to remove it.
    Remove,
}

/// User answer to a prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptAnswer {
    /// Apply this destructive change.
    Yes,
    /// Skip this destructive change (keep the previous block / retain item).
    No,
    /// Apply this change and all *remaining* changes in the same category
    /// without further prompting. Does not affect the other category.
    YesToAll,
}

/// Pluggable per-item prompt implementation. Production code uses
/// `StdinPrompter`; tests use `ScriptedPrompter`.
pub trait Prompter {
    fn prompt(&mut self, category: PromptCategory, item_name: &str) -> PromptAnswer;
}

/// Reads from stdin, writes to stderr (so prompt text doesn't pollute
/// redirected stdout). Re-prompts on unrecognised input rather than
/// crashing.
pub struct StdinPrompter;

impl Prompter for StdinPrompter {
    fn prompt(&mut self, category: PromptCategory, item_name: &str) -> PromptAnswer {
        let question = match category {
            PromptCategory::Overwrite => format!("Overwrite stats for \"{}\"? (y/n/a)", item_name),
            PromptCategory::Remove => {
                format!("Remove \"{}\" (no longer in export)? (y/n/a)", item_name)
            }
        };
        let stdin = std::io::stdin();
        let mut stderr = std::io::stderr();
        loop {
            let _ = write!(stderr, "{} ", question);
            let _ = stderr.flush();
            let mut line = String::new();
            // EOF on stdin during a `--force` run is unrecoverable; treat
            // it as a No so we keep the previous block rather than silently
            // applying a destructive change.
            if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
                let _ = writeln!(stderr, "(stdin closed; treating as 'n')");
                return PromptAnswer::No;
            }
            match line.trim() {
                "y" | "Y" => return PromptAnswer::Yes,
                "n" | "N" => return PromptAnswer::No,
                "a" | "A" => return PromptAnswer::YesToAll,
                _ => {
                    let _ = writeln!(stderr, "Please answer y, n, or a.");
                }
            }
        }
    }
}

/// Force mode: either off, or on with a Prompter for per-item decisions.
pub enum ForceMode {
    NoForce,
    Force { prompter: Box<dyn Prompter> },
}

impl std::fmt::Debug for ForceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForceMode::NoForce => f.write_str("NoForce"),
            ForceMode::Force { .. } => f.write_str("Force { .. }"),
        }
    }
}

/// Per-merge summary returned to the caller.
#[derive(Debug, Default)]
pub struct MergeOutcome {
    pub added: Vec<String>,
    pub preserved: Vec<String>,
    pub overwritten: Vec<String>,
    pub removed: Vec<String>,
    /// Item names whose slot was still `Unknown` after slot resolution
    /// (i.e. not in `data/lgo_items.json`). Reported to the user as
    /// candidates for hand-editing.
    pub unknown_slot: Vec<String>,
    /// The merged TOML text the caller should write to the canonical path.
    pub merged_text: String,
}

/// Pure: combine `previous` (canonical file contents, if any) with
/// `incoming_resolved` (slot-resolved bookmarklet output) per the merge
/// rules above. Performs no I/O; all prompting goes through the
/// `Prompter` carried by `force`.
///
/// `previous = None` is the first-run case: the canonical file is taken
/// from `incoming_resolved` verbatim and every item is reported as
/// `added`.
pub fn merge_into_canonical(
    previous: Option<&str>,
    incoming_resolved: &str,
    mut force: ForceMode,
) -> Result<MergeOutcome, ResolveError> {
    // Collect names whose resolved slot is still "Unknown" in incoming —
    // reported regardless of merge outcome (the user may need to hand-edit).
    let unknown_slot = collect_unknown_slot_names(incoming_resolved)?;

    // First run: take incoming verbatim. All items are "added".
    let Some(previous_src) = previous else {
        let added = item_names(incoming_resolved)?;
        return Ok(MergeOutcome {
            added,
            preserved: Vec::new(),
            overwritten: Vec::new(),
            removed: Vec::new(),
            unknown_slot,
            merged_text: incoming_resolved.to_string(),
        });
    };

    // Subsequent run: start from `previous` (so the document header /
    // top-level decor round-trips), replace its `[[item]]` array with the
    // merged set, and let push_group regroup by family.
    let mut prev_doc: DocumentMut = previous_src.parse().map_err(|e| ResolveError::ParseToml {
        path: PathBuf::from("<previous>"),
        source: e,
    })?;
    let mut incoming_doc: DocumentMut =
        incoming_resolved
            .parse()
            .map_err(|e| ResolveError::ParseToml {
                path: PathBuf::from("<incoming>"),
                source: e,
            })?;

    let prev_tables = take_item_tables(&mut prev_doc, "<previous>")?;
    let incoming_tables = take_item_tables(&mut incoming_doc, "<incoming>")?;

    // Carry forward `character` and `class` from the incoming resolved TOML
    // into the canonical document so the metadata is always current after a merge.
    for key in &["character", "class"] {
        if let Some(val) = incoming_doc.get(key).cloned() {
            prev_doc.insert(key, val);
        }
    }

    let incoming_by_name: HashMap<String, Vec<Table>> = {
        let mut m: HashMap<String, Vec<Table>> = HashMap::new();
        for t in incoming_tables {
            if let Some(name) = table_name(&t) {
                m.entry(name).or_default().push(t);
            }
            // Incoming tables without a `name` field are dropped: they cannot
            // be matched to any previous item and would corrupt the merge.
            // Previous nameless tables are handled separately below and are
            // preserved in-place unchanged.
        }
        m
    };

    let mut outcome = MergeOutcome {
        unknown_slot,
        ..MergeOutcome::default()
    };

    // Two independent "yes to all" flags: `a` for overwrite does not
    // auto-accept removals and vice versa.
    let mut yes_all_overwrite = false;
    let mut yes_all_remove = false;

    let mut merged_tables: Vec<Table> = Vec::new();

    // Build an ordered list of unique names from `prev_tables` (first-
    // occurrence order), plus a per-name Vec of the actual prev tables.
    // Nameless tables are preserved in-place without merge participation.
    let mut prev_order: Vec<String> = Vec::new();
    let mut prev_groups: HashMap<String, Vec<Table>> = HashMap::new();
    for t in prev_tables {
        match table_name(&t) {
            Some(name) => {
                if !prev_groups.contains_key(&name) {
                    prev_order.push(name.clone());
                }
                prev_groups.entry(name).or_default().push(t);
            }
            None => {
                // No `name` field — preserve in place; can't participate
                // in merge. (Shouldn't happen in practice; gearstats
                // reading would have already rejected such an item.)
                merged_tables.push(t);
            }
        }
    }

    // Track which incoming names have been fully processed so we can find
    // names that are new (incoming-only) at the end.
    let mut processed_incoming_names: HashSet<String> = HashSet::new();

    for name in &prev_order {
        let prev_list = prev_groups.remove(name).unwrap_or_default();
        // Remove from the map so new-only names are what remains at the end.
        let incoming_list: Vec<Table> = incoming_by_name
            .get(name)
            .cloned()
            .unwrap_or_default();
        processed_incoming_names.insert(name.clone());

        // ── Phase 1: pair exact-equal instances first ────────────────────
        // For each prev item, find the first unmatched incoming item that is
        // field-equal (name + slot + all tracked stats).  These pairs are
        // preserved unconditionally — identical data never needs a prompt.
        let n_inc = incoming_list.len();
        let mut incoming_claimed = vec![false; n_inc];
        let mut prev_exact_matched = vec![false; prev_list.len()];

        for (pi, prev) in prev_list.iter().enumerate() {
            if let Some(ii) = incoming_list
                .iter()
                .enumerate()
                .find(|(i, inc)| !incoming_claimed[*i] && item_data_equal(prev, inc))
                .map(|(i, _)| i)
            {
                incoming_claimed[ii] = true;
                prev_exact_matched[pi] = true;
            }
        }

        // Collect unmatched incoming items in their original stable order.
        let mut remaining_incoming: Vec<Table> = incoming_list
            .into_iter()
            .enumerate()
            .filter_map(|(i, t)| if !incoming_claimed[i] { Some(t) } else { None })
            .collect();
        let mut remaining_inc_iter = remaining_incoming.drain(..);

        // ── Phases 2/3/4: process each prev item ────────────────────────
        for (pi, prev) in prev_list.into_iter().enumerate() {
            if prev_exact_matched[pi] {
                // Phase 1 result: identical — always preserve, no prompt.
                outcome.preserved.push(name.clone());
                merged_tables.push(prev);
                continue;
            }

            if let Some(incoming) = remaining_inc_iter.next() {
                // Phase 2: pair with the next unmatched incoming in order.
                match &mut force {
                    ForceMode::NoForce => {
                        // Preserve-by-default: keep previous (hand-edit
                        // preservation policy).
                        outcome.preserved.push(name.clone());
                        merged_tables.push(prev);
                    }
                    ForceMode::Force { prompter } => {
                        // Phase 1 consumed all exact-equal pairs; this
                        // incoming differs from prev, so prompt the user.
                        let answer = if yes_all_overwrite {
                            PromptAnswer::Yes
                        } else {
                            prompter.prompt(PromptCategory::Overwrite, name)
                        };
                        match answer {
                            PromptAnswer::YesToAll => {
                                yes_all_overwrite = true;
                                outcome.overwritten.push(name.clone());
                                merged_tables.push(incoming);
                            }
                            PromptAnswer::Yes => {
                                outcome.overwritten.push(name.clone());
                                merged_tables.push(incoming);
                            }
                            PromptAnswer::No => {
                                outcome.preserved.push(name.clone());
                                merged_tables.push(prev);
                            }
                        }
                    }
                }
            } else {
                // Phase 3: more prev instances than incoming — removal
                // candidate (this instance disappeared from the export).
                match &mut force {
                    ForceMode::NoForce => {
                        outcome.removed.push(name.clone());
                        // prev is dropped.
                    }
                    ForceMode::Force { prompter } => {
                        let answer = if yes_all_remove {
                            PromptAnswer::Yes
                        } else {
                            prompter.prompt(PromptCategory::Remove, name)
                        };
                        match answer {
                            PromptAnswer::YesToAll => {
                                yes_all_remove = true;
                                outcome.removed.push(name.clone());
                            }
                            PromptAnswer::Yes => {
                                outcome.removed.push(name.clone());
                            }
                            PromptAnswer::No => {
                                outcome.preserved.push(name.clone());
                                merged_tables.push(prev);
                            }
                        }
                    }
                }
            }
        }

        // Phase 4: more incoming instances than prev — leftover incoming
        // items are new additions (no prompt).
        for incoming in remaining_inc_iter {
            outcome.added.push(name.clone());
            merged_tables.push(incoming);
        }
    }

    // Items whose name was not in `previous` at all → added (no prompt).
    // Sort for deterministic output order.
    let mut new_names: Vec<&String> = incoming_by_name
        .keys()
        .filter(|n| !processed_incoming_names.contains(*n))
        .collect();
    new_names.sort();
    for name in new_names {
        for table in incoming_by_name.get(name).into_iter().flatten() {
            outcome.added.push(name.clone());
            merged_tables.push(table.clone());
        }
    }

    // Regroup the merged set by canonical slot family. Strip any
    // pre-existing `# --- ... ---` divider lines from each table's prefix
    // first so dividers don't accumulate across runs (idempotency).
    for t in &mut merged_tables {
        strip_family_dividers_from_prefix(t);
    }

    let (mut buckets, unknowns) = bucket_by_table_slot(merged_tables);

    let mut new_arr = ArrayOfTables::new();
    let mut next_pos: usize = 0;
    for family in slot_family_order() {
        if let Some(group_items) = buckets.remove(&family) {
            if group_items.is_empty() {
                continue;
            }
            push_group(
                &mut new_arr,
                group_items,
                slot_family_label(family),
                &mut next_pos,
            );
        }
    }
    if !unknowns.is_empty() {
        push_group(
            &mut new_arr,
            unknowns,
            "Unknown (not in items DB)",
            &mut next_pos,
        );
    }

    // Replace prev_doc's items array with the merged one.
    let prev_items = prev_doc
        .get_mut("item")
        .and_then(|i| i.as_array_of_tables_mut())
        .ok_or_else(|| ResolveError::NoItemsArray {
            path: PathBuf::from("<previous>"),
        })?;
    *prev_items = new_arr;

    outcome.merged_text = prev_doc.to_string();
    Ok(outcome)
}

/// Bucket pre-resolved tables by canonical slot family, looking only at
/// each table's own `slot` field. Tables with an unrecognised slot string
/// fall into the `Unknown` group.
fn bucket_by_table_slot(tables: Vec<Table>) -> (HashMap<Slot, Vec<Table>>, Vec<Table>) {
    let mut buckets: HashMap<Slot, Vec<Table>> = HashMap::new();
    let mut unknowns: Vec<Table> = Vec::new();
    for table in tables {
        let slot_str = table.get("slot").and_then(|v| v.as_str()).unwrap_or("");
        match crate::gearstats::parse_slot_display(slot_str) {
            Some(slot) => {
                buckets.entry(slot_family(slot)).or_default().push(table);
            }
            None => {
                unknowns.push(table);
            }
        }
    }
    (buckets, unknowns)
}

/// Strip any line of the form `# --- ... ---` from a table's prefix decor.
/// Used during the merge regroup so divider comments don't accumulate
/// across re-runs.
fn strip_family_dividers_from_prefix(table: &mut Table) {
    let prefix = match table.decor().prefix().and_then(|s| s.as_str()) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return,
    };
    let kept: Vec<&str> = prefix
        .split_inclusive('\n')
        .filter(|line| {
            let trimmed = line.trim_end_matches('\n').trim();
            !(trimmed.starts_with("# ---") && trimmed.ends_with("---"))
        })
        .collect();
    let new_prefix: String = kept.concat();
    table.decor_mut().set_prefix(new_prefix);
}

/// Extract `[[item]]` tables from a parsed document, leaving the array
/// itself empty (caller can refill it).
fn take_item_tables(doc: &mut DocumentMut, label: &str) -> Result<Vec<Table>, ResolveError> {
    let arr = doc
        .get_mut("item")
        .and_then(|i| i.as_array_of_tables_mut())
        .ok_or_else(|| ResolveError::NoItemsArray {
            path: PathBuf::from(label),
        })?;
    let taken = std::mem::replace(arr, ArrayOfTables::new());
    Ok(taken.iter().cloned().collect())
}

fn table_name(t: &Table) -> Option<String> {
    t.get("name").and_then(|v| v.as_str()).map(String::from)
}

/// Return `[[item]]` `name`s in document order.
fn item_names(src: &str) -> Result<Vec<String>, ResolveError> {
    let doc: DocumentMut = src.parse().map_err(|e| ResolveError::ParseToml {
        path: PathBuf::from("<incoming>"),
        source: e,
    })?;
    let arr = doc
        .get("item")
        .and_then(|i| i.as_array_of_tables())
        .ok_or_else(|| ResolveError::NoItemsArray {
            path: PathBuf::from("<incoming>"),
        })?;
    Ok(arr.iter().filter_map(table_name).collect())
}

/// Names of items whose canonical `slot` field is the literal string
/// `"Unknown"` after resolution — i.e. items the resolver couldn't map
/// to a canonical slot. Reported to the user as candidates for
/// hand-editing.
fn collect_unknown_slot_names(src: &str) -> Result<Vec<String>, ResolveError> {
    let doc: DocumentMut = src.parse().map_err(|e| ResolveError::ParseToml {
        path: PathBuf::from("<incoming>"),
        source: e,
    })?;
    let arr = doc
        .get("item")
        .and_then(|i| i.as_array_of_tables())
        .ok_or_else(|| ResolveError::NoItemsArray {
            path: PathBuf::from("<incoming>"),
        })?;
    Ok(arr
        .iter()
        .filter(|t| t.get("slot").and_then(|v| v.as_str()) == Some("Unknown"))
        .filter_map(table_name)
        .collect())
}

/// Compare two `[[item]]` tables on their canonical fields only:
/// `name`, `slot`, and the 14 tracked stats. Comments, whitespace,
/// and other decor are ignored.
fn item_data_equal(a: &Table, b: &Table) -> bool {
    if table_str(a, "name") != table_str(b, "name") {
        return false;
    }
    if table_str(a, "slot") != table_str(b, "slot") {
        return false;
    }
    for (_, key) in TRACKED_STATS {
        if table_int(a, key) != table_int(b, key) {
            return false;
        }
    }
    true
}

fn table_str(t: &Table, key: &str) -> Option<String> {
    t.get(key).and_then(|v| v.as_str()).map(String::from)
}

fn table_int(t: &Table, key: &str) -> Option<i64> {
    t.get(key).and_then(|v| v.as_integer())
}

// =============================================================================
// resolve_stats_file — file-level wrapper (does I/O)
// =============================================================================

/// End-to-end iteration step:
///
///   1. Read `lgo_<character>_stats.toml` (the bookmarklet's output).
///   2. Slot-resolve it via `db`.
///   3. Read `lgo_<character>_gear.toml` (the canonical merged file) if
///      it exists.
///   4. Merge per `merge_into_canonical` semantics.
///   5. Write the result back to `lgo_<character>_gear.toml`.
///
/// Returns a `Report` describing what happened, for `main.rs` to display.
///
/// If no bookmarklet output file is present, the canonical file is left
/// untouched and the returned `Report` carries `bookmarklet_path = None`.
pub fn resolve_stats_file(
    char_dir: &Path,
    character: &str,
    db: &ItemsDb,
    force: ForceMode,
) -> Result<Report, ResolveError> {
    // --- Case-insensitive lookups (read path) --------------------------------
    // Use directory scans so that e.g. `lgo_thalya_stats.toml` is found when
    // querying with `"Thalya"`. Collisions (two files differing only in case)
    // return an error; this cannot happen on Windows but is caught cleanly on
    // Linux.
    let bookmarklet_found = crate::gearstats::find_bookmarklet_output(char_dir, character)
        .map_err(|msg| ResolveError::AmbiguousFiles { message: msg })?;
    let canonical_found = crate::gearstats::find_canonical_gear_file(char_dir, character)
        .map_err(|msg| ResolveError::AmbiguousFiles { message: msg })?;

    // Write-follows-read: if an existing canonical file was found (possibly
    // with different casing), write back to that exact path.  If none exists
    // yet, create at the path derived from the supplied character name.
    let canonical_path = canonical_found
        .clone()
        .unwrap_or_else(|| canonical_gear_path(char_dir, character));

    let bookmarklet_exists = bookmarklet_found.is_some();
    let canonical_existed = canonical_found.is_some();

    if !bookmarklet_exists {
        // No new export. If the canonical file exists, leave it alone and
        // report. If neither file exists, that's a hard error.
        if !canonical_existed {
            return Err(ResolveError::NoInputFiles {
                dir: char_dir.to_path_buf(),
                character: character.to_string(),
            });
        }
        return Ok(Report {
            outcome: MergeOutcome::default(),
            bookmarklet_path: None,
            canonical_path,
            previous_existed: true,
            no_new_export: true,
        });
    }

    let bookmarklet_path = bookmarklet_found.unwrap();

    // `--force` requires interactive stdin. Reject piped input loudly
    // rather than silently auto-accepting destructive changes.
    if matches!(force, ForceMode::Force { .. }) && !std::io::stdin().is_terminal() {
        return Err(ResolveError::ForceRequiresTty);
    }

    let bookmarklet_src =
        fs::read_to_string(&bookmarklet_path).map_err(|e| ResolveError::IoRead {
            path: bookmarklet_path.clone(),
            source: e,
        })?;

    let (resolved_src, _outcomes) =
        resolve_toml_str(&bookmarklet_src, db).map_err(|e| match e {
            ResolveError::ParseToml { source, .. } => ResolveError::ParseToml {
                path: bookmarklet_path.clone(),
                source,
            },
            ResolveError::NoItemsArray { .. } => ResolveError::NoItemsArray {
                path: bookmarklet_path.clone(),
            },
            other => other,
        })?;

    let previous_src = if canonical_existed {
        Some(
            fs::read_to_string(&canonical_path).map_err(|e| ResolveError::IoRead {
                path: canonical_path.clone(),
                source: e,
            })?,
        )
    } else {
        None
    };

    let outcome = merge_into_canonical(previous_src.as_deref(), &resolved_src, force).map_err(
        |e| match e {
            ResolveError::ParseToml { path, source } if path == Path::new("<previous>") => {
                ResolveError::ParseToml {
                    path: canonical_path.clone(),
                    source,
                }
            }
            ResolveError::ParseToml { path, source } if path == Path::new("<incoming>") => {
                ResolveError::ParseToml {
                    path: bookmarklet_path.clone(),
                    source,
                }
            }
            other => other,
        },
    )?;

    fs::write(&canonical_path, &outcome.merged_text).map_err(|e| ResolveError::IoWrite {
        path: canonical_path.clone(),
        source: e,
    })?;

    Ok(Report {
        outcome,
        bookmarklet_path: Some(bookmarklet_path),
        canonical_path,
        previous_existed: canonical_existed,
        no_new_export: false,
    })
}

/// Path the bookmarklet writes its TOML to, per the new file-naming
/// scheme: `<dir>/lgo_<character>_stats.toml`.
pub fn bookmarklet_stats_path(dir: &Path, character: &str) -> PathBuf {
    dir.join(format!("lgo_{}_stats.toml", character))
}

/// Canonical merged gear file: `<dir>/lgo_<character>_gear.toml`.
pub fn canonical_gear_path(dir: &Path, character: &str) -> PathBuf {
    dir.join(format!("lgo_{}_gear.toml", character))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Small synthetic fixture exercising five slot shapes:
    /// identity (Head), paired (Wrist1), weapon (MainHand), space-split
    /// (ClassItem), and an item with empty stats (legitimate in the real DB).
    const FIXTURE: &str = r#"{
        "Test Helm": {
            "name": "Test Helm",
            "slot": "Head",
            "stats": {}
        },
        "Test Bracelet": {
            "name": "Test Bracelet",
            "slot": "Wrist1",
            "stats": { "armor": 100 }
        },
        "Test Sword": {
            "name": "Test Sword",
            "slot": "MainHand",
            "stats": { "critical_rating": 50 }
        },
        "Test Tome": {
            "name": "Test Tome",
            "slot": "ClassItem",
            "stats": {}
        }
    }"#;

    fn dummy_path() -> &'static Path {
        Path::new("<test-fixture>")
    }

    fn fixture_db() -> ItemsDb {
        ItemsDb::from_json_str(FIXTURE, dummy_path()).expect("fixture must parse")
    }

    // -- ItemsDb tests (from step 3) --

    #[test]
    fn loads_fixture_and_resolves_known_items() {
        let db = fixture_db();
        assert_eq!(db.len(), 4);
        assert!(!db.is_empty());

        assert_eq!(db.lookup("Test Helm"), Some(Slot::Head));
        assert_eq!(db.lookup("Test Bracelet"), Some(Slot::Wrist1));
        assert_eq!(db.lookup("Test Sword"), Some(Slot::MainHand));
        assert_eq!(db.lookup("Test Tome"), Some(Slot::ClassItem));
    }

    #[test]
    fn lookup_returns_none_for_unknown_name() {
        let db = fixture_db();
        assert_eq!(db.lookup("Forgotten Elvish Healer's Hood"), None);
        assert_eq!(db.lookup(""), None);
    }

    #[test]
    fn unknown_slot_value_is_an_error() {
        let bad = r#"{
            "Bad Item": {
                "name": "Bad Item",
                "slot": "Frobnicate",
                "stats": {}
            }
        }"#;
        let err =
            ItemsDb::from_json_str(bad, dummy_path()).expect_err("unknown slot string must error");
        match err {
            ItemsDbError::UnknownSlot {
                item_name,
                slot_string,
            } => {
                assert_eq!(item_name, "Bad Item");
                assert_eq!(slot_string, "Frobnicate");
            }
            other => panic!("expected UnknownSlot, got {:?}", other),
        }
    }

    #[test]
    fn excluded_lotro_slot_is_an_error() {
        let bad = r#"{
            "Mining Pick": {
                "name": "Mining Pick",
                "slot": "CraftItem",
                "stats": {}
            }
        }"#;
        let err = ItemsDb::from_json_str(bad, dummy_path()).expect_err("excluded slot must error");
        assert!(matches!(err, ItemsDbError::UnknownSlot { .. }));
    }

    #[test]
    fn malformed_json_is_an_error() {
        let err = ItemsDb::from_json_str("{ this is not json", dummy_path())
            .expect_err("malformed JSON must error");
        assert!(matches!(err, ItemsDbError::ParseJson { .. }));
    }

    #[test]
    fn empty_object_yields_empty_db() {
        let db = ItemsDb::from_json_str("{}", dummy_path()).expect("empty {} must parse");
        assert_eq!(db.len(), 0);
        assert!(db.is_empty());
        assert_eq!(db.lookup("anything"), None);
    }

    #[test]
    fn lookup_is_case_sensitive() {
        let db = fixture_db();
        assert_eq!(db.lookup("test helm"), None);
        assert_eq!(db.lookup("TEST HELM"), None);
        assert_eq!(db.lookup("Test Helm"), Some(Slot::Head));
    }

    #[test]
    fn extra_fields_in_entry_are_ignored() {
        let extra = r#"{
            "Future Item": {
                "name": "Future Item",
                "slot": "Head",
                "stats": { "armor": 999 },
                "future_field": "some new thing"
            }
        }"#;
        let db =
            ItemsDb::from_json_str(extra, dummy_path()).expect("extra fields must be tolerated");
        assert_eq!(db.lookup("Future Item"), Some(Slot::Head));
    }

    // -- Real-DB integration test (ignored by default) --

    /// Confirms `data/lgo_items.json` actually loads end-to-end with the
    /// expected schema. Skipped in regular `cargo test` runs because it
    /// requires the 8 MB file on disk. To run it:
    ///
    ///     cargo test -- --ignored
    #[test]
    #[ignore = "requires data/lgo_items.json on disk; run with `cargo test -- --ignored`"]
    fn loads_real_items_db_smoke_test() {
        let db = ItemsDb::load_default().expect("real lgo_items.json must load");
        assert!(
            db.len() > 1000,
            "real DB should have many entries; got {}",
            db.len()
        );
        // Spot-check that at least one entry has a recognised Slot.
        let any_slot = db
            .by_name
            .values()
            .next()
            .and_then(|v| v.first())
            .map(|item| item.slot);
        assert!(any_slot.is_some(), "DB has entries but none has a slot");
    }

    // -- resolve_toml_str tests --

    #[test]
    fn resolves_known_item_to_canonical_slot() {
        let db = fixture_db();
        let input = "# header comment kept across resolution\n\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
Armor = 100\n";
        let (out, outcomes) = resolve_toml_str(input, &db).expect("must resolve");
        assert_eq!(outcomes.len(), 1);
        assert!(matches!(
            &outcomes[0],
            ResolutionOutcome::Resolved {
                to_slot: Slot::Head,
                ..
            }
        ));
        assert!(
            out.contains("slot = \"Head\""),
            "resolved output missing canonical slot:\n{}",
            out
        );
        assert!(
            out.contains("# header comment"),
            "header comment was lost:\n{}",
            out
        );
    }

    #[test]
    fn unknown_item_keeps_original_slot_and_records_outcome() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Lore-master's Staff of Legends\"\n";
        let (out, outcomes) = resolve_toml_str(input, &db).expect("must resolve");
        assert_eq!(outcomes.len(), 1);
        assert!(matches!(
            &outcomes[0],
            ResolutionOutcome::Unknown {
                reason: UnknownReason::NotInDb,
                ..
            }
        ));
        assert!(
            out.contains("slot = \"Unknown\""),
            "unknown item should retain original slot:\n{}",
            out
        );
    }

    /// Verifies the contract: items appear in the output in the same order
    /// their (resolved) slot families appear in `slot_family_order()`. Does
    /// not hard-code "Head before Main-hand" or any other specific pairing;
    /// if the canonical order ever changes, this test still passes as long
    /// as the resolver continues to honour whatever order is declared.
    #[test]
    fn output_orders_items_by_slot_family_order() {
        let db = fixture_db();
        // Provide items in scrambled order, all four fixture slots present.
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Tome\"\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Bracelet\"\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");

        // For each fixture item, what family does its slot resolve to?
        let item_to_family: Vec<(&str, Slot)> = vec![
            ("Test Helm", slot_family(Slot::Head)),
            ("Test Bracelet", slot_family(Slot::Wrist1)),
            ("Test Sword", slot_family(Slot::MainHand)),
            ("Test Tome", slot_family(Slot::ClassItem)),
        ];

        // Walk slot_family_order(); for each family that has a fixture item,
        // collect that item's name. The result is the expected sequence of
        // item appearances in the output.
        let order = slot_family_order();
        let expected_sequence: Vec<&str> = order
            .iter()
            .filter_map(|fam| {
                item_to_family
                    .iter()
                    .find(|(_, f)| f == fam)
                    .map(|(name, _)| *name)
            })
            .collect();

        // Find each expected item's position in the output, and assert that
        // those positions are strictly increasing.
        let positions: Vec<(&str, usize)> = expected_sequence
            .iter()
            .map(|name| {
                let pos = out
                    .find(name)
                    .unwrap_or_else(|| panic!("'{}' missing from output:\n{}", name, out));
                (*name, pos)
            })
            .collect();

        for window in positions.windows(2) {
            let (a_name, a_pos) = window[0];
            let (b_name, b_pos) = window[1];
            assert!(
                a_pos < b_pos,
                "'{}' (pos {}) should appear before '{}' (pos {}) per slot_family_order():\n{}",
                a_name,
                a_pos,
                b_name,
                b_pos,
                out
            );
        }
    }

    /// Pin the canonical family order itself, so that an unexpected change
    /// in `Slot::ALL` or `slot_family` (the inputs to `slot_family_order`)
    /// is caught directly, with a clear failure, rather than via a cascade
    /// of failures in higher-level tests.
    #[test]
    fn slot_family_order_is_canonical() {
        let order = slot_family_order();
        let expected = vec![
            Slot::Head,
            Slot::Chest,
            Slot::Legs,
            Slot::Hands,
            Slot::Feet,
            Slot::Shoulders,
            Slot::Back,
            Slot::Wrist1,
            Slot::Neck,
            Slot::Finger1,
            Slot::Ear1,
            Slot::Pocket,
            Slot::MainHand,
            Slot::OffHand,
            Slot::Ranged,
            Slot::ClassItem,
        ];
        assert_eq!(order, expected);
    }

    #[test]
    fn group_dividers_are_inserted_per_family() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Bracelet\"\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");
        assert!(
            out.contains("# --- Head ---"),
            "Head divider missing:\n{}",
            out
        );
        assert!(
            out.contains("# --- Wrist ---"),
            "Wrist divider missing:\n{}",
            out
        );
        assert!(
            out.contains("# --- Main-hand ---"),
            "Main-hand divider missing:\n{}",
            out
        );
    }

    #[test]
    fn unknown_items_get_their_own_section_at_end() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Mystery Renamed Legendary\"\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");
        let helm_pos = out.find("Test Helm").expect("Test Helm present");
        let mystery_pos = out
            .find("Mystery Renamed Legendary")
            .expect("Mystery item present");
        assert!(
            helm_pos < mystery_pos,
            "unknowns should come after resolved items:\n{}",
            out
        );
        assert!(
            out.contains("# --- Unknown (not in items DB) ---"),
            "unknown-section divider missing:\n{}",
            out
        );
    }

    #[test]
    fn warning_comments_inside_items_are_preserved() {
        // The bookmarklet writes "# WARNING: all stats unknown" inside
        // [[item]] blocks for legendary items. That comment lives as decor
        // on the next stat key (Armor). It must survive the rewrite.
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Mystery Item\"\n\
# WARNING: all stats unknown\n\
Armor = 0\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");
        assert!(
            out.contains("# WARNING: all stats unknown"),
            "per-item warning comment was lost:\n{}",
            out
        );
    }

    #[test]
    fn document_header_comments_precede_first_group_divider() {
        let db = fixture_db();
        let input = "\
# Doc header line 1\n\
# Doc header line 2\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");
        let hdr = out
            .find("# Doc header line 1")
            .expect("doc header preserved");
        let div = out.find("# --- Head ---").expect("divider present");
        assert!(
            hdr < div,
            "doc header must precede first divider, got:\n{}",
            out
        );
    }

    #[test]
    fn parse_error_surfaces_as_parse_error() {
        let db = fixture_db();
        let bad = "this is not valid toml = = =";
        let err = resolve_toml_str(bad, &db).expect_err("malformed TOML must error");
        assert!(matches!(err, ResolveError::ParseToml { .. }));
    }

    #[test]
    fn missing_item_array_surfaces_as_no_items_array() {
        let db = fixture_db();
        let no_items = "# bookmarklet output without [[item]] entries\n";
        let err = resolve_toml_str(no_items, &db).expect_err("missing array must error");
        assert!(matches!(err, ResolveError::NoItemsArray { .. }));
    }

    // -- compute_resolved_path tests retired —
    //   the `_resolved.toml` suffix is no longer used; the merge step
    //   writes directly to the canonical `lgo_<character>_gear.toml`.
    //   See `merge_into_canonical` and `resolve_stats_file`.

    // -- merge_into_canonical tests --

    /// Helper: a fresh resolved bookmarklet-style document with one item.
    fn make_doc(items: &[(&str, &str, &[(&str, i64)])]) -> String {
        let mut s = String::from("# LGO gear stats file\n");
        for (name, slot, stats) in items {
            s.push_str("\n[[item]]\n");
            s.push_str(&format!("slot = \"{}\"\n", slot));
            s.push_str(&format!("name = \"{}\"\n", name));
            for (k, v) in *stats {
                s.push_str(&format!("{} = {}\n", k, v));
            }
        }
        s
    }

    /// Scripted prompter for tests: returns canned answers in order.
    /// Panics if asked more questions than answers were supplied — the
    /// tests use this to assert that prompting was (or wasn't) invoked.
    pub struct ScriptedPrompter {
        answers: std::collections::VecDeque<(PromptCategory, PromptAnswer)>,
        pub asked: Vec<(PromptCategory, String)>,
    }

    impl ScriptedPrompter {
        pub fn new(answers: Vec<(PromptCategory, PromptAnswer)>) -> Self {
            Self {
                answers: answers.into(),
                asked: Vec::new(),
            }
        }
    }

    impl Prompter for ScriptedPrompter {
        fn prompt(&mut self, category: PromptCategory, item_name: &str) -> PromptAnswer {
            self.asked.push((category, item_name.to_string()));
            let (expected_cat, ans) = self
                .answers
                .pop_front()
                .unwrap_or_else(|| panic!("ScriptedPrompter ran out: asked {:?}", self.asked));
            assert_eq!(
                category, expected_cat,
                "prompt category mismatch for {}",
                item_name
            );
            ans
        }
    }

    fn force_with(answers: Vec<(PromptCategory, PromptAnswer)>) -> ForceMode {
        ForceMode::Force {
            prompter: Box::new(ScriptedPrompter::new(answers)),
        }
    }

    #[test]
    fn merge_first_run_takes_incoming_verbatim() {
        let incoming = make_doc(&[("Test Helm", "Head", &[("Armor", 100)])]);
        let outcome =
            merge_into_canonical(None, &incoming, ForceMode::NoForce).expect("must merge");
        assert_eq!(outcome.added, vec!["Test Helm"]);
        assert!(outcome.preserved.is_empty());
        assert!(outcome.removed.is_empty());
        assert_eq!(outcome.merged_text, incoming);
    }

    /// The cornerstone idempotency test: running the merge twice in a row
    /// with no new bookmarklet output must produce a bit-identical
    /// canonical file the second time.
    #[test]
    fn merge_idempotent_when_nothing_changes() {
        let db = fixture_db();
        let bookmarklet = make_doc(&[
            ("Test Helm", "Unknown", &[("Armor", 100)]),
            ("Test Bracelet", "Unknown", &[("Armor", 50)]),
        ]);
        let (resolved, _) = resolve_toml_str(&bookmarklet, &db).expect("resolve");
        let first = merge_into_canonical(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;
        let second = merge_into_canonical(Some(&first), &resolved, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        assert_eq!(
            first, second,
            "second merge must be byte-identical:\n--- first ---\n{}\n--- second ---\n{}",
            first, second
        );

        // And a third time, just to be sure dividers don't accumulate.
        let third = merge_into_canonical(Some(&second), &resolved, ForceMode::NoForce)
            .expect("third merge")
            .merged_text;
        assert_eq!(second, third, "third merge must also be byte-identical");
    }

    #[test]
    fn merge_adds_new_items_from_incoming() {
        let db = fixture_db();
        let prev_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let inc_in = make_doc(&[
            ("Test Helm", "Unknown", &[("Armor", 100)]),
            ("Test Sword", "Unknown", &[("CriticalRating", 50)]),
        ]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");
        let outcome =
            merge_into_canonical(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");
        assert_eq!(outcome.added, vec!["Test Sword"]);
        assert_eq!(outcome.preserved, vec!["Test Helm"]);
        assert!(outcome.removed.is_empty());
        assert!(outcome.merged_text.contains("Test Sword"));
        assert!(outcome.merged_text.contains("Test Helm"));
    }

    #[test]
    fn merge_removes_items_absent_from_incoming() {
        let db = fixture_db();
        let prev_in = make_doc(&[
            ("Test Helm", "Unknown", &[("Armor", 100)]),
            ("Test Sword", "Unknown", &[("CriticalRating", 50)]),
        ]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let inc_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        let outcome =
            merge_into_canonical(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");
        assert_eq!(outcome.removed, vec!["Test Sword"]);
        assert_eq!(outcome.preserved, vec!["Test Helm"]);
        assert!(!outcome.merged_text.contains("Test Sword"));
    }

    #[test]
    fn merge_preserves_previous_when_stats_differ_no_force() {
        let db = fixture_db();
        let prev_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 999)])]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let inc_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        let outcome =
            merge_into_canonical(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");
        assert_eq!(outcome.preserved, vec!["Test Helm"]);
        assert!(outcome.overwritten.is_empty());
        assert!(
            outcome.merged_text.contains("Armor = 999"),
            "previous value must be preserved:\n{}",
            outcome.merged_text
        );
    }

    #[test]
    fn merge_force_yes_overwrites() {
        let db = fixture_db();
        let prev_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 999)])]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let inc_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        let outcome = merge_into_canonical(
            Some(&prev),
            &incoming,
            force_with(vec![(PromptCategory::Overwrite, PromptAnswer::Yes)]),
        )
        .expect("must merge");
        assert_eq!(outcome.overwritten, vec!["Test Helm"]);
        assert!(outcome.merged_text.contains("Armor = 100"));
        assert!(!outcome.merged_text.contains("Armor = 999"));
    }

    #[test]
    fn merge_force_no_keeps_previous() {
        let db = fixture_db();
        let prev_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 999)])]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let inc_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        let outcome = merge_into_canonical(
            Some(&prev),
            &incoming,
            force_with(vec![(PromptCategory::Overwrite, PromptAnswer::No)]),
        )
        .expect("must merge");
        assert_eq!(outcome.preserved, vec!["Test Helm"]);
        assert!(outcome.overwritten.is_empty());
        assert!(outcome.merged_text.contains("Armor = 999"));
    }

    #[test]
    fn merge_force_yes_to_all_overwrite_skips_subsequent_overwrite_prompts() {
        let db = fixture_db();
        let inc_in = make_doc(&[
            ("Test Helm", "Unknown", &[("Armor", 100)]),
            ("Test Bracelet", "Unknown", &[("Armor", 5)]),
            // also a removal candidate, to verify YesToAll on overwrite
            // does NOT auto-accept removals.
        ]);
        let prev_in = make_doc(&[
            ("Test Helm", "Unknown", &[("Armor", 999)]),
            ("Test Bracelet", "Unknown", &[("Armor", 50)]),
            ("Test Sword", "Unknown", &[("CriticalRating", 1)]),
        ]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        // Expect: first overwrite → 'a'; second overwrite never asked;
        // removal of "Test Sword" → still asked (independent).
        let outcome = merge_into_canonical(
            Some(&prev),
            &incoming,
            force_with(vec![
                (PromptCategory::Overwrite, PromptAnswer::YesToAll),
                (PromptCategory::Remove, PromptAnswer::Yes),
            ]),
        )
        .expect("must merge");

        assert_eq!(outcome.overwritten.len(), 2);
        assert!(outcome.overwritten.contains(&"Test Helm".to_string()));
        assert!(outcome.overwritten.contains(&"Test Bracelet".to_string()));
        assert_eq!(outcome.removed, vec!["Test Sword"]);
    }

    #[test]
    fn merge_force_identical_data_never_prompts() {
        let db = fixture_db();
        let prev_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let inc_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        // Empty answer queue: any prompt would panic.
        let outcome = merge_into_canonical(Some(&prev), &incoming, force_with(vec![]))
            .expect("must merge with no prompts");
        assert_eq!(outcome.preserved, vec!["Test Helm"]);
        assert!(outcome.overwritten.is_empty());
    }

    #[test]
    fn merge_force_remove_yes_drops_item() {
        let db = fixture_db();
        let prev_in = make_doc(&[("Test Sword", "Unknown", &[("CriticalRating", 50)])]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let inc_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        let outcome = merge_into_canonical(
            Some(&prev),
            &incoming,
            force_with(vec![(PromptCategory::Remove, PromptAnswer::Yes)]),
        )
        .expect("must merge");
        assert_eq!(outcome.removed, vec!["Test Sword"]);
        assert!(!outcome.merged_text.contains("Test Sword"));
    }

    #[test]
    fn merge_force_remove_no_retains_item() {
        let db = fixture_db();
        let prev_in = make_doc(&[("Test Sword", "Unknown", &[("CriticalRating", 50)])]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let inc_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        let outcome = merge_into_canonical(
            Some(&prev),
            &incoming,
            force_with(vec![(PromptCategory::Remove, PromptAnswer::No)]),
        )
        .expect("must merge");
        assert!(outcome.removed.is_empty());
        assert!(outcome.preserved.contains(&"Test Sword".to_string()));
        assert!(outcome.merged_text.contains("Test Sword"));
    }

    #[test]
    fn merge_unknown_slot_names_reported() {
        let db = fixture_db();
        // "Mystery Renamed Legendary" is not in the fixture DB, so it stays
        // with slot = "Unknown" after resolve_toml_str.
        let inc_in = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Mystery Renamed Legendary\"\n\
";
        let (incoming, _) = resolve_toml_str(inc_in, &db).expect("resolve incoming");
        let outcome = merge_into_canonical(None, &incoming, ForceMode::NoForce).expect("merge");
        assert_eq!(outcome.unknown_slot, vec!["Mystery Renamed Legendary"]);
    }

    #[test]
    fn merge_preserves_per_item_comments_across_iterations() {
        let db = fixture_db();
        // Hand-edited canonical: warning-style comment inside the [[item]].
        let prev = "\
[[item]]\n\
slot = \"Head\"\n\
name = \"Test Helm\"\n\
# essence: +1500 tactical mastery\n\
Armor = 100\n";
        let inc_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        let outcome =
            merge_into_canonical(Some(prev), &incoming, ForceMode::NoForce).expect("must merge");
        assert!(
            outcome
                .merged_text
                .contains("# essence: +1500 tactical mastery"),
            "hand-written comment must survive merge:\n{}",
            outcome.merged_text
        );
    }

    // -- metadata (character / class) preservation --

    /// Helper: build a bookmarklet-style TOML string with character and class
    /// metadata fields at the top level.
    fn make_doc_with_meta(
        character: &str,
        class: &str,
        items: &[(&str, &str, &[(&str, i64)])],
    ) -> String {
        let mut s = String::from("# LGO gear stats file — generated by bookmarklet\n");
        s.push_str(&format!("character          = \"{}\"\n", character));
        s.push_str(&format!("class              = \"{}\"\n", class));
        for (name, slot, stats) in items {
            s.push_str("\n[[item]]\n");
            s.push_str(&format!("slot = \"{}\"\n", slot));
            s.push_str(&format!("name = \"{}\"\n", name));
            for (k, v) in *stats {
                s.push_str(&format!("{} = {}\n", k, v));
            }
        }
        s
    }

    #[test]
    fn resolve_toml_str_preserves_character_and_class() {
        let db = fixture_db();
        let input = make_doc_with_meta(
            "Thalya",
            "Lore-master",
            &[("Test Helm", "Unknown", &[("Armor", 100)])],
        );
        let (out, _) = resolve_toml_str(&input, &db).expect("must resolve");
        assert!(
            out.contains("character"),
            "character field must survive resolve_toml_str:\n{}",
            out
        );
        assert!(
            out.contains("Thalya"),
            "character value must survive resolve_toml_str:\n{}",
            out
        );
        assert!(
            out.contains("class"),
            "class field must survive resolve_toml_str:\n{}",
            out
        );
        assert!(
            out.contains("Lore-master"),
            "class value must survive resolve_toml_str:\n{}",
            out
        );
    }

    #[test]
    fn merge_first_run_preserves_character_and_class() {
        let db = fixture_db();
        let bookmarklet = make_doc_with_meta(
            "Thalya",
            "Lore-master",
            &[("Test Helm", "Unknown", &[("Armor", 100)])],
        );
        let (resolved, _) = resolve_toml_str(&bookmarklet, &db).expect("resolve");
        let outcome = merge_into_canonical(None, &resolved, ForceMode::NoForce).expect("merge");
        assert!(
            outcome.merged_text.contains("Thalya"),
            "character must be in first-run canonical output:\n{}",
            outcome.merged_text
        );
        assert!(
            outcome.merged_text.contains("Lore-master"),
            "class must be in first-run canonical output:\n{}",
            outcome.merged_text
        );
    }

    #[test]
    fn merge_subsequent_run_carries_character_and_class_into_canonical() {
        let db = fixture_db();
        let bookmarklet = make_doc_with_meta(
            "Thalya",
            "Lore-master",
            &[("Test Helm", "Unknown", &[("Armor", 100)])],
        );
        let (resolved, _) = resolve_toml_str(&bookmarklet, &db).expect("resolve");

        // First run — creates canonical.
        let first = merge_into_canonical(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;

        // Second run — merge back in.
        let second = merge_into_canonical(Some(&first), &resolved, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;

        assert!(
            second.contains("Thalya"),
            "character must survive repeat merge:\n{}",
            second
        );
        assert!(
            second.contains("Lore-master"),
            "class must survive repeat merge:\n{}",
            second
        );
    }

    #[test]
    fn merge_updates_metadata_in_pre_existing_canonical_without_metadata() {
        // Simulate a canonical file that predates the character/class feature
        // (no metadata fields). The new incoming TOML carries metadata; the
        // merge must copy it into the canonical output.
        let db = fixture_db();
        let prev = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (prev_resolved, _) = resolve_toml_str(&prev, &db).expect("resolve prev");

        let incoming = make_doc_with_meta(
            "Thalya",
            "Lore-master",
            &[("Test Helm", "Unknown", &[("Armor", 100)])],
        );
        let (resolved, _) = resolve_toml_str(&incoming, &db).expect("resolve incoming");

        let outcome = merge_into_canonical(Some(&prev_resolved), &resolved, ForceMode::NoForce)
            .expect("must merge");
        assert!(
            outcome.merged_text.contains("Thalya"),
            "character must be injected into pre-existing canonical:\n{}",
            outcome.merged_text
        );
        assert!(
            outcome.merged_text.contains("Lore-master"),
            "class must be injected into pre-existing canonical:\n{}",
            outcome.merged_text
        );
    }

    // -- duplicate same-name item tests --

    #[test]
    fn merge_first_run_preserves_both_duplicate_instances() {
        // Two incoming items with the same name (two owned copies).
        // First run: both must appear in `added` and in the merged output.
        let incoming = make_doc(&[
            ("Twin Ring", "Finger (1)", &[("CriticalRating", 500)]),
            ("Twin Ring", "Finger (1)", &[("CriticalRating", 500)]),
        ]);
        let outcome =
            merge_into_canonical(None, &incoming, ForceMode::NoForce).expect("must merge");
        assert_eq!(
            outcome.added.iter().filter(|n| *n == "Twin Ring").count(),
            2,
            "both instances must appear in `added`"
        );
        assert_eq!(
            outcome.merged_text.matches("Twin Ring").count(),
            2,
            "both instances must be in merged output"
        );
    }

    #[test]
    fn merge_idempotent_with_duplicate_instances() {
        // Two copies of the same item survive repeated merges without
        // collapsing to one.
        let db = fixture_db();
        let bookmarklet = make_doc(&[
            ("Twin Ring", "Unknown", &[("CriticalRating", 500)]),
            ("Twin Ring", "Unknown", &[("CriticalRating", 500)]),
        ]);
        let (resolved, _) = resolve_toml_str(&bookmarklet, &db).expect("resolve");
        let first = merge_into_canonical(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;
        let second = merge_into_canonical(Some(&first), &resolved, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        assert_eq!(
            first, second,
            "second merge must be bit-identical:\n--- first ---\n{}\n--- second ---\n{}",
            first, second
        );
        // Both copies must still be present.
        assert_eq!(second.matches("Twin Ring").count(), 2);
    }

    #[test]
    fn merge_duplicate_prev_loses_one_when_incoming_has_one() {
        // Previous has two copies of "Twin Ring"; new export has only one.
        // One copy must be preserved, one removed.
        let db = fixture_db();
        let two_copies = make_doc(&[
            ("Twin Ring", "Unknown", &[("CriticalRating", 500)]),
            ("Twin Ring", "Unknown", &[("CriticalRating", 500)]),
        ]);
        let (prev, _) = resolve_toml_str(&two_copies, &db).expect("resolve prev");
        let prev = merge_into_canonical(None, &prev, ForceMode::NoForce)
            .expect("first run")
            .merged_text;

        let one_copy = make_doc(&[("Twin Ring", "Unknown", &[("CriticalRating", 500)])]);
        let (incoming, _) = resolve_toml_str(&one_copy, &db).expect("resolve incoming");

        let outcome =
            merge_into_canonical(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");

        assert_eq!(
            outcome.preserved.iter().filter(|n| *n == "Twin Ring").count(),
            1,
            "one copy must be preserved"
        );
        assert_eq!(
            outcome.removed.iter().filter(|n| *n == "Twin Ring").count(),
            1,
            "one copy must be removed"
        );
        assert_eq!(
            outcome.merged_text.matches("Twin Ring").count(),
            1,
            "merged output must contain exactly one copy"
        );
    }

    #[test]
    fn merge_duplicate_incoming_gains_one_when_prev_has_one() {
        // Previous has one copy of "Twin Ring"; new export has two.
        // One copy must be preserved, one added.
        let db = fixture_db();
        let one_copy = make_doc(&[("Twin Ring", "Unknown", &[("CriticalRating", 500)])]);
        let (prev, _) = resolve_toml_str(&one_copy, &db).expect("resolve prev");
        let prev = merge_into_canonical(None, &prev, ForceMode::NoForce)
            .expect("first run")
            .merged_text;

        let two_copies = make_doc(&[
            ("Twin Ring", "Unknown", &[("CriticalRating", 500)]),
            ("Twin Ring", "Unknown", &[("CriticalRating", 500)]),
        ]);
        let (incoming, _) = resolve_toml_str(&two_copies, &db).expect("resolve incoming");

        let outcome =
            merge_into_canonical(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");

        assert_eq!(
            outcome.preserved.iter().filter(|n| *n == "Twin Ring").count(),
            1,
            "one copy must be preserved"
        );
        assert_eq!(
            outcome.added.iter().filter(|n| *n == "Twin Ring").count(),
            1,
            "one copy must be added"
        );
        assert_eq!(
            outcome.merged_text.matches("Twin Ring").count(),
            2,
            "merged output must contain two copies"
        );
    }

    #[test]
    fn merge_duplicate_exact_equal_never_prompts_force() {
        // Two identical copies in both prev and incoming → exact-match
        // Phase 1 should pair them without any force prompt.
        let db = fixture_db();
        let two_copies = make_doc(&[
            ("Twin Ring", "Unknown", &[("CriticalRating", 500)]),
            ("Twin Ring", "Unknown", &[("CriticalRating", 500)]),
        ]);
        let (src, _) = resolve_toml_str(&two_copies, &db).expect("resolve");
        let prev = merge_into_canonical(None, &src, ForceMode::NoForce)
            .expect("first run")
            .merged_text;

        // Empty answer queue: any prompt would panic.
        let outcome = merge_into_canonical(Some(&prev), &src, force_with(vec![]))
            .expect("must merge with no prompts");
        assert_eq!(
            outcome.preserved.iter().filter(|n| *n == "Twin Ring").count(),
            2
        );
        assert!(outcome.overwritten.is_empty());
    }
}
