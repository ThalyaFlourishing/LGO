//! Base-stat derivation loading and conversion.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::stat::{Stat, BASE_STATS};

/// File name resolved under the install directory's `data/` folder.
const DEFAULT_DERIVATIONS_FILE: &str = "base_stat_derivations.json";

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
            DerivationError::ResolveDefaultPath { file_name, source } => write!(
                f,
                "Cannot resolve the install-directory path for derivation data '{}': {}",
                file_name, source
            ),
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
        let path = crate::install::data_path(DEFAULT_DERIVATIONS_FILE).map_err(|source| {
            DerivationError::ResolveDefaultPath {
                file_name: DEFAULT_DERIVATIONS_FILE,
                source,
            }
        })?;
        Self::load(&path)
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
                // Rounding rule (empirically confirmed in-game): each product
                // `coefficient × base_stat_value` rounds UP via `f64::ceil()`,
                // per item, per stat. Negative values follow plain `ceil()`
                // semantics (round toward zero).
                let contribution = (*base_value as f64 * coeff).ceil() as i64;
                if contribution != 0 {
                    *derived.entry(*derived_stat).or_insert(0) += contribution;
                }
            }
        }
        Ok(derived)
    }

    /// The derivation pre-pass primitive: convert `base_stats` into
    /// tracked-stat contributions for `class_name` (per-product `ceil()`
    /// rule) and add them into `tracked`. Zero-valued entries are dropped
    /// afterwards, matching the runtime convention that absence means zero.
    pub fn apply_derivations(
        &self,
        class_name: &str,
        base_stats: &HashMap<Stat, i64>,
        tracked: &mut HashMap<Stat, i64>,
    ) -> Result<(), DerivationError> {
        for (stat, value) in self.derive_stats(class_name, base_stats)? {
            *tracked.entry(stat).or_insert(0) += value;
        }
        tracked.retain(|_, value| *value != 0);
        Ok(())
    }

    /// Apply the derivation pre-pass to a whole gear document: once to the
    /// innate base stats (into the innate tracked-stat map) and once per item
    /// (over item base stats, essence base values already merged by the
    /// reader). Runs in the optimize code path, before `optimize()` — the
    /// optimizer sees ordinary tracked-stat maps and is never aware of Base
    /// stats.
    pub fn derive_doc(
        &self,
        class_name: &str,
        doc: &mut crate::gearstats::GearDoc,
    ) -> Result<(), DerivationError> {
        self.apply_derivations(class_name, &doc.innate_base_stats, &mut doc.innate_stats)?;
        for doc_item in &mut doc.items {
            self.apply_derivations(class_name, &doc_item.base_stats, &mut doc_item.item.stats)?;
        }
        Ok(())
    }

    pub fn merge_explicit_and_base(
        &self,
        class_name: &str,
        explicit_stats: &HashMap<Stat, i64>,
        base_stats: &HashMap<Stat, i64>,
    ) -> Result<HashMap<Stat, i64>, DerivationError> {
        let mut merged = explicit_stats.clone();
        self.apply_derivations(class_name, base_stats, &mut merged)?;
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

/// Tracked-stat links: `(source, target, coefficient)`. Applied after Base
/// derivation so every source total (explicit + essence + Base-derived +
/// Virtue) is already final. Class-independent. Coefficients are integers
/// deliberately: with a non-integer coefficient the per-source vs.
/// whole-total rounding question becomes real and must be decided
/// explicitly before adding such an entry.
pub const TRACKED_LINKS: &[(Stat, Stat, i64)] = &[
    // Every point of Tactical Mastery adds one point of Outgoing Healing.
    (Stat::TacticalMastery, Stat::OutgoingHealing, 1),
];

/// Apply [`TRACKED_LINKS`] to one tracked-stat map in place. Reads all
/// source values before writing any target so a future link chain cannot
/// double-apply within a single call. Zero-valued entries are dropped
/// afterwards (absence means zero).
pub fn apply_tracked_links(tracked: &mut HashMap<Stat, i64>) {
    let additions: Vec<(Stat, i64)> = TRACKED_LINKS
        .iter()
        .filter_map(|(source, target, coefficient)| {
            let value = tracked.get(source).copied().unwrap_or(0) * coefficient;
            (value != 0).then_some((*target, value))
        })
        .collect();
    for (target, value) in additions {
        *tracked.entry(target).or_insert(0) += value;
    }
    tracked.retain(|_, value| *value != 0);
}

/// Apply the link pass to a whole gear document: once to `innate_stats`
/// and once per item. Must run **after** `derive_doc`.
pub fn link_doc(doc: &mut crate::gearstats::GearDoc) {
    apply_tracked_links(&mut doc.innate_stats);
    for doc_item in &mut doc.items {
        apply_tracked_links(&mut doc_item.item.stats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal synthetic derivation table for the rounding/apply logic tests,
    /// so they own their coefficients instead of depending on the committed
    /// `data/base_stat_derivations.json`. Every expected class is present; only
    /// Lore-master carries the coefficient rows these tests exercise.
    const SYNTHETIC_DERIVATIONS: &str = r#"{
        "Beorning": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Brawler": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Burglar": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Captain": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Champion": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Guardian": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Hunter": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
        "Lore-master": {
            "Might": { "CriticalRating": 1.5, "Finesse": 1.5, "TacticalMastery": 2.0, "Parry": 1.0 },
            "Agility": {},
            "Vitality": {},
            "Will": {},
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
            .expect("synthetic derivations parse")
    }

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
            "Beorning": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Brawler": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Burglar": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Captain": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Champion": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Guardian": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Hunter": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Lore-master": { "Might": { "Bogus": 1.0 }, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Mariner": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Minstrel": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Rune-keeper": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} },
            "Warden": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {}, "Armour": {} }
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

    // ── Ceil rounding rule ────────────────────────────────────────────────────

    #[test]
    fn fractional_products_round_up_per_product() {
        // A Lore-master +9 Might item contributes ceil(9 × 1.5) = 14
        // Critical Rating (empirically confirmed in-game).
        let derivations = synthetic_derivations();
        let base: HashMap<Stat, i64> = [(Stat::Might, 9)].into_iter().collect();
        let derived = derivations
            .derive_stats("Lore-master", &base)
            .expect("derive");
        assert_eq!(derived.get(&Stat::CriticalRating), Some(&14));
        assert_eq!(derived.get(&Stat::Finesse), Some(&14));
        assert_eq!(derived.get(&Stat::TacticalMastery), Some(&18));
        assert_eq!(derived.get(&Stat::Parry), Some(&9));
    }

    #[test]
    fn exact_products_stay_exact() {
        // ceil() must not disturb integral products: 10 × 1.5 = 15 exactly.
        let derivations = synthetic_derivations();
        let base: HashMap<Stat, i64> = [(Stat::Might, 10)].into_iter().collect();
        let derived = derivations
            .derive_stats("Lore-master", &base)
            .expect("derive");
        assert_eq!(derived.get(&Stat::CriticalRating), Some(&15));
        assert_eq!(derived.get(&Stat::TacticalMastery), Some(&20));
    }

    #[test]
    fn negative_values_follow_plain_ceil_semantics() {
        // Pinned: ceil(-9 × 1.5) = ceil(-13.5) = -13 (rounds toward zero).
        let derivations = synthetic_derivations();
        let base: HashMap<Stat, i64> = [(Stat::Might, -9)].into_iter().collect();
        let derived = derivations
            .derive_stats("Lore-master", &base)
            .expect("derive");
        assert_eq!(derived.get(&Stat::CriticalRating), Some(&-13));
        assert_eq!(derived.get(&Stat::Finesse), Some(&-13));
        assert_eq!(derived.get(&Stat::TacticalMastery), Some(&-18));
        assert_eq!(derived.get(&Stat::Parry), Some(&-9));
    }

    // ── Real-data derivation ──────────────────────────────────────────────────

    #[test]
    fn lore_master_might_derives_real_coefficients() {
        // Real data/base_stat_derivations.json: Lore-master Might row is
        // CriticalRating 1.5, Finesse 1.5, TacticalMastery 2.0, Parry 1.0.
        let derivations = BaseStatDerivations::load_default().expect("default derivations load");
        let base: HashMap<Stat, i64> = [(Stat::Might, 1000)].into_iter().collect();
        let derived = derivations
            .derive_stats("Lore-master", &base)
            .expect("derive");
        let expected: HashMap<Stat, i64> = [
            (Stat::CriticalRating, 1500),
            (Stat::Finesse, 1500),
            (Stat::TacticalMastery, 2000),
            (Stat::Parry, 1000),
        ]
        .into_iter()
        .collect();
        assert_eq!(derived, expected);
    }

    #[test]
    fn lore_master_will_derives_real_coefficients() {
        // Real data/base_stat_derivations.json: Lore-master Will row is
        // CriticalRating 1.0, TacticalMastery 3.0, Resistance 1.0,
        // Evade 2.0, PhysicalMitigation 1.0, TacticalMitigation 1.0.
        let derivations = BaseStatDerivations::load_default().expect("default derivations load");
        let base: HashMap<Stat, i64> = [(Stat::Will, 1000)].into_iter().collect();
        let derived = derivations
            .derive_stats("Lore-master", &base)
            .expect("derive");
        let expected: HashMap<Stat, i64> = [
            (Stat::CriticalRating, 1000),
            (Stat::TacticalMastery, 3000),
            (Stat::Resistance, 1000),
            (Stat::Evade, 2000),
            (Stat::PhysicalMitigation, 1000),
            (Stat::TacticalMitigation, 1000),
        ]
        .into_iter()
        .collect();
        assert_eq!(derived, expected);
    }

    #[test]
    fn apply_derivations_adds_into_existing_tracked_map() {
        let derivations = synthetic_derivations();
        let base: HashMap<Stat, i64> = [(Stat::Might, 1000)].into_iter().collect();
        let mut tracked: HashMap<Stat, i64> = [(Stat::CriticalRating, 100), (Stat::Morale, 50)]
            .into_iter()
            .collect();
        derivations
            .apply_derivations("Lore-master", &base, &mut tracked)
            .expect("apply");
        assert_eq!(tracked.get(&Stat::CriticalRating), Some(&1600));
        assert_eq!(tracked.get(&Stat::Morale), Some(&50));
        assert_eq!(tracked.get(&Stat::TacticalMastery), Some(&2000));
    }

    #[test]
    fn armour_derives_real_coefficients() {
        // Real data/base_stat_derivations.json: every class's Armour row is
        // PhysicalMitigation 1.0, TacticalMitigation 0.2 (class-independent).
        // 1000 Armour → +1000 PhysicalMitigation, +200 TacticalMitigation.
        let derivations = BaseStatDerivations::load_default().expect("default derivations load");
        let base: HashMap<Stat, i64> = [(Stat::Armor, 1000)].into_iter().collect();
        let derived = derivations
            .derive_stats("Lore-master", &base)
            .expect("derive");
        let expected: HashMap<Stat, i64> = [
            (Stat::PhysicalMitigation, 1000),
            (Stat::TacticalMitigation, 200),
        ]
        .into_iter()
        .collect();
        assert_eq!(derived, expected);
    }

    #[test]
    fn armour_tactical_mitigation_rounds_up_per_product() {
        // 1001 Armour → PhysicalMitigation 1001 (exact) and
        // TacticalMitigation ceil(0.2 × 1001) = ceil(200.2) = 201.
        let derivations = BaseStatDerivations::load_default().expect("default derivations load");
        let base: HashMap<Stat, i64> = [(Stat::Armor, 1001)].into_iter().collect();
        let derived = derivations
            .derive_stats("Lore-master", &base)
            .expect("derive");
        assert_eq!(derived.get(&Stat::PhysicalMitigation), Some(&1001));
        assert_eq!(derived.get(&Stat::TacticalMitigation), Some(&201));
    }

    #[test]
    fn armour_is_never_a_key_in_derived_output() {
        // Armour is a derivation INPUT, never a derived (tracked) output key.
        let derivations = BaseStatDerivations::load_default().expect("default derivations load");
        let base: HashMap<Stat, i64> = [(Stat::Armor, 1000)].into_iter().collect();
        let derived = derivations
            .derive_stats("Lore-master", &base)
            .expect("derive");
        assert!(!derived.contains_key(&Stat::Armor));
    }

    #[test]
    fn empty_base_stats_never_require_a_class_entry() {
        // Documents with no Base stats must keep working even when the class
        // is unknown to the derivations table.
        let derivations = synthetic_derivations();
        let mut tracked: HashMap<Stat, i64> = [(Stat::Morale, 50)].into_iter().collect();
        derivations
            .apply_derivations("Unknown", &HashMap::new(), &mut tracked)
            .expect("empty base stats must not error");
        assert_eq!(tracked.get(&Stat::Morale), Some(&50));
    }

    fn map(entries: &[(Stat, i64)]) -> HashMap<Stat, i64> {
        entries.iter().copied().collect()
    }

    #[test]
    fn tracked_link_adds_tm_to_oh() {
        let mut tracked = map(&[(Stat::TacticalMastery, 1000), (Stat::OutgoingHealing, 50)]);
        apply_tracked_links(&mut tracked);
        assert_eq!(tracked.get(&Stat::OutgoingHealing), Some(&1050));
        assert_eq!(tracked.get(&Stat::TacticalMastery), Some(&1000));
    }

    #[test]
    fn tracked_link_with_no_tm_adds_no_oh_key() {
        let mut tracked = map(&[(Stat::CriticalRating, 10)]);
        apply_tracked_links(&mut tracked);
        assert_eq!(tracked, map(&[(Stat::CriticalRating, 10)]));
        assert!(!tracked.contains_key(&Stat::OutgoingHealing));
    }

    #[test]
    fn tracked_link_zero_tm_is_dropped_not_created() {
        let mut tracked = map(&[(Stat::TacticalMastery, 0)]);
        apply_tracked_links(&mut tracked);
        assert!(!tracked.contains_key(&Stat::OutgoingHealing));
        assert!(!tracked.contains_key(&Stat::TacticalMastery));
    }

    #[test]
    fn tracked_link_negative_tm_subtracts() {
        let mut tracked = map(&[(Stat::TacticalMastery, -30), (Stat::OutgoingHealing, 100)]);
        apply_tracked_links(&mut tracked);
        assert_eq!(tracked.get(&Stat::OutgoingHealing), Some(&70));
    }

    #[test]
    fn link_after_derivation_captures_base_derived_tm() {
        let derivations = synthetic_derivations();
        let base = map(&[(Stat::Might, 10)]);
        let mut tracked: HashMap<Stat, i64> = HashMap::new();
        derivations
            .apply_derivations("Lore-master", &base, &mut tracked)
            .expect("derive");
        apply_tracked_links(&mut tracked);
        assert_eq!(tracked.get(&Stat::TacticalMastery), Some(&20));
        assert_eq!(tracked.get(&Stat::OutgoingHealing), Some(&20));
    }

    #[test]
    fn link_doc_applies_to_innate_and_every_item() {
        use crate::gear::{GearItem, Slot};
        use crate::gearstats::{DocItem, GearDoc};

        let make_item = |name: &str, tm: i64| DocItem {
            item: GearItem {
                name: name.to_string(),
                slot: Slot::Head,
                two_handed: false,
                either_hand: false,
                stats: map(&[(Stat::TacticalMastery, tm), (Stat::OutgoingHealing, 5)]),
            },
            base_stats: HashMap::new(),
        };
        let mut doc = GearDoc {
            character: None,
            class: None,
            level: None,
            measured: None,
            innate_stats: map(&[(Stat::TacticalMastery, 9275)]),
            innate_base_stats: HashMap::new(),
            selected_virtues: Default::default(),
            items: vec![make_item("A", 100), make_item("B", 0)],
        };
        link_doc(&mut doc);
        assert_eq!(doc.innate_stats.get(&Stat::OutgoingHealing), Some(&9275));
        assert_eq!(
            doc.items[0].item.stats.get(&Stat::OutgoingHealing),
            Some(&105)
        );
        assert_eq!(
            doc.items[1].item.stats.get(&Stat::OutgoingHealing),
            Some(&5)
        );
    }
}
