//! Integration tests for the Base-stat derivation pre-pass in the optimize
//! path: raw Base stats (innate and per item, essence values included) must
//! be converted into tracked-stat contributions via the per-class
//! coefficients in `data/base_stat_derivations.json` before candidates enter
//! the optimizer.

use std::collections::HashMap;
use std::path::Path;

use lgo::base_stats::BaseStatDerivations;
use lgo::gear::optimizer_candidate_key;
use lgo::gearstats::read_stats_file;
use lgo::optimizer::optimize;
use lgo::stat::{Stat, StatGoal};

const THALYA_FIXTURE: &str = "TestData/lgo_Thalya_gearReady.toml";
const LORE_MASTER: &str = "Lore-master";

fn load_derivations() -> BaseStatDerivations {
    // Integration tests run with CWD = crate root, matching the
    // working-directory convention `lgo optimize` uses at runtime.
    BaseStatDerivations::load_default().expect("data/base_stat_derivations.json must load")
}

fn tm_goal() -> Vec<StatGoal> {
    vec!["tm:0".parse::<StatGoal>().expect("tm:0 must parse")]
}

fn make_test_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lgo_base_stats_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// End-to-end over the Thalya fixture (Lore-master; innate Might 5300,
/// Agility 2650, Vitality 10200, Will 7950, Fate 4000): derived
/// contributions must be present in optimize totals, and the achieved
/// TacticalMastery for a `tm:0` goal must be far above the underived value.
/// The expected total is recomputed independently from the raw fixture
/// values — no magic numbers.
#[test]
fn thalya_fixture_optimize_totals_include_derived_contributions() {
    let fixture = Path::new(THALYA_FIXTURE);
    let derivations = load_derivations();
    let goals = tm_goal();

    // ── Underived run (raw tracked stats only, PR #57 mid-state) ────────────
    let raw_doc = read_stats_file(fixture).expect("fixture must parse");
    assert_eq!(raw_doc.class.as_deref(), Some(LORE_MASTER));
    let raw_resolved: HashMap<String, lgo::gear::GearItem> = raw_doc
        .items
        .iter()
        .enumerate()
        .map(|(idx, doc_item)| {
            (
                optimizer_candidate_key(idx, &doc_item.item),
                doc_item.item.clone(),
            )
        })
        .collect();
    let raw_keys: Vec<String> = raw_resolved.keys().cloned().collect();
    let underived = optimize(&raw_resolved, &raw_keys, &goals, &raw_doc.innate_stats);
    let tm_underived = underived.gear_set.total(&Stat::TacticalMastery);

    // Per-item derived TM contributions and raw tracked TM, keyed by display
    // name (the name the optimizer reports for chosen items). Duplicate
    // names in the fixture are identical owned copies; the insert asserts
    // that so name-based matching stays exact.
    let mut raw_tm_by_name: HashMap<String, i64> = HashMap::new();
    let mut derived_tm_by_name: HashMap<String, i64> = HashMap::new();
    for doc_item in &raw_doc.items {
        let raw_tm = doc_item.item.stat(&Stat::TacticalMastery);
        let derived = derivations
            .derive_stats(LORE_MASTER, &doc_item.base_stats)
            .expect("item derivation must succeed");
        let derived_tm = derived.get(&Stat::TacticalMastery).copied().unwrap_or(0);
        if let Some(previous) = raw_tm_by_name.insert(doc_item.item.name.clone(), raw_tm) {
            assert_eq!(
                previous, raw_tm,
                "duplicate-named fixture item '{}' must be an identical copy",
                doc_item.item.name
            );
        }
        if let Some(previous) = derived_tm_by_name.insert(doc_item.item.name.clone(), derived_tm) {
            assert_eq!(
                previous, derived_tm,
                "duplicate-named fixture item '{}' must be an identical copy",
                doc_item.item.name
            );
        }
    }
    let innate_derived = derivations
        .derive_stats(LORE_MASTER, &raw_doc.innate_base_stats)
        .expect("innate derivation must succeed");
    let innate_derived_tm = innate_derived
        .get(&Stat::TacticalMastery)
        .copied()
        .unwrap_or(0);
    assert!(
        innate_derived_tm > 0,
        "fixture innate base stats must derive a TacticalMastery contribution"
    );

    // ── Derived run (this PR's optimize path) ────────────────────────────────
    let mut derived_doc = read_stats_file(fixture).expect("fixture must parse");
    derivations
        .derive_doc(LORE_MASTER, &mut derived_doc)
        .expect("derivation pre-pass must succeed");
    let resolved: HashMap<String, lgo::gear::GearItem> = derived_doc
        .items
        .into_iter()
        .map(|doc_item| doc_item.item)
        .enumerate()
        .map(|(idx, item)| (optimizer_candidate_key(idx, &item), item))
        .collect();
    let keys: Vec<String> = resolved.keys().cloned().collect();
    let result = optimize(&resolved, &keys, &goals, &derived_doc.innate_stats);
    let tm_derived = result.gear_set.total(&Stat::TacticalMastery);

    // Recompute the expected total independently from the raw fixture: each
    // chosen item's contribution is its raw tracked TM plus its derived TM.
    // `[empty ...]` placeholders are absent from the maps and contribute 0.
    let innate_tracked_tm = derived_doc
        .innate_stats
        .get(&Stat::TacticalMastery)
        .copied()
        .unwrap_or(0);
    let expected_tm: i64 = innate_tracked_tm
        + result
            .gear_set
            .items
            .values()
            .map(|item| {
                raw_tm_by_name.get(&item.name).copied().unwrap_or(0)
                    + derived_tm_by_name.get(&item.name).copied().unwrap_or(0)
            })
            .sum::<i64>();
    assert_eq!(
        tm_derived, expected_tm,
        "optimize total must equal raw tracked TM plus derived contributions for the chosen build"
    );

    // Far above the underived value: at minimum the innate derivation alone
    // (a computed value, not a magic number) must separate the two runs.
    assert!(
        tm_derived >= tm_underived + innate_derived_tm,
        "derived TM total ({}) must exceed underived total ({}) by at least the innate derived contribution ({})",
        tm_derived,
        tm_underived,
        innate_derived_tm
    );
}

