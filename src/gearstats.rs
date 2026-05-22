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

/// Find the most recent `lgo_stats_*.toml` in `dir`.
/// Returns `None` if no matching file exists.
pub fn find_latest_stats_file(dir: &Path) -> Option<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("toml")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("lgo_stats_"))
                    .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        return None;
    }

    entries.sort();
    entries.into_iter().last()
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

    #[test]
    fn finds_latest_stats_file_by_name_order() {
        let dir = std::env::temp_dir().join(format!(
            "lgo_gearstats_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");

        let older = dir.join("lgo_stats_A_20250101_000000.toml");
        let newer = dir.join("lgo_stats_A_20260101_000000.toml");
        std::fs::write(&older, "").expect("write older");
        std::fs::write(&newer, "").expect("write newer");

        let found = find_latest_stats_file(&dir).expect("latest file not found");
        assert_eq!(found, newer);

        std::fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }
}
