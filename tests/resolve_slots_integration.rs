use lgo::slot_resolver::ResolutionOutcome;
use std::path::{Path, PathBuf};

fn data_json_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/lgo_items.json")
}

fn setup() -> (String, Vec<ResolutionOutcome>) {
    let json_path = data_json_path();
    assert!(
        json_path.exists(),
        "data/lgo_items.json missing — re-clone with LFS / restore from git"
    );

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("data/lgo_items.json must load");
    let input_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_stats.toml");
    let src = std::fs::read_to_string(&input_path).expect("fixture must read");
    lgo::slot_resolver::resolve_toml_str(&src, &db).expect("must resolve")
}

#[test]
fn resolves_full_bookmarklet_output_matches_known_summary() {
    let (_, outcomes) = setup();

    let resolved = outcomes
        .iter()
        .filter(|o| matches!(o, ResolutionOutcome::Resolved { .. }))
        .count();
    let unknown_names: Vec<&str> = outcomes
        .iter()
        .filter_map(|o| match o {
            ResolutionOutcome::Unknown { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(resolved, 68, "resolved count drift");
    assert_eq!(unknown_names.len(), 2, "unknown count drift");

    let mut sorted_unknowns = unknown_names.clone();
    sorted_unknowns.sort();
    assert_eq!(
        sorted_unknowns,
        vec![
            "Extraordinary Elf Prospector's Pickaxe",
            "Scholar's Light Bridle",
        ],
        "unknown item set drift"
    );

    assert_eq!(outcomes.len(), 70, "total item count drift");
}

#[test]
fn resolved_output_round_trips_through_gearstats_reader() {
    let (out, _) = setup();
    let mut doc: toml_edit::DocumentMut = out.parse().expect("resolved output parses as TOML");
    let items = doc
        .get_mut("item")
        .and_then(|v| v.as_array_of_tables_mut())
        .expect("resolved output has [[item]]");

    let mut filtered = toml_edit::ArrayOfTables::new();
    let mut next_pos: usize = 0;
    for mut table in items.iter().cloned() {
        let is_unknown = table
            .get("slot")
            .and_then(|v| v.as_str())
            .map(|s| s == "Unknown")
            .unwrap_or(false);
        if !is_unknown {
            table.set_position(next_pos);
            next_pos += 1;
            filtered.push(table);
        }
    }
    *items = filtered;

    let tmp = std::env::temp_dir().join(format!(
        "lgo_resolve_test_{}_{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::write(&tmp, doc.to_string()).expect("write temp file");
    let parsed_len = lgo::gearstats::read_stats_file(&tmp).map(|v| v.len());
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(
        parsed_len,
        Ok(68),
        "resolved canonical-slot subset must parse"
    );
}

#[test]
fn bookmarklet_warning_comments_survive_resolution() {
    let (out, _) = setup();
    assert!(
        out.contains("# UNRESOLVED:"),
        "UNRESOLVED comments must survive resolution"
    );
}

#[test]
fn divider_comments_appear_in_canonical_family_order() {
    let (out, _) = setup();
    let expected = [
        "# --- Head ---",
        "# --- Chest ---",
        "# --- Legs ---",
        "# --- Hands ---",
        "# --- Feet ---",
        "# --- Shoulders ---",
        "# --- Back ---",
        "# --- Wrist ---",
        "# --- Neck ---",
        "# --- Finger ---",
        "# --- Ear ---",
        "# --- Pocket ---",
        "# --- Main-hand ---",
        "# --- Unknown (not in items DB) ---",
    ];

    let mut positions = Vec::with_capacity(expected.len());
    for divider in expected {
        let pos = out
            .find(divider)
            .unwrap_or_else(|| panic!("missing divider '{}'\n{}", divider, out));
        positions.push(pos);
    }

    for pair in positions.windows(2) {
        assert!(
            pair[0] < pair[1],
            "divider order drifted; expected canonical increasing positions"
        );
    }
}

#[test]
fn document_header_precedes_first_divider_in_real_fixture() {
    let (out, _) = setup();
    let header_pos = out
        .find("# LGO gear stats file")
        .expect("file header preserved");
    let first_divider_pos = out.find("# --- Head ---").expect("first divider present");
    assert!(
        header_pos < first_divider_pos,
        "file header must precede first slot-family divider"
    );
}

#[test]
fn no_item_name_maps_to_multiple_slots_in_lgo_items_json() {
    let path = data_json_path();
    let raw = std::fs::read_to_string(&path).expect("data/lgo_items.json must read");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("parse JSON");
    let obj = v.as_object().expect("top-level must be object");

    use std::collections::{BTreeSet, HashMap};
    let mut name_to_slots: HashMap<String, BTreeSet<String>> = HashMap::new();
    for (_key, entry) in obj {
        let name = entry
            .get("name")
            .and_then(|n| n.as_str())
            .expect("entry has name");
        let slot = entry
            .get("slot")
            .and_then(|s| s.as_str())
            .expect("entry has slot");
        name_to_slots
            .entry(name.to_string())
            .or_default()
            .insert(slot.to_string());
    }

    let collisions: Vec<(String, Vec<String>)> = name_to_slots
        .iter()
        .filter_map(|(name, slots)| {
            if slots.len() > 1 {
                Some((name.clone(), slots.iter().cloned().collect()))
            } else {
                None
            }
        })
        .collect();
    assert!(
        collisions.is_empty(),
        "name → multiple slot collisions found (first-match-wins is unsafe): {:#?}",
        collisions.iter().take(10).collect::<Vec<_>>()
    );
}

#[test]
fn bookmarklet_typo_slot_strings_are_canonicalized_when_name_is_known() {
    let (out, outcomes) = setup();
    let earring_was_resolved = outcomes.iter().any(|o| {
        matches!(
            o,
            ResolutionOutcome::Resolved { name, to_slot, .. }
                if name == "Keen Pristine Madáshi Earring"
                    && *to_slot == lgo::gear::Slot::Ear1
        )
    });

    if earring_was_resolved {
        assert!(
            !out.contains("Ears (1)"),
            "typo slot string must be replaced with canonical form"
        );
    } else {
        println!(
            "Keen Pristine Madáshi Earring was unresolved in this data snapshot; skipping typo-canonicalization assertion."
        );
    }
}

// =============================================================================
// File-level merge integration tests
// =============================================================================

fn make_temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lgo_merge_test_{}_{}_{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn file_level_merge_first_run_creates_canonical_file() {
    let dir = make_temp_dir("first_run");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_stats.toml"),
        &bookmarklet,
    )
    .expect("copy fixture");
    assert!(!canonical.exists());

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run must succeed");

    assert!(canonical.exists(), "canonical file must be written");
    assert!(!report.previous_existed);
    assert!(!report.no_new_export);
    assert!(report.outcome.preserved.is_empty());
    assert_eq!(report.outcome.added.len(), 70);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_merge_idempotent_on_repeat() {
    let dir = make_temp_dir("idempotent");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_stats.toml"),
        &bookmarklet,
    )
    .expect("copy fixture");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run");
    let after_first = std::fs::read_to_string(&canonical).expect("read canonical");

    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("second run");
    let after_second = std::fs::read_to_string(&canonical).expect("read canonical");

    assert_eq!(
        after_first, after_second,
        "second run must produce a bit-identical canonical file"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_merge_preserves_hand_edits_on_re_export() {
    let dir = make_temp_dir("hand_edits");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_stats.toml");
    std::fs::copy(&fixture, &bookmarklet).expect("copy fixture");
    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run");

    // Simulate a hand-edit: bump every Armor value by inserting a sentinel
    // line into the canonical file. We do it by injecting a unique
    // comment that must round-trip.
    let mut canon_text = std::fs::read_to_string(&canonical).expect("read canonical");
    canon_text = canon_text.replacen("[[item]]", "# user hand-edit: keep this line\n[[item]]", 1);
    std::fs::write(&canonical, &canon_text).expect("write canonical");

    // Re-export (same fixture; "no actual changes from the new export
    // POV"). Default mode should preserve everything.
    std::fs::copy(&fixture, &bookmarklet).expect("re-copy fixture");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("second run");
    let after = std::fs::read_to_string(&canonical).expect("read canonical");

    assert!(
        after.contains("# user hand-edit: keep this line"),
        "hand-edited comment must survive re-run"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_merge_no_new_export_leaves_canonical_untouched() {
    let dir = make_temp_dir("no_new_export");
    let character = "TestChar";
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let canon_text = "# canonical placeholder\n[[item]]\nslot = \"Head\"\nname = \"X\"\n";
    std::fs::write(&canonical, canon_text).expect("write canonical");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("must succeed even with no bookmarklet output");
    assert!(report.no_new_export);
    assert!(report.bookmarklet_path.is_none());
    let after = std::fs::read_to_string(&canonical).expect("read canonical");
    assert_eq!(after, canon_text, "canonical must be untouched");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_merge_no_files_at_all_is_an_error() {
    let dir = make_temp_dir("nothing");
    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let err = lgo::slot_resolver::resolve_stats_file(
        &dir,
        "TestChar",
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect_err("must error when nothing to read");
    let msg = err.to_string();
    assert!(
        msg.contains("No lgo_TestChar_stats.toml") && msg.contains("lgo_TestChar_gear.toml"),
        "error must mention both expected filenames: got {}",
        msg
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// =============================================================================
// Case-insensitive file matching integration tests
// =============================================================================

/// Bookmarklet file saved with all-lowercase character name (`lgo_thalya_stats.toml`)
/// must be found when the resolver is invoked with the mixed-case name `"Thalya"`.
/// The canonical output file must be written at the path derived from the
/// supplied character name (write-follows-read: no existing canonical, so use
/// `canonical_gear_path(dir, "Thalya")`).
#[test]
fn resolve_stats_file_finds_lowercase_bookmarklet_for_mixed_case_query() {
    let dir = make_temp_dir("case_insensitive_bookmarklet");
    let character = "Thalya";

    // Write the fixture as `lgo_thalya_stats.toml` (all lowercase).
    let bookmarklet_lowercase = dir.join("lgo_thalya_stats.toml");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_stats.toml"),
        &bookmarklet_lowercase,
    )
    .expect("copy fixture");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("must succeed with lowercase bookmarklet");

    // Canonical file must exist and must be named using the supplied character.
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);
    assert!(canonical.exists(), "canonical file must be written");
    assert!(
        !report.previous_existed,
        "no pre-existing canonical should have been found"
    );
    assert!(!report.no_new_export);

    // The bookmarklet path in the report must be the on-disk path that was
    // actually found (lowercase), not the constructed path.
    assert_eq!(
        report.bookmarklet_path.as_deref(),
        Some(bookmarklet_lowercase.as_path()),
        "report must reference the actual on-disk bookmarklet path"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Windows is case-insensitive for ordinary filenames: if the canonical gear
/// file already exists on disk under a different casing (e.g.
/// `lgo_thalya_gear.toml`), the resolver must still find and reuse that
/// existing file when invoked with `"Thalya"`.
#[test]
fn resolve_stats_file_reuses_existing_canonical_case_insensitively_on_windows() {
    let dir = make_temp_dir("write_follows_read_windows");
    let character = "Thalya";

    // Create the canonical file first using the normal mixed-case path.
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_stats.toml");
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);

    std::fs::copy(&fixture, &bookmarklet).expect("copy fixture for first run");
    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run must succeed");

    // Rename the canonical file to a lowercase spelling.
    let canonical_exact = lgo::slot_resolver::canonical_gear_path(&dir, character);
    let canonical_lowercase = dir.join("lgo_thalya_gear.toml");
    std::fs::rename(&canonical_exact, &canonical_lowercase).expect("rename to lowercase canonical");

    assert!(
        canonical_lowercase.exists(),
        "lowercase canonical must exist after rename"
    );

    // Second pass: re-run with a fresh bookmarklet copy.
    std::fs::copy(&fixture, &bookmarklet).expect("copy fixture for second run");
    let report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("second run must succeed");

    // On Windows, the resolver should find and reuse the existing canonical file
    // regardless of case, and must report that a previous canonical existed.
    assert!(
        canonical_lowercase.exists(),
        "lowercase canonical must still exist after second run"
    );
    assert!(
        report.previous_existed,
        "resolver must have found an existing canonical file"
    );
    assert_eq!(
        report.canonical_path, canonical_lowercase,
        "report canonical_path must be the on-disk path that was found"
    );

    let _ = std::fs::remove_dir_all(&dir);
}