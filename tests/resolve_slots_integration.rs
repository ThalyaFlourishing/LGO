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
    let input_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("TestData/lgo_stats_Thalya_20260525_215012.toml");
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

    assert_eq!(resolved, 64, "resolved count drift");
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

    assert_eq!(outcomes.len(), 66, "total item count drift");
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
        Ok(64),
        "resolved canonical-slot subset must parse"
    );
}

#[test]
fn bookmarklet_warning_comments_survive_resolution() {
    let (out, _) = setup();
    assert!(
        out.contains("# WARNING: all stats unknown"),
        "WARNING comments must survive resolution"
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
