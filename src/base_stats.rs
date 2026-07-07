//! Base-stat derivation loading and conversion.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::stat::{Stat, BASE_STATS};

pub const DEFAULT_DERIVATIONS_PATH: &str = "data/base_stat_derivations.json";

const EXPECTED_CLASSES: &[&str] = &[
    "Beorning",
    "Brawler",
    "Burglar",
    "Captain",
    "Champion",
    "Guardian",
    "Hunter",
    "Lore-master",
    "Mariner",
    "Minstrel",
    "Rune-keeper",
    "Warden",
];

#[derive(Debug, Clone)]
pub struct BaseStatDerivations {
    by_class: HashMap<String, HashMap<Stat, HashMap<Stat, f64>>>,
}

#[derive(Debug)]
pub enum DerivationError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    TopLevelNotObject,
    ClassNotObject {
        class_name: String,
    },
    MissingClass {
        class_name: String,
    },
    MissingBaseStat {
        class_name: String,
        base_stat: &'static str,
    },
    UnknownStat {
        stat_name: String,
    },
    NonBaseStatRow {
        class_name: String,
        stat_name: String,
    },
    DerivedRowNotObject {
        class_name: String,
        base_stat: String,
    },
    CoefficientNotNumber {
        class_name: String,
        base_stat: String,
        derived_stat: String,
    },
}

impl std::fmt::Display for DerivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DerivationError::Io { path, source } => {
                write!(
                    f,
                    "Cannot read derivation data '{}': {}",
                    path.display(),
                    source
                )
            }
            DerivationError::ParseJson { path, source } => {
                write!(
                    f,
                    "Cannot parse derivation data '{}': {}",
                    path.display(),
                    source
                )
            }
            DerivationError::TopLevelNotObject => {
                write!(f, "derivation data top level must be an object")
            }
            DerivationError::ClassNotObject { class_name } => {
                write!(
                    f,
                    "derivation data for class '{}' must be an object",
                    class_name
                )
            }
            DerivationError::MissingClass { class_name } => {
                write!(f, "derivation data missing expected class '{}'", class_name)
            }
            DerivationError::MissingBaseStat {
                class_name,
                base_stat,
            } => write!(
                f,
                "derivation data for class '{}' missing base-stat row '{}'",
                class_name, base_stat
            ),
            DerivationError::UnknownStat { stat_name } => {
                write!(f, "derivation data contains unknown stat '{}'", stat_name)
            }
            DerivationError::NonBaseStatRow {
                class_name,
                stat_name,
            } => write!(
                f,
                "derivation data for class '{}' has non-base stat row '{}'",
                class_name, stat_name
            ),
            DerivationError::DerivedRowNotObject {
                class_name,
                base_stat,
            } => write!(
                f,
                "derivation row '{}.{}' must be an object",
                class_name, base_stat
            ),
            DerivationError::CoefficientNotNumber {
                class_name,
                base_stat,
                derived_stat,
            } => write!(
                f,
                "derivation coefficient '{}.{}.{}' must be numeric",
                class_name, base_stat, derived_stat
            ),
        }
    }
}

impl std::error::Error for DerivationError {}

impl BaseStatDerivations {
    pub fn load_default() -> Result<Self, DerivationError> {
        Self::load(Path::new(DEFAULT_DERIVATIONS_PATH))
    }

