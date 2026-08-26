//! TOML gear stats file reader.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::gear::{GearItem, Slot};
use crate::stat::{Stat, BASE_STATS, TRACKED_STATS};

const ESSENCE_TOTALS_KEY: &str = "EssenceTotals";

/// The parsed contents of a gear stats TOML file, including any top-level
/// metadata and the list of items.
#[derive(Debug)]
pub struct GearDoc {
    /// Character name, if present as `character = "..."` at top level.
    pub character: Option<String>,
    /// Character class, if present as `class = "..."` at top level.
    pub class: Option<String>,
    /// Tracked-stat totals a user hand-added to top-level `[InnateStats]`.
    /// Raw Base-stat keys never land here — Base stats must not seed any
    /// tracked total; they go to `innate_base_stats` for derivation instead.
    pub innate_stats: HashMap<Stat, i64>,
    /// The five raw Base-stat values from top-level `[InnateStats]`
    /// (Might/Agility/Vitality/Will/Fate). Derivation inputs only: the
    /// optimize path converts them into tracked-stat contributions via
    /// per-class coefficients before optimization.
    pub innate_base_stats: HashMap<Stat, i64>,
    /// Parsed gear items from `[[item]]` entries.
    pub items: Vec<DocItem>,
}

/// One `[[item]]` entry as read from the TOML: the optimizer-facing item plus
/// its raw Base-stat values, kept apart from the tracked-stat map so Base
/// stats can never leak raw into any tracked total.
#[derive(Debug)]
pub struct DocItem {
    /// The optimizer-facing item (tracked stats only).
    pub item: GearItem,
    /// Raw Base-stat values from the item block, with `[item.EssenceTotals]`
    /// base values merged in — derivation inputs for the optimize pre-pass.
    pub base_stats: HashMap<Stat, i64>,
}

/// Parse a TOML gear stats file back into a `GearDoc` carrying top-level
/// metadata (`character`, `class`) and a list of items.
///
/// Only non-zero stat values are stored in each item's stats map,
/// consistent with how the optimizer treats missing stats as 0.
pub fn read_stats_file(path: &Path) -> Result<GearDoc, String> {
    let src = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read gear stats file {}: {}", path.display(), e))?;

    let doc: toml::Value = src
        .parse()
        .map_err(|e| format!("Malformed TOML in {}: {}", path.display(), e))?;

    let items_arr = doc
        .get("item")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("No [[item]] entries found in {}", path.display()))?;

    let mut items = Vec::new();

    for (idx, entry) in items_arr.iter().enumerate() {
        let entry_table = entry
            .as_table()
            .ok_or_else(|| format!("[[item]] #{} must be a TOML table", idx + 1))?;
        let slot_str = entry_table
            .get("slot")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("[[item]] #{} missing 'slot'", idx + 1))?;

        let name = entry_table
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("[[item]] #{} missing 'name'", idx + 1))?
            .to_string();

        validate_item_keys(entry_table, &name)?;

        let two_handed = match entry_table.get("two_handed") {
            None => false,
            Some(v) => v.as_bool().ok_or_else(|| {
                format!(
                    "Key `two_handed` for item `{}` must be a boolean (true/false).",
                    name
                )
            })?,
        };

        let slot = match parse_slot_str(slot_str) {
            Some(s) => s,
            None => {
                // "Unknown" is the bookmarklet's explicit marker for items it
                // couldn't resolve (no wiki page, disambiguation, etc.).
                // These are dropped silently because the user already saw the
                // UNRESOLVED/AUTO-PICKED comment in the TOML; no extra noise.
                // Any other unrecognised string (e.g. "Bridle", "tool") gets a
                // warning so the user knows the item was skipped.
                if slot_str != "Unknown" {
                    eprintln!(
                        "Warning: skipping \"{}\": slot \"{}\" is not optimizer-relevant",
                        name, slot_str
                    );
                }
                continue;
            }
        };

        let mut stats = read_stats_map(entry_table, TRACKED_STATS);
        let mut base_stats = read_stats_map(entry_table, BASE_STATS);
        if let Some(essence_totals) = entry_table.get(ESSENCE_TOTALS_KEY) {
            let essence_table = essence_totals.as_table().ok_or_else(|| {
                format!(
                    "`{}` for item `{}` must be a TOML table",
                    ESSENCE_TOTALS_KEY, name
                )
            })?;
            validate_essence_keys(essence_table, &name)?;
            for (stat, value) in read_stats_map(essence_table, TRACKED_STATS) {
                *stats.entry(stat).or_insert(0) += value;
            }
            for (stat, value) in read_stats_map(essence_table, BASE_STATS) {
                *base_stats.entry(stat).or_insert(0) += value;
            }
            // Runtime item stat maps store only non-zero effective totals; if
            // base and essence values cancel out, absence still means zero.
            stats.retain(|_, value| *value != 0);
            base_stats.retain(|_, value| *value != 0);
        }

        items.push(DocItem {
            item: GearItem {
                name,
                slot,
                two_handed,
                stats,
            },
            base_stats,
        });
    }

    let character = doc
        .get("character")
        .and_then(|v| v.as_str())
        .map(String::from);
    let class = doc.get("class").and_then(|v| v.as_str()).map(String::from);
    if let Some(innate) = doc.get("InnateStats") {
        let innate_table = innate
            .as_table()
            .ok_or_else(|| "`InnateStats` must be a TOML table".to_string())?;
        validate_innate_keys(innate_table)?;
    }
    let innate_stats = read_innate_stats(&doc, TRACKED_STATS);
    let innate_base_stats = read_innate_stats(&doc, BASE_STATS);

    Ok(GearDoc {
        character,
        class,
        innate_stats,
        innate_base_stats,
        items,
    })
}

