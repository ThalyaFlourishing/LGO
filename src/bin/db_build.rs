//! Offline item database builder.
//!
//! Reads:
//!   data/items.xml        — LotroCompanion item database
//!   data/progressions.xml — LotroCompanion progression curves
//!
//! Writes:
//!   data/lgo_items.json   — flat resolved stat cache
//!                           (JSON object: item name -> {name, slot, stats})
//!                           Same shape as lgo_cache.json; loadable by src/db.rs.
//!
//! Usage (defaults shown):
//!   cargo run --bin db_build
//!   cargo run --bin db_build -- --items data/items.xml --progressions data/progressions.xml --out data/lgo_items.json
//!
//! If data/lgo_items.json already exists the build is skipped. Delete it to rebuild.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

// ── Local copies of the shared types ─────────────────────────────────────────
//
// Binaries in src/bin/ cannot import from the parent crate by name.
// We duplicate the minimal type definitions needed for serialisation.
// These must stay in sync with src/gear.rs, src/stat.rs, and src/cache.rs.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Stat {
    Armor, CriticalRating, Finesse, PhysicalMastery, TacticalMastery,
    OutgoingHealing, Resistance, CriticalDefense, IncomingHealing,
    Block, Parry, Evade, PhysicalMitigation, TacticalMitigation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Slot {
    Head, Chest, Legs, Hands, Feet, Shoulders, Back,
    Wrist1, Wrist2, Neck, Finger1, Finger2, Ear1, Ear2,
    Pocket, MainHand, OffHand, Ranged, ClassItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedItem {
    name:  String,
    slot:  Slot,
    stats: HashMap<Stat, i64>,
}

// ── CLI ───────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let items_path = flag(&args, "--items")
        .unwrap_or_else(|| "data/items.xml".to_string());
    let prog_path  = flag(&args, "--progressions")
        .unwrap_or_else(|| "data/progressions.xml".to_string());
    let out_path   = flag(&args, "--out")
        .unwrap_or_else(|| "data/lgo_items.json".to_string());

    if Path::new(&out_path).exists() {
        eprintln!("[db_build] '{}' already exists — skipping build.", out_path);
        eprintln!("[db_build] Delete it and re-run to rebuild.");
        return;
    }

    eprintln!("[db_build] Loading progressions from: {}", prog_path);
    let progressions = load_progressions(&prog_path)
        .unwrap_or_else(|e| { eprintln!("[db_build] FATAL: {}", e); std::process::exit(1); });
    eprintln!("[db_build] Loaded {} progression curves.", progressions.len());

    eprintln!("[db_build] Loading items from: {}", items_path);
    let items = load_items(&items_path, &progressions)
        .unwrap_or_else(|e| { eprintln!("[db_build] FATAL: {}", e); std::process::exit(1); });
    eprintln!("[db_build] Resolved {} equippable items with stats.", items.len());

    let json = serde_json::to_string_pretty(&items)
        .expect("Failed to serialise items");
    fs::write(&out_path, &json)
        .unwrap_or_else(|e| { eprintln!("[db_build] FATAL: Cannot write '{}': {}", out_path, e); std::process::exit(1); });

    eprintln!("[db_build] Written to: {}", out_path);
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == name)
        .map(|w| w[1].clone())
}

// ── Progression curves ────────────────────────────────────────────────────────

enum Progression {
    Array  { min_x: i32, values: Vec<f64> },
    Linear { points: Vec<(f64, f64)> },      // sorted by x
}

