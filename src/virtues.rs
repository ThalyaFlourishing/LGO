//! Virtue selection parsing and fixed-stat loading.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::gearstats::GearDoc;
use crate::stat::{Stat, BASE_STATS, TRACKED_STATS};

pub const DEFAULT_VIRTUES_PATH: &str = "data/lgo_virtues.json";
/// File name resolved under the install directory's `data/` folder.
const DEFAULT_VIRTUES_FILE: &str = "lgo_virtues.json";
pub const VIRTUE_TABLE_KEY: &str = "Virtues";
pub const VIRTUE_FIELD_KEYS: [&str; 5] = ["Virtue1", "Virtue2", "Virtue3", "Virtue4", "Virtue5"];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectedVirtues {
    pub slots: [Option<String>; 5],
}

impl SelectedVirtues {
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtueStats {
    pub tracked_stats: HashMap<Stat, i64>,
    pub base_stats: HashMap<Stat, i64>,
}

impl VirtueStats {
    fn new() -> Self {
        Self {
            tracked_stats: HashMap::new(),
            base_stats: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedVirtues {
    pub slots: [Option<String>; 5],
    pub tracked_stats: HashMap<Stat, i64>,
    pub base_stats: HashMap<Stat, i64>,
}

#[derive(Debug, Clone)]
pub struct VirtuesDb {
    by_name: HashMap<String, VirtueStats>,
    canonical_by_folded_name: HashMap<String, String>,
}

#[derive(Debug)]
pub enum VirtuesError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    TopLevelNotObject,
    VirtueNotObject {
        virtue_name: String,
    },
    StatValueNotInteger {
        virtue_name: String,
        stat_name: String,
    },
    UnknownStat {
        virtue_name: String,
        stat_name: String,
    },
    DuplicateVirtueName {
        first: String,
        second: String,
    },
    UnknownSelectedVirtue {
        entered_name: String,
    },
    DuplicateSelectedVirtue {
        virtue_name: String,
    },
}

impl std::fmt::Display for VirtuesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VirtuesError::Io { path, source } => {
                write!(
                    f,
                    "Cannot read virtue data '{}': {}",
                    path.display(),
                    source
                )
            }
            VirtuesError::ParseJson { path, source } => {
                write!(
                    f,
                    "Cannot parse virtue data '{}': {}",
                    path.display(),
                    source
                )
            }
            VirtuesError::TopLevelNotObject => write!(f, "virtue data top level must be an object"),
            VirtuesError::VirtueNotObject { virtue_name } => write!(
                f,
                "virtue data for Virtue '{}' must be an object",
                virtue_name
            ),
            VirtuesError::StatValueNotInteger {
                virtue_name,
                stat_name,
            } => write!(
                f,
                "virtue data stat '{}.{}' must be an integer",
                virtue_name, stat_name
            ),
            VirtuesError::UnknownStat {
                virtue_name,
                stat_name,
            } => write!(
                f,
                "virtue data for Virtue '{}' contains unknown stat '{}'",
                virtue_name, stat_name
            ),
            VirtuesError::DuplicateVirtueName { first, second } => write!(
                f,
                "virtue data contains duplicate Virtue names differing only by case: '{}' and '{}'",
                first, second
            ),
            VirtuesError::UnknownSelectedVirtue { entered_name } => write!(
                f,
                "Unknown Virtue '{}'; check the spelling in {}.",
                entered_name, DEFAULT_VIRTUES_PATH
            ),
            VirtuesError::DuplicateSelectedVirtue { virtue_name } => write!(
                f,
                "Duplicate Virtue '{}'; a character cannot select the same Virtue twice.",
                virtue_name
            ),
        }
    }
}

impl std::error::Error for VirtuesError {}

impl VirtuesDb {
    pub fn load_default() -> Result<Self, VirtuesError> {
        let path = crate::install::data_path(DEFAULT_VIRTUES_FILE).map_err(|source| {
            VirtuesError::Io {
                path: PathBuf::from(DEFAULT_VIRTUES_PATH),
                source,
            }
        })?;
        Self::load(&path)
    }

