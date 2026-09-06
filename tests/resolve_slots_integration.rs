use lgo::slot_resolver::{
    ResolutionOutcome, AUTO_PICKED_COMMENT_PREFIX, UNRESOLVED_COMMENT_PREFIX,
};
use lgo::stat::{BASE_STATS, TRACKED_STATS};
use lgo::virtues::VIRTUE_FIELD_KEYS;
use std::path::{Path, PathBuf};

fn data_json_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/lgo_items.json")
}

fn setup() -> (String, Vec<ResolutionOutcome>) {
    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let input_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml");
    let src = std::fs::read_to_string(&input_path).expect("fixture must read");

    let base_stats: std::collections::HashMap<lgo::stat::Stat, i64> = [
        (lgo::stat::Stat::Might, 5300),
        (lgo::stat::Stat::Agility, 2650),
        (lgo::stat::Stat::Vitality, 10200),
        (lgo::stat::Stat::Will, 7950),
        (lgo::stat::Stat::Fate, 4000),
    ]
    .into_iter()
    .collect();

    lgo::slot_resolver::resolve_toml_str_with_metadata(
        &src,
        &db,
        Some("Thalya"),
        "Lore-master",
        &base_stats,
    )
    .expect("must resolve")
}

// HELPERS:
fn resolved_item_slots(out: &str) -> Vec<String> {
    let doc: toml_edit::DocumentMut = out.parse().expect("resolved output parses as TOML");
    doc.get("item")
        .and_then(|v| v.as_array_of_tables())
        .expect("resolved output has [[item]]")
        .iter()
        .map(|table| {
            table
                .get("slot")
                .and_then(|v| v.as_str())
                .expect("[[item]] has slot")
                .to_string()
        })
        .collect()
}

fn item_table_count(src: &str) -> usize {
    let doc: toml_edit::DocumentMut = src.parse().expect("output parses as TOML");
    doc.get("item")
        .and_then(|v| v.as_array_of_tables())
        .expect("output has [[item]]")
        .len()
}

fn with_item_tables<R>(src: &str, f: impl FnOnce(&toml_edit::ArrayOfTables) -> R) -> R {
    let doc: toml_edit::DocumentMut = src.parse().expect("output parses as TOML");
    let tables = doc
        .get("item")
        .and_then(|v| v.as_array_of_tables())
        .expect("output has [[item]]");
    f(tables)
}

/// Canonical per-item key layout: the 16 tracked stats then the 5 Base stats.
fn canonical_stat_keys() -> Vec<&'static str> {
    TRACKED_STATS
        .iter()
        .chain(BASE_STATS.iter())
        .map(|(_, key)| *key)
        .collect()
}

fn has_assignment_line(src: &str, key: &str, expected: i64) -> bool {
    let expected = expected.to_string();
    src.lines().any(|line| {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix(key) else {
            return false;
        };
        let Some(rest) = rest.trim_start_matches(' ').strip_prefix('=') else {
            return false;
        };
        rest.trim_start_matches(' ') == expected
    })
}

fn has_string_assignment_line(src: &str, key: &str, expected: &str) -> bool {
    let expected = format!("\"{}\"", expected);
    src.lines().any(|line| {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix(key) else {
            return false;
        };
        let Some(rest) = rest.trim_start_matches(' ').strip_prefix('=') else {
            return false;
        };
        rest.trim_start_matches(' ') == expected
    })
}

fn assert_header_note_immediately_after(src: &str, header: &str, note: &str) {
    let expected = format!("{header}\n{note}\n");
    assert!(
        src.contains(&expected),
        "expected note immediately after {header}:\n{src}"
    );
    assert_eq!(
        src.matches(note).count(),
        1,
        "expected note exactly once for {header}:\n{src}"
    );
}

fn assert_stat_assignments_align_to_column_20(src: &str) {
    let keys = canonical_stat_keys();
    let mut saw_stat_line = false;
    for line in src.lines() {
        let trimmed = line.trim_start();
        let Some((key, _)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim_end_matches([' ', '\t']);
        if keys.contains(&key) {
            saw_stat_line = true;
            assert_eq!(
                trimmed.find('=').map(|idx| idx + 1),
                Some(20),
                "stat assignment must align '=' to column 20:\n{}",
                line
            );
        }
    }
    assert!(
        saw_stat_line,
        "test input should contain stat assignment lines"
    );
}

fn item_block_for_name(src: &str, name: &str) -> String {
    let name_marker = format!("\"{}\"", name);
    let name_pos = src
        .find(&name_marker)
        .unwrap_or_else(|| panic!("item {name} must be present:\n{src}"));
    let start = src[..name_pos]
        .rfind("[[item]]")
        .expect("item block starts before name");
    let rest = &src[start..];
    let end = rest[1..]
        .find("[[item]]")
        .map(|idx| idx + 1)
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

fn assert_outcome_comment_is_before_stat_block(block: &str, comment: &str) {
    let comment_pos = block
        .find(comment)
        .unwrap_or_else(|| panic!("expected comment {comment:?} in block:\n{block}"));
    let stat_positions: Vec<usize> = canonical_stat_keys()
        .iter()
        .filter_map(|key| block.find(&format!("{key} ")))
        .collect();
    let first_stat_pos = stat_positions
        .iter()
        .copied()
        .min()
        .expect("item block contains canonical stats");
    assert!(
        comment_pos < first_stat_pos,
        "outcome comment must be in the header before the stat block:\n{block}"
    );
    assert!(
        stat_positions.iter().all(|pos| comment_pos < *pos),
        "outcome comment must not remain attached to a later stat line:\n{block}"
    );
}

fn with_item_table_named<R>(src: &str, name: &str, f: impl FnOnce(&toml_edit::Table) -> R) -> R {
    let doc: toml_edit::DocumentMut = src.parse().expect("output parses as TOML");
    let tables = doc
        .get("item")
        .and_then(|v| v.as_array_of_tables())
        .expect("output has [[item]]");
    let table = tables
        .iter()
        .find(|table| table.get("name").and_then(|value| value.as_str()) == Some(name))
        .unwrap_or_else(|| panic!("item {name} must be present"));
    f(table)
}

fn decor_repr(decor: Option<&toml_edit::RawString>) -> &str {
    decor.and_then(|raw| raw.as_str()).unwrap_or("")
}

fn assert_outcome_comment_is_attached_to_header_not_stats(
    src: &str,
    name: &str,
    header_key: &str,
    comment: &str,
) {
    with_item_table_named(src, name, |table| {
        let (header_key_decor, header_value) = table
            .get_key_value(header_key)
            .unwrap_or_else(|| panic!("{header_key} must be present in item {name}"));
        let header_prefix = decor_repr(header_key_decor.leaf_decor().prefix());
        let header_suffix = header_value
            .as_value()
            .map(|value| decor_repr(value.decor().suffix()))
            .unwrap_or("");
        assert!(
            header_prefix.contains(comment) || header_suffix.contains(comment),
            "outcome comment must be attached to {header_key} header decor:\nprefix={header_prefix:?}\nsuffix={header_suffix:?}"
        );

        for key in canonical_stat_keys() {
            let Some((stat_key, stat_item)) = table.get_key_value(key) else {
                continue;
            };
            let key_prefix = decor_repr(stat_key.leaf_decor().prefix());
            assert!(
                !key_prefix.contains(UNRESOLVED_COMMENT_PREFIX)
                    && !key_prefix.contains(AUTO_PICKED_COMMENT_PREFIX),
                "outcome comment must not be attached to {key} key prefix:\n{key_prefix:?}"
            );

            let value_suffix = stat_item
                .as_value()
                .map(|value| decor_repr(value.decor().suffix()))
                .unwrap_or("");
            assert!(
                !value_suffix.contains(UNRESOLVED_COMMENT_PREFIX)
                    && !value_suffix.contains(AUTO_PICKED_COMMENT_PREFIX),
                "outcome comment must not be attached to {key} value suffix:\n{value_suffix:?}"
            );
        }
    });
}

fn current_plugindata_fixture_path() -> PathBuf {
    let test_data = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData");
    let mut matches: Vec<PathBuf> = std::fs::read_dir(&test_data)
        .unwrap_or_else(|e| panic!("TestData directory must be readable: {}", e))
        .map(|entry| {
            entry
                .expect("failed to read TestData directory entry")
                .path()
        })
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| {
                    name.starts_with("lgo_Thalya_gearNames_") && name.ends_with(".plugindata")
                })
                .unwrap_or(false)
        })
        .collect();
    matches.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    assert!(
        !matches.is_empty(),
        "expected at least one Thalya plugindata fixture in {}",
        test_data.display()
    );
    matches.pop().unwrap()
}

