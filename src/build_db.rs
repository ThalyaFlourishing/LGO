//! Offline item database builder.
//!
//! Reads `data/items.xml` (LotroCompanion item database) and
//! `data/progressions.xml` (LotroCompanion progression curves), then writes
//! `data/lgo_items.json` — the flat resolved stat cache consumed by the slot
//! resolver.
//!
//! Exposed as `lgo build-db [options]`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::gear::{CachedItem, Slot};
use crate::stat::Stat;

// ── Public entry point ────────────────────────────────────────────────────────

/// Build the item database from `items_path` and `progressions_path` and write
/// the result to `out_path`. Always overwrites an existing file.
pub fn build(items_path: &Path, progressions_path: &Path, out_path: &Path) -> Result<(), String> {
    let prog_str = progressions_path
        .to_str()
        .ok_or_else(|| format!("Invalid path: {}", progressions_path.display()))?;
    let items_str = items_path
        .to_str()
        .ok_or_else(|| format!("Invalid path: {}", items_path.display()))?;
    let out_str = out_path
        .to_str()
        .ok_or_else(|| format!("Invalid path: {}", out_path.display()))?;

    eprintln!("[build-db] Loading progressions from: {}", prog_str);
    let progressions = load_progressions(prog_str)?;
    eprintln!(
        "[build-db] Loaded {} progression curves.",
        progressions.len()
    );

    eprintln!("[build-db] Loading items from: {}", items_str);
    let items = load_items(items_str, &progressions)?;
    eprintln!(
        "[build-db] Resolved {} equippable items with stats.",
        items.len()
    );

    let json = serde_json::to_string_pretty(&items)
        .map_err(|e| format!("Failed to serialise items: {}", e))?;
    fs::write(out_path, &json).map_err(|e| format!("Cannot write '{}': {}", out_str, e))?;

    eprintln!("[build-db] Written to: {}", out_str);
    Ok(())
}

// ── Progression curves ────────────────────────────────────────────────────────

enum Progression {
    Array { min_x: i32, values: Vec<f64> },
    Linear { points: Vec<(f64, f64)> }, // sorted by x
}

impl Progression {
    fn value_at(&self, level: i32) -> Option<f64> {
        match self {
            Progression::Array { min_x, values } => {
                let idx = (level - min_x) as usize;
                values.get(idx).copied()
            }
            Progression::Linear { points } => {
                if points.is_empty() {
                    return None;
                }
                let x = level as f64;
                if x <= points[0].0 {
                    return Some(points[0].1);
                }
                if x >= points[points.len() - 1].0 {
                    return Some(points[points.len() - 1].1);
                }
                for w in points.windows(2) {
                    let (x0, y0) = w[0];
                    let (x1, y1) = w[1];
                    if x >= x0 && x <= x1 {
                        let t = (x - x0) / (x1 - x0);
                        return Some(y0 + t * (y1 - y0));
                    }
                }
                None
            }
        }
    }
}

