//! Integration coverage for optimizer-chosen Virtues.
//!
//! Virtues are no longer picked by the user: `optimize` selects the best five
//! distinct Virtues from `data/lgo_virtues.json` jointly with the gear. These
//! tests pin the data-file contract at the binary boundary (the file is
//! required; fewer than five Virtues is a hard error; the removed `[Virtues]`
//! block is rejected) and that `VirtuesDb::candidates` folds each Virtue's
//! Base stats through the class derivation table before the optimizer sees
//! them.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lgo::base_stats::BaseStatDerivations;
use lgo::gear::optimizer_candidate_key;
use lgo::gearstats::read_stats_file;
use lgo::optimizer::optimize;
use lgo::stat::{Stat, StatGoal};
use lgo::virtues::VirtuesDb;

mod common;

const LORE_MASTER: &str = "Lore-master";

fn load_derivations() -> BaseStatDerivations {
    BaseStatDerivations::from_json_str(common::DERIVATIONS_JSON, Path::new("synthetic"))
        .expect("synthetic derivations must parse")
}

fn tm_goal() -> Vec<StatGoal> {
    vec!["tm:0".parse::<StatGoal>().expect("tm:0 must parse")]
}

fn make_test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lgo_virtues_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

const STATLESS_GEAR: &str = r#"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Statless Helm"
"#;

fn run_optimize(dir: &Path, gear: &Path) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_lgo"))
        .args([
            "optimize",
            "--file",
            gear.to_str().expect("utf-8 path"),
            "tm:0",
        ])
        .current_dir(dir)
        .env("LGO_HOME", dir)
        .output()
        .expect("lgo optimize must run")
}

/// Optimize a statless gear doc against `virtues_json` so every tracked total
/// comes from the five chosen Virtues alone.
fn optimize_statless_with_virtues(virtues_json: &str) -> lgo::gear::GearSet {
    let dir = make_test_dir();
    let path = dir.join("gear.toml");
    std::fs::write(&path, STATLESS_GEAR).expect("write toml");

    let mut doc = read_stats_file(&path).expect("gear toml must parse");
    let derivations = load_derivations();
    derivations
        .derive_doc(LORE_MASTER, &mut doc)
        .expect("derivation pre-pass must succeed");
    let virtues = VirtuesDb::from_json_str(virtues_json, Path::new("<test>"))
        .expect("virtue db must parse")
        .candidates(LORE_MASTER, &derivations)
        .expect("virtue candidates must derive");

    let resolved: HashMap<String, lgo::gear::GearItem> = doc
        .items
        .into_iter()
        .map(|doc_item| doc_item.item)
        .enumerate()
        .map(|(idx, item)| (optimizer_candidate_key(idx, &item), item))
        .collect();
    let keys: Vec<String> = resolved.keys().cloned().collect();
    let result = optimize(&resolved, &keys, &tm_goal(), &doc.innate_stats, &virtues);

    std::fs::remove_dir_all(&dir).expect("cleanup");
    result.gear_set
}

#[test]
fn chosen_virtue_tracked_stats_contribute_directly() {
    // Five Virtues, exactly five slots: all are chosen, so the total is the
    // sum of their direct tracked stats.
    let gear_set = optimize_statless_with_virtues(
        r#"{
            "A": { "TacticalMastery": 75 },
            "B": { "TacticalMastery": 10 },
            "C": { "CriticalRating": 5 },
            "D": { "Finesse": 5 },
            "E": { "Resistance": 5 }
        }"#,
    );
    assert_eq!(gear_set.virtues.len(), 5);
    assert_eq!(gear_set.total(&Stat::TacticalMastery), 85);
    assert_eq!(gear_set.total(&Stat::CriticalRating), 5);
}

#[test]
fn chosen_virtue_base_stats_flow_through_derivation_path() {
    let gear_set = optimize_statless_with_virtues(
        r#"{
            "Wisdom": { "Will": 100 },
            "B": { "Vitality": 1 },
            "C": { "Vitality": 2 },
            "D": { "Vitality": 3 },
            "E": { "Vitality": 4 }
        }"#,
    );
    // Wisdom's 100 Will must flow through the derivation path into Tactical
    // Mastery. Derive the expectation from the same synthetic table rather
    // than a real fixture number.
    let expected = load_derivations()
        .derive_stats(LORE_MASTER, &HashMap::from([(Stat::Will, 100)]))
        .expect("synthetic derivation must succeed")
        .get(&Stat::TacticalMastery)
        .copied()
        .expect("Will must derive Tactical Mastery in the synthetic table");
    assert!(expected > 0, "synthetic table must map Will to TM");
    assert_eq!(gear_set.total(&Stat::TacticalMastery), expected);

    // The raw Base stat is kept alongside so reports can show it in the
    // projected Base-stat pool.
    let wisdom = gear_set
        .virtues
        .iter()
        .find(|v| v.name == "Wisdom")
        .expect("Wisdom must be slotted");
    assert_eq!(wisdom.base_stats.get(&Stat::Will), Some(&100));
}

