//! Smoke test: every hand-maintained fixture in `TestData/` still loads.
//!
//! Broken fixtures (e.g. a missing `]`) otherwise surface only as confusing
//! failures in unrelated tests. These tests assert nothing about content;
//! they only prove each file loads, and name the file and error if not.

use std::path::{Path, PathBuf};

fn test_data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData")
}

/// The newest `lgo_Thalya_gearNames_*.plugindata` fixture, chosen by sorted
/// filename so a fresh export can replace the old one without code changes.
fn newest_plugindata_fixture_path() -> PathBuf {
    let test_data = test_data_dir();
    let mut matches: Vec<PathBuf> = std::fs::read_dir(&test_data)
        .unwrap_or_else(|e| panic!("cannot read {}: {}", test_data.display(), e))
        .map(|entry| {
            entry
                .unwrap_or_else(|e| panic!("cannot read entry in {}: {}", test_data.display(), e))
                .path()
        })
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("lgo_Thalya_gearNames_") && name.ends_with(".plugindata")
                })
        })
        .collect();
    matches.sort();
    matches.pop().unwrap_or_else(|| {
        panic!(
            "no lgo_Thalya_gearNames_*.plugindata fixture found in {}",
            test_data.display()
        )
    })
}

#[test]
fn gear_ready_fixture_loads() {
    let path = test_data_dir().join("lgo_Thalya_gearReady.toml");
    if let Err(e) = lgo::gearstats::read_stats_file(&path) {
        panic!("fixture {} failed to load: {}", path.display(), e);
    }
}

#[test]
fn gear_stats_fixture_parses_as_toml() {
    let path = test_data_dir().join("lgo_Thalya_gearStats.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {} cannot be read: {}", path.display(), e));
    if let Err(e) = text.parse::<toml_edit::DocumentMut>() {
        panic!("fixture {} is not valid TOML: {}", path.display(), e);
    }
}

#[test]
fn newest_plugindata_fixture_loads() {
    let path = newest_plugindata_fixture_path();
    if let Err(e) = lgo::plugindata::load(&path) {
        panic!("fixture {} failed to load: {}", path.display(), e);
    }
}
