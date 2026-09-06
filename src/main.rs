//! LGO - LOTRO Gear Optimizer

use chrono::Local;
use lgo::{
    base_stats, build_db, build_profiles, gear, gearstats, install, optimizer, report,
    report_files, slot_resolver, stat, virtues,
};

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process;

use stat::StatGoal;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        process::exit(0);
    }

    let command = match parse_command(&args) {
        Ok(c) => c,
        Err(CliParseError::MissingSubcommand) => {
            eprintln!(
                "Error: missing subcommand. Did you mean: lgo optimize <stat:min> [<stat:min> ...]?"
            );
            eprintln!("Run with --help for usage.");
            process::exit(1);
        }
        Err(CliParseError::Message(e)) => {
            eprintln!("Error: {}", e);
            eprintln!("Run with --help for usage.");
            process::exit(1);
        }
    };

    match command {
        Command::Help => {
            print_usage();
        }
        Command::StatList => print_stat_list(),
        Command::Optimize(cli) => run_optimize(&cli),
        Command::ScrapGear(cli) => run_scrap_gear(&cli),
        Command::BaseStats(cli) => run_base_stats(&cli),
        Command::ResolveSlots(cli) => run_resolve_slots(&cli),
        Command::BuildDb(cli) => run_build_db(&cli),
    }
}

fn read_gear_doc_or_exit(stats_file: &Path) -> gearstats::GearDoc {
    match gearstats::read_stats_file(stats_file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading gear stats file: {}", e);
            process::exit(1);
        }
    }
}

fn prepare_gear_doc_for_optimization(gear_doc: &mut gearstats::GearDoc, class: &str) {
    if !gear_doc.selected_virtues.is_empty() {
        let virtues = load_virtues_or_exit();
        if let Err(e) = virtues.apply_selected_virtues(gear_doc) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }

    let derivations = load_derivations_or_exit();
    if let Err(e) = derivations.derive_doc(class, gear_doc) {
        eprintln!("Error deriving Base stats for class '{}': {}", class, e);
        process::exit(1);
    }
}

fn build_resolved_candidates(
    gear_doc: &gearstats::GearDoc,
) -> (HashMap<String, gear::GearItem>, Vec<String>) {
    let resolved: HashMap<String, gear::GearItem> = gear_doc
        .items
        .iter()
        .enumerate()
        .map(|(idx, doc_item)| {
            (
                gear::optimizer_candidate_key(idx, &doc_item.item),
                doc_item.item.clone(),
            )
        })
        .collect();
    let candidate_names: Vec<String> = resolved.keys().cloned().collect();
    (resolved, candidate_names)
}

fn extract_character_segment_from_canonical_gear_filename(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let file_name_lower = file_name.to_ascii_lowercase();
    let prefix = "lgo_";
    let suffix = "_gearready.toml";
    if !file_name_lower.starts_with(prefix) || !file_name_lower.ends_with(suffix) {
        return None;
    }
    Some(file_name[prefix.len()..file_name.len() - suffix.len()].to_string())
}

fn resolve_builds_file(
    explicit_builds_file: Option<&PathBuf>,
    gear_file: &Path,
    gear_doc_character: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(path) = explicit_builds_file {
        return Ok(path.clone());
    }

    let character = extract_character_segment_from_canonical_gear_filename(gear_file)
        .or_else(|| gear_doc_character.map(String::from))
        .ok_or_else(|| {
            format!(
                "Cannot determine saved-builds filename beside {}. Use a canonical `lgo_<character>_gearReady.toml` filename, add a top-level `character = \"...\"` field, or pass --builds-file <path>.",
                gear_file.display()
            )
        })?;
    let dir = gear_file
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    match gearstats::find_builds_file(dir, &character)? {
        Some(path) => Ok(path),
        None => Ok(dir.join(format!("lgo_{}_builds.toml", character))),
    }
}

fn read_saved_builds_if_present(path: &Path) -> Result<build_profiles::SavedBuilds, String> {
    if !path.exists() {
        return Ok(build_profiles::SavedBuilds::default());
    }
    build_profiles::SavedBuilds::read_file(path)
}

fn no_saved_builds_message(builds_file: &Path) -> String {
    format!(
        "No saved builds found in {}. Save one with: lgo optimize --save-build <name> <goals...>",
        builds_file.display()
    )
}

fn count_selected_real_items_by_name(gear_set: &gear::GearSet) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for item in gear_set.items.values() {
        if is_empty_placeholder(&item.name) {
            continue;
        }
        *counts.entry(item.name.clone()).or_insert(0) += 1;
    }
    counts
}

fn is_empty_placeholder(name: &str) -> bool {
    name.starts_with("[empty") || name == "NO ITEMS" || name == "(2-handed item)"
}

const UNKNOWN: &str = "Unknown";

/// Determine the install directory (exe-anchored, or `LGO_HOME`), exiting with
/// a clear message if it cannot be resolved.
fn install_dir_or_exit() -> PathBuf {
    match install::install_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error: cannot determine the install directory: {}", e);
            process::exit(1);
        }
    }
}

/// The install-tree reports directory for a run: `<install>/<char>_Gear/<char>_Reports`.
/// The character is the authoritative discovered folder name when auto-detected,
/// otherwise resolved from the gear TOML's `character` field or filename.
fn resolve_reports_dir(
    stats_file: &Path,
    discovered_character: Option<&str>,
    gear_doc_character: Option<&str>,
) -> PathBuf {
    let install = install_dir_or_exit();
    let character = match discovered_character {
        Some(c) => c.to_string(),
        None => match install::resolve_report_character(stats_file, gear_doc_character) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
    };
    install::reports_dir(&install, &character)
}

