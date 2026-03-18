use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::stat::Stat;

/// Equipment slots matching the LOTRO Turbine.Gameplay.EquipmentSlot API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    Head,
    Chest,
    Legs,
    Hands,
    Feet,
    Shoulders,
    Back,
    Neck,
    Ear1,
    Ear2,
    Finger1,
    Finger2,
    Wrist1,
    Wrist2,
    MainHand,
    OffHand,
    Pocketed,
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Slot::Head => "Head",
            Slot::Chest => "Chest",
            Slot::Legs => "Legs",
            Slot::Hands => "Hands",
            Slot::Feet => "Feet",
            Slot::Shoulders => "Shoulders",
            Slot::Back => "Back",
            Slot::Neck => "Neck",
            Slot::Ear1 => "Ear (1)",
            Slot::Ear2 => "Ear (2)",
            Slot::Finger1 => "Finger (1)",
            Slot::Finger2 => "Finger (2)",
            Slot::Wrist1 => "Wrist (1)",
            Slot::Wrist2 => "Wrist (2)",
            Slot::MainHand => "Main-hand",
            Slot::OffHand => "Off-hand",
            Slot::Pocketed => "Pocketed",
        };
        write!(f, "{}", name)
    }
}

/// A single piece of gear with its name, slot, item level, and stat bonuses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GearItem {
    pub name: String,
    pub slot: Slot,
    pub item_level: u32,
    pub stats: HashMap<Stat, i64>,
}

impl GearItem {
    /// Compute a weighted score for this item given a set of stat weights.
    pub fn score(&self, weights: &crate::stat::StatWeights) -> f64 {
        self.stats
            .iter()
            .map(|(stat, &value)| weights.weight_of(stat) * value as f64)
            .sum()
    }
}

/// A complete set of equipped gear, one item per slot.
#[derive(Debug, Clone, Default)]
pub struct GearSet {
    pub items: HashMap<Slot, GearItem>,
}

impl GearSet {
    /// Compute the total weighted score for the entire gear set.
    pub fn score(&self, weights: &crate::stat::StatWeights) -> f64 {
        self.items.values().map(|item| item.score(weights)).sum()
    }

    /// Sum all stat values across the equipped items.
    pub fn total_stats(&self) -> HashMap<Stat, i64> {
        let mut totals: HashMap<Stat, i64> = HashMap::new();
        for item in self.items.values() {
            for (stat, &value) in &item.stats {
                *totals.entry(*stat).or_insert(0) += value;
            }
        }
        totals
    }
}
