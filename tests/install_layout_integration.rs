//! End-to-end tests for the exe-anchored install-directory layout: character
//! discovery, case-insensitive selection, the auto-selection message, the
//! stray-file move rule, mismatched-folder warnings, and `--to-file` report
//! routing (including out-of-tree `--file` TOMLs). Each test drives the real
//! `lgo` binary with `LGO_HOME` pointed at a private temp install directory, so
//! nothing touches the working directory or the repository tree.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn lgo_bin() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_BIN_EXE_lgo").expect("cargo must set lgo binary path"))
}

fn unique_dir(tag: &str) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "lgo_install_layout_{}_{}_{}",
        tag,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        n
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Create a temp install directory seeded with the `data/` files the commands
/// need (base-stat derivations and the items DB).
fn seed_install(tag: &str) -> PathBuf {
    let install = unique_dir(tag);
    let data = install.join("data");
    std::fs::create_dir_all(&data).expect("create data dir");
    let repo_data = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    for file in ["base_stat_derivations.json", "lgo_items.json"] {
        std::fs::copy(repo_data.join(file), data.join(file))
            .unwrap_or_else(|e| panic!("copy {file}: {e}"));
    }
    install
}

fn gear_ready_contents(character: &str) -> String {
    format!(
        r#"character = "{character}"
class = "Lore-master"

[[item]]
slot = "Head"
name = "Test Hat"
TacticalMastery = 50
"#
    )
}

/// Create `<install>/<character>_Gear/lgo_<character>_gearReady.toml`.
fn make_gear_folder(install: &Path, character: &str) -> PathBuf {
    let gear_dir = install.join(format!("{character}_Gear"));
    std::fs::create_dir_all(&gear_dir).expect("create gear dir");
    let ready = gear_dir.join(format!("lgo_{character}_gearReady.toml"));
    std::fs::write(&ready, gear_ready_contents(character)).expect("write gearReady");
    gear_dir
}

fn run(install: &Path, args: &[&str]) -> Output {
    Command::new(lgo_bin())
        .args(args)
        .current_dir(install)
        .env("LGO_HOME", install)
        .output()
        .expect("run lgo")
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Rewrite `path` until its mtime is strictly greater than `floor`, so recency
/// ordering is deterministic regardless of filesystem mtime granularity.
fn bump_mtime_past(path: &Path, floor: SystemTime) {
    loop {
        let text = std::fs::read_to_string(path).expect("read");
        std::fs::write(path, format!("{text}\n")).expect("rewrite");
        let m = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .expect("mtime");
        if m > floor {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn optimize_auto_selects_single_gear_folder() {
    let install = seed_install("single");
    make_gear_folder(&install, "Thalya");

    let output = run(&install, &["optimize", "tm:0"]);
    assert!(
        output.status.success(),
        "optimize must succeed; stderr:\n{}",
        stderr_of(&output)
    );
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains("Character : Thalya"),
        "should report Thalya; got:\n{stdout}"
    );
    // A single folder auto-selects silently (no recency note).
    assert!(!stdout.contains("most recently updated"));

    std::fs::remove_dir_all(&install).ok();
}

#[test]
fn optimize_auto_selects_most_recently_updated_of_multiple() {
    let install = seed_install("multi");
    make_gear_folder(&install, "Alpha");
    let beta_dir = make_gear_folder(&install, "Beta");
    let beta_ready = beta_dir.join("lgo_Beta_gearReady.toml");
    let alpha_ready = install.join("Alpha_Gear/lgo_Alpha_gearReady.toml");
    let alpha_mtime = std::fs::metadata(&alpha_ready)
        .and_then(|m| m.modified())
        .expect("alpha mtime");
    bump_mtime_past(&beta_ready, alpha_mtime);

    let output = run(&install, &["optimize", "tm:0"]);
    assert!(
        output.status.success(),
        "optimize must succeed; stderr:\n{}",
        stderr_of(&output)
    );
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains("Using character: Beta (most recently updated)"),
        "should announce the auto-selected character; got:\n{stdout}"
    );

    std::fs::remove_dir_all(&install).ok();
}

#[test]
fn optimize_character_flag_selects_named_folder_case_insensitively() {
    let install = seed_install("cflag");
    make_gear_folder(&install, "Alpha");
    make_gear_folder(&install, "Beta");

    // Lowercase request must match the `Alpha_Gear` folder.
    let output = run(&install, &["optimize", "-c", "alpha", "tm:0"]);
    assert!(
        output.status.success(),
        "optimize -c must succeed; stderr:\n{}",
        stderr_of(&output)
    );
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains("Character : Alpha"),
        "should select Alpha; got:\n{stdout}"
    );
    assert!(!stdout.contains("most recently updated"));

    std::fs::remove_dir_all(&install).ok();
}

#[test]
fn optimize_errors_when_no_gear_folders_exist() {
    let install = seed_install("zero");

    let output = run(&install, &["optimize", "tm:0"]);
    assert!(!output.status.success(), "must fail with no gear folders");
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("No character gear folders"),
        "should explain the expected layout; got:\n{stderr}"
    );

    std::fs::remove_dir_all(&install).ok();
}

