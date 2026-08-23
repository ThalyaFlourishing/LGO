use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Primary and secondary stats available on LOTRO gear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stat {
    // Primary stats (internal use / wiki parsing only — not tracked in stats file)
    Might,
    Agility,
    Vitality,
    Will,
    Fate,
    Morale,
    Power,
    DevRating,
    OffensiveOverpower,
    IncMitigations,

    // Tracked stats (appear in stats file and optimizer)
    Armor,
    CriticalRating,
    Finesse,
    PhysicalMastery,
    TacticalMastery,
    OutgoingHealing,
    Resistance,
    CriticalDefense,
    IncomingHealing,
    Block,
    Parry,
    Evade,
    PhysicalMitigation,
    TacticalMitigation,
}

/// Raw primary stats read by the resolver as internal derivation inputs.
pub const BASE_STATS: &[(Stat, &str)] = &[
    (Stat::Might, "Might"),
    (Stat::Agility, "Agility"),
    (Stat::Vitality, "Vitality"),
    (Stat::Will, "Will"),
    (Stat::Fate, "Fate"),
];

/// Optimizer-facing stats that may appear in the canonical stats file, in
/// canonical display order. Each entry is (Stat, PascalCase TOML key).
pub const TRACKED_STATS: &[(Stat, &str)] = &[
    (Stat::Morale, "Morale"),
    (Stat::Power, "Power"),
    (Stat::Armor, "Armor"),
    (Stat::CriticalRating, "CriticalRating"),
    (Stat::Finesse, "Finesse"),
    (Stat::PhysicalMastery, "PhysicalMastery"),
    (Stat::TacticalMastery, "TacticalMastery"),
    (Stat::OutgoingHealing, "OutgoingHealing"),
    (Stat::Resistance, "Resistance"),
    (Stat::CriticalDefense, "CriticalDefense"),
    (Stat::IncomingHealing, "IncomingHealing"),
    (Stat::Block, "Block"),
    (Stat::Parry, "Parry"),
    (Stat::Evade, "Evade"),
    (Stat::PhysicalMitigation, "PhysicalMitigation"),
    (Stat::TacticalMitigation, "TacticalMitigation"),
];

/// Returns the canonical two-letter CLI abbreviation for tracked stats.
///
/// Mitigation uses `pt`/`tt` to avoid colliding with the mastery abbreviations
/// `pm`/`tm`.
pub fn abbreviation_for(stat: Stat) -> Option<&'static str> {
    match stat {
        Stat::Morale => Some("ml"),
        Stat::Power => Some("pw"),
        Stat::Armor => Some("am"),
        Stat::CriticalRating => Some("cr"),
        Stat::Finesse => Some("fn"),
        Stat::PhysicalMastery => Some("pm"),
        Stat::TacticalMastery => Some("tm"),
        Stat::OutgoingHealing => Some("oh"),
        Stat::Resistance => Some("rs"),
        Stat::CriticalDefense => Some("cd"),
        Stat::IncomingHealing => Some("ih"),
        Stat::Block => Some("bl"),
        Stat::Parry => Some("pa"),
        Stat::Evade => Some("ev"),
        Stat::PhysicalMitigation => Some("pt"),
        Stat::TacticalMitigation => Some("tt"),
        _ => None,
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            // Internal stats
            Stat::Might => "Might",
            Stat::Agility => "Agility",
            Stat::Vitality => "Vitality",
            Stat::Will => "Will",
            Stat::Fate => "Fate",
            Stat::Morale => "Morale",
            Stat::Power => "Power",
            Stat::DevRating => "Dev Rating",
            Stat::OffensiveOverpower => "Offensive Overpower",
            Stat::IncMitigations => "Inc. Mitigations",
            // Tracked stats
            Stat::Armor => "Armour",
            Stat::CriticalRating => "Critical Rating",
            Stat::Finesse => "Finesse",
            Stat::PhysicalMastery => "Physical Mastery",
            Stat::TacticalMastery => "Tactical Mastery",
            Stat::OutgoingHealing => "Outgoing Healing",
            Stat::Resistance => "Resistance",
            Stat::CriticalDefense => "Critical Defense",
            Stat::IncomingHealing => "Incoming Healing",
            Stat::Block => "Block",
            Stat::Parry => "Parry",
            Stat::Evade => "Evade",
            Stat::PhysicalMitigation => "Physical Mitigation",
            Stat::TacticalMitigation => "Tactical Mitigation",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for Stat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace([' ', '_', '-'], "").as_str() {
            // Two-letter abbreviations
            "am" => Ok(Stat::Armor),
            "ml" => Ok(Stat::Morale),
            "pw" => Ok(Stat::Power),
            "cr" => Ok(Stat::CriticalRating),
            "fn" => Ok(Stat::Finesse),
            "pm" => Ok(Stat::PhysicalMastery),
            "tm" => Ok(Stat::TacticalMastery),
            "oh" => Ok(Stat::OutgoingHealing),
            "rs" => Ok(Stat::Resistance),
            "cd" => Ok(Stat::CriticalDefense),
            "ih" => Ok(Stat::IncomingHealing),
            "bl" => Ok(Stat::Block),
            "pa" => Ok(Stat::Parry),
            "ev" => Ok(Stat::Evade),
            "pt" => Ok(Stat::PhysicalMitigation),
            "tt" => Ok(Stat::TacticalMitigation),

            // Tracked stats — full and legacy names
            "armor" | "armour" => Ok(Stat::Armor),
            "criticalrating" | "critrating" => Ok(Stat::CriticalRating),
            "finesse" | "finesserating" => Ok(Stat::Finesse),
            "physicalmastery" | "physmastery" | "physmast" => Ok(Stat::PhysicalMastery),
            "tacticalmastery" | "tactmastery" | "tactmast" => Ok(Stat::TacticalMastery),
            "outgoinghealing" | "outheal" => Ok(Stat::OutgoingHealing),
            "resistance" => Ok(Stat::Resistance),
            "criticaldefense" | "criticaldefence" | "critdefense" | "critdefence" => {
                Ok(Stat::CriticalDefense)
            }
            "incominghealing" | "incheal" => Ok(Stat::IncomingHealing),
            "block" => Ok(Stat::Block),
            "parry" => Ok(Stat::Parry),
            "evade" => Ok(Stat::Evade),
            "physicalmitigation" | "physmitigation" | "physmit" => Ok(Stat::PhysicalMitigation),
            "tacticalmitigation" | "tactmitigation" | "tactmit" => Ok(Stat::TacticalMitigation),

            // Internal stats (wiki parsing / future use)
            "might" => Ok(Stat::Might),
            "agility" => Ok(Stat::Agility),
            "vitality" => Ok(Stat::Vitality),
            "will" => Ok(Stat::Will),
            "fate" => Ok(Stat::Fate),
            "morale" => Ok(Stat::Morale),
            "power" => Ok(Stat::Power),
            "devrating" | "devastatingcriticalrating" => Ok(Stat::DevRating),
            "offensiveoverpower" | "overpower" => Ok(Stat::OffensiveOverpower),
            "incmitigations" | "incmit" => Ok(Stat::IncMitigations),

            _ => Err(format!("Unknown stat: '{}'", s)),
        }
    }
}