fn load_progressions(path: &str) -> Result<HashMap<u32, Progression>, String> {
    let xml = fs::read_to_string(path).map_err(|e| format!("Cannot read '{}': {}", path, e))?;

    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut map: HashMap<u32, Progression> = HashMap::new();
    let mut current_id: Option<u32> = None;
    let mut current_array: Option<(i32, Vec<f64>)> = None;
    let mut current_linear: Option<Vec<(f64, f64)>> = None;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(format!("XML error in '{}': {}", path, e)),
            Ok(Event::Eof) => break,

            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref())
                    .unwrap_or("")
                    .to_string();
                let attrs = collect_attrs(e);

                match tag.as_str() {
                    "arrayProgression" => {
                        let id = parse_u32(&attrs, "identifier").unwrap_or(0);
                        let min_x = parse_i32(&attrs, "minX").unwrap_or(1);
                        let nb = parse_usize(&attrs, "nbPoints").unwrap_or(0);
                        current_id = Some(id);
                        current_array = Some((min_x, Vec::with_capacity(nb)));
                        current_linear = None;
                    }
                    "linearInterpolationProgression" => {
                        let id = parse_u32(&attrs, "identifier").unwrap_or(0);
                        let nb = parse_usize(&attrs, "nbPoints").unwrap_or(0);
                        current_id = Some(id);
                        current_linear = Some(Vec::with_capacity(nb));
                        current_array = None;
                    }
                    "point" => {
                        if let Some((_, ref mut values)) = current_array {
                            let y = parse_f64(&attrs, "y").unwrap_or(0.0);
                            let count = parse_usize(&attrs, "count").unwrap_or(0);
                            for _ in 0..count {
                                values.push(y);
                            }
                        } else if let Some(ref mut points) = current_linear {
                            let x = parse_f64(&attrs, "x").unwrap_or(0.0);
                            let y = parse_f64(&attrs, "y").unwrap_or(0.0);
                            points.push((x, y));
                        }
                    }
                    _ => {}
                }
            }

            Ok(Event::End(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");
                match tag {
                    "arrayProgression" => {
                        if let (Some(id), Some((min_x, values))) =
                            (current_id.take(), current_array.take())
                        {
                            map.insert(id, Progression::Array { min_x, values });
                        }
                    }
                    "linearInterpolationProgression" => {
                        if let (Some(id), Some(mut points)) =
                            (current_id.take(), current_linear.take())
                        {
                            points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                            map.insert(id, Progression::Linear { points });
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }
        buf.clear();
    }
    Ok(map)
}

// ── Item parsing ──────────────────────────────────────────────────────────────

fn load_items(
    path: &str,
    progressions: &HashMap<u32, Progression>,
) -> Result<HashMap<String, CachedItem>, String> {
    let xml = fs::read_to_string(path).map_err(|e| format!("Cannot read '{}': {}", path, e))?;

    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut out: HashMap<String, CachedItem> = HashMap::new();
    // Tracks the highest item level stored for each name, so that when the
    // same item name appears at multiple levels we always keep the highest.
    let mut out_levels: HashMap<String, i32> = HashMap::new();

    let mut cur_name: Option<String> = None;
    let mut cur_slot: Option<Slot> = None;
    let mut cur_level: Option<i32> = None;
    let mut cur_two_handed = false;
    let mut cur_stats: HashMap<Stat, i64> = HashMap::new();
    let mut in_stats = false;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(format!("XML error in '{}': {}", path, e)),
            Ok(Event::Eof) => break,

            Ok(Event::Start(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref())
                    .unwrap_or("")
                    .to_string();
                let attrs = collect_attrs(e);

                match tag.as_str() {
                    "item" => {
                        let category = attrs.get("category").map(|s| s.as_str()).unwrap_or("");
                        if category == "LEGENDARY_WEAPON" {
                            cur_name = None;
                            cur_slot = None;
                            cur_level = None;
                            cur_two_handed = false;
                            cur_stats.clear();
                            in_stats = false;
                        } else {
                            cur_name = attrs.get("name").cloned();
                            cur_level = attrs.get("level").and_then(|v| v.parse().ok());
                            cur_slot = attrs.get("slot").and_then(|s| parse_slot_key(s));
                            cur_two_handed = is_two_handed(cur_slot, &attrs);
                            cur_stats.clear();
                            in_stats = false;
                        }
                    }
                    "stats" => {
                        in_stats = true;
                    }
                    "stat" if in_stats => {
                        handle_stat_element(&attrs, cur_level, progressions, &mut cur_stats);
                    }
                    _ => {}
                }
            }

            Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref())
                    .unwrap_or("")
                    .to_string();
                let attrs = collect_attrs(e);
                if tag == "stat" && in_stats {
                    handle_stat_element(&attrs, cur_level, progressions, &mut cur_stats);
                }
            }

            Ok(Event::End(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref())
                    .unwrap_or("")
                    .to_string();
                match tag.as_str() {
                    "stats" => {
                        in_stats = false;
                    }
                    "item" => {
                        if let (Some(name), Some(slot)) = (cur_name.take(), cur_slot.take()) {
                            let new_level = cur_level.unwrap_or(0);
                            let best_level = out_levels.get(&name).copied().unwrap_or(-1);
                            if new_level > best_level {
                                out_levels.insert(name.clone(), new_level);
                                out.insert(
                                    name.clone(),
                                    CachedItem {
                                        name,
                                        slot,
                                        two_handed: cur_two_handed,
                                        stats: cur_stats.clone(),
                                    },
                                );
                            }
                        }
                        cur_level = None;
                        cur_two_handed = false;
                        cur_stats.clear();
                    }
                    _ => {}
                }
            }

            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

fn handle_stat_element(
    attrs: &HashMap<String, String>,
    item_level: Option<i32>,
    progressions: &HashMap<u32, Progression>,
    stats: &mut HashMap<Stat, i64>,
) {
    let stat_name = match attrs.get("name") {
        Some(n) => n.as_str(),
        None => return,
    };
    let stat = match parse_stat_name(stat_name) {
        Some(s) => s,
        None => return,
    };

    if let Some(prog_id_str) = attrs.get("scaling") {
        if let Ok(prog_id) = prog_id_str.parse::<u32>() {
            if let Some(prog) = progressions.get(&prog_id) {
                let level = item_level.unwrap_or(1);
                if let Some(val) = prog.value_at(level) {
                    // LotRO uses truncation for Armor, and round() for all other stats.
                    let converted = if stat == Stat::Armor {
                        val as i64
                    } else {
                        val.round() as i64
                    };
                    *stats.entry(stat).or_insert(0) += converted;
                } else {
                    eprintln!(
                        "[build-db] WARN: progression {} has no value at level {}",
                        prog_id, level
                    );
                }
            } else {
                eprintln!("[build-db] WARN: progression {} not found", prog_id);
            }
        }
        return;
    }

    let fixed = attrs.get("constant").or_else(|| attrs.get("value"));
    if let Some(val_str) = fixed {
        if let Ok(val) = val_str.parse::<f64>() {
            *stats.entry(stat).or_insert(0) += val as i64;
        }
    }
}

// ── Slot key mapping ──────────────────────────────────────────────────────────

/// True when this XML item is a two-handed `MainHand` weapon: it carries a
/// `precludedSlots` attribute referencing the off-hand. `precludedSlots` is
/// the game data's marker that equipping the item blocks the `OFF_HAND`
/// slot; only `MAIN_HAND` items carry it.
fn is_two_handed(slot: Option<Slot>, attrs: &HashMap<String, String>) -> bool {
    slot == Some(Slot::MainHand)
        && attrs
            .get("precludedSlots")
            .is_some_and(|precluded| precluded.contains("OFF_HAND"))
}

fn parse_slot_key(s: &str) -> Option<Slot> {
    match s {
        "HEAD" => Some(Slot::Head),
        "CHEST" => Some(Slot::Chest),
        "LEGS" => Some(Slot::Legs),
        "HAND" => Some(Slot::Hands),
        "FEET" => Some(Slot::Feet),
        "SHOULDER" => Some(Slot::Shoulders),
        "BACK" => Some(Slot::Back),
        "WRIST" | "LEFT_WRIST" | "RIGHT_WRIST" => Some(Slot::Wrist1),
        "NECK" => Some(Slot::Neck),
        "FINGER" | "LEFT_FINGER" | "RIGHT_FINGER" => Some(Slot::Finger1),
        "EAR" | "LEFT_EAR" | "RIGHT_EAR" => Some(Slot::Ear1),
        "POCKET" => Some(Slot::Pocket),
        "MAIN_HAND" => Some(Slot::MainHand),
        "EITHER_HAND" => Some(Slot::OffHand),
        "OFF_HAND" => Some(Slot::OffHand),
        "RANGED_ITEM" => Some(Slot::Ranged),
        "CLASS_SLOT" => Some(Slot::ClassItem),
        _ => None,
    }
}

// ── Stat name mapping ─────────────────────────────────────────────────────────

fn parse_stat_name(s: &str) -> Option<Stat> {
    match s {
        "ARMOUR" => Some(Stat::Armor),
        "CRITICAL_RATING" => Some(Stat::CriticalRating),
        "FINESSE" => Some(Stat::Finesse),
        "PHYSICAL_MASTERY" => Some(Stat::PhysicalMastery),
        "TACTICAL_MASTERY" => Some(Stat::TacticalMastery),
        "OUTGOING_HEALING" => Some(Stat::OutgoingHealing),
        "RESISTANCE" => Some(Stat::Resistance),
        "CRITICAL_DEFENCE" => Some(Stat::CriticalDefense),
        "INCOMING_HEALING" => Some(Stat::IncomingHealing),
        "BLOCK" => Some(Stat::Block),
        "PARRY" => Some(Stat::Parry),
        "EVADE" => Some(Stat::Evade),
        "PHYSICAL_MITIGATION" => Some(Stat::PhysicalMitigation),
        "TACTICAL_MITIGATION" => Some(Stat::TacticalMitigation),
        _ => None,
    }
}

// ── XML attribute helpers ─────────────────────────────────────────────────────

fn collect_attrs(e: &quick_xml::events::BytesStart) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref())
            .unwrap_or("")
            .to_string();
        let val = attr
            .unescape_value()
            .map(|v| v.into_owned())
            .unwrap_or_default();
        map.insert(key, val);
    }
    map
}