#[test]
fn optimize_character_flag_without_match_errors() {
    let install = seed_install("nomatch");
    make_gear_folder(&install, "Thalya");

    let output = run(&install, &["optimize", "-c", "Legolas", "tm:0"]);
    assert!(
        !output.status.success(),
        "must fail with no matching folder"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("Legolas_Gear"),
        "should name the expected folder; got:\n{stderr}"
    );

    std::fs::remove_dir_all(&install).ok();
}

#[test]
fn base_stats_auto_selects_single_gear_folder() {
    let install = seed_install("basestats");
    make_gear_folder(&install, "Thalya");

    let output = run(&install, &["base-stats"]);
    assert!(
        output.status.success(),
        "base-stats must succeed; stderr:\n{}",
        stderr_of(&output)
    );
    assert!(stdout_of(&output).contains("Thalya"));

    std::fs::remove_dir_all(&install).ok();
}

#[test]
fn mismatched_folder_contents_warn_but_proceed() {
    let install = seed_install("mismatch");
    let gear_dir = make_gear_folder(&install, "Thalya");
    // A file named for a different character inside Thalya_Gear.
    std::fs::write(
        gear_dir.join("lgo_Bilbo_gearStats.toml"),
        "character = \"Bilbo\"\n",
    )
    .expect("write mismatched file");

    let output = run(&install, &["optimize", "tm:0"]);
    assert!(
        output.status.success(),
        "optimize must proceed on the folder name; stderr:\n{}",
        stderr_of(&output)
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("WARNING") && stderr.contains("Bilbo"),
        "should warn loudly about the mismatch; got:\n{stderr}"
    );
    // Discovery still used the directory name.
    assert!(stdout_of(&output).contains("Character : Thalya"));

    std::fs::remove_dir_all(&install).ok();
}

#[test]
fn resolve_slots_moves_stray_stats_file_and_creates_folder() {
    let install = seed_install("stray");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml");
    let stray = install.join("lgo_Thalya_gearStats.toml");
    std::fs::copy(&fixture, &stray).expect("seed stray gearStats");

    let output = run(&install, &["resolve-slots"]);
    assert!(
        output.status.success(),
        "resolve-slots must succeed; stderr:\n{}",
        stderr_of(&output)
    );
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains("Moved stray") && stdout.contains("Thalya_Gear"),
        "should report the stray-file move; got:\n{stdout}"
    );
    assert!(!stray.exists(), "stray file must be moved out of the root");
    assert!(
        install
            .join("Thalya_Gear/lgo_Thalya_gearStats.toml")
            .exists(),
        "stats file must land in the gear folder"
    );
    assert!(
        install
            .join("Thalya_Gear/lgo_Thalya_gearReady.toml")
            .exists(),
        "resolve-slots must write the canonical gearReady file"
    );

    std::fs::remove_dir_all(&install).ok();
}

