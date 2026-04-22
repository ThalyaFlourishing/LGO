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

/// Generate a timestamp string in the same format as the plugin export:
/// `YYYYMMDD_HHMMSS` (UTC).
pub fn now_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (year, month, day, hour, min, sec) = epoch_secs_to_utc(secs);
    format!("{:04}{:02}{:02}_{:02}{:02}{:02}", year, month, day, hour, min, sec)
}

/// Convert seconds since the Unix epoch to `(year, month, day, hour, minute, second)` in UTC.
/// Uses only integer arithmetic — no external crate required.
fn epoch_secs_to_utc(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let s   = (secs % 60) as u32;
    let min = ((secs / 60) % 60) as u32;
    let h   = ((secs / 3600) % 24) as u32;
    let mut days = (secs / 86400) as u32;

    let mut year = 1970u32;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year { break; }
        days -= days_in_year;
        year += 1;
    }

    let month_lengths = [
        31u32,
        if is_leap_year(year) { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1u32;
    for &ml in &month_lengths {
        if days < ml { break; }
        days -= ml;
        month += 1;
    }
    let day = days + 1;

    (year, month, day, h, min, s)
}

fn is_leap_year(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
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

// -- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // epoch_secs_to_utc ---

    #[test]
    fn epoch_zero_is_unix_epoch() {
        assert_eq!(epoch_secs_to_utc(0), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn known_timestamp_2000_01_01_midnight() {
        // 2000-01-01 00:00:00 UTC = 946684800
        assert_eq!(epoch_secs_to_utc(946684800), (2000, 1, 1, 0, 0, 0));
    }

    #[test]
    fn known_timestamp_with_time_components() {
        // 2024-03-15 11:34:56 UTC = 1710502496
        assert_eq!(epoch_secs_to_utc(1710502496), (2024, 3, 15, 11, 34, 56));
    }

    #[test]
    fn leap_day_feb_29() {
        // 2000-02-29 00:00:00 UTC = 951782400
        assert_eq!(epoch_secs_to_utc(951782400), (2000, 2, 29, 0, 0, 0));
    }

    #[test]
    fn non_leap_year_skips_feb_29() {
        // 1900 is NOT a leap year (div by 100 but not 400)
        assert!(!is_leap_year(1900));
        // 2100 is NOT a leap year
        assert!(!is_leap_year(2100));
    }

    #[test]
    fn leap_year_400_rule() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2400));
    }

    #[test]
    fn regular_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
    }

    // now_timestamp ---

    #[test]
    fn now_timestamp_format() {
        let ts = now_timestamp();
        // Must be exactly "YYYYMMDD_HHMMSS" = 15 chars
        assert_eq!(ts.len(), 15, "unexpected length: {ts}");
        // Underscore in position 8
        assert_eq!(ts.chars().nth(8), Some('_'), "missing underscore: {ts}");
        // All other chars are digits
        for (i, c) in ts.chars().enumerate() {
            if i != 8 {
                assert!(c.is_ascii_digit(), "non-digit at position {i}: {ts}");
            }
        }
    }
}