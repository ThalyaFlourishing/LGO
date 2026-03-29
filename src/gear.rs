use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::stat::Stat;

/// Equipment slots that the optimizer considers.
/// Excluded: MainHand (16), CraftItem (19), ClassItem (20), Bridle (21).
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
    OffHand,
    Ranged,
}

impl Slot {
    pub const ALL: &'static [Slot] = &[
        Slot::Head, Slot::Chest, Slot::Legs, Slot::Hands, Slot::Feet,
        Slot::Shoulders, Slot::Back, Slot::Wrist1, Slot::Wrist2, Slot::Neck,
        Slot::Finger1, Slot::Finger2, Slot::Ear1, Slot::Ear2,
        Slot::Pocket, Slot::OffHand, Slot::Ranged,
    ];

    /// Map the integer slot index returned by the LotRO plugin to a Slot.
    /// The float `12.000000` form is handled by the caller casting f64 to u32.
    /// Returns None for excluded or unrecognised slot indices.
    pub fn from_plugin_index(n: u32) -> Option<Slot> {
        match n {
            1  => Some(Slot::Head),
            2  => Some(Slot::Chest),
            3  => Some(Slot::Legs),
            4  => Some(Slot::Hands),
            5  => Some(Slot::Feet),
            6  => Some(Slot::Shoulders),
            7  => Some(Slot::Back),
            8  => Some(Slot::Wrist1),
            9  => Some(Slot::Wrist2),
            10 => Some(Slot::Neck),
            11 => Some(Slot::Finger1),
            12 => Some(Slot::Finger2),
            13 => Some(Slot::Ear1),
            14 => Some(Slot::Ear2),
            15 => Some(Slot::Pocket),
            17 => Some(Slot::OffHand),
            18 => Some(Slot::Ranged),
            // 16 = MainHand, 19 = CraftItem, 20 = ClassItem, 21 = Bridle — excluded
            _  => None,
        }
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Slot::Head      => "Head",
            Slot::Chest     => "Chest",
            Slot::Legs      => "Legs",
            Slot::Hands     => "Hands",
            Slot::Feet      => "Feet",
            Slot::Shoulders => "Shoulders",
            Slot::Back      => "Back",
            Slot::Wrist1    => "Wrist (1)",
            Slot::Wrist2    => "Wrist (2)",
            Slot::Neck      => "Neck",
            Slot::Finger1   => "Finger (1)",
            Slot::Finger2   => "Finger (2)",
            Slot::Ear1      => "Ear (1)",
            Slot::Ear2      => "Ear (2)",
            Slot::Pocket    => "Pocket",
            Slot::OffHand   => "Off-hand",
            Slot::Ranged    => "Ranged",
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
        GearSet { items: HashMap::new() }
    }

    /// Sum a single stat across all equipped items.
    pub fn total(&self, s: &Stat) -> i64 {
        self.items.values().map(|item| item.stat(s)).sum()
    }
}