#[test]
fn optimize_to_file_routes_report_into_reports_folder() {
    let install = seed_install("tofile");
    make_gear_folder(&install, "Thalya");

    let output = run(&install, &["optimize", "--to-file", "tm:0"]);
    assert!(
        output.status.success(),
        "optimize --to-file must succeed; stderr:\n{}",
        stderr_of(&output)
    );
    let reports = install.join("Thalya_Gear/Thalya_Reports");
    assert!(
        reports.join("lgo_GearReport_000.txt").exists()
            && reports.join("lgo_GearReport_000.html").exists(),
        "text and HTML reports must be written under the character's Reports folder"
    );

    std::fs::remove_dir_all(&install).ok();
}

#[test]
fn to_file_out_of_tree_uses_character_field_and_creates_folder() {
    let install = seed_install("outtree_field");
    let outside = unique_dir("outtree_field_src");
    let gear = outside.join("whatever.toml");
    std::fs::write(&gear, gear_ready_contents("Rosie")).expect("write out-of-tree gear");

    let output = run(
        &install,
        &[
            "optimize",
            "--file",
            gear.to_str().expect("utf-8"),
            "--to-file",
            "tm:0",
        ],
    );
    assert!(
        output.status.success(),
        "optimize must succeed; stderr:\n{}",
        stderr_of(&output)
    );
    let reports = install.join("Rosie_Gear/Rosie_Reports");
    assert!(
        reports.join("lgo_GearReport_000.txt").exists(),
        "report must route to the character from the `character` field"
    );

    std::fs::remove_dir_all(&install).ok();
    std::fs::remove_dir_all(&outside).ok();
}

#[test]
fn to_file_out_of_tree_uses_filename_fallback() {
    let install = seed_install("outtree_filename");
    let outside = unique_dir("outtree_filename_src");
    // No `character` field; the routing must fall back to the filename.
    let gear = outside.join("lgo_Sam_gearReady.toml");
    std::fs::write(
        &gear,
        "class = \"Lore-master\"\n\n[[item]]\nslot = \"Head\"\nname = \"Hat\"\nTacticalMastery = 5\n",
    )
    .expect("write out-of-tree gear");

    let output = run(
        &install,
        &[
            "optimize",
            "--file",
            gear.to_str().expect("utf-8"),
            "--to-file",
            "tm:0",
        ],
    );
    assert!(
        output.status.success(),
        "optimize must succeed; stderr:\n{}",
        stderr_of(&output)
    );
    assert!(
        install
            .join("Sam_Gear/Sam_Reports/lgo_GearReport_000.txt")
            .exists(),
        "report must route to the character from the filename convention"
    );

    std::fs::remove_dir_all(&install).ok();
    std::fs::remove_dir_all(&outside).ok();
}

#[test]
fn to_file_out_of_tree_unresolvable_character_errors() {
    let install = seed_install("outtree_unresolvable");
    let outside = unique_dir("outtree_unresolvable_src");
    // Neither a `character` field nor a canonical filename.
    let gear = outside.join("mystery.toml");
    std::fs::write(
        &gear,
        "class = \"Lore-master\"\n\n[[item]]\nslot = \"Head\"\nname = \"Hat\"\nTacticalMastery = 5\n",
    )
    .expect("write out-of-tree gear");

    let output = run(
        &install,
        &[
            "optimize",
            "--file",
            gear.to_str().expect("utf-8"),
            "--to-file",
            "tm:0",
        ],
    );
    assert!(
        !output.status.success(),
        "must fail when the report character cannot be resolved"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("Cannot determine the character"),
        "should explain the routing failure; got:\n{stderr}"
    );

    std::fs::remove_dir_all(&install).ok();
    std::fs::remove_dir_all(&outside).ok();
}
