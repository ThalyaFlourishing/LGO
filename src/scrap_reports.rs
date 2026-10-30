//! Reading saved `lgo optimize --to-file` text reports for `lgo scrap-gear`.
//!
//! `scrap-gear` no longer re-runs the optimizer. Instead it reads the plain
//! text optimize reports the user already wrote into the character's
//! `<CharacterName>_Reports` folder, counts which item names each report's
//! "Recommended Gear:" table uses, and compares those counts against the
//! owned gear. Everything here is pure and unit-testable: folder listing,
//! scrap-report filename recognition, `--reports-folder` validation, and the
//! report-text parser.

use crate::gear::Slot;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One successfully parsed optimize report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedReport {
    /// The `Build Name:` header value; `None` when the header showed `-`.
    pub build_name: Option<String>,
    /// Occurrences of each real item name in the gear table.
    pub item_counts: HashMap<String, usize>,
}

/// Placeholder names that never count as real items. This is the single home
/// of the rule, shared by `scrap-gear` parsing and anything else that must
/// ignore empty-slot placeholders.
pub fn is_empty_placeholder(name: &str) -> bool {
    name.starts_with("[empty") || name == "NO ITEMS" || name == "(2-handed item)"
}

/// Whether `file_name` is one of scrap-gear's own report files
/// (`lgo_<anything>_ScrapGear_Report_<anything>.txt`, case-insensitive).
/// These are skipped silently when scanning a reports folder.
pub fn is_scrap_gear_report_filename(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    let Some(middle) = lower
        .strip_prefix("lgo_")
        .and_then(|rest| rest.strip_suffix(".txt"))
    else {
        return false;
    };
    middle.contains("_scrapgear_report_")
}

/// Validate a `--reports-folder` value: a folder name (or relative sub-path)
/// inside `<CharacterName>_Reports`. Anything absolute, containing `..`, or
/// otherwise able to escape the reports folder is rejected with a clear
/// message. The value itself is preserved as typed.
pub fn validate_reports_folder_name(name: &str) -> Result<(), String> {
    let reject = |why: &str| {
        Err(format!(
            "Invalid --reports-folder value '{}': {}. It must name a folder inside the character's _Reports folder.",
            name, why
        ))
    };
    if name.trim().is_empty() {
        return reject("it is empty");
    }
    if Path::new(name).is_absolute() || name.starts_with('/') || name.starts_with('\\') {
        return reject("it is an absolute path");
    }
    if name.contains(':') {
        return reject("it contains a drive or ':'");
    }
    for component in name.split(['/', '\\']) {
        if component.is_empty() {
            return reject("it contains an empty path component");
        }
        if component == "." || component == ".." {
            return reject("it contains a '.' or '..' component");
        }
    }
    Ok(())
}

/// List candidate report files: every `*.txt` directly in `folder` (no
/// recursion), excluding scrap-gear's own reports, sorted by file name.
pub fn list_candidate_report_files(folder: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(folder)
        .map_err(|e| format!("cannot read reports folder {}: {}", folder.display(), e))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "cannot read entry in reports folder {}: {}",
                folder.display(),
                e
            )
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let is_txt = Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"));
        if !is_txt || is_scrap_gear_report_filename(name) {
            continue;
        }
        files.push(path);
    }
    files.sort();
    Ok(files)
}

