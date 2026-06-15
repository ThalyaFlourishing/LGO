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

        let slot = parse_slot_str(slot_str)
            .ok_or_else(|| format!("[[item]] #{} unrecognised slot '{}'", idx + 1, slot_str))?;

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

/// Find the gear stats file the optimizer should read.
///
/// Preferred: `lgo_<character>_gear.toml` (the canonical merged file
/// produced by `resolve-slots`), matched case-insensitively. Fallback:
/// lexicographic scan over `lgo_stats_*.toml` for backward compatibility
/// with users who haven't re-run the new resolver yet.
///
/// Returns `Ok(Some(path))`, `Ok(None)` (nothing found), or
/// `Err(collision_message)` (two or more gear files match case-insensitively).
pub fn find_latest_stats_file(dir: &Path, character: &str) -> Result<Option<PathBuf>, String> {
    // Preferred: case-insensitive scan for lgo_<char>_gear.toml.
    let gear = find_canonical_gear_file(dir, character)?;
    if gear.is_some() {
        return Ok(gear);
    }

    // Fallback: lex-latest lgo_stats_*.toml.  The prefix match is
    // case-insensitive; the character segment is intentionally NOT filtered —
    // the lex-latest file wins regardless of which character it belongs to.
    // (See test `finds_latest_across_different_character_prefixes`.)
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory {}: {}", dir.display(), e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = match p.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => return false,
            };
            let name_lower = name.to_ascii_lowercase();
            name_lower.ends_with(".toml") && name_lower.starts_with("lgo_stats_")
        })
        .collect();

    if entries.is_empty() {
        return Ok(None);
    }

    entries.sort();
    Ok(entries.into_iter().last())
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
    fn finds_latest_stats_file_by_name_order() {
        let dir = make_test_dir();

        let older = dir.join("lgo_stats_A_20250101_000000.toml");
        let newer = dir.join("lgo_stats_A_20260101_000000.toml");
        std::fs::write(&older, "").expect("write older");
        std::fs::write(&newer, "").expect("write newer");

        // Character with no canonical file → falls back to lex scan.
        let found = find_latest_stats_file(&dir, "A")
            .expect("no error")
            .expect("latest file not found");
        assert_eq!(found, newer);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn finds_latest_across_different_character_prefixes() {
        let dir = make_test_dir();

        let older = dir.join("lgo_stats_CharA_20260101_000000.toml");
        let newer = dir.join("lgo_stats_CharB_20270101_000000.toml");
        std::fs::write(&older, "").expect("write older");
        std::fs::write(&newer, "").expect("write newer");

        let found = find_latest_stats_file(&dir, "CharA")
            .expect("no error")
            .expect("latest file not found");
        assert_eq!(found, newer);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn canonical_gear_file_is_preferred_over_lex_scan() {
        let dir = make_test_dir();

        let canonical = dir.join("lgo_Thalya_gear.toml");
        let bookmarklet = dir.join("lgo_stats_Thalya_99999999_999999.toml");
        std::fs::write(&canonical, "").expect("write canonical");
        std::fs::write(&bookmarklet, "").expect("write bookmarklet");

        let found = find_latest_stats_file(&dir, "Thalya")
            .expect("no error")
            .expect("must find a file");
        assert_eq!(found, canonical, "canonical gear file must win over lex scan");

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn find_bookmarklet_output_returns_exact_filename() {
        let dir = make_test_dir();

        // Decoy lgo_stats_*.toml — must NOT be returned.
        std::fs::write(dir.join("lgo_stats_Thalya_20260101_000000.toml"), "")
            .expect("write decoy");
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
    fn find_latest_stats_file_finds_lowercase_gear_file_for_mixed_case_query() {
        let dir = make_test_dir();
        let f = dir.join("lgo_thalya_gear.toml");
        std::fs::write(&f, "").expect("write file");

        let found = find_latest_stats_file(&dir, "Thalya")
            .expect("no error")
            .expect("must find file");
        assert_eq!(found, f);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn find_latest_stats_file_finds_uppercase_gear_file_for_lowercase_query() {
        let dir = make_test_dir();
        let f = dir.join("lgo_THALYA_gear.toml");
        std::fs::write(&f, "").expect("write file");

        let found = find_latest_stats_file(&dir, "thalya")
            .expect("no error")
            .expect("must find file");
        assert_eq!(found, f);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn find_latest_stats_file_errors_on_gear_file_collision() {
        let dir = make_test_dir();
        std::fs::write(dir.join("lgo_Thalya_gear.toml"), "").expect("write 1");
        std::fs::write(dir.join("lgo_thalya_gear.toml"), "").expect("write 2");

        let err = find_latest_stats_file(&dir, "Thalya").expect_err("should error on collision");
        assert!(
            err.contains("lgo_Thalya_gear.toml") && err.contains("lgo_thalya_gear.toml"),
            "error must name both colliding files: {err}"
        );

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
    fn find_bookmarklet_output_errors_on_collision() {
        let dir = make_test_dir();
        std::fs::write(dir.join("lgo_Thalya_stats.toml"), "").expect("write 1");
        std::fs::write(dir.join("lgo_thalya_stats.toml"), "").expect("write 2");

        let err =
            find_bookmarklet_output(&dir, "Thalya").expect_err("should error on collision");
        assert!(
            err.contains("lgo_Thalya_stats.toml") && err.contains("lgo_thalya_stats.toml"),
            "error must name both colliding files: {err}"
        );

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }
}