/// Locate the canonical gear file for optimize/base-stats/scrap-gear: an
/// explicit `--file` path when given, otherwise auto-discovery of
/// `lgo_<character>_gearReady.toml` inside the selected `<character>_Gear`
/// folder under the install directory. Returns the path plus the discovered
/// character name (the authoritative folder name, or `None` for `--file`).
/// Exits the process with a clear message on failure.
fn locate_canonical_gear_file(
    character: Option<&str>,
    file: Option<&PathBuf>,
) -> (PathBuf, Option<String>) {
    if let Some(path) = file {
        if !path.exists() {
            eprintln!("Error: File not found: {}", path.display());
            process::exit(1);
        }
        return (path.clone(), None);
    }

    let install = install_dir_or_exit();
    let selection = match install::select_character(&install, character) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    if let Some(note) = &selection.note {
        println!("{}", note);
    }
    install::warn_on_name_mismatch(&selection.gear_dir, &selection.character);

    let stats_file = match gearstats::find_canonical_gear_file(
        &selection.gear_dir,
        &selection.character,
    ) {
        Ok(Some(path)) => path,
        Ok(None) => {
            eprintln!(
                "No lgo_{}_gearReady.toml file found in {}",
                selection.character,
                selection.gear_dir.display()
            );
            eprintln!(
                "\nThis file is created by running 'lgo resolve-slots' after completing the bookmarklet workflow."
            );
            eprintln!("Please follow these steps:");
            eprintln!("  1) Place candidate items in a Shared Storage chest named 'lgo'");
            eprintln!("  2) Run /lgo export in-game");
            eprintln!("  3) Navigate to https://lotro-wiki.com in your browser");
            eprintln!("  4) Click the LGO bookmarklet");
            eprintln!(
                "  5) Paste lgo_<character-name>_gearNames_<timestamp>.plugindata when prompted"
            );
            eprintln!(
                "  6) Save the generated lgo_{}_gearStats.toml into {} (or the install root — it will be moved automatically)",
                selection.character,
                selection.gear_dir.display()
            );
            eprintln!(
                "  7) Run: lgo resolve-slots  (creates lgo_{}_gearReady.toml in that folder)",
                selection.character
            );
            eprintln!("  8) Run: lgo optimize <stat:min> [<stat:min> ...]");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    (stats_file, Some(selection.character))
}

/// The game's Documents export directory for `character`
/// (`Documents\The Lord of the Rings Online\PluginData\<char>\AllServers`).
/// The `.plugindata` file's location is fixed by the Turbine API and is the
/// only input that still lives outside the install tree. Returns `None` if the
/// Documents root cannot be determined.
fn plugindata_dir_for(character: &str) -> Option<PathBuf> {
    let docs = documents_dir().ok()?;
    Some(
        docs.join("The Lord of the Rings Online")
            .join("PluginData")
            .join(character)
            .join("AllServers"),
    )
}

/// Load `data/base_stat_derivations.json` from the install directory (the
/// folder containing `lgo.exe`, or `LGO_HOME`), exiting with a clear message
/// if the file is missing or malformed.
fn load_derivations_or_exit() -> base_stats::BaseStatDerivations {
    match base_stats::BaseStatDerivations::load_default() {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "Failed to load base-stat derivation data ({}): {}",
                base_stats::DEFAULT_DERIVATIONS_PATH,
                e
            );
            eprintln!(
                "Ensure data/base_stat_derivations.json exists in the install directory (beside lgo.exe)."
            );
            process::exit(1);
        }
    }
}

/// Load `data/lgo_virtues.json` from the install directory, exiting with a
/// clear message if the file is missing or malformed.
fn load_virtues_or_exit() -> virtues::VirtuesDb {
    match virtues::VirtuesDb::load_default() {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "Failed to load virtue data ({}): {}",
                virtues::DEFAULT_VIRTUES_PATH,
                e
            );
            eprintln!(
                "Ensure data/lgo_virtues.json exists in the install directory (beside lgo.exe)."
            );
            process::exit(1);
        }
    }
}

fn run_scrap_gear(cli: &ScrapGearCli) {
    let (stats_file, auto_discovered_character) =
        locate_canonical_gear_file(cli.character.as_deref(), cli.file.as_ref());
    let gear_doc = read_gear_doc_or_exit(&stats_file);
    let builds_file = match resolve_builds_file(
        cli.builds_file.as_ref(),
        &stats_file,
        gear_doc.character.as_deref(),
    ) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    let saved_builds = match read_saved_builds_if_present(&builds_file) {
        Ok(builds) => builds,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    if saved_builds.is_empty() {
        eprintln!("Error: {}", no_saved_builds_message(&builds_file));
        process::exit(1);
    }

    let character = resolve_report_character(
        gear_doc.character.clone(),
        cli.character.as_deref(),
        auto_discovered_character.clone(),
    );
    let mut gear_doc = gear_doc;
    let class = gear_doc
        .class
        .clone()
        .unwrap_or_else(|| UNKNOWN.to_string());

    prepare_gear_doc_for_optimization(&mut gear_doc, &class);
    let (resolved, candidate_names) = build_resolved_candidates(&gear_doc);

    let mut max_used_by_name: HashMap<String, usize> = HashMap::new();
    let mut evaluated_builds = Vec::new();
    for build in saved_builds.builds() {
        evaluated_builds.push((build.name.clone(), build.goals.clone()));
        let result = optimizer::optimize(
            &resolved,
            &candidate_names,
            &build.goals,
            &gear_doc.innate_stats,
        );
        for (name, used_count) in count_selected_real_items_by_name(&result.gear_set) {
            max_used_by_name
                .entry(name)
                .and_modify(|current| *current = (*current).max(used_count))
                .or_insert(used_count);
        }
    }

    let mut owned_by_name: HashMap<String, (gear::Slot, usize)> = HashMap::new();
    for doc_item in &gear_doc.items {
        let entry = owned_by_name
            .entry(doc_item.item.name.clone())
            .or_insert((doc_item.item.slot, 0));
        if slot_order_index(doc_item.item.slot) < slot_order_index(entry.0) {
            entry.0 = doc_item.item.slot;
        }
        entry.1 += 1;
    }

    let mut unused_items = Vec::new();
    for (name, (slot, owned_count)) in owned_by_name {
        let max_used = max_used_by_name.get(&name).copied().unwrap_or(0);
        if owned_count > max_used {
            unused_items.push(report::ScrapUnusedItem {
                slot,
                name,
                owned_count,
                max_used_count: max_used,
            });
        }
    }

    let stats_file_display = stats_file.display().to_string();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S %:z").to_string();
    let text_report = report::format_scrap_gear_report(
        &character,
        &class,
        &stats_file_display,
        &timestamp,
        &evaluated_builds,
        &unused_items,
    );
    print!("{}", text_report);

    let reports_dir = resolve_reports_dir(
        &stats_file,
        auto_discovered_character.as_deref(),
        gear_doc.character.as_deref(),
    );
    match report_files::write_scrap_gear_report_file(&reports_dir, &text_report) {
        Ok(path) => {
            println!("  Scrap-gear report written to: {}", path.display());
        }
        Err(err) => {
            eprintln!("Warning: could not write scrap-gear report file: {}", err);
        }
    }
}

fn run_optimize(cli: &OptimizeCli) {
    if cli.build.is_none() && cli.goals.is_empty() {
        eprintln!("Error: at least one stat goal is required.");
        eprintln!("Run with --help for usage.");
        process::exit(1);
    }

    let (stats_file, auto_discovered_character) =
        locate_canonical_gear_file(cli.character.as_deref(), cli.file.as_ref());

    let mut gear_doc = read_gear_doc_or_exit(&stats_file);

    let character = resolve_report_character(
        gear_doc.character.clone(),
        cli.character.as_deref(),
        auto_discovered_character.clone(),
    );
    let class = gear_doc
        .class
        .clone()
        .unwrap_or_else(|| UNKNOWN.to_string());

    let builds_file = if cli.build.is_some() || cli.save_build.is_some() {
        Some(
            match resolve_builds_file(
                cli.builds_file.as_ref(),
                &stats_file,
                gear_doc.character.as_deref(),
            ) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            },
        )
    } else {
        None
    };

    let goals = if let Some(build_name) = &cli.build {
        let builds_file = builds_file
            .as_ref()
            .expect("builds file path must exist when --build is used");
        let saved_builds = match read_saved_builds_if_present(builds_file) {
            Ok(builds) => builds,
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        };
        if saved_builds.is_empty() {
            eprintln!("Error: {}", no_saved_builds_message(builds_file.as_path()));
            process::exit(1);
        }
        match saved_builds.find(build_name) {
            Some(build) => build.goals.clone(),
            None => {
                eprintln!(
                    "Error: no saved build named '{}' found in {}.",
                    build_name,
                    builds_file.display()
                );
                eprintln!(
                    "Save one with: lgo optimize --save-build {} <goals...>",
                    build_name
                );
                process::exit(1);
            }
        }
    } else {
        cli.goals.clone()
    };

    prepare_gear_doc_for_optimization(&mut gear_doc, &class);

    let (resolved, candidate_names) = build_resolved_candidates(&gear_doc);
    let result = optimizer::optimize(&resolved, &candidate_names, &goals, &gear_doc.innate_stats);

    let projected_base_stats = projected_base_stats(
        &gear_doc.innate_base_stats,
        &result.gear_set,
        &gear_doc.items,
    );
    let stats_file_display = stats_file.display().to_string();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S %:z").to_string();
    let text_report = report::format_optimize_report(
        &result,
        &goals,
        &character,
        &class,
        &stats_file_display,
        &timestamp,
        &projected_base_stats,
    );
    print!("{}", report::colorize_terminal_status_markers(&text_report));

    if let Some(build_name) = &cli.save_build {
        let builds_file = builds_file
            .as_ref()
            .expect("builds file path must exist when --save-build is used");
        let mut saved_builds = match read_saved_builds_if_present(builds_file) {
            Ok(builds) => builds,
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        };
        saved_builds.upsert(build_name.clone(), goals.clone());
        if let Err(e) = saved_builds.write_file(builds_file) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        println!(
            "  Saved build '{}' to {}",
            build_name,
            builds_file.display()
        );
    }

    if cli.to_file {
        let html_report = report::format_optimize_report_html(
            &result,
            &goals,
            &character,
            &class,
            &stats_file_display,
            &timestamp,
            &projected_base_stats,
        );

        let reports_dir = resolve_reports_dir(
            &stats_file,
            auto_discovered_character.as_deref(),
            gear_doc.character.as_deref(),
        );
        match report_files::write_optimize_report_files(&reports_dir, &text_report, &html_report) {
            Ok(paths) => {
                println!(
                    "  Report written to: {} and {}",
                    paths.text_path.display(),
                    paths.html_path.display()
                );
            }
            Err(err) => {
                eprintln!("Warning: could not write optimize report files: {}", err);
            }
        }
    }
}

fn projected_base_stats(
    innate_base_stats: &HashMap<stat::Stat, i64>,
    gear_set: &gear::GearSet,
    doc_items: &[gearstats::DocItem],
) -> HashMap<stat::Stat, i64> {
    let mut totals = innate_base_stats.clone();
    let mut used = vec![false; doc_items.len()];

    for slot in gear::Slot::all() {
        let Some(selected_item) = gear_set.items.get(&slot) else {
            continue;
        };
        if selected_item.name.starts_with("[empty") {
            continue;
        }
        let Some((idx, doc_item)) = doc_items.iter().enumerate().find(|(idx, doc_item)| {
            !used[*idx] && doc_item_matches_selected(doc_item, selected_item, slot)
        }) else {
            continue;
        };
        used[idx] = true;
        merge_stat_totals(&doc_item.base_stats, &mut totals);
    }

    totals.retain(|_, value| *value != 0);
    totals
}

fn doc_item_matches_selected(
    doc_item: &gearstats::DocItem,
    selected_item: &gear::GearItem,
    equipped_slot: gear::Slot,
) -> bool {
    doc_item.item.name == selected_item.name
        && doc_item.item.two_handed == selected_item.two_handed
        && doc_item.item.either_hand == selected_item.either_hand
        && doc_item.item.stats == selected_item.stats
        && slots_match_for_report(doc_item.item.slot, equipped_slot, selected_item.either_hand)
}

fn slots_match_for_report(
    original_slot: gear::Slot,
    equipped_slot: gear::Slot,
    either_hand: bool,
) -> bool {
    original_slot.display_name() == equipped_slot.display_name()
        || (either_hand
            && original_slot == gear::Slot::OffHand
            && equipped_slot == gear::Slot::MainHand)
}

fn merge_stat_totals(src: &HashMap<stat::Stat, i64>, dst: &mut HashMap<stat::Stat, i64>) {
    for (stat, value) in src {
        *dst.entry(*stat).or_insert(0) += value;
    }
}

fn resolve_report_character(
    gear_doc_character: Option<String>,
    cli_character: Option<&str>,
    auto_discovered_character: Option<String>,
) -> String {
    gear_doc_character
        .or_else(|| cli_character.map(String::from))
        .or(auto_discovered_character)
        .unwrap_or_else(|| UNKNOWN.to_string())
}

fn run_base_stats(cli: &BaseStatsCli) {
    let (stats_file, auto_discovered_character) =
        locate_canonical_gear_file(cli.character.as_deref(), cli.file.as_ref());

    let gear_doc = match gearstats::read_stats_file(&stats_file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading gear stats file: {}", e);
            process::exit(1);
        }
    };

    let character = resolve_report_character(
        gear_doc.character.clone(),
        cli.character.as_deref(),
        auto_discovered_character,
    );
    let class = gear_doc
        .class
        .clone()
        .unwrap_or_else(|| UNKNOWN.to_string());

    let derivations = load_derivations_or_exit();
    let derived = match derivations.derive_stats(&class, &gear_doc.innate_base_stats) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error deriving Base stats for class '{}': {}", class, e);
            process::exit(1);
        }
    };

    report::print_base_stats_report(
        &character,
        &class,
        &stats_file.display().to_string(),
        &gear_doc.innate_base_stats,
        &derived,
    );
}

fn run_resolve_slots(cli: &ResolveSlotsCli) {
    let install = install_dir_or_exit();
    let selection = match install::prepare_resolve_slots(&install, cli.character.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    if let Some(note) = &selection.note {
        println!("{}", note);
    }
    install::warn_on_name_mismatch(&selection.gear_dir, &selection.character);

    let db = match slot_resolver::ItemsDb::load_default() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to load items DB (data/lgo_items.json): {}", e);
            eprintln!(
                "Ensure data/lgo_items.json exists in the install directory (beside lgo.exe)."
            );
            process::exit(1);
        }
    };

    let force = if cli.force {
        slot_resolver::ForceMode::Force {
            prompter: Box::new(slot_resolver::StdinPrompter),
        }
    } else {
        slot_resolver::ForceMode::NoForce
    };

    let plugindata_dir = plugindata_dir_for(&selection.character);
    let report = match slot_resolver::resolve_stats_file(
        &selection.gear_dir,
        plugindata_dir.as_deref(),
        &selection.character,
        &db,
        force,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    print_resolve_slots_report(&report);
}

fn print_resolve_slots_report(report: &slot_resolver::Report) {
    let outcome = &report.outcome;

    if report.no_new_export {
        println!("No new export found; canonical file is unchanged.");
        println!("Canonical: {}", report.canonical_path.display());
        return;
    }

    println!("Added: {}", outcome.added.len());
    if outcome.overwritten.is_empty() {
        println!("Preserved: {}", outcome.preserved.len());
    } else {
        println!(
            "Overwritten: {} / Preserved: {}",
            outcome.overwritten.len(),
            outcome.preserved.len()
        );
    }
    println!("Removed: {}", outcome.removed.len());
    for name in &outcome.removed {
        println!("Removed (no longer in export): {}", name);
    }
    let mut unknowns = outcome.unknown_slot.clone();
    unknowns.sort();
    for name in &unknowns {
        println!("Unknown slot (may need hand-edit): {}", name);
    }
    if report.previous_existed {
        println!("Previous: {}", report.canonical_path.display());
    } else {
        println!("Previous: (none — first run)");
    }
    if let Some(p) = &report.bookmarklet_path {
        println!("New export: {}", p.display());
    }
    println!("Wrote: {}", report.canonical_path.display());
}

