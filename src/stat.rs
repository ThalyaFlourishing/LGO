use std::fmt;
use serde::{Deserialize, Serialize};

/// Primary and secondary stats available on LOTRO gear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stat {
    // Primary stats
    Might,
    Agility,
    Vitality,
    Will,
    Fate,

    // Defensive stats
    Armor,
    Resistance,
    CritDefense,
    IncMitigations,
    PhysMitigation,
    TactMitigation,

    // Offensive stats
    CritRating,
    DevRating,
    FinesseRating,
    OffensiveOverpower,

    // Support stats
    Morale,
    Power,
    IncomingHealing,
    OutgoingHealing,
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Stat::Might => "Might",
            Stat::Agility => "Agility",
            Stat::Vitality => "Vitality",
            Stat::Will => "Will",
            Stat::Fate => "Fate",
            Stat::Armor => "Armor",
            Stat::Resistance => "Resistance",
            Stat::CritDefense => "Crit Defense",
            Stat::IncMitigations => "Inc. Mitigations",
            Stat::PhysMitigation => "Phys. Mitigation",
            Stat::TactMitigation => "Tact. Mitigation",
            Stat::CritRating => "Crit Rating",
            Stat::DevRating => "Dev Rating",
            Stat::FinesseRating => "Finesse Rating",
            Stat::OffensiveOverpower => "Offensive Overpower",
            Stat::Morale => "Morale",
            Stat::Power => "Power",
            Stat::IncomingHealing => "Incoming Healing",
            Stat::OutgoingHealing => "Outgoing Healing",
        };
        write!(f, "{}", name)
    }
}

/// A weighting map from stats to their relative importance (0.0–1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatWeights(pub Vec<(Stat, f64)>);

impl StatWeights {
    pub fn weight_of(&self, stat: &Stat) -> f64 {
        self.0
            .iter()
            .find(|(s, _)| s == stat)
            .map(|(_, w)| *w)
            .unwrap_or(0.0)
    }
}
