//! Virtue data loading and optimizer-candidate construction.
//!
//! Virtues are not chosen by the user. `data/lgo_virtues.json` is the whole
//! candidate pool; the optimizer picks the best five distinct Virtues jointly
//! with gear (see `optimizer.rs`). This module only loads and validates the
//! pool and turns each Virtue into a final tracked-stat map for the class
//! being optimized.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::base_stats::{apply_tracked_links, BaseStatDerivations, DerivationError};
use crate::stat::{Stat, BASE_STATS, TRACKED_STATS};

/// File name resolved under the install directory's `data/` folder.
const DEFAULT_VIRTUES_FILE: &str = "lgo_virtues.json";

/// The optimizer always slots exactly this many distinct Virtues.
pub const VIRTUE_SLOT_COUNT: usize = 5;

pub const VIRTUE_INNATE_TABLE_KEY: &str = "VirtueInnateStats";
/// (Stat, TOML key) pairs, in the fixed emit order for the block
/// (canonical TRACKED_STATS order: PhysicalMastery before TacticalMastery).
pub const VIRTUE_INNATE_FIELDS: [(Stat, &str); 2] = [
    (Stat::PhysicalMastery, "PhysicalMastery"),
    (Stat::TacticalMastery, "TacticalMastery"),
];
pub const VIRTUE_INNATE_DEFAULT: i64 = 9275;

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

/// One Virtue as the optimizer sees it: a **final** tracked-stat map
/// (JSON tracked stats + class-derived Base-stat contributions + the
/// tracked-stat link pass), plus the raw Base stats kept alongside so reports
/// can show the chosen Virtues' Base stats in the projected Base-stat pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtueCandidate {
    pub name: String,
    pub tracked: HashMap<Stat, i64>,
    pub base: HashMap<Stat, i64>,
}

#[derive(Debug, Clone)]
pub struct VirtuesDb {
    by_name: HashMap<String, VirtueStats>,
    path: PathBuf,
}

#[derive(Debug)]
pub enum VirtuesError {
    ResolveDefaultPath {
        file_name: &'static str,
        source: std::io::Error,
    },
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
    /// Fewer than [`VIRTUE_SLOT_COUNT`] Virtues in the data file: the
    /// optimizer cannot fill five slots.
    TooFewVirtues {
        path: PathBuf,
        found: usize,
    },
    Derivation(DerivationError),
}