fn run_build_db(cli: &BuildDbCli) {
    if let Err(e) = build_db::build(&cli.items, &cli.out) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

#[derive(Debug)]
enum Command {
    Help,
    StatList,
    Optimize(OptimizeCli),
    ScrapGear(ScrapGearCli),
    BaseStats(BaseStatsCli),
    ResolveSlots(ResolveSlotsCli),
    BuildDb(BuildDbCli),
}

#[derive(Debug)]
enum CliParseError {
    MissingSubcommand,
    Message(String),
}

#[derive(Debug)]
struct OptimizeCli {
    character: Option<String>,
    file: Option<PathBuf>,
    builds_file: Option<PathBuf>,
    build: Option<String>,
    save_build: Option<String>,
    to_file: bool,
    goals: Vec<StatGoal>,
}

#[derive(Debug)]
struct ScrapGearCli {
    character: Option<String>,
    file: Option<PathBuf>,
    builds_file: Option<PathBuf>,
}

/// `lgo base-stats`: show the raw innate Base stats and the tracked-stat
/// contributions derived from them. Shares `optimize`'s file discovery.
#[derive(Debug)]
struct BaseStatsCli {
    character: Option<String>,
    file: Option<PathBuf>,
}

#[derive(Debug)]
struct ResolveSlotsCli {
    character: Option<String>,
    /// `--force` / `-f`: prompt the user per item before overwriting or
    /// removing entries in the canonical gear file. Without this flag the
    /// resolver preserves existing entries on every iteration.
    force: bool,
}

#[derive(Debug)]
struct BuildDbCli {
    items: PathBuf,
    out: PathBuf,
}

fn parse_command(args: &[String]) -> Result<Command, CliParseError> {
    // `--help` / `-h` / `help` anywhere on the command line wins: print usage.
    if args.iter().any(|a| {
        let t = a.to_ascii_lowercase();
        t == "--help" || t == "-h" || t == "help"
    }) {
        return Ok(Command::Help);
    }

    let verb = args[0].to_ascii_lowercase();
    match verb.as_str() {
        "statlist" | "--statlist" => Ok(Command::StatList),
        "optimize" | "--optimize" | "-o" => parse_optimize_args(&args[1..])
            .map(Command::Optimize)
            .map_err(CliParseError::Message),
        "scrap-gear" | "scrapgear" | "--scrap-gear" | "-s" => parse_scrap_gear_args(&args[1..])
            .map(Command::ScrapGear)
            .map_err(CliParseError::Message),
        "base-stats" | "--base-stats" => parse_base_stats_args(&args[1..])
            .map(Command::BaseStats)
            .map_err(CliParseError::Message),
        "resolve-slots" | "--resolve-slots" | "-r" => parse_resolve_slots_args(&args[1..])
            .map(Command::ResolveSlots)
            .map_err(CliParseError::Message),
        "build-db" | "--build-db" | "-b" => parse_build_db_args(&args[1..])
            .map(Command::BuildDb)
            .map_err(CliParseError::Message),
        _ => Err(CliParseError::MissingSubcommand),
    }
}

fn parse_optimize_args(args: &[String]) -> Result<OptimizeCli, String> {
    let mut character = None;
    let mut file = None;
    let mut builds_file = None;
    let mut build = None;
    let mut save_build = None;
    let mut to_file = false;
    let mut goals = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();
        if option_matches(arg, &["--character", "-c"]) {
            i += 1;
            character = Some(args.get(i).ok_or("--character requires a value")?.clone());
        } else if option_matches(arg, &["--file", "-f"]) {
            i += 1;
            file = Some(PathBuf::from(args.get(i).ok_or("--file requires a path")?));
        } else if option_matches(arg, &["--builds-file"]) {
            i += 1;
            builds_file = Some(PathBuf::from(
                args.get(i).ok_or("--builds-file requires a path")?,
            ));
        } else if option_matches(arg, &["--build"]) {
            i += 1;
            build = Some(args.get(i).ok_or("--build requires a value")?.clone());
        } else if option_matches(arg, &["--save-build"]) {
            i += 1;
            save_build = Some(args.get(i).ok_or("--save-build requires a value")?.clone());
        } else if option_matches(arg, &["--to-file"]) {
            to_file = true;
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: '{}'", arg));
        } else {
            let goal: StatGoal = arg
                .parse()
                .map_err(|e| format!("Invalid stat goal '{}': {}", arg, e))?;
            goals.push(goal);
        }
        i += 1;
    }

    if build.is_some() && !goals.is_empty() {
        return Err("Cannot supply stat goals when using --build <name>.".to_string());
    }
    if save_build.is_some() && goals.is_empty() && build.is_none() {
        return Err("--save-build requires at least one stat goal.".to_string());
    }

    Ok(OptimizeCli {
        character,
        file,
        builds_file,
        build,
        save_build,
        to_file,
        goals,
    })
}

fn parse_scrap_gear_args(args: &[String]) -> Result<ScrapGearCli, String> {
    let mut character = None;
    let mut file = None;
    let mut builds_file = None;
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();
        if option_matches(arg, &["--character", "-c"]) {
            i += 1;
            character = Some(args.get(i).ok_or("--character requires a value")?.clone());
        } else if option_matches(arg, &["--file", "-f"]) {
            i += 1;
            file = Some(PathBuf::from(args.get(i).ok_or("--file requires a path")?));
        } else if option_matches(arg, &["--builds-file"]) {
            i += 1;
            builds_file = Some(PathBuf::from(
                args.get(i).ok_or("--builds-file requires a path")?,
            ));
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: '{}'", arg));
        } else {
            return Err("'scrap-gear' takes no positional arguments".to_string());
        }
        i += 1;
    }

    Ok(ScrapGearCli {
        character,
        file,
        builds_file,
    })
}

fn parse_base_stats_args(args: &[String]) -> Result<BaseStatsCli, String> {
    let mut character = None;
    let mut file = None;
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();
        if option_matches(arg, &["--character", "-c"]) {
            i += 1;
            character = Some(args.get(i).ok_or("--character requires a value")?.clone());
        } else if option_matches(arg, &["--file", "-f"]) {
            i += 1;
            file = Some(PathBuf::from(args.get(i).ok_or("--file requires a path")?));
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: '{}'", arg));
        } else {
            return Err("'base-stats' takes no positional arguments".to_string());
        }
        i += 1;
    }

    Ok(BaseStatsCli { character, file })
}

fn parse_resolve_slots_args(args: &[String]) -> Result<ResolveSlotsCli, String> {
    let mut character = None;
    let mut force = false;
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();
        if option_matches(arg, &["--character", "-c"]) {
            i += 1;
            character = Some(args.get(i).ok_or("--character requires a value")?.clone());
        } else if option_matches(arg, &["--force", "-f"]) {
            force = true;
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: '{}'", arg));
        } else {
            return Err("'resolve-slots' takes no positional arguments".to_string());
        }
        i += 1;
    }

    Ok(ResolveSlotsCli { character, force })
}

