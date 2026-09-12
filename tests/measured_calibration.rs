//! Integration tests for the measured innate Morale/Power calibration pass.
//!
//! The innate baseline is measured, not modelled (Bug 14): `[MeasuredStats]`
//! records the dressed character's Max Morale/Power and equipped item names,
//! and LGO subtracts everything it already accounts for. What remains is the
//! baseline. The invariant these tests pin down: re-equipping exactly the
//! exported set reproduces the exported maxima.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lgo::base_stats::BaseStatDerivations;
use lgo::gear::optimizer_candidate_key;
use lgo::gearstats::read_stats_file;
use lgo::measured::calibrate_innate;
use lgo::optimizer::optimize;
use lgo::stat::{Stat, StatGoal};

const LORE_MASTER: &str = "Lore-master";

/// A gear file carrying a measured export: one equipped Head item plus a
/// spare the character owns but is not wearing.
const MEASURED_DOC: &str = r#"character = "Thalya"
class = "Lore-master"
level = 160

[InnateStats]
Might              = 0
Agility            = 0
Vitality           = 100
Will               = 0
Fate               = 0

[MeasuredStats]
MaxMorale          = 100000
MaxPower           = 20000
ActiveEffects      = 0
Equipped           = ["Measured Helm"]

[[item]]
slot = "Head"
name = "Measured Helm"
Morale             = 5000
Power              = 1000

[[item]]
slot = "Head"
name = "Spare Helm"
Morale             = 1
Power              = 1
"#;

fn make_test_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lgo_measured_{}_{}",
        label,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_doc(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write gear file");
    path
}

fn run_lgo(args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_lgo"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("lgo must run")
}

/// The calibrated baseline is the measurement minus the equipped set, and
/// re-selecting that set reproduces the measured maxima exactly.
#[test]
fn calibrated_innate_reproduces_the_measured_maxima() {
    let dir = make_test_dir("roundtrip");
    let path = write_doc(&dir, "lgo_Thalya_gearReady.toml", MEASURED_DOC);

    let derivations =
        BaseStatDerivations::load_default().expect("data/base_stat_derivations.json must load");
    let mut doc = read_stats_file(&path).expect("fixture must parse");
    derivations
        .derive_doc(LORE_MASTER, &mut doc)
        .expect("derivation pre-pass must succeed");

    // Lore-master Vitality → Morale 4.5: the derived-only innate Morale is
    // ceil(100 × 4.5) = 450, nowhere near the measured maximum.
    assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&450));

    let report = calibrate_innate(&mut doc).expect("the file carries [MeasuredStats]");

    assert_eq!(report.level, Some(160));
    assert_eq!(report.active_effects, 0);
    assert!(report.unmatched.is_empty());
    // 100,000 measured − 5,000 on the equipped helm. The spare helm is not
    // equipped, so it is not subtracted.
    assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&95_000));
    assert_eq!(doc.innate_stats.get(&Stat::Power), Some(&19_000));
    assert_eq!(report.innate_morale, 95_000);
    assert_eq!(report.innate_power, 19_000);

    let resolved: HashMap<String, lgo::gear::GearItem> = doc
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
    let keys: Vec<String> = resolved.keys().cloned().collect();
    let goals = vec!["ml:100000".parse::<StatGoal>().expect("goal must parse")];
    let result = optimize(&resolved, &keys, &goals, &doc.innate_stats);

    assert_eq!(
        result.gear_set.total(&Stat::Morale),
        100_000,
        "wearing the exported set again must reproduce the measured Max Morale"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

/// `base-stats` reports the measurement and the baseline derived from it.
#[test]
fn base_stats_prints_measured_and_calibrated_values() {
    let dir = make_test_dir("basestats");
    let path = write_doc(&dir, "lgo_Thalya_gearReady.toml", MEASURED_DOC);

    let output = run_lgo(&["base-stats", "--file", &path.display().to_string()]);
    assert!(
        output.status.success(),
        "base-stats must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("[MeasuredStats]"), "got:\n{}", stdout);
    assert!(
        stdout.contains("100,000"),
        "measured Max Morale; got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("20,000"),
        "measured Max Power; got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("95,000") && stdout.contains("19,000"),
        "calibrated baseline; got:\n{}",
        stdout
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

/// Without a `[MeasuredStats]` block nothing is calibrated, and the report
/// says so instead of inventing a baseline.
#[test]
fn base_stats_notes_a_missing_measured_block() {
    let dir = make_test_dir("nomeasured");
    let body = "character = \"Thalya\"\nclass = \"Lore-master\"\n\n[InnateStats]\nVitality           = 100\n\n[[item]]\nslot = \"Head\"\nname = \"Measured Helm\"\nMorale             = 5000\n";
    let path = write_doc(&dir, "lgo_Thalya_gearReady.toml", body);

    let output = run_lgo(&["base-stats", "--file", &path.display().to_string()]);
    assert!(
        output.status.success(),
        "base-stats must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No [MeasuredStats] block"),
        "got:\n{}",
        stdout
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

/// An equipped name LGO owns no item for warns on stderr and is absorbed
/// into the baseline — never an error.
#[test]
fn unmatched_equipped_item_warns_without_failing() {
    let dir = make_test_dir("unmatched");
    let body = MEASURED_DOC.replace(
        "Equipped           = [\"Measured Helm\"]",
        "Equipped           = [\"Measured Helm\", \"Mystery Trinket\"]",
    );
    let path = write_doc(&dir, "lgo_Thalya_gearReady.toml", &body);

    let output = run_lgo(&["base-stats", "--file", &path.display().to_string()]);
    assert!(
        output.status.success(),
        "an unmatched equipped item must not fail the run; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Mystery Trinket"),
        "the warning must name the item; got:\n{}",
        stderr
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("95,000"),
        "the residual still absorbs the gap; got:\n{}",
        stdout
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}
