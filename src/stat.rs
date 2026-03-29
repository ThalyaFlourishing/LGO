use std::fmt;
use std::str::FromStr;

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
    TactMastery,
    PhysMastery,

    // Support stats
    Morale,
    Power,
    IncomingHealing,
    OutgoingHealing,
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Stat::Might              => "Might",
            Stat::Agility            => "Agility",
            Stat::Vitality           => "Vitality",
            Stat::Will               => "Will",
            Stat::Fate               => "Fate",
            Stat::Armor              => "Armor",
            Stat::Resistance         => "Resistance",
            Stat::CritDefense        => "Crit Defense",
            Stat::IncMitigations     => "Inc. Mitigations",
            Stat::PhysMitigation     => "Phys. Mitigation",
            Stat::TactMitigation     => "Tact. Mitigation",
            Stat::CritRating         => "Crit Rating",
            Stat::DevRating          => "Dev Rating",
            Stat::FinesseRating      => "Finesse Rating",
            Stat::OffensiveOverpower => "Offensive Overpower",
            Stat::TactMastery        => "Tact. Mastery",
            Stat::PhysMastery        => "Phys. Mastery",
            Stat::Morale             => "Morale",
            Stat::Power              => "Power",
            Stat::IncomingHealing    => "Incoming Healing",
            Stat::OutgoingHealing    => "Outgoing Healing",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for Stat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace([' ', '_', '-'], "").as_str() {
            "might"                                          => Ok(Stat::Might),
            "agility"                                        => Ok(Stat::Agility),
            "vitality"                                       => Ok(Stat::Vitality),
            "will"                                           => Ok(Stat::Will),
            "fate"                                           => Ok(Stat::Fate),
            "armor" | "armour"                               => Ok(Stat::Armor),
            "resistance"                                     => Ok(Stat::Resistance),
            "critdefense" | "critdefence"                    => Ok(Stat::CritDefense),
            "incmitigations" | "incmit"                      => Ok(Stat::IncMitigations),
            "physmitigation" | "physicalmitigation" |
            "physmit"                                        => Ok(Stat::PhysMitigation),
            "tactmitigation" | "tacticalmitigation" |
            "tactmit"                                        => Ok(Stat::TactMitigation),
            "critrating"                                     => Ok(Stat::CritRating),
            "devrating" | "devastatingcriticalrating"        => Ok(Stat::DevRating),
            "finesserating" | "finesse"                      => Ok(Stat::FinesseRating),
            "offensiveoverpower" | "overpower"               => Ok(Stat::OffensiveOverpower),
            "tactmastery" | "tacticalmastery" | "tactmast"   => Ok(Stat::TactMastery),
            "physmastery" | "physicalmastery" | "physmast"   => Ok(Stat::PhysMastery),
            "morale"                                         => Ok(Stat::Morale),
            "power"                                          => Ok(Stat::Power),
            "incominghealing" | "incheal"                    => Ok(Stat::IncomingHealing),
            "outgoinghealing" | "outheal"                    => Ok(Stat::OutgoingHealing),
            _ => Err(format!("Unknown stat: '{}'", s)),
        }
    }
}

// -- StatGoal ------------------------------------------------------------------

/// A stat with an associated minimum value, parsed from CLI input.
/// Format: `StatName:minimum`  e.g. `CritRating:450`
/// A minimum of 0 means "maximise but no floor required".
#[derive(Debug, Clone)]
pub struct StatGoal {
    pub stat:    Stat,
    pub minimum: i64,
}

impl FromStr for StatGoal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            Some((stat_str, min_str)) => {
                let stat = stat_str.parse::<Stat>()?;
                let minimum = min_str.parse::<i64>()
                    .map_err(|_| format!("Invalid minimum '{}' in goal '{}'", min_str, s))?;
                Ok(StatGoal { stat, minimum })
            }
            None => {
                // No colon - treat as stat name with minimum 0.
                let stat = s.parse::<Stat>()?;
                Ok(StatGoal { stat, minimum: 0 })
            }
        }
    }
}

// -- StatWeights ---------------------------------------------------------------

/// A weighting map from stats to their relative importance (0.0-1.0).
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