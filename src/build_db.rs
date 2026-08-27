//! Offline item database builder.
//!
//! Reads `data/items.xml` (LotroCompanion item database) and writes
//! `data/lgo_items.json` — the name → slot (+ `two_handed` flag) index
//! consumed by the slot resolver. Item stats are *not* extracted here;
//! they come from the bookmarklet.
//!
//! Exposed as `lgo build-db [options]`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::gear::{CachedItem, Slot};

// ── Public entry point ────────────────────────────────────────────────────────

/// Build the item database from `items_path` and write the result to
/// `out_path`. Always overwrites an existing file.
pub fn build(items_path: &Path, out_path: &Path) -> Result<(), String> {
    let items_str = items_path
        .to_str()
        .ok_or_else(|| format!("Invalid path: {}", items_path.display()))?;
    let out_str = out_path
        .to_str()
        .ok_or_else(|| format!("Invalid path: {}", out_path.display()))?;

    eprintln!("[build-db] Loading items from: {}", items_str);
    let items = load_items(items_str)?;
    eprintln!("[build-db] Indexed {} equippable items.", items.len());

    let json = serde_json::to_string_pretty(&items)
        .map_err(|e| format!("Failed to serialise items: {}", e))?;
    fs::write(out_path, &json).map_err(|e| format!("Cannot write '{}': {}", out_str, e))?;

    eprintln!("[build-db] Written to: {}", out_str);
    Ok(())
}

// ── Item parsing ──────────────────────────────────────────────────────────────

fn load_items(path: &str) -> Result<HashMap<String, CachedItem>, String> {
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
    let mut cur_either_hand = false;

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

                if tag == "item" {
                    let attrs = collect_attrs(e);
                    let category = attrs.get("category").map(|s| s.as_str()).unwrap_or("");
                    if category == "LEGENDARY_WEAPON" {
                        cur_name = None;
                        cur_slot = None;
                        cur_level = None;
                        cur_two_handed = false;
                        cur_either_hand = false;
                    } else {
                        cur_name = attrs.get("name").cloned();
                        cur_level = attrs.get("level").and_then(|v| v.parse().ok());
                        cur_slot = attrs.get("slot").and_then(|s| parse_slot_key(s));
                        cur_two_handed = is_two_handed(cur_slot, &attrs);
                        cur_either_hand = is_either_hand(&attrs);
                    }
                }
            }

            Ok(Event::End(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");

                if tag == "item" {
                    if let (Some(name), Some(slot)) = (cur_name.take(), cur_slot.take()) {
                        insert_if_highest_level(
                            &mut out,
                            &mut out_levels,
                            name,
                            slot,
                            cur_level,
                            cur_two_handed,
                            cur_either_hand,
                        );
                    }
                    cur_level = None;
                    cur_two_handed = false;
                    cur_either_hand = false;
                }
            }

            Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name();
                let tag = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");

                if tag == "item" {
                    let attrs = collect_attrs(e);
                    let category = attrs.get("category").map(|s| s.as_str()).unwrap_or("");
                    if category != "LEGENDARY_WEAPON" {
                        let name = attrs.get("name").cloned();
                        let level = attrs.get("level").and_then(|v| v.parse().ok());
                        let slot = attrs.get("slot").and_then(|s| parse_slot_key(s));
                        let two_handed = is_two_handed(slot, &attrs);
                        let either_hand = is_either_hand(&attrs);

                        if let (Some(name), Some(slot)) = (name, slot) {
                            insert_if_highest_level(
                                &mut out,
                                &mut out_levels,
                                name,
                                slot,
                                level,
                                two_handed,
                                either_hand,
                            );
                        }
                    }
                }
            }

            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

