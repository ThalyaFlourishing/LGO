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

use chrono::Local;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use toml_edit::{value, ArrayOfTables, Decor, DocumentMut, Item, Table};
use unicode_normalization::{is_nfc, UnicodeNormalization};

use crate::base_stats::DerivationError;
use crate::gear::{parse_slot_display, Slot};
use crate::stat::{Stat, BASE_STATS, TRACKED_STATS};
use crate::virtues::{VIRTUE_FIELD_KEYS, VIRTUE_TABLE_KEY};

/// Default path to the offline items DB, relative to the working directory.
pub const DEFAULT_ITEMS_DB_PATH: &str = "data/lgo_items.json";
const ESSENCE_TOTALS_KEY: &str = "EssenceTotals";
const STAT_EQUALS_COLUMN: usize = 20;
const STAT_KEY_PAD_WIDTH: usize = STAT_EQUALS_COLUMN - 1;
pub const UNRESOLVED_COMMENT_PREFIX: &str = "# UNRESOLVED:";
pub const AUTO_PICKED_COMMENT_PREFIX: &str = "# AUTO-PICKED ";
const INNATE_STATS_NOTE: &str = "# Extracted by in-game plugin; do not edit.";
const VIRTUES_NOTE: &str = "# Not extracted, you must add these yourself.";

// =============================================================================
// ItemsDb — name → Slot index
// =============================================================================

/// In-memory name → canonical Slot index, built once from
/// `data/lgo_items.json` at startup.
///
/// Lookups are O(1). The internal `Vec<DbItem>` per name is a deliberate
/// shape choice (deliberate future-proofing) leaving room for future
/// disambiguation by tier / item-level / quality if that ever becomes
/// necessary; today, JSON object keys are unique by construction so each
/// Vec has length 1.
#[derive(Debug)]
pub struct ItemsDb {
    by_name: HashMap<String, Vec<DbItem>>,
}

/// A single resolved entry: an item name, its canonical equipment Slot, and
/// whether it is a two-handed `MainHand` weapon (occupies both hand slots).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbItem {
    pub name: String,
    pub slot: Slot,
    pub two_handed: bool,
    pub either_hand: bool,
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
    /// A DB entry's `slot` field is not a recognised canonical display-form
    /// Slot string. Indicates the JSON file's schema has drifted from what
    /// `gear.rs` knows about.
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
/// The DB carries only `name`, `slot`, and (for two-handed weapons)
/// `two_handed` — stats come from the bookmarklet, not the DB. Serde
/// silently ignores unknown fields by default, so old DBs that still
/// carry a `stats` object are tolerated without ceremony.
#[derive(Debug, Deserialize)]
struct RawEntry {
    name: String,
    slot: String,
    /// Absent in DBs built before two-handed support; defaults to false.
    #[serde(default)]
    two_handed: bool,
    /// Absent in DBs built before either-hand support; defaults to false.
    #[serde(default)]
    either_hand: bool,
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
        // Top-level shape: { "<item name>": { name, slot[, two_handed] }, ... }
        let raw: HashMap<String, RawEntry> =
            serde_json::from_str(json).map_err(|e| ItemsDbError::ParseJson {
                path: path_for_errors.to_path_buf(),
                source: e,
            })?;