    pub fn load(path: &Path) -> Result<Self, VirtuesError> {
        let src = fs::read_to_string(path).map_err(|e| VirtuesError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Self::from_json_str(&src, path)
    }

    pub fn from_json_str(src: &str, path_for_errors: &Path) -> Result<Self, VirtuesError> {
        let value: serde_json::Value =
            serde_json::from_str(src).map_err(|e| VirtuesError::ParseJson {
                path: path_for_errors.to_path_buf(),
                source: e,
            })?;
        let root = value.as_object().ok_or(VirtuesError::TopLevelNotObject)?;

        let mut by_name = HashMap::new();
        let mut canonical_by_folded_name = HashMap::new();

        for (virtue_name, virtue_value) in root {
            let virtue_obj =
                virtue_value
                    .as_object()
                    .ok_or_else(|| VirtuesError::VirtueNotObject {
                        virtue_name: virtue_name.clone(),
                    })?;
            let folded_name = fold_name(virtue_name);
            if let Some(previous) =
                canonical_by_folded_name.insert(folded_name, virtue_name.clone())
            {
                return Err(VirtuesError::DuplicateVirtueName {
                    first: previous,
                    second: virtue_name.clone(),
                });
            }

            let mut stats = VirtueStats::new();
            for (stat_name, stat_value) in virtue_obj {
                let value =
                    stat_value
                        .as_i64()
                        .ok_or_else(|| VirtuesError::StatValueNotInteger {
                            virtue_name: virtue_name.clone(),
                            stat_name: stat_name.clone(),
                        })?;
                if value == 0 {
                    continue;
                }
                let stat = parse_canonical_stat_key(stat_name).ok_or_else(|| {
                    VirtuesError::UnknownStat {
                        virtue_name: virtue_name.clone(),
                        stat_name: stat_name.clone(),
                    }
                })?;
                if is_base_stat(stat) {
                    stats.base_stats.insert(stat, value);
                } else {
                    stats.tracked_stats.insert(stat, value);
                }
            }
            by_name.insert(virtue_name.clone(), stats);
        }

        Ok(Self {
            by_name,
            canonical_by_folded_name,
        })
    }

    pub fn resolve_selected(
        &self,
        selected: &SelectedVirtues,
    ) -> Result<ResolvedVirtues, VirtuesError> {
        let mut slots: [Option<String>; 5] = Default::default();
        let mut tracked_stats = HashMap::new();
        let mut base_stats = HashMap::new();
        let mut seen = HashSet::new();

        for (idx, name) in selected.slots.iter().enumerate() {
            let Some(name) = name.as_ref() else {
                continue;
            };
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let canonical_name = self
                .canonical_by_folded_name
                .get(&fold_name(name))
                .ok_or_else(|| VirtuesError::UnknownSelectedVirtue {
                    entered_name: name.to_string(),
                })?;
            if !seen.insert(fold_name(canonical_name)) {
                return Err(VirtuesError::DuplicateSelectedVirtue {
                    virtue_name: canonical_name.clone(),
                });
            }
            slots[idx] = Some(canonical_name.clone());
            let stats = self
                .by_name
                .get(canonical_name)
                .expect("canonical virtue name must exist in db");
            merge_stats(&stats.tracked_stats, &mut tracked_stats);
            merge_stats(&stats.base_stats, &mut base_stats);
        }

        Ok(ResolvedVirtues {
            slots,
            tracked_stats,
            base_stats,
        })
    }

    pub fn apply_selected_virtues(
        &self,
        doc: &mut GearDoc,
    ) -> Result<ResolvedVirtues, VirtuesError> {
        let resolved = self.resolve_selected(&doc.selected_virtues)?;
        merge_stats(&resolved.tracked_stats, &mut doc.innate_stats);
        merge_stats(&resolved.base_stats, &mut doc.innate_base_stats);
        doc.selected_virtues.slots = resolved.slots.clone();
        Ok(resolved)
    }
}

fn merge_stats(src: &HashMap<Stat, i64>, dst: &mut HashMap<Stat, i64>) {
    for (stat, value) in src {
        *dst.entry(*stat).or_insert(0) += value;
    }
    dst.retain(|_, value| *value != 0);
}

fn fold_name(name: &str) -> String {
    name.to_lowercase()
}

fn is_base_stat(stat: Stat) -> bool {
    BASE_STATS.iter().any(|(candidate, _)| *candidate == stat)
}

fn parse_canonical_stat_key(key: &str) -> Option<Stat> {
    TRACKED_STATS
        .iter()
        .chain(BASE_STATS.iter())
        .find_map(|(stat, canonical_key)| (*canonical_key == key).then_some(*stat))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gearstats::read_stats_file;

    fn db_from_json(src: &str) -> VirtuesDb {
        VirtuesDb::from_json_str(src, Path::new("<test>")).expect("virtue db should parse")
    }

    #[test]
    fn default_virtue_data_loads() {
        let db = VirtuesDb::load_default().expect("default virtue data should load");
        let zeal = db
            .by_name
            .get("Zeal")
            .expect("default virtue data should contain Zeal");
        assert_eq!(zeal.base_stats.get(&Stat::Might), Some(&3088));
        assert_eq!(zeal.tracked_stats.get(&Stat::CriticalRating), Some(&5024));
        assert_eq!(zeal.tracked_stats.get(&Stat::PhysicalMastery), Some(&8265));
        let wisdom = db
            .by_name
            .get("Wisdom")
            .expect("default virtue data should contain Wisdom");
        assert_eq!(wisdom.base_stats.get(&Stat::Will), Some(&3088));
        assert_eq!(wisdom.tracked_stats.get(&Stat::Finesse), Some(&6651));
        assert_eq!(
            wisdom.tracked_stats.get(&Stat::TacticalMastery),
            Some(&8265)
        );
        let justice = db
            .by_name
            .get("Justice")
            .expect("default virtue data should contain Justice");
        assert_eq!(justice.tracked_stats.get(&Stat::Morale), Some(&8261));
        assert_eq!(
            justice.tracked_stats.get(&Stat::TacticalMitigation),
            Some(&4007)
        );
        assert_eq!(
            justice.tracked_stats.len(),
            2,
            "unsupported virtue-only stats such as ICMR must be omitted"
        );
    }

    #[test]
    fn resolve_selected_trims_case_insensitively_and_canonicalizes_names() {
        let db = db_from_json(
            r#"{
                "Wisdom": { "Will": 80, "TacticalMastery": 192 },
                "Zeal":   { "Might": 80, "CriticalRating": 136 }
            }"#,
        );
        let selected = SelectedVirtues {
            slots: [
                Some(" wisdom ".to_string()),
                Some("ZEAL".to_string()),
                None,
                None,
                None,
            ],
        };

        let resolved = db
            .resolve_selected(&selected)
            .expect("selection must resolve");
        assert_eq!(resolved.slots[0].as_deref(), Some("Wisdom"));
        assert_eq!(resolved.slots[1].as_deref(), Some("Zeal"));
        assert_eq!(
            resolved.tracked_stats.get(&Stat::TacticalMastery),
            Some(&192)
        );
        assert_eq!(
            resolved.tracked_stats.get(&Stat::CriticalRating),
            Some(&136)
        );
        assert_eq!(resolved.base_stats.get(&Stat::Will), Some(&80));
        assert_eq!(resolved.base_stats.get(&Stat::Might), Some(&80));
    }

