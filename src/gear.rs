use crate::stat::Stat;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

    /// Parse the slot string as it appears in `data/lgo_items.json`
    /// (e.g. `"Head"`, `"Wrist"`, `"MainHand"`, `"ClassItem"`).
    ///
    /// This is the DB-side representation of `Slot`. The pooled families use
    /// one unnumbered external string that maps to the first internal variant.
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
            "Wrist" => Some(Slot::Wrist1),
            "Neck" => Some(Slot::Neck),
            "Finger" => Some(Slot::Finger1),
            "Ear" => Some(Slot::Ear1),
            "Pocket" => Some(Slot::Pocket),
            "MainHand" => Some(Slot::MainHand),
            "OffHand" => Some(Slot::OffHand),
            "Ranged" => Some(Slot::Ranged),
            "ClassItem" => Some(Slot::ClassItem),
            _ => None,
        }
    }

    /// String used for serialized item DB entries.
    pub fn json_variant(self) -> &'static str {
        match self {
            Slot::Head => "Head",
            Slot::Chest => "Chest",
            Slot::Legs => "Legs",
            Slot::Hands => "Hands",
            Slot::Feet => "Feet",
            Slot::Shoulders => "Shoulders",
            Slot::Back => "Back",
            Slot::Wrist1 | Slot::Wrist2 => "Wrist",
            Slot::Neck => "Neck",
            Slot::Finger1 | Slot::Finger2 => "Finger",
            Slot::Ear1 | Slot::Ear2 => "Ear",
            Slot::Pocket => "Pocket",
            Slot::MainHand => "MainHand",
            Slot::OffHand => "OffHand",
            Slot::Ranged => "Ranged",
            Slot::ClassItem => "ClassItem",
        }
    }
    
        /// Canonical display string, used in TOML `slot = "..."` values,
    /// report labels, and anywhere the slot is shown to the user.
    pub fn display_name(self) -> &'static str {
        match self {
            Slot::Head => "Head",
            Slot::Chest => "Chest",
            Slot::Legs => "Legs",
            Slot::Hands => "Hands",
            Slot::Feet => "Feet",
            Slot::Shoulders => "Shoulders",
            Slot::Back => "Back",
            Slot::Wrist1 | Slot::Wrist2 => "Wrist",
            Slot::Neck => "Neck",
            Slot::Finger1 | Slot::Finger2 => "Finger",
            Slot::Ear1 | Slot::Ear2 => "Ear",
            Slot::Pocket => "Pocket",
            Slot::MainHand => "Main-hand",
            Slot::OffHand => "Off-hand",
            Slot::Ranged => "Ranged",
            Slot::ClassItem => "Class Item",
        }
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

/// A single gear item with its stats resolved (from wiki lookup + cache).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GearItem {
    /// Display name as returned by the plugin and used as wiki lookup key.
    pub name: String,
    pub slot: Slot,
    /// True for two-handed `MainHand` weapons, which occupy both hand slots
    /// and forbid any `OffHand` selection. Sourced from `precludedSlots` in
    /// `data/items.xml`; false (and omitted from JSON) for everything else.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub two_handed: bool,
    /// All stats on this item. Missing stats are treated as 0.
    pub stats: HashMap<Stat, i64>,
}

/// One entry in the offline items DB (`data/lgo_items.json`): the item's
/// canonical slot plus the `two_handed` flag. Item stats come from the
/// bookmarklet, not the DB, so no stats are carried here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedItem {
    /// Display name as it appears in `data/items.xml`; also the map key.
    pub name: String,
    #[serde(
        serialize_with = "serialize_json_slot",
        deserialize_with = "deserialize_json_slot"
    )]
    pub slot: Slot,
    /// True for two-handed `MainHand` weapons (from `precludedSlots` in
    /// `data/items.xml`); false (and omitted from JSON) for everything else.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub two_handed: bool,
}

fn serialize_json_slot<S>(slot: &Slot, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(slot.json_variant())
}

fn deserialize_json_slot<'de, D>(deserializer: D) -> Result<Slot, D::Error>
where
    D: Deserializer<'de>,
{
    let slot = String::deserialize(deserializer)?;
    Slot::from_json_variant(&slot)
        .ok_or_else(|| serde::de::Error::custom(format!("unknown slot '{}'", slot)))
}

/// Synthetic optimizer key for one TOML item instance.
///
/// Display names are not unique: a user may own multiple copies of the same
/// item, so optimizer identity must include the instance's document index.
pub fn optimizer_candidate_key(idx: usize, item: &GearItem) -> String {
    format!("{:04}::{}::{}", idx, item.slot, item.name)
}

impl GearItem {
    /// Return the value of a stat, or 0 if not present.
    pub fn stat(&self, s: &Stat) -> i64 {
        self.stats.get(s).copied().unwrap_or(0)
    }
}

/// A candidate gear set: exactly one item per slot.
#[derive(Debug, Clone)]
pub struct GearSet {
    pub innate_stats: HashMap<Stat, i64>,
    pub items: HashMap<Slot, GearItem>,
}

impl GearSet {
    pub fn new(innate_stats: HashMap<Stat, i64>) -> Self {
        GearSet {
            innate_stats,
            items: HashMap::new(),
        }
    }

    /// Sum a single stat across innate totals and all equipped items.
    pub fn total(&self, s: &Stat) -> i64 {
        self.innate_stats.get(s).copied().unwrap_or(0)
            + self.items.values().map(|item| item.stat(s)).sum::<i64>()
    }
}

impl Default for GearSet {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every external JSON-form slot must translate to the canonical display form.
    #[test]
    fn from_json_variant_translates_external_slots() {
        // (json_form, canonical_form)
        let pairs: &[(&str, &str)] = &[
            ("Head", "Head"),
            ("Chest", "Chest"),
            ("Legs", "Legs"),
            ("Hands", "Hands"),
            ("Feet", "Feet"),
            ("Shoulders", "Shoulders"),
            ("Back", "Back"),
            ("Wrist", "Wrist"),
            ("Neck", "Neck"),
            ("Finger", "Finger"),
            ("Ear", "Ear"),
            ("Pocket", "Pocket"),
            ("MainHand", "Main-hand"),
            ("OffHand", "Off-hand"),
            ("Ranged", "Ranged"),
            ("ClassItem", "Class Item"),
        ];

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

    /// Spot-checks for variants whose DB slot string differs from the
    /// display form. If any of these regress, the resolver will silently
    /// emit wrong slot strings.
    #[test]
    fn db_string_and_display_differences_translate_correctly() {
        assert_eq!(
            format!("{}", Slot::from_json_variant("MainHand").unwrap()),
            "Main-hand"
        );
        assert_eq!(
            format!("{}", Slot::from_json_variant("OffHand").unwrap()),
            "Off-hand"
        );
        assert_eq!(
            format!("{}", Slot::from_json_variant("ClassItem").unwrap()),
            "Class Item"
        );
    }

    /// Every internal variant's DB slot string must round-trip through
    /// from_json_variant back to the first variant of its family. This
    /// guarantees Slot::ALL, json_variant, and from_json_variant can
    /// never drift apart.
    #[test]
    fn json_variant_round_trips_to_family_first_variant_for_all_slots() {
        for &slot in Slot::ALL.iter() {
            let expected = match slot {
                Slot::Wrist2 => Slot::Wrist1,
                Slot::Finger2 => Slot::Finger1,
                Slot::Ear2 => Slot::Ear1,
                other => other,
            };
            let parsed = Slot::from_json_variant(slot.json_variant())
                .unwrap_or_else(|| {
                    panic!(
                        "from_json_variant rejected json_variant of {:?} ({:?})",
                        slot,
                        slot.json_variant()
                    )
                });
            assert_eq!(
                parsed, expected,
                "{:?} must round-trip to its family's first variant",
                slot
            );
        }
    }

    #[test]
    fn json_variant_uses_unnumbered_pooled_families() {
        assert_eq!(Slot::Wrist1.json_variant(), "Wrist");
        assert_eq!(Slot::Wrist2.json_variant(), "Wrist");
        assert_eq!(Slot::Finger1.json_variant(), "Finger");
        assert_eq!(Slot::Finger2.json_variant(), "Finger");
        assert_eq!(Slot::Ear1.json_variant(), "Ear");
        assert_eq!(Slot::Ear2.json_variant(), "Ear");
    }

    /// Defensive: anything not in the external slot table must be rejected,
    /// not silently accepted as a near-match.
    #[test]
    fn from_json_variant_rejects_unknown_inputs() {
        // Empty
        assert!(Slot::from_json_variant("").is_none());
        // Canonical form (with spaces/parens) is NOT the JSON form
        assert!(Slot::from_json_variant("Wrist (1)").is_none());
        assert!(Slot::from_json_variant("Wrist1").is_none());
        assert!(Slot::from_json_variant("Main-hand").is_none());
        assert!(Slot::from_json_variant("Class Item").is_none());
        // Wrong case
        assert!(Slot::from_json_variant("head").is_none());
        assert!(Slot::from_json_variant("WRIST").is_none());
        // Excluded LotRO slots
        assert!(Slot::from_json_variant("CraftItem").is_none());
        assert!(Slot::from_json_variant("Bridle").is_none());
        // Plain garbage
        assert!(Slot::from_json_variant("Unknown").is_none());
        assert!(Slot::from_json_variant("Wrist3").is_none());
    }

    #[test]
    fn total_counts_innate_stats_once() {
        let mut innate = HashMap::new();
        innate.insert(Stat::CriticalRating, 100);
        let mut gear_set = GearSet::new(innate);
        gear_set.items.insert(
            Slot::Head,
            GearItem {
                name: "Test Helm".to_string(),
                slot: Slot::Head,
                two_handed: false,
                stats: [(Stat::CriticalRating, 25)].into_iter().collect(),
            },
        );

        assert_eq!(gear_set.total(&Stat::CriticalRating), 125);
    }
}