fn parse_build_db_args(args: &[String]) -> Result<BuildDbCli, String> {
    let mut items: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();
        if option_matches(arg, &["--items"]) {
            i += 1;
            items = Some(PathBuf::from(args.get(i).ok_or("--items requires a path")?));
        } else if option_matches(arg, &["--out"]) {
            i += 1;
            out = Some(PathBuf::from(args.get(i).ok_or("--out requires a path")?));
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: '{}'", arg));
        } else {
            return Err("'build-db' takes no positional arguments".to_string());
        }
        i += 1;
    }

    let items = match items {
        Some(path) => path,
        None => install::data_path("items.xml")
            .map_err(|e| format!("cannot resolve the default data/items.xml path: {}", e))?,
    };
    let out = match out {
        Some(path) => path,
        None => install::data_path("lgo_items.json")
            .map_err(|e| format!("cannot resolve the default data/lgo_items.json path: {}", e))?,
    };

    Ok(BuildDbCli { items, out })
}

fn option_matches(arg: &str, options: &[&str]) -> bool {
    options
        .iter()
        .any(|expected| arg.eq_ignore_ascii_case(expected))
}

fn documents_dir() -> Result<PathBuf, String> {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let docs = PathBuf::from(profile).join("Documents");
        if docs.exists() {
            return Ok(docs);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let docs = PathBuf::from(home).join("Documents");
        if docs.exists() {
            return Ok(docs);
        }
    }
    std::env::current_dir().map_err(|e| format!("Cannot determine working directory: {}", e))
}

fn format_stat_list() -> String {
    let mut output = String::new();
    writeln!(&mut output, "{:<17}  Stat Name", "Stat Abbreviation")
        .expect("writing to String cannot fail");
    writeln!(
        &mut output,
        "{:<17}  ------------------",
        "-----------------"
    )
    .expect("writing to String cannot fail");

    for (stat, _) in stat::TRACKED_STATS {
        let abbreviation = stat::abbreviation_for(*stat)
            .expect("every TRACKED_STATS entry must have an abbreviation");
        writeln!(&mut output, "     {:<12}  {}", abbreviation, stat)
            .expect("writing to String cannot fail");
    }

    output
}

fn print_stat_list() {
    print!("{}", format_stat_list());
}

fn print_usage() {
    println!("LGO - Thalya's LOTRO Gear Optimizer");
    println!();
    println!("Usage:");
    println!("  lgo optimize      [options] <stat:min> [<stat:min> ...]");
    println!("  lgo optimize      [options] --build <name>");
    println!("  lgo scrap-gear    [options]");
    println!("  lgo base-stats    [options]");
    println!("  lgo resolve-slots [options]");
    println!("  lgo build-db      [options]");
    println!("  lgo --statlist");
    println!("  lgo --help | -h | help");
    println!();
    println!("Top-level commands:");
    println!("  --statlist          Print stat abbreviations and full stat names");
    println!();
    println!("Options (optimize):");
    println!("  --character <name>  Character name; selects the <name>_Gear folder");
    println!("                      (auto-detected when only one exists)");
    println!(
        "  --file      <path>  Explicit canonical gear TOML to optimize instead of auto-detect"
    );
    println!("  --builds-file <path> Explicit saved-builds TOML path (overrides discovery)");
    println!("  --build     <name>  Load saved goals from lgo_<character>_builds.toml");
    println!("  --save-build <name> Save these goals to lgo_<character>_builds.toml");
    println!("  --to-file            Also write .txt and .html reports for this run");
    println!();
    println!("  `--build` may not be combined with positional goals.");
    println!();
    println!("  All paths resolve from the install directory (the folder containing lgo.exe).");
    println!("  Characters live in <install>\\<CharacterName>_Gear folders; when several");
    println!("  exist the most recently updated is chosen and announced. optimize requires");
    println!("  data\\base_stat_derivations.json in <install>\\data to derive Base-stat");
    println!("  contributions before optimization. If [Virtues] contains any non-empty");
    println!("  selections, optimize also resolves them against data\\lgo_virtues.json.");
    println!("  By default optimize prints only to the terminal; add --to-file to also write");
    println!("  matching .txt and .html reports into <install>\\<CharacterName>_Gear\\");
    println!("  <CharacterName>_Reports.");
    println!();
    println!("Options (scrap-gear):");
    println!("  --character <name>  Character name; selects the <name>_Gear folder");
    println!("                      (auto-detected when only one exists)");
    println!(
        "  --file      <path>  Explicit canonical gear TOML whose sibling builds file will be used"
    );
    println!("  --builds-file <path> Explicit saved-builds TOML path (overrides discovery)");
    println!();
    println!("  scrap-gear reruns optimize once per saved build and lists items not used");
    println!("  in any saved build. Save builds with:");
    println!("    lgo optimize --save-build <name> <stat:min> [<stat:min> ...]");
    println!();
    println!("Options (base-stats):");
    println!("  --character <name>  Character name; selects the <name>_Gear folder");
    println!("                      (auto-detected when only one exists)");
    println!("  --file      <path>  Explicit canonical gear TOML to read instead of auto-detect");
    println!();
    println!("  base-stats prints the five raw innate Base stats from [InnateStats] and the");
    println!("  tracked-stat contributions derived from them (already included in optimize");
    println!("  totals). It does not run an optimization.");
    println!();
    println!("Options (resolve-slots):");
    println!("  --character <name>  Character name; selects or creates the <name>_Gear folder");
    println!("                      (auto-detected when only one exists)");
    println!("  --force, -f         Prompt per item before overwriting or removing entries");
    println!("                      in the canonical gear file. Without --force, existing");
    println!("                      entries are preserved on every iteration.");
    println!();
    println!("  resolve-slots creates <install>\\<CharacterName>_Gear if needed. A");
    println!("  lgo_<character>_gearStats.toml left loose in the install root is moved into");
    println!("  its character folder automatically before resolving.");
    println!();
    println!("Options (build-db):");
    println!("  --items        <path>  Items XML  (default: <install>\\data\\items.xml)");
    println!("  --out          <path>  Output JSON  (default: <install>\\data\\lgo_items.json)");
    println!();
    println!("Workflow:");
    println!("  1) Place candidate items in a Shared Storage chest named 'lgo'");
    println!("  2) Run /lgo export in-game");
    println!("  3) Navigate to https://lotro-wiki.com in your browser");
    println!("  4) Click the LGO bookmarklet");
    println!("  5) Paste the contents of lgo_<character-name>_gearNames_<timestamp>.plugindata when prompted");
    println!(
        "  6) Save the generated lgo_<character>_gearStats.toml into <install>\\<CharacterName>_Gear"
    );
    println!("     (or the install root — resolve-slots will move it into the folder)");
    println!("  7) Run: lgo resolve-slots   (writes lgo_<character>_gearReady.toml)");
    println!("  8) Run: lgo optimize <stat:min> [<stat:min> ...]");
    println!("     Optional: add --save-build <name> to save those goals for later");
    println!("  9) Run: lgo scrap-gear   (reruns all saved builds and lists items not used");
    println!("     in any saved build)");
    println!();
    println!("Stat goals:");
    println!("  Each goal is a stat name and a minimum value, separated by ':'.");
    println!("  Goals are listed in priority order.");
    println!("  The optimizer first meets higher-priority goals, then gets");
    println!("  still-unmet goals as close to target as possible.");
    println!("  A minimum of 0 means 'no floor, but maximise as later polish'.");
    println!();
    println!("  Examples:");
    println!("    lgo optimize TacticalMastery:450000 CriticalRating:350000 Finesse:0");
    println!("    lgo optimize tm:450000 cr:350000 fn:0");
    println!("    lgo optimize --character Thalya tm:450000 oh:100000");
    println!("    lgo optimize --save-build healer oh:200000 cr:350000 ml:0");
    println!("    lgo optimize --to-file tm:450000 cr:350000 fn:0");
    println!("    lgo optimize --build healer");
    println!("    lgo optimize --file path/to/lgo_Thalya_gearReady.toml tm:450000 cr:350000");
    println!("    lgo scrap-gear");
    println!("    lgo scrap-gear --file path/to/lgo_Thalya_gearReady.toml");
    println!("    lgo base-stats");
    println!("    lgo base-stats --character Thalya");
    println!("    lgo resolve-slots");
    println!("    lgo resolve-slots --force");
    println!("    lgo build-db");
    println!("    lgo build-db --items data/items.xml --out data/lgo_items.json");
}

fn slot_order_index(slot: gear::Slot) -> usize {
    gear::Slot::all()
        .position(|candidate| candidate == slot)
        .unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn help_tokens_parse_as_help_command() {
        for token in ["--help", "-h", "help"] {
            let cmd = parse_command(&s(&[token])).expect("help token should parse");
            assert!(matches!(cmd, Command::Help));
        }
    }

    #[test]
    fn statlist_switch_parses_as_statlist_command() {
        let cmd = parse_command(&s(&["--statlist"])).expect("--statlist should parse");
        assert!(matches!(cmd, Command::StatList));
    }

    #[test]
    fn statlist_switch_is_case_insensitive() {
        let cmd = parse_command(&s(&["--StAtLiSt"])).expect("--StAtLiSt should parse");
        assert!(matches!(cmd, Command::StatList));
    }

    #[test]
    fn help_flag_after_any_verb_parses_as_help() {
        for cmdline in [
            vec!["optimize", "--help"],
            vec!["resolve-slots", "-h"],
            vec!["scrap-gear", "--help"],
            vec!["base-stats", "-h"],
            vec!["build-db", "--help"],
            vec!["optimize", "tm:1", "--help"],
        ] {
            let cmd = parse_command(&s(&cmdline)).expect("help should parse");
            assert!(matches!(cmd, Command::Help), "failed for {:?}", cmdline);
        }
    }

    #[test]
    fn statlist_output_contains_headers_and_canonical_pairs() {
        let output = format_stat_list();
        assert!(output.contains("Stat Abbreviation"));
        assert!(output.contains("Stat Name"));
        assert!(output.contains("ml"));
        assert!(output.contains("Morale"));
        assert!(output.contains("tm"));
        assert!(output.contains("Tactical Mastery"));
        assert!(output.contains("tt"));
        assert!(output.contains("Tactical Mitigation"));
    }

    #[test]
    fn optimize_verb_is_case_insensitive_across_aliases() {
        for verb in ["optimize", "Optimize", "OPTIMIZE", "--Optimize", "-O"] {
            let cmd = parse_command(&s(&[verb, "tm:1"])).expect("optimize alias should parse");
            match cmd {
                Command::Optimize(cli) => assert_eq!(cli.goals.len(), 1),
                _ => panic!("expected optimize command"),
            }
        }
    }

    #[test]
    fn base_stats_verb_is_case_insensitive_across_aliases() {
        for verb in ["base-stats", "Base-Stats", "BASE-STATS", "--Base-Stats"] {
            let cmd = parse_command(&s(&[verb])).expect("base-stats alias should parse");
            assert!(matches!(cmd, Command::BaseStats(_)));
        }
    }

    #[test]
    fn base_stats_accepts_character_and_file_flags() {
        let cmd = parse_command(&s(&[
            "base-stats",
            "--character",
            "Thalya",
            "--file",
            "my/gear.toml",
        ]))
        .expect("base-stats flags should parse");
        match cmd {
            Command::BaseStats(cli) => {
                assert_eq!(cli.character.as_deref(), Some("Thalya"));
                assert_eq!(cli.file, Some(PathBuf::from("my/gear.toml")));
            }
            _ => panic!("expected base-stats command"),
        }

        let cmd = parse_command(&s(&["base-stats", "-c", "Thalya", "-f", "gear.toml"]))
            .expect("base-stats short flags should parse");
        match cmd {
            Command::BaseStats(cli) => {
                assert_eq!(cli.character.as_deref(), Some("Thalya"));
                assert_eq!(cli.file, Some(PathBuf::from("gear.toml")));
            }
            _ => panic!("expected base-stats command"),
        }
    }

    #[test]
    fn base_stats_flags_are_case_insensitive_and_preserve_values() {
        let cmd = parse_command(&s(&[
            "base-stats",
            "--ChArAcTeR",
            "ThAlYa",
            "--FiLe",
            "MiXeD/Path.toml",
        ]))
        .expect("base-stats mixed-case flags should parse");
        match cmd {
            Command::BaseStats(cli) => {
                assert_eq!(cli.character.as_deref(), Some("ThAlYa"));
                assert_eq!(cli.file, Some(PathBuf::from("MiXeD/Path.toml")));
            }
            _ => panic!("expected base-stats command"),
        }

        let cmd = parse_command(&s(&[
            "base-stats",
            "-C",
            "CharName",
            "-F",
            "Case/Path.toml",
        ]))
        .expect("base-stats mixed-case short flags should parse");
        match cmd {
            Command::BaseStats(cli) => {
                assert_eq!(cli.character.as_deref(), Some("CharName"));
                assert_eq!(cli.file, Some(PathBuf::from("Case/Path.toml")));
            }
            _ => panic!("expected base-stats command"),
        }
    }

    #[test]
    fn base_stats_rejects_positional_arguments() {
        let err = parse_command(&s(&["base-stats", "tm:450000"])).unwrap_err();
        match err {
            CliParseError::Message(msg) => {
                assert_eq!(msg, "'base-stats' takes no positional arguments")
            }
            _ => panic!("expected message parse error"),
        }
    }

    #[test]
    fn resolve_slots_verb_is_case_insensitive_across_aliases() {
        for verb in [
            "resolve-slots",
            "Resolve-Slots",
            "RESOLVE-SLOTS",
            "--Resolve-Slots",
            "-R",
        ] {
            let cmd = parse_command(&s(&[verb])).expect("resolve alias should parse");
            assert!(matches!(cmd, Command::ResolveSlots(_)));
        }
    }

    #[test]
    fn old_style_goal_invocation_is_missing_subcommand_error() {
        let err = parse_command(&s(&["tm:450000", "cr:350000"])).unwrap_err();
        assert!(matches!(err, CliParseError::MissingSubcommand));
    }

    #[test]
    fn resolve_slots_rejects_positional_arguments() {
        let err = parse_command(&s(&["resolve-slots", "tm:450000"])).unwrap_err();
        match err {
            CliParseError::Message(msg) => {
                assert_eq!(msg, "'resolve-slots' takes no positional arguments")
            }
            _ => panic!("expected message parse error"),
        }
    }

    #[test]
    fn resolve_slots_accepts_shared_flags_only() {
        let cmd = parse_command(&s(&["resolve-slots", "--character", "Thalya"]))
            .expect("resolve-slots flags should parse");
        match cmd {
            Command::ResolveSlots(cli) => {
                assert_eq!(cli.character.as_deref(), Some("Thalya"));
                assert!(!cli.force);
            }
            _ => panic!("expected resolve-slots command"),
        }
    }

    #[test]
    fn resolve_slots_flags_are_case_insensitive_and_preserve_values() {
        let cmd = parse_command(&s(&["resolve-slots", "--ChArAcTeR", "ThAlYa", "--FoRcE"]))
            .expect("resolve-slots mixed-case flags should parse");
        match cmd {
            Command::ResolveSlots(cli) => {
                assert_eq!(cli.character.as_deref(), Some("ThAlYa"));
                assert!(cli.force);
            }
            _ => panic!("expected resolve-slots command"),
        }

        let cmd = parse_command(&s(&["resolve-slots", "-C", "RaWName", "-F"]))
            .expect("resolve-slots mixed-case short flags should parse");
        match cmd {
            Command::ResolveSlots(cli) => {
                assert_eq!(cli.character.as_deref(), Some("RaWName"));
                assert!(cli.force);
            }
            _ => panic!("expected resolve-slots command"),
        }
    }

    #[test]
    fn resolve_slots_accepts_force_flag() {
        for token in ["--force", "-f"] {
            let cmd = parse_command(&s(&["resolve-slots", token])).expect("must parse");
            match cmd {
                Command::ResolveSlots(cli) => {
                    assert!(cli.force, "force should be set for {}", token)
                }
                _ => panic!("expected resolve-slots"),
            }
        }
    }

    #[test]
    fn build_db_verb_is_case_insensitive_across_aliases() {
        for verb in ["build-db", "Build-Db", "BUILD-DB", "--Build-Db", "-B"] {
            let cmd = parse_command(&s(&[verb])).expect("build-db alias should parse");
            assert!(matches!(cmd, Command::BuildDb(_)));
        }
    }

    #[test]
    fn build_db_rejects_positional_arguments() {
        let err = parse_command(&s(&["build-db", "tm:450000"])).unwrap_err();
        match err {
            CliParseError::Message(msg) => {
                assert_eq!(msg, "'build-db' takes no positional arguments")
            }
            _ => panic!("expected message parse error"),
        }
    }

    #[test]
    fn build_db_defaults_are_correct() {
        let cmd = parse_command(&s(&["build-db"])).expect("build-db should parse with no args");
        match cmd {
            Command::BuildDb(cli) => {
                // Defaults resolve under the install directory's data/ folder.
                assert_eq!(
                    cli.items,
                    install::data_path("items.xml").expect("install data path")
                );
                assert_eq!(
                    cli.out,
                    install::data_path("lgo_items.json").expect("install data path")
                );
            }
            _ => panic!("expected build-db command"),
        }
    }

    #[test]
    fn build_db_accepts_path_flags() {
        let cmd = parse_command(&s(&[
            "build-db",
            "--items",
            "my/items.xml",
            "--out",
            "my/out.json",
        ]))
        .expect("build-db path flags should parse");
        match cmd {
            Command::BuildDb(cli) => {
                assert_eq!(cli.items, PathBuf::from("my/items.xml"));
                assert_eq!(cli.out, PathBuf::from("my/out.json"));
            }
            _ => panic!("expected build-db command"),
        }
    }

    #[test]
    fn build_db_flags_are_case_insensitive_and_preserve_values() {
        let cmd = parse_command(&s(&[
            "build-db",
            "--ItEmS",
            "MiXeD/Input.xml",
            "--OuT",
            "MiXeD/Output.json",
        ]))
        .expect("build-db mixed-case flags should parse");
        match cmd {
            Command::BuildDb(cli) => {
                assert_eq!(cli.items, PathBuf::from("MiXeD/Input.xml"));
                assert_eq!(cli.out, PathBuf::from("MiXeD/Output.json"));
            }
            _ => panic!("expected build-db command"),
        }
    }

    #[test]
    fn optimize_file_flag_sets_toml_path() {
        let cmd = parse_command(&s(&["optimize", "--file", "some.toml", "tm:1"]))
            .expect("optimize --file should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert_eq!(cli.file, Some(PathBuf::from("some.toml")));
                assert!(cli.builds_file.is_none());
                assert!(cli.build.is_none());
                assert!(cli.save_build.is_none());
                assert!(!cli.to_file);
                assert_eq!(cli.goals.len(), 1);
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn optimize_file_short_flag_sets_toml_path() {
        let cmd = parse_command(&s(&["optimize", "-f", "gear.toml", "tm:1"]))
            .expect("optimize -f should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert_eq!(cli.file, Some(PathBuf::from("gear.toml")));
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn optimize_file_and_character_can_be_combined() {
        let cmd = parse_command(&s(&[
            "optimize",
            "--file",
            "my/gear.toml",
            "--character",
            "Thalya",
            "tm:1",
        ]))
        .expect("optimize --file + --character should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert_eq!(cli.file, Some(PathBuf::from("my/gear.toml")));
                assert_eq!(cli.character.as_deref(), Some("Thalya"));
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn optimize_accepts_save_build_flag() {
        let cmd = parse_command(&s(&[
            "optimize",
            "--save-build",
            "healer",
            "--builds-file",
            "builds.toml",
            "oh:200000",
            "cr:350000",
        ]))
        .expect("optimize --save-build should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert_eq!(cli.save_build.as_deref(), Some("healer"));
                assert_eq!(cli.builds_file, Some(PathBuf::from("builds.toml")));
                assert!(!cli.to_file);
                assert_eq!(cli.goals.len(), 2);
                assert!(cli.build.is_none());
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn optimize_accepts_to_file_flag() {
        let cmd = parse_command(&s(&["optimize", "--to-file", "tm:1"]))
            .expect("optimize --to-file should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert!(cli.to_file);
                assert_eq!(cli.goals.len(), 1);
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn optimize_to_file_can_be_combined_with_other_flags() {
        let cmd = parse_command(&s(&[
            "optimize",
            "--file",
            "gear.toml",
            "--save-build",
            "healer",
            "--to-file",
            "tm:1",
        ]))
        .expect("optimize combined flags should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert_eq!(cli.file, Some(PathBuf::from("gear.toml")));
                assert_eq!(cli.save_build.as_deref(), Some("healer"));
                assert!(cli.to_file);
                assert_eq!(cli.goals.len(), 1);
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn optimize_flags_are_case_insensitive_and_preserve_values() {
        let cmd = parse_command(&s(&[
            "optimize",
            "--ChArAcTeR",
            "ThAlYa",
            "--FiLe",
            "MiXeD/Path.toml",
            "--BuIlDs-FiLe",
            "BuIlDs/MiXeD.toml",
            "--SaVe-BuIlD",
            "HeAlEr Build",
            "--to-File",
            "tm:1",
        ]))
        .expect("optimize mixed-case flags should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert_eq!(cli.character.as_deref(), Some("ThAlYa"));
                assert_eq!(cli.file, Some(PathBuf::from("MiXeD/Path.toml")));
                assert_eq!(cli.builds_file, Some(PathBuf::from("BuIlDs/MiXeD.toml")));
                assert_eq!(cli.save_build.as_deref(), Some("HeAlEr Build"));
                assert!(cli.to_file);
                assert_eq!(cli.goals.len(), 1);
            }
            _ => panic!("expected optimize command"),
        }

        let cmd = parse_command(&s(&[
            "optimize",
            "-C",
            "MiXeDName",
            "-F",
            "Case/Path.toml",
            "tm:1",
        ]))
        .expect("optimize mixed-case short flags should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert_eq!(cli.character.as_deref(), Some("MiXeDName"));
                assert_eq!(cli.file, Some(PathBuf::from("Case/Path.toml")));
                assert_eq!(cli.goals.len(), 1);
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn optimize_accepts_build_flag_without_goals() {
        let cmd = parse_command(&s(&[
            "optimize",
            "--build",
            "Healer",
            "--save-build",
            "Copy",
        ]))
        .expect("optimize --build should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert_eq!(cli.build.as_deref(), Some("Healer"));
                assert_eq!(cli.save_build.as_deref(), Some("Copy"));
                assert!(cli.goals.is_empty());
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn optimize_build_flag_is_case_insensitive_and_preserves_build_name() {
        let cmd = parse_command(&s(&[
            "optimize",
            "--BuIlD",
            "HeAlEr Build",
            "--BuIlDs-FiLe",
            "MiXeD/Builds.toml",
            "--SaVe-BuIlD",
            "CoPy Build",
        ]))
        .expect("optimize mixed-case build flags should parse");
        match cmd {
            Command::Optimize(cli) => {
                assert_eq!(cli.build.as_deref(), Some("HeAlEr Build"));
                assert_eq!(cli.builds_file, Some(PathBuf::from("MiXeD/Builds.toml")));
                assert_eq!(cli.save_build.as_deref(), Some("CoPy Build"));
                assert!(cli.goals.is_empty());
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn optimize_rejects_build_with_goals() {
        let err = parse_command(&s(&["optimize", "--build", "healer", "oh:1"]))
            .expect_err("must reject goals with --build");
        match err {
            CliParseError::Message(msg) => {
                assert_eq!(msg, "Cannot supply stat goals when using --build <name>.")
            }
            _ => panic!("expected message parse error"),
        }
    }

    #[test]
    fn optimize_rejects_save_build_without_goals() {
        let err = parse_command(&s(&["optimize", "--save-build", "healer"]))
            .expect_err("must reject save-build without goals");
        match err {
            CliParseError::Message(msg) => {
                assert_eq!(msg, "--save-build requires at least one stat goal.")
            }
            _ => panic!("expected message parse error"),
        }
    }

    #[test]
    fn optimize_without_file_has_none_file() {
        let cmd =
            parse_command(&s(&["optimize", "tm:1"])).expect("optimize without --file must parse");
        match cmd {
            Command::Optimize(cli) => {
                assert!(cli.file.is_none(), "--file must be None when not supplied");
            }
            _ => panic!("expected optimize command"),
        }
    }

    #[test]
    fn scrap_gear_verb_is_case_insensitive_across_aliases() {
        for verb in [
            "scrap-gear",
            "Scrap-Gear",
            "SCRAP-GEAR",
            "scrapgear",
            "--Scrap-Gear",
            "-S",
        ] {
            let cmd = parse_command(&s(&[verb])).expect("scrap-gear alias should parse");
            assert!(matches!(cmd, Command::ScrapGear(_)));
        }
    }

    #[test]
    fn scrap_gear_accepts_character_and_file_flags() {
        let cmd = parse_command(&s(&[
            "scrap-gear",
            "--character",
            "Thalya",
            "--file",
            "gear.toml",
            "--builds-file",
            "builds.toml",
        ]))
        .expect("scrap-gear flags should parse");
        match cmd {
            Command::ScrapGear(cli) => {
                assert_eq!(cli.character.as_deref(), Some("Thalya"));
                assert_eq!(cli.file, Some(PathBuf::from("gear.toml")));
                assert_eq!(cli.builds_file, Some(PathBuf::from("builds.toml")));
            }
            _ => panic!("expected scrap-gear command"),
        }
    }

    #[test]
    fn scrap_gear_flags_are_case_insensitive_and_preserve_values() {
        let cmd = parse_command(&s(&[
            "scrap-gear",
            "--ChArAcTeR",
            "ThAlYa",
            "--FiLe",
            "MiXeD/Gear.toml",
            "--BuIlDs-FiLe",
            "MiXeD/Builds.toml",
        ]))
        .expect("scrap-gear mixed-case flags should parse");
        match cmd {
            Command::ScrapGear(cli) => {
                assert_eq!(cli.character.as_deref(), Some("ThAlYa"));
                assert_eq!(cli.file, Some(PathBuf::from("MiXeD/Gear.toml")));
                assert_eq!(cli.builds_file, Some(PathBuf::from("MiXeD/Builds.toml")));
            }
            _ => panic!("expected scrap-gear command"),
        }

        let cmd = parse_command(&s(&["scrap-gear", "-C", "Name", "-F", "Case/Path.toml"]))
            .expect("scrap-gear mixed-case short flags should parse");
        match cmd {
            Command::ScrapGear(cli) => {
                assert_eq!(cli.character.as_deref(), Some("Name"));
                assert_eq!(cli.file, Some(PathBuf::from("Case/Path.toml")));
            }
            _ => panic!("expected scrap-gear command"),
        }
    }

    #[test]
    fn scrap_gear_rejects_positional_arguments() {
        let err = parse_command(&s(&["scrap-gear", "extra"])).expect_err("must reject positional");
        match err {
            CliParseError::Message(msg) => {
                assert_eq!(msg, "'scrap-gear' takes no positional arguments")
            }
            _ => panic!("expected message parse error"),
        }
    }

    #[test]
    fn extracts_character_segment_from_canonical_gear_filename() {
        assert_eq!(
            extract_character_segment_from_canonical_gear_filename(Path::new(
                "/tmp/lgo_Thalya_gearReady.toml"
            )),
            Some("Thalya".to_string())
        );
        assert_eq!(
            extract_character_segment_from_canonical_gear_filename(Path::new(
                "/tmp/lgo_THALYA_GEARREADY.toml"
            )),
            Some("THALYA".to_string())
        );
        assert_eq!(
            extract_character_segment_from_canonical_gear_filename(Path::new("/tmp/gear.toml")),
            None
        );
    }

    #[test]
    fn report_character_fallback_precedence_matches_optimize_logic() {
        assert_eq!(
            resolve_report_character(
                Some("FromToml".to_string()),
                Some("FromCli"),
                Some("AutoDiscovered".to_string())
            ),
            "FromToml"
        );
        assert_eq!(
            resolve_report_character(None, Some("FromCli"), Some("AutoDiscovered".to_string())),
            "FromCli"
        );
        assert_eq!(
            resolve_report_character(None, None, Some("AutoDiscovered".to_string())),
            "AutoDiscovered"
        );
        assert_eq!(resolve_report_character(None, None, None), "Unknown");
    }
}
