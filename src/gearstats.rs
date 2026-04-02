//! TOML gear stats file: the editable intermediate file between plugin export
//! and the optimizer.
//!
//! Workflow:
//!   1. `lgo --stats` (or the `stats` binary) writes
//!      `lgo_stats_{character}_{timestamp}.toml` to the character's
//!      AllServers directory.
//!   2. The user edits it to fill in any missing stats (e.g. player-renamed LIs).
//!   3. `lgo <goals>` auto-detects the most recent stats file and uses it as
//!      input instead of the db/wiki/cache pipeline.
//!
//! Format: one `[[item]]` block per item, with all 14 tracked stats in
//! canonical order. Stats known from the db/wiki are pre-filled; unknowns
//! are written as 0. Items where all tracked stats are zero get a warning
//! comment prompting the user to fill them in.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cache::CachedItem;
use crate::gear::Slot;
use crate::stat::{Stat, TRACKED_STATS};

// -- Writer --------------------------------------------------------------------

/// Write a TOML gear stats file for the given resolved items.
///
/// Items are written in the order supplied (caller should pass them in slot
/// order). All 14 tracked stats are emitted for every item in canonical order,
/// with known values filled in and unknowns as 0.
/// Items where every tracked stat is 0 receive a warning comment.
pub fn write_stats_file(
    items:     &[CachedItem],
    path:      &Path,
    character: &str,
) -> Result<(), String> {
    let mut out = String::new();

    out.push_str(&format!(
        "# LGO gear stats file — character: {}\n", character
    ));
    out.push_str("# Edit stat values below, then run: lgo <stat:minimum> ...\n");
    out.push_str("# All 14 tracked stats are listed for every item in canonical order.\n");
    out.push_str("# Items with all zeros need manual entry before running the optimizer.\n");
    out.push('\n');

    for item in items {
        let all_zero = TRACKED_STATS.iter().all(|(stat, _)| {
            item.stats.get(stat).copied().unwrap_or(0) == 0
        });

        out.push_str("[[item]]\n");
        out.push_str(&format!("slot = \"{}\"\n", item.slot));
        out.push_str(&format!("name = \"{}\"\n", item.name));

        if all_zero {
            out.push_str("# WARNING: all stats unknown — edit before running optimizer\n");
        }

        for (stat, key) in TRACKED_STATS {
            let value = item.stats.get(stat).copied().unwrap_or(0);
            out.push_str(&format!("{:<22}= {}\n", format!("{} ", key), value));
        }

        out.push('\n');
    }

    fs::write(path, &out)
        .map_err(|e| format!("Cannot write gear stats file {}: {}", path.display(), e))?;

    Ok(())
}

/// Build the stats file path for the given character and timestamp.
/// Format: `lgo_stats_{character}_{timestamp}.toml`
pub fn stats_file_path(dir: &Path, character: &str, timestamp: &str) -> PathBuf {
    dir.join(format!("lgo_stats_{}_{}.toml", character, timestamp))
}

/// Generate a sortable timestamp string (seconds since Unix epoch).
/// Replace with a proper time crate call if one is added later.
pub fn now_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

// -- Reader --------------------------------------------------------------------

/// Parse a TOML gear stats file back into a list of CachedItems.
///
/// Only non-zero stat values are stored in the returned items' stats maps,
/// consistent with how the rest of the pipeline treats missing stats as 0.
pub fn read_stats_file(path: &Path) -> Result<Vec<CachedItem>, String> {
    let src = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read gear stats file {}: {}", path.display(), e))?;

    let doc: toml::Value = src.parse()
        .map_err(|e| format!("Malformed TOML in {}: {}", path.display(), e))?;

    let items_arr = doc
        .get("item")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("No [[item]] entries found in {}", path.display()))?;

    let mut items = Vec::new();

    for (idx, entry) in items_arr.iter().enumerate() {
        let slot_str = entry.get("slot")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("[[item]] #{} missing 'slot'", idx + 1))?;

        let name = entry.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("[[item]] #{} missing 'name'", idx + 1))?
            .to_string();

        let slot = parse_slot_str(slot_str)
            .ok_or_else(|| format!(
                "[[item]] #{} unrecognised slot '{}'", idx + 1, slot_str
            ))?;

        let mut stats: HashMap<Stat, i64> = HashMap::new();

        for (stat, key) in TRACKED_STATS {
            if let Some(val) = entry.get(*key).and_then(|v| v.as_integer()) {
                if val != 0 {
                    stats.insert(*stat, val);
                }
            }
        }

        items.push(CachedItem { name, slot, stats });
    }

    Ok(items)
}

/// Find the most recent `lgo_stats_{character}_*.toml` in `dir`.
/// Returns `None` if no matching file exists.
pub fn find_latest_stats_file(dir: &Path, character: &str) -> Option<PathBuf> {
    let prefix = format!("lgo_stats_{}_", character);

    let mut entries: Vec<PathBuf> = fs::read_dir(dir).ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("toml")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(&prefix))
                    .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        return None;
    }

    entries.sort();
    entries.into_iter().last()
}

// -- Helpers -------------------------------------------------------------------

/// Parse a slot display string back to a Slot variant.
/// Must match the Display impl in gear.rs exactly.
fn parse_slot_str(s: &str) -> Option<Slot> {
    match s {
        "Head"       => Some(Slot::Head),
        "Chest"      => Some(Slot::Chest),
        "Legs"       => Some(Slot::Legs),
        "Hands"      => Some(Slot::Hands),
        "Feet"       => Some(Slot::Feet),
        "Shoulders"  => Some(Slot::Shoulders),
        "Back"       => Some(Slot::Back),
        "Wrist (1)"  => Some(Slot::Wrist1),
        "Wrist (2)"  => Some(Slot::Wrist2),
        "Neck"       => Some(Slot::Neck),
        "Finger (1)" => Some(Slot::Finger1),
        "Finger (2)" => Some(Slot::Finger2),
        "Ear (1)"    => Some(Slot::Ear1),
        "Ear (2)"    => Some(Slot::Ear2),
        "Pocket"     => Some(Slot::Pocket),
        "Main-hand"  => Some(Slot::MainHand),
        "Off-hand"   => Some(Slot::OffHand),
        "Ranged"     => Some(Slot::Ranged),
        "Class Item" => Some(Slot::ClassItem),
        _            => None,
    }
}