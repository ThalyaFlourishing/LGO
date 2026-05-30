use crate::stat::Stat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Equipment slots that the optimizer considers.
/// Excluded: CraftItem (19), Bridle (21).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Slot {
    Head,
    Chest,
    Legs,
    Hands,
    Feet,
    Shoulders,
    Back,
    Wrist1,
    Wrist2,
    Neck,
    Finger1,
    Finger2,
    Ear1,
    Ear2,
    Pocket,
    MainHand,
    OffHand,
    Ranged,
    ClassItem,
}

impl Slot {
    pub const ALL: &'static [Slot] = &[
        Slot::Head,
        Slot::Chest,
        Slot::Legs,
        Slot::Hands,
        Slot::Feet,
        Slot::Shoulders,
        Slot::Back,
        Slot::Wrist1,
        Slot::Wrist2,
        Slot::Neck,
        Slot::Finger1,
        Slot::Finger2,
        Slot::Ear1,
        Slot::Ear2,
        Slot::Pocket,
        Slot::MainHand,
        Slot::OffHand,
        Slot::Ranged,
        Slot::ClassItem,
    ];

    /// Map the integer slot index returned by the LotRO plugin to a Slot.
    /// The float `12.000000` form is handled by the caller casting f64 to u32.
    /// Returns None for excluded or unrecognised slot indices.
    pub fn from_plugin_index(n: u32) -> Option<Slot> {
        match n {
            1 => Some(Slot::Head),
            2 => Some(Slot::Chest),
            3 => Some(Slot::Legs),
            4 => Some(Slot::Hands),
            5 => Some(Slot::Feet),
            6 => Some(Slot::Shoulders),
            7 => Some(Slot::Back),
            8 => Some(Slot::Wrist1),
            9 => Some(Slot::Wrist2),
            10 => Some(Slot::Neck),
            11 => Some(Slot::Finger1),
            12 => Some(Slot::Finger2),
            13 => Some(Slot::Ear1),
            14 => Some(Slot::Ear2),
            15 => Some(Slot::Pocket),
            16 => Some(Slot::MainHand),
            17 => Some(Slot::OffHand),
            18 => Some(Slot::Ranged),
            20 => Some(Slot::ClassItem),
            // 19 = CraftItem, 21 = Bridle — excluded
            _ => None,
        }
    }

    /// Parse the bare PascalCase variant name as it appears in
    /// `data/lgo_items.json` (e.g. `"Head"`, `"Wrist1"`, `"MainHand"`,
    /// `"ClassItem"`).
    ///
    /// This is the JSON-side (serde default) representation of `Slot`,
    /// which is *not* the same as `Display` (the canonical display form
    /// used in the `.toml` and accepted by `gearstats::parse_slot_str`).
    /// The resolver uses this to translate JSON → canonical at load time.
    ///
    /// Returns `None` for any unrecognised input.
    pub fn from_json_variant(s: &str) -> Option<Slot> {
        match s {
            "Head" => Some(Slot::Head),
            "Chest" => Some(Slot::Chest),
            "Legs" => Some(Slot::Legs),
            "Hands" => Some(Slot::Hands),
            "Feet" => Some(Slot::Feet),
            "Shoulders" => Some(Slot::Shoulders),
            "Back" => Some(Slot::Back),
            "Wrist1" => Some(Slot::Wrist1),
            "Wrist2" => Some(Slot::Wrist2),
            "Neck" => Some(Slot::Neck),
            "Finger1" => Some(Slot::Finger1),
            "Finger2" => Some(Slot::Finger2),
            "Ear1" => Some(Slot::Ear1),
            "Ear2" => Some(Slot::Ear2),
            "Pocket" => Some(Slot::Pocket),
            "MainHand" => Some(Slot::MainHand),
            "OffHand" => Some(Slot::OffHand),
            "Ranged" => Some(Slot::Ranged),
            "ClassItem" => Some(Slot::ClassItem),
            _ => None,
        }
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Slot::Head => "Head",
            Slot::Chest => "Chest",
            Slot::Legs => "Legs",
            Slot::Hands => "Hands",
            Slot::Feet => "Feet",
            Slot::Shoulders => "Shoulders",
            Slot::Back => "Back",
            Slot::Wrist1 => "Wrist (1)",
            Slot::Wrist2 => "Wrist (2)",
            Slot::Neck => "Neck",
            Slot::Finger1 => "Finger (1)",
            Slot::Finger2 => "Finger (2)",
            Slot::Ear1 => "Ear (1)",
            Slot::Ear2 => "Ear (2)",
            Slot::Pocket => "Pocket",
            Slot::MainHand => "Main-hand",
            Slot::OffHand => "Off-hand",
            Slot::Ranged => "Ranged",
            Slot::ClassItem => "Class Item",
        };
        write!(f, "{}", s)
    }
}

/// A single gear item with its stats resolved (from wiki lookup + cache).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GearItem {
    /// Display name as returned by the plugin and used as wiki lookup key.
    pub name: String,
    pub slot: Slot,
    /// All stats on this item. Missing stats are treated as 0.
    pub stats: HashMap<Stat, i64>,
}

pub type CachedItem = GearItem;

impl GearItem {
    /// Return the value of a stat, or 0 if not present.
    pub fn stat(&self, s: &Stat) -> i64 {
        self.stats.get(s).copied().unwrap_or(0)
    }
}

/// A candidate gear set: exactly one item per slot.
#[derive(Debug, Clone)]
pub struct GearSet {
    pub items: HashMap<Slot, GearItem>,
}

impl GearSet {
    pub fn new() -> Self {
        GearSet {
            items: HashMap::new(),
        }
    }

    /// Sum a single stat across all equipped items.
    pub fn total(&self, s: &Stat) -> i64 {
        self.items.values().map(|item| item.stat(s)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every Slot variant must round-trip JSON-form ↔ canonical-form.
    /// Catches drift between Display (canonical) and from_json_variant
    /// (JSON form), and between either of those and Slot::ALL.
    #[test]
    fn from_json_variant_round_trips_for_all_slots() {
        // (json_form, canonical_form)
        let pairs: &[(&str, &str)] = &[
            ("Head",      "Head"),
            ("Chest",     "Chest"),
            ("Legs",      "Legs"),
            ("Hands",     "Hands"),
            ("Feet",      "Feet"),
            ("Shoulders", "Shoulders"),
            ("Back",      "Back"),
            ("Wrist1",    "Wrist (1)"),
            ("Wrist2",    "Wrist (2)"),
            ("Neck",      "Neck"),
            ("Finger1",   "Finger (1)"),
            ("Finger2",   "Finger (2)"),
            ("Ear1",      "Ear (1)"),
            ("Ear2",      "Ear (2)"),
            ("Pocket",    "Pocket"),
            ("MainHand",  "Main-hand"),
            ("OffHand",   "Off-hand"),
            ("Ranged",    "Ranged"),
            ("ClassItem", "Class Item"),
        ];

        // Sanity: the table covers exactly the 19 ALL slots.
        assert_eq!(pairs.len(), Slot::ALL.len());

        for (json_form, canonical_form) in pairs {
            let slot = Slot::from_json_variant(json_form)
                .unwrap_or_else(|| panic!("from_json_variant rejected {:?}", json_form));
            assert_eq!(
                format!("{}", slot),
                *canonical_form,
                "Display impl for {:?} != expected canonical form",
                slot
            );
            assert!(
                Slot::ALL.contains(&slot),
                "Slot::{:?} (from {:?}) is not in Slot::ALL",
                slot,
                json_form
            );
        }
    }

    /// Specific spot-checks for the four variants whose JSON form differs
    /// most from the canonical form. If any of these regress, the resolver
    /// will silently emit wrong slot strings.
    #[test]
    fn punctuation_variants_translate_correctly() {
        assert_eq!(
            format!("{}", Slot::from_json_variant("Wrist1").unwrap()),
            "Wrist (1)"
        );
        assert_eq!(
            format!("{}", Slot::from_json_variant("Finger2").unwrap()),
            "Finger (2)"
        );
        assert_eq!(
            format!("{}", Slot::from_json_variant("MainHand").unwrap()),
            "Main-hand"
        );
        assert_eq!(
            format!("{}", Slot::from_json_variant("ClassItem").unwrap()),
            "Class Item"
        );
    }

    /// Defensive: anything not in the 19-variant table must be rejected,
    /// not silently accepted as a near-match.
    #[test]
    fn from_json_variant_rejects_unknown_inputs() {
        // Empty
        assert!(Slot::from_json_variant("").is_none());
        // Canonical form (with spaces/parens) is NOT the JSON form
        assert!(Slot::from_json_variant("Wrist (1)").is_none());
        assert!(Slot::from_json_variant("Main-hand").is_none());
        assert!(Slot::from_json_variant("Class Item").is_none());
        // Wrong case
        assert!(Slot::from_json_variant("head").is_none());
        assert!(Slot::from_json_variant("WRIST1").is_none());
        // Excluded LotRO slots
        assert!(Slot::from_json_variant("CraftItem").is_none());
        assert!(Slot::from_json_variant("Bridle").is_none());
        // Plain garbage
        assert!(Slot::from_json_variant("Unknown").is_none());
        assert!(Slot::from_json_variant("Wrist3").is_none());
    }
}