// -- StatGoal ------------------------------------------------------------------

/// A stat with an associated minimum value, parsed from CLI input.
/// Format: `StatName:minimum`  e.g. `CriticalRating:450000`
/// A minimum of 0 means "maximise but no floor required".
#[derive(Debug, Clone)]
pub struct StatGoal {
    pub stat: Stat,
    pub minimum: i64,
}

impl FromStr for StatGoal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            Some((stat_str, min_str)) => {
                let stat = stat_str.parse::<Stat>()?;
                let minimum = min_str
                    .parse::<i64>()
                    .map_err(|_| format!("Invalid minimum '{}' in goal '{}'", min_str, s))?;
                Ok(StatGoal { stat, minimum })
            }
            None => {
                // No colon — treat as stat name with minimum 0.
                let stat = s.parse::<Stat>()?;
                Ok(StatGoal { stat, minimum: 0 })
            }
        }
    }
}

// -- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_morale_and_power_abbreviations() {
        assert_eq!("ml".parse::<Stat>().unwrap(), Stat::Morale);
        assert_eq!("pw".parse::<Stat>().unwrap(), Stat::Power);
    }

    #[test]
    fn parses_morale_and_power_full_names() {
        assert_eq!("Morale".parse::<Stat>().unwrap(), Stat::Morale);
        assert_eq!("Power".parse::<Stat>().unwrap(), Stat::Power);
    }

    #[test]
    fn tracked_stats_are_the_canonical_sixteen_in_order() {
        let actual: Vec<&str> = TRACKED_STATS.iter().map(|(_, key)| *key).collect();
        assert_eq!(
            actual,
            vec![
                "Morale",
                "Power",
                "Armor",
                "CriticalRating",
                "Finesse",
                "PhysicalMastery",
                "TacticalMastery",
                "OutgoingHealing",
                "Resistance",
                "CriticalDefense",
                "IncomingHealing",
                "Block",
                "Parry",
                "Evade",
                "PhysicalMitigation",
                "TacticalMitigation",
            ]
        );
    }

    #[test]
    fn tracked_stats_have_abbreviations_in_canonical_order() {
        let actual: Vec<&str> = TRACKED_STATS
            .iter()
            .map(|(stat, _)| abbreviation_for(*stat).unwrap())
            .collect();

        assert_eq!(
            actual,
            vec![
                "ml", "pw", "am", "cr", "fn", "pm", "tm", "oh", "rs", "cd", "ih", "bl", "pa", "ev",
                "pt", "tt",
            ]
        );
    }

    #[test]
    fn abbreviation_for_internal_stats_is_none() {
        assert_eq!(abbreviation_for(Stat::Might), None);
        assert_eq!(abbreviation_for(Stat::Agility), None);
        assert_eq!(abbreviation_for(Stat::Vitality), None);
        assert_eq!(abbreviation_for(Stat::Will), None);
        assert_eq!(abbreviation_for(Stat::Fate), None);
        assert_eq!(abbreviation_for(Stat::DevRating), None);
        assert_eq!(abbreviation_for(Stat::OffensiveOverpower), None);
        assert_eq!(abbreviation_for(Stat::IncMitigations), None);
    }
}

#[test]
fn parses_morale_and_power_normalized_spellings() {
    assert_eq!("morale".parse::<Stat>().unwrap(), Stat::Morale);
    assert_eq!("power".parse::<Stat>().unwrap(), Stat::Power);
}