impl std::fmt::Display for VirtuesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VirtuesError::ResolveDefaultPath { file_name, source } => write!(
                f,
                "Cannot resolve the install-directory path for virtue data '{}': {}",
                file_name, source
            ),
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
            VirtuesError::TooFewVirtues { path, found } => write!(
                f,
                "virtue data '{}' defines only {} Virtue(s); the optimizer needs at least {} to fill the five Virtue slots",
                path.display(),
                found,
                VIRTUE_SLOT_COUNT
            ),
            VirtuesError::Derivation(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for VirtuesError {}

impl From<DerivationError> for VirtuesError {
    fn from(err: DerivationError) -> Self {
        VirtuesError::Derivation(err)
    }
}

impl VirtuesDb {
    pub fn load_default() -> Result<Self, VirtuesError> {
        let path = crate::install::data_path(DEFAULT_VIRTUES_FILE).map_err(|source| {
            VirtuesError::ResolveDefaultPath {
                file_name: DEFAULT_VIRTUES_FILE,
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
        let mut canonical_by_folded_name: HashMap<String, String> = HashMap::new();

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

        if by_name.len() < VIRTUE_SLOT_COUNT {
            return Err(VirtuesError::TooFewVirtues {
                path: path_for_errors.to_path_buf(),
                found: by_name.len(),
            });
        }

        Ok(Self {
            by_name,
            path: path_for_errors.to_path_buf(),
        })
    }

    /// Path the data was loaded from (for error messages).
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Number of Virtues in the pool.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// Build the optimizer's Virtue candidate pool for `class`, in JSON
    /// canonical (sorted-name) order. Each candidate's `tracked` map is final:
    /// JSON tracked stats, plus the class-derived contributions of its Base
    /// stats (per-product `ceil()`, like every other derivation), plus the
    /// tracked-stat link pass (TM → OH). The optimizer therefore treats a
    /// Virtue exactly like an item whose stats are already fully derived.
    pub fn candidates(
        &self,
        class: &str,
        derivations: &BaseStatDerivations,
    ) -> Result<Vec<VirtueCandidate>, VirtuesError> {
        let mut names: Vec<&String> = self.by_name.keys().collect();
        names.sort();
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            let stats = &self.by_name[name];
            let mut tracked = stats.tracked_stats.clone();
            derivations.apply_derivations(class, &stats.base_stats, &mut tracked)?;
            apply_tracked_links(&mut tracked);
            out.push(VirtueCandidate {
                name: name.clone(),
                tracked,
                base: stats.base_stats.clone(),
            });
        }
        Ok(out)
    }
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
    use std::collections::HashSet;

    fn db_from_json(src: &str) -> VirtuesDb {
        VirtuesDb::from_json_str(src, Path::new("<test>")).expect("virtue db should parse")
    }

    /// Synthetic derivation table owned by these tests: every expected class
    /// is present; only Lore-master carries coefficients (Might → CR/Finesse,
    /// Will → TM) so the candidate-building tests have something to derive.
    const SYNTHETIC_DERIVATIONS: &str = r#"{
        "Beorning": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Brawler": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Burglar": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Captain": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Champion": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Guardian": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Hunter": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Lore-master": {
            "Might": { "CriticalRating": 1.5, "Finesse": 1.5 },
            "Agility": {},
            "Vitality": {},
            "Will": { "TacticalMastery": 3.0 },
            "Fate": {},
            "Armour": { "PhysicalMitigation": 1.0, "TacticalMitigation": 0.2 }
        },
        "Mariner": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Minstrel": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Rune-keeper": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Warden": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} }
    }"#;

    fn synthetic_derivations() -> BaseStatDerivations {
        BaseStatDerivations::from_json_str(SYNTHETIC_DERIVATIONS, Path::new("<synthetic>"))
            .expect("synthetic derivations must parse")
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
            Some(&6940)
        );
        let justice = db
            .by_name
            .get("Justice")
            .expect("default virtue data should contain Justice");
        assert_eq!(
            justice.tracked_stats.get(&Stat::TacticalMitigation),
            Some(&4007)
        );
        assert_eq!(
            justice.tracked_stats.len(),
            1,
            "Justice must contain only its maintained TacticalMitigation value"
        );
    }

    #[test]
    fn default_virtue_data_stat_keys_are_all_canonical() {
        // Independently re-parse the raw JSON (rather than relying on
        // `VirtuesDb::from_json_str`'s own validation) so this test fails
        // loudly, naming the offending virtue and key, if
        // `data/lgo_virtues.json` ever contains a stat key that isn't an
        // exact `TRACKED_STATS`/`BASE_STATS` match (e.g. the `Armor` vs.
        // `Armour` mismatch that broke Compassion/Empathy/Loyalty).
        let path = crate::install::data_path(DEFAULT_VIRTUES_FILE)
            .expect("default virtue data path should resolve");
        let src = fs::read_to_string(&path).expect("default virtue data should be readable");
        let value: serde_json::Value =
            serde_json::from_str(&src).expect("default virtue data should be valid JSON");
        let root = value
            .as_object()
            .expect("default virtue data top level should be an object");

        let canonical_keys: HashSet<&str> = TRACKED_STATS
            .iter()
            .chain(BASE_STATS.iter())
            .map(|(_, key)| *key)
            .collect();

        for (virtue_name, virtue_value) in root {
            let virtue_obj = virtue_value
                .as_object()
                .unwrap_or_else(|| panic!("virtue '{}' should be an object", virtue_name));
            for stat_key in virtue_obj.keys() {
                assert!(
                    canonical_keys.contains(stat_key.as_str()),
                    "virtue '{}' has stat key '{}' that is not a canonical TRACKED_STATS/BASE_STATS key",
                    virtue_name,
                    stat_key
                );
            }
        }
    }

    #[test]
    fn from_json_str_rejects_fewer_than_five_virtues() {
        let err = VirtuesDb::from_json_str(
            r#"{ "Wisdom": { "Will": 80 }, "Zeal": { "Might": 80 } }"#,
            Path::new("data/lgo_virtues.json"),
        )
        .expect_err("two virtues must fail");
        assert!(matches!(err, VirtuesError::TooFewVirtues { found: 2, .. }));
        let msg = err.to_string();
        assert!(msg.contains("data/lgo_virtues.json"), "{msg}");
        assert!(msg.contains("only 2 Virtue(s)"), "{msg}");
        assert!(msg.contains("at least 5"), "{msg}");
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
    fn virtue_data_rejects_names_differing_only_by_case() {
        let err = VirtuesDb::from_json_str(
            r#"{ "Wisdom": { "Will": 1 }, "wisdom": { "Will": 2 },
                 "A": {}, "B": {}, "C": {}, "D": {} }"#,
            Path::new("<test>"),
        )
        .expect_err("case-duplicate names must fail");
        assert!(matches!(err, VirtuesError::DuplicateVirtueName { .. }));
    }

    #[test]
    fn candidates_are_sorted_by_name_and_fully_derived() {
        let derivations = synthetic_derivations();
        let db = db_from_json(
            r#"{
                "Zeal":   { "Might": 100, "CriticalRating": 136 },
                "Wisdom": { "Will": 80, "TacticalMastery": 192 },
                "Alpha":  { "Finesse": 10 },
                "Beta":   { "Evade": 10 },
                "Gamma":  { "Block": 10 }
            }"#,
        );

        let candidates = db
            .candidates("Lore-master", &derivations)
            .expect("candidates must build");
        let names: Vec<&str> = candidates.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["Alpha", "Beta", "Gamma", "Wisdom", "Zeal"]);

        // Wisdom: JSON TM 192 + Will-derived TM, then TM → OH link on the
        // final TM total. Expected values come from the same synthetic table.
        let wisdom = &candidates[3];
        let derived = derivations
            .derive_stats("Lore-master", &HashMap::from([(Stat::Will, 80)]))
            .expect("derive");
        let derived_tm = derived.get(&Stat::TacticalMastery).copied().unwrap_or(0);
        assert!(derived_tm > 0, "synthetic table must map Will to TM");
        let expected_tm = 192 + derived_tm;
        assert_eq!(
            wisdom.tracked.get(&Stat::TacticalMastery),
            Some(&expected_tm)
        );
        assert_eq!(
            wisdom.tracked.get(&Stat::OutgoingHealing),
            Some(&expected_tm)
        );
        assert_eq!(wisdom.base.get(&Stat::Will), Some(&80));
        assert!(
            !wisdom.tracked.contains_key(&Stat::Will),
            "Base stats must never leak into the tracked map"
        );

        let zeal = &candidates[4];
        let derived = derivations
            .derive_stats("Lore-master", &HashMap::from([(Stat::Might, 100)]))
            .expect("derive");
        let expected_cr = 136 + derived.get(&Stat::CriticalRating).copied().unwrap_or(0);
        assert_eq!(zeal.tracked.get(&Stat::CriticalRating), Some(&expected_cr));
        assert_eq!(zeal.base.get(&Stat::Might), Some(&100));
    }

    #[test]
    fn candidates_reject_unknown_class() {
        let derivations = synthetic_derivations();
        let db = db_from_json(r#"{ "A": { "Might": 1 }, "B": {}, "C": {}, "D": {}, "E": {} }"#);
        let err = db
            .candidates("Not-a-class", &derivations)
            .expect_err("unknown class must fail");
        assert!(matches!(err, VirtuesError::Derivation(_)));
    }
}