fn read_innate_stats(doc: &toml::Value, keys: &[(Stat, &str)]) -> HashMap<Stat, i64> {
    let Some(table) = doc.get("InnateStats").and_then(|v| v.as_table()) else {
        return HashMap::new();
    };
    read_stats_map(table, keys)
}

/// Read the non-zero values for `keys` (a `(Stat, TOML key)` table such as
/// `TRACKED_STATS` or `BASE_STATS`) out of a TOML table.
fn read_stats_map(table: &toml::value::Table, keys: &[(Stat, &str)]) -> HashMap<Stat, i64> {
    let mut stats = HashMap::new();
    for (stat, key) in keys {
        if let Some(val) = table.get(*key).and_then(|v| v.as_integer()) {
            if val != 0 {
                stats.insert(*stat, val);
            }
        }
    }
    stats
}

fn is_tracked_stat_key(key: &str) -> bool {
    TRACKED_STATS.iter().any(|(_, stat_key)| *stat_key == key)
}

fn is_base_stat_key(key: &str) -> bool {
    BASE_STATS.iter().any(|(_, stat_key)| *stat_key == key)
}

/// A key allowed wherever stat values live: the 16 tracked stats plus the
/// five raw Base stats the resolver passes through. Anything else is a typo
/// and must hard-error rather than silently dropping user data.
fn is_allowed_stat_key(key: &str) -> bool {
    is_tracked_stat_key(key) || is_base_stat_key(key)
}

fn validate_item_keys(table: &toml::value::Table, item_name: &str) -> Result<(), String> {
    for key in table.keys() {
        if key == "slot"
            || key == "name"
            || key == "two_handed"
            || key == ESSENCE_TOTALS_KEY
            || is_allowed_stat_key(key)
        {
            continue;
        }
        return Err(format!(
            "Unknown stat key `{}` in `item` for item `{}`.",
            key, item_name
        ));
    }
    Ok(())
}

fn validate_essence_keys(table: &toml::value::Table, item_name: &str) -> Result<(), String> {
    for key in table.keys() {
        if is_allowed_stat_key(key) {
            continue;
        }
        return Err(format!(
            "Unknown stat key `{}` in `EssenceTotals` for item `{}`.",
            key, item_name
        ));
    }
    Ok(())
}

fn validate_innate_keys(table: &toml::value::Table) -> Result<(), String> {
    for key in table.keys() {
        if is_allowed_stat_key(key) {
            continue;
        }
        return Err(format!("Unknown stat key `{}` in `InnateStats`.", key));
    }
    Ok(())
}

