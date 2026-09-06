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

/// Seed a temp install directory with the `data/` files the optimize and
/// scrap-gear paths need (base-stat derivations), so tests are self-contained
/// and route their output under the temp tree rather than the repo.
fn seed_install(dir: &Path) {
    let data_dir = dir.join("data");
    fs::create_dir_all(&data_dir).expect("create data dir");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data/base_stat_derivations.json"),
        data_dir.join("base_stat_derivations.json"),
    )
    .expect("copy derivations");
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

    let loaded = run_lgo(
        &["optimize", "--file", gear_arg, "--build", "burst"],
        &dir,
    );
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
fn explicit_builds_file_override_is_used_for_save_build_build_and_scrap_gear() {
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

    let scrap_output = run_lgo(
        &[
            "scrap-gear",
            "--file",
            gear_arg,
            "--builds-file",
            builds_arg,
        ],
        &dir,
    );
    assert!(
        scrap_output.status.success(),
        "scrap-gear failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&scrap_output.stdout),
        String::from_utf8_lossy(&scrap_output.stderr)
    );
    let scrap_stdout = String::from_utf8_lossy(&scrap_output.stdout);
    assert!(scrap_stdout.contains("Saved builds evaluated:"));
    assert!(scrap_stdout.contains("- healer: oh:10"));
    assert!(scrap_stdout.contains("- copy: oh:10"));

    fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn scrap_gear_reports_items_not_used_in_any_saved_build_with_copy_counts() {
    let dir = make_test_dir("scrap_gear");
    seed_install(&dir);
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

    let output = run_lgo(
        &[
            "scrap-gear",
            "--file",
            gear.to_str().expect("utf-8 gear path"),
        ],
        &dir,
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
    assert!(stdout.contains("Items you can scrap:"));
    assert!(stdout.contains("Items of which you have more than one, but only need one:"));
    assert!(stdout.contains("    - Shared Focus"));
    assert!(stdout.contains("    - Unused Focus"));
    assert!(!stdout.contains("Items not used in any saved build"));
    assert!(!stdout.contains("near-misses"));
    assert!(!stdout.contains("owned, at most"));

    fs::remove_dir_all(&dir).expect("cleanup");
}