fn insert_if_highest_level(
    out: &mut HashMap<String, CachedItem>,
    out_levels: &mut HashMap<String, i32>,
    name: String,
    slot: Slot,
    level: Option<i32>,
    two_handed: bool,
    either_hand: bool,
) {
    let new_level = level.unwrap_or(0);
    let best_level = out_levels.get(&name).copied().unwrap_or(-1);
    if new_level > best_level {
        out_levels.insert(name.clone(), new_level);
        out.insert(
            name.clone(),
            CachedItem {
                name,
                slot,
                two_handed,
                either_hand,
            },
        );
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

/// True when this XML item is an Either-hand item: its raw `slot` attribute is
/// `EITHER_HAND`. Such items are stored with a canonical `Off-hand` slot plus
/// this flag, which marks them equippable in the main hand as well.
fn is_either_hand(attrs: &HashMap<String, String>) -> bool {
    attrs.get("slot").map(|s| s.as_str()) == Some("EITHER_HAND")
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Writes `xml` to a unique temp file and runs `load_items` on it.
    fn load_items_from_str(label: &str, xml: &str) -> HashMap<String, CachedItem> {
        let path = std::env::temp_dir().join(format!(
            "lgo_build_db_test_{}_{}.xml",
            label,
            std::process::id()
        ));
        fs::write(&path, xml).expect("write temp xml");
        let result = load_items(path.to_str().unwrap());
        let _ = fs::remove_file(&path);
        result.expect("load_items should succeed")
    }

    // ── Level preference: highest item level wins on name collision ───────────

    #[test]
    fn highest_level_entry_wins() {
        // Two XML entries for the same item name at different levels; the
        // higher-level entry (distinguishable by its slot) must be the one
        // kept in the output map.
        let xml = r#"<items>
            <item name="Wilful Bracer of the Bear in Winter" level="122" slot="WRIST"></item>
            <item name="Wilful Bracer of the Bear in Winter" level="160" slot="NECK"></item>
        </items>"#;
        let out = load_items_from_str("highest_wins", xml);
        assert_eq!(out["Wilful Bracer of the Bear in Winter"].slot, Slot::Neck);
    }

    #[test]
    fn lower_level_entry_does_not_overwrite_higher() {
        // Same as above but insertion order reversed: high first, then low.
        let xml = r#"<items>
            <item name="Test Item" level="160" slot="NECK"></item>
            <item name="Test Item" level="122" slot="WRIST"></item>
        </items>"#;
        let out = load_items_from_str("no_overwrite", xml);
        assert_eq!(out["Test Item"].slot, Slot::Neck);
    }

    #[test]
    fn mixed_paired_and_self_closing_forms_highest_level_wins() {
        let low_paired_then_high_empty = r#"<items>
            <item name="Mixed Form Item" level="122" slot="WRIST"></item>
            <item name="Mixed Form Item" level="160" slot="NECK"/>
        </items>"#;
        let out = load_items_from_str("mixed_forms_high_empty", low_paired_then_high_empty);
        assert_eq!(out["Mixed Form Item"].slot, Slot::Neck);

        let high_empty_then_low_paired = r#"<items>
            <item name="Mixed Form Item" level="160" slot="NECK"/>
            <item name="Mixed Form Item" level="122" slot="WRIST"></item>
        </items>"#;
        let out = load_items_from_str("mixed_forms_high_first", high_empty_then_low_paired);
        assert_eq!(out["Mixed Form Item"].slot, Slot::Neck);
    }

    // ── Self-closing item elements ─────────────────────────────────────────────

    #[test]
    fn self_closing_item_is_loaded() {
        let xml = r#"<items>
            <item name="Compact Dagger" level="160" slot="MAIN_HAND"/>
        </items>"#;
        let out = load_items_from_str("self_closing_loaded", xml);
        let item = &out["Compact Dagger"];
        assert_eq!(item.slot, Slot::MainHand);
        assert!(!item.two_handed);
    }

    #[test]
    fn self_closing_two_handed_item_sets_flag() {
        let xml = r#"<items>
            <item name="Compact Greatsword" level="160" slot="MAIN_HAND" precludedSlots="OFF_HAND"/>
        </items>"#;
        let out = load_items_from_str("self_closing_two_handed", xml);
        let item = &out["Compact Greatsword"];
        assert_eq!(item.slot, Slot::MainHand);
        assert!(item.two_handed);
    }

    #[test]
    fn self_closing_legendary_weapon_is_skipped() {
        let xml = r#"<items>
            <item name="Compact Legendary" level="160" slot="MAIN_HAND" category="LEGENDARY_WEAPON"/>
        </items>"#;
        let out = load_items_from_str("self_closing_legendary", xml);
        assert!(!out.contains_key("Compact Legendary"));
    }

    // ── Two-handed detection via precludedSlots ────────────────────────────────

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

    // ── Either-hand detection via EITHER_HAND slot ─────────────────────────────

    #[test]
    fn either_hand_item_maps_to_off_hand_with_flag() {
        let xml = r#"<items>
            <item name="Example Rune-stone" level="160" slot="EITHER_HAND"></item>
        </items>"#;
        let out = load_items_from_str("either_hand", xml);
        let item = &out["Example Rune-stone"];
        assert_eq!(item.slot, Slot::OffHand);
        assert!(item.either_hand);
        assert!(!item.two_handed);
    }

    #[test]
    fn off_hand_item_is_not_either_hand() {
        let xml = r#"<items>
            <item name="Example Shield" level="160" slot="OFF_HAND"></item>
        </items>"#;
        let out = load_items_from_str("off_hand_not_either", xml);
        let item = &out["Example Shield"];
        assert_eq!(item.slot, Slot::OffHand);
        assert!(!item.either_hand);
    }

    #[test]
    fn either_hand_flag_serializes_and_absent_flag_is_omitted() {
        // The JSON DB is the transport for either-handedness into
        // resolve-slots: `true` must round-trip and everything else omits it.
        let xml = r#"<items>
            <item name="Example Rune-stone" level="160" slot="EITHER_HAND"></item>
            <item name="Example Shield" level="160" slot="OFF_HAND"></item>
        </items>"#;
        let out = load_items_from_str("either_serialize", xml);
        let json = serde_json::to_string(&out).expect("serialize db");
        let parsed: HashMap<String, CachedItem> = serde_json::from_str(&json).expect("reparse db");
        assert!(parsed["Example Rune-stone"].either_hand);
        assert!(!parsed["Example Shield"].either_hand);
        // `false` is skipped during serialization, so the only occurrence of
        // the key belongs to the rune-stone.
        assert_eq!(json.matches("either_hand").count(), 1);
    }

    #[test]
    fn pooled_family_slots_serialize_unnumbered() {
        let xml = r#"<items>
            <item name="Example Bracelet" level="160" slot="WRIST"></item>
            <item name="Example Ring" level="160" slot="FINGER"></item>
            <item name="Example Earring" level="160" slot="EAR"></item>
        </items>"#;
        let out = load_items_from_str("pooled_slots", xml);
        let json = serde_json::to_string(&out).expect("serialize db");

        assert!(json.contains(r#""slot":"Wrist""#), "{json}");
        assert!(json.contains(r#""slot":"Finger""#), "{json}");
        assert!(json.contains(r#""slot":"Ear""#), "{json}");
        assert!(!json.contains("Wrist1"), "{json}");
        assert!(!json.contains("Finger1"), "{json}");
        assert!(!json.contains("Ear1"), "{json}");
    }

    #[test]
    fn db_slots_serialize_as_display_strings() {
        let xml = r#"<items>
            <item name="Example Sword" level="160" slot="MAIN_HAND"></item>
            <item name="Example Shield" level="160" slot="OFF_HAND"></item>
            <item name="Example Rune" level="160" slot="CLASS_SLOT"></item>
        </items>"#;
        let out = load_items_from_str("display_slots", xml);
        let json = serde_json::to_string(&out).expect("serialize db");

        assert!(json.contains(r#""slot":"Main-hand""#), "{json}");
        assert!(json.contains(r#""slot":"Off-hand""#), "{json}");
        assert!(json.contains(r#""slot":"Class Item""#), "{json}");
        assert!(!json.contains("MainHand"), "{json}");
        assert!(!json.contains("OffHand"), "{json}");
        assert!(!json.contains("ClassItem"), "{json}");
    }
}