// TESTS:
#[test]
fn resolves_full_bookmarklet_output_matches_known_summary() {
    let (out, outcomes) = setup();
    let slots = resolved_item_slots(&out);

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
    let emitted_unknown = slots
        .iter()
        .filter(|slot| slot.as_str() == "Unknown")
        .count();
    let emitted_non_unknown = slots.len() - emitted_unknown;

    assert_eq!(
        outcomes.len(),
        slots.len(),
        "each emitted [[item]] must have one resolution outcome"
    );
    assert_eq!(
        resolved, emitted_non_unknown,
        "resolved outcomes must match emitted canonical-slot items"
    );
    assert_eq!(
        unknown_names.len(),
        emitted_unknown,
        "unknown outcomes must match emitted Unknown-slot items"
    );
}

#[test]
fn resolved_output_round_trips_through_gearstats_reader_and_skips_unknown_slots() {
    let (out, _) = setup();
    let tmp = std::env::temp_dir().join(format!(
        "lgo_resolve_test_{}_{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::write(&tmp, &out).expect("write temp file");
    let parsed_len = lgo::gearstats::read_stats_file(&tmp).map(|d| d.items.len());
    let _ = std::fs::remove_file(&tmp);

    let expected_non_unknown = resolved_item_slots(&out)
        .iter()
        .filter(|slot| slot.as_str() != "Unknown")
        .count();

    assert_eq!(
        parsed_len,
        Ok(expected_non_unknown),
        "resolved canonical-slot subset must parse"
    );
}

#[test]
fn bookmarklet_warning_comments_survive_resolution() {
    let (out, _) = setup();
    assert!(
        out.contains(UNRESOLVED_COMMENT_PREFIX),
        "UNRESOLVED comments must survive resolution"
    );
}

#[test]
fn divider_comments_appear_in_canonical_family_order() {
    let (out, outcomes) = setup();
    const MIN_DIVIDERS_FOR_ORDERING_TEST: usize = 2;
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
        "# --- Off-hand ---",
        "# --- Ranged ---",
        "# --- Class Item ---",
        "# --- Unknown (not in items DB) ---",
    ];

    let mut positions = Vec::with_capacity(expected.len());
    for divider in expected {
        if let Some(pos) = out.find(divider) {
            positions.push(pos);
        }
    }

    let has_unknown_outcomes = outcomes
        .iter()
        .any(|o| matches!(o, ResolutionOutcome::Unknown { .. }));
    assert_eq!(
        out.contains("# --- Unknown (not in items DB) ---"),
        has_unknown_outcomes,
        "Unknown divider should appear exactly when unresolved items exist"
    );
    assert!(
        !positions.is_empty(),
        "resolved output should contain at least one slot-family divider"
    );
    assert!(
        positions.len() >= MIN_DIVIDERS_FOR_ORDERING_TEST,
        "real fixture should contain multiple dividers to verify ordering"
    );
    for divider in out
        .lines()
        .filter(|line| line.starts_with("# --- ") && line.ends_with(" ---"))
    {
        assert!(
            expected.contains(&divider),
            "unexpected divider '{}'\n{}",
            divider,
            out
        );
    }

    for pair in positions.windows(2) {
        assert!(
            pair[0] < pair[1],
            "divider order drifted; expected canonical increasing positions"
        );
    }
}

#[test]
fn resolved_output_has_essence_totals_for_every_item_with_all_tracked_stats() {
    let (out, _) = setup();
    let keys = canonical_stat_keys();

    with_item_tables(&out, |tables| {
        for item in tables.iter() {
            for key in &keys {
                assert!(item.contains_key(key), "base item missing {}", key);
            }
            let essence = item
                .get("EssenceTotals")
                .and_then(|essence_item| essence_item.as_table())
                .expect("every item has EssenceTotals");
            for key in &keys {
                assert_eq!(
                    essence.get(key).and_then(|value| value.as_integer()),
                    Some(0),
                    "new EssenceTotals should zero {}",
                    key
                );
            }
        }
    });
}

#[test]
fn resolved_output_keeps_base_and_essence_blocks_attached_in_canonical_order() {
    let (out, _) = setup();
    let first_item = out.find("[[item]]").expect("item emitted");
    let first_essence = out
        .find("[item.EssenceTotals]")
        .expect("EssenceTotals emitted");
    let base = &out[first_item..first_essence];
    let essence = &out[first_essence..];
    let keys = canonical_stat_keys();

    assert!(
        !out.contains("\n\n[item.EssenceTotals]"),
        "base and EssenceTotals blocks must not have a blank line between them"
    );

    let base_positions: Vec<usize> = keys
        .iter()
        .map(|key| {
            base.find(key)
                .unwrap_or_else(|| panic!("base stat {} missing", key))
        })
        .collect();
    assert!(base_positions.windows(2).all(|pair| pair[0] < pair[1]));

    let essence_positions: Vec<usize> = keys
        .iter()
        .map(|key| {
            essence
                .find(key)
                .unwrap_or_else(|| panic!("essence stat {} missing", key))
        })
        .collect();
    assert!(essence_positions.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn resolved_output_aligns_all_stat_assignments_to_column_20() {
    let (out, _) = setup();
    assert_stat_assignments_align_to_column_20(&out);
}

#[test]
fn first_divider_present_in_real_fixture_output() {
    let (out, _) = setup();
    let first_divider_pos = out.find("# --- Head ---").expect("first divider present");
    assert!(
        first_divider_pos > 0,
        "resolved output should contain content before the first slot-family divider"
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
        "name ? multiple slot collisions found (first-match-wins is unsafe): {:#?}",
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
        Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml"),
        &bookmarklet,
    )
    .expect("copy fixture");
    assert!(!canonical.exists());

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run must succeed");

    assert!(canonical.exists(), "canonical file must be written");
    assert!(!report.previous_existed);
    assert!(!report.no_new_export);
    assert!(report.outcome.preserved.is_empty());
    assert!(
        report.outcome.overwritten.is_empty(),
        "first run should not overwrite existing entries"
    );
    assert!(
        report.outcome.removed.is_empty(),
        "first run should not remove entries"
    );

    let canonical_text = std::fs::read_to_string(&canonical).expect("read canonical");
    let emitted_items = item_table_count(&canonical_text);

    assert!(
        emitted_items > 0,
        "fixture should produce at least one canonical item"
    );
    assert_eq!(
        report.outcome.added.len(),
        emitted_items,
        "first-run added count must match emitted canonical [[item]] count"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_merge_preserves_hand_edits_on_re_export() {
    let dir = make_temp_dir("hand_edits");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml");
    std::fs::copy(&fixture, &bookmarklet).expect("copy fixture");
    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
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
        Some(&dir),
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
fn file_level_repeat_run_preserves_parseable_canonical_output() {
    let dir = make_temp_dir("idempotent");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml"),
        &bookmarklet,
    )
    .expect("copy fixture");

    // Copy the latest plugindata fixture in under the test character's name so
    // the canonical output reflects exported metadata rather than zero-default
    // Base stats. Matching is case-insensitive on the character segment, but
    // the `lgo_TestChar_` prefix itself must be present.
    std::fs::copy(
        current_plugindata_fixture_path(),
        dir.join(format!(
            "lgo_{}_gearNames_20260820_000000.plugindata",
            character
        )),
    )
    .expect("copy plugindata fixture");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");

    let first_report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run");
    let after_first = std::fs::read_to_string(&canonical).expect("read canonical after first run");
    let parsed_first =
        lgo::gearstats::read_stats_file(&canonical).expect("first canonical file must parse");
    let first_item_count = parsed_first.items.len();

    let second_report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("second run");
    let after_second =
        std::fs::read_to_string(&canonical).expect("read canonical after second run");
    let parsed_second =
        lgo::gearstats::read_stats_file(&canonical).expect("second canonical file must parse");
    let second_item_count = parsed_second.items.len();

    assert!(
        canonical.exists(),
        "canonical file must still exist after repeat run"
    );
    assert!(
        after_first.contains("# gearReady.toml updated:"),
        "first run must write generated timestamp header"
    );
    assert!(
        after_second.contains("# gearReady.toml updated:"),
        "second run must preserve generated timestamp header"
    );
    assert_eq!(
        after_second
            .lines()
            .filter(|line| line.contains("# gearReady.toml updated:"))
            .count(),
        1,
        "canonical file must contain exactly one generated timestamp header"
    );
    assert!(
        after_second.contains("character"),
        "repeat run canonical output must retain character metadata"
    );
    assert!(
        after_second.contains("class"),
        "repeat run canonical output must retain class metadata"
    );
    assert!(
        after_second.contains("[InnateStats]"),
        "repeat run canonical output must retain InnateStats"
    );
    assert_eq!(
        first_item_count, second_item_count,
        "repeat run must preserve parsed canonical item count"
    );
    assert_eq!(
        parsed_first.character, parsed_second.character,
        "repeat run must preserve canonical character metadata"
    );
    assert_eq!(
        parsed_first.class, parsed_second.class,
        "repeat run must preserve canonical class metadata"
    );
    assert_eq!(
        parsed_first.innate_stats, parsed_second.innate_stats,
        "repeat run must preserve canonical innate stats"
    );
    assert_eq!(
        first_report.outcome.unknown_slot, second_report.outcome.unknown_slot,
        "repeat run must report the same unknown-slot items"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_rerun_preserves_essence_totals_and_normalizes_partial_canonical_file() {
    let dir = make_temp_dir("essence_rerun");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let db = lgo::slot_resolver::ItemsDb::from_json_str(
        r#"{
            "Test Helm": {
                "name": "Test Helm",
                "slot": "Head",
                "stats": {}
            }

        }"#,
        Path::new("<test-fixture>"),
    )
    .expect("synthetic DB must parse");

    std::fs::write(
        &canonical,
        "\
[[item]]
slot = \"Head\"
name = \"Test Helm\"
CriticalRating = 100
# user note: essence totals maintained by hand
[item.EssenceTotals]
CriticalRating = 300
Finesse = 5
",
    )
    .expect("write existing canonical");
    std::fs::write(
        &bookmarklet,
        "\
[[item]]
slot = \"Unknown\"
name = \"Test Helm\"
CriticalRating = 200
",
    )
    .expect("write bookmarklet export");

    let report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("rerun must succeed");
    let after_first = std::fs::read_to_string(&canonical).expect("read canonical");

    assert_eq!(report.outcome.preserved, vec!["Test Helm"]);
    with_item_tables(&after_first, |tables| {
        let item = tables.iter().next().expect("one item");
        let essence = item
            .get("EssenceTotals")
            .and_then(|value| value.as_table())
            .expect("EssenceTotals table");

        assert_eq!(
            item.get("CriticalRating")
                .and_then(|value| value.as_integer()),
            Some(100),
            "default rerun must preserve existing user-maintained base stats"
        );
        assert_eq!(
            essence
                .get("CriticalRating")
                .and_then(|value| value.as_integer()),
            Some(300),
            "nonzero essence total must survive rerun"
        );
        assert_eq!(
            essence.get("Finesse").and_then(|value| value.as_integer()),
            Some(5),
            "partial EssenceTotals data must survive rerun"
        );
        for key in canonical_stat_keys() {
            assert!(item.contains_key(key), "base item missing {}", key);
            assert!(essence.contains_key(key), "EssenceTotals missing {}", key);
        }
    });
    // Keep the user's EssenceTotals note visually attached to the child table;
    // a blank gap makes the note look detached from the hand-maintained data.
    assert!(
        has_assignment_line(&after_first, "Fate", 0)
            && after_first
                .contains("# user note: essence totals maintained by hand\n[item.EssenceTotals]"),
        "EssenceTotals comment should remain attached without a blank gap:\n{}",
        after_first
    );

    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("second rerun must succeed");
    let after_second = std::fs::read_to_string(&canonical).expect("read canonical");
    assert_eq!(
        after_first, after_second,
        "canonical file with existing EssenceTotals must be idempotent after rerun"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_unresolved_comment_stays_in_item_header_across_repeated_resolution() {
    let dir = make_temp_dir("unresolved_header");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let db = lgo::slot_resolver::ItemsDb::from_json_str(
        r#"{
            "Test Helm": {
                "name": "Test Helm",
                "slot": "Head"
            },
            "Test Greatsword": {
                "name": "Test Greatsword",
                "slot": "Main-hand",
                "two_handed": true
            }
        }"#,
        Path::new("<test-fixture>"),
    )
    .expect("synthetic DB must parse");

    let bookmarklet_text = "\
[[item]]
slot = \"Unknown\"
name = \"Test Greatsword\"
Morale = 0
Power = 0
Armor = 0
# UNRESOLVED: multiple wiki variants exist — you should hand-edit stats
CriticalRating = 7

[[item]]
slot = \"Unknown\"
name = \"Test Helm\"
Morale = 0
Power = 0
Armor = 0
# AUTO-PICKED highest-item-level variant: Item:Test_Helm_(Item_Level_999)
CriticalRating = 3
";

    std::fs::write(&bookmarklet, bookmarklet_text).expect("write bookmarklet first run");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run must succeed");
    let after_first = std::fs::read_to_string(&canonical).expect("read canonical first run");
    let first_block = item_block_for_name(&after_first, "Test Greatsword");
    assert_outcome_comment_is_before_stat_block(&first_block, UNRESOLVED_COMMENT_PREFIX);
    assert_outcome_comment_is_attached_to_header_not_stats(
        &after_first,
        "Test Greatsword",
        "two_handed",
        UNRESOLVED_COMMENT_PREFIX,
    );
    let first_helm_block = item_block_for_name(&after_first, "Test Helm");
    assert_outcome_comment_is_before_stat_block(&first_helm_block, AUTO_PICKED_COMMENT_PREFIX);
    assert_outcome_comment_is_attached_to_header_not_stats(
        &after_first,
        "Test Helm",
        "name",
        AUTO_PICKED_COMMENT_PREFIX,
    );

    std::fs::write(&bookmarklet, bookmarklet_text).expect("write bookmarklet second run");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("second run must succeed");
    let after_second = std::fs::read_to_string(&canonical).expect("read canonical second run");
    let second_block = item_block_for_name(&after_second, "Test Greatsword");
    assert_outcome_comment_is_before_stat_block(&second_block, UNRESOLVED_COMMENT_PREFIX);
    assert_outcome_comment_is_attached_to_header_not_stats(
        &after_second,
        "Test Greatsword",
        "two_handed",
        UNRESOLVED_COMMENT_PREFIX,
    );
    assert_eq!(
        second_block.matches(UNRESOLVED_COMMENT_PREFIX).count(),
        1,
        "unresolved comment should not duplicate across reruns:\n{second_block}"
    );
    let second_helm_block = item_block_for_name(&after_second, "Test Helm");
    assert_outcome_comment_is_before_stat_block(&second_helm_block, AUTO_PICKED_COMMENT_PREFIX);
    assert_outcome_comment_is_attached_to_header_not_stats(
        &after_second,
        "Test Helm",
        "name",
        AUTO_PICKED_COMMENT_PREFIX,
    );
    assert_eq!(
        second_helm_block
            .matches(AUTO_PICKED_COMMENT_PREFIX)
            .count(),
        1,
        "auto-picked comment should not duplicate across reruns:\n{second_helm_block}"
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
        Some(&dir),
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
        Some(&dir),
        "TestChar",
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect_err("must error when nothing to read");
    let msg = err.to_string();
    assert!(
        msg.contains("No lgo_TestChar_gearStats.toml")
            && msg.contains("lgo_TestChar_gearReady.toml"),
        "error must mention both expected filenames: got {}",
        msg
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// =============================================================================
// Case-insensitive file matching integration tests
// =============================================================================

/// Bookmarklet file saved with all-lowercase character name (`lgo_thalya_gearStats.toml`)
/// must be found when the resolver is invoked with the mixed-case name `"Thalya"`.
/// The canonical output file must be written at the path derived from the
/// supplied character name (write-follows-read: no existing canonical, so use
/// `canonical_gear_path(dir, "Thalya")`).
#[test]
fn resolve_stats_file_finds_lowercase_bookmarklet_for_mixed_case_query() {
    let dir = make_temp_dir("case_insensitive_bookmarklet");
    let character = "Thalya";

    // Write the fixture as `lgo_thalya_gearStats.toml` (all lowercase).
    let bookmarklet_lowercase = dir.join("lgo_thalya_gearStats.toml");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml"),
        &bookmarklet_lowercase,
    )
    .expect("copy fixture");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
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
/// `lgo_thalya_gearReady.toml`), the resolver must still find and reuse that
/// existing file when invoked with `"Thalya"`.
#[test]
fn resolve_stats_file_reuses_existing_canonical_case_insensitively_on_windows() {
    let dir = make_temp_dir("write_follows_read_windows");
    let character = "Thalya";

    // Create the canonical file first using the normal mixed-case path.
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml");
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);

    std::fs::copy(&fixture, &bookmarklet).expect("copy fixture for first run");
    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run must succeed");

    // Rename the canonical file to a lowercase spelling.
    let canonical_exact = lgo::slot_resolver::canonical_gear_path(&dir, character);
    let canonical_lowercase = dir.join("lgo_thalya_gearReady.toml");
    std::fs::rename(&canonical_exact, &canonical_lowercase).expect("rename to lowercase canonical");

    assert!(
        canonical_lowercase.exists(),
        "lowercase canonical must exist after rename"
    );

    // Second pass: re-run with a fresh bookmarklet copy.
    std::fs::copy(&fixture, &bookmarklet).expect("copy fixture for second run");
    let report = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
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

// =============================================================================
// Metadata (character / class) integration tests
// =============================================================================

/// Build a minimal bookmarklet TOML string with character and class at the top.
fn make_bookmarklet_with_meta(character: &str, class: &str) -> String {
    format!(
        "\
# LGO gear stats file — generated by bookmarklet
character          = \"{character}\"
class              = \"{class}\"

[[item]]
slot               = \"Head\"
name               = \"Test Helm\"
"
    )
}

#[test]
fn file_level_merge_first_run_canonical_contains_metadata() {
    let dir = make_temp_dir("meta_first_run");
    let character = "MetaChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    std::fs::write(
        &bookmarklet,
        make_bookmarklet_with_meta("MetaChar", "Lore-master"),
    )
    .expect("write bookmarklet");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run must succeed");

    let canonical_text = std::fs::read_to_string(&canonical).expect("read canonical");
    assert!(
        canonical_text.contains("MetaChar"),
        "character must be in canonical file after first run:\n{}",
        canonical_text
    );
    assert!(
        canonical_text.contains("Lore-master"),
        "class must be in canonical file after first run:\n{}",
        canonical_text
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_merge_repeat_merge_keeps_metadata() {
    let dir = make_temp_dir("meta_repeat");
    let character = "MetaChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let bm_content = make_bookmarklet_with_meta("MetaChar", "Guardian");
    std::fs::write(&bookmarklet, &bm_content).expect("write bookmarklet first run");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run");

    // Re-write bookmarklet (same content) and run again.
    std::fs::write(&bookmarklet, &bm_content).expect("write bookmarklet second run");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("second run");

    let canonical_text = std::fs::read_to_string(&canonical).expect("read canonical");
    assert!(
        canonical_text.contains("MetaChar"),
        "character must survive repeat merge:\n{}",
        canonical_text
    );
    assert!(
        canonical_text.contains("Guardian"),
        "class must survive repeat merge:\n{}",
        canonical_text
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_merge_hand_edited_canonical_retains_metadata_on_re_export() {
    let dir = make_temp_dir("meta_hand_edit");
    let character = "MetaChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let bm_content = make_bookmarklet_with_meta("MetaChar", "Minstrel");
    std::fs::write(&bookmarklet, &bm_content).expect("write bookmarklet first run");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first run");

    // Simulate hand-edit.
    let mut text = std::fs::read_to_string(&canonical).expect("read canonical");
    text = text.replacen("[[item]]", "# hand-edit: keep this\n[[item]]", 1);
    std::fs::write(&canonical, &text).expect("write hand-edited canonical");

    // Re-export.
    std::fs::write(&bookmarklet, &bm_content).expect("write bookmarklet second run");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("second run");

    let after = std::fs::read_to_string(&canonical).expect("read canonical after second run");
    assert!(
        after.contains("MetaChar"),
        "character must survive hand-edit + re-export:\n{}",
        after
    );
    assert!(
        after.contains("Minstrel"),
        "class must survive hand-edit + re-export:\n{}",
        after
    );
    assert!(
        after.contains("# hand-edit: keep this"),
        "hand-edited comment must survive re-export:\n{}",
        after
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// =============================================================================
// Plugin export exclusion regression tests
// =============================================================================

/// Verify that the current plugindata fixture no longer contains the equipped
/// craft tool or bridle.  src/lgo.lua skips slot 19 (CraftItem) and slot 21
/// (Bridle) so players do not need to unequip those items before exporting.
#[test]
fn current_plugindata_excludes_craft_tool_and_bridle() {
    let path = current_plugindata_fixture_path();
    let raw = std::fs::read_to_string(&path).expect("current plugindata fixture must be readable");

    // These are the specific items Thalya had equipped in the old export.
    // They must stay absent as long as the plugin skips those slots.
    assert!(
        !raw.contains("Extraordinary Elf Prospector's Pickaxe"),
        "craft tool (slot 19) must be absent from the current plugindata export"
    );
    assert!(
        !raw.contains("Scholar's Light Bridle"),
        "bridle (slot 21) must be absent from the current plugindata export"
    );
}

/// Regression test for the [InnateStats] position-drift bug.
///
/// toml_edit renders top-level tables by their internal `position` index,
/// not map insertion order. Before the fix, `push_group` renumbered every
/// [[item]] table to 0..n while [InnateStats] kept a stale position (from
/// wherever it physically sat in the previous gearReady.toml), so the block
/// drifted to a different mid-file location on every run — each run's
/// output feeding the next run's input.
///
/// This test runs resolve_stats_file three times and asserts after every
/// run that [InnateStats] sits between the `class = ...` line and the
/// first `# --- Head ---` divider. Three runs matter: the first exercises
/// first-run creation, the second exercises the merge path against a
/// correct file, and the third catches any single-run-delayed drift.
#[test]
fn file_level_innate_stats_stays_between_class_and_first_divider_across_reruns() {
    let dir = make_temp_dir("innate_position");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml");

    // Copy the plugindata fixture in under the test character's name so the
    // canonical output reflects the exported raw Base stats.
    std::fs::copy(
        current_plugindata_fixture_path(),
        dir.join(format!(
            "lgo_{}_gearNames_20260820_000000.plugindata",
            character
        )),
    )
    .expect("copy plugindata fixture");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");

    for run in 1..=3 {
        // Re-copy the bookmarklet output each iteration: resolve_stats_file
        // consumes it, and each run must exercise a fresh export → merge.
        std::fs::copy(&fixture, &bookmarklet).expect("copy gearStats fixture");

        let _ = lgo::slot_resolver::resolve_stats_file(
            &dir,
            Some(&dir),
            character,
            &db,
            lgo::slot_resolver::ForceMode::NoForce,
        )
        .unwrap_or_else(|e| panic!("run {} must succeed: {}", run, e));

        let out = std::fs::read_to_string(&canonical).expect("read canonical");

        let class_pos = out
            .find("\nclass")
            .unwrap_or_else(|| panic!("run {}: class line missing:\n{}", run, out));
        let innate_pos = out
            .find("[InnateStats]")
            .unwrap_or_else(|| panic!("run {}: [InnateStats] missing:\n{}", run, out));
        let divider_pos = out
            .find("# --- Head ---")
            .unwrap_or_else(|| panic!("run {}: Head divider missing:\n{}", run, out));

        assert_eq!(
            out.matches("[InnateStats]").count(),
            1,
            "run {}: exactly one [InnateStats] block expected:\n{}",
            run,
            out
        );
        assert!(
            class_pos < innate_pos,
            "run {}: [InnateStats] must come after the class line (drift regression):\n{}",
            run,
            out
        );
        assert!(
            innate_pos < divider_pos,
            "run {}: [InnateStats] must come before the first slot divider (drift regression):\n{}",
            run,
            out
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// The pass-through design: [InnateStats] holds exactly the character's five
/// *raw* Base stats from the plugindata export — no derived tracked stats.
/// Assert content, canonical key order, and position (between the `class`
/// line and the first `# --- Head ---` divider) across three consecutive
/// runs: fresh creation, merge against a correct file, and delayed drift.
#[test]
fn file_level_innate_stats_holds_raw_base_stats_across_reruns() {
    let dir = make_temp_dir("innate_raw");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml");

    std::fs::copy(
        current_plugindata_fixture_path(),
        dir.join(format!(
            "lgo_{}_gearNames_20260820_000000.plugindata",
            character
        )),
    )
    .expect("copy plugindata fixture");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");

    for run in 1..=3 {
        std::fs::copy(&fixture, &bookmarklet).expect("copy gearStats fixture");

        let _ = lgo::slot_resolver::resolve_stats_file(
            &dir,
            Some(&dir),
            character,
            &db,
            lgo::slot_resolver::ForceMode::NoForce,
        )
        .unwrap_or_else(|e| panic!("run {} must succeed: {}", run, e));

        let out = std::fs::read_to_string(&canonical).expect("read canonical");
        let doc: toml_edit::DocumentMut = out.parse().expect("canonical output parses");
        let innate = doc
            .get("InnateStats")
            .and_then(|item| item.as_table())
            .unwrap_or_else(|| panic!("run {}: [InnateStats] missing:\n{}", run, out));

        // Exactly the five raw Base stats, in canonical order, with the
        // plugindata fixture's values — no derived tracked stats.
        let keys: Vec<&str> = innate.iter().map(|(key, _)| key).collect();
        assert_eq!(
            keys,
            vec!["Might", "Agility", "Vitality", "Will", "Fate"],
            "run {}: [InnateStats] must hold exactly the five raw Base stats:\n{}",
            run,
            out
        );
        for (key, expected) in [
            ("Might", 5300),
            ("Agility", 2650),
            ("Vitality", 10200),
            ("Will", 7950),
            ("Fate", 4000),
        ] {
            assert_eq!(
                innate.get(key).and_then(|value| value.as_integer()),
                Some(expected),
                "run {}: [InnateStats] {} must pass through raw:\n{}",
                run,
                key,
                out
            );
        }

        let class_pos = out
            .find("\nclass")
            .unwrap_or_else(|| panic!("run {}: class line missing:\n{}", run, out));
        let innate_pos = out
            .find("[InnateStats]")
            .unwrap_or_else(|| panic!("run {}: [InnateStats] missing:\n{}", run, out));
        let divider_pos = out
            .find("# --- Head ---")
            .unwrap_or_else(|| panic!("run {}: Head divider missing:\n{}", run, out));
        assert!(
            class_pos < innate_pos && innate_pos < divider_pos,
            "run {}: [InnateStats] must sit between class and the first divider:\n{}",
            run,
            out
        );
        assert_header_note_immediately_after(
            &out,
            "[InnateStats]",
            "# Extracted by in-game plugin; do not edit.",
        );
        assert_header_note_immediately_after(
            &out,
            "[Virtues]",
            "# Not extracted, you must add these yourself.",
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_zero_base_stat_export_still_writes_innate_stats_and_virtues() {
    let dir = make_temp_dir("zero_base_export");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);
    let plugindata = dir.join(format!(
        "lgo_{}_gearNames_20260820_000000.plugindata",
        character
    ));

    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml"),
        &bookmarklet,
    )
    .expect("copy gearStats fixture");
    std::fs::write(
        &plugindata,
        format!(
            "return {{\n    [\"character\"] = \"{character}\",\n    [\"class\"] = \"Lore-master\",\n    [\"baseStats\"] = {{\n        [\"GetBaseMight\"] = 0.000000,\n        [\"GetBaseAgility\"] = 0.000000,\n        [\"GetBaseVitality\"] = 0.000000,\n        [\"GetBaseWill\"] = 0.000000,\n        [\"GetBaseFate\"] = 0.000000,\n    }},\n}}\n"
        ),
    )
    .expect("write zero-base plugindata fixture");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("resolve with zero-base export");

    let out = std::fs::read_to_string(&canonical).expect("read canonical");
    let doc: toml_edit::DocumentMut = out.parse().expect("canonical output parses");
    let innate = doc
        .get("InnateStats")
        .and_then(|item| item.as_table())
        .expect("InnateStats table exists");
    let virtues = doc
        .get("Virtues")
        .and_then(|item| item.as_table())
        .expect("Virtues table exists");

    let innate_keys: Vec<&str> = innate.iter().map(|(key, _)| key).collect();
    assert_eq!(
        innate_keys,
        vec!["Might", "Agility", "Vitality", "Will", "Fate"]
    );
    for key in ["Might", "Agility", "Vitality", "Will", "Fate"] {
        assert_eq!(
            innate.get(key).and_then(|item| item.as_integer()),
            Some(0),
            "zero export must still write {key} = 0:\n{out}"
        );
    }
    for key in VIRTUE_FIELD_KEYS {
        assert_eq!(
            virtues.get(key).and_then(|item| item.as_str()),
            Some(""),
            "zero export must still write default {key}:\n{out}"
        );
    }

    let innate_pos = out.find("[InnateStats]").expect("InnateStats block exists");
    let virtues_pos = out.find("[Virtues]").expect("Virtues block exists");
    let divider_pos = out.find("# --- Head ---").expect("Head divider exists");
    assert!(
        innate_pos < virtues_pos && virtues_pos < divider_pos,
        "zero export must still place Virtues after InnateStats and before items:\n{}",
        out
    );
    assert_header_note_immediately_after(
        &out,
        "[InnateStats]",
        "# Extracted by in-game plugin; do not edit.",
    );
    assert_header_note_immediately_after(
        &out,
        "[Virtues]",
        "# Not extracted, you must add these yourself.",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_virtues_block_stays_between_innate_stats_and_first_divider_across_reruns() {
    let dir = make_temp_dir("virtues_position");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml");

    std::fs::copy(
        current_plugindata_fixture_path(),
        dir.join(format!(
            "lgo_{}_gearNames_20260820_000000.plugindata",
            character
        )),
    )
    .expect("copy plugindata fixture");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");

    for run in 1..=3 {
        std::fs::copy(&fixture, &bookmarklet).expect("copy gearStats fixture");

        let _ = lgo::slot_resolver::resolve_stats_file(
            &dir,
            Some(&dir),
            character,
            &db,
            lgo::slot_resolver::ForceMode::NoForce,
        )
        .unwrap_or_else(|e| panic!("run {} must succeed: {}", run, e));

        let out = std::fs::read_to_string(&canonical).expect("read canonical");
        let innate_pos = out
            .find("[InnateStats]")
            .unwrap_or_else(|| panic!("run {}: [InnateStats] missing:\n{}", run, out));
        let virtues_pos = out
            .find("[Virtues]")
            .unwrap_or_else(|| panic!("run {}: [Virtues] missing:\n{}", run, out));
        let divider_pos = out
            .find("# --- Head ---")
            .unwrap_or_else(|| panic!("run {}: Head divider missing:\n{}", run, out));

        assert!(
            innate_pos < virtues_pos && virtues_pos < divider_pos,
            "run {}: [Virtues] must sit between [InnateStats] and the first divider:\n{}",
            run,
            out
        );
        assert_header_note_immediately_after(
            &out,
            "[InnateStats]",
            "# Extracted by in-game plugin; do not edit.",
        );
        assert_header_note_immediately_after(
            &out,
            "[Virtues]",
            "# Not extracted, you must add these yourself.",
        );
        for key in VIRTUE_FIELD_KEYS {
            assert!(
                has_string_assignment_line(&out, key, ""),
                "run {}: missing default {} assignment:\n{}",
                run,
                key,
                out
            );
        }
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_level_virtues_values_survive_reruns_and_missing_fields_are_restored() {
    let dir = make_temp_dir("virtues_merge");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData/lgo_Thalya_gearStats.toml");

    std::fs::copy(
        current_plugindata_fixture_path(),
        dir.join(format!(
            "lgo_{}_gearNames_20260820_000000.plugindata",
            character
        )),
    )
    .expect("copy plugindata fixture");
    std::fs::copy(&fixture, &bookmarklet).expect("copy gearStats fixture");

    let db = lgo::slot_resolver::ItemsDb::load_default().expect("load DB");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first resolve must succeed");

    let first = std::fs::read_to_string(&canonical).expect("read canonical");
    let hand_edited = first.replace(
        "[Virtues]\n# Not extracted, you must add these yourself.\nVirtue1            = \"\"\nVirtue2            = \"\"\nVirtue3            = \"\"\nVirtue4            = \"\"\nVirtue5            = \"\"\n",
        "[Virtues]\n# Not extracted, you must add these yourself.\nVirtue1            = \"Wisdom\"\nVirtue3            = \" zeal \"\nVirtue5            = \"Honour\"\n",
    );
    assert_ne!(hand_edited, first, "hand-edit must apply");
    std::fs::write(&canonical, hand_edited).expect("write edited canonical");

    std::fs::copy(&fixture, &bookmarklet).expect("copy gearStats fixture");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("second resolve must succeed");

    let out = std::fs::read_to_string(&canonical).expect("read canonical");
    assert!(has_string_assignment_line(&out, "Virtue1", "Wisdom"));
    assert!(has_string_assignment_line(&out, "Virtue2", ""));
    assert!(has_string_assignment_line(&out, "Virtue3", " zeal "));
    assert!(has_string_assignment_line(&out, "Virtue4", ""));
    assert!(has_string_assignment_line(&out, "Virtue5", "Honour"));
    assert_header_note_immediately_after(
        &out,
        "[InnateStats]",
        "# Extracted by in-game plugin; do not edit.",
    );
    assert_header_note_immediately_after(
        &out,
        "[Virtues]",
        "# Not extracted, you must add these yourself.",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Strip the volatile `# gearReady.toml updated: ...` line so successive
/// runs can be compared for idempotency modulo the generated timestamp.
fn strip_generated_timestamp_line(src: &str) -> String {
    src.lines()
        .filter(|line| !line.starts_with("# gearReady.toml updated:"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Issue 1 tripwire: a hand-written comment inside `[item.EssenceTotals]`
/// must survive re-running `resolve_stats_file` and must not duplicate on
/// further reruns (idempotency modulo timestamp).
#[test]
fn file_level_essence_block_comment_survives_reruns() {
    let dir = make_temp_dir("essence_comment");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let db = lgo::slot_resolver::ItemsDb::from_json_str(
        r#"{
            "Test Helm": {
                "name": "Test Helm",
                "slot": "Head",
                "stats": {}
            }
        }"#,
        Path::new("<test-fixture>"),
    )
    .expect("synthetic DB must parse");

    let export = "\
[[item]]
slot = \"Unknown\"
name = \"Test Helm\"
CriticalRating = 200
";
    std::fs::write(&bookmarklet, export).expect("write bookmarklet export");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first resolve must succeed");

    // Hand-edit the canonical file: annotate + adjust an essence total, the
    // way `docs/User Workflow.txt` step 9 tells users to.
    let first = std::fs::read_to_string(&canonical).expect("read canonical");
    let comment = "# 2x Vivid Essence of Critical Rating";
    let edited = first.replacen(
        "CriticalRating     = 0",
        &format!("{}\nCriticalRating     = 850", comment),
        1,
    );
    assert_ne!(edited, first, "hand-edit must apply");
    // The base block's CriticalRating is 200, so the single `= 0` line
    // replaced above is the essence one.
    std::fs::write(&canonical, edited).expect("write edited canonical");

    let mut previous = String::new();
    for run in 2..=3 {
        std::fs::write(&bookmarklet, export).expect("write bookmarklet export");
        let _ = lgo::slot_resolver::resolve_stats_file(
            &dir,
            Some(&dir),
            character,
            &db,
            lgo::slot_resolver::ForceMode::NoForce,
        )
        .unwrap_or_else(|e| panic!("run {} must succeed: {}", run, e));

        let out = std::fs::read_to_string(&canonical).expect("read canonical");
        assert_eq!(
            out.matches(comment).count(),
            1,
            "run {}: essence comment must survive exactly once:\n{}",
            run,
            out
        );
        assert!(
            out.contains(&format!("{}\nCriticalRating     = 850", comment)),
            "run {}: comment must stay attached to the edited essence stat:\n{}",
            run,
            out
        );
        if run > 2 {
            assert_eq!(
                strip_generated_timestamp_line(&previous),
                strip_generated_timestamp_line(&out),
                "run {}: output must be idempotent modulo timestamp",
                run
            );
        }
        previous = out;
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// Trailing-comment gap: a hand-written comment *after the last key* of an
/// `[item.EssenceTotals]` block must survive re-running `resolve_stats_file`,
/// stay attached below its essence block (not migrate across item or family
/// boundaries), and not duplicate on further reruns (idempotency modulo
/// timestamp).
#[test]
fn file_level_essence_trailing_comment_survives_reruns() {
    let dir = make_temp_dir("essence_trailing_comment");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let db = lgo::slot_resolver::ItemsDb::from_json_str(
        r#"{
            "Test Helm": {
                "name": "Test Helm",
                "slot": "Head"
            },
            "Test Chestpiece": {
                "name": "Test Chestpiece",
                "slot": "Chest"
            },
            "Test Sword": {
                "name": "Test Sword",
                "slot": "Main-hand"
            }
        }"#,
        Path::new("<test-fixture>"),
    )
    .expect("synthetic DB must parse");

    let two_item_export = "\
[[item]]
slot = \"Unknown\"
name = \"Test Helm\"
CriticalRating = 200

[[item]]
slot = \"Unknown\"
name = \"Test Sword\"
Armor = 40
";
    std::fs::write(&bookmarklet, two_item_export).expect("write bookmarklet export");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first resolve must succeed");

    // Hand-edit the canonical file: set an essence total and write a note
    // *after the last essence key* of the Head item — the line directly
    // above the Main-hand divider.
    let first = std::fs::read_to_string(&canonical).expect("read canonical");
    let comment = "# TODO: re-check after next essence swap";
    let edited = first.replacen(
        "Fate               = 0\n\n# --- Main-hand ---",
        &format!("Fate               = 77\n{comment}\n\n# --- Main-hand ---"),
        1,
    );
    assert_ne!(edited, first, "hand-edit must apply");
    std::fs::write(&canonical, edited).expect("write edited canonical");

    // Re-export with an additional Chest item so the merge regroups: the
    // new block lands between the Head and Main-hand families. The trailing
    // comment must stay below the helm's essence block instead of riding
    // the Main-hand divider's table past the new Chest block.
    let three_item_export = "\
[[item]]
slot = \"Unknown\"
name = \"Test Helm\"
CriticalRating = 200

[[item]]
slot = \"Unknown\"
name = \"Test Chestpiece\"
Armor = 30

[[item]]
slot = \"Unknown\"
name = \"Test Sword\"
Armor = 40
";

    let mut previous = String::new();
    for run in 2..=3 {
        std::fs::write(&bookmarklet, three_item_export).expect("write bookmarklet export");
        let _ = lgo::slot_resolver::resolve_stats_file(
            &dir,
            Some(&dir),
            character,
            &db,
            lgo::slot_resolver::ForceMode::NoForce,
        )
        .unwrap_or_else(|e| panic!("run {} must succeed: {}", run, e));

        let out = std::fs::read_to_string(&canonical).expect("read canonical");
        assert_eq!(
            out.matches(comment).count(),
            1,
            "run {}: trailing essence comment must survive exactly once:\n{}",
            run,
            out
        );
        assert!(
            out.contains(&format!("Fate               = 77\n{comment}")),
            "run {}: comment must stay attached below the helm's essence block:\n{}",
            run,
            out
        );
        for divider in ["# --- Head ---", "# --- Chest ---", "# --- Main-hand ---"] {
            assert_eq!(
                out.matches(divider).count(),
                1,
                "run {}: divider {} must appear exactly once:\n{}",
                run,
                divider,
                out
            );
        }
        if run > 2 {
            assert_eq!(
                strip_generated_timestamp_line(&previous),
                strip_generated_timestamp_line(&out),
                "run {}: output must be idempotent modulo timestamp",
                run
            );
        }
        previous = out;
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// Issue 6 tripwire: a hand-corrected slot on an item the DB does not know
/// (bookmarklet leaves it in the Unknown section) must survive re-export,
/// move the item under the corrected slot's divider, and keep a hand-added
/// `two_handed` flag — idempotently.
#[test]
fn file_level_hand_corrected_slot_survives_re_export_across_reruns() {
    let dir = make_temp_dir("hand_slot");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    // "Renamed Legendary X" is deliberately absent from the DB.
    let db = lgo::slot_resolver::ItemsDb::from_json_str(
        r#"{
            "Test Helm": {
                "name": "Test Helm",
                "slot": "Head",
                "stats": {}
            }
        }"#,
        Path::new("<test-fixture>"),
    )
    .expect("synthetic DB must parse");

    let export = "\
[[item]]
slot = \"Unknown\"
name = \"Test Helm\"
Armor = 50

[[item]]
slot = \"Unknown\"
name = \"Renamed Legendary X\"
CriticalRating = 123
";
    std::fs::write(&bookmarklet, export).expect("write bookmarklet export");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first resolve must succeed");

    let first = std::fs::read_to_string(&canonical).expect("read canonical");
    assert!(
        first.contains("# --- Unknown"),
        "run 1: unresolved item must land in the Unknown section:\n{}",
        first
    );

    // Hand-correct the legendary the way the user workflow prescribes: fix
    // the slot and record that it is a two-hander. Test Helm resolved to
    // "Head", so the only remaining Unknown slot line is the legendary's.
    let edited = first.replacen("slot = \"Unknown\"", "slot = \"Main-hand\"", 1);
    assert_ne!(edited, first, "slot hand-edit must apply");
    let edited = edited.replacen(
        "name = \"Renamed Legendary X\"\n",
        "name = \"Renamed Legendary X\"\ntwo_handed = true\n",
        1,
    );
    assert!(
        edited.contains("two_handed = true"),
        "two_handed hand-edit must apply"
    );
    std::fs::write(&canonical, edited).expect("write edited canonical");

    let mut previous = String::new();
    for run in 2..=3 {
        std::fs::write(&bookmarklet, export).expect("write bookmarklet export");
        let _ = lgo::slot_resolver::resolve_stats_file(
            &dir,
            Some(&dir),
            character,
            &db,
            lgo::slot_resolver::ForceMode::NoForce,
        )
        .unwrap_or_else(|e| panic!("run {} must succeed: {}", run, e));

        let out = std::fs::read_to_string(&canonical).expect("read canonical");
        let block = item_block_for_name(&out, "Renamed Legendary X");
        assert!(
            block.contains("slot = \"Main-hand\""),
            "run {}: hand-corrected slot must survive re-export:\n{}",
            run,
            out
        );
        assert!(
            !out.contains("slot = \"Unknown\""),
            "run {}: no item may fall back to the Unknown slot:\n{}",
            run,
            out
        );
        assert!(
            !out.contains("# --- Unknown"),
            "run {}: Unknown section must disappear once the slot is fixed:\n{}",
            run,
            out
        );
        let divider_pos = out
            .find("# --- Main-hand ---")
            .unwrap_or_else(|| panic!("run {}: Main-hand divider missing:\n{}", run, out));
        let item_pos = out
            .find("name = \"Renamed Legendary X\"")
            .unwrap_or_else(|| panic!("run {}: legendary item missing:\n{}", run, out));
        assert!(
            divider_pos < item_pos,
            "run {}: corrected item must sit under the Main-hand divider:\n{}",
            run,
            out
        );
        with_item_table_named(&out, "Renamed Legendary X", |table| {
            assert_eq!(
                table.get("two_handed").and_then(|value| value.as_bool()),
                Some(true),
                "run {}: hand-added two_handed flag must survive:\n{}",
                run,
                out
            );
        });
        if run > 2 {
            assert_eq!(
                strip_generated_timestamp_line(&previous),
                strip_generated_timestamp_line(&out),
                "run {}: output must be idempotent modulo timestamp",
                run
            );
        }
        previous = out;
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// Issue 6 tripwire: hand-edited base-stat and essence-total values must
/// survive re-running against the same bookmarklet export, idempotently.
#[test]
fn file_level_hand_edited_stat_values_survive_re_export_across_reruns() {
    let dir = make_temp_dir("hand_stats");
    let character = "TestChar";
    let bookmarklet = lgo::slot_resolver::bookmarklet_stats_path(&dir, character);
    let canonical = lgo::slot_resolver::canonical_gear_path(&dir, character);

    let db = lgo::slot_resolver::ItemsDb::from_json_str(
        r#"{
            "Test Helm": {
                "name": "Test Helm",
                "slot": "Head",
                "stats": {}
            }
        }"#,
        Path::new("<test-fixture>"),
    )
    .expect("synthetic DB must parse");

    let export = "\
[[item]]
slot = \"Unknown\"
name = \"Test Helm\"
CriticalRating = 200
";
    std::fs::write(&bookmarklet, export).expect("write bookmarklet export");
    let _ = lgo::slot_resolver::resolve_stats_file(
        &dir,
        Some(&dir),
        character,
        &db,
        lgo::slot_resolver::ForceMode::NoForce,
    )
    .expect("first resolve must succeed");

    // Hand-edit one base stat and one essence total. With a single item the
    // base CriticalRating is the `= 200` line; the essence one is the sole
    // CriticalRating `= 0` line.
    let first = std::fs::read_to_string(&canonical).expect("read canonical");
    let edited = first.replacen("CriticalRating     = 200", "CriticalRating     = 4321", 1);
    assert_ne!(edited, first, "base stat hand-edit must apply");
    let with_essence = edited.replacen("CriticalRating     = 0", "CriticalRating     = 777", 1);
    assert_ne!(with_essence, edited, "essence total hand-edit must apply");
    std::fs::write(&canonical, with_essence).expect("write edited canonical");

    let mut previous = String::new();
    for run in 2..=3 {
        std::fs::write(&bookmarklet, export).expect("write bookmarklet export");
        let _ = lgo::slot_resolver::resolve_stats_file(
            &dir,
            Some(&dir),
            character,
            &db,
            lgo::slot_resolver::ForceMode::NoForce,
        )
        .unwrap_or_else(|e| panic!("run {} must succeed: {}", run, e));

        let out = std::fs::read_to_string(&canonical).expect("read canonical");
        with_item_table_named(&out, "Test Helm", |table| {
            assert_eq!(
                table
                    .get("CriticalRating")
                    .and_then(|value| value.as_integer()),
                Some(4321),
                "run {}: hand-edited base stat must survive:\n{}",
                run,
                out
            );
            let essence = table
                .get("EssenceTotals")
                .and_then(|value| value.as_table())
                .expect("EssenceTotals table");
            assert_eq!(
                essence
                    .get("CriticalRating")
                    .and_then(|value| value.as_integer()),
                Some(777),
                "run {}: hand-edited essence total must survive:\n{}",
                run,
                out
            );
        });
        if run > 2 {
            assert_eq!(
                strip_generated_timestamp_line(&previous),
                strip_generated_timestamp_line(&out),
                "run {}: output must be idempotent modulo timestamp",
                run
            );
        }
        previous = out;
    }

    let _ = std::fs::remove_dir_all(&dir);
}