fn parse_u32(attrs: &HashMap<String, String>, key: &str) -> Option<u32> {
    attrs.get(key)?.parse().ok()
}
fn parse_i32(attrs: &HashMap<String, String>, key: &str) -> Option<i32> {
    attrs.get(key)?.parse().ok()
}
fn parse_usize(attrs: &HashMap<String, String>, key: &str) -> Option<usize> {
    attrs.get(key)?.parse().ok()
}
fn parse_f64(attrs: &HashMap<String, String>, key: &str) -> Option<f64> {
    attrs.get(key)?.parse().ok()
}

// ── Rounding tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Build a single-entry array progression whose value at `level` is `val`.
    fn prog_with_value(val: f64) -> HashMap<u32, Progression> {
        let mut map = HashMap::new();
        map.insert(
            1u32,
            Progression::Array {
                min_x: 1,
                values: vec![val],
            },
        );
        map
    }

    fn scaling_attrs(stat_name: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("name".to_string(), stat_name.to_string());
        m.insert("scaling".to_string(), "1".to_string());
        m
    }

    fn fixed_attrs(stat_name: &str, val_str: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("name".to_string(), stat_name.to_string());
        m.insert("value".to_string(), val_str.to_string());
        m
    }

    // ── Armor: progression value is truncated (not rounded) ───────────────────

    #[test]
    fn armor_scaling_truncates_fractional_part() {
        // 123.9 → 123, not 124
        let progs = prog_with_value(123.9);
        let mut stats = HashMap::new();
        handle_stat_element(&scaling_attrs("ARMOUR"), Some(1), &progs, &mut stats);
        assert_eq!(stats[&Stat::Armor], 123);
    }

    #[test]
    fn armor_scaling_truncates_at_exactly_half() {
        // 123.5 → 123, not 124 (truncation, not round-half-up)
        let progs = prog_with_value(123.5);
        let mut stats = HashMap::new();
        handle_stat_element(&scaling_attrs("ARMOUR"), Some(1), &progs, &mut stats);
        assert_eq!(stats[&Stat::Armor], 123);
    }

    // ── Non-armor: progression value is rounded to nearest integer ────────────

    #[test]
    fn non_armor_scaling_rounds_up_at_exactly_half() {
        // 123.5 → 124 (round-half-away-from-zero)
        let progs = prog_with_value(123.5);
        let mut stats = HashMap::new();
        handle_stat_element(
            &scaling_attrs("CRITICAL_RATING"),
            Some(1),
            &progs,
            &mut stats,
        );
        assert_eq!(stats[&Stat::CriticalRating], 124);
    }

    #[test]
    fn non_armor_scaling_rounds_up_above_half() {
        // 123.9 → 124
        let progs = prog_with_value(123.9);
        let mut stats = HashMap::new();
        handle_stat_element(&scaling_attrs("FINESSE"), Some(1), &progs, &mut stats);
        assert_eq!(stats[&Stat::Finesse], 124);
    }

    #[test]
    fn non_armor_scaling_rounds_down_below_half() {
        // 123.4 → 123
        let progs = prog_with_value(123.4);
        let mut stats = HashMap::new();
        handle_stat_element(&scaling_attrs("RESISTANCE"), Some(1), &progs, &mut stats);
        assert_eq!(stats[&Stat::Resistance], 123);
    }

    // ── Fixed values: truncated for all stats ─────────────────────────────────

    #[test]
    fn fixed_value_armor_is_truncated() {
        // "123.9" → 123 (truncation via `val as i64`)
        let mut stats = HashMap::new();
        handle_stat_element(
            &fixed_attrs("ARMOUR", "123.9"),
            None,
            &HashMap::new(),
            &mut stats,
        );
        assert_eq!(stats[&Stat::Armor], 123);
    }

    #[test]
    fn fixed_value_non_armor_is_truncated() {
        // "123.9" → 123 (truncation via `val as i64`)
        let mut stats = HashMap::new();
        handle_stat_element(
            &fixed_attrs("CRITICAL_RATING", "123.9"),
            None,
            &HashMap::new(),
            &mut stats,
        );
        assert_eq!(stats[&Stat::CriticalRating], 123);
    }

    // ── Level preference: highest item level wins on name collision ───────────

    #[test]
    fn highest_level_entry_wins() {
        // Simulate two XML entries for the same item name at different levels.
        // The higher-level entry should be the one kept in the output map.
        let _progressions: HashMap<u32, Progression> = HashMap::new();

        // Low-level entry: CriticalRating = 50
        let mut low_stats = HashMap::new();
        low_stats.insert(Stat::CriticalRating, 50i64);

        // High-level entry: CriticalRating = 8713
        let mut high_stats = HashMap::new();
        high_stats.insert(Stat::CriticalRating, 8713i64);

        let mut out: HashMap<String, CachedItem> = HashMap::new();
        let mut out_levels: HashMap<String, i32> = HashMap::new();

        // Insert low-level entry first (level 122).
        let name = "Wilful Bracer of the Bear in Winter".to_string();
        let level_low = 122i32;
        let best = out_levels.get(&name).copied().unwrap_or(-1);
        if level_low > best {
            out_levels.insert(name.clone(), level_low);
            out.insert(
                name.clone(),
                CachedItem {
                    name: name.clone(),
                    slot: Slot::Wrist1,
                    two_handed: false,
                    stats: low_stats,
                },
            );
        }

        // Insert high-level entry second (level 160).
        let level_high = 160i32;
        let best = out_levels.get(&name).copied().unwrap_or(-1);
        if level_high > best {
            out_levels.insert(name.clone(), level_high);
            out.insert(
                name.clone(),
                CachedItem {
                    name: name.clone(),
                    slot: Slot::Wrist1,
                    two_handed: false,
                    stats: high_stats,
                },
            );
        }

        assert_eq!(out[&name].stats[&Stat::CriticalRating], 8713);
    }

    #[test]
    fn lower_level_entry_does_not_overwrite_higher() {
        // Same as above but insertion order reversed: high first, then low.
        let mut out: HashMap<String, CachedItem> = HashMap::new();
        let mut out_levels: HashMap<String, i32> = HashMap::new();

        let name = "Test Item".to_string();

        let mut high_stats = HashMap::new();
        high_stats.insert(Stat::CriticalRating, 8713i64);
        let level_high = 160i32;
        let best = out_levels.get(&name).copied().unwrap_or(-1);
        if level_high > best {
            out_levels.insert(name.clone(), level_high);
            out.insert(
                name.clone(),
                CachedItem {
                    name: name.clone(),
                    slot: Slot::Wrist1,
                    two_handed: false,
                    stats: high_stats,
                },
            );
        }

        let mut low_stats = HashMap::new();
        low_stats.insert(Stat::CriticalRating, 50i64);
        let level_low = 122i32;
        let best = out_levels.get(&name).copied().unwrap_or(-1);
        if level_low > best {
            out_levels.insert(name.clone(), level_low);
            out.insert(
                name.clone(),
                CachedItem {
                    name: name.clone(),
                    slot: Slot::Wrist1,
                    two_handed: false,
                    stats: low_stats,
                },
            );
        }

        assert_eq!(out[&name].stats[&Stat::CriticalRating], 8713);
    }

    // ── Two-handed detection via precludedSlots ────────────────────────────────

    /// Writes `xml` to a unique temp file and runs `load_items` on it.
    fn load_items_from_str(label: &str, xml: &str) -> HashMap<String, CachedItem> {
        let path = std::env::temp_dir().join(format!(
            "lgo_build_db_test_{}_{}.xml",
            label,
            std::process::id()
        ));
        fs::write(&path, xml).expect("write temp xml");
        let result = load_items(path.to_str().unwrap(), &HashMap::new());
        let _ = fs::remove_file(&path);
        result.expect("load_items should succeed")
    }

    #[test]
    fn main_hand_with_precluded_slots_is_two_handed() {
        let xml = r#"<items>
            <item name="Example Greatsword" level="160" slot="MAIN_HAND" precludedSlots="OFF_HAND"></item>
        </items>"#;
        let out = load_items_from_str("two_handed", xml);
        let item = &out["Example Greatsword"];
        assert_eq!(item.slot, Slot::MainHand);
        assert!(item.two_handed);
    }

    #[test]
    fn main_hand_without_precluded_slots_is_one_handed() {
        let xml = r#"<items>
            <item name="Example Dagger" level="160" slot="MAIN_HAND"></item>
        </items>"#;
        let out = load_items_from_str("one_handed", xml);
        let item = &out["Example Dagger"];
        assert_eq!(item.slot, Slot::MainHand);
        assert!(!item.two_handed);
    }

    #[test]
    fn two_handed_flag_serializes_and_one_handed_flag_is_omitted() {
        // The JSON DB is the transport for two-handedness into resolve-slots:
        // `true` must round-trip and `false` must be omitted entirely.
        let xml = r#"<items>
            <item name="Example Greatsword" level="160" slot="MAIN_HAND" precludedSlots="OFF_HAND"></item>
            <item name="Example Dagger" level="160" slot="MAIN_HAND"></item>
        </items>"#;
        let out = load_items_from_str("serialize", xml);
        let json = serde_json::to_string(&out).expect("serialize db");
        let parsed: HashMap<String, CachedItem> = serde_json::from_str(&json).expect("reparse db");
        assert!(parsed["Example Greatsword"].two_handed);
        assert!(!parsed["Example Dagger"].two_handed);
        // `false` is skipped during serialization, so the only occurrence of
        // the key belongs to the greatsword.
        assert_eq!(json.matches("two_handed").count(), 1);
    }
}
