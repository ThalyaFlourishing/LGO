//! Terminal report formatter.
//!
//! Produces a human-readable summary of the optimizer result, including:
//!   - The recommended item for each slot
//!   - The total value of each goal stat across the full gear set
//!   - Whether each minimum was met
//!   - Any warnings (missing items, etc.)
//!   - A clear INFEASIBLE banner explaining the clamped-satisfaction result

use crate::gear::{GearSet, Slot};
use crate::optimizer::OptimizeResult;
use crate::stat::{Stat, StatGoal, BASE_STATS, TRACKED_STATS};
use std::collections::HashMap;
use std::fmt::Write as _;

// ?? Column widths ?????????????????????????????????????????????????????????????

const COL_SLOT: usize = 14;
const COL_ITEM: usize = 48;
const COL_STAT: usize = 22;
const COL_VALUE: usize = 10;
const COL_MIN: usize = 10;
const COL_MET: usize = 5;

// ?? Public entry point ????????????????????????????????????????????????????????

/// Print the full optimizer report to stdout.
pub fn print_report(
    result: &OptimizeResult,
    goals: &[StatGoal],
    character: &str,
    class: &str,
    input_file: &str,
) {
    print_header(character, class, input_file);

    if !result.warnings.is_empty() {
        print_warnings(&result.warnings);
    }

    print_gear_table(&result.gear_set);
    print_stat_summary(&result.gear_set, goals, &result.failed_minima);

    if result.feasible {
        println!();
        println!("  ✓  All stat minima met.");
    } else {
        print_infeasible_banner(&result.failed_minima);
    }

    println!();
}

/// Print the `lgo base-stats` report to stdout.
pub fn print_base_stats_report(
    character: &str,
    class: &str,
    input_file: &str,
    innate_base: &HashMap<Stat, i64>,
    derived: &HashMap<Stat, i64>,
) {
    print!(
        "{}",
        format_base_stats_report(character, class, input_file, innate_base, derived)
    );
}

/// Build the `lgo base-stats` report: the five raw innate Base stats from
/// `[InnateStats]`, then the 16 tracked-stat contributions derived from them.
/// Base stats are derivation inputs only — they are never added raw to any
/// tracked total, and the derived contributions shown here are already
/// included in `lgo optimize` totals.
pub fn format_base_stats_report(
    character: &str,
    class: &str,
    input_file: &str,
    innate_base: &HashMap<Stat, i64>,
    derived: &HashMap<Stat, i64>,
) -> String {
    let divider = "─".repeat(COL_STAT + COL_VALUE + 2);
    let mut out = String::new();
    let w = &mut out;

    writeln!(w).unwrap();
    writeln!(w, "  LGO — Innate Base Stats").unwrap();
    writeln!(w, "  Character : {} ({})", character, class).unwrap();
    writeln!(w, "  Stats file: {}", input_file).unwrap();
    writeln!(w, "  {}", divider).unwrap();

    writeln!(w).unwrap();
    writeln!(
        w,
        "  Raw Base stats from [InnateStats] (derivation inputs only):"
    )
    .unwrap();
    writeln!(w).unwrap();
    for (stat, _) in BASE_STATS {
        let value = innate_base.get(stat).copied().unwrap_or(0);
        writeln!(
            w,
            "  {:<COL_STAT$}  {:>COL_VALUE$}",
            format!("{}", stat),
            format_number(value),
            COL_STAT = COL_STAT,
            COL_VALUE = COL_VALUE,
        )
        .unwrap();
    }

    writeln!(w).unwrap();
    writeln!(
        w,
        "  Derived tracked-stat contributions (already included in optimize totals):"
    )
    .unwrap();
    writeln!(w).unwrap();
    for (stat, _) in TRACKED_STATS {
        let value = derived.get(stat).copied().unwrap_or(0);
        writeln!(
            w,
            "  {:<COL_STAT$}  {:>COL_VALUE$}",
            format!("{}", stat),
            format_number(value),
            COL_STAT = COL_STAT,
            COL_VALUE = COL_VALUE,
        )
        .unwrap();
    }
    writeln!(w, "  {}", divider).unwrap();

    out
}

fn print_header(character: &str, class: &str, input_file: &str) {
    let divider = "─".repeat(COL_SLOT + COL_ITEM + 3);
    println!();
    println!("  LGO — Thalya's Gear Optimizer");
    println!("  Character : {} ({})", character, class);
    println!("  Stats file: {}", input_file);
    println!("  {}", divider);
}

fn print_warnings(warnings: &[String]) {
    println!();
    println!("  WARNINGS:");
    for w in warnings {
        println!("    ⚠  {}", w);
    }
    println!();
}

fn print_gear_table(gear_set: &GearSet) {
    let divider = "─".repeat(COL_SLOT + COL_ITEM + 3);

    println!();
    println!(
        "  {:<COL_SLOT$}  Recommended Item",
        "Slot",
        COL_SLOT = COL_SLOT
    );
    println!("  {}", divider);

    // Print slots in a fixed, readable order.
    for &slot in Slot::ALL {
        let slot_label = slot_label(slot);
        let item_name = gear_set
            .items
            .get(&slot)
            .map(|i| i.name.as_str())
            .unwrap_or("—");

        // Truncate long item names with ellipsis.
        let item_display = truncate(item_name, COL_ITEM);
        println!(
            "  {:<COL_SLOT$}  {}",
            slot_label,
            item_display,
            COL_SLOT = COL_SLOT
        );
    }

    println!("  {}", divider);
}

fn print_stat_summary(gear_set: &GearSet, goals: &[StatGoal], failed_minima: &[(Stat, i64, i64)]) {
    if goals.is_empty() {
        return;
    }

    let failed_stats: std::collections::HashSet<Stat> =
        failed_minima.iter().map(|(s, _, _)| *s).collect();

    let divider = "─".repeat(COL_STAT + COL_VALUE + COL_MIN + COL_MET + 6);

    println!();
    println!(
        "  {:<COL_STAT$}  {:>COL_VALUE$}  {:>COL_MIN$}  Met?",
        "Stat",
        "Total",
        "Minimum",
        COL_STAT = COL_STAT,
        COL_VALUE = COL_VALUE,
        COL_MIN = COL_MIN,
    );
    println!("  {}", divider);

    for goal in goals {
        let total = gear_set.total(&goal.stat);
        let minimum = goal.minimum;
        let met = total >= minimum;
        let met_str = if minimum == 0 {
            "  —  ".to_string()
        } else if met {
            "  ✓  ".to_string()
        } else {
            "  ✗  ".to_string()
        };

        let flag = if failed_stats.contains(&goal.stat) {
            " ⚠"
        } else {
            ""
        };

        println!(
            "  {:<COL_STAT$}  {:>COL_VALUE$}  {:>COL_MIN$}  {}{}",
            format!("{}", goal.stat),
            format_number(total),
            if minimum > 0 {
                format_number(minimum)
            } else {
                "—".to_string()
            },
            met_str,
            flag,
            COL_STAT = COL_STAT,
            COL_VALUE = COL_VALUE,
            COL_MIN = COL_MIN,
        );
    }

    println!("  {}", divider);
}

fn print_infeasible_banner(failed_minima: &[(Stat, i64, i64)]) {
    println!();
    println!("  ════════════════════════════════════════════════");
    println!("  ✗  INFEASIBLE — not all stat minima can be met ✗");
    println!("  ════════════════════════════════════════════════");
    println!();
    println!("  The following stats could not reach their minima");
    println!("  with any combination of the available items:");
    println!();
    for (stat, minimum, achieved) in failed_minima {
        println!(
            "    {:<COL_STAT$}  needed {:>8}  achieved {:>8}  short by {:>8}",
            format!("{}", stat),
            format_number(*minimum),
            format_number(*achieved),
            format_number(minimum - achieved),
            COL_STAT = COL_STAT,
        );
    }
    println!();
    println!("  The result shown gets your highest-priority goals as close");
    println!("  to their targets as possible; once a goal is met, extra");
    println!("  points in it are not pursued at the expense of lower-priority");
    println!("  goals still short of target.");
}

// ?? Formatting helpers ????????????????????????????????????????????????????????

/// Slot label for the gear table — uses the display name from Slot::ALL order.
fn slot_label(slot: Slot) -> &'static str {
    match slot {
        Slot::Head => "Head",
        Slot::Chest => "Chest",
        Slot::Legs => "Legs",
        Slot::Hands => "Hands",
        Slot::Feet => "Feet",
        Slot::Shoulders => "Shoulders",
        Slot::Back => "Back",
        Slot::Wrist1 | Slot::Wrist2 => "Wrist",
        Slot::Neck => "Neck",
        Slot::Finger1 | Slot::Finger2 => "Finger",
        Slot::Ear1 | Slot::Ear2 => "Ear",
        Slot::Pocket => "Pocket",
        Slot::MainHand => "Main-hand",
        Slot::OffHand => "Off-hand",
        Slot::Ranged => "Ranged",
        Slot::ClassItem => "Class Item",
    }
}

/// Format an i64 with thousands separators: 1234567 ? "1,234,567".
fn format_number(n: i64) -> String {
    let s = n.abs().to_string();
    let with_commas = s
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join(",");
    if n < 0 {
        format!("-{}", with_commas)
    } else {
        with_commas
    }
}

/// Truncate a string to `max_chars`, appending "…" if truncated.
fn truncate(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = chars[..max_chars - 1].iter().collect();
        format!("{}…", truncated)
    }
}

// ?? Tests ?????????????????????????????????????????????????????????????????????

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(-6140), "-6,140");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Short", 20), "Short");
        assert_eq!(
            truncate("Umbari Robe of Beasts of the Nameless Deeps and More", 20),
            "Umbari Robe of Beas…"
        );
    }

    #[test]
    fn base_stats_report_lists_raw_and_derived_sections() {
        let innate_base: HashMap<Stat, i64> = [
            (Stat::Might, 5300),
            (Stat::Agility, 2650),
            (Stat::Vitality, 10200),
            (Stat::Will, 7950),
            (Stat::Fate, 4000),
        ]
        .into_iter()
        .collect();
        let derived: HashMap<Stat, i64> = [
            (Stat::Morale, 45900),
            (Stat::TacticalMastery, 39750),
            (Stat::CriticalRating, 21200),
        ]
        .into_iter()
        .collect();

        let report = format_base_stats_report(
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            &innate_base,
            &derived,
        );

        assert!(report.contains("Character : Thalya (Lore-master)"));
        assert!(report.contains("Stats file: lgo_Thalya_gearReady.toml"));
        // Raw section: all five Base stats, labeled as derivation inputs.
        assert!(report.contains("derivation inputs only"));
        for (name, value) in [
            ("Might", "5,300"),
            ("Agility", "2,650"),
            ("Vitality", "10,200"),
            ("Will", "7,950"),
            ("Fate", "4,000"),
        ] {
            assert!(
                report.contains(name) && report.contains(value),
                "raw section must list {} = {}",
                name,
                value
            );
        }
        // Derived section: labeled as already included in optimize totals,
        // with all 16 tracked stats present (zeros included).
        assert!(report.contains("already included in optimize totals"));
        assert!(report.contains("45,900"));
        assert!(report.contains("39,750"));
        assert!(report.contains("21,200"));
        for (stat, _) in TRACKED_STATS {
            assert!(
                report.contains(&format!("{}", stat)),
                "derived section must list tracked stat {}",
                stat
            );
        }
    }
}