#[test]
fn chosen_virtue_armour_derives_mitigations_and_leaves_no_tracked_armour() {
    // Armour is a Base stat: a Virtue granting Armour = 500 must derive
    // +500 PhysicalMitigation and +ceil(0.2 × 500) = +100 TacticalMitigation,
    // with no raw Armour in the tracked totals.
    let gear_set = optimize_statless_with_virtues(
        r#"{
            "Loyalty": { "Armour": 500 },
            "B": { "Vitality": 1 },
            "C": { "Vitality": 2 },
            "D": { "Vitality": 3 },
            "E": { "Vitality": 4 }
        }"#,
    );
    assert_eq!(gear_set.total(&Stat::PhysicalMitigation), 500);
    assert_eq!(gear_set.total(&Stat::TacticalMitigation), 100);
    assert_eq!(
        gear_set.total(&Stat::Armor),
        0,
        "Armour is a derivation input and must never appear in tracked totals"
    );
}

#[test]
fn optimize_requires_the_virtues_data_file() {
    let dir = make_test_dir();
    common::write_derivations(&dir.join("data"));

    let gear = dir.join("gear.toml");
    std::fs::write(&gear, STATLESS_GEAR).expect("write toml");

    let output = run_optimize(&dir, &gear);
    assert!(
        !output.status.success(),
        "optimize must fail without data/lgo_virtues.json"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to load virtue data") && stderr.contains("lgo_virtues.json"),
        "stderr must explain the missing file:\n{stderr}"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn optimize_rejects_fewer_than_five_virtues_in_the_data_file() {
    let dir = make_test_dir();
    let data_dir = dir.join("data");
    common::write_derivations(&data_dir);
    std::fs::write(
        data_dir.join("lgo_virtues.json"),
        r#"{ "Wisdom": { "Will": 100 }, "Valour": { "TacticalMastery": 10 } }"#,
    )
    .expect("write virtues data");

    let gear = dir.join("gear.toml");
    std::fs::write(&gear, STATLESS_GEAR).expect("write toml");

    let output = run_optimize(&dir, &gear);
    assert!(
        !output.status.success(),
        "optimize must fail with only two Virtues"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("only 2 Virtue(s)") && stderr.contains("at least 5"),
        "stderr must explain the shortfall:\n{stderr}"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn optimize_rejects_a_virtues_block_in_the_gear_file() {
    let dir = make_test_dir();
    let data_dir = dir.join("data");
    common::write_derivations(&data_dir);
    common::write_virtues(&data_dir);

    let gear = dir.join("gear.toml");
    std::fs::write(
        &gear,
        r#"
class = "Lore-master"

[Virtues]
Virtue1 = "Wisdom"

[[item]]
slot = "Head"
name = "Statless Helm"
"#,
    )
    .expect("write toml");

    let output = run_optimize(&dir, &gear);
    assert!(
        !output.status.success(),
        "a [Virtues] block must be rejected as an unknown top-level key"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown top-level key `Virtues`"),
        "stderr must name the unknown key:\n{stderr}"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn optimize_report_lists_the_five_chosen_virtues() {
    let dir = make_test_dir();
    let data_dir = dir.join("data");
    common::write_derivations(&data_dir);
    common::write_virtues(&data_dir);

    let gear = dir.join("gear.toml");
    std::fs::write(&gear, STATLESS_GEAR).expect("write toml");

    let output = run_optimize(&dir, &gear);
    assert!(
        output.status.success(),
        "optimize must succeed; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Virtues (chosen by the optimizer):"),
        "report must carry a Virtues section:\n{stdout}"
    );
    let listed = [
        "Test Charity",
        "Test Fidelity",
        "Test Honour",
        "Test Loyalty",
        "Test Valour",
        "Test Wisdom",
    ]
    .iter()
    .filter(|name| stdout.contains(*name))
    .count();
    assert_eq!(
        listed, 5,
        "exactly five of six Virtues must be listed:\n{stdout}"
    );
    // tm:0 with a statless helm: Test Valour (TM 10) and Test Wisdom (Will →
    // TM) are the only TM sources and must both be chosen.
    assert!(stdout.contains("Test Valour") && stdout.contains("Test Wisdom"));
    assert!(
        !stdout.contains("Contributed stats"),
        "Virtues section must list names only:\n{stdout}"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}