/// Scan `dir` for a file whose name matches `<prefix><X><suffix>` where
/// `X` equals `char_segment` case-insensitively (ASCII).  The prefix and
/// suffix are also matched case-insensitively so that user-typed or
/// Windows-filesystem-typed filenames are handled uniformly.
///
/// Returns:
/// - `Ok(Some(path))` — exactly one match (the on-disk path, preserving
///   whatever casing the filesystem has)
/// - `Ok(None)` — no match
/// - `Err(msg)` — two or more matches (collision); message names both files
///   and tells the user what to do
fn find_case_insensitive_char_file(
    dir: &Path,
    prefix: &str,
    char_segment: &str,
    suffix: &str,
) -> Result<Option<PathBuf>, String> {
    let prefix_lower = prefix.to_ascii_lowercase();
    let suffix_lower = suffix.to_ascii_lowercase();
    let char_lower = char_segment.to_ascii_lowercase();

    let matches: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory {}: {}", dir.display(), e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = match p.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => return false,
            };
            let name_lower = name.to_ascii_lowercase();
            if !name_lower.starts_with(&prefix_lower) || !name_lower.ends_with(&suffix_lower) {
                return false;
            }
            let mid_start = prefix_lower.len();
            let mid_end = name_lower.len() - suffix_lower.len();
            if mid_start > mid_end {
                return false;
            }
            name_lower[mid_start..mid_end] == char_lower
        })
        .collect();

    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches.into_iter().next().unwrap())),
        _ => {
            let mut names: Vec<String> = matches
                .iter()
                .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
                .collect();
            names.sort();
            Err(format!(
                "ambiguous character-name files found: {} — please delete one",
                names.join(", ")
            ))
        }
    }
}

/// Find a `lgo_<X>_gearReady.toml` file in `dir` where `X` matches `character`
/// case-insensitively.  Used by both the optimizer and the resolver.
///
/// Returns `Ok(Some(path))`, `Ok(None)`, or `Err(collision_message)`.
pub fn find_canonical_gear_file(dir: &Path, character: &str) -> Result<Option<PathBuf>, String> {
    find_case_insensitive_char_file(dir, "lgo_", character, "_gearReady.toml")
}

/// Find the bookmarklet output for `character` (`lgo_<X>_gearStats.toml`),
/// matched case-insensitively on both prefix/suffix and character segment.
///
/// The resolver reads this file *exactly* — no scanning over other patterns,
/// no lex-latest fallback.
///
/// Returns `Ok(Some(path))`, `Ok(None)`, or `Err(collision_message)`.
pub fn find_bookmarklet_output(dir: &Path, character: &str) -> Result<Option<PathBuf>, String> {
    find_case_insensitive_char_file(dir, "lgo_", character, "_gearStats.toml")
}

/// Parse a slot display string back to a Slot variant.
/// Must match the Display impl in gear.rs exactly.
pub fn parse_slot_display(s: &str) -> Option<Slot> {
    parse_slot_str(s)
}

