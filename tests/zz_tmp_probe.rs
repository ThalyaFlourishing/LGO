use std::path::{Path, PathBuf};

#[test]
fn probe_header_output() {
    let dir = std::env::temp_dir().join("lgo_probe_hdr");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let character = "Thalya";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = root.join("TestData/lgo_Thalya_gearStats.toml");
    let pd: PathBuf = std::fs::read_dir(root.join("TestData")).unwrap()
        .filter_map(|e| e.ok()).map(|e| e.path())
        .find(|p| p.to_string_lossy().contains("gearNames_")).unwrap();
    std::fs::copy(&pd, dir.join("lgo_Thalya_gearNames_20260906_025012.plugindata")).unwrap();
    let db = lgo::slot_resolver::ItemsDb::load_default().unwrap();
    for run in 1..=3 {
        std::fs::copy(&fixture, &bookmarklet).unwrap();
        lgo::slot_resolver::resolve_stats_file(&dir, Some(&dir), character, &db, lgo::slot_resolver::ForceMode::NoForce).unwrap();
        let out = std::fs::read_to_string(&canonical).unwrap();
        if run == 3 {
            let head: String = out.lines().take(30).collect::<Vec<_>>().join("\n");
            println!("--- run {run} ---\n{head}");
            let doc = lgo::gearstats::read_stats_file(&canonical).unwrap();
            println!("level={:?} measured={:?}", doc.level, doc.measured.as_ref().map(|m| (m.max_morale, m.max_power, m.active_effects, m.equipped.len())));
        }
    }
}
