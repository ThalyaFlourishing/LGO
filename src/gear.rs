use crate::stat::Stat;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy)]
struct SlotRow {
    slot: Slot,
    display_str: &'static str,
}

const SLOT_TABLE: &[SlotRow] = &[
    SlotRow {
        slot: Slot::Head,
        display_str: "Head",
    },
    SlotRow {
        slot: Slot::Chest,
        display_str: "Chest",
    },
    SlotRow {
        slot: Slot::Legs,
        display_str: "Legs",
    },
    SlotRow {
        slot: Slot::Hands,
        display_str: "Hands",
    },
    SlotRow {
        slot: Slot::Feet,
        display_str: "Feet",
    },
    SlotRow {
        slot: Slot::Shoulders,
        display_str: "Shoulders",
    },
    SlotRow {
        slot: Slot::Back,
        display_str: "Back",
    },
    SlotRow {
        slot: Slot::Wrist1,
        display_str: "Wrist",
    },
    SlotRow {
        slot: Slot::Wrist2,
        display_str: "Wrist",
    },
    SlotRow {
        slot: Slot::Neck,
        display_str: "Neck",
    },
    SlotRow {
        slot: Slot::Finger1,
        display_str: "Finger",
    },
    SlotRow {
        slot: Slot::Finger2,
        display_str: "Finger",
    },
    SlotRow {
        slot: Slot::Ear1,
        display_str: "Ear",
    },
    SlotRow {
        slot: Slot::Ear2,
        display_str: "Ear",
    },
    SlotRow {
        slot: Slot::Pocket,
        display_str: "Pocket",
    },
    SlotRow {
        slot: Slot::MainHand,
        display_str: "Main-hand",
    },
    SlotRow {
        slot: Slot::OffHand,
        display_str: "Off-hand",
    },
    SlotRow {
        slot: Slot::Ranged,
        display_str: "Ranged",
    },
    SlotRow {
        slot: Slot::ClassItem,
        display_str: "Class Item",
    },
];

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
    pub fn all() -> impl ExactSizeIterator<Item = Slot> {
        SLOT_TABLE.iter().map(|row| row.slot)
    }

    /// Canonical display string, used in TOML `slot = "..."` values,
    /// item DB entries, report labels, and anywhere the slot is shown to the
    /// user.
    pub fn display_name(self) -> &'static str {
        SLOT_TABLE
            .iter()
            .find(|row| row.slot == self)
            .map(|row| row.display_str)
            .expect("every Slot variant must appear in SLOT_TABLE")
    }
}

/// Parse a canonical external slot string back to a Slot variant.
///
/// Pooled-family strings map to the first internal variant (`Wrist1`,
/// `Finger1`, `Ear1`) by table order.
pub fn parse_slot_display(s: &str) -> Option<Slot> {
    SLOT_TABLE
        .iter()
        .find(|row| row.display_str == s)
        .map(|row| row.slot)
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
    serializer.serialize_str(slot.display_name())
}

fn deserialize_json_slot<'de, D>(deserializer: D) -> Result<Slot, D::Error>
where
    D: Deserializer<'de>,
{
    let slot = String::deserialize(deserializer)?;
    parse_slot_display(&slot)
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

    #[test]
    fn display_string_round_trips_to_family_first_variant_for_all_slots() {
        for slot in Slot::all() {
            let expected = match slot {
                Slot::Wrist2 => Slot::Wrist1,
                Slot::Finger2 => Slot::Finger1,
                Slot::Ear2 => Slot::Ear1,
                other => other,
            };
            let parsed = parse_slot_display(slot.display_name()).unwrap_or_else(|| {
                panic!(
                    "parse_slot_display rejected display_name of {:?} ({:?})",
                    slot,
                    slot.display_name()
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
    fn display_string_uses_unnumbered_pooled_families() {
        assert_eq!(Slot::Wrist1.display_name(), "Wrist");
        assert_eq!(Slot::Wrist2.display_name(), "Wrist");
        assert_eq!(Slot::Finger1.display_name(), "Finger");
        assert_eq!(Slot::Finger2.display_name(), "Finger");
        assert_eq!(Slot::Ear1.display_name(), "Ear");
        assert_eq!(Slot::Ear2.display_name(), "Ear");
    }

    /// Defensive: anything not in the external slot table must be rejected,
    /// not silently accepted as a near-match.
    #[test]
    fn parse_slot_display_rejects_unknown_inputs() {
        assert!(parse_slot_display("").is_none());
        assert!(parse_slot_display("MainHand").is_none());
        assert!(parse_slot_display("OffHand").is_none());
        assert!(parse_slot_display("ClassItem").is_none());
        assert!(parse_slot_display("Wrist1").is_none());
        assert!(parse_slot_display("CraftItem").is_none());
        assert!(parse_slot_display("Bridle").is_none());
        assert!(parse_slot_display("Unknown").is_none());
        assert!(parse_slot_display("head").is_none());
        assert!(parse_slot_display("WRIST").is_none());
        assert!(parse_slot_display("Wrist (1)").is_none());
        assert!(parse_slot_display("Wrist3").is_none());
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