/// Parse a slot display string back to a Slot variant.
/// Must match the Display impl in gear.rs exactly.
fn parse_slot_str(s: &str) -> Option<Slot> {
    match s {
        "Head" => Some(Slot::Head),
        "Chest" => Some(Slot::Chest),
        "Legs" => Some(Slot::Legs),
        "Hands" => Some(Slot::Hands),
        "Feet" => Some(Slot::Feet),
        "Shoulders" => Some(Slot::Shoulders),
        "Back" => Some(Slot::Back),
        "Wrist" => Some(Slot::Wrist1),
        "Neck" => Some(Slot::Neck),
        "Finger" => Some(Slot::Finger1),
        "Ear" => Some(Slot::Ear1),
        "Pocket" => Some(Slot::Pocket),
        "Main-hand" => Some(Slot::MainHand),
        "Off-hand" => Some(Slot::OffHand),
        "Ranged" => Some(Slot::Ranged),
        "Class Item" => Some(Slot::ClassItem),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "lgo_gearstats_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn find_bookmarklet_output_returns_exact_filename() {
        let dir = make_test_dir();

        // Decoy lgo_stats_*.toml — must NOT be returned.
        std::fs::write(dir.join("lgo_stats_Thalya_20260101_000000.toml"), "").expect("write decoy");
        assert!(
            find_bookmarklet_output(&dir, "Thalya")
                .expect("no error")
                .is_none(),
            "decoy timestamped file must not be picked up"
        );

        let target = dir.join("lgo_Thalya_gearStats.toml");
        std::fs::write(&target, "").expect("write target");
        assert_eq!(
            find_bookmarklet_output(&dir, "Thalya").expect("no error"),
            Some(target)
        );

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    // ── New case-insensitive tests ────────────────────────────────────────────

    #[test]
    fn find_canonical_gear_file_case_insensitive_lowercase() {
        let dir = make_test_dir();
        let f = dir.join("lgo_thalya_gearReady.toml");
        std::fs::write(&f, "").expect("write file");

        let found = find_canonical_gear_file(&dir, "Thalya")
            .expect("no error")
            .expect("must find file");
        assert_eq!(found, f);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn find_canonical_gear_file_case_insensitive_uppercase() {
        let dir = make_test_dir();
        let f = dir.join("lgo_THALYA_gearReady.toml");
        std::fs::write(&f, "").expect("write file");

        let found = find_canonical_gear_file(&dir, "thalya")
            .expect("no error")
            .expect("must find file");
        assert_eq!(found, f);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn find_bookmarklet_output_finds_lowercase_stats_file_for_mixed_case_query() {
        let dir = make_test_dir();
        let f = dir.join("lgo_thalya_gearStats.toml");
        std::fs::write(&f, "").expect("write file");

        let found = find_bookmarklet_output(&dir, "Thalya")
            .expect("no error")
            .expect("must find file");
        assert_eq!(found, f);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn find_bookmarklet_output_finds_mixed_case_stats_file_for_lowercase_query() {
        let dir = make_test_dir();
        let f = dir.join("lgo_Thalya_gearStats.toml");
        std::fs::write(&f, "").expect("write file");

        let found = find_bookmarklet_output(&dir, "thalya")
            .expect("no error")
            .expect("must find file");
        assert_eq!(found, f);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    // ── read_stats_file: non-canonical slot handling ──────────────────────────

    #[test]
    fn read_stats_file_silently_skips_unknown_slot() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Head"
name = "Good Helm"

[[item]]
slot = "Unknown"
name = "Mystery Item"
"#;
        std::fs::write(&path, toml).expect("write toml");
        let result = read_stats_file(&path).expect("must return Ok");
        assert_eq!(
            result.items.len(),
            1,
            "Unknown slot must be silently skipped"
        );
        assert_eq!(result.items[0].item.slot, crate::gear::Slot::Head);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_skips_bridle_slot_with_warning() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Head"
name = "Good Helm"

[[item]]
slot = "Bridle"
name = "Scholar's Light Bridle"
"#;
        std::fs::write(&path, toml).expect("write toml");
        let result = read_stats_file(&path).expect("must return Ok");
        assert_eq!(result.items.len(), 1, "Bridle slot must be skipped");
        assert_eq!(result.items[0].item.slot, crate::gear::Slot::Head);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_skips_tool_slot_variants() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Head"
name = "Good Helm"

[[item]]
slot = "tool"
name = "Craft Tool A"

[[item]]
slot = "Tool"
name = "Craft Tool B"

[[item]]
slot = "Craft Tool"
name = "Craft Tool C"
"#;
        std::fs::write(&path, toml).expect("write toml");
        let result = read_stats_file(&path).expect("must return Ok");
        assert_eq!(
            result.items.len(),
            1,
            "all tool slot variants must be skipped"
        );
        assert_eq!(result.items[0].item.slot, crate::gear::Slot::Head);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_parses_all_19_canonical_slots() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let mut toml = String::new();
        for slot in crate::gear::Slot::ALL {
            toml.push_str(&format!(
                "\n[[item]]\nslot = \"{}\"\nname = \"Item for {}\"\n",
                slot, slot
            ));
        }
        // Add one Unknown and one Bridle — both must be skipped.
        toml.push_str("\n[[item]]\nslot = \"Unknown\"\nname = \"Mystery\"\n");
        toml.push_str("\n[[item]]\nslot = \"Bridle\"\nname = \"A Bridle\"\n");
        std::fs::write(&path, toml).expect("write toml");
        let result = read_stats_file(&path).expect("must return Ok");
        assert_eq!(
            result.items.len(),
            crate::gear::Slot::ALL.len(),
            "all internal slots' external display strings must parse; Unknown and Bridle must be skipped"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_rejects_legacy_numbered_pooled_slots() {
        assert!(parse_slot_display("Wrist (1)").is_none());
        assert!(parse_slot_display("Wrist (2)").is_none());
        assert!(parse_slot_display("Finger (1)").is_none());
        assert!(parse_slot_display("Finger (2)").is_none());
        assert!(parse_slot_display("Ear (1)").is_none());
        assert!(parse_slot_display("Ear (2)").is_none());
    }

    #[test]
    fn read_stats_file_errors_on_missing_name() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = "[[item]]\nslot = \"Head\"\n";
        std::fs::write(&path, toml).expect("write toml");
        let result = read_stats_file(&path);
        assert!(result.is_err(), "missing name must return Err");
        assert!(
            result.unwrap_err().contains("missing 'name'"),
            "error must mention missing 'name'"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_extracts_character_and_class() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
character          = "Thalya"
class              = "Lore-master"

[[item]]
slot = "Head"
name = "Test Helm"
"#;
        std::fs::write(&path, toml).expect("write toml");
        let doc = read_stats_file(&path).expect("must return Ok");
        assert_eq!(doc.character.as_deref(), Some("Thalya"));
        assert_eq!(doc.class.as_deref(), Some("Lore-master"));
        assert!(doc.innate_stats.is_empty());
        assert_eq!(doc.items.len(), 1);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_returns_none_for_absent_metadata() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = "[[item]]\nslot = \"Head\"\nname = \"Test Helm\"\n";
        std::fs::write(&path, toml).expect("write toml");
        let doc = read_stats_file(&path).expect("must return Ok");
        assert!(
            doc.character.is_none(),
            "character must be None when absent"
        );
        assert!(doc.class.is_none(), "class must be None when absent");
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_extracts_innate_stats_with_morale_and_power() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[InnateStats]
Morale = 100
Power = 50
CriticalRating = 25
Might = 999

[[item]]
slot = "Head"
name = "Test Helm"
"#;
        std::fs::write(&path, toml).expect("write toml");
        let doc = read_stats_file(&path).expect("must return Ok");
        assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&100));
        assert_eq!(doc.innate_stats.get(&Stat::Power), Some(&50));
        assert_eq!(doc.innate_stats.get(&Stat::CriticalRating), Some(&25));
        assert!(!doc.innate_stats.contains_key(&Stat::Might));
        assert_eq!(doc.innate_base_stats.get(&Stat::Might), Some(&999));
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_merges_full_essence_totals() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Head"
name = "Essenced Helm"
CriticalRating = 100
Finesse = 10
[item.EssenceTotals]
CriticalRating = 25
Finesse = 5
"#;
        std::fs::write(&path, toml).expect("write toml");
        let doc = read_stats_file(&path).expect("must parse");
        assert_eq!(doc.items[0].item.stat(&Stat::CriticalRating), 125);
        assert_eq!(doc.items[0].item.stat(&Stat::Finesse), 15);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_treats_missing_essence_totals_as_zero() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Head"
name = "Plain Helm"
CriticalRating = 100
"#;
        std::fs::write(&path, toml).expect("write toml");
        let doc = read_stats_file(&path).expect("must parse");
        assert_eq!(doc.items[0].item.stat(&Stat::CriticalRating), 100);
        assert_eq!(doc.items[0].item.stat(&Stat::Finesse), 0);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_treats_partial_essence_totals_as_zero_for_omissions() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Head"
name = "Partly Essenced Helm"
CriticalRating = 100
[item.EssenceTotals]
Finesse = 5
"#;
        std::fs::write(&path, toml).expect("write toml");
        let doc = read_stats_file(&path).expect("must parse");
        assert_eq!(doc.items[0].item.stat(&Stat::CriticalRating), 100);
        assert_eq!(doc.items[0].item.stat(&Stat::Finesse), 5);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_errors_on_unknown_essence_key() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Head"
name = "Typo Helm"
[item.EssenceTotals]
# Deliberately misspelled to verify unknown essence stat keys are hard errors.
CritcalRating = 25
"#;
        std::fs::write(&path, toml).expect("write toml");
        let err = read_stats_file(&path).expect_err("unknown essence key must fail");
        assert!(err.contains("CritcalRating"));
        assert!(err.contains("EssenceTotals"));
        assert!(err.contains("Typo Helm"));
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_errors_on_unknown_base_item_key() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Head"
name = "Typo Helm"
# Deliberately misspelled to verify unknown base stat keys are hard errors.
CritcalRating = 25
"#;
        std::fs::write(&path, toml).expect("write toml");
        let err = read_stats_file(&path).expect_err("unknown base key must fail");
        assert!(err.contains("CritcalRating"));
        assert!(err.contains("item"));
        assert!(err.contains("Typo Helm"));
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_accepts_base_stat_keys_in_all_three_locations() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[InnateStats]
Might = 5300
Agility = 2650
Vitality = 10200
Will = 7950
Fate = 4000

[[item]]
slot = "Head"
name = "Base Stat Helm"
CriticalRating = 100
Might = 9
Vitality = 3434
[item.EssenceTotals]
Finesse = 5
Will = 12
"#;
        std::fs::write(&path, toml).expect("write toml");
        let doc = read_stats_file(&path).expect("base stat keys must be accepted");
        // Base values are carried in the separate base-stat maps, never in
        // any tracked total: item stat maps and innate stats stay tracked-only.
        assert_eq!(doc.items[0].item.stat(&Stat::CriticalRating), 100);
        assert_eq!(doc.items[0].item.stat(&Stat::Finesse), 5);
        assert!(!doc.items[0].item.stats.contains_key(&Stat::Might));
        assert!(!doc.items[0].item.stats.contains_key(&Stat::Vitality));
        assert!(!doc.items[0].item.stats.contains_key(&Stat::Will));
        assert_eq!(doc.items[0].base_stats.get(&Stat::Might), Some(&9));
        assert_eq!(doc.items[0].base_stats.get(&Stat::Vitality), Some(&3434));
        // EssenceTotals base values merge into the item's base-stat map.
        assert_eq!(doc.items[0].base_stats.get(&Stat::Will), Some(&12));
        assert!(doc.innate_stats.is_empty());
        assert_eq!(doc.innate_base_stats.get(&Stat::Might), Some(&5300));
        assert_eq!(doc.innate_base_stats.get(&Stat::Agility), Some(&2650));
        assert_eq!(doc.innate_base_stats.get(&Stat::Vitality), Some(&10200));
        assert_eq!(doc.innate_base_stats.get(&Stat::Will), Some(&7950));
        assert_eq!(doc.innate_base_stats.get(&Stat::Fate), Some(&4000));
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_errors_on_unknown_innate_stats_key() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[InnateStats]
Might = 5300
Wisdom = 42

[[item]]
slot = "Head"
name = "Test Helm"
"#;
        std::fs::write(&path, toml).expect("write toml");
        let err = read_stats_file(&path).expect_err("unknown InnateStats key must fail");
        assert!(err.contains("Wisdom"));
        assert!(err.contains("InnateStats"));
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_errors_on_genuinely_unknown_key_next_to_base_stats() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Head"
name = "Wise Helm"
Might = 9
Wisdom = 42
"#;
        std::fs::write(&path, toml).expect("write toml");
        let err = read_stats_file(&path).expect_err("unknown key must fail even beside base keys");
        assert!(err.contains("Wisdom"));
        assert!(err.contains("Wise Helm"));
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    // ── two_handed parsing ─────────────────────────────────────────────────────

    #[test]
    fn read_stats_file_parses_two_handed_true() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Main-hand"
name = "Example Greatsword"
two_handed = true
CriticalRating = 100
"#;
        std::fs::write(&path, toml).expect("write toml");
        let result = read_stats_file(&path).expect("must return Ok");
        assert_eq!(result.items.len(), 1);
        assert!(result.items[0].item.two_handed);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_missing_two_handed_defaults_false() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Main-hand"
name = "Example Dagger"
CriticalRating = 100
"#;
        std::fs::write(&path, toml).expect("write toml");
        let result = read_stats_file(&path).expect("must return Ok");
        assert_eq!(result.items.len(), 1);
        assert!(!result.items[0].item.two_handed);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_errors_on_non_bool_two_handed() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Main-hand"
name = "Example Greatsword"
two_handed = "yes"
"#;
        std::fs::write(&path, toml).expect("write toml");
        let err = read_stats_file(&path).expect_err("non-bool two_handed must fail");
        assert!(err.contains("two_handed"));
        assert!(err.contains("Example Greatsword"));
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn read_stats_file_rejects_two_handed_under_essence_totals() {
        let dir = make_test_dir();
        let path = dir.join("test.toml");
        let toml = r#"
[[item]]
slot = "Main-hand"
name = "Example Greatsword"
[item.EssenceTotals]
two_handed = true
"#;
        std::fs::write(&path, toml).expect("write toml");
        let err = read_stats_file(&path).expect_err("two_handed under EssenceTotals must fail");
        assert!(err.contains("two_handed"));
        assert!(err.contains("EssenceTotals"));
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }
}
