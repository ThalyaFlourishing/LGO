//! TOML gear stats file reader.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::gear::{GearItem, Slot};
use crate::stat::{Stat, TRACKED_STATS};

/// Parse a TOML gear stats file back into a list of items.
///
/// Only non-zero stat values are stored in each item's stats map,
/// consistent with how the optimizer treats missing stats as 0.
pub fn read_stats_file(path: &Path) -> Result<Vec<GearItem>, String> {
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
        let slot_str = entry
            .get("slot")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("[[item]] #{} missing 'slot'", idx + 1))?;

        let name = entry
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("[[item]] #{} missing 'name'", idx + 1))?
            .to_string();

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

        let mut stats: HashMap<Stat, i64> = HashMap::new();

        for (stat, key) in TRACKED_STATS {
            if let Some(val) = entry.get(*key).and_then(|v| v.as_integer()) {
                if val != 0 {
                    stats.insert(*stat, val);
                }
            }
        }

        items.push(GearItem { name, slot, stats });
    }

    Ok(items)
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

/// Find a `lgo_<X>_gear.toml` file in `dir` where `X` matches `character`
/// case-insensitively.  Used by both the optimizer and the resolver.
///
/// Returns `Ok(Some(path))`, `Ok(None)`, or `Err(collision_message)`.
pub fn find_canonical_gear_file(dir: &Path, character: &str) -> Result<Option<PathBuf>, String> {
    find_case_insensitive_char_file(dir, "lgo_", character, "_gear.toml")
}

/// Find the bookmarklet output for `character` (`lgo_<X>_stats.toml`),
/// matched case-insensitively on both prefix/suffix and character segment.
///
/// The resolver reads this file *exactly* — no scanning over other patterns,
/// no lex-latest fallback.
///
/// Returns `Ok(Some(path))`, `Ok(None)`, or `Err(collision_message)`.
pub fn find_bookmarklet_output(dir: &Path, character: &str) -> Result<Option<PathBuf>, String> {
    find_case_insensitive_char_file(dir, "lgo_", character, "_stats.toml")
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
        "Wrist (1)" => Some(Slot::Wrist1),
        "Wrist (2)" => Some(Slot::Wrist2),
        "Neck" => Some(Slot::Neck),
        "Finger (1)" => Some(Slot::Finger1),
        "Finger (2)" => Some(Slot::Finger2),
        "Ear (1)" => Some(Slot::Ear1),
        "Ear (2)" => Some(Slot::Ear2),
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

        let target = dir.join("lgo_Thalya_stats.toml");
        std::fs::write(&target, "").expect("write target");
        assert_eq!(
            find_bookmarklet_output(&dir, "Thalya").expect("no error"),
            Some(target)
        );

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    // ── New case-insensitive tests ────────────────────────────────────────────

    #[test]
    fn find_canonical_gear_file_finds_lowercase_gear_file_for_mixed_case_query() {
        let dir = make_test_dir();
        let f = dir.join("lgo_thalya_gear.toml");
        std::fs::write(&f, "").expect("write file");

        let found = find_canonical_gear_file(&dir, "Thalya")
            .expect("no error")
            .expect("must find file");
        assert_eq!(found, f);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn find_canonical_gear_file_finds_uppercase_gear_file_for_lowercase_query() {
        let dir = make_test_dir();
        let f = dir.join("lgo_THALYA_gear.toml");
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
        let f = dir.join("lgo_thalya_stats.toml");
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
        let f = dir.join("lgo_Thalya_stats.toml");
        std::fs::write(&f, "").expect("write file");

        let found = find_bookmarklet_output(&dir, "thalya")
            .expect("no error")
            .expect("must find file");
        assert_eq!(found, f);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn find_canonical_gear_file_case_only_duplicate_names_alias_on_windows() {
        let dir = make_test_dir();

        let first = dir.join("lgo_Thalya_gear.toml");
        let second = dir.join("lgo_thalya_gear.toml");
        std::fs::write(&first, "").expect("write 1");
        std::fs::write(&second, "").expect("write 2");

        let found = find_canonical_gear_file(&dir, "Thalya")
            .expect("case-only duplicate names should alias, not collide")
            .expect("must find file");

        assert!(
            found == first || found == second,
            "returned path must be one of the aliased spellings: {}",
            found.display()
        );

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn find_bookmarklet_output_case_only_duplicate_names_alias_on_windows() {
        let dir = make_test_dir();

        let first = dir.join("lgo_Thalya_stats.toml");
        let second = dir.join("lgo_thalya_stats.toml");
        std::fs::write(&first, "").expect("write 1");
        std::fs::write(&second, "").expect("write 2");

        let found = find_bookmarklet_output(&dir, "Thalya")
            .expect("case-only duplicate names should alias, not collide")
            .expect("must find file");

        assert!(
            found == first || found == second,
            "returned path must be one of the aliased spellings: {}",
            found.display()
        );

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
        assert_eq!(result.len(), 1, "Unknown slot must be silently skipped");
        assert_eq!(result[0].slot, crate::gear::Slot::Head);
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
        assert_eq!(result.len(), 1, "Bridle slot must be skipped");
        assert_eq!(result[0].slot, crate::gear::Slot::Head);
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
        assert_eq!(result.len(), 1, "all tool slot variants must be skipped");
        assert_eq!(result[0].slot, crate::gear::Slot::Head);
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
            result.len(),
            crate::gear::Slot::ALL.len(),
            "all 19 canonical slots must parse; Unknown and Bridle must be skipped"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
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
}