    #[test]
    fn resolve_selected_rejects_unknown_virtue_name() {
        let db = db_from_json(r#"{ "Wisdom": { "Will": 80 } }"#);
        let selected = SelectedVirtues {
            slots: [Some("Insight".to_string()), None, None, None, None],
        };

        let err = db
            .resolve_selected(&selected)
            .expect_err("unknown virtue must fail");
        assert_eq!(
            err.to_string(),
            "Unknown Virtue 'Insight'; check the spelling in data/lgo_virtues.json."
        );
    }

    #[test]
    fn resolve_selected_rejects_duplicate_virtues_case_insensitively() {
        let db = db_from_json(r#"{ "Wisdom": { "Will": 80 } }"#);
        let selected = SelectedVirtues {
            slots: [
                Some("Wisdom".to_string()),
                Some(" wisdom ".to_string()),
                None,
                None,
                None,
            ],
        };

        let err = db
            .resolve_selected(&selected)
            .expect_err("duplicate virtue must fail");
        assert_eq!(
            err.to_string(),
            "Duplicate Virtue 'Wisdom'; a character cannot select the same Virtue twice."
        );
    }

    #[test]
    fn virtue_data_rejects_unknown_stat_keys() {
        let err = VirtuesDb::from_json_str(
            r#"{ "Wisdom": { "WisdomRating": 80 } }"#,
            Path::new("<test>"),
        )
        .expect_err("unknown stat key must fail");
        assert_eq!(
            err.to_string(),
            "virtue data for Virtue 'Wisdom' contains unknown stat 'WisdomRating'"
        );
    }

    #[test]
    fn apply_selected_virtues_adds_tracked_and_base_stats_to_doc() {
        let dir = std::env::temp_dir().join(format!(
            "lgo_virtues_apply_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("gear.toml");
        std::fs::write(
            &path,
            r#"
class = "Lore-master"

[InnateStats]
Will = 10

[Virtues]
Virtue1 = " wisdom "
Virtue2 = "ZEAL"

[[item]]
slot = "Head"
name = "Test Helm"
"#,
        )
        .expect("write toml");

        let mut doc = read_stats_file(&path).expect("toml must parse");
        let db = db_from_json(
            r#"{
                "Wisdom": { "Will": 80, "TacticalMastery": 192 },
                "Zeal":   { "Might": 80, "CriticalRating": 136 }
            }"#,
        );
        db.apply_selected_virtues(&mut doc)
            .expect("virtues must apply");

        assert_eq!(doc.selected_virtues.slots[0].as_deref(), Some("Wisdom"));
        assert_eq!(doc.selected_virtues.slots[1].as_deref(), Some("Zeal"));
        assert_eq!(doc.innate_stats.get(&Stat::TacticalMastery), Some(&192));
        assert_eq!(doc.innate_stats.get(&Stat::CriticalRating), Some(&136));
        assert_eq!(doc.innate_base_stats.get(&Stat::Will), Some(&90));
        assert_eq!(doc.innate_base_stats.get(&Stat::Might), Some(&80));

        std::fs::remove_dir_all(&dir).expect("cleanup");
    }
}