    pub fn load(path: &Path) -> Result<Self, DerivationError> {
        let src = fs::read_to_string(path).map_err(|e| DerivationError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Self::from_json_str(&src, path)
    }

    pub fn from_json_str(src: &str, path_for_errors: &Path) -> Result<Self, DerivationError> {
        let value: serde_json::Value =
            serde_json::from_str(src).map_err(|e| DerivationError::ParseJson {
                path: path_for_errors.to_path_buf(),
                source: e,
            })?;
        let root = value
            .as_object()
            .ok_or(DerivationError::TopLevelNotObject)?;

        for class_name in EXPECTED_CLASSES {
            let class = root
                .get(*class_name)
                .ok_or_else(|| DerivationError::MissingClass {
                    class_name: (*class_name).to_string(),
                })?
                .as_object()
                .ok_or_else(|| DerivationError::ClassNotObject {
                    class_name: (*class_name).to_string(),
                })?;
            for (_, base_key) in BASE_STATS {
                if !class.contains_key(*base_key) {
                    return Err(DerivationError::MissingBaseStat {
                        class_name: (*class_name).to_string(),
                        base_stat: base_key,
                    });
                }
            }
        }

        let mut by_class = HashMap::new();
        for (class_name, class_value) in root {
            let class_obj =
                class_value
                    .as_object()
                    .ok_or_else(|| DerivationError::ClassNotObject {
                        class_name: class_name.clone(),
                    })?;
            let mut by_base: HashMap<Stat, HashMap<Stat, f64>> = HashMap::new();
            for (base_name, row_value) in class_obj {
                let base_stat = parse_stat_name(base_name)?;
                if !BASE_STATS.iter().any(|(stat, _)| *stat == base_stat) {
                    return Err(DerivationError::NonBaseStatRow {
                        class_name: class_name.clone(),
                        stat_name: base_name.clone(),
                    });
                }
                let row =
                    row_value
                        .as_object()
                        .ok_or_else(|| DerivationError::DerivedRowNotObject {
                            class_name: class_name.clone(),
                            base_stat: base_name.clone(),
                        })?;
                let mut derived = HashMap::new();
                for (derived_name, coeff_value) in row {
                    let derived_stat = parse_stat_name(derived_name)?;
                    let coeff = coeff_value.as_f64().ok_or_else(|| {
                        DerivationError::CoefficientNotNumber {
                            class_name: class_name.clone(),
                            base_stat: base_name.clone(),
                            derived_stat: derived_name.clone(),
                        }
                    })?;
                    derived.insert(derived_stat, coeff);
                }
                by_base.insert(base_stat, derived);
            }
            by_class.insert(class_name.clone(), by_base);
        }

        Ok(Self { by_class })
    }

    pub fn class_names(&self) -> impl Iterator<Item = &str> {
        self.by_class.keys().map(String::as_str)
    }

    pub fn derive_stats(
        &self,
        class_name: &str,
        base_stats: &HashMap<Stat, i64>,
    ) -> Result<HashMap<Stat, i64>, DerivationError> {
        if base_stats.is_empty() {
            return Ok(HashMap::new());
        }
        let class = self
            .by_class
            .get(class_name)
            .ok_or_else(|| DerivationError::MissingClass {
                class_name: class_name.to_string(),
            })?;
        let mut derived: HashMap<Stat, i64> = HashMap::new();
        for (base_stat, base_value) in base_stats {
            if *base_value == 0 {
                continue;
            }
            let row = class
                .get(base_stat)
                .ok_or_else(|| DerivationError::MissingBaseStat {
                    class_name: class_name.to_string(),
                    base_stat: stat_key(*base_stat).unwrap_or("<unknown>"),
                })?;
            for (derived_stat, coeff) in row {
                // Derived stats are stored as integers everywhere downstream.
                // The JSON coefficients are decimal game formulas (mostly .0
                // and .5); round each final base-stat contribution once so
                // fractional half-point formulas do not leak out of this layer.
                let contribution = (*base_value as f64 * coeff).round() as i64;
                if contribution != 0 {
                    *derived.entry(*derived_stat).or_insert(0) += contribution;
                }
            }
        }
        Ok(derived)
    }

    pub fn merge_explicit_and_base(
        &self,
        class_name: &str,
        explicit_stats: &HashMap<Stat, i64>,
        base_stats: &HashMap<Stat, i64>,
    ) -> Result<HashMap<Stat, i64>, DerivationError> {
        let mut merged = explicit_stats.clone();
        for (stat, value) in self.derive_stats(class_name, base_stats)? {
            *merged.entry(stat).or_insert(0) += value;
        }
        merged.retain(|_, value| *value != 0);
        Ok(merged)
    }
}

fn parse_stat_name(name: &str) -> Result<Stat, DerivationError> {
    name.parse::<Stat>()
        .map_err(|_| DerivationError::UnknownStat {
            stat_name: name.to_string(),
        })
}

fn stat_key(stat: Stat) -> Option<&'static str> {
    BASE_STATS
        .iter()
        .find_map(|(candidate, key)| (*candidate == stat).then_some(*key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_derivations_include_plugin_class_spellings() {
        let derivations = BaseStatDerivations::load_default().expect("default derivations load");
        let names: std::collections::HashSet<&str> = derivations.class_names().collect();
        assert!(names.contains("Lore-master"));
        assert!(names.contains("Rune-keeper"));
    }

    #[test]
    fn unknown_stat_name_is_reported() {
        let src = r#"{
            "Beorning": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Brawler": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Burglar": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Captain": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Champion": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Guardian": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Hunter": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Lore-master": { "Might": { "Bogus": 1.0 }, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Mariner": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Minstrel": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Rune-keeper": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
            "Warden": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} }
        }"#;
        let err = BaseStatDerivations::from_json_str(src, Path::new("<test>"))
            .expect_err("unknown stat should fail");
        assert!(matches!(err, DerivationError::UnknownStat { .. }));
    }

    #[test]
    fn missing_expected_class_is_reported() {
        let err = BaseStatDerivations::from_json_str("{}", Path::new("<test>"))
            .expect_err("missing class should fail");
        assert!(matches!(err, DerivationError::MissingClass { .. }));
    }
}
