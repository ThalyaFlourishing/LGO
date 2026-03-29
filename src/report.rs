//! Terminal report formatter.
//!
//! Produces a human-readable summary of the optimizer result, including:
//!   - The recommended item for each slot
//!   - The total value of each goal stat across the full gear set
//!   - Whether each minimum was met
//!   - Any warnings (truncated pools, missing items, etc.)
//!   - A clear INFEASIBLE banner when no combination meets all minima

use std::collections::HashMap;

use crate::gear::{GearSet, Slot};
use crate::optimizer::OptimizeResult;
use crate::stat::{Stat, StatGoal};

// ?? Column widths ?????????????????????????????????????????????????????????????

const COL_SLOT:  usize = 14;
const COL_ITEM:  usize = 48;
const COL_STAT:  usize = 22;
const COL_VALUE: usize = 10;
const COL_MIN:   usize = 10;
const COL_MET:   usize = 5;

// ?? Public entry point ????????????????????????????????????????????????????????

/// Print the full optimizer report to stdout.
pub fn print_report(
    result: &OptimizeResult,
    goals: &[StatGoal],
    character: &str,
    input_file: &str,
) {
    print_header(character, input_file);

    if !result.warnings.is_empty() {
        print_warnings(&result.warnings);
    }

    print_gear_table(&result.gear_set);
    print_stat_summary(&result.gear_set, goals, &result.failed_minima);

    if result.feasible {
        println!();
        println!("  ?  All stat minima met.");
    } else {
        print_infeasible_banner(&result.failed_minima);
    }

    println!();
}

// ?? Sections ??????????????????????????????????????????????????????????????????

fn print_header(character: &str, input_file: &str) {
    let divider = "?".repeat(COL_SLOT + COL_ITEM + 3);
    println!();
    println!("  LGO — Gear Optimizer");
    println!("  Character : {}", character);
    println!("  Input     : {}", input_file);
    println!("  {}", divider);
}

fn print_warnings(warnings: &[String]) {
    println!();
    println!("  WARNINGS:");
    for w in warnings {
        println!("    ?  {}", w);
    }
    println!();
}

fn print_gear_table(gear_set: &GearSet) {
    let divider = "?".repeat(COL_SLOT + COL_ITEM + 3);

    println!();
    println!("  {:<COL_SLOT$}  {}", "Slot", "Recommended Item", COL_SLOT = COL_SLOT);
    println!("  {}", divider);

    // Print slots in a fixed, readable order.
    for &slot in Slot::ALL {
        let slot_label = slot_label(slot);
        let item_name = gear_set.items.get(&slot)
            .map(|i| i.name.as_str())
            .unwrap_or("—");

        // Truncate long item names with ellipsis.
        let item_display = truncate(item_name, COL_ITEM);
        println!("  {:<COL_SLOT$}  {}", slot_label, item_display, COL_SLOT = COL_SLOT);
    }

    println!("  {}", divider);
}

fn print_stat_summary(
    gear_set: &GearSet,
    goals: &[StatGoal],
    failed_minima: &[(Stat, i64, i64)],
) {
    if goals.is_empty() {
        return;
    }

    let failed_stats: std::collections::HashSet<Stat> =
        failed_minima.iter().map(|(s, _, _)| *s).collect();

    let divider = "?".repeat(COL_STAT + COL_VALUE + COL_MIN + COL_MET + 6);

    println!();
    println!(
        "  {:<COL_STAT$}  {:>COL_VALUE$}  {:>COL_MIN$}  {}",
        "Stat", "Total", "Minimum", "Met?",
        COL_STAT = COL_STAT, COL_VALUE = COL_VALUE, COL_MIN = COL_MIN,
    );
    println!("  {}", divider);

    for goal in goals {
        let total   = gear_set.total(&goal.stat);
        let minimum = goal.minimum;
        let met     = total >= minimum;
        let met_str = if minimum == 0 {
            "  — ".to_string()
        } else if met {
            "  ? ".to_string()
        } else {
            "  ? ".to_string()
        };

        let flag = if failed_stats.contains(&goal.stat) { " ?" } else { "" };

        println!(
            "  {:<COL_STAT$}  {:>COL_VALUE$}  {:>COL_MIN$}  {}{}",
            format!("{}", goal.stat),
            format_number(total),
            if minimum > 0 { format_number(minimum) } else { "—".to_string() },
            met_str,
            flag,
            COL_STAT = COL_STAT, COL_VALUE = COL_VALUE, COL_MIN = COL_MIN,
        );
    }

    println!("  {}", divider);
}

fn print_infeasible_banner(failed_minima: &[(Stat, i64, i64)]) {
    println!();
    println!("  ????????????????????????????????????????????????");
    println!("  ?  INFEASIBLE — not all stat minima can be met ?");
    println!("  ????????????????????????????????????????????????");
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
    println!("  The result shown is the best available given the");
    println!("  priority order of your stat list.");
}

// ?? Formatting helpers ????????????????????????????????????????????????????????

/// Slot label for the gear table — uses the display name from Slot::ALL order.
fn slot_label(slot: Slot) -> &'static str {
    match slot {
        Slot::Head      => "Head",
        Slot::Chest     => "Chest",
        Slot::Legs      => "Legs",
        Slot::Hands     => "Hands",
        Slot::Feet      => "Feet",
        Slot::Shoulders => "Shoulders",
        Slot::Back      => "Back",
        Slot::Wrist1    => "Wrist (1)",
        Slot::Wrist2    => "Wrist (2)",
        Slot::Neck      => "Neck",
        Slot::Finger1   => "Finger (1)",
        Slot::Finger2   => "Finger (2)",
        Slot::Ear1      => "Ear (1)",
        Slot::Ear2      => "Ear (2)",
        Slot::Pocket    => "Pocket",
        Slot::OffHand   => "Off-hand",
        Slot::Ranged    => "Ranged",
    }
}

/// Format an i64 with thousands separators: 1234567 ? "1,234,567".
fn format_number(n: i64) -> String {
    let s = n.abs().to_string();
    let with_commas = s.as_bytes()
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
        assert_eq!(format_number(0),        "0");
        assert_eq!(format_number(999),      "999");
        assert_eq!(format_number(1000),     "1,000");
        assert_eq!(format_number(1234567),  "1,234,567");
        assert_eq!(format_number(-6140),    "-6,140");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Short", 20),  "Short");
        assert_eq!(truncate("Umbari Robe of Beasts of the Nameless Deeps and More", 20),
                   "Umbari Robe of Beas…");
    }
}