        let mut by_name: HashMap<String, Vec<DbItem>> = HashMap::new();
        for (_key, entry) in raw {
            // Trust the inner `name` field; the outer key is identical in the
            // real file but the inner field is what `db_build` was guaranteed
            // to write and is thus the authoritative copy. Index keys are
            // NFC-normalized so lookups match regardless of which Unicode
            // normalization form either side carries.
            let slot =
                parse_slot_display(&entry.slot).ok_or_else(|| ItemsDbError::UnknownSlot {
                    item_name: entry.name.clone(),
                    slot_string: entry.slot.clone(),
                })?;
            by_name
                .entry(nfc_name(&entry.name).into_owned())
                .or_default()
                .push(DbItem {
                    name: entry.name,
                    slot,
                    two_handed: entry.two_handed,
                    either_hand: entry.either_hand,
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
    /// Lookups are NFC-normalized (both sides), so Unicode-equivalent byte
    /// sequences resolve to the same entry. Resolution policy is "first match
    /// wins". In practice every Vec has length 1 because JSON object keys are
    /// unique; the Vec is structural future-proofing only.
    pub fn lookup(&self, name: &str) -> Option<Slot> {
        self.by_name
            .get(nfc_name(name).as_ref())
            .and_then(|v| v.first())
            .map(|item| item.slot)
    }

    /// True when the named item is a two-handed `MainHand` weapon per the DB.
    /// Unknown names return false — the resolver preserves any user-provided
    /// `two_handed` flag for those instead of consulting the DB.
    pub fn lookup_two_handed(&self, name: &str) -> bool {
        self.by_name
            .get(nfc_name(name).as_ref())
            .and_then(|v| v.first())
            .is_some_and(|item| item.two_handed)
    }

    /// True when the named item is an Either-hand item per the DB. Unknown
    /// names return false — the resolver preserves any user-provided
    /// `either_hand` flag for those instead of consulting the DB.
    pub fn lookup_either_hand(&self, name: &str) -> bool {
        self.by_name
            .get(nfc_name(name).as_ref())
            .and_then(|v| v.first())
            .is_some_and(|item| item.either_hand)
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
        source: Box<toml_edit::TomlError>,
    },
    NoItemsArray {
        path: PathBuf,
    },
    /// Neither `lgo_<character>_gearStats.toml` nor `lgo_<character>_gearReady.toml`
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
    Derivation {
        source: DerivationError,
    },
    PluginData {
        path: PathBuf,
        message: String,
    },
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
                "No lgo_{}_gearStats.toml or lgo_{}_gearReady.toml found in {}",
                character,
                character,
                dir.display()
            ),
            ResolveError::AmbiguousFiles { message } => write!(f, "Error: {}", message),
            ResolveError::ForceRequiresTty => {
                write!(f, "--force requires interactive stdin for prompts.")
            }
            ResolveError::Derivation { source } => write!(f, "{}", source),
            ResolveError::PluginData { path, message } => {
                write!(
                    f,
                    "Cannot read plugin export '{}': {}",
                    path.display(),
                    message
                )
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
    slot_family(family).display_name()
}

/// Order in which slot families should appear in the output, matching the
/// canonical Slot::all() traversal (paired slots collapsed to their first
/// representative).
fn slot_family_order() -> Vec<Slot> {
    let mut seen: Vec<Slot> = Vec::new();
    for slot in Slot::all() {
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
    resolve_toml_str_inner(src, db, None)
}

pub fn resolve_toml_str_with_metadata(
    src: &str,
    db: &ItemsDb,
    character: Option<&str>,
    class_name: &str,
    base_stats: &HashMap<Stat, i64>,
) -> Result<(String, Vec<ResolutionOutcome>), ResolveError> {
    let metadata = ResolveMetadata {
        character,
        class_name,
        base_stats,
    };
    resolve_toml_str_inner(src, db, Some(&metadata))
}

struct ResolveMetadata<'a> {
    character: Option<&'a str>,
    class_name: &'a str,
    base_stats: &'a HashMap<Stat, i64>,
}

fn resolve_toml_str_inner(
    src: &str,
    db: &ItemsDb,
    metadata: Option<&ResolveMetadata<'_>>,
) -> Result<(String, Vec<ResolutionOutcome>), ResolveError> {
    let mut doc: DocumentMut = src.parse().map_err(|e| ResolveError::ParseToml {
        path: PathBuf::from("<in-memory>"),
        source: Box::new(e),
    })?;

    if let Some(metadata) = metadata {
        apply_top_level_metadata(&mut doc, metadata);
    }

    let mut original_tables: Vec<Table> = {
        let items_arr = doc
            .get_mut("item")
            .and_then(|i| i.as_array_of_tables_mut())
            .ok_or_else(|| ResolveError::NoItemsArray {
                path: PathBuf::from("<in-memory>"),
            })?;

        // Take ownership of the existing entries by replacing the array with
        // an empty one, then rebuild from scratch in canonical order.
        let taken = std::mem::replace(items_arr, ArrayOfTables::new());
        taken.iter().cloned().collect()
    };

    // Rescue trailing comments written after the last essence key before
    // canonicalization rebuilds the essence blocks and regrouping rewrites
    // next-table prefix decor.
    capture_essence_trailing_comments(&mut original_tables, &mut doc);

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

    // Re-acquire the (drained, still present) array to install the rebuilt
    // set. The borrow was released above so `doc` could be threaded through
    // capture_essence_trailing_comments; every step in between is infallible,
    // and `doc` is local, so no partially-drained document can escape.
    let items_arr = doc
        .get_mut("item")
        .and_then(|i| i.as_array_of_tables_mut())
        .ok_or_else(|| ResolveError::NoItemsArray {
            path: PathBuf::from("<in-memory>"),
        })?;
    *items_arr = new_arr;
    reorder_resolved_header_before_items(&mut doc);
    Ok((doc.to_string(), outcomes_local))
}

/// Write `character`, `class`, `[InnateStats]`, and `[Virtues]` into the
/// document header.
///
/// `[InnateStats]` carries the character's five *raw* Base stats from the
/// plugindata export — no derivation happens in the resolver. `[Virtues]`
/// carries the user's five fixed Virtue selection slots as empty strings for
/// later hand-editing. Derivation into tracked stats is the optimize path's
/// job; see the pass-through design in the repo docs.
fn apply_top_level_metadata(doc: &mut DocumentMut, metadata: &ResolveMetadata<'_>) {
    if let Some(character) = metadata.character {
        doc.insert("character", value(character));
    }
    doc.insert("class", value(metadata.class_name));

    doc.insert(
        "InnateStats",
        toml_edit::Item::Table(build_innate_stats_table(metadata.base_stats)),
    );
    doc.insert(VIRTUE_TABLE_KEY, Item::Table(build_virtues_table()));
}

fn reorder_resolved_header_before_items(doc: &mut DocumentMut) {
    let mut next_position = 0;
    if let Some(innate) = doc.get_mut("InnateStats").and_then(|i| i.as_table_mut()) {
        innate.set_position(next_position);
        innate.decor_mut().set_prefix("\n");
        next_position += 1;
    }
    if let Some(virtues) = doc.get_mut(VIRTUE_TABLE_KEY).and_then(|i| i.as_table_mut()) {
        virtues.set_position(next_position);
        virtues.decor_mut().set_prefix("\n");
        next_position += 1;
    }

    if let Some(items) = doc.get_mut("item").and_then(|i| i.as_array_of_tables_mut()) {
        for (offset, table) in items.iter_mut().enumerate() {
            table.set_position(offset + next_position);
        }
    }
}

fn build_virtues_table() -> Table {
    let mut table = Table::new();
    ensure_virtue_fields(&mut table);
    table
}

fn build_innate_stats_table(base_stats: &HashMap<Stat, i64>) -> Table {
    let mut table = Table::new();
    ensure_table_header_note(&mut table, INNATE_STATS_NOTE);
    for (stat, key) in BASE_STATS {
        table.insert(key, value(base_stats.get(stat).copied().unwrap_or(0)));
        normalize_assignment_decor(&mut table, key);
    }
    table
}

fn read_innate_base_stats(table: &Table) -> HashMap<Stat, i64> {
    let mut stats = HashMap::new();
    for (stat, key) in BASE_STATS {
        if let Some(value) = table.get(key).and_then(|item| item.as_integer()) {
            if value != 0 {
                stats.insert(*stat, value);
            }
        }
    }
    stats
}

fn ensure_virtue_fields(table: &mut Table) {
    ensure_table_header_note(table, VIRTUES_NOTE);
    for key in VIRTUE_FIELD_KEYS {
        if table.get(key).is_none() {
            table.insert(key, value(""));
        }
        normalize_assignment_decor(table, key);
    }
}

fn ensure_table_header_note(table: &mut Table, note: &str) {
    let existing_suffix = table
        .decor()
        .suffix()
        .and_then(|suffix| suffix.as_str())
        .unwrap_or("");
    let mut kept_lines: Vec<&str> = Vec::new();
    for line in existing_suffix.trim_start_matches('\n').lines() {
        if line != note {
            kept_lines.push(line);
        }
    }

    let mut suffix = format!("\n{note}\n");
    for line in kept_lines {
        suffix.push_str(line);
        suffix.push('\n');
    }
    table.decor_mut().set_suffix(suffix);
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

        let db_slot = db.lookup(&name);
        // Refresh generated two-handed metadata before canonicalization so
        // the flag lands after `name` and before the re-inserted stat block
        // (canonicalization removes all stat keys and re-appends them).
        // Unknown items are left untouched: a user-provided `two_handed`
        // flag on a hand-edited legendary/renamed item is preserved.
        let outcome_comments = take_outcome_comments_from_item(&mut table);
        if db_slot.is_some() {
            set_two_handed_flag(&mut table, db.lookup_two_handed(&name));
            set_either_hand_flag(&mut table, db.lookup_either_hand(&name));
        }

        canonicalize_item_stats(&mut table, &outcome_comments);
        match db_slot {
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

/// Enforce the DB-derived `two_handed` flag on an item table.
///
/// Generated metadata, not a user stat: the DB is the source of truth for
/// items it knows, so any existing value (stale, hand-edited, or non-bool)
/// is discarded. The key is re-inserted only when true — `gearReady.toml`
/// omits the flag for one-handed items and "missing means false".
///
/// Call this *before* stat canonicalization: canonicalization strips and
/// re-appends every stat key, which is what places `two_handed` after
/// `name` and before the stat block in the rendered output.
fn set_two_handed_flag(table: &mut Table, two_handed: bool) {
    table.remove("two_handed");
    if two_handed {
        table.insert("two_handed", value(true));
    }
}

/// Enforce the DB-derived `either_hand` flag on an item table.
///
/// Generated metadata handled exactly like `two_handed`: the DB is the source
/// of truth for items it knows, so any existing value is discarded and the key
/// is re-inserted only when true. `gearReady.toml` omits the flag otherwise
/// and "missing means false". Call this *before* stat canonicalization so the
/// flag lands after `name` and before the re-appended stat block.
fn set_either_hand_flag(table: &mut Table, either_hand: bool) {
    table.remove("either_hand");
    if either_hand {
        table.insert("either_hand", value(true));
    }
}

/// The canonical per-item stat key layout: the 16 tracked stats in canonical
/// order, then the five raw Base stats (Might, Agility, Vitality, Will,
/// Fate). Base-stat lines pass through the resolver verbatim — they are never
/// derived into (or added to) tracked totals here.
fn canonical_stat_entries() -> impl Iterator<Item = &'static (Stat, &'static str)> {
    TRACKED_STATS.iter().chain(BASE_STATS.iter())
}

fn assignment_key_spacing(key: &str) -> String {
    if key.len() >= STAT_KEY_PAD_WIDTH {
        " ".to_string()
    } else {
        " ".repeat(STAT_KEY_PAD_WIDTH - key.len())
    }
}

fn normalize_assignment_decor(table: &mut Table, key: &str) {
    if let Some((mut key_mut, item)) = table.get_key_value_mut(key) {
        key_mut
            .leaf_decor_mut()
            .set_suffix(assignment_key_spacing(key));
        if let Some(value) = item.as_value_mut() {
            value.decor_mut().set_prefix(" ");
        }
    }
}

fn normalize_existing_canonical_stat_decor(table: &mut Table) {
    for (_, key) in canonical_stat_entries() {
        normalize_assignment_decor(table, key);
    }
}

/// Rewrite an item's stat block into canonical shape: all 16 tracked stats
/// followed by the five raw Base stats (zeros for omissions), then a fully
/// populated `[item.EssenceTotals]` child table with the same key layout.
/// Existing values — tracked and Base alike — pass through unchanged.
fn canonicalize_item_stats(table: &mut Table, outcome_comments: &str) {
    let explicit = read_item_stats(table);
    let essence = read_essence_stats(table);
    let essence_decor = read_essence_decor(table);
    // Capture per-key decor from the existing essence block before it is
    // dropped, so comments the user wrote inside [item.EssenceTotals]
    // survive re-canonicalization exactly like base-block comments do.
    let old_essence_items = table
        .get_mut(ESSENCE_TOTALS_KEY)
        .and_then(|essence_item| essence_item.as_table_mut())
        .map(remove_canonical_stat_items)
        .unwrap_or_default();
    let old_items = remove_canonical_stat_items(table);
    table.remove(ESSENCE_TOTALS_KEY);
    insert_canonical_stats(table, &explicit, &old_items);
    attach_outcome_comments_to_header(table, outcome_comments);
    insert_essence_totals(table, &essence, essence_decor, &old_essence_items);
}

/// Drain bookmarklet outcome comments from every place older outputs may have
/// stored them, then reattach them only to item-header decor. This prevents
/// comments parsed as stat-key prefixes from drifting with the stat block on
/// repeated canonicalization.
fn take_outcome_comments_from_item(table: &mut Table) -> String {
    let mut header_comments = take_outcome_comments_from_header_decor(table);
    header_comments.push_str(&take_outcome_comments_from_stat_prefixes(table));
    header_comments
}

fn take_outcome_comments_from_header_decor(table: &mut Table) -> String {
    let mut header_comments = String::new();
    for key in ["slot", "name", "two_handed", "either_hand"] {
        if let Some((mut key_mut, _)) = table.get_key_value_mut(key) {
            let prefix = key_mut
                .leaf_decor()
                .prefix()
                .and_then(|prefix| prefix.as_str())
                .unwrap_or("");
            let (outcome_comments, remaining_prefix) = split_outcome_comments(prefix);
            if !outcome_comments.is_empty() {
                header_comments.push_str(&outcome_comments);
                let remaining_prefix = if remaining_prefix.trim().is_empty() {
                    String::new()
                } else {
                    remaining_prefix
                };
                key_mut.leaf_decor_mut().set_prefix(remaining_prefix);
            }
        }

        let Some(value) = table.get_mut(key).and_then(Item::as_value_mut) else {
            continue;
        };
        let suffix = value
            .decor()
            .suffix()
            .and_then(|suffix| suffix.as_str())
            .unwrap_or("");
        if suffix.is_empty() {
            continue;
        }

        let (outcome_comments, remaining_suffix) = split_outcome_comments(suffix);
        if !outcome_comments.is_empty() {
            header_comments.push_str(&outcome_comments);
            value.decor_mut().set_suffix(remaining_suffix);
        }
    }
    header_comments
}

fn take_outcome_comments_from_stat_prefixes(table: &mut Table) -> String {
    let mut outcome_comments_from_stats = String::new();
    for (_, key) in canonical_stat_entries() {
        let Some((mut key_mut, _)) = table.get_key_value_mut(key) else {
            continue;
        };
        let prefix = key_mut
            .leaf_decor()
            .prefix()
            .and_then(|prefix| prefix.as_str())
            .unwrap_or("");
        if prefix.is_empty() {
            continue;
        }

        let (outcome_comments, remaining_prefix) = split_outcome_comments(prefix);
        if !outcome_comments.is_empty() {
            outcome_comments_from_stats.push_str(&outcome_comments);
            let remaining_prefix = if remaining_prefix.trim().is_empty() {
                String::new()
            } else {
                remaining_prefix
            };
            key_mut.leaf_decor_mut().set_prefix(remaining_prefix);
        }
    }
    outcome_comments_from_stats
}

fn split_outcome_comments(prefix: &str) -> (String, String) {
    let mut outcome_comments = String::new();
    let mut remaining_prefix = String::new();

    for line in prefix.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with(UNRESOLVED_COMMENT_PREFIX)
            || trimmed.starts_with(AUTO_PICKED_COMMENT_PREFIX)
        {
            outcome_comments.push_str(trimmed);
            if !outcome_comments.ends_with('\n') {
                outcome_comments.push('\n');
            }
        } else {
            remaining_prefix.push_str(line);
        }
    }

    (outcome_comments, remaining_prefix)
}

fn attach_outcome_comments_to_header(table: &mut Table, outcome_comments: &str) {
    if outcome_comments.is_empty() {
        return;
    }
    let target_key = ["two_handed", "either_hand", "name", "slot"]
        .into_iter()
        .find(|key| table.contains_key(key));
    let Some(target_key) = target_key else {
        return;
    };
    if let Some((mut key_mut, _)) = table.get_key_value_mut(target_key) {
        let existing_prefix = key_mut
            .leaf_decor()
            .prefix()
            .and_then(|prefix| prefix.as_str())
            .unwrap_or("")
            .to_string();
        key_mut
            .leaf_decor_mut()
            .set_prefix(format!("{}{}", outcome_comments, existing_prefix));
    }
}

#[derive(Clone)]
struct RemovedStatItem {
    key_decor: toml_edit::Decor,
    item: Item,
}

fn remove_canonical_stat_items(table: &mut Table) -> HashMap<&'static str, RemovedStatItem> {
    let mut removed = HashMap::new();
    for (_, key) in canonical_stat_entries() {
        let key_decor = table
            .get_key_value(key)
            .map(|(key, _)| key.leaf_decor().clone())
            .unwrap_or_default();
        if let Some(item) = table.remove(key) {
            removed.insert(*key, RemovedStatItem { key_decor, item });
        }
    }
    removed
}

fn insert_canonical_stats(
    table: &mut Table,
    stats: &HashMap<Stat, i64>,
    old_items: &HashMap<&'static str, RemovedStatItem>,
) {
    for (stat, key) in canonical_stat_entries() {
        let mut item = value(stats.get(stat).copied().unwrap_or(0));
        if let (Some(old_value), Some(new_value)) = (
            old_items
                .get(key)
                .and_then(|removed| removed.item.as_value()),
            item.as_value_mut(),
        ) {
            new_value.decor_mut().set_suffix(
                old_value
                    .decor()
                    .suffix()
                    .map_or("", |s| s.as_str().unwrap_or("")),
            );
        }
        table.insert(key, item);
        if let Some(removed) = old_items.get(key) {
            if let Some((mut key_mut, _)) = table.get_key_value_mut(key) {
                key_mut.leaf_decor_mut().set_prefix(
                    removed
                        .key_decor
                        .prefix()
                        .map_or("", |s| s.as_str().unwrap_or("")),
                );
            }
        }
        normalize_assignment_decor(table, key);
    }
}

fn insert_essence_totals(
    table: &mut Table,
    essence: &HashMap<Stat, i64>,
    previous_decor: Option<Decor>,
    old_items: &HashMap<&'static str, RemovedStatItem>,
) {
    let mut essence_table = Table::new();
    if let Some(decor) = previous_decor {
        *essence_table.decor_mut() = decor;
    }
    // toml_edit's default standard-table prefix is a blank line. This child
    // table is intentionally attached to its parent item, so set table decor
    // structurally instead of post-processing the rendered TOML text.
    attach_table_to_previous_line(&mut essence_table);
    // Re-insert via the shared base-block path so per-key prefix decor
    // (comments above essence keys) and value-suffix decor round-trip.
    insert_canonical_stats(&mut essence_table, essence, old_items);
    // Re-anchor captured trailing comments to the rebuilt block's last line.
    hoist_essence_trailing_comments_to_last_line(&mut essence_table);
    table.insert(ESSENCE_TOTALS_KEY, Item::Table(essence_table));
}

fn attach_table_to_previous_line(table: &mut Table) {
    let prefix = table
        .decor()
        .prefix()
        .and_then(|prefix| prefix.as_str())
        .map(strip_leading_blank_lines)
        .unwrap_or("")
        .to_string();
    table.decor_mut().set_prefix(prefix);
}

fn strip_leading_blank_lines(mut prefix: &str) -> &str {
    loop {
        let Some(newline) = prefix.find('\n') else {
            return prefix;
        };
        let line = &prefix[..newline];
        if !line.trim().is_empty() {
            return prefix;
        }
        prefix = &prefix[newline + 1..];
    }
}

fn read_essence_stats(table: &Table) -> HashMap<Stat, i64> {
    table
        .get(ESSENCE_TOTALS_KEY)
        .and_then(|essence_item| essence_item.as_table())
        .map(read_item_stats)
        .unwrap_or_default()
}

fn read_essence_decor(table: &Table) -> Option<Decor> {
    table
        .get(ESSENCE_TOTALS_KEY)
        .and_then(|essence_item| essence_item.as_table())
        .map(|essence_table| essence_table.decor().clone())
}

/// True if a decor line is a generated family divider (`# --- <label> ---`).
fn is_family_divider_line(line: &str) -> bool {
    let trimmed = line.trim_end_matches('\n').trim();
    trimmed.starts_with("# ---") && trimmed.ends_with("---")
}

/// Split decor text into a leading run of plain user-comment lines and the
/// remainder.
///
/// Blank-line boundary heuristic: a trailing comment belongs to the essence
/// block above only when it follows the last essence key *directly*. The
/// capture therefore stops at the first blank line — anything after it
/// (comments included) sits visually apart from the essence block and stays
/// where it is, e.g. as header decor for the next `[[item]]`. Generated
/// family dividers and outcome comments are never captured either: dividers
/// are regenerated by push_group and outcome comments have their own
/// drain/reattach path.
fn split_essence_trailing_comment_run(decor_text: &str) -> (String, String) {
    let mut boundary = 0;
    for line in decor_text.split_inclusive('\n') {
        let trimmed = line.trim();
        let is_plain_comment = trimmed.starts_with('#')
            && !is_family_divider_line(line)
            && !trimmed.starts_with(UNRESOLVED_COMMENT_PREFIX)
            && !trimmed.starts_with(AUTO_PICKED_COMMENT_PREFIX);
        if !is_plain_comment {
            break;
        }
        boundary += line.len();
    }
    (
        decor_text[..boundary].to_string(),
        decor_text[boundary..].to_string(),
    )
}

/// The essence-table key whose value suffix carries stashed trailing
/// comments through canonicalization. Any canonical stat key present in the
/// parsed essence block works — value-suffix decor rides `RemovedStatItem`
/// into `insert_canonical_stats` — so use the last present one in canonical
/// order. `None` when the item has no essence block with canonical keys, in
/// which case nothing is captured (trailing decor after a plain base block
/// keeps its previous behavior).
fn essence_trailing_stash_key(table: &Table) -> Option<&'static str> {
    let essence = table.get(ESSENCE_TOTALS_KEY)?.as_table()?;
    let mut stash_key = None;
    for (_, key) in canonical_stat_entries() {
        if essence.contains_key(key) {
            stash_key = Some(*key);
        }
    }
    stash_key
}

/// Append captured comment lines below the stash key's value line. A
/// `\n`-prefixed value suffix renders each comment on its own line directly
/// after the assignment; on the next parse toml_edit hands the comment back
/// as next-table prefix (or document-trailing) decor, where
/// `capture_essence_trailing_comments` re-collects it — a stable round-trip.
fn stash_essence_trailing_comment(table: &mut Table, captured: &str) {
    let Some(stash_key) = essence_trailing_stash_key(table) else {
        return;
    };
    let Some(stash_value) = table
        .get_mut(ESSENCE_TOTALS_KEY)
        .and_then(|essence_item| essence_item.as_table_mut())
        .and_then(|essence_table| essence_table.get_mut(stash_key))
        .and_then(Item::as_value_mut)
    else {
        return;
    };
    let mut suffix = stash_value
        .decor()
        .suffix()
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    for line in captured.lines() {
        suffix.push('\n');
        suffix.push_str(line);
    }
    stash_value.decor_mut().set_suffix(suffix);
}

/// Capture user comments written after the last key of an
/// `[item.EssenceTotals]` block and stash them inside that block so they
/// survive canonicalization.
///
/// toml_edit never stores such comments on the essence table itself: they
/// parse as prefix decor of the *next* `[[item]]` table, or as
/// document-trailing decor when the essence block ends the file. Both spots
/// are rewritten wholesale during regrouping (push_group rebuilds next-table
/// prefixes around family dividers, and tables move), so without capture the
/// comment is dropped or migrates across item boundaries. Only items that
/// actually end in an essence block participate; see
/// `split_essence_trailing_comment_run` for the blank-line boundary
/// heuristic that keeps next-item header comments untouched.
fn capture_essence_trailing_comments(tables: &mut [Table], doc: &mut DocumentMut) {
    for i in 1..tables.len() {
        let (head, tail) = tables.split_at_mut(i);
        let prev = head.last_mut().expect("head is non-empty when i >= 1");
        if essence_trailing_stash_key(prev).is_none() {
            continue;
        }
        let next = &mut tail[0];
        let prefix = next
            .decor()
            .prefix()
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let (captured, remainder) = split_essence_trailing_comment_run(&prefix);
        if captured.is_empty() {
            continue;
        }
        next.decor_mut().set_prefix(remainder);
        stash_essence_trailing_comment(prev, &captured);
    }

    let Some(last) = tables.last_mut() else {
        return;
    };
    if essence_trailing_stash_key(last).is_none() {
        return;
    }
    let trailing = doc.trailing().as_str().unwrap_or("").to_string();
    let (captured, remainder) = split_essence_trailing_comment_run(&trailing);
    if captured.is_empty() {
        return;
    }
    doc.set_trailing(remainder);
    stash_essence_trailing_comment(last, &captured);
}

/// Move any stashed below-line comments (value suffixes containing `\n`)
/// onto the *last* canonical essence key so they render after the final key
/// line of the rebuilt block. The stash key at capture time is whatever
/// canonical key the hand-written block happened to end with; after
/// canonicalization the block always contains every canonical key, so the
/// comment's "after the last essence key" position is re-anchored here.
/// Same-line suffixes (no `\n`) are per-key comments and stay put.
fn hoist_essence_trailing_comments_to_last_line(essence_table: &mut Table) {
    let mut below_lines = String::new();
    for (_, key) in canonical_stat_entries() {
        let Some(stat_value) = essence_table.get_mut(key).and_then(Item::as_value_mut) else {
            continue;
        };
        let suffix = stat_value
            .decor()
            .suffix()
            .and_then(|s| s.as_str())
            .unwrap_or("");
        let Some(newline) = suffix.find('\n') else {
            continue;
        };
        let (same_line, below) = suffix.split_at(newline);
        below_lines.push_str(below);
        let same_line = same_line.to_string();
        stat_value.decor_mut().set_suffix(same_line);
    }
    if below_lines.is_empty() {
        return;
    }
    let Some((_, last_key)) = canonical_stat_entries().last() else {
        return;
    };
    let Some(last_value) = essence_table.get_mut(last_key).and_then(Item::as_value_mut) else {
        return;
    };
    let mut suffix = last_value
        .decor()
        .suffix()
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    suffix.push_str(&below_lines);
    last_value.decor_mut().set_suffix(suffix);
}

/// Read all canonical stat keys (tracked + Base) from a table. Zero values
/// are omitted — absence means zero throughout the pipeline.
fn read_item_stats(table: &Table) -> HashMap<Stat, i64> {
    let mut values = HashMap::new();
    for (stat, key) in canonical_stat_entries() {
        if let Some(stat_value) = table.get(key).and_then(|v| v.as_integer()) {
            if stat_value != 0 {
                values.insert(*stat, stat_value);
            }
        }
    }
    values
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
//   * Items present in both `previous` and `incoming` are matched per owned
//     instance: exact canonical data equality first, then the most-similar
//     remaining same-name candidate (fewest differing canonical fields, ties
//     by occurrence order). Names are compared NFC-normalized so a Unicode
//     re-encoding of the same name never orphans hand-edits. Matching never
//     uses display name alone as a unique key.
//   * Items present only in `incoming` are added.
//   * Items present only in `previous` are removed (they have disappeared
//     from the new export).
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
///
/// `db` is consulted to refresh generated metadata (`two_handed`) on merged
/// item tables: preserving a previous block must not preserve a stale flag,
/// and only the DB can distinguish "known one-handed item" (remove flag)
/// from "unknown item" (preserve the user's hand-edited flag).
pub fn merge_into_canonical(
    previous: Option<&str>,
    incoming_resolved: &str,
    db: &ItemsDb,
    mut force: ForceMode,
) -> Result<MergeOutcome, ResolveError> {
    // Collect names whose resolved slot is still "Unknown" in incoming —
    // reported regardless of merge outcome (the user may need to hand-edit).
    let unknown_slot = collect_unknown_slot_names(incoming_resolved)?;

    // First run: take incoming verbatim. All items are "added".
    let previous_src = match previous {
        Some(previous_src) => previous_src,
        None => {
            let added = item_names(incoming_resolved)?;
            return Ok(MergeOutcome {
                added,
                preserved: Vec::new(),
                overwritten: Vec::new(),
                removed: Vec::new(),
                unknown_slot,
                merged_text: apply_generated_timestamp_comment(incoming_resolved),
            });
        }
    };

    // Subsequent run: start from `previous` (so the document header /
    // top-level decor round-trips), replace its `[[item]]` array with the
    // merged set, and let push_group regroup by family.
    let mut prev_doc: DocumentMut = previous_src.parse().map_err(|e| ResolveError::ParseToml {
        path: PathBuf::from("<previous>"),
        source: Box::new(e),
    })?;
    let mut incoming_doc: DocumentMut =
        incoming_resolved
            .parse()
            .map_err(|e| ResolveError::ParseToml {
                path: PathBuf::from("<incoming>"),
                source: Box::new(e),
            })?;

    let mut prev_tables = take_item_tables(&mut prev_doc, "<previous>")?;
    let mut incoming_tables = take_item_tables(&mut incoming_doc, "<incoming>")?;
    // Rescue trailing comments written after the last essence key in both
    // documents before canonicalization/regrouping rewrites the decor spots
    // toml_edit parked them in (next-table prefix or document trailing).
    capture_essence_trailing_comments(&mut prev_tables, &mut prev_doc);
    capture_essence_trailing_comments(&mut incoming_tables, &mut incoming_doc);

    // Carry forward `character` and `class` from the incoming resolved TOML into
    // the canonical document so that metadata stays current after a merge. When
    // the value is already current, leave the previous item untouched:
    // toml_edit stores leading comments/alignment as decor on these top-level
    // values, and replacing them would make repeat merges serialize
    // differently.
    for key in &["character", "class"] {
        if let Some(val) = incoming_doc.get(key).cloned() {
            if prev_doc.get(key).and_then(|existing| existing.as_str()) != val.as_str() {
                prev_doc.insert(key, val);
            }
        }
    }

    // Carry forward `[InnateStats]` from the incoming resolved TOML. The block
    // is generated metadata and must stay Base-stats-only: refresh the five raw
    // Base stats from current export truth and drop any stale or hand-added
    // extra keys. When values are already current, keep the existing table in
    // place so repeat merges serialize byte-identically apart from spacing
    // normalization.
    if let Some(incoming_innate) = incoming_doc.get("InnateStats").and_then(|i| i.as_table()) {
        let canonical_incoming_innate =
            build_innate_stats_table(&read_innate_base_stats(incoming_innate));
        match prev_doc
            .get_mut("InnateStats")
            .and_then(|i| i.as_table_mut())
        {
            Some(prev_innate) => {
                if innate_table_matches_canonical(prev_innate, &canonical_incoming_innate) {
                    ensure_table_header_note(prev_innate, INNATE_STATS_NOTE);
                    normalize_existing_canonical_stat_decor(prev_innate);
                } else {
                    *prev_innate = canonical_incoming_innate;
                }
            }
            None => {
                prev_doc.insert("InnateStats", Item::Table(canonical_incoming_innate));
            }
        }
    }

    // Carry forward or create `[Virtues]`. These five fields are user-maintained:
    // preserve existing values verbatim, add any missing fields as empty
    // strings, and normalize their alignment.
    let should_have_virtues = incoming_doc.get("InnateStats").is_some()
        || prev_doc.get("InnateStats").is_some()
        || prev_doc.get(VIRTUE_TABLE_KEY).is_some();
    if should_have_virtues {
        match prev_doc
            .get_mut(VIRTUE_TABLE_KEY)
            .and_then(|i| i.as_table_mut())
        {
            Some(prev_virtues) => ensure_virtue_fields(prev_virtues),
            None => {
                let mut virtues = incoming_doc
                    .get(VIRTUE_TABLE_KEY)
                    .and_then(|i| i.as_table())
                    .cloned()
                    .unwrap_or_else(build_virtues_table);
                ensure_virtue_fields(&mut virtues);
                prev_doc.insert(VIRTUE_TABLE_KEY, Item::Table(virtues));
            }
        }
    }

    let (mut incoming_by_name, incoming_order) = group_incoming_by_name(incoming_tables);

    let mut outcome = MergeOutcome {
        unknown_slot,
        ..MergeOutcome::default()
    };

    // Two independent "yes to all" flags: per the brief, `a` for overwrite
    // does not auto-accept removals and vice versa.
    let mut yes_all_overwrite = false;
    let mut yes_all_remove = false;

    let mut merged_tables: Vec<Table> = Vec::new();

    for prev in prev_tables {
        let Some(name) = table_name(&prev) else {
            // No `name` field — preserve the table in place; the item
            // can't participate in the merge. (This shouldn't happen in
            // practice; gearstats reading would already have rejected it.)
            merged_tables.push(prev);
            continue;
        };

        if let Some(incoming) = take_matching_incoming(&prev, &name, &mut incoming_by_name) {
            match &mut force {
                ForceMode::NoForce => {
                    outcome.preserved.push(name);
                    merged_tables.push(prev);
                }
                ForceMode::Force { prompter } => {
                    if item_data_equal(&prev, &incoming) {
                        // Identical data — never prompt, never count.
                        outcome.preserved.push(name);
                        merged_tables.push(prev);
                    } else {
                        let answer = if yes_all_overwrite {
                            PromptAnswer::Yes
                        } else {
                            prompter.prompt(PromptCategory::Overwrite, &name)
                        };
                        match answer {
                            PromptAnswer::YesToAll => {
                                yes_all_overwrite = true;
                                outcome.overwritten.push(name);
                                merged_tables.push(incoming);
                            }
                            PromptAnswer::Yes => {
                                outcome.overwritten.push(name);
                                merged_tables.push(incoming);
                            }
                            PromptAnswer::No => {
                                outcome.preserved.push(name);
                                merged_tables.push(prev);
                            }
                        }
                    }
                }
            }
        } else {
            // Item disappeared from the new export.
            match &mut force {
                ForceMode::NoForce => {
                    outcome.removed.push(name);
                    // drop prev
                }
                ForceMode::Force { prompter } => {
                    let answer = if yes_all_remove {
                        PromptAnswer::Yes
                    } else {
                        prompter.prompt(PromptCategory::Remove, &name)
                    };
                    match answer {
                        PromptAnswer::YesToAll => {
                            yes_all_remove = true;
                            outcome.removed.push(name);
                        }
                        PromptAnswer::Yes => {
                            outcome.removed.push(name);
                        }
                        PromptAnswer::No => {
                            // Keep prev, do not count toward removed.
                            outcome.preserved.push(name);
                            merged_tables.push(prev);
                        }
                    }
                }
            }
        }
    }

    // Items in incoming not matched to a previous instance → added (no prompt).
    // Iterate in original incoming occurrence order so duplicate same-name
    // additions remain deterministic and preserve multiplicity.
    for name in incoming_order {
        if let Some(table) = pop_first_incoming(&name, &mut incoming_by_name) {
            outcome.added.push(name);
            merged_tables.push(table);
        }
    }

    // Regroup the merged set by canonical slot family. Strip any
    // pre-existing `# --- ... ---` divider lines from each table's prefix
    // first so dividers don't accumulate across runs (idempotency).
    // Also refresh generated `two_handed` metadata from the DB: a preserved
    // previous block must gain/lose the flag per current DB truth, while
    // unknown items keep whatever the user wrote by hand.
    for t in &mut merged_tables {
        if let Some(name) = table_name(t) {
            let outcome_comments = take_outcome_comments_from_item(t);
            if db.lookup(&name).is_some() {
                set_two_handed_flag(t, db.lookup_two_handed(&name));
                set_either_hand_flag(t, db.lookup_either_hand(&name));
            }
            canonicalize_item_stats(t, &outcome_comments);
        } else {
            canonicalize_item_stats(t, "");
        }
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

    reorder_resolved_header_before_items(&mut prev_doc);

    outcome.merged_text = apply_generated_timestamp_comment(&prev_doc.to_string());
    Ok(outcome)
}

fn apply_generated_timestamp_comment(src: &str) -> String {
    const HEADER_DELIM: &str = "# _-=-_-=-_-=-_-=-_-=-_-=-_";
    const AUTHOR_LINE: &str = "# LGO 2026, by Thalya";
    const TIMESTAMP_PREFIX: &str = "# gearReady.toml updated:";

    let stripped = strip_generated_header_block(src);

    format!(
        "{delim}\n{author}\n{timestamp_prefix} {timestamp}\n{delim}\n{body}",
        delim = HEADER_DELIM,
        author = AUTHOR_LINE,
        timestamp_prefix = TIMESTAMP_PREFIX,
        timestamp = format_generated_timestamp(),
        body = stripped
    )
}

fn strip_generated_header_block(src: &str) -> String {
    const HEADER_DELIM: &str = "# _-=-_-=-_-=-_-=-_-=-_-=-_";
    const AUTHOR_LINE: &str = "# LGO 2026, by Thalya";
    const TIMESTAMP_PREFIX: &str = "# gearReady.toml updated:";

    let lines: Vec<&str> = src.split_inclusive('\n').collect();
    if lines.is_empty() {
        return String::new();
    }

    let mut start = 0usize;

    while start < lines.len() {
        let trimmed = lines[start].trim_end_matches(['\r', '\n']);

        if trimmed == HEADER_DELIM {
            let mut end = start + 1;
            while end < lines.len() {
                let trimmed_end = lines[end].trim_end_matches(['\r', '\n']);
                if trimmed_end == HEADER_DELIM {
                    end += 1;
                    break;
                }
                end += 1;
            }

            let mut kept = String::new();
            for (idx, line) in lines.iter().enumerate() {
                if idx < start || idx >= end {
                    kept.push_str(line);
                }
            }
            return strip_generated_header_block(&kept);
        }

        if trimmed == AUTHOR_LINE || trimmed.starts_with(TIMESTAMP_PREFIX) {
            start += 1;
            continue;
        }

        break;
    }

    // Beyond the top-of-file banner, also drop stray generated lines anywhere
    // in the body (e.g. an old-format timestamp comment left mid-file) so
    // generated header lines never accumulate across merges.
    lines[start..]
        .iter()
        .filter(|line| {
            let trimmed = line.trim_end_matches(['\r', '\n']);
            trimmed != HEADER_DELIM
                && trimmed != AUTHOR_LINE
                && !trimmed.starts_with(TIMESTAMP_PREFIX)
        })
        .copied()
        .collect()
}

fn count_generated_timestamp_comments(src: &str) -> usize {
    src.lines()
        .filter(|line| {
            line.trim_end_matches(['\r', '\n'])
                .starts_with("# gearReady.toml updated:")
        })
        .count()
}

fn strip_generated_timestamp_comment(src: &str) -> String {
    strip_generated_header_block(src)
}

fn format_generated_timestamp() -> String {
    Local::now().format("%m/%d/%y %H:%M:%S").to_string()
}

/// Group incoming tables by display name while retaining one occurrence-order
/// entry per owned item instance. The merge consumes these vectors one table at
/// a time, so duplicate same-name instances cannot collapse into one HashMap
/// value. Keys are NFC-normalized so Unicode-equivalent byte sequences group
/// together; each stored table keeps its original `name` bytes.
fn group_incoming_by_name(tables: Vec<Table>) -> (HashMap<String, Vec<Table>>, Vec<String>) {
    let mut by_name: HashMap<String, Vec<Table>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for table in tables {
        if let Some(name) = table_name(&table) {
            let key = nfc_name(&name).into_owned();
            by_name.entry(key.clone()).or_default().push(table);
            order.push(key);
        }
    }
    (by_name, order)
}

/// Normalize an item name to NFC for matching and lookup purposes only.
///
/// LGO item names carry non-ASCII characters (see Bug 4 in
/// `docs/BUG_HISTORY.md`), and the browser save path can deliver NFD where a
/// previous file holds NFC (or vice versa). Matching on normalized names
/// keeps such items paired so hand-edits survive; stored `name` values are
/// never rewritten — the output keeps whatever bytes the preserved table
/// carries. This is deliberately not general rename detection (deferred per
/// `docs/AGENT_CONTEXT.md` §10).
fn nfc_name(name: &str) -> Cow<'_, str> {
    if is_nfc(name) {
        Cow::Borrowed(name)
    } else {
        Cow::Owned(name.nfc().collect())
    }
}

/// True when two names are equal after NFC normalization, without allocating
/// when both sides are already NFC (the common case).
fn nfc_eq(a: &str, b: &str) -> bool {
    if is_nfc(a) && is_nfc(b) {
        a == b
    } else {
        a.nfc().eq(b.nfc())
    }
}

/// Consume the best incoming match for one previous owned instance:
/// exact canonical fields first (ignoring comments/decor), then the
/// most-similar remaining same-name candidate — fewest differing canonical
/// fields per `item_data_distance` — with ties broken by occurrence order.
/// The similarity fallback keeps pairing deterministic while ensuring a
/// hand-edited instance pairs with its true incoming counterpart instead of
/// whichever same-name occurrence happens to come first.
fn take_matching_incoming(
    prev: &Table,
    name: &str,
    incoming_by_name: &mut HashMap<String, Vec<Table>>,
) -> Option<Table> {
    let key = nfc_name(name);
    let (matched, empty_after) = {
        let candidates = incoming_by_name.get_mut(key.as_ref())?;
        let idx = candidates
            .iter()
            .position(|incoming| item_data_equal(prev, incoming))
            .or_else(|| {
                // No exact canonical match remains (e.g. this instance was
                // hand-edited), so pair with the candidate that differs in
                // the fewest canonical fields; ties keep occurrence order.
                candidates
                    .iter()
                    .enumerate()
                    .min_by_key(|(idx, incoming)| (item_data_distance(prev, incoming), *idx))
                    .map(|(idx, _)| idx)
            })
            .unwrap_or(0);
        let matched = candidates.remove(idx);
        (matched, candidates.is_empty())
    };
    if empty_after {
        incoming_by_name.remove(key.as_ref());
    }
    Some(matched)
}

fn pop_first_incoming(
    name: &str,
    incoming_by_name: &mut HashMap<String, Vec<Table>>,
) -> Option<Table> {
    let key = nfc_name(name);
    let (matched, empty_after) = {
        let candidates = incoming_by_name.get_mut(key.as_ref())?;
        let matched = candidates.remove(0);
        (matched, candidates.is_empty())
    };
    if empty_after {
        incoming_by_name.remove(key.as_ref());
    }
    Some(matched)
}

/// Bucket pre-resolved tables by canonical slot family, looking only at
/// each table's own `slot` field. Tables with an unrecognised slot string
/// fall into the `Unknown` group.
fn bucket_by_table_slot(tables: Vec<Table>) -> (HashMap<Slot, Vec<Table>>, Vec<Table>) {
    let mut buckets: HashMap<Slot, Vec<Table>> = HashMap::new();
    let mut unknowns: Vec<Table> = Vec::new();
    for table in tables {
        let slot_str = table.get("slot").and_then(|v| v.as_str()).unwrap_or("");
        match parse_slot_display(slot_str) {
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
        .filter(|line| !is_family_divider_line(line))
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
        source: Box::new(e),
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
        source: Box::new(e),
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
/// `name`, `slot`, all canonical stats (tracked + Base), and the same key set
/// in EssenceTotals. Comments, whitespace, and other decor are ignored, and
/// `name` is compared NFC-normalized so Unicode-equivalent byte sequences
/// count as the same item.
///
/// `two_handed` is deliberately *not* compared: it is generated metadata
/// that the merge refreshes from the DB unconditionally, so a flag-only
/// difference must not trigger an overwrite prompt under `--force`. The same
/// reasoning applies to `either_hand`.
fn item_data_equal(a: &Table, b: &Table) -> bool {
    item_data_distance(a, b) == 0
}

/// Count how many canonical fields differ between two `[[item]]` tables —
/// the exact comparison basis of `item_data_equal` (`name`, `slot`, the 21
/// canonical stats, and the same 21 keys under EssenceTotals) expressed as
/// a distance. Used by `take_matching_incoming` to pair a hand-edited
/// previous instance with its most-similar remaining incoming candidate.
fn item_data_distance(a: &Table, b: &Table) -> usize {
    let mut distance = 0;
    let names_equal = match (table_name(a), table_name(b)) {
        (Some(a_name), Some(b_name)) => nfc_eq(&a_name, &b_name),
        (a_name, b_name) => a_name == b_name,
    };
    if !names_equal {
        distance += 1;
    }
    if table_str(a, "slot") != table_str(b, "slot") {
        distance += 1;
    }
    for (_, key) in canonical_stat_entries() {
        if table_int_or_zero(a, key) != table_int_or_zero(b, key) {
            distance += 1;
        }
        if table_nested_int_or_zero(a, ESSENCE_TOTALS_KEY, key)
            != table_nested_int_or_zero(b, ESSENCE_TOTALS_KEY, key)
        {
            distance += 1;
        }
    }
    distance
}

fn table_str(t: &Table, key: &str) -> Option<String> {
    t.get(key).and_then(|v| v.as_str()).map(String::from)
}

fn table_int_or_zero(t: &Table, key: &str) -> i64 {
    t.get(key).and_then(|v| v.as_integer()).unwrap_or(0)
}

fn table_nested_int_or_zero(t: &Table, nested: &str, key: &str) -> i64 {
    t.get(nested)
        .and_then(|item| item.as_table())
        .and_then(|table| table.get(key))
        .and_then(|v| v.as_integer())
        .unwrap_or(0)
}

// =============================================================================
// resolve_stats_file — file-level wrapper (does I/O)
// =============================================================================

fn plugin_base_stats_to_stats(
    raw: &HashMap<String, i64>,
) -> Result<HashMap<Stat, i64>, ResolveError> {
    let mut stats = HashMap::new();
    for (_, key) in BASE_STATS {
        if let Some(value) = raw.get(*key).copied().filter(|value| *value != 0) {
            let stat = key.parse::<Stat>().map_err(|_| ResolveError::Derivation {
                source: DerivationError::UnknownStat {
                    stat_name: (*key).to_string(),
                },
            })?;
            stats.insert(stat, value);
        }
    }
    Ok(stats)
}

fn innate_table_matches_canonical(table: &Table, expected: &Table) -> bool {
    table
        .iter()
        .map(|(key, _)| key)
        .eq(BASE_STATS.iter().map(|(_, key)| *key))
        && BASE_STATS.iter().all(|(_, key)| {
            table.get(key).and_then(|item| item.as_integer())
                == expected.get(key).and_then(|item| item.as_integer())
        })
}

fn find_latest_plugindata_file(
    dir: &Path,
    character: &str,
) -> Result<Option<PathBuf>, ResolveError> {
    let prefix = format!("lgo_{}_", character.to_ascii_lowercase());
    let mut matches: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| ResolveError::IoRead {
            path: dir.to_path_buf(),
            source: e,
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| {
                    let lower = name.to_ascii_lowercase();
                    lower.starts_with(&prefix)
                        && lower.contains("_gearnames_")
                        && lower.ends_with(".plugindata")
                })
                .unwrap_or(false)
        })
        .collect();
    // Plugin exports are named `lgo_<character>_gearNames_YYYYMMDD_HHMMSS.plugindata`,
    // so lexicographic filename order is timestamp order.
    matches.sort_by(|a, b| {
        a.file_name()
            .expect("directory entry path has a file name")
            .cmp(b.file_name().expect("directory entry path has a file name"))
    });
    Ok(matches.pop())
}

/// End-to-end iteration step:
///
///   1. Read `lgo_<character>_gearStats.toml` (the bookmarklet's output).
///   2. Slot-resolve it via `db`.
///   3. Read `lgo_<character>_gearReady.toml` (the canonical merged file) if
///      it exists.
///   4. Merge per `merge_into_canonical` semantics.
///   5. Write the result back to `lgo_<character>_gearReady.toml`.
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
    // Use directory scans so that e.g. `lgo_thalya_gearStats.toml` is found when
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

    let plugin_export = find_latest_plugindata_file(char_dir, character)?
        .map(|path| {
            // `plugindata` parses the in-game `gearNames` export, including
            // character class and raw base stats for the `[InnateStats]` block.
            crate::plugindata::load(&path)
                .map_err(|message| ResolveError::PluginData { path, message })
        })
        .transpose()?;

    let bookmarklet_doc: DocumentMut =
        bookmarklet_src
            .parse()
            .map_err(|e| ResolveError::ParseToml {
                path: bookmarklet_path.clone(),
                source: Box::new(e),
            })?;
    let fallback_class = bookmarklet_doc
        .get("class")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let fallback_character = bookmarklet_doc
        .get("character")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    drop(bookmarklet_doc);

    let class_name = plugin_export
        .as_ref()
        .map(|export| export.class.as_str())
        .or(fallback_class.as_deref());
    let character_name = plugin_export
        .as_ref()
        .map(|export| export.character.as_str())
        .or(fallback_character.as_deref());
    let base_stats = plugin_export
        .as_ref()
        .map(|export| plugin_base_stats_to_stats(&export.base_stats))
        .transpose()?;
    let empty_base_stats: HashMap<Stat, i64> = HashMap::new();
    let base_stats_ref = base_stats.as_ref().unwrap_or(&empty_base_stats);

    let (resolved_src, _outcomes) = if let Some(class_name) = class_name {
        resolve_toml_str_with_metadata(
            &bookmarklet_src,
            db,
            character_name,
            class_name,
            base_stats_ref,
        )
    } else {
        resolve_toml_str(&bookmarklet_src, db)
    }
    .map_err(|e| match e {
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

    let outcome = merge_into_canonical(previous_src.as_deref(), &resolved_src, db, force).map_err(
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
/// scheme: `<dir>/lgo_<character>_gearStats.toml`.
pub fn bookmarklet_stats_path(dir: &Path, character: &str) -> PathBuf {
    dir.join(format!("lgo_{}_gearStats.toml", character))
}

/// Canonical merged gear file: `<dir>/lgo_<character>_gearReady.toml`.
pub fn canonical_gear_path(dir: &Path, character: &str) -> PathBuf {
    dir.join(format!("lgo_{}_gearReady.toml", character))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    type TestStats<'a> = &'a [(&'a str, i64)];
    type TestItem<'a> = (&'a str, &'a str, TestStats<'a>);

    /// Small synthetic fixture exercising four slot shapes:
    /// identity (Head), paired (Wrist), weapon (Main-hand), and space-split
    /// (Class Item).
    /// "Test Greatsword" additionally carries `two_handed: true` as emitted
    /// by `build-db` for `MAIN_HAND` items with `precludedSlots`.
    const FIXTURE: &str = r#"{
        "Test Helm": {
            "name": "Test Helm",
            "slot": "Head"
        },
        "Test Bracelet": {
            "name": "Test Bracelet",
            "slot": "Wrist"
        },
        "Test Sword": {
            "name": "Test Sword",
            "slot": "Main-hand"
        },
        "Test Greatsword": {
            "name": "Test Greatsword",
            "slot": "Main-hand",
            "two_handed": true
        },
        "Test Rune-stone": {
            "name": "Test Rune-stone",
            "slot": "Off-hand",
            "either_hand": true
        },
        "Test Tome": {
            "name": "Test Tome",
            "slot": "Class Item"
        }
    }"#;

    fn dummy_path() -> &'static Path {
        Path::new("<test-fixture>")
    }

    fn fixture_db() -> ItemsDb {
        ItemsDb::from_json_str(FIXTURE, dummy_path()).expect("fixture must parse")
    }

    fn has_assignment_line(src: &str, key: &str, expected: i64) -> bool {
        let expected = expected.to_string();
        src.lines().any(|line| {
            let trimmed = line.trim_start();
            let Some(rest) = trimmed.strip_prefix(key) else {
                return false;
            };
            let Some(rest) = rest.trim_start_matches(' ').strip_prefix('=') else {
                return false;
            };
            rest.trim_start_matches(' ') == expected
        })
    }

    fn assert_header_note_immediately_after(src: &str, header: &str, note: &str) {
        let expected = format!("{header}\n{note}\n");
        assert!(
            src.contains(&expected),
            "expected note immediately after {header}:\n{src}"
        );
        assert_eq!(
            src.matches(note).count(),
            1,
            "expected note exactly once for {header}:\n{src}"
        );
    }

    fn assert_stat_assignments_align_to_column_20(src: &str) {
        let mut saw_stat_line = false;
        for line in src.lines() {
            let trimmed = line.trim_start();
            let Some((key, _)) = trimmed.split_once('=') else {
                continue;
            };
            let key = key.trim_end_matches([' ', '\t']);
            if canonical_stat_entries().any(|(_, canonical_key)| *canonical_key == key) {
                saw_stat_line = true;
                assert_eq!(
                    trimmed.find('=').map(|idx| idx + 1),
                    Some(STAT_EQUALS_COLUMN),
                    "stat assignment must align '=' to column {}:\n{}",
                    STAT_EQUALS_COLUMN,
                    line
                );
            }
        }
        assert!(
            saw_stat_line,
            "test input should contain stat assignment lines"
        );
    }

    /// Test shorthand for `merge_into_canonical` against the shared fixture DB.
    fn merge_ic(
        previous: Option<&str>,
        incoming_resolved: &str,
        force: ForceMode,
    ) -> Result<MergeOutcome, ResolveError> {
        merge_into_canonical(previous, incoming_resolved, &fixture_db(), force)
    }

    // -- ItemsDb tests (from step 3) --

    #[test]
    fn loads_fixture_and_resolves_known_items() {
        let db = fixture_db();
        assert_eq!(db.len(), 6);
        assert!(!db.is_empty());

        assert_eq!(db.lookup("Test Helm"), Some(Slot::Head));
        assert_eq!(db.lookup("Test Bracelet"), Some(Slot::Wrist1));
        assert_eq!(db.lookup("Test Sword"), Some(Slot::MainHand));
        assert_eq!(db.lookup("Test Greatsword"), Some(Slot::MainHand));
        assert_eq!(db.lookup("Test Rune-stone"), Some(Slot::OffHand));
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
                "slot": "Frobnicate"
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
    fn old_db_slot_value_is_an_error() {
        let bad = r#"{
            "Old Sword": {
                "name": "Old Sword",
                "slot": "MainHand"
            }
        }"#;
        let err =
            ItemsDb::from_json_str(bad, dummy_path()).expect_err("old DB slot string must error");
        match err {
            ItemsDbError::UnknownSlot {
                item_name,
                slot_string,
            } => {
                assert_eq!(item_name, "Old Sword");
                assert_eq!(slot_string, "MainHand");
            }
            other => panic!("expected UnknownSlot, got {:?}", other),
        }
    }

    #[test]
    fn excluded_lotro_slot_is_an_error() {
        let bad = r#"{
            "Mining Pick": {
                "name": "Mining Pick",
                "slot": "CraftItem"
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
    /// requires the 5 MB file on disk. To run it:
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
    /// in `Slot::all()` or `slot_family` (the inputs to `slot_family_order`)
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
    fn resolver_writes_raw_base_stats_into_innate_stats() {
        let db = fixture_db();
        let base_stats: HashMap<Stat, i64> =
            [(Stat::Agility, 1000), (Stat::Vitality, 2), (Stat::Fate, 2)]
                .into_iter()
                .collect();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n";

        let (out, _) =
            resolve_toml_str_with_metadata(input, &db, Some("Thalya"), "Lore-master", &base_stats)
                .expect("must resolve with metadata");

        let doc: DocumentMut = out.parse().expect("output parses");
        let innate = doc
            .get("InnateStats")
            .and_then(|item| item.as_table())
            .expect("InnateStats table exists");
        // Exactly the five raw Base stats, in canonical order, no derived
        // tracked stats. Missing plugindata values are written as zero.
        let keys: Vec<&str> = innate.iter().map(|(key, _)| key).collect();
        assert_eq!(keys, vec!["Might", "Agility", "Vitality", "Will", "Fate"]);
        assert_eq!(
            innate.get("Might").and_then(|item| item.as_integer()),
            Some(0)
        );
        assert_eq!(
            innate.get("Agility").and_then(|item| item.as_integer()),
            Some(1000)
        );
        assert_eq!(
            innate.get("Vitality").and_then(|item| item.as_integer()),
            Some(2)
        );
        assert_eq!(
            innate.get("Will").and_then(|item| item.as_integer()),
            Some(0)
        );
        assert_eq!(
            innate.get("Fate").and_then(|item| item.as_integer()),
            Some(2)
        );
    }

    #[test]
    fn resolver_places_innate_stats_before_virtues_and_items() {
        let db = fixture_db();
        let base_stats: HashMap<Stat, i64> =
            [(Stat::Agility, 1000), (Stat::Vitality, 2), (Stat::Fate, 2)]
                .into_iter()
                .collect();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n";

        let (out, _) =
            resolve_toml_str_with_metadata(input, &db, Some("Thalya"), "Lore-master", &base_stats)
                .expect("must resolve with metadata");

        let character_pos = out.find("character").expect("character field exists");
        let class_pos = out.find("class").expect("class field exists");
        let innate_pos = out.find("[InnateStats]").expect("InnateStats block exists");
        let virtues_pos = out.find("[Virtues]").expect("Virtues block exists");
        let item_pos = out.find("[[item]]").expect("item array exists");

        assert!(
            character_pos < class_pos,
            "character should appear before class:\n{}",
            out
        );
        assert!(
            class_pos < innate_pos,
            "class should appear before InnateStats:\n{}",
            out
        );
        assert!(
            innate_pos < virtues_pos,
            "InnateStats should appear before Virtues:\n{}",
            out
        );
        assert!(
            virtues_pos < item_pos,
            "Virtues should appear before the first [[item]] block:\n{}",
            out
        );
    }

    #[test]
    fn resolver_writes_virtues_block_between_innate_stats_and_first_item() {
        let db = fixture_db();
        let base_stats: HashMap<Stat, i64> = [(Stat::Will, 77)].into_iter().collect();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n";

        let (out, _) =
            resolve_toml_str_with_metadata(input, &db, Some("Thalya"), "Lore-master", &base_stats)
                .expect("must resolve with metadata");

        let doc: DocumentMut = out.parse().expect("output parses");
        let virtues = doc
            .get(VIRTUE_TABLE_KEY)
            .and_then(|item| item.as_table())
            .expect("Virtues table exists");
        let keys: Vec<&str> = virtues.iter().map(|(key, _)| key).collect();
        assert_eq!(keys, VIRTUE_FIELD_KEYS);
        for key in VIRTUE_FIELD_KEYS {
            assert_eq!(virtues.get(key).and_then(|item| item.as_str()), Some(""));
        }

        let innate_pos = out.find("[InnateStats]").expect("InnateStats block exists");
        let virtues_pos = out.find("[Virtues]").expect("Virtues block exists");
        let item_pos = out.find("[[item]]").expect("item array exists");
        assert!(
            innate_pos < virtues_pos && virtues_pos < item_pos,
            "Virtues must sit between InnateStats and the first item:\n{}",
            out
        );
    }

    #[test]
    fn resolver_writes_innate_stats_and_virtues_even_when_base_stats_are_empty() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n";
        let empty_base_stats = HashMap::new();

        let (out, _) = resolve_toml_str_with_metadata(
            input,
            &db,
            Some("Thalya"),
            "Lore-master",
            &empty_base_stats,
        )
        .expect("must resolve with metadata");

        let doc: DocumentMut = out.parse().expect("output parses");
        let innate = doc
            .get("InnateStats")
            .and_then(|item| item.as_table())
            .expect("InnateStats table exists");
        let virtues = doc
            .get(VIRTUE_TABLE_KEY)
            .and_then(|item| item.as_table())
            .expect("Virtues table exists");

        let innate_keys: Vec<&str> = innate.iter().map(|(key, _)| key).collect();
        assert_eq!(
            innate_keys,
            vec!["Might", "Agility", "Vitality", "Will", "Fate"]
        );
        for key in ["Might", "Agility", "Vitality", "Will", "Fate"] {
            assert_eq!(innate.get(key).and_then(|item| item.as_integer()), Some(0));
        }

        let virtue_keys: Vec<&str> = virtues.iter().map(|(key, _)| key).collect();
        assert_eq!(virtue_keys, VIRTUE_FIELD_KEYS);
        for key in VIRTUE_FIELD_KEYS {
            assert_eq!(virtues.get(key).and_then(|item| item.as_str()), Some(""));
        }

        let innate_pos = out.find("[InnateStats]").expect("InnateStats block exists");
        let virtues_pos = out.find("[Virtues]").expect("Virtues block exists");
        let item_pos = out.find("[[item]]").expect("item array exists");
        assert!(
            innate_pos < virtues_pos && virtues_pos < item_pos,
            "Virtues must sit between InnateStats and the first item:\n{}",
            out
        );
    }

    #[test]
    fn resolver_passes_item_base_stats_through_verbatim() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 1000\n\
Agility = 1000\n";
        let empty_base_stats = HashMap::new();

        let (out, _) = resolve_toml_str_with_metadata(
            input,
            &db,
            Some("Thalya"),
            "Lore-master",
            &empty_base_stats,
        )
        .expect("must resolve with metadata");

        let doc: DocumentMut = out.parse().expect("output parses");
        let item = doc
            .get("item")
            .and_then(|item| item.as_array_of_tables())
            .and_then(|items| items.iter().next())
            .expect("one item");
        // No derivation: explicit tracked stats and raw Base stats survive
        // untouched, and no derived contributions appear anywhere.
        assert_eq!(
            item.get("CriticalRating")
                .and_then(|item| item.as_integer()),
            Some(1000)
        );
        assert_eq!(
            item.get("Agility").and_then(|item| item.as_integer()),
            Some(1000)
        );
        assert_eq!(
            item.get("Finesse").and_then(|item| item.as_integer()),
            Some(0)
        );
        assert_eq!(
            item.get("TacticalMastery")
                .and_then(|item| item.as_integer()),
            Some(0)
        );
    }

    #[test]
    fn resolver_writes_base_stats_after_tracked_stats_in_canonical_order() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
Fate = 7\n\
Might = 9\n";
        let empty_base_stats = HashMap::new();

        let (out, _) = resolve_toml_str_with_metadata(
            input,
            &db,
            Some("Thalya"),
            "Lore-master",
            &empty_base_stats,
        )
        .expect("must resolve with metadata");
        let item_pos = out.find("[[item]]").expect("item array exists");
        let item_src = &out[item_pos..];
        let doc: DocumentMut = out.parse().expect("output parses");
        let item = doc
            .get("item")
            .and_then(|item| item.as_array_of_tables())
            .and_then(|items| items.iter().next())
            .expect("one item");

        // The five Base stats come after all 16 tracked stats, in canonical
        // Might/Agility/Vitality/Will/Fate order, and pass through verbatim.
        let positions: Vec<usize> = [
            "TacticalMitigation",
            "Might",
            "Agility",
            "Vitality",
            "Will",
            "Fate",
        ]
        .into_iter()
        .map(|key| {
            item_src
                .find(&format!("{key} "))
                .unwrap_or_else(|| panic!("missing {} in item block:\n{}", key, item_src))
        })
        .collect();
        assert!(
            positions.windows(2).all(|pair| pair[0] < pair[1]),
            "base stats not in canonical position/order:\n{}",
            out
        );
        assert_eq!(
            item.get("Might").and_then(|item| item.as_integer()),
            Some(9)
        );
        assert_eq!(
            item.get("Agility").and_then(|item| item.as_integer()),
            Some(0)
        );
        assert_eq!(
            item.get("Vitality").and_then(|item| item.as_integer()),
            Some(0)
        );
        assert_eq!(item.get("Will").and_then(|item| item.as_integer()), Some(0));
        assert_eq!(item.get("Fate").and_then(|item| item.as_integer()), Some(7));
    }

    #[test]
    fn resolver_keeps_base_stats_out_of_morale_and_power() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
Vitality = 2\n\
Fate = 2\n";
        let empty_base_stats = HashMap::new();

        let (out, _) = resolve_toml_str_with_metadata(
            input,
            &db,
            Some("Thalya"),
            "Lore-master",
            &empty_base_stats,
        )
        .expect("must resolve with metadata");

        // Pass-through means no derived Morale/Power contributions: the raw
        // Base lines survive and tracked totals stay zero.
        assert!(has_assignment_line(&out, "Vitality", 2));
        assert!(has_assignment_line(&out, "Fate", 2));
        assert!(has_assignment_line(&out, "Morale", 0));
        assert!(has_assignment_line(&out, "Power", 0));
        assert!(!out.contains("Regen"));
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
    fn resolver_emits_all_stats_and_zeroed_essence_totals_without_blank_gap() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");
        let item_start = out.find("[[item]]").expect("item emitted");
        let essence_start = out
            .find("[item.EssenceTotals]")
            .expect("EssenceTotals emitted");
        let item_text = &out[item_start..essence_start];
        let essence_text = &out[essence_start..];

        assert!(
            !out.contains("Fate = 0\n\n[item.EssenceTotals]"),
            "EssenceTotals must stay attached to base item:\n{}",
            out
        );
        for (_, key) in canonical_stat_entries() {
            assert!(
                item_text.contains(key),
                "base item missing {}:\n{}",
                key,
                out
            );
            assert!(
                has_assignment_line(essence_text, key, 0),
                "EssenceTotals missing zeroed {}:\n{}",
                key,
                out
            );
        }
    }

    #[test]
    fn resolver_aligns_all_stat_assignments_to_column_20() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating\t=\t100\n\
Might=9\n";
        let base_stats: HashMap<Stat, i64> = [(Stat::Vitality, 3434), (Stat::Will, 77)]
            .into_iter()
            .collect();

        let (out, _) =
            resolve_toml_str_with_metadata(input, &db, Some("Thalya"), "Lore-master", &base_stats)
                .expect("must resolve with metadata");

        assert_stat_assignments_align_to_column_20(&out);
    }

    #[test]
    fn strip_leading_blank_lines_handles_crlf_prefixes() {
        assert_eq!(
            strip_leading_blank_lines("\r\n  \r\n# note\r\n"),
            "# note\r\n"
        );
        assert_eq!(strip_leading_blank_lines("\r\n"), "");
    }

    #[test]
    fn attach_table_to_previous_line_strips_only_leading_blank_lines() {
        let mut table = Table::new();
        table.decor_mut().set_prefix(
            "\n  \r\n# user note: essence totals maintained by hand\n  # keep detail\n\n",
        );

        attach_table_to_previous_line(&mut table);

        assert_eq!(
            table.decor().prefix().and_then(|prefix| prefix.as_str()),
            Some("# user note: essence totals maintained by hand\n  # keep detail\n\n")
        );
    }

    #[test]
    fn resolver_preserves_existing_essence_totals_on_normal_merge() {
        let db = fixture_db();
        let previous = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n\
[item.EssenceTotals]\n\
CriticalRating = 300\n";
        let (prev, _) = resolve_toml_str(previous, &db).expect("resolve previous");
        let incoming_input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 200\n";
        let (incoming, _) = resolve_toml_str(incoming_input, &db).expect("resolve incoming");

        let outcome = merge_ic(Some(&prev), &incoming, ForceMode::NoForce).expect("merge");
        let doc: DocumentMut = outcome.merged_text.parse().expect("output parses");
        let item = doc
            .get("item")
            .and_then(|item| item.as_array_of_tables())
            .and_then(|items| items.iter().next())
            .expect("one item");
        let essence = item
            .get(ESSENCE_TOTALS_KEY)
            .and_then(|item| item.as_table())
            .expect("EssenceTotals table");

        assert_eq!(
            item.get("CriticalRating")
                .and_then(|item| item.as_integer()),
            Some(100),
            "normal merge preserves current base block under existing semantics"
        );
        assert_eq!(
            essence
                .get("CriticalRating")
                .and_then(|item| item.as_integer()),
            Some(300)
        );
    }

    #[test]
    fn merge_force_overwrite_resets_essence_totals_from_incoming() {
        let db = fixture_db();
        let previous = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n\
[item.EssenceTotals]\n\
CriticalRating = 300\n";
        let (prev, _) = resolve_toml_str(previous, &db).expect("resolve previous");
        let incoming_input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 200\n";
        let (incoming, _) = resolve_toml_str(incoming_input, &db).expect("resolve incoming");

        let outcome = merge_ic(
            Some(&prev),
            &incoming,
            force_with(vec![(PromptCategory::Overwrite, PromptAnswer::Yes)]),
        )
        .expect("merge");
        let doc: DocumentMut = outcome.merged_text.parse().expect("output parses");
        let item = doc
            .get("item")
            .and_then(|item| item.as_array_of_tables())
            .and_then(|items| items.iter().next())
            .expect("one item");
        let essence = item
            .get(ESSENCE_TOTALS_KEY)
            .and_then(|item| item.as_table())
            .expect("EssenceTotals table");

        assert_eq!(
            item.get("CriticalRating")
                .and_then(|item| item.as_integer()),
            Some(200)
        );
        assert_eq!(
            essence
                .get("CriticalRating")
                .and_then(|item| item.as_integer()),
            Some(0)
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
    //   writes directly to the canonical `lgo_<character>_gearReady.toml`.
    //   See `merge_into_canonical` and `resolve_stats_file`.

    // -- merge_into_canonical tests --

    /// Helper: a fresh resolved bookmarklet-style document with one item.
    fn make_doc(items: &[TestItem<'_>]) -> String {
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

    fn count_item_name(src: &str, name: &str) -> usize {
        let doc: DocumentMut = src.parse().expect("test TOML must parse");
        doc.get("item")
            .and_then(|v| v.as_array_of_tables())
            .expect("test TOML has [[item]]")
            .iter()
            .filter(|t| table_name(t).as_deref() == Some(name))
            .count()
    }

    /// Strip every generated-header line (delimiters, author line, timestamp)
    /// so outputs can be compared modulo the generated banner block.
    fn strip_generated_timestamp_comment(src: &str) -> String {
        src.split_inclusive('\n')
            .filter(|line| {
                let trimmed = line.trim_end_matches(['\r', '\n']);
                trimmed != "# _-=-_-=-_-=-_-=-_-=-_-=-_"
                    && trimmed != "# LGO 2026, by Thalya"
                    && !trimmed.contains("# gearReady.toml updated:")
            })
            .collect()
    }

    fn count_generated_timestamp_comments(src: &str) -> usize {
        src.lines()
            .filter(|line| line.contains("# gearReady.toml updated:"))
            .count()
    }

    #[test]
    fn merge_first_run_takes_incoming_modulo_timestamp() {
        let incoming = make_doc(&[("Test Helm", "Head", &[("Armor", 100)])]);
        let outcome = merge_ic(None, &incoming, ForceMode::NoForce).expect("must merge");
        assert_eq!(outcome.added, vec!["Test Helm"]);
        assert!(outcome.preserved.is_empty());
        assert!(outcome.removed.is_empty());
        assert_eq!(count_generated_timestamp_comments(&outcome.merged_text), 1);
        assert!(
            outcome.merged_text.starts_with(
                "# _-=-_-=-_-=-_-=-_-=-_-=-_\n# LGO 2026, by Thalya\n# gearReady.toml updated:"
            ),
            "generated banner with timestamp must open the canonical output:\n{}",
            outcome.merged_text
        );
        assert_eq!(
            strip_generated_timestamp_comment(&outcome.merged_text),
            incoming
        );
    }

    #[test]
    fn merge_first_run_preserves_duplicate_same_name_instances() {
        let incoming = make_doc(&[
            ("Test Bracelet", "Wrist", &[("Armor", 100)]),
            ("Test Bracelet", "Wrist", &[("Armor", 200)]),
        ]);
        let outcome = merge_ic(None, &incoming, ForceMode::NoForce).expect("must merge");

        assert_eq!(outcome.added, vec!["Test Bracelet", "Test Bracelet"]);
        assert_eq!(count_item_name(&outcome.merged_text, "Test Bracelet"), 2);
        assert!(has_assignment_line(&outcome.merged_text, "Armor", 100));
        assert!(has_assignment_line(&outcome.merged_text, "Armor", 200));
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
        let first = merge_ic(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;
        let second = merge_ic(Some(&first), &resolved, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        assert_eq!(
            strip_generated_timestamp_comment(&first),
            strip_generated_timestamp_comment(&second),
            "second merge must be identical apart from timestamp:\n--- first ---\n{}\n--- second ---\n{}",
            first,
            second
        );

        // And a third time, just to be sure dividers don't accumulate.
        let third = merge_ic(Some(&second), &resolved, ForceMode::NoForce)
            .expect("third merge")
            .merged_text;
        assert_eq!(
            strip_generated_timestamp_comment(&second),
            strip_generated_timestamp_comment(&third),
            "third merge must also be identical apart from timestamp"
        );
        assert_eq!(count_generated_timestamp_comments(&third), 1);
    }

    #[test]
    fn merge_normalizes_sloppy_stat_spacing_drops_invalid_innate_extras_and_stays_idempotent() {
        let previous = "\
character = \"Thalya\"\n\
class = \"Lore-master\"\n\
\n\
[InnateStats]\n\
CriticalRating\t=\t7\n\
Might=5300\n\
Fate              = 4000\n\
\n\
[[item]]\n\
slot = \"Head\"\n\
name = \"Test Helm\"\n\
CriticalRating\t=\t100\n\
Armor        = 5\n\
Might=9\n\
[item.EssenceTotals]\n\
CriticalRating\t=\t3\n\
Vitality=12\n";
        let incoming = "\
character = \"Thalya\"\n\
class = \"Lore-master\"\n\
\n\
[InnateStats]\n\
Might = 5300\n\
Fate = 4000\n\
\n\
[[item]]\n\
slot = \"Head\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n\
Armor = 5\n\
Might = 9\n";

        let first = merge_ic(Some(previous), incoming, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;
        assert_stat_assignments_align_to_column_20(&first);
        let doc: DocumentMut = first.parse().expect("merged output parses");
        let innate = doc
            .get("InnateStats")
            .and_then(|item| item.as_table())
            .expect("InnateStats table exists");
        let innate_keys: Vec<&str> = innate.iter().map(|(key, _)| key).collect();
        assert_eq!(
            innate_keys,
            vec!["Might", "Agility", "Vitality", "Will", "Fate"]
        );
        assert!(innate.get("CriticalRating").is_none());
        assert_eq!(
            innate.get("Might").and_then(|item| item.as_integer()),
            Some(5300)
        );
        assert_eq!(
            innate.get("Agility").and_then(|item| item.as_integer()),
            Some(0)
        );
        assert_eq!(
            innate.get("Vitality").and_then(|item| item.as_integer()),
            Some(0)
        );
        assert_eq!(
            innate.get("Will").and_then(|item| item.as_integer()),
            Some(0)
        );
        assert_eq!(
            innate.get("Fate").and_then(|item| item.as_integer()),
            Some(4000)
        );

        let item = doc
            .get("item")
            .and_then(|item| item.as_array_of_tables())
            .and_then(|items| items.iter().next())
            .expect("one item");
        assert_eq!(
            item.get("CriticalRating")
                .and_then(|item| item.as_integer()),
            Some(100)
        );
        assert_eq!(
            item.get("Armor").and_then(|item| item.as_integer()),
            Some(5)
        );
        assert_eq!(
            item.get("Might").and_then(|item| item.as_integer()),
            Some(9)
        );
        let essence = item
            .get(ESSENCE_TOTALS_KEY)
            .and_then(|item| item.as_table())
            .expect("EssenceTotals exists");
        assert_eq!(
            essence
                .get("CriticalRating")
                .and_then(|item| item.as_integer()),
            Some(3)
        );
        assert_eq!(
            essence.get("Vitality").and_then(|item| item.as_integer()),
            Some(12)
        );

        let second = merge_ic(Some(&first), incoming, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        assert_eq!(
            strip_generated_timestamp_comment(&first),
            strip_generated_timestamp_comment(&second),
            "normalized output must be idempotent after the first merge"
        );
    }

    #[test]
    fn resolve_toml_str_does_not_insert_canonical_timestamp() {
        let db = fixture_db();
        let bookmarklet = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);

        let (resolved, _) = resolve_toml_str(&bookmarklet, &db).expect("resolve");

        assert_eq!(count_generated_timestamp_comments(&resolved), 0);
    }

    #[test]
    fn merge_removes_old_timestamp_comments_before_prepending_one() {
        let db = fixture_db();
        let bookmarklet = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (resolved, _) = resolve_toml_str(&bookmarklet, &db).expect("resolve");
        let first = merge_ic(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;
        let previous_with_extra_timestamp = first.replacen(
            "[[item]]",
            "# gearReady.toml updated: 01/01/25 00:00:00\n[[item]]",
            1,
        );

        let second = merge_ic(
            Some(&previous_with_extra_timestamp),
            &resolved,
            ForceMode::NoForce,
        )
        .expect("second merge")
        .merged_text;

        assert_eq!(count_generated_timestamp_comments(&second), 1);
        assert!(
            second.starts_with(
                "# _-=-_-=-_-=-_-=-_-=-_-=-_\n# LGO 2026, by Thalya\n# gearReady.toml updated:"
            ),
            "fresh generated banner must be prepended:\n{}",
            second
        );
        assert!(
            !second.contains("01/01/25 00:00:00"),
            "old timestamp comments must be removed:\n{}",
            second
        );
    }

    #[test]
    fn merge_idempotent_with_duplicate_same_name_stat_divergence() {
        let db = fixture_db();
        let bookmarklet = make_doc(&[
            ("Test Bracelet", "Unknown", &[("Armor", 100)]),
            ("Test Bracelet", "Unknown", &[("Armor", 200)]),
        ]);
        let (resolved, _) = resolve_toml_str(&bookmarklet, &db).expect("resolve");
        let first = merge_ic(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;
        let second = merge_ic(Some(&first), &resolved, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        let third = merge_ic(Some(&second), &resolved, ForceMode::NoForce)
            .expect("third merge")
            .merged_text;

        assert_eq!(
            strip_generated_timestamp_comment(&first),
            strip_generated_timestamp_comment(&second)
        );
        assert_eq!(
            strip_generated_timestamp_comment(&second),
            strip_generated_timestamp_comment(&third)
        );
        assert_eq!(count_item_name(&third, "Test Bracelet"), 2);
        assert!(has_assignment_line(&third, "Armor", 100));
        assert!(has_assignment_line(&third, "Armor", 200));
    }

    #[test]
    fn merge_duplicate_matching_prefers_exact_canonical_data_before_occurrence_order() {
        let prev = "\
[[item]]\n\
slot = \"Wrist\"\n\
name = \"Test Bracelet\"\n\
# user note that must not affect identity\n\
Armor = 100\n\
\n\
[[item]]\n\
slot = \"Wrist\"\n\
name = \"Test Bracelet\"\n\
Armor = 900\n";
        let incoming = make_doc(&[
            ("Test Bracelet", "Wrist", &[("Armor", 200)]),
            ("Test Bracelet", "Wrist", &[("Armor", 100)]),
        ]);

        let outcome = merge_ic(
            Some(prev),
            &incoming,
            force_with(vec![(PromptCategory::Overwrite, PromptAnswer::Yes)]),
        )
        .expect("must merge");

        assert_eq!(outcome.preserved, vec!["Test Bracelet"]);
        assert_eq!(outcome.overwritten, vec!["Test Bracelet"]);
        assert!(has_assignment_line(&outcome.merged_text, "Armor", 100));
        assert!(has_assignment_line(&outcome.merged_text, "Armor", 200));
        assert!(!has_assignment_line(&outcome.merged_text, "Armor", 900));
        assert_eq!(count_item_name(&outcome.merged_text, "Test Bracelet"), 2);
    }

    #[test]
    fn merge_comments_and_decor_do_not_create_distinct_instance_identity() {
        let prev = "\
[[item]]\n\
slot = \"Head\"\n\
name = \"Test Helm\"\n\
# Ash Nazg Gimbatul\n\
Armor = 100\n";
        let incoming = make_doc(&[("Test Helm", "Head", &[("Armor", 100)])]);

        let outcome = merge_ic(Some(prev), &incoming, force_with(vec![]))
            .expect("must merge without prompting");

        assert_eq!(outcome.preserved, vec!["Test Helm"]);
        assert!(outcome.added.is_empty());
        assert!(outcome.removed.is_empty());
        assert_eq!(count_item_name(&outcome.merged_text, "Test Helm"), 1);
        assert!(outcome.merged_text.contains("# Ash Nazg Gimbatul"));
    }

    #[test]
    fn merge_count_increase_adds_only_new_duplicate_instances() {
        let prev = make_doc(&[("Test Bracelet", "Wrist", &[("Armor", 100)])]);
        let incoming = make_doc(&[
            ("Test Bracelet", "Wrist", &[("Armor", 100)]),
            ("Test Bracelet", "Wrist", &[("Armor", 200)]),
        ]);

        let outcome = merge_ic(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");

        assert_eq!(outcome.preserved, vec!["Test Bracelet"]);
        assert_eq!(outcome.added, vec!["Test Bracelet"]);
        assert!(outcome.removed.is_empty());
        assert_eq!(count_item_name(&outcome.merged_text, "Test Bracelet"), 2);
    }

    #[test]
    fn merge_count_decrease_removes_only_missing_duplicate_instances() {
        let prev = make_doc(&[
            ("Test Bracelet", "Wrist", &[("Armor", 100)]),
            ("Test Bracelet", "Wrist", &[("Armor", 200)]),
        ]);
        let incoming = make_doc(&[("Test Bracelet", "Wrist", &[("Armor", 100)])]);

        let outcome = merge_ic(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");

        assert_eq!(outcome.preserved, vec!["Test Bracelet"]);
        assert_eq!(outcome.removed, vec!["Test Bracelet"]);
        assert!(outcome.added.is_empty());
        assert_eq!(count_item_name(&outcome.merged_text, "Test Bracelet"), 1);
        assert!(has_assignment_line(&outcome.merged_text, "Armor", 100));
        assert!(!has_assignment_line(&outcome.merged_text, "Armor", 200));
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
        let outcome = merge_ic(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");
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

        let outcome = merge_ic(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");
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

        let outcome = merge_ic(Some(&prev), &incoming, ForceMode::NoForce).expect("must merge");
        assert_eq!(outcome.preserved, vec!["Test Helm"]);
        assert!(outcome.overwritten.is_empty());
        assert!(
            has_assignment_line(&outcome.merged_text, "Armor", 999),
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

        let outcome = merge_ic(
            Some(&prev),
            &incoming,
            force_with(vec![(PromptCategory::Overwrite, PromptAnswer::Yes)]),
        )
        .expect("must merge");
        assert_eq!(outcome.overwritten, vec!["Test Helm"]);
        assert!(has_assignment_line(&outcome.merged_text, "Armor", 100));
        assert!(!has_assignment_line(&outcome.merged_text, "Armor", 999));
    }

    #[test]
    fn merge_force_no_keeps_previous() {
        let db = fixture_db();
        let prev_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 999)])]);
        let (prev, _) = resolve_toml_str(&prev_in, &db).expect("resolve prev");
        let inc_in = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let (incoming, _) = resolve_toml_str(&inc_in, &db).expect("resolve incoming");

        let outcome = merge_ic(
            Some(&prev),
            &incoming,
            force_with(vec![(PromptCategory::Overwrite, PromptAnswer::No)]),
        )
        .expect("must merge");
        assert_eq!(outcome.preserved, vec!["Test Helm"]);
        assert!(outcome.overwritten.is_empty());
        assert!(has_assignment_line(&outcome.merged_text, "Armor", 999));
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
        let outcome = merge_ic(
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
        let outcome = merge_ic(Some(&prev), &incoming, force_with(vec![]))
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

        let outcome = merge_ic(
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

        let outcome = merge_ic(
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
        let outcome = merge_ic(None, &incoming, ForceMode::NoForce).expect("merge");
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

        let outcome = merge_ic(Some(prev), &incoming, ForceMode::NoForce).expect("must merge");
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
    fn make_doc_with_meta(character: &str, class: &str, items: &[TestItem<'_>]) -> String {
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
        let outcome = merge_ic(None, &resolved, ForceMode::NoForce).expect("merge");
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
        let first = merge_ic(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;

        // Second run — merge back in.
        let second = merge_ic(Some(&first), &resolved, ForceMode::NoForce)
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
        assert_eq!(
            strip_generated_timestamp_comment(&first),
            strip_generated_timestamp_comment(&second),
            "metadata carry-forward must not disturb repeat-merge serialization apart from timestamp"
        );
    }

    #[test]
    fn merge_base_stat_pass_through_is_idempotent() {
        let db = fixture_db();
        let base_stats: HashMap<Stat, i64> = [
            (Stat::Might, 5300),
            (Stat::Agility, 2650),
            (Stat::Vitality, 10200),
            (Stat::Will, 7950),
            (Stat::Fate, 4000),
        ]
        .into_iter()
        .collect();
        let bookmarklet = make_doc(&[(
            "Test Helm",
            "Unknown",
            &[("Armor", 100), ("Might", 9), ("Vitality", 3434)],
        )]);
        let (resolved, _) = resolve_toml_str_with_metadata(
            &bookmarklet,
            &db,
            Some("Thalya"),
            "Lore-master",
            &base_stats,
        )
        .expect("resolve");

        let first = merge_ic(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;
        let second = merge_ic(Some(&first), &resolved, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        let third = merge_ic(Some(&second), &resolved, ForceMode::NoForce)
            .expect("third merge")
            .merged_text;

        for (label, out) in [("first", &first), ("second", &second), ("third", &third)] {
            assert!(
                has_assignment_line(out, "Might", 9) && has_assignment_line(out, "Vitality", 3434),
                "{} merge must pass per-item base stats through verbatim:\n{}",
                label,
                out
            );
            assert_header_note_immediately_after(out, "[InnateStats]", INNATE_STATS_NOTE);
            assert_header_note_immediately_after(out, "[Virtues]", VIRTUES_NOTE);
        }
        assert_eq!(
            strip_generated_timestamp_comment(&first),
            strip_generated_timestamp_comment(&second),
            "base-stat pass-through must merge idempotently:\n--- first ---\n{}\n--- second ---\n{}",
            first,
            second
        );
        assert_eq!(
            strip_generated_timestamp_comment(&second),
            strip_generated_timestamp_comment(&third),
            "third merge must also be identical apart from timestamp"
        );
    }

    #[test]
    fn merge_refreshes_innate_base_stats_and_drops_extra_keys() {
        let db = fixture_db();
        let bookmarklet = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);

        let stale_base: HashMap<Stat, i64> = [(Stat::Might, 100)].into_iter().collect();
        let (resolved_stale, _) = resolve_toml_str_with_metadata(
            &bookmarklet,
            &db,
            Some("Thalya"),
            "Lore-master",
            &stale_base,
        )
        .expect("resolve");
        let first = merge_ic(None, &resolved_stale, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;

        // Simulate a stale canonical file carrying an invalid extra key inside
        // [InnateStats].
        let hand_edited = first.replace(
            "[InnateStats]\n# Extracted by in-game plugin; do not edit.\n",
            "[InnateStats]\n# Extracted by in-game plugin; do not edit.\nCriticalRating = 25\n",
        );
        assert_ne!(hand_edited, first, "hand-edit must apply");

        let fresh_base: HashMap<Stat, i64> = [(Stat::Might, 5300), (Stat::Fate, 4000)]
            .into_iter()
            .collect();
        let (resolved_fresh, _) = resolve_toml_str_with_metadata(
            &bookmarklet,
            &db,
            Some("Thalya"),
            "Lore-master",
            &fresh_base,
        )
        .expect("resolve");
        let merged = merge_ic(Some(&hand_edited), &resolved_fresh, ForceMode::NoForce)
            .expect("merge")
            .merged_text;

        let doc: DocumentMut = merged.parse().expect("merged output parses");
        let innate = doc
            .get("InnateStats")
            .and_then(|item| item.as_table())
            .expect("InnateStats table exists");
        let keys: Vec<&str> = innate.iter().map(|(key, _)| key).collect();
        assert_eq!(
            keys,
            vec!["Might", "Agility", "Vitality", "Will", "Fate"],
            "InnateStats must be normalized back to the five Base stats:\n{}",
            merged
        );
        assert!(
            innate.get("CriticalRating").is_none(),
            "invalid extra InnateStats key must be removed:\n{}",
            merged
        );
        assert_eq!(
            innate.get("Might").and_then(|item| item.as_integer()),
            Some(5300),
            "base keys must refresh to current export truth:\n{}",
            merged
        );
        assert_eq!(
            innate.get("Agility").and_then(|item| item.as_integer()),
            Some(0),
            "missing base keys must be written as zero:\n{}",
            merged
        );
        assert_eq!(
            innate.get("Vitality").and_then(|item| item.as_integer()),
            Some(0),
            "missing base keys must be written as zero:\n{}",
            merged
        );
        assert_eq!(
            innate.get("Will").and_then(|item| item.as_integer()),
            Some(0),
            "missing base keys must be written as zero:\n{}",
            merged
        );
        assert_eq!(
            innate.get("Fate").and_then(|item| item.as_integer()),
            Some(4000),
            "base keys must refresh to current export truth:\n{}",
            merged
        );
        assert_header_note_immediately_after(&merged, "[InnateStats]", INNATE_STATS_NOTE);
    }

    #[test]
    fn merge_preserves_existing_virtue_values_and_restores_missing_fields() {
        let db = fixture_db();
        let bookmarklet = make_doc(&[("Test Helm", "Unknown", &[("Armor", 100)])]);
        let base_stats: HashMap<Stat, i64> = [(Stat::Might, 5300)].into_iter().collect();
        let (resolved, _) = resolve_toml_str_with_metadata(
            &bookmarklet,
            &db,
            Some("Thalya"),
            "Lore-master",
            &base_stats,
        )
        .expect("resolve");
        let first = merge_ic(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;

        let hand_edited = first.replace(
            "[Virtues]\n# Not extracted, you must add these yourself.\nVirtue1            = \"\"\nVirtue2            = \"\"\nVirtue3            = \"\"\nVirtue4            = \"\"\nVirtue5            = \"\"\n",
            "[Virtues]\n# Not extracted, you must add these yourself.\nVirtue1            = \"Wisdom\"\nVirtue3            = \" Zeal \"\nVirtue5            = \"Honour\"\n",
        );
        assert_ne!(hand_edited, first, "hand-edit must apply");

        let merged = merge_ic(Some(&hand_edited), &resolved, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        let doc: DocumentMut = merged.parse().expect("merged output parses");
        let virtues = doc
            .get(VIRTUE_TABLE_KEY)
            .and_then(|item| item.as_table())
            .expect("Virtues table exists");
        assert_eq!(
            virtues.get("Virtue1").and_then(|item| item.as_str()),
            Some("Wisdom")
        );
        assert_eq!(
            virtues.get("Virtue3").and_then(|item| item.as_str()),
            Some(" Zeal ")
        );
        assert_eq!(
            virtues.get("Virtue5").and_then(|item| item.as_str()),
            Some("Honour")
        );
        assert_eq!(
            virtues.get("Virtue2").and_then(|item| item.as_str()),
            Some("")
        );
        assert_eq!(
            virtues.get("Virtue4").and_then(|item| item.as_str()),
            Some("")
        );
        assert_header_note_immediately_after(&merged, "[Virtues]", VIRTUES_NOTE);
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

        let outcome =
            merge_ic(Some(&prev_resolved), &resolved, ForceMode::NoForce).expect("must merge");
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

    // ── two_handed metadata ───────────────────────────────────────────────────

    #[test]
    fn lookup_two_handed_reflects_db_flag() {
        let db = fixture_db();
        assert!(db.lookup_two_handed("Test Greatsword"));
        assert!(
            !db.lookup_two_handed("Test Sword"),
            "entry without two_handed in JSON must default false"
        );
        assert!(!db.lookup_two_handed("No Such Item"));
    }

    #[test]
    fn resolved_two_handed_item_gains_flag_after_name_before_stats() {
        let db = fixture_db();
        let input = make_doc(&[("Test Greatsword", "Unknown", &[("Armor", 100)])]);
        let (out, _) = resolve_toml_str(&input, &db).expect("must resolve");
        assert!(
            out.contains("name = \"Test Greatsword\"\ntwo_handed = true\nMorale"),
            "two_handed must sit after name and before the stat block:\n{}",
            out
        );
    }

    #[test]
    fn resolved_one_handed_item_omits_and_strips_two_handed() {
        let db = fixture_db();
        // Stale user flag on a known one-handed weapon: the DB is the source
        // of truth, so the flag must be removed, not preserved.
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
two_handed = true\n\
Armor = 100\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");
        assert!(
            !out.contains("two_handed"),
            "known one-handed item must not carry two_handed:\n{}",
            out
        );
    }

    #[test]
    fn unknown_item_preserves_user_two_handed_flag() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Main-hand\"\n\
name = \"Renamed Legendary Greatclub\"\n\
two_handed = true\n\
Armor = 100\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");
        assert!(
            out.contains("two_handed = true"),
            "hand-edited two_handed on an unknown item must survive:\n{}",
            out
        );
    }

    #[test]
    fn merge_refreshes_two_handed_into_preserved_block() {
        let db = fixture_db();
        // Previous canonical block predates two-handed support (no flag) and
        // carries hand-edited stats + essence totals that must survive.
        let prev = "\
[[item]]\n\
slot = \"Main-hand\"\n\
name = \"Test Greatsword\"\n\
Armor = 555\n\
[item.EssenceTotals]\n\
Morale = 77\n";
        let incoming = make_doc(&[("Test Greatsword", "Unknown", &[("Armor", 100)])]);
        let (resolved, _) = resolve_toml_str(&incoming, &db).expect("resolve incoming");

        let outcome = merge_ic(Some(prev), &resolved, ForceMode::NoForce).expect("must merge");
        let merged = &outcome.merged_text;
        assert!(
            merged.contains("two_handed = true"),
            "preserved block must gain the generated flag:\n{}",
            merged
        );
        assert!(
            has_assignment_line(merged, "Armor", 555),
            "hand-edited stats must be preserved:\n{}",
            merged
        );
        assert!(
            has_assignment_line(merged, "Morale", 77),
            "hand-edited essence totals must be preserved:\n{}",
            merged
        );
    }

    #[test]
    fn merge_removes_stale_two_handed_from_known_one_handed_item() {
        let db = fixture_db();
        let prev = "\
[[item]]\n\
slot = \"Main-hand\"\n\
name = \"Test Sword\"\n\
two_handed = true\n\
Armor = 555\n";
        let incoming = make_doc(&[("Test Sword", "Unknown", &[("Armor", 100)])]);
        let (resolved, _) = resolve_toml_str(&incoming, &db).expect("resolve incoming");

        let outcome = merge_ic(Some(prev), &resolved, ForceMode::NoForce).expect("must merge");
        let merged = &outcome.merged_text;
        assert!(
            !merged.contains("two_handed"),
            "stale flag on a known one-handed item must be removed:\n{}",
            merged
        );
        assert!(
            has_assignment_line(merged, "Armor", 555),
            "hand-edited stats must still be preserved:\n{}",
            merged
        );
    }

    #[test]
    fn merge_preserves_user_two_handed_on_unknown_item() {
        let db = fixture_db();
        let prev = "\
[[item]]\n\
slot = \"Main-hand\"\n\
name = \"Renamed Legendary Greatclub\"\n\
two_handed = true\n\
Armor = 555\n";
        let incoming = make_doc(&[(
            "Renamed Legendary Greatclub",
            "Main-hand",
            &[("Armor", 100)],
        )]);
        let (resolved, _) = resolve_toml_str(&incoming, &db).expect("resolve incoming");

        let outcome = merge_ic(Some(prev), &resolved, ForceMode::NoForce).expect("must merge");
        assert!(
            outcome.merged_text.contains("two_handed = true"),
            "user flag on an item not in the DB must be preserved:\n{}",
            outcome.merged_text
        );
    }

    #[test]
    fn merge_with_two_handed_flag_is_idempotent_modulo_timestamp() {
        let db = fixture_db();
        let bookmarklet = make_doc(&[
            ("Test Greatsword", "Unknown", &[("Armor", 100)]),
            ("Test Helm", "Unknown", &[("Armor", 50)]),
        ]);
        let (resolved, _) = resolve_toml_str(&bookmarklet, &db).expect("resolve");
        let first = merge_ic(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;
        let second = merge_ic(Some(&first), &resolved, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        let third = merge_ic(Some(&second), &resolved, ForceMode::NoForce)
            .expect("third merge")
            .merged_text;

        assert!(first.contains("two_handed = true"));
        assert_eq!(
            strip_generated_timestamp_comment(&first),
            strip_generated_timestamp_comment(&second)
        );
        assert_eq!(
            strip_generated_timestamp_comment(&second),
            strip_generated_timestamp_comment(&third)
        );
    }

    // ── either_hand metadata ──────────────────────────────────────────────────

    #[test]
    fn lookup_either_hand_reflects_db_flag() {
        let db = fixture_db();
        assert!(db.lookup_either_hand("Test Rune-stone"));
        assert!(
            !db.lookup_either_hand("Test Sword"),
            "entry without either_hand in JSON must default false"
        );
        assert!(!db.lookup_either_hand("No Such Item"));
    }

    #[test]
    fn resolved_either_hand_item_gains_flag_after_name_before_stats() {
        let db = fixture_db();
        let input = make_doc(&[("Test Rune-stone", "Unknown", &[("Armor", 100)])]);
        let (out, _) = resolve_toml_str(&input, &db).expect("must resolve");
        assert!(
            out.contains("name = \"Test Rune-stone\"\neither_hand = true\nMorale"),
            "either_hand must sit after name and before the stat block:\n{}",
            out
        );
        // Resolves to the canonical Off-hand slot; the flag is what makes it
        // main-hand-eligible in the optimizer.
        assert!(out.contains("slot = \"Off-hand\""), "{}", out);
    }

    #[test]
    fn resolved_non_either_hand_item_omits_and_strips_either_hand() {
        let db = fixture_db();
        // Stale user flag on a known plain off-hand item: the DB is the source
        // of truth, so the flag must be removed, not preserved.
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
either_hand = true\n\
Armor = 100\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");
        assert!(
            !out.contains("either_hand"),
            "known non-either-hand item must not carry either_hand:\n{}",
            out
        );
    }

    #[test]
    fn unknown_item_preserves_user_either_hand_flag() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Off-hand\"\n\
name = \"Renamed Legendary Rune-stone\"\n\
either_hand = true\n\
Armor = 100\n";
        let (out, _) = resolve_toml_str(input, &db).expect("must resolve");
        assert!(
            out.contains("either_hand = true"),
            "hand-edited either_hand on an unknown item must survive:\n{}",
            out
        );
    }

    #[test]
    fn merge_refreshes_either_hand_into_preserved_block() {
        let db = fixture_db();
        // Previous canonical block predates either-hand support (no flag) and
        // carries hand-edited stats + essence totals that must survive.
        let prev = "\
[[item]]\n\
slot = \"Off-hand\"\n\
name = \"Test Rune-stone\"\n\
Armor = 555\n\
[item.EssenceTotals]\n\
Morale = 77\n";
        let incoming = make_doc(&[("Test Rune-stone", "Unknown", &[("Armor", 100)])]);
        let (resolved, _) = resolve_toml_str(&incoming, &db).expect("resolve incoming");

        let outcome = merge_ic(Some(prev), &resolved, ForceMode::NoForce).expect("must merge");
        let merged = &outcome.merged_text;
        assert!(
            merged.contains("either_hand = true"),
            "preserved block must gain the generated flag:\n{}",
            merged
        );
        assert!(
            has_assignment_line(merged, "Armor", 555),
            "hand-edited stats must be preserved:\n{}",
            merged
        );
        assert!(
            has_assignment_line(merged, "Morale", 77),
            "hand-edited essence totals must be preserved:\n{}",
            merged
        );
    }

    #[test]
    fn merge_removes_stale_either_hand_from_known_plain_item() {
        let db = fixture_db();
        let prev = "\
[[item]]\n\
slot = \"Main-hand\"\n\
name = \"Test Sword\"\n\
either_hand = true\n\
Armor = 555\n";
        let incoming = make_doc(&[("Test Sword", "Unknown", &[("Armor", 100)])]);
        let (resolved, _) = resolve_toml_str(&incoming, &db).expect("resolve incoming");

        let outcome = merge_ic(Some(prev), &resolved, ForceMode::NoForce).expect("must merge");
        let merged = &outcome.merged_text;
        assert!(
            !merged.contains("either_hand"),
            "stale flag on a known non-either-hand item must be removed:\n{}",
            merged
        );
        assert!(
            has_assignment_line(merged, "Armor", 555),
            "hand-edited stats must still be preserved:\n{}",
            merged
        );
    }

    #[test]
    fn merge_preserves_user_either_hand_on_unknown_item() {
        let db = fixture_db();
        let prev = "\
[[item]]\n\
slot = \"Off-hand\"\n\
name = \"Renamed Legendary Rune-stone\"\n\
either_hand = true\n\
Armor = 555\n";
        let incoming = make_doc(&[(
            "Renamed Legendary Rune-stone",
            "Off-hand",
            &[("Armor", 100)],
        )]);
        let (resolved, _) = resolve_toml_str(&incoming, &db).expect("resolve incoming");

        let outcome = merge_ic(Some(prev), &resolved, ForceMode::NoForce).expect("must merge");
        assert!(
            outcome.merged_text.contains("either_hand = true"),
            "user flag on an item not in the DB must be preserved:\n{}",
            outcome.merged_text
        );
    }

    #[test]
    fn merge_with_either_hand_flag_is_idempotent_modulo_timestamp() {
        let db = fixture_db();
        let bookmarklet = make_doc(&[
            ("Test Rune-stone", "Unknown", &[("Armor", 100)]),
            ("Test Helm", "Unknown", &[("Armor", 50)]),
        ]);
        let (resolved, _) = resolve_toml_str(&bookmarklet, &db).expect("resolve");
        let first = merge_ic(None, &resolved, ForceMode::NoForce)
            .expect("first merge")
            .merged_text;
        let second = merge_ic(Some(&first), &resolved, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        let third = merge_ic(Some(&second), &resolved, ForceMode::NoForce)
            .expect("third merge")
            .merged_text;

        assert!(first.contains("either_hand = true"));
        assert_eq!(
            strip_generated_timestamp_comment(&first),
            strip_generated_timestamp_comment(&second)
        );
        assert_eq!(
            strip_generated_timestamp_comment(&second),
            strip_generated_timestamp_comment(&third)
        );
    }

    // -- Issue 1: comments inside [item.EssenceTotals] survive merges --

    #[test]
    fn essence_block_comment_survives_resolution_and_merges_without_duplicating() {
        let db = fixture_db();
        let essence_comment = "# 3x Vivid Essence of Critical Rating";
        let input = format!(
            "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n\
[item.EssenceTotals]\n\
{essence_comment}\n\
CriticalRating = 4200 # hand-tallied\n"
        );

        let (resolved, _) = resolve_toml_str(&input, &db).expect("resolve");
        let essence_start = resolved
            .find("[item.EssenceTotals]")
            .expect("essence block present");
        let comment_pos = resolved
            .find(essence_comment)
            .expect("essence comment must survive resolve_toml_str");
        assert!(
            comment_pos > essence_start,
            "essence comment must stay inside the essence block:\n{}",
            resolved
        );
        assert!(
            resolved.contains("# hand-tallied"),
            "essence value-suffix comment must survive resolve_toml_str:\n{}",
            resolved
        );

        // The comment must also survive a NoForce merge and must not
        // duplicate across three successive merges (idempotency tripwire).
        let incoming_input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n";
        let (incoming, _) = resolve_toml_str(incoming_input, &db).expect("resolve incoming");

        let mut previous = resolved;
        for run in 1..=3 {
            let outcome = merge_ic(Some(&previous), &incoming, ForceMode::NoForce)
                .unwrap_or_else(|e| panic!("merge {} must succeed: {}", run, e));
            let merged = outcome.merged_text;
            assert_eq!(
                merged.matches(essence_comment).count(),
                1,
                "merge {}: essence comment must survive exactly once:\n{}",
                run,
                merged
            );
            assert_eq!(
                merged
                    .matches("CriticalRating     = 4200 # hand-tallied")
                    .count(),
                1,
                "merge {}: hand-edited essence value and suffix comment must survive:\n{}",
                run,
                merged
            );
            if run > 1 {
                assert_eq!(
                    strip_generated_timestamp_comment(&previous),
                    strip_generated_timestamp_comment(&merged),
                    "merge {} must be idempotent modulo timestamp",
                    run
                );
            }
            previous = merged;
        }
    }

    // -- Trailing comments after the last key of [item.EssenceTotals] --

    #[test]
    fn essence_trailing_comment_survives_resolution_and_merges_in_position() {
        let db = fixture_db();
        let trailing_comment = "# TODO: re-check after next essence swap";
        // Document order puts the Main-hand item first so resolution must
        // regroup (Head renders before Main-hand). The commented item ends
        // last in the input file, so toml_edit parses the trailing comment
        // as document-trailing decor; without capture it would stay glued
        // to the end of the file instead of the helm's essence block.
        let input = format!(
            "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
Armor = 5\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n\
[item.EssenceTotals]\n\
CriticalRating = 4200\n\
Fate = 3\n\
{trailing_comment}\n"
        );

        let assert_comment_position = |src: &str, run: &str| {
            assert_eq!(
                src.matches(trailing_comment).count(),
                1,
                "{run}: trailing essence comment must survive exactly once:\n{src}"
            );
            let comment_pos = src.find(trailing_comment).unwrap();
            let last_essence_key_pos = src.find("Fate               = 3").unwrap_or_else(|| {
                panic!("{run}: hand-edited essence Fate line must survive:\n{src}")
            });
            assert!(
                comment_pos > last_essence_key_pos,
                "{run}: comment must render after the last essence key line:\n{src}"
            );
            let next_item_pos = src
                .find("name = \"Test Sword\"")
                .unwrap_or_else(|| panic!("{run}: Main-hand item must survive:\n{src}"));
            assert!(
                comment_pos < next_item_pos,
                "{run}: comment must render before the next [[item]] block:\n{src}"
            );
        };

        let (resolved, _) = resolve_toml_str(&input, &db).expect("resolve");
        assert_comment_position(&resolved, "resolve");
        assert!(
            has_assignment_line(&resolved, "CriticalRating", 4200),
            "hand-edited essence total must survive resolve:\n{resolved}"
        );

        let plain_input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
Armor = 5\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n";
        let (incoming, _) = resolve_toml_str(plain_input, &db).expect("resolve incoming");

        let mut previous = resolved;
        for run in 1..=3 {
            let outcome = merge_ic(Some(&previous), &incoming, ForceMode::NoForce)
                .unwrap_or_else(|e| panic!("merge {} must succeed: {}", run, e));
            let merged = outcome.merged_text;
            assert_comment_position(&merged, &format!("merge {run}"));
            if run > 1 {
                assert_eq!(
                    strip_generated_timestamp_comment(&previous),
                    strip_generated_timestamp_comment(&merged),
                    "merge {} must be idempotent modulo timestamp",
                    run
                );
            }
            previous = merged;
        }
    }

    #[test]
    fn blank_line_separated_comment_stays_with_next_item_header_across_merges() {
        let db = fixture_db();
        let header_comment = "# gearing note: swap the sword after raid night";
        // The blank line after the essence block separates the comment from
        // it: per the blank-line boundary heuristic the comment belongs to
        // the next item's header and must not be pulled into the essence
        // block above.
        let input = format!(
            "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n\
[item.EssenceTotals]\n\
CriticalRating = 4200\n\
Fate = 7\n\
\n\
{header_comment}\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
Armor = 5\n"
        );

        let assert_comment_with_next_header = |src: &str, run: &str| {
            assert_eq!(
                src.matches(header_comment).count(),
                1,
                "{run}: header comment must survive exactly once:\n{src}"
            );
            assert!(
                !src.contains(&format!("Fate               = 7\n{header_comment}")),
                "{run}: comment must not be pulled into the essence block above:\n{src}"
            );
            let comment_pos = src.find(header_comment).unwrap();
            let essence_end_pos = src.find("Fate               = 7").unwrap_or_else(|| {
                panic!("{run}: hand-edited essence Fate line must survive:\n{src}")
            });
            let next_item_pos = src
                .find("name = \"Test Sword\"")
                .unwrap_or_else(|| panic!("{run}: Main-hand item must survive:\n{src}"));
            assert!(
                comment_pos > essence_end_pos && comment_pos < next_item_pos,
                "{run}: comment must stay attached to the next item's header:\n{src}"
            );
        };

        let (resolved, _) = resolve_toml_str(&input, &db).expect("resolve");
        assert_comment_with_next_header(&resolved, "resolve");

        let plain_input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
CriticalRating = 100\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
Armor = 5\n";
        let (incoming, _) = resolve_toml_str(plain_input, &db).expect("resolve incoming");

        let mut previous = resolved;
        for run in 1..=3 {
            let outcome = merge_ic(Some(&previous), &incoming, ForceMode::NoForce)
                .unwrap_or_else(|e| panic!("merge {} must succeed: {}", run, e));
            let merged = outcome.merged_text;
            assert_comment_with_next_header(&merged, &format!("merge {run}"));
            if run > 1 {
                assert_eq!(
                    strip_generated_timestamp_comment(&previous),
                    strip_generated_timestamp_comment(&merged),
                    "merge {} must be idempotent modulo timestamp",
                    run
                );
            }
            previous = merged;
        }
    }

    #[test]
    fn essence_trailing_comment_does_not_interfere_with_family_dividers() {
        let db = fixture_db();
        let trailing_comment = "# remember: two sockets still empty";
        // Document order: Off-hand item first, then the two Main-hand items.
        // After regrouping, the commented item (Test Greatsword) is last in
        // the Main-hand family and the Off-hand item opens a new family
        // directly below it — the exact spot where push_group writes the
        // next family's divider.
        let input = format!(
            "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Rune-stone\"\n\
Armor = 9\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
Armor = 5\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Greatsword\"\n\
Armor = 7\n\
[item.EssenceTotals]\n\
CriticalRating = 4200\n\
Fate = 3\n\
{trailing_comment}\n"
        );

        let assert_dividers_and_comment = |src: &str, run: &str| {
            assert_eq!(
                src.matches(trailing_comment).count(),
                1,
                "{run}: trailing essence comment must survive exactly once:\n{src}"
            );
            for divider in ["# --- Main-hand ---", "# --- Off-hand ---"] {
                assert_eq!(
                    src.matches(divider).count(),
                    1,
                    "{run}: divider {divider} must appear exactly once:\n{src}"
                );
            }
            let comment_pos = src.find(trailing_comment).unwrap();
            let last_essence_key_pos = src.find("Fate               = 3").unwrap_or_else(|| {
                panic!("{run}: hand-edited essence Fate line must survive:\n{src}")
            });
            let next_divider_pos = src.find("# --- Off-hand ---").unwrap();
            assert!(
                comment_pos > last_essence_key_pos && comment_pos < next_divider_pos,
                "{run}: comment must stay between its essence block and the \
                 next family's divider:\n{src}"
            );
        };

        let (resolved, _) = resolve_toml_str(&input, &db).expect("resolve");
        assert_dividers_and_comment(&resolved, "resolve");

        let plain_input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Rune-stone\"\n\
Armor = 9\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n\
Armor = 5\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Greatsword\"\n\
Armor = 7\n";
        let (incoming, _) = resolve_toml_str(plain_input, &db).expect("resolve incoming");

        let mut previous = resolved;
        for run in 1..=3 {
            let outcome = merge_ic(Some(&previous), &incoming, ForceMode::NoForce)
                .unwrap_or_else(|e| panic!("merge {} must succeed: {}", run, e));
            let merged = outcome.merged_text;
            assert_dividers_and_comment(&merged, &format!("merge {run}"));
            if run > 1 {
                assert_eq!(
                    strip_generated_timestamp_comment(&previous),
                    strip_generated_timestamp_comment(&merged),
                    "merge {} must be idempotent modulo timestamp",
                    run
                );
            }
            previous = merged;
        }
    }

    // -- Issue 4: duplicate same-name pairing prefers the most-similar
    //    incoming candidate when no exact match exists --

    #[test]
    fn merge_force_duplicate_hand_edited_instance_pairs_with_most_similar_incoming() {
        // Two owned copies of "Test Bracelet": the Armor-100 instance carries
        // a hand-edited essence total; the Armor-200 instance is untouched
        // but its incoming counterpart changed (wiki update to Armor 250).
        // Incoming arrives in reversed occurrence order so the old
        // first-remaining-occurrence fallback would pair both instances with
        // the wrong counterpart.
        let prev = "\
[[item]]\n\
slot = \"Wrist\"\n\
name = \"Test Bracelet\"\n\
Armor = 100\n\
[item.EssenceTotals]\n\
CriticalRating = 999\n\
\n\
[[item]]\n\
slot = \"Wrist\"\n\
name = \"Test Bracelet\"\n\
Armor = 200\n";
        let incoming = make_doc(&[
            ("Test Bracelet", "Wrist", &[("Armor", 250)]),
            ("Test Bracelet", "Wrist", &[("Armor", 100)]),
        ]);

        // Prompt order follows prev occurrence order: first the hand-edited
        // instance (against its true Armor-100 counterpart -> keep edits),
        // then the unedited instance (against the Armor-250 update -> take).
        let outcome = merge_ic(
            Some(prev),
            &incoming,
            force_with(vec![
                (PromptCategory::Overwrite, PromptAnswer::No),
                (PromptCategory::Overwrite, PromptAnswer::Yes),
            ]),
        )
        .expect("must merge");

        assert_eq!(outcome.preserved, vec!["Test Bracelet"]);
        assert_eq!(outcome.overwritten, vec!["Test Bracelet"]);
        assert!(outcome.added.is_empty());
        assert!(outcome.removed.is_empty());
        assert_eq!(count_item_name(&outcome.merged_text, "Test Bracelet"), 2);
        assert!(
            has_assignment_line(&outcome.merged_text, "Armor", 100),
            "hand-edited instance's base data must survive:\n{}",
            outcome.merged_text
        );
        assert!(
            has_assignment_line(&outcome.merged_text, "CriticalRating", 999),
            "hand-edited essence total must survive:\n{}",
            outcome.merged_text
        );
        assert!(
            has_assignment_line(&outcome.merged_text, "Armor", 250),
            "unedited instance must be the one overwritten by its update:\n{}",
            outcome.merged_text
        );
        assert!(
            !has_assignment_line(&outcome.merged_text, "Armor", 200),
            "stale unedited data must be replaced:\n{}",
            outcome.merged_text
        );
    }

    #[test]
    fn merge_noforce_duplicate_hand_edited_instance_preserved_and_idempotent() {
        let prev = "\
[[item]]\n\
slot = \"Wrist\"\n\
name = \"Test Bracelet\"\n\
Armor = 100\n\
[item.EssenceTotals]\n\
CriticalRating = 999\n\
\n\
[[item]]\n\
slot = \"Wrist\"\n\
name = \"Test Bracelet\"\n\
Armor = 200\n";
        let incoming = make_doc(&[
            ("Test Bracelet", "Wrist", &[("Armor", 250)]),
            ("Test Bracelet", "Wrist", &[("Armor", 100)]),
        ]);

        let first = merge_ic(Some(prev), &incoming, ForceMode::NoForce).expect("first merge");
        assert_eq!(first.preserved, vec!["Test Bracelet", "Test Bracelet"]);
        assert!(first.added.is_empty());
        assert!(first.removed.is_empty());
        assert_eq!(count_item_name(&first.merged_text, "Test Bracelet"), 2);
        assert!(has_assignment_line(&first.merged_text, "Armor", 100));
        assert!(has_assignment_line(&first.merged_text, "Armor", 200));
        assert!(has_assignment_line(
            &first.merged_text,
            "CriticalRating",
            999
        ));

        let second = merge_ic(Some(&first.merged_text), &incoming, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        let third = merge_ic(Some(&second), &incoming, ForceMode::NoForce)
            .expect("third merge")
            .merged_text;
        assert_eq!(
            strip_generated_timestamp_comment(&first.merged_text),
            strip_generated_timestamp_comment(&second)
        );
        assert_eq!(
            strip_generated_timestamp_comment(&second),
            strip_generated_timestamp_comment(&third)
        );
    }

    // -- Issue 5: Unicode-equivalent names must not orphan hand-edits --

    #[test]
    fn merge_nfd_incoming_name_preserves_nfc_previous_hand_edits() {
        // NFC: 'á' is the single code point U+00E1.
        let nfc_name_str = "Keen Pristine Mad\u{e1}shi Ring";
        // NFD: 'a' followed by the combining acute accent U+0301.
        let nfd_name_str = "Keen Pristine Mada\u{301}shi Ring";
        assert_ne!(nfc_name_str, nfd_name_str);

        let prev = format!(
            "\
[[item]]\n\
slot = \"Finger\"\n\
name = \"{nfc_name_str}\"\n\
Armor = 100\n\
[item.EssenceTotals]\n\
CriticalRating = 4200\n"
        );
        let incoming = make_doc(&[(nfd_name_str, "Finger", &[("Armor", 100)])]);

        let first = merge_ic(Some(&prev), &incoming, ForceMode::NoForce).expect("first merge");
        assert_eq!(
            first.preserved,
            vec![nfc_name_str],
            "NFD incoming must match the NFC previous instance as preserved"
        );
        assert!(first.added.is_empty(), "must not re-add under the NFD name");
        assert!(
            first.removed.is_empty(),
            "must not remove the NFC previous instance"
        );
        assert!(
            first.merged_text.contains(nfc_name_str),
            "stored name must keep the preserved table's NFC bytes:\n{}",
            first.merged_text
        );
        assert!(
            !first.merged_text.contains(nfd_name_str),
            "incoming NFD bytes must not replace the preserved name:\n{}",
            first.merged_text
        );
        assert!(
            has_assignment_line(&first.merged_text, "CriticalRating", 4200),
            "hand-edited essence total must survive:\n{}",
            first.merged_text
        );

        // Idempotency across two further reruns of the same NFD export.
        let second = merge_ic(Some(&first.merged_text), &incoming, ForceMode::NoForce)
            .expect("second merge")
            .merged_text;
        let third = merge_ic(Some(&second), &incoming, ForceMode::NoForce)
            .expect("third merge")
            .merged_text;
        assert_eq!(
            strip_generated_timestamp_comment(&first.merged_text),
            strip_generated_timestamp_comment(&second)
        );
        assert_eq!(
            strip_generated_timestamp_comment(&second),
            strip_generated_timestamp_comment(&third)
        );
    }
}
