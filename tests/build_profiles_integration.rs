use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn run_lgo(args: &[&str], current_dir: &Path) -> std::process::Output {
    Command::new(lgo_bin())
        .args(args)
        .current_dir(current_dir)
        .output()
        .expect("run lgo")
}

#[test]
fn optimize_save_build_round_trips_and_build_flag_loads_goals() {
    let dir = make_test_dir("save_roundtrip");
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

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
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
        repo_root,
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
    assert!(builds_text.contains(r#"[builds."Burst"]"#));
    assert!(builds_text.contains(r#"goals = ["cr:50"]"#));

    let loaded = run_lgo(&["optimize", "--file", gear_arg, "--build", "burst"], repo_root);
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
fn scrap_gear_reports_items_not_used_in_any_saved_build_with_copy_counts() {
    let dir = make_test_dir("scrap_gear");
    let gear = dir.join("lgo_Thalya_gearReady.toml");
    let builds = dir.join("lgo_Thalya_builds.toml");
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
Morale = 1
"#,
    )
    .expect("write gear file");
    fs::write(
        &builds,
        r#"
[builds.healer]
goals = ["oh:10"]

[builds.tank]
goals = ["tt:10"]
"#,
    )
    .expect("write builds file");

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = run_lgo(
        &[
            "scrap-gear",
            "--file",
            gear.to_str().expect("utf-8 gear path"),
        ],
        repo_root,
    );
    assert!(
        output.status.success(),
        "scrap-gear failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Saved builds evaluated:"));
    assert!(stdout.contains("- healer: oh:10"));
    assert!(stdout.contains("- tank: tt:10"));
    assert!(stdout.contains("Items not used in any saved build"));
    assert!(stdout.contains("These items may still be near-misses."));
    assert!(stdout.contains(
        "Shared Focus — 2 owned, at most 1 used in any build (1 copy not used in any saved build)"
    ));
    assert!(stdout.contains(
        "Unused Focus — 1 owned, at most 0 used in any build (1 copy not used in any saved build)"
    ));

    fs::remove_dir_all(&dir).expect("cleanup");
}