/// Parse one optimize report's text. Returns `Err` with a reason when the
/// text does not contain a parsable "Recommended Gear:" section; headers
/// (`Build Name`) are read best-effort from the lines before it.
pub fn parse_optimize_report(text: &str) -> Result<ParsedReport, String> {
    let mut build_name = None;
    let mut lines = text.lines();

    // Header lines appear before the gear table; nothing after the table's
    // closing divider (e.g. the Virtues section) is ever read.
    let mut found_heading = false;
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed == "Recommended Gear:" {
            found_heading = true;
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Build Name:") {
            let value = value.trim();
            if !value.is_empty() && value != "-" {
                build_name = Some(value.to_string());
            }
        }
    }
    if !found_heading {
        return Err("no \"Recommended Gear:\" section found".to_string());
    }

    // Skip the column-header line(s) up to the divider that opens the rows.
    let mut found_open_divider = false;
    for line in lines.by_ref() {
        if is_divider_line(line) {
            found_open_divider = true;
            break;
        }
    }
    if !found_open_divider {
        return Err("\"Recommended Gear:\" section has no divider line".to_string());
    }

    let mut item_counts: HashMap<String, usize> = HashMap::new();
    let mut found_closing_divider = false;
    for line in lines {
        if is_divider_line(line) {
            found_closing_divider = true;
            break;
        }
        let row = line.trim();
        if row.is_empty() {
            continue;
        }
        let Some(item_name) = split_gear_row(row) else {
            return Err(format!(
                "gear table row does not start with a known slot name: \"{}\"",
                row
            ));
        };
        if item_name.is_empty() || is_empty_placeholder(item_name) {
            continue;
        }
        *item_counts.entry(item_name.to_string()).or_insert(0) += 1;
    }
    if !found_closing_divider {
        return Err("\"Recommended Gear:\" table has no closing divider".to_string());
    }

    Ok(ParsedReport {
        build_name,
        item_counts,
    })
}

fn is_divider_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && trimmed.chars().all(|ch| ch == '─')
}

/// Split a trimmed gear-table row into slot + item name by matching the
/// longest canonical slot display name at the start of the row. Slot names
/// may contain spaces or hyphens (`Class Item`, `Main-hand`), so the row is
/// never split on whitespace.
fn split_gear_row(row: &str) -> Option<&str> {
    let mut best: Option<&'static str> = None;
    for slot in Slot::all() {
        let display = slot.display_name();
        let matches = row
            .strip_prefix(display)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace));
        if matches && best.is_none_or(|current| display.len() > current.len()) {
            best = Some(display);
        }
    }
    best.map(|display| row[display.len()..].trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report(build_name: &str, rows: &[(&str, &str)]) -> String {
        let divider = "─".repeat(65);
        let mut text = String::new();
        text.push_str("  LGO — Thalya's Gear Optimizer\n");
        text.push_str("  Character : Thalya (Lore-master)\n");
        text.push_str(&format!("  Build Name: {}\n", build_name));
        text.push_str("  Gear List : lgo_Thalya_gearReady\n");
        text.push_str("  Run time  : 2026-08-31 08:00:00 +00:00\n");
        text.push_str(&format!("  {}\n", divider));
        text.push_str("\n  ✓  All stat minima met.\n\n");
        text.push_str(&format!("  {}\n", divider));
        text.push_str("  Recommended Gear:\n");
        text.push_str("          Slot  Recommended Item\n");
        text.push_str(&format!("  {}\n", divider));
        for (slot, item) in rows {
            text.push_str(&format!("  {:>14}  {}\n", slot, item));
        }
        text.push_str(&format!("  {}\n", divider));
        text.push_str("\n  Virtues (chosen by the optimizer):\n");
        text.push_str("       Charity\n");
        text
    }

    #[test]
    fn parses_build_name_and_item_counts() {
        let text = sample_report(
            "healer",
            &[
                ("Head", "Simple Helm"),
                ("Chest", "Simple Robe"),
                ("Off-hand", "(2-handed item)"),
                ("Ranged", "[empty Ranged]"),
                ("Class Item", "NO ITEMS"),
            ],
        );
        let parsed = parse_optimize_report(&text).expect("report must parse");
        assert_eq!(parsed.build_name.as_deref(), Some("healer"));
        assert_eq!(parsed.item_counts.len(), 2);
        assert_eq!(parsed.item_counts.get("Simple Helm"), Some(&1));
        assert_eq!(parsed.item_counts.get("Simple Robe"), Some(&1));
    }

    #[test]
    fn dash_build_name_parses_as_none() {
        let text = sample_report("-", &[("Head", "Simple Helm")]);
        let parsed = parse_optimize_report(&text).expect("report must parse");
        assert_eq!(parsed.build_name, None);
    }

    #[test]
    fn paired_family_produces_count_of_two() {
        let text = sample_report(
            "-",
            &[
                ("Wrist", "Twin Bracelet"),
                ("Wrist", "Twin Bracelet"),
                ("Finger", "Lone Ring"),
            ],
        );
        let parsed = parse_optimize_report(&text).expect("report must parse");
        assert_eq!(parsed.item_counts.get("Twin Bracelet"), Some(&2));
        assert_eq!(parsed.item_counts.get("Lone Ring"), Some(&1));
    }

    #[test]
    fn item_names_with_spaces_survive_slot_prefix_matching() {
        let text = sample_report(
            "-",
            &[
                ("Class Item", "Tome of the Wandering Moon"),
                (
                    "Main-hand",
                    "Staff of Many Long Words and Even Longer Names",
                ),
            ],
        );
        let parsed = parse_optimize_report(&text).expect("report must parse");
        assert_eq!(
            parsed.item_counts.get("Tome of the Wandering Moon"),
            Some(&1)
        );
        assert_eq!(
            parsed
                .item_counts
                .get("Staff of Many Long Words and Even Longer Names"),
            Some(&1)
        );
    }

    #[test]
    fn missing_gear_section_is_an_error() {
        let err =
            parse_optimize_report("just some notes\nno gear here\n").expect_err("must not parse");
        assert!(err.contains("Recommended Gear"));
    }

    #[test]
    fn virtues_section_after_closing_divider_is_never_read() {
        let mut text = sample_report("-", &[("Head", "Simple Helm")]);
        text.push_str("       Honesty Is An Item Name Trap\n");
        let parsed = parse_optimize_report(&text).expect("report must parse");
        assert_eq!(parsed.item_counts.len(), 1);
    }

    #[test]
    fn scrap_gear_report_filenames_are_recognized_case_insensitively() {
        assert!(is_scrap_gear_report_filename(
            "lgo_Thalya_ScrapGear_Report_20260831_080000.txt"
        ));
        assert!(is_scrap_gear_report_filename(
            "LGO_THALYA_SCRAPGEAR_REPORT_X.TXT"
        ));
        assert!(!is_scrap_gear_report_filename("lgo_GearReport_000.txt"));
        assert!(!is_scrap_gear_report_filename("lgo_Thalya_healer.txt"));
        assert!(!is_scrap_gear_report_filename(
            "lgo_Thalya_ScrapGear_Report_1.html"
        ));
        assert!(!is_scrap_gear_report_filename("notes.txt"));
    }

    #[test]
    fn reports_folder_names_are_validated() {
        assert!(validate_reports_folder_name("archive").is_ok());
        assert!(validate_reports_folder_name("My Reports").is_ok());
        assert!(validate_reports_folder_name("nested\\inner").is_ok());
        assert!(validate_reports_folder_name("").is_err());
        assert!(validate_reports_folder_name("   ").is_err());
        assert!(validate_reports_folder_name("..").is_err());
        assert!(validate_reports_folder_name("..\\escape").is_err());
        assert!(validate_reports_folder_name("a/../b").is_err());
        assert!(validate_reports_folder_name("/absolute").is_err());
        assert!(validate_reports_folder_name("\\absolute").is_err());
        assert!(validate_reports_folder_name("C:\\absolute").is_err());
        assert!(validate_reports_folder_name("trailing\\").is_err());
    }

    #[test]
    fn empty_placeholders_cover_all_three_forms() {
        assert!(is_empty_placeholder("[empty Head]"));
        assert!(is_empty_placeholder("NO ITEMS"));
        assert!(is_empty_placeholder("(2-handed item)"));
        assert!(!is_empty_placeholder("Simple Helm"));
    }
}