/// Innate base stats contribute on their own: with `[InnateStats]`
/// Will = 7950 and no stat-bearing items, a Lore-master's TacticalMastery
/// total must include the derived Will contribution.
#[test]
fn innate_base_stats_contribute_to_optimize_totals() {
    let dir = make_test_dir();
    let path = dir.join("test.toml");
    let toml = r#"
character = "Thalya"
class = "Lore-master"

[InnateStats]
Will = 7950

[[item]]
slot = "Head"
name = "Statless Helm"
"#;
    std::fs::write(&path, toml).expect("write toml");

    let derivations = load_derivations();
    let mut doc = read_stats_file(&path).expect("must parse");
    derivations
        .derive_doc(LORE_MASTER, &mut doc)
        .expect("derivation pre-pass must succeed");

    let resolved: HashMap<String, lgo::gear::GearItem> = doc
        .items
        .into_iter()
        .map(|doc_item| doc_item.item)
        .enumerate()
        .map(|(idx, item)| (optimizer_candidate_key(idx, &item), item))
        .collect();
    let keys: Vec<String> = resolved.keys().cloned().collect();
    let result = optimize(&resolved, &keys, &tm_goal(), &doc.innate_stats);

    // Real Lore-master coefficients: Will → TacticalMastery 3.0, so
    // ceil(7950 × 3.0) = 23850, integral and exact.
    let expected: i64 = (7950f64 * 3.0).ceil() as i64;
    assert_eq!(expected, 23850);
    assert_eq!(result.gear_set.total(&Stat::TacticalMastery), expected);

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

/// Base values entered in `[item.EssenceTotals]` (e.g. Vitality/Will
/// essences) must derive identically to the same value on the item itself.
#[test]
fn essence_base_values_derive_identically_to_item_base_values() {
    let dir = make_test_dir();

    let on_item = dir.join("on_item.toml");
    std::fs::write(
        &on_item,
        r#"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Vitality Helm"
Vitality = 100
"#,
    )
    .expect("write toml");

    let in_essence = dir.join("in_essence.toml");
    std::fs::write(
        &in_essence,
        r#"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Vitality Helm"
[item.EssenceTotals]
Vitality = 100
"#,
    )
    .expect("write toml");

    let derivations = load_derivations();
    let mut doc_item = read_stats_file(&on_item).expect("must parse");
    let mut doc_essence = read_stats_file(&in_essence).expect("must parse");
    derivations
        .derive_doc(LORE_MASTER, &mut doc_item)
        .expect("derive item variant");
    derivations
        .derive_doc(LORE_MASTER, &mut doc_essence)
        .expect("derive essence variant");

    // Lore-master Vitality → Morale 4.5: ceil(100 × 4.5) = 450 either way.
    assert_eq!(doc_item.items[0].item.stat(&Stat::Morale), 450);
    assert_eq!(
        doc_item.items[0].item.stats, doc_essence.items[0].item.stats,
        "essence base values must derive identically to item base values"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

/// The `base-stats` verb: raw innate Base stats plus the derived tracked
/// contributions, clearly labeled, without running an optimization.
#[test]
fn base_stats_verb_prints_raw_and_derived_sections() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lgo"))
        .args(["base-stats", "--file", THALYA_FIXTURE])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("lgo base-stats must run");
    assert!(
        output.status.success(),
        "base-stats must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Character : Thalya (Lore-master)"));
    // Raw section: the five Base stats with the fixture's innate values.
    assert!(stdout.contains("derivation inputs only"));
    for (name, value) in [
        ("Might", "5,300"),
        ("Agility", "2,650"),
        ("Vitality", "10,200"),
        ("Will", "7,950"),
        ("Fate", "4,000"),
    ] {
        assert!(
            stdout.contains(name) && stdout.contains(value),
            "raw section must list {} = {}; got:\n{}",
            name,
            value,
            stdout
        );
    }
    // Derived section: labeled as already included in optimize totals, with
    // values matching an independent derivation of the fixture's innate
    // base stats (all products integral for these inputs).
    assert!(stdout.contains("already included in optimize totals"));
    let derivations = load_derivations();
    let doc = read_stats_file(Path::new(THALYA_FIXTURE)).expect("fixture must parse");
    let derived = derivations
        .derive_stats(LORE_MASTER, &doc.innate_base_stats)
        .expect("innate derivation must succeed");
    for (stat, expected) in [
        (Stat::Morale, 45_900),
        (Stat::TacticalMastery, 39_750),
        (Stat::CriticalRating, 21_200),
    ] {
        assert_eq!(derived.get(&stat), Some(&expected));
        let formatted = format!("{}", expected)
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(|chunk| String::from_utf8_lossy(chunk).to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert!(
            stdout.contains(&formatted),
            "derived section must list {} = {}; got:\n{}",
            stat,
            formatted,
            stdout
        );
    }
    // No optimization output.
    assert!(!stdout.contains("Recommended Item"));
}