impl Progression {
    fn value_at(&self, level: i32) -> Option<f64> {
        match self {
            Progression::Array { min_x, values } => {
                let idx = (level - min_x) as usize;
                values.get(idx).copied()
            }
            Progression::Linear { points } => {
                if points.is_empty() { return None; }
                let x = level as f64;
                if x <= points[0].0                  { return Some(points[0].1); }
                if x >= points[points.len() - 1].0   { return Some(points[points.len() - 1].1); }
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
    let xml = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {}", path, e))?;

    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut map: HashMap<u32, Progression> = HashMap::new();
    let mut current_id:     Option<u32>             = None;
    let mut current_array:  Option<(i32, Vec<f64>)> = None;
    let mut current_linear: Option<Vec<(f64, f64)>> = None;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e)         => return Err(format!("XML error in '{}': {}", path, e)),
            Ok(Event::Eof) => break,

            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("").to_string();
                let attrs = collect_attrs(e);

                match tag.as_str() {
                    "arrayProgression" => {
                        let id    = parse_u32(&attrs, "identifier").unwrap_or(0);
                        let min_x = parse_i32(&attrs, "minX").unwrap_or(1);
                        let nb    = parse_usize(&attrs, "nbPoints").unwrap_or(0);
                        current_id     = Some(id);
                        current_array  = Some((min_x, Vec::with_capacity(nb)));
                        current_linear = None;
                    }
                    "linearInterpolationProgression" => {
                        let id = parse_u32(&attrs, "identifier").unwrap_or(0);
                        let nb = parse_usize(&attrs, "nbPoints").unwrap_or(0);
                        current_id     = Some(id);
                        current_linear = Some(Vec::with_capacity(nb));
                        current_array  = None;
                    }
                    "point" => {
                        if let Some((_, ref mut values)) = current_array {
                            let y     = parse_f64(&attrs, "y").unwrap_or(0.0);
                            let count = parse_usize(&attrs, "count").unwrap_or(0);
                            for _ in 0..count { values.push(y); }
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
    let xml = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {}", path, e))?;

    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut out: HashMap<String, CachedItem> = HashMap::new();

    let mut cur_name:  Option<String>     = None;
    let mut cur_slot:  Option<Slot>       = None;
    let mut cur_level: Option<i32>        = None;
    let mut cur_stats: HashMap<Stat, i64> = HashMap::new();
    let mut in_stats = false;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e)         => return Err(format!("XML error in '{}': {}", path, e)),
            Ok(Event::Eof) => break,

            Ok(Event::Start(ref e)) => {
                let name_bytes = e.name();        
                let tag = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("").to_string();
                let attrs = collect_attrs(e);

                match tag.as_str() {
                    "item" => {
                        let category = attrs.get("category").map(|s| s.as_str()).unwrap_or("");
                        if category == "LEGENDARY_WEAPON" {
                            cur_name  = None;
                            cur_slot  = None;
                            cur_level = None;
                            cur_stats.clear();
                            in_stats  = false;
                        } else {
                            cur_name  = attrs.get("name").cloned();
                            cur_level = attrs.get("level").and_then(|v| v.parse().ok());
                            cur_slot  = attrs.get("slot").and_then(|s| parse_slot_key(s));
                            cur_stats.clear();
                            in_stats  = false;
                        }
                    }
                    "stats" => { in_stats = true; }
                    "stat" if in_stats => {
                        handle_stat_element(&attrs, cur_level, progressions, &mut cur_stats);
                    }
                    _ => {}
                }
            }

            Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("").to_string();
                let attrs = collect_attrs(e);
                if tag == "stat" && in_stats {
                    handle_stat_element(&attrs, cur_level, progressions, &mut cur_stats);
                }
            }

            Ok(Event::End(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("").to_string();
                match tag.as_str() {
                    "stats" => { in_stats = false; }
                    "item"  => {
                        if let (Some(name), Some(slot)) = (cur_name.take(), cur_slot.take()) {
                            out.insert(name.clone(), CachedItem {
                                name,
                                slot,
                                stats: cur_stats.clone(),
                            });
                        }
                        cur_level = None;
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
    attrs:        &HashMap<String, String>,
    item_level:   Option<i32>,
    progressions: &HashMap<u32, Progression>,
    stats:        &mut HashMap<Stat, i64>,
) {
    let stat_name = match attrs.get("name") {
        Some(n) => n.as_str(),
        None    => return,
    };
    let stat = match parse_stat_name(stat_name) {
        Some(s) => s,
        None    => return,
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
                        "[db_build] WARN: progression {} has no value at level {}",
                        prog_id, level
                    );
                }
            } else {
                eprintln!("[db_build] WARN: progression {} not found", prog_id);
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

fn parse_slot_key(s: &str) -> Option<Slot> {
    match s {
        "HEAD"                          => Some(Slot::Head),
        "CHEST"                         => Some(Slot::Chest),
        "LEGS"                          => Some(Slot::Legs),
        "HAND"                          => Some(Slot::Hands),
        "FEET"                          => Some(Slot::Feet),
        "SHOULDER"                      => Some(Slot::Shoulders),
        "BACK"                          => Some(Slot::Back),
        "WRIST" | "LEFT_WRIST"
        | "RIGHT_WRIST"                 => Some(Slot::Wrist1),
        "NECK"                          => Some(Slot::Neck),
        "FINGER" | "LEFT_FINGER"
        | "RIGHT_FINGER"                => Some(Slot::Finger1),
        "EAR"   | "LEFT_EAR"
        | "RIGHT_EAR"                   => Some(Slot::Ear1),
        "POCKET"                        => Some(Slot::Pocket),
        "MAIN_HAND"                     => Some(Slot::MainHand),
        "EITHER_HAND"                   => Some(Slot::OffHand),
        "OFF_HAND"                      => Some(Slot::OffHand),
        "RANGED_ITEM"                   => Some(Slot::Ranged),
        "CLASS_SLOT"                    => Some(Slot::ClassItem),
        _                               => None,
    }
}

// ── Stat name mapping ─────────────────────────────────────────────────────────

fn parse_stat_name(s: &str) -> Option<Stat> {
    match s {
        "ARMOUR"              => Some(Stat::Armor),
        "CRITICAL_RATING"     => Some(Stat::CriticalRating),
        "FINESSE"             => Some(Stat::Finesse),
        "PHYSICAL_MASTERY"    => Some(Stat::PhysicalMastery),
        "TACTICAL_MASTERY"    => Some(Stat::TacticalMastery),
        "OUTGOING_HEALING"    => Some(Stat::OutgoingHealing),
        "RESISTANCE"          => Some(Stat::Resistance),
        "CRITICAL_DEFENCE"    => Some(Stat::CriticalDefense),
        "INCOMING_HEALING"    => Some(Stat::IncomingHealing),
        "BLOCK"               => Some(Stat::Block),
        "PARRY"               => Some(Stat::Parry),
        "EVADE"               => Some(Stat::Evade),
        "PHYSICAL_MITIGATION" => Some(Stat::PhysicalMitigation),
        "TACTICAL_MITIGATION" => Some(Stat::TacticalMitigation),
        _                     => None,
    }
}

// ── XML attribute helpers ─────────────────────────────────────────────────────

fn collect_attrs(e: &quick_xml::events::BytesStart) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref())
            .unwrap_or("").to_string();
        let val = attr.unescape_value()
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