//! Terminal and HTML report formatter.
//!
//! Produces a human-readable summary of the optimizer result, including:
//!   - The recommended item for each slot
//!   - The total value of each goal stat across the full gear set
//!   - The full projected tracked-stat table
//!   - The projected raw Base-stat pool
//!   - Whether each minimum was met
//!   - Any warnings (missing items, etc.)
//!   - A clear INFEASIBLE banner explaining the clamped-satisfaction result

use crate::gear::{GearSet, Slot};
use crate::optimizer::OptimizeResult;
use crate::stat::{Stat, StatGoal, BASE_STATS, TRACKED_STATS};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

const COL_SLOT: usize = 14;
const COL_ITEM: usize = 48;
const COL_STAT: usize = 22;
const COL_VALUE: usize = 10;
const COL_MIN: usize = 10;
const REPORT_TITLE: &str = "LGO — Thalya's Gear Optimizer";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrapUnusedItem {
    pub slot: Slot,
    pub name: String,
    pub owned_count: usize,
    pub max_used_count: usize,
}

/// Build the full text optimize report used for terminal and `.txt` output.
pub fn format_optimize_report(
    result: &OptimizeResult,
    goals: &[StatGoal],
    character: &str,
    class: &str,
    input_file: &str,
    timestamp: &str,
    projected_base_stats: &HashMap<Stat, i64>,
) -> String {
    let mut out = String::new();
    let w = &mut out;

    write_header_text(w, character, class, input_file, timestamp);
    write_feasibility_status_text(w, result.feasible);

    if !result.warnings.is_empty() {
        write_warnings_text(w, &result.warnings);
    }

    write_stat_summary_text(w, &result.gear_set, goals, &result.failed_minima);
    write_gear_table_text(w, &result.gear_set);
    write_projected_tracked_stats_text(w, &result.gear_set);
    write_projected_base_stats_text(w, projected_base_stats);

    if !result.feasible {
        write_infeasible_details_text(w, &result.failed_minima);
    }

    writeln!(w).unwrap();
    out
}

/// Add terminal-only ANSI colors to success/failure markers.
///
/// Keep this separate from `format_optimize_report` because that plain-text
/// formatter is also used for `.txt` report files. If ANSI escape codes were
/// inserted there, saved text reports would contain color-control characters.
pub fn colorize_terminal_status_markers(report: &str) -> String {
    const GREEN: &str = "\x1b[32m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const RESET: &str = "\x1b[0m";

    let mut colored = String::with_capacity(report.len());
    for ch in report.chars() {
        match ch {
            '✓' => {
                colored.push_str(GREEN);
                colored.push(ch);
                colored.push_str(RESET);
            }
            '✗' => {
                colored.push_str(RED);
                colored.push(ch);
                colored.push_str(RESET);
            }
            '⚠' => {
                colored.push_str(YELLOW);
                colored.push(ch);
                colored.push_str(RESET);
            }
            _ => colored.push(ch),
        }
    }
    colored
}

/// Build a self-contained HTML optimize report.
pub fn format_optimize_report_html(
    result: &OptimizeResult,
    goals: &[StatGoal],
    character: &str,
    class: &str,
    input_file: &str,
    timestamp: &str,
    projected_base_stats: &HashMap<Stat, i64>,
) -> String {
    let failed_stats: HashSet<Stat> = result.failed_minima.iter().map(|(s, _, _)| *s).collect();
    let mut out = String::new();
    let w = &mut out;

    writeln!(w, "<!DOCTYPE html>").unwrap();
    writeln!(w, "<html lang=\"en\">").unwrap();
    writeln!(w, "<head>").unwrap();
    writeln!(w, "  <meta charset=\"utf-8\">").unwrap();
    writeln!(w, "  <title>{}</title>", html_escape(REPORT_TITLE)).unwrap();
    writeln!(w, "  <style>").unwrap();
    writeln!(w, "    body {{ font-family: system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif; margin: 2rem; color: #222; background: #fff; }}").unwrap();
    writeln!(w, "    h1, h2 {{ margin-bottom: 0.4rem; }}").unwrap();
    writeln!(w, "    p.meta {{ margin: 0.2rem 0; }}").unwrap();
    writeln!(
        w,
        "    table {{ border-collapse: collapse; margin: 1rem 0 1.5rem; min-width: 32rem; }}"
    )
    .unwrap();
    writeln!(w, "    th, td {{ border: 1px solid #c9c9c9; padding: 0.45rem 0.6rem; text-align: left; vertical-align: top; }}").unwrap();
    writeln!(
        w,
        "    th.num, td.num {{ text-align: right; font-variant-numeric: tabular-nums; }}"
    )
    .unwrap();
    writeln!(w, "    .banner {{ font-weight: 700; padding: 0.75rem 1rem; border-radius: 0.35rem; margin: 1.25rem 0; }}").unwrap();
    writeln!(
        w,
        "    .feasible {{ background: #edf9ed; border: 1px solid #9fcd9f; }}"
    )
    .unwrap();
    writeln!(
        w,
        "    .infeasible {{ background: #fff1f1; border: 1px solid #d79b9b; }}"
    )
    .unwrap();
    writeln!(w, "    ul {{ margin-top: 0.4rem; }}").unwrap();
    writeln!(
        w,
        "    code {{ font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }}"
    )
    .unwrap();
    writeln!(w, "  </style>").unwrap();
    writeln!(w, "</head>").unwrap();
    writeln!(w, "<body>").unwrap();
    writeln!(w, "  <h1>{}</h1>", html_escape(REPORT_TITLE)).unwrap();
    writeln!(
        w,
        "  <p class=\"meta\"><strong>Character:</strong> {} ({})</p>",
        html_escape(character),
        html_escape(class),
    )
    .unwrap();
    writeln!(
        w,
        "  <p class=\"meta\"><strong>Gear List:</strong> <code>{}</code></p>",
        html_escape(input_file),
    )
    .unwrap();
    writeln!(
        w,
        "  <p class=\"meta\"><strong>Run time:</strong> {}</p>",
        html_escape(timestamp),
    )
    .unwrap();
    write_feasibility_status_html(w, result.feasible);

    if !result.warnings.is_empty() {
        writeln!(w, "  <h2>Warnings</h2>").unwrap();
        writeln!(w, "  <ul>").unwrap();
        for warning in &result.warnings {
            writeln!(w, "    <li>⚠ {}</li>", html_escape(warning)).unwrap();
        }
        writeln!(w, "  </ul>").unwrap();
    }

    if !goals.is_empty() {
        writeln!(w, "  <h2>Goal stat summary</h2>").unwrap();
        writeln!(w, "  <table>").unwrap();
        writeln!(
            w,
            "    <thead><tr><th>Stat</th><th class=\"num\">Goal</th><th class=\"num\">Total</th><th>Met?</th></tr></thead>"
        )
        .unwrap();
        writeln!(w, "    <tbody>").unwrap();
        for goal in goals {
            let total = result.gear_set.total(&goal.stat);
            let met_marker = goal_met_marker(goal.minimum, total);
            let flag = if failed_stats.contains(&goal.stat) {
                " ⚠"
            } else {
                ""
            };
            writeln!(
                w,
                "      <tr><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td>{}{}</td></tr>",
                html_escape(&goal.stat.to_string()),
                if goal.minimum > 0 {
                    format_number(goal.minimum)
                } else {
                    "—".to_string()
                },
                format_number(total),
                met_marker,
                flag,
            )
            .unwrap();
        }
        writeln!(w, "    </tbody>").unwrap();
        writeln!(w, "  </table>").unwrap();
    }

    writeln!(w, "  <h2>Recommended gear</h2>").unwrap();
    writeln!(w, "  <table>").unwrap();
    writeln!(
        w,
        "    <thead><tr><th>Slot</th><th>Recommended Item</th></tr></thead>"
    )
    .unwrap();
    writeln!(w, "    <tbody>").unwrap();
    let main_is_two_handed = result
        .gear_set
        .items
        .get(&Slot::MainHand)
        .is_some_and(|item| item.two_handed);
    for slot in Slot::all() {
        let item_display = hand_or_item_label(&result.gear_set, slot, main_is_two_handed);
        writeln!(
            w,
            "      <tr><td>{}</td><td>{}</td></tr>",
            html_escape(slot.display_name()),
            html_escape(&item_display),
        )
        .unwrap();
    }
    writeln!(w, "    </tbody>").unwrap();
    writeln!(w, "  </table>").unwrap();

    writeln!(w, "  <h2>Projected tracked stats</h2>").unwrap();
    writeln!(
        w,
        "  <p class=\"meta\">Effective totals worn (gear + essences + derived Base-stat contributions + Virtue fixed stats).</p>"
    )
    .unwrap();
    writeln!(w, "  <table>").unwrap();
    writeln!(
        w,
        "    <thead><tr><th>Stat</th><th class=\"num\">Total</th></tr></thead>"
    )
    .unwrap();
    writeln!(w, "    <tbody>").unwrap();
    for (stat, _) in TRACKED_STATS {
        writeln!(
            w,
            "      <tr><td>{}</td><td class=\"num\">{}</td></tr>",
            html_escape(&stat.to_string()),
            format_number(result.gear_set.total(stat)),
        )
        .unwrap();
    }
    writeln!(w, "    </tbody>").unwrap();
    writeln!(w, "  </table>").unwrap();

    writeln!(w, "  <h2>Projected raw Base stats</h2>").unwrap();
    writeln!(
        w,
        "  <p class=\"meta\">Combined Base-stat pool (innate + gear + essences + Virtues). These are derivation inputs only.</p>"
    )
    .unwrap();
    writeln!(w, "  <table>").unwrap();
    writeln!(
        w,
        "    <thead><tr><th>Base Stat</th><th class=\"num\">Total</th></tr></thead>"
    )
    .unwrap();
    writeln!(w, "    <tbody>").unwrap();
    for (stat, _) in BASE_STATS {
        writeln!(
            w,
            "      <tr><td>{}</td><td class=\"num\">{}</td></tr>",
            html_escape(&stat.to_string()),
            format_number(projected_base_stats.get(stat).copied().unwrap_or(0)),
        )
        .unwrap();
    }
    writeln!(w, "    </tbody>").unwrap();
    writeln!(w, "  </table>").unwrap();

    if !result.feasible {
        writeln!(
            w,
            "  <p>The following stats could not reach their minima with any combination of the available items:</p>"
        )
        .unwrap();
        writeln!(w, "  <table>").unwrap();
        writeln!(
            w,
            "    <thead><tr><th>Stat</th><th class=\"num\">Needed</th><th class=\"num\">Achieved</th><th class=\"num\">Short by</th></tr></thead>"
        )
        .unwrap();
        writeln!(w, "    <tbody>").unwrap();
        for (stat, minimum, achieved) in &result.failed_minima {
            writeln!(
                w,
                "      <tr><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
                html_escape(&stat.to_string()),
                format_number(*minimum),
                format_number(*achieved),
                format_number(minimum - achieved),
            )
            .unwrap();
        }
        writeln!(w, "    </tbody>").unwrap();
        writeln!(w, "  </table>").unwrap();
        writeln!(
            w,
            "  <p>The result shown gets your highest-priority goals as close to their targets as possible; once a goal is met, extra points in it are not pursued at the expense of lower-priority goals still short of target.</p>"
        )
        .unwrap();
    }

    writeln!(w, "</body>").unwrap();
    writeln!(w, "</html>").unwrap();
    out
}

/// Build the terminal report for `lgo scrap-gear`.
pub fn format_scrap_gear_report(
    character: &str,
    class: &str,
    input_file: &str,
    timestamp: &str,
    evaluated_builds: &[(String, Vec<StatGoal>)],
    unused_items: &[ScrapUnusedItem],
) -> String {
    let mut out = String::new();
    let w = &mut out;
    let divider = "─".repeat(COL_SLOT + COL_ITEM + 3);

    writeln!(w, "  LGO — Saved-build item usage").unwrap();
    writeln!(w, "  Character : {} ({})", character, class).unwrap();
    writeln!(w, "  Gear List: {}", input_file).unwrap();
    writeln!(w, "  Run time  : {}", timestamp).unwrap();
    writeln!(w, "  {}", divider).unwrap();
    writeln!(w).unwrap();
    writeln!(w, "  Saved builds evaluated:").unwrap();
    for (name, goals) in evaluated_builds {
        writeln!(w, "    - {}: {}", name, format_goal_list(goals)).unwrap();
    }
    writeln!(w).unwrap();
    writeln!(w, "  Items not used in any saved build").unwrap();
    writeln!(w, "  These items may still be near-misses.").unwrap();

    if unused_items.is_empty() {
        writeln!(w).unwrap();
        writeln!(
            w,
            "  All current items are used by at least one saved build."
        )
        .unwrap();
        writeln!(w).unwrap();
        return out;
    }

    let mut seen_slot_families = HashSet::new();
    for slot in Slot::all() {
        if !seen_slot_families.insert(slot.display_name()) {
            continue;
        }
        let mut slot_items: Vec<&ScrapUnusedItem> = unused_items
            .iter()
            .filter(|item| item.slot.display_name() == slot.display_name())
            .collect();
        if slot_items.is_empty() {
            continue;
        }
        slot_items.sort_by(|left, right| left.name.cmp(&right.name));

        writeln!(w).unwrap();
        writeln!(w, "  {}:", slot.display_name()).unwrap();
        for item in slot_items {
            let unused_count = item.owned_count.saturating_sub(item.max_used_count);
            writeln!(
                w,
                "    - {} — {} owned, at most {} used in any build ({} {} not used in any saved build)",
                item.name,
                item.owned_count,
                item.max_used_count,
                unused_count,
                copy_word(unused_count),
            )
            .unwrap();
        }
    }

    writeln!(w).unwrap();
    out
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

    writeln!(w, "  LGO — Innate Base Stats").unwrap();
    writeln!(w, "  Character : {} ({})", character, class).unwrap();
    writeln!(w, "  Gear List: {}", input_file).unwrap();
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

fn write_header_text(
    w: &mut String,
    character: &str,
    class: &str,
    input_file: &str,
    timestamp: &str,
) {
    let divider = "─".repeat(COL_SLOT + COL_ITEM + 3);
    writeln!(w, "  {}", REPORT_TITLE).unwrap();
    writeln!(w, "  Character : {} ({})", character, class).unwrap();
    writeln!(w, "  Gear List: {}", input_file).unwrap();
    writeln!(w, "  Run time  : {}", timestamp).unwrap();
    writeln!(w, "  {}", divider).unwrap();
}

fn write_feasibility_status_text(w: &mut String, feasible: bool) {
    writeln!(w).unwrap();
    if feasible {
        writeln!(w, "  ✓  All stat minima met.").unwrap();
    } else {
        writeln!(w, "  ════════════════════════════════════════════════").unwrap();
        writeln!(w, "  ✗  INFEASIBLE — not all stat minima can be met ✗").unwrap();
        writeln!(w, "  ════════════════════════════════════════════════").unwrap();
    }
}

fn write_warnings_text(w: &mut String, warnings: &[String]) {
    writeln!(w).unwrap();
    writeln!(w, "  WARNINGS:").unwrap();
    for warning in warnings {
        writeln!(w, "    ⚠  {}", warning).unwrap();
    }
    writeln!(w).unwrap();
}

fn write_gear_table_text(w: &mut String, gear_set: &GearSet) {
    let divider = "─".repeat(COL_SLOT + COL_ITEM + 3);

    writeln!(w).unwrap();
    writeln!(w, "  {}", divider).unwrap();
    writeln!(w, "  Recommended Gear:").unwrap();
    writeln!(w, "  {:>12}  Recommended Item", "Slot").unwrap();
    writeln!(w, "  {}", divider).unwrap();

    let main_is_two_handed = gear_set
        .items
        .get(&Slot::MainHand)
        .is_some_and(|item| item.two_handed);

    for slot in Slot::all() {
        let item_display = truncate(
            &hand_or_item_label(gear_set, slot, main_is_two_handed),
            COL_ITEM,
        );
        writeln!(
            w,
            "  {:>COL_SLOT$}  {}",
            slot.display_name(),
            item_display,
            COL_SLOT = COL_SLOT
        )
        .unwrap();
    }

    writeln!(w, "  {}", divider).unwrap();
}

fn hand_or_item_label(gear_set: &GearSet, slot: Slot, main_is_two_handed: bool) -> String {
    if slot == Slot::OffHand && main_is_two_handed {
        return "(2-handed item)".to_string();
    }
    match gear_set.items.get(&slot) {
        Some(item) if is_empty_placeholder(&item.name) => "NO ITEMS".to_string(),
        Some(item) => item.name.clone(),
        None => "NO ITEMS".to_string(),
    }
}

fn is_empty_placeholder(name: &str) -> bool {
    name.starts_with("[empty")
}

fn write_stat_summary_text(
    w: &mut String,
    gear_set: &GearSet,
    goals: &[StatGoal],
    failed_minima: &[(Stat, i64, i64)],
) {
    if goals.is_empty() {
        return;
    }

    let failed_stats: HashSet<Stat> = failed_minima.iter().map(|(s, _, _)| *s).collect();
    let divider = "─".repeat(COL_STAT + COL_VALUE + COL_MIN + 11);

    writeln!(w).unwrap();
    writeln!(w, "  {}", divider).unwrap();
    writeln!(w, "  Goal Stat Summary:").unwrap();
    writeln!(
        w,
        "  {:<COL_STAT$}  {:>COL_VALUE$}  {:>COL_MIN$}  Met?",
        "Stat",
        "Goal",
        "Total",
        COL_STAT = COL_STAT,
        COL_VALUE = COL_VALUE,
        COL_MIN = COL_MIN,
    )
    .unwrap();
    writeln!(w, "  {}", divider).unwrap();

    for goal in goals {
        let total = gear_set.total(&goal.stat);
        let met_marker = goal_met_marker(goal.minimum, total);
        let flag = if failed_stats.contains(&goal.stat) {
            " ⚠"
        } else {
            ""
        };

        writeln!(
            w,
            "  {:<COL_STAT$}  {:>COL_VALUE$}  {:>COL_MIN$}  {}{}",
            format!("{}", goal.stat),
            if goal.minimum > 0 {
                format_number(goal.minimum)
            } else {
                "—".to_string()
            },
            format_number(total),
            met_marker,
            flag,
            COL_STAT = COL_STAT,
            COL_VALUE = COL_VALUE,
            COL_MIN = COL_MIN,
        )
        .unwrap();
    }

    writeln!(w, "  {}", divider).unwrap();
}

fn write_projected_tracked_stats_text(w: &mut String, gear_set: &GearSet) {
    let divider = "─".repeat(COL_STAT + COL_VALUE + 2);
    writeln!(w).unwrap();
    writeln!(
        w,
        "  Projected tracked stats (gear + essences + derived Base-stat contributions + Virtue fixed stats):"
    )
    .unwrap();
    writeln!(w).unwrap();
    for (stat, _) in TRACKED_STATS {
        writeln!(
            w,
            "  {:<COL_STAT$}  {:>COL_VALUE$}",
            format!("{}", stat),
            format_number(gear_set.total(stat)),
            COL_STAT = COL_STAT,
            COL_VALUE = COL_VALUE,
        )
        .unwrap();
    }
    writeln!(w, "  {}", divider).unwrap();
}

fn write_projected_base_stats_text(w: &mut String, projected_base_stats: &HashMap<Stat, i64>) {
    let divider = "─".repeat(COL_STAT + COL_VALUE + 2);
    writeln!(w).unwrap();
    writeln!(
        w,
        "  Projected raw Base stats (innate + gear + essences + Virtues; derivation inputs only):"
    )
    .unwrap();
    writeln!(w).unwrap();
    for (stat, _) in BASE_STATS {
        writeln!(
            w,
            "  {:<COL_STAT$}  {:>COL_VALUE$}",
            format!("{}", stat),
            format_number(projected_base_stats.get(stat).copied().unwrap_or(0)),
            COL_STAT = COL_STAT,
            COL_VALUE = COL_VALUE,
        )
        .unwrap();
    }
    writeln!(w, "  {}", divider).unwrap();
}

fn write_infeasible_details_text(w: &mut String, failed_minima: &[(Stat, i64, i64)]) {
    writeln!(w).unwrap();
    writeln!(w, "  The following stats could not reach their minima").unwrap();
    writeln!(w, "  with any combination of the available items:").unwrap();
    writeln!(w).unwrap();
    for (stat, minimum, achieved) in failed_minima {
        writeln!(
            w,
            "    {:<COL_STAT$}  needed {:>8}  achieved {:>8}  short by {:>8}",
            format!("{}", stat),
            format_number(*minimum),
            format_number(*achieved),
            format_number(minimum - achieved),
            COL_STAT = COL_STAT,
        )
        .unwrap();
    }
    writeln!(w).unwrap();
    writeln!(
        w,
        "  The result shown gets your highest-priority goals as close"
    )
    .unwrap();
    writeln!(
        w,
        "  to their targets as possible; once a goal is met, extra"
    )
    .unwrap();
    writeln!(
        w,
        "  points in it are not pursued at the expense of lower-priority"
    )
    .unwrap();
    writeln!(w, "  goals still short of target.").unwrap();
}

fn write_feasibility_status_html(w: &mut String, feasible: bool) {
    if feasible {
        writeln!(
            w,
            "  <p class=\"banner feasible\">✓ All stat minima met.</p>"
        )
        .unwrap();
    } else {
        writeln!(
            w,
            "  <div class=\"banner infeasible\">✗ INFEASIBLE — not all stat minima can be met ✗</div>"
        )
        .unwrap();
    }
}

fn goal_met_marker(minimum: i64, total: i64) -> &'static str {
    if minimum == 0 {
        "—"
    } else if total >= minimum {
        "✓"
    } else {
        "✗"
    }
}

fn format_goal_list(goals: &[StatGoal]) -> String {
    goals
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn html_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Format an i64 with thousands separators: 1234567 → "1,234,567".
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

fn copy_word(n: usize) -> &'static str {
    if n == 1 {
        "copy"
    } else {
        "copies"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gear::GearItem;

    #[test]
    fn terminal_status_markers_are_colored_without_changing_other_text() {
        let colored = colorize_terminal_status_markers("✓ met\n✗ failed\n⚠ warning\n— ignored");

        assert!(colored.contains("\x1b[32m✓\x1b[0m"));
        assert!(colored.contains("\x1b[31m✗\x1b[0m"));
        assert!(colored.contains("\x1b[33m⚠\x1b[0m"));
        assert!(colored.contains("— ignored"));
    }

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
        assert!(report.contains("Gear List: lgo_Thalya_gearReady.toml"));
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

    #[test]
    fn optimize_report_lists_timestamp_and_all_projected_stats() {
        let report = format_optimize_report(
            &sample_optimize_result(),
            &[StatGoal {
                stat: Stat::Morale,
                minimum: 1000,
            }],
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &[
                (Stat::Might, 5300),
                (Stat::Agility, 2650),
                (Stat::Vitality, 10200),
                (Stat::Will, 7950),
                (Stat::Fate, 4000),
            ]
            .into_iter()
            .collect(),
        );

        assert!(report.contains("Run time  : 2026-08-31 08:00:00 +00:00"));
        assert!(report.contains("Projected tracked stats"));
        assert!(report.contains("Projected raw Base stats"));
        for (stat, _) in TRACKED_STATS {
            assert!(
                report.contains(&format!("{}", stat)),
                "projected section must list tracked stat {}",
                stat
            );
        }
        assert_eq!(projected_tracked_stat_value(&report, "Block"), Some(0));
        assert_eq!(projected_tracked_stat_value(&report, "Evade"), Some(0));
    }

    #[test]
    fn text_optimize_report_uses_section_headers_and_new_order() {
        let report = format_optimize_report(
            &sample_optimize_result(),
            &[StatGoal {
                stat: Stat::Morale,
                minimum: 1000,
            }],
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &HashMap::new(),
        );

        assert!(report.contains("Goal Stat Summary:"));
        assert!(report.contains("Recommended Gear:"));
        assert!(
            report
                .find("✓  All stat minima met.")
                .expect("status present")
                < report
                    .find("Goal Stat Summary:")
                    .expect("goal summary present")
        );
        assert!(
            report
                .find("Goal Stat Summary:")
                .expect("goal summary present")
                < report
                    .find("Recommended Gear:")
                    .expect("recommended gear present")
        );
        let summary_header = report
            .lines()
            .find(|line| line.contains("Stat") && line.contains("Goal") && line.contains("Total"))
            .expect("summary header present");
        assert!(
            summary_header.find("Goal").expect("goal header present")
                < summary_header.find("Total").expect("total header present")
        );
        assert!(!summary_header.contains("Minimum"));
        let morale_row = report
            .lines()
            .find(|line| line.trim_start().starts_with("Morale"))
            .expect("morale row present");
        assert!(
            morale_row.find("1,000").expect("goal value present")
                < morale_row.find("1,250").expect("total value present")
        );
        assert_eq!(report.matches("✓  All stat minima met.").count(), 1);
    }

    #[test]
    fn infeasible_text_report_shows_single_top_banner_and_lower_details() {
        let report = format_optimize_report(
            &infeasible_optimize_result(),
            &[StatGoal {
                stat: Stat::Morale,
                minimum: 2000,
            }],
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &HashMap::new(),
        );

        assert_eq!(
            report
                .matches("✗  INFEASIBLE — not all stat minima can be met ✗")
                .count(),
            1
        );
        assert!(
            report
                .find("✗  INFEASIBLE — not all stat minima can be met ✗")
                .expect("banner present")
                < report
                    .find("Goal Stat Summary:")
                    .expect("goal summary present")
        );
        assert!(
            report
                .find("The following stats could not reach their minima")
                .expect("details present")
                > report
                    .find("Projected raw Base stats")
                    .expect("base stats present")
        );
    }

    #[test]
    fn html_report_escapes_item_names() {
        let report = format_optimize_report_html(
            &sample_optimize_result_with_name("Shield <&> \"Quote\" 'Single'"),
            &[StatGoal {
                stat: Stat::Morale,
                minimum: 1000,
            }],
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &HashMap::new(),
        );

        assert!(report.contains("&lt;&amp;&gt;"));
        assert!(report.contains("&quot;Quote&quot;"));
        assert!(report.contains("&#39;Single&#39;"));
        assert!(report.contains("<meta charset=\"utf-8\">"));
    }

    #[test]
    fn html_optimize_report_uses_new_order_and_goal_headers() {
        let report = format_optimize_report_html(
            &sample_optimize_result(),
            &[StatGoal {
                stat: Stat::Morale,
                minimum: 1000,
            }],
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &HashMap::new(),
        );

        assert!(
            report
                .find("✓ All stat minima met.")
                .expect("status present")
                < report
                    .find("<h2>Goal stat summary</h2>")
                    .expect("goal summary present")
        );
        assert!(
            report
                .find("<h2>Goal stat summary</h2>")
                .expect("goal summary present")
                < report
                    .find("<h2>Recommended gear</h2>")
                    .expect("gear section present")
        );
        assert!(report.contains(
            "<thead><tr><th>Stat</th><th class=\"num\">Goal</th><th class=\"num\">Total</th><th>Met?</th></tr></thead>"
        ));
        assert!(!report.contains("Minimum"));
        assert_eq!(report.matches("✓ All stat minima met.").count(), 1);
    }

    #[test]
    fn off_hand_label_shows_two_handed_marker_when_main_is_two_handed() {
        let mut items = HashMap::new();
        items.insert(Slot::MainHand, item("Greatsword", Slot::MainHand, true));
        items.insert(
            Slot::OffHand,
            item("[empty Off-hand]", Slot::OffHand, false),
        );
        let gear_set = GearSet::new(HashMap::new());
        let gear_set = GearSet { items, ..gear_set };

        assert_eq!(
            hand_or_item_label(&gear_set, Slot::MainHand, true),
            "Greatsword"
        );
        assert_eq!(
            hand_or_item_label(&gear_set, Slot::OffHand, true),
            "(2-handed item)"
        );
    }

    #[test]
    fn empty_pool_placeholder_renders_as_no_items() {
        let mut items = HashMap::new();
        items.insert(Slot::Head, item("[empty Head]", Slot::Head, false));
        let gear_set = GearSet::new(HashMap::new());
        let gear_set = GearSet { items, ..gear_set };

        assert_eq!(hand_or_item_label(&gear_set, Slot::Head, false), "NO ITEMS");
        assert_eq!(
            hand_or_item_label(&gear_set, Slot::Chest, false),
            "NO ITEMS"
        );
    }

    #[test]
    fn normal_off_hand_shows_item_name_when_main_is_one_handed() {
        let mut items = HashMap::new();
        items.insert(Slot::MainHand, item("Sword", Slot::MainHand, false));
        items.insert(Slot::OffHand, item("Shield", Slot::OffHand, false));
        let gear_set = GearSet::new(HashMap::new());
        let gear_set = GearSet { items, ..gear_set };

        assert_eq!(
            hand_or_item_label(&gear_set, Slot::OffHand, false),
            "Shield"
        );
    }

    #[test]
    fn scrap_gear_report_lists_builds_and_unused_copies() {
        let report = format_scrap_gear_report(
            "Thalya",
            "Lore-master",
            "TestData/lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &[
                (
                    "healer".to_string(),
                    vec![
                        StatGoal {
                            stat: Stat::OutgoingHealing,
                            minimum: 200000,
                        },
                        StatGoal {
                            stat: Stat::CriticalRating,
                            minimum: 350000,
                        },
                    ],
                ),
                (
                    "tank".to_string(),
                    vec![StatGoal {
                        stat: Stat::TacticalMitigation,
                        minimum: 450000,
                    }],
                ),
            ],
            &[ScrapUnusedItem {
                slot: Slot::Finger1,
                name: "Keen Pristine Madáshi Ring".to_string(),
                owned_count: 2,
                max_used_count: 1,
            }],
        );

        assert!(report.contains("Saved builds evaluated:"));
        assert!(report.contains("- healer: oh:200000, cr:350000"));
        assert!(report.contains("- tank: tt:450000"));
        assert!(report.contains("Items not used in any saved build"));
        assert!(report.contains(
            "Keen Pristine Madáshi Ring — 2 owned, at most 1 used in any build (1 copy not used in any saved build)"
        ));
    }

    #[test]
    fn scrap_gear_report_handles_fully_used_pool() {
        let report = format_scrap_gear_report(
            "Thalya",
            "Lore-master",
            "gear.toml",
            "2026-08-31 08:00:00 +00:00",
            &[(
                "healer".to_string(),
                vec![StatGoal {
                    stat: Stat::OutgoingHealing,
                    minimum: 200000,
                }],
            )],
            &[],
        );

        assert!(report.contains("All current items are used by at least one saved build."));
    }

    fn sample_optimize_result() -> OptimizeResult {
        sample_optimize_result_with_name("Simple Helm")
    }

    fn sample_optimize_result_with_name(name: &str) -> OptimizeResult {
        let mut innate_stats = HashMap::new();
        innate_stats.insert(Stat::Morale, 1000);
        let mut gear_set = GearSet::new(innate_stats);
        gear_set.items.insert(
            Slot::Head,
            GearItem {
                name: name.to_string(),
                slot: Slot::Head,
                two_handed: false,
                either_hand: false,
                stats: [(Stat::Morale, 250), (Stat::Armor, 300)]
                    .into_iter()
                    .collect(),
            },
        );

        OptimizeResult {
            gear_set,
            feasible: true,
            failed_minima: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn infeasible_optimize_result() -> OptimizeResult {
        let mut result = sample_optimize_result();
        result.feasible = false;
        result.failed_minima = vec![(Stat::Morale, 2000, 1250)];
        result
    }

    fn projected_tracked_stat_value(report: &str, stat_name: &str) -> Option<i64> {
        let mut in_section = false;

        for line in report.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Projected tracked stats ") {
                in_section = true;
                continue;
            }
            if !in_section {
                continue;
            }
            if trimmed.is_empty() || trimmed.starts_with('─') {
                continue;
            }
            if trimmed.starts_with("Projected raw Base stats") {
                break;
            }
            let Some(rest) = trimmed.strip_prefix(stat_name) else {
                continue;
            };
            let value = rest
                .split_whitespace()
                .last()?
                .replace(',', "")
                .parse::<i64>()
                .ok()?;
            return Some(value);
        }

        None
    }

    fn item(name: &str, slot: Slot, two_handed: bool) -> GearItem {
        GearItem {
            name: name.to_string(),
            slot,
            two_handed,
            either_hand: false,
            stats: HashMap::new(),
        }
    }
}
