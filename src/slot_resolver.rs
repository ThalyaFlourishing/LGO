//! Slot resolver — name → canonical Slot lookup from `data/lgo_items.json`,
//! plus end-to-end `.toml` rewrite.
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

// `pub` items below are not yet called from `main.rs`; they will be wired in
// by step 5 of the resolver work (the `resolve-slots` subcommand). Until then,
// suppress dead-code warnings so the module compiles cleanly. Remove this
// attribute when step 5 lands.
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use toml_edit::{value, ArrayOfTables, DocumentMut, Table};

use crate::gear::Slot;

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
            ItemsDbError::Io { path, source } => write!(
                f,
                "Cannot read items DB '{}': {}",
                path.display(),
                source
            ),
            ItemsDbError::ParseJson { path, source } => write!(
                f,
                "Cannot parse items DB '{}': {}",
                path.display(),
                source
            ),
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
            let slot = Slot::from_json_variant(&entry.slot).ok_or_else(|| {
                ItemsDbError::UnknownSlot {
                    item_name: entry.name.clone(),
                    slot_string: entry.slot.clone(),
                }
            })?;
            by_name
                .entry(entry.name.clone())
                .or_default()
                .push(DbItem {
                    name: entry.name,
                    slot,
                });
        }

        Ok(ItemsDb { by_name })
    }

    /// Number of unique item names in the index.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// True if no items were loaded.
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
    pub outcomes: Vec<ResolutionOutcome>,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

impl Report {
    pub fn resolved_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| matches!(o, ResolutionOutcome::Resolved { .. }))
            .count()
    }
    pub fn unknown_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| matches!(o, ResolutionOutcome::Unknown { .. }))
            .count()
    }
    pub fn unknown_names(&self) -> Vec<&str> {
        self.outcomes
            .iter()
            .filter_map(|o| match o {
                ResolutionOutcome::Unknown { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect()
    }
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
                write!(f, "Cannot parse TOML in '{}': {}", path.display(), source)
            }
            ResolveError::NoItemsArray { path } => {
                write!(f, "No [[item]] entries in '{}'", path.display())
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
                let header = format!("\n# --- {} ---\n", slot_family_label(family));
                push_group(&mut new_arr, group_items, &header, &mut next_pos);
            }
        }
        if !unknowns.is_empty() {
            push_group(
                &mut new_arr,
                unknowns,
                "\n# --- Unknown (not in items DB) ---\n",
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

/// Push a slot group onto the new array of tables, prepending `header` to
/// the prefix decor of the first table so it appears as a divider comment
/// above the group. Each table is also assigned a fresh sequential
/// `position` (via `next_pos`) so the renderer emits the rebuilt array in
/// the order we constructed it, ignoring the original-source positions.
fn push_group(arr: &mut ArrayOfTables, items: Vec<Table>, header: &str, next_pos: &mut usize) {
    for (i, mut table) in items.into_iter().enumerate() {
        if i == 0 {
            let existing_prefix = table
                .decor()
                .prefix()
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            table
                .decor_mut()
                .set_prefix(format!("{}{}", header, existing_prefix));
        }
        table.set_position(*next_pos);
        *next_pos += 1;
        arr.push(table);
    }
}

// =============================================================================
// resolve_stats_file — file-level wrapper (does I/O)
// =============================================================================

/// End-to-end: read `.toml` at `path`, resolve via `db`, write a sibling
/// `*_resolved.toml`, return summary.
pub fn resolve_stats_file(path: &Path, db: &ItemsDb) -> Result<Report, ResolveError> {
    let src = fs::read_to_string(path).map_err(|e| ResolveError::IoRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    let (new_src, outcomes) = resolve_toml_str(&src, db).map_err(|e| match e {
        // Replace placeholder paths with the real input path for diagnostics.
        ResolveError::ParseToml { source, .. } => ResolveError::ParseToml {
            path: path.to_path_buf(),
            source,
        },
        ResolveError::NoItemsArray { .. } => ResolveError::NoItemsArray {
            path: path.to_path_buf(),
        },
        other => other,
    })?;

    let output_path = compute_resolved_path(path);
    fs::write(&output_path, new_src).map_err(|e| ResolveError::IoWrite {
        path: output_path.clone(),
        source: e,
    })?;

    Ok(Report {
        outcomes,
        input_path: path.to_path_buf(),
        output_path,
    })
}

/// Compute the output path for a given input path: `name.toml` →
/// `name_resolved.toml` in the same directory. The `_resolved` suffix is
/// chosen so that `find_latest_stats_file`'s lexicographic sort places the
/// resolved file *after* its source on the next optimizer run (verified by
/// `resolved_path_sorts_after_original_lexicographically` test).
fn compute_resolved_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("lgo_stats");
    let ext = input
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("toml");
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{}_resolved.{}", stem, ext))
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

        assert_eq!(db.lookup("Test Helm"),     Some(Slot::Head));
        assert_eq!(db.lookup("Test Bracelet"), Some(Slot::Wrist1));
        assert_eq!(db.lookup("Test Sword"),    Some(Slot::MainHand));
        assert_eq!(db.lookup("Test Tome"),     Some(Slot::ClassItem));
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
        let err = ItemsDb::from_json_str(bad, dummy_path())
            .expect_err("unknown slot string must error");
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
        let err = ItemsDb::from_json_str(bad, dummy_path())
            .expect_err("excluded slot must error");
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
        let db = ItemsDb::from_json_str(extra, dummy_path())
            .expect("extra fields must be tolerated");
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
            ("Test Helm",     slot_family(Slot::Head)),
            ("Test Bracelet", slot_family(Slot::Wrist1)),
            ("Test Sword",    slot_family(Slot::MainHand)),
            ("Test Tome",     slot_family(Slot::ClassItem)),
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

    // -- compute_resolved_path tests --

    #[test]
    fn compute_resolved_path_appends_resolved_before_extension() {
        let p = compute_resolved_path(Path::new(
            "/tmp/lgo_stats_Char_20260101_000000.toml",
        ));
        assert_eq!(
            p,
            PathBuf::from("/tmp/lgo_stats_Char_20260101_000000_resolved.toml")
        );
    }

    /// Verifies the §7 / §11 design assumption: the resolved file's name
    /// sorts lexicographically *after* its source, so that
    /// `find_latest_stats_file` in `gearstats.rs` will pick the resolved
    /// version on the next optimizer run.
    #[test]
    fn resolved_path_sorts_after_original_lexicographically() {
        let original = "lgo_stats_Char_20260101_000000.toml";
        let resolved = compute_resolved_path(Path::new(original))
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert!(
            resolved.as_str() > original,
            "{} must sort after {}",
            resolved,
            original
        );
    }

    // -- Report tests --

    #[test]
    fn report_counts_resolved_and_unknown_correctly() {
        let db = fixture_db();
        let input = "\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Helm\"\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Mystery Item\"\n\
\n\
[[item]]\n\
slot = \"Unknown\"\n\
name = \"Test Sword\"\n";
        let (_, outcomes) = resolve_toml_str(input, &db).expect("must resolve");
        let report = Report {
            outcomes,
            input_path: PathBuf::from("<test>"),
            output_path: PathBuf::from("<test_resolved>"),
        };
        assert_eq!(report.resolved_count(), 2);
        assert_eq!(report.unknown_count(), 1);
        assert_eq!(report.unknown_names(), vec!["Mystery Item"]);
    }
}
