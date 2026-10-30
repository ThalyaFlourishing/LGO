use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod common;

fn make_test_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lgo_build_profiles_{label}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn lgo_bin() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_BIN_EXE_lgo").expect("cargo must set lgo binary path"))
}

/// Seed a temp install directory with the `data/` files the optimize and
/// scrap-gear paths need (base-stat derivations), so tests are self-contained
/// and route their output under the temp tree rather than the repo. The
/// derivation table is a minimal synthetic one — these tests exercise saved
/// builds and scrap reporting, not the real per-class coefficients.
fn seed_install(dir: &Path) {
    common::write_derivations(&dir.join("data"));
    common::write_virtues(&dir.join("data"));
}

fn run_lgo(args: &[&str], install_dir: &Path) -> std::process::Output {
    Command::new(lgo_bin())
        .args(args)
        .current_dir(install_dir)
        .env("LGO_HOME", install_dir)
        .output()
        .expect("run lgo")
}

#[test]
fn optimize_save_build_round_trips_and_build_flag_loads_goals() {
    let dir = make_test_dir("save_roundtrip");
    seed_install(&dir);
    let gear = dir.join("lgo_Thalya_gearReady.toml");
    fs::write(
        &gear,
        r#"
character = "Thalya"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Crit Hat"
CriticalRating = 100

[[item]]
slot = "Head"
name = "Tank Hat"
TacticalMitigation = 100
"#,
    )
    .expect("write gear file");

    let gear_arg = gear.to_str().expect("utf-8 gear path");

    let saved = run_lgo(
        &[
            "optimize",
            "--file",
            gear_arg,
            "--save-build",
            "Burst",
            "cr:50",
        ],
        &dir,
    );
    assert!(
        saved.status.success(),
        "save-build optimize failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&saved.stdout),
        String::from_utf8_lossy(&saved.stderr)
    );
    let saved_stdout = String::from_utf8_lossy(&saved.stdout);
    assert!(saved_stdout.contains("Crit Hat"));
    assert!(saved_stdout.contains("Saved build 'Burst'"));

    let builds = dir.join("lgo_Thalya_builds.toml");
    let builds_text = fs::read_to_string(&builds).expect("builds file must exist");
    assert!(builds_text.contains(r#"[builds.Burst]"#));
    assert!(builds_text.contains(r#"goals = ["cr:50"]"#));

    let loaded = run_lgo(&["optimize", "--file", gear_arg, "--build", "burst"], &dir);
    assert!(
        loaded.status.success(),
        "build optimize failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&loaded.stdout),
        String::from_utf8_lossy(&loaded.stderr)
    );
    let loaded_stdout = String::from_utf8_lossy(&loaded.stdout);
    assert!(loaded_stdout.contains("Crit Hat"));

    fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn plain_optimize_ignores_builds_discovery_when_not_using_saved_build_flags() {
    let dir = make_test_dir("plain_optimize");
    seed_install(&dir);
    let gear = dir.join("my_test_gear.toml");
    fs::write(
        &gear,
        r#"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Crit Hat"
CriticalRating = 100
"#,
    )
    .expect("write gear file");

    let output = run_lgo(
        &[
            "optimize",
            "--file",
            gear.to_str().expect("utf-8 gear path"),
            "cr:50",
        ],
        &dir,
    );
    assert!(
        output.status.success(),
        "plain optimize failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn custom_named_gear_file_uses_toml_character_for_builds_file_discovery() {
    let dir = make_test_dir("custom_named_gear");
    seed_install(&dir);
    let gear = dir.join("my_test_gear.toml");
    fs::write(
        &gear,
        r#"
character = "Thalya"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Healing Hat"
OutgoingHealing = 20
"#,
    )
    .expect("write gear file");

    let output = run_lgo(
        &[
            "optimize",
            "--file",
            gear.to_str().expect("utf-8 gear path"),
            "--save-build",
            "healer",
            "oh:10",
        ],
        &dir,
    );
    assert!(
        output.status.success(),
        "save-build optimize failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let builds = dir.join("lgo_Thalya_builds.toml");
    let builds_text = fs::read_to_string(&builds).expect("builds file must exist");
    assert!(builds_text.contains(r#"[builds.healer]"#));

    fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn explicit_builds_file_override_is_used_for_save_build_and_build() {
    let dir = make_test_dir("explicit_builds_file");
    seed_install(&dir);
    let gear = dir.join("my_test_gear.toml");
    let builds = dir.join("my_saved_builds.toml");
    fs::write(
        &gear,
        r#"
character = "Thalya"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Healing Hat"
OutgoingHealing = 20

[[item]]
slot = "Head"
name = "Tank Hat"
TacticalMitigation = 20
"#,
    )
    .expect("write gear file");

    let gear_arg = gear.to_str().expect("utf-8 gear path");
    let builds_arg = builds.to_str().expect("utf-8 builds path");

    let save_output = run_lgo(
        &[
            "optimize",
            "--file",
            gear_arg,
            "--builds-file",
            builds_arg,
            "--save-build",
            "healer",
            "oh:10",
        ],
        &dir,
    );
    assert!(
        save_output.status.success(),
        "save-build optimize failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&save_output.stdout),
        String::from_utf8_lossy(&save_output.stderr)
    );

    let builds_text = fs::read_to_string(&builds).expect("explicit builds file must exist");
    assert!(builds_text.contains(r#"[builds.healer]"#));

    let build_output = run_lgo(
        &[
            "optimize",
            "--file",
            gear_arg,
            "--builds-file",
            builds_arg,
            "--build",
            "healer",
            "--save-build",
            "copy",
        ],
        &dir,
    );
    assert!(
        build_output.status.success(),
        "build optimize failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build_output.stdout),
        String::from_utf8_lossy(&build_output.stderr)
    );
    let rebuilt_text =
        fs::read_to_string(&builds).expect("explicit builds file must remain readable");
    assert!(rebuilt_text.contains(r#"[builds.healer]"#));
    assert!(rebuilt_text.contains(r#"[builds.copy]"#));

    fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn scrap_gear_reports_items_not_used_in_any_optimize_report_with_copy_counts() {
    let dir = make_test_dir("scrap_gear");
    seed_install(&dir);
    let gear_dir = dir.join("Thalya_Gear");
    fs::create_dir_all(&gear_dir).expect("create character gear folder");
    let gear = gear_dir.join("lgo_Thalya_gearReady.toml");
    fs::write(
        &gear,
        r#"
character = "Thalya"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Healing Hat"
OutgoingHealing = 20

[[item]]
slot = "Head"
name = "Tank Hat"
TacticalMitigation = 20

[[item]]
slot = "Off-hand"
name = "Shared Focus"
OutgoingHealing = 5
TacticalMitigation = 5

[[item]]
slot = "Off-hand"
name = "Shared Focus"
OutgoingHealing = 5
TacticalMitigation = 5

[[item]]
slot = "Off-hand"
name = "Unused Focus"
CriticalRating = 1
"#,
    )
    .expect("write gear file");

    for goal in ["oh:10", "tt:10"] {
        let optimize = run_lgo(&["optimize", "--to-file", goal], &dir);
        assert!(
            optimize.status.success(),
            "optimize --to-file failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&optimize.stdout),
            String::from_utf8_lossy(&optimize.stderr)
        );
    }

    let output = run_lgo(&["scrap-gear"], &dir);
    assert!(
        output.status.success(),
        "scrap-gear failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("treated as an optimize report"));
    assert!(stdout.contains("Reports read:"));
    assert!(stdout.contains("- lgo_GearReport_000.txt: -"));
    assert!(stdout.contains("- lgo_GearReport_001.txt: -"));
    assert!(!stdout.contains("Skipped files:"));
    assert!(stdout.contains("Items you can scrap:"));
    assert!(stdout.contains("Items of which you have more than one, but only need one:"));
    assert!(stdout.contains("    - Shared Focus"));
    assert!(stdout.contains("    - Unused Focus"));
    assert!(!stdout.contains("Saved builds evaluated:"));

    let reports_dir = gear_dir.join("Thalya_Reports");
    let scrap_files = |dir: &Path| -> Vec<String> {
        fs::read_dir(dir)
            .expect("read reports dir")
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name.starts_with("lgo_Thalya_ScrapGear_Report_"))
            .collect()
    };
    let first_run_files = scrap_files(&reports_dir);
    assert_eq!(
        first_run_files.len(),
        1,
        "exactly one scrap-gear report file must be written: {:?}",
        first_run_files
    );
    assert!(first_run_files[0].ends_with(".txt"));

    // A second run must silently skip the first run's own report file.
    let second = run_lgo(&["scrap-gear"], &dir);
    assert!(
        second.status.success(),
        "second scrap-gear failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );
    let second_stdout = String::from_utf8_lossy(&second.stdout);
    assert!(!second_stdout.contains("ScrapGear_Report_: "));
    assert!(!second_stdout.contains(&format!("- {}", first_run_files[0])));
    assert!(!second_stdout.contains("Skipped files:"));

    fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn scrap_gear_reads_reports_from_named_subfolder() {
    let dir = make_test_dir("scrap_gear_subfolder");
    seed_install(&dir);
    let gear_dir = dir.join("Thalya_Gear");
    let reports_dir = gear_dir.join("Thalya_Reports");
    let subfolder = reports_dir.join("healer");
    fs::create_dir_all(&subfolder).expect("create reports subfolder");
    let gear = gear_dir.join("lgo_Thalya_gearReady.toml");
    fs::write(
        &gear,
        r#"
character = "Thalya"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Healing Hat"
OutgoingHealing = 20

[[item]]
slot = "Head"
name = "Tank Hat"
TacticalMitigation = 20
"#,
    )
    .expect("write gear file");

    // A handcrafted optimize report in the subfolder. A sibling report in the
    // parent folder must be ignored when --reports-folder is used.
    let divider = "─".repeat(65);
    let healer_report = format!(
        "  LGO — Thalya's Gear Optimizer\n  Build Name: healer build\n  Run time  : 2026-08-31 08:00:00 +00:00\n  {divider}\n  Recommended Gear:\n          Slot  Recommended Item\n  {divider}\n            Head  Healing Hat\n  {divider}\n"
    );
    fs::write(subfolder.join("healer.txt"), &healer_report).expect("write healer report");
    fs::write(
        reports_dir.join("parent_only.txt"),
        healer_report.replace("Healing Hat", "Tank Hat"),
    )
    .expect("write parent report");

    let output = run_lgo(&["scrap-gear", "--reports-folder", "healer"], &dir);
    assert!(
        output.status.success(),
        "scrap-gear --reports-folder failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Reports read:"));
    assert!(stdout.contains("- healer.txt: healer build"));
    assert!(
        !stdout.contains("parent_only.txt"),
        "--reports-folder must not read the parent folder"
    );
    // Healing Hat was used in the subfolder report; Tank Hat was not.
    assert!(stdout.contains("    - Tank Hat"));
    assert!(!stdout.contains("    - Healing Hat"));

    // The scrap report itself still lands in the parent Reports folder.
    let wrote_to_parent = fs::read_dir(&reports_dir)
        .expect("read reports dir")
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .any(|name| name.starts_with("lgo_Thalya_ScrapGear_Report_"));
    assert!(
        wrote_to_parent,
        "scrap report must be written to the parent Reports folder"
    );
    let wrote_to_subfolder = fs::read_dir(&subfolder)
        .expect("read subfolder")
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .any(|name| name.starts_with("lgo_Thalya_ScrapGear_Report_"));
    assert!(!wrote_to_subfolder);

    let missing = run_lgo(&["scrap-gear", "--reports-folder", "no-such"], &dir);
    assert!(!missing.status.success());
    let missing_stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(missing_stderr.contains("does not exist"));

    let escape = run_lgo(&["scrap-gear", "--reports-folder", "../escape"], &dir);
    assert!(!escape.status.success());
    let escape_stderr = String::from_utf8_lossy(&escape.stderr);
    assert!(escape_stderr.contains("Invalid --reports-folder value"));

    fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn scrap_gear_errors_when_no_optimize_reports_can_be_read() {
    let dir = make_test_dir("scrap_gear_no_reports");
    seed_install(&dir);
    let gear_dir = dir.join("Thalya_Gear");
    let reports_dir = gear_dir.join("Thalya_Reports");
    fs::create_dir_all(&reports_dir).expect("create reports folder");
    let gear = gear_dir.join("lgo_Thalya_gearReady.toml");
    fs::write(
        &gear,
        r#"
character = "Thalya"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Healing Hat"
OutgoingHealing = 20
"#,
    )
    .expect("write gear file");

    // Only a scrap-gear report (skipped silently) — no optimize reports.
    fs::write(
        reports_dir.join("lgo_Thalya_ScrapGear_Report_20000101_000000.txt"),
        "old scrap output",
    )
    .expect("write old scrap report");

    let output = run_lgo(&["scrap-gear"], &dir);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no optimize reports could be read"));
    assert!(stderr.contains("lgo optimize --to-file"));

    fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn scrap_gear_lists_unparsable_files_under_skipped_files() {
    let dir = make_test_dir("scrap_gear_skipped");
    seed_install(&dir);
    let gear_dir = dir.join("Thalya_Gear");
    let reports_dir = gear_dir.join("Thalya_Reports");
    fs::create_dir_all(&reports_dir).expect("create reports folder");
    let gear = gear_dir.join("lgo_Thalya_gearReady.toml");
    fs::write(
        &gear,
        r#"
character = "Thalya"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Healing Hat"
OutgoingHealing = 20
"#,
    )
    .expect("write gear file");

    let divider = "─".repeat(65);
    let good_report = format!(
        "  Build Name: -\n  Run time  : 2026-08-31 08:00:00 +00:00\n  Recommended Gear:\n          Slot  Recommended Item\n  {divider}\n            Head  Healing Hat\n  {divider}\n"
    );
    fs::write(reports_dir.join("good.txt"), &good_report).expect("write good report");
    fs::write(reports_dir.join("notes.txt"), "shopping list\n").expect("write notes file");

    let output = run_lgo(&["scrap-gear"], &dir);
    assert!(
        output.status.success(),
        "scrap-gear failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("- good.txt: -"));
    assert!(stdout.contains("Skipped files:"));
    assert!(stdout.contains("- notes.txt: no \"Recommended Gear:\" section found"));

    fs::remove_dir_all(&dir).expect("cleanup");
}
