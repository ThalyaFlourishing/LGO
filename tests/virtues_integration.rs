use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lgo::base_stats::BaseStatDerivations;
use lgo::gear::optimizer_candidate_key;
use lgo::gearstats::read_stats_file;
use lgo::optimizer::optimize;
use lgo::stat::{Stat, StatGoal};
use lgo::virtues::VirtuesDb;

const LORE_MASTER: &str = "Lore-master";

fn load_derivations() -> BaseStatDerivations {
    BaseStatDerivations::load_default().expect("data/base_stat_derivations.json must load")
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

fn optimize_tm_total(path: &Path, virtues_json: &str) -> i64 {
    let mut doc = read_stats_file(path).expect("gear toml must parse");
    let virtues =
        VirtuesDb::from_json_str(virtues_json, Path::new("<test>")).expect("virtue db must parse");
    virtues
        .apply_selected_virtues(&mut doc)
        .expect("virtues must apply");
    load_derivations()
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
    optimize(&resolved, &keys, &tm_goal(), &doc.innate_stats)
        .gear_set
        .total(&Stat::TacticalMastery)
}

#[test]
fn selected_virtue_tracked_stats_contribute_directly() {
    let dir = make_test_dir();
    let path = dir.join("gear.toml");
    std::fs::write(
        &path,
        r#"
class = "Lore-master"

[Virtues]
Virtue1 = "Valour"

[[item]]
slot = "Head"
name = "Statless Helm"
"#,
    )
    .expect("write toml");

    let total = optimize_tm_total(&path, r#"{ "Valour": { "TacticalMastery": 75 } }"#);
    assert_eq!(total, 75);

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn selected_virtue_base_stats_flow_through_derivation_path() {
    let dir = make_test_dir();
    let path = dir.join("gear.toml");
    std::fs::write(
        &path,
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

    let total = optimize_tm_total(&path, r#"{ "Wisdom": { "Will": 100 } }"#);
    // Lore-master Will → TacticalMastery uses the current 3.0 derivation
    // coefficient, so 100 Will contributes 300 Tactical Mastery here.
    assert_eq!(total, 300);

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn optimize_succeeds_with_empty_virtues_even_without_virtues_data_file() {
    let dir = make_test_dir();
    let data_dir = dir.join("data");
    std::fs::create_dir_all(&data_dir).expect("create data dir");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data/base_stat_derivations.json"),
        data_dir.join("base_stat_derivations.json"),
    )
    .expect("copy derivations");

    let gear = dir.join("gear.toml");
    std::fs::write(
        &gear,
        r#"
class = "Lore-master"

[InnateStats]
Will = 7950

[Virtues]
Virtue1 = ""
Virtue2 = " "
Virtue3 = ""
Virtue4 = ""
Virtue5 = ""

[[item]]
slot = "Head"
name = "Statless Helm"
"#,
    )
    .expect("write toml");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lgo"))
        .args([
            "optimize",
            "--file",
            gear.to_str().expect("utf-8 path"),
            "tm:0",
        ])
        .current_dir(&dir)
        .output()
        .expect("lgo optimize must run");
    assert!(
        output.status.success(),
        "optimize must succeed without a virtues data file when all Virtue fields are empty; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn optimize_reports_unknown_virtue_name_clearly() {
    let dir = make_test_dir();
    let data_dir = dir.join("data");
    std::fs::create_dir_all(&data_dir).expect("create data dir");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data/base_stat_derivations.json"),
        data_dir.join("base_stat_derivations.json"),
    )
    .expect("copy derivations");
    std::fs::write(
        data_dir.join("lgo_virtues.json"),
        r#"{ "Wisdom": { "Will": 100 } }"#,
    )
    .expect("write virtues data");

    let gear = dir.join("gear.toml");
    std::fs::write(
        &gear,
        r#"
class = "Lore-master"

[Virtues]
Virtue1 = "Insight"

[[item]]
slot = "Head"
name = "Statless Helm"
"#,
    )
    .expect("write toml");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lgo"))
        .args([
            "optimize",
            "--file",
            gear.to_str().expect("utf-8 path"),
            "tm:0",
        ])
        .current_dir(&dir)
        .output()
        .expect("lgo optimize must run");
    assert!(
        !output.status.success(),
        "optimize must fail on unknown Virtue"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown Virtue 'Insight'"));
    assert!(stderr.contains("data/lgo_virtues.json"));

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn optimize_reports_duplicate_virtue_name_clearly() {
    let dir = make_test_dir();
    let data_dir = dir.join("data");
    std::fs::create_dir_all(&data_dir).expect("create data dir");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data/base_stat_derivations.json"),
        data_dir.join("base_stat_derivations.json"),
    )
    .expect("copy derivations");
    std::fs::write(
        data_dir.join("lgo_virtues.json"),
        r#"{ "Wisdom": { "Will": 100 } }"#,
    )
    .expect("write virtues data");

    let gear = dir.join("gear.toml");
    std::fs::write(
        &gear,
        r#"
class = "Lore-master"

[Virtues]
Virtue1 = "Wisdom"
Virtue2 = " wisdom "

[[item]]
slot = "Head"
name = "Statless Helm"
"#,
    )
    .expect("write toml");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lgo"))
        .args([
            "optimize",
            "--file",
            gear.to_str().expect("utf-8 path"),
            "tm:0",
        ])
        .current_dir(&dir)
        .output()
        .expect("lgo optimize must run");
    assert!(
        !output.status.success(),
        "optimize must fail on duplicate Virtues"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Duplicate Virtue 'Wisdom'"));

    std::fs::remove_dir_all(&dir).expect("cleanup");
}
