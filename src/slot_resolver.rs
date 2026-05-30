//! Slot resolver — name → canonical Slot lookup from `data/lgo_items.json`.
//!
//! The bookmarklet writes correct item *names* but cannot reliably determine
//! *slots* (the wiki has no enforced slot allow-list, and weapons carry their
//! slot in a different template field). This module provides an offline
//! name → Slot index sourced from the canonical game data dump.
//!
//! See `docs/RESOLVER_DESIGN.md` for the overall design and decisions.

// `pub` items below are not yet called from `main.rs`; they will be wired in
// by step 5 of the resolver work (the `resolve-slots` subcommand). Until then,
// suppress dead-code warnings so step 3 compiles cleanly. Remove this attribute
// when step 5 lands.
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::gear::Slot;

/// Default path to the offline items DB, relative to the working directory.
pub const DEFAULT_ITEMS_DB_PATH: &str = "data/lgo_items.json";

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
/// about stats — only slots. We let `serde` ignore unknown fields by default,
/// which means the `stats` object is silently skipped.
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

    #[test]
    fn loads_fixture_and_resolves_known_items() {
        let db = ItemsDb::from_json_str(FIXTURE, dummy_path()).expect("fixture must parse");
        assert_eq!(db.len(), 4);
        assert!(!db.is_empty());

        assert_eq!(db.lookup("Test Helm"),     Some(Slot::Head));
        assert_eq!(db.lookup("Test Bracelet"), Some(Slot::Wrist1));
        assert_eq!(db.lookup("Test Sword"),    Some(Slot::MainHand));
        assert_eq!(db.lookup("Test Tome"),     Some(Slot::ClassItem));
    }

    #[test]
    fn lookup_returns_none_for_unknown_name() {
        let db = ItemsDb::from_json_str(FIXTURE, dummy_path()).expect("fixture must parse");
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
        // CraftItem and Bridle are valid Slot enum names in db_build's old
        // mapping but are *excluded* by gear.rs::Slot. If the JSON ever
        // contains them, the resolver should refuse rather than silently
        // ignore — they shouldn't be candidates for the optimizer.
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
        // Defensive: names round-trip through the plugin and the bookmarklet
        // unchanged, so case should match exactly. Lowercased / uppercased
        // queries should miss, not partial-match.
        let db = ItemsDb::from_json_str(FIXTURE, dummy_path()).expect("fixture must parse");
        assert_eq!(db.lookup("test helm"), None);
        assert_eq!(db.lookup("TEST HELM"), None);
        assert_eq!(db.lookup("Test Helm"), Some(Slot::Head));
    }

    #[test]
    fn extra_fields_in_entry_are_ignored() {
        // The real file has a `stats` field which is not in our RawEntry.
        // Confirm serde silently ignores unknown fields rather than failing.
        // Also adds a hypothetical future field to prove the pattern holds.
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
}
