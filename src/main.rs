//! LGO - LOTRO Gear Optimizer
//!
//! Usage:
//!   lgo --stats [options]                  Generate editable gear stats file
//!   lgo [options] <stat:minimum> [...]     Run optimizer
//!
//! The plugin export file and gear stats file are discovered automatically from:
//!   Documents\The Lord of the Rings Online\PluginData\<character>\AllServers\

mod cache;
mod db;
mod gear;
mod gearstats;
mod optimizer;
mod plugindata;
mod report;
mod stat;
mod wiki;

use std::path::{Path, PathBuf};
use std::process;

use cache::Cache;
use stat::StatGoal;

// -- Entry point ---------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        process::exit(0);
    }

    // Parse CLI arguments.
    let cli = match parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Run with --help for usage.");
            process::exit(1);
        }
    };

    // Discover the plugindata file and character name.
    let (plugindata_path, character) = match resolve_plugindata(&cli) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    eprintln!("[lgo] Using file : {}", plugindata_path.display());
    eprintln!("[lgo] Character  : {}", character);

    // Parse the plugin export.
    let export = match plugindata::load(&plugindata_path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading plugin data: {}", e);
            process::exit(1);
        }
    };

    let char_dir = plugindata_path.parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    // -- Stats-file generation mode --------------------------------------------
    // If --stats was passed: resolve items, write the gear stats file, exit.
    if cli.write_stats {
        let items = resolve_to_cached_items(&export, &char_dir, &cli);
        let timestamp = gearstats::now_timestamp();
        let out_path = gearstats::stats_file_path(&char_dir, &character, &timestamp);
        match gearstats::write_stats_file(&items, &out_path, &character) {
            Ok(()) => {
                println!("[lgo] Gear stats file written: {}", out_path.display());
                println!("[lgo] Edit it as needed, then run: lgo <stat:minimum> ...");
            }
            Err(e) => {
                eprintln!("Error writing gear stats file: {}", e);
                process::exit(1);
            }
        }
        process::exit(0);
    }

    // -- Optimizer mode --------------------------------------------------------
    if cli.goals.is_empty() {
        eprintln!("Error: at least one stat goal is required.");
        eprintln!("Run with --help for usage.");
        process::exit(1);
    }

    // Determine item stats source:
    //   1. Explicit --stats-file path
    //   2. Auto-detected most recent lgo_stats_*.toml in AllServers
    //   3. db / wiki / cache pipeline
    let resolved: std::collections::HashMap<String, cache::CachedItem> =
        if let Some(sf_path) = stats_file_to_use(&cli, &char_dir, &character) {
            eprintln!("[lgo] Using gear stats file — skipping db/wiki/cache lookup");
            eprintln!("[lgo] Stats file: {}", sf_path.display());
            match gearstats::read_stats_file(&sf_path) {
                Ok(items) => items.into_iter().map(|i| (i.name.clone(), i)).collect(),
                Err(e) => {
                    eprintln!("Error reading gear stats file: {}", e);
                    process::exit(1);
                }
            }
        } else {
            resolve_to_cached_items(&export, &char_dir, &cli)
                .into_iter()
                .map(|i| (i.name.clone(), i))
                .collect()
        };

    let equipped_names: Vec<String> = export.equipped.iter()
        .map(|i| i.name.clone())
        .collect();
    let candidate_names: Vec<String> = export.candidates.iter()
        .map(|i| i.name.clone())
        .collect();

    // Run the optimizer.
    let result = optimizer::optimize(
        &resolved,
        &equipped_names,
        &candidate_names,
        &cli.goals,
    );

    // Print the report.
    report::print_report(
        &result,
        &cli.goals,
        &character,
        &plugindata_path.display().to_string(),
    );

    // Exit with a non-zero code if infeasible, so shell scripts can detect it.
    if !result.feasible {
        process::exit(2);
    }
}

// -- Shared item resolution ----------------------------------------------------

/// Resolve all items in the export via db/wiki/cache and return as a Vec
/// in slot order: equipped items first (one per slot, preserving duplicates),
/// then candidates (deduped by name).
fn resolve_to_cached_items(
    export:   &plugindata::PluginExport,
    char_dir: &Path,
    cli:      &Cli,
) -> Vec<cache::CachedItem> {
    // Load the cache.
    let cache_path = cli.cache_path
        .clone()
        .unwrap_or_else(|| cache::default_cache_path(Some(char_dir)));
    let mut item_cache = match Cache::load(&cache_path) {
        Ok(c)  => c,
        Err(e) => {
            eprintln!("Warning: could not load cache ({}); starting empty.", e);
            Cache::empty(&cache_path)
        }
    };

    // Load the offline item database.
    let item_db = db::load_item_db(Path::new("data/lgo_items.json"))
        .unwrap_or_else(|e| {
            eprintln!("[lgo] Warning: could not load item db: {}", e);
            None
        });

    // Resolve unique names via db/wiki/cache.
    let mut unique_names: Vec<String> = Vec::new();
    for item in export.equipped.iter().chain(export.candidates.iter()) {
        if !unique_names.contains(&item.name) {
            unique_names.push(item.name.clone());
        }
    }
    let name_map = resolve_items(&unique_names, &item_db, &mut item_cache);

    // Flush cache after any wiki lookups.
    if let Err(e) = item_cache.flush() {
        eprintln!("[lgo] Warning: could not save cache: {}", e);
    }

    // Build output Vec in slot order, one entry per equipped slot.
    // Two slots with the same item name each get their own entry with the
    // correct slot — this is what was missing before.
    let mut items: Vec<cache::CachedItem> = Vec::new();

    for partial in &export.equipped {
        let slot = match partial.slot {
            Some(s) => s,
            None    => continue,
        };
        let stats = name_map.get(&partial.name)
            .map(|ci| ci.stats.clone())
            .unwrap_or_else(|| {
                eprintln!("[lgo] WARN: '{}' has no stats — manual entry required.", partial.name);
                std::collections::HashMap::new()
            });
        items.push(cache::CachedItem {
            name:  partial.name.clone(),
            slot,
            stats,
        });
    }

    // Append candidates (deduped by name, skipping those already equipped).
    let equipped_names: std::collections::HashSet<&String> =
        export.equipped.iter().map(|i| &i.name).collect();

    for partial in &export.candidates {
        if equipped_names.contains(&partial.name) { continue; }
        if let Some(ci) = name_map.get(&partial.name) {
            items.push(ci.clone());
        } else {
            eprintln!("[lgo] WARN: '{}' could not be resolved — skipped.", partial.name);
        }
    }

    items
}

/// Determine which gear stats file to use, if any.
/// Priority: explicit --stats-file flag > auto-detected most recent file.
fn stats_file_to_use(cli: &Cli, char_dir: &Path, character: &str) -> Option<PathBuf> {
    if let Some(ref p) = cli.stats_file {
        return Some(p.clone());
    }
    gearstats::find_latest_stats_file(char_dir, character)
}

// -- CLI parsing ---------------------------------------------------------------

struct Cli {
    character:   Option<String>,
    cache_path:  Option<PathBuf>,
    file:        Option<PathBuf>,
    stats_file:  Option<PathBuf>,
    write_stats: bool,
    goals:       Vec<StatGoal>,
}

fn parse_args(args: &[String]) -> Result<Cli, String> {
    let mut character   = None;
    let mut cache_path  = None;
    let mut file        = None;
    let mut stats_file  = None;
    let mut write_stats = false;
    let mut goals       = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--character" | "-c" => {
                i += 1;
                character = Some(args.get(i)
                    .ok_or("--character requires a value")?
                    .clone());
            }
            "--cache" => {
                i += 1;
                cache_path = Some(PathBuf::from(args.get(i)
                    .ok_or("--cache requires a path")?));
            }
            "--file" | "-f" => {
                i += 1;
                file = Some(PathBuf::from(args.get(i)
                    .ok_or("--file requires a path")?));
            }
            "--stats-file" | "-S" => {
                i += 1;
                stats_file = Some(PathBuf::from(args.get(i)
                    .ok_or("--stats-file requires a path")?));
            }
            "--stats" | "-s" => {
                write_stats = true;
            }
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option: '{}'", arg));
            }
            arg => {
                let goal: StatGoal = arg.parse()
                    .map_err(|e| format!("Invalid stat goal '{}': {}", arg, e))?;
                goals.push(goal);
            }
        }
        i += 1;
    }

    Ok(Cli { character, cache_path, file, stats_file, write_stats, goals })
}

// -- File discovery ------------------------------------------------------------

fn resolve_plugindata(cli: &Cli) -> Result<(PathBuf, String), String> {
    if let Some(path) = &cli.file {
        if !path.exists() {
            return Err(format!("File not found: {}", path.display()));
        }
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Could not read filename: {}", path.display()))?;
        let character = stem.strip_prefix("lgo_export_")
            .and_then(|s| s.rsplitn(2, '_').nth(1))
            .ok_or_else(|| format!(
                "Filename '{}' does not match expected pattern \
                 lgo_export_{{character}}_{{timestamp}}",
                stem
            ))?
            .to_string();
        return Ok((path.clone(), character));
    }

    let docs = documents_dir()?;
    let plugin_root = docs
        .join("The Lord of the Rings Online")
        .join("PluginData");

    if !plugin_root.exists() {
        return Err(format!(
            "PluginData directory not found: {}",
            plugin_root.display()
        ));
    }

    let character = match &cli.character {
        Some(c) => c.clone(),
        None    => discover_character(&plugin_root)?,
    };

    let char_dir = plugin_root.join(&character).join("AllServers");
    if !char_dir.exists() {
        return Err(format!(
            "Character directory not found: {}",
            char_dir.display()
        ));
    }

    ensure_lgo_dir(&char_dir)?;

    let path = find_latest_export(&char_dir)?;
    Ok((path, character))
}

fn ensure_lgo_dir(all_servers_dir: &Path) -> Result<PathBuf, String> {
    let lgo_dir = all_servers_dir.join("lgo");
    std::fs::create_dir_all(&lgo_dir).map_err(|e| {
        format!(
            "Cannot create lgo directory {}: {}",
            lgo_dir.display(),
            e
        )
    })?;
    Ok(lgo_dir)
}

fn find_latest_export(dir: &Path) -> Result<PathBuf, String> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory {}: {}", dir.display(), e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("plugindata")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("lgo_export_"))
                    .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        return Err(format!(
            "No lgo_export_*.plugindata files found in {}",
            dir.display()
        ));
    }

    entries.sort();
    Ok(entries.into_iter().last().unwrap())
}

fn discover_character(plugin_root: &Path) -> Result<String, String> {
    let dirs: Vec<String> = std::fs::read_dir(plugin_root)
        .map_err(|e| format!("Cannot read PluginData directory: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    match dirs.len() {
        0 => Err("No character directories found in PluginData.".to_string()),
        1 => Ok(dirs.into_iter().next().unwrap()),
        _ => {
            let mut msg = String::from(
                "Multiple characters found. Specify one with --character:\n"
            );
            for d in &dirs {
                msg.push_str(&format!("  {}\n", d));
            }
            Err(msg)
        }
    }
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
    std::env::current_dir()
        .map_err(|e| format!("Cannot determine working directory: {}", e))
}

// -- Item resolution -----------------------------------------------------------

fn resolve_items(
    names:      &[String],
    item_db:    &Option<std::collections::HashMap<String, cache::CachedItem>>,
    item_cache: &mut cache::Cache,
) -> std::collections::HashMap<String, cache::CachedItem> {
    let mut resolved: std::collections::HashMap<String, cache::CachedItem> =
        std::collections::HashMap::new();
    let mut wiki_names: Vec<String> = Vec::new();

    for name in names {
        if let Some(db) = item_db {
            if let Some(cached) = db.get(strip_level_suffix(name)) {
                resolved.insert(name.clone(), cached.clone());
                continue;
            }
        }
        wiki_names.push(name.clone());
    }

    if !wiki_names.is_empty() {
        let wiki_resolved = wiki::resolve_items(&wiki_names, item_cache);
        for (name, item) in wiki_resolved {
            resolved.insert(name, item);
        }
    }

    resolved
}

fn strip_level_suffix(name: &str) -> &str {
    if let Some(idx) = name.rfind(" (Level ") {
        if name[idx..].ends_with(')') {
            return &name[..idx];
        }
    }
    name
}

// -- Usage ---------------------------------------------------------------------

fn print_usage() {
    println!("LGO - LOTRO Gear Optimizer");
    println!();
    println!("Usage:");
    println!("  lgo --stats [options]                  Generate editable gear stats file");
    println!("  lgo [options] <stat:minimum> [...]     Run optimizer");
    println!();
    println!("Options:");
    println!("  --character  <name>   Character name (auto-detected if only one exists)");
    println!("  --file       <path>   Explicit path to lgo_export_*.plugindata");
    println!("  --stats-file <path>   Explicit path to lgo_stats_*.toml");
    println!("  --cache      <path>   Path to the item cache JSON file");
    println!("  --help                Show this message");
    println!();
    println!("Workflow:");
    println!("  1) Place candidate items in a Shared Storage chest named 'lgo'");
    println!("  2) Run /lgo export in-game");
    println!("  3) Run: lgo --stats");
    println!("  4) Edit the generated lgo_stats_*.toml file as needed");
    println!("  5) Run: lgo <stat:minimum> [<stat:minimum> ...]");
    println!();
    println!("Stat goals:");
    println!("  Each goal is a stat name and a minimum value, separated by ':'.");
    println!("  Goals are listed in priority order — the first stat is maximised");
    println!("  first, with later stats used only as tiebreakers.");
    println!("  A minimum of 0 means 'maximise but no floor required'.");
    println!();
    println!("  Examples:");
    println!("    lgo TacticalMastery:450000 CriticalRating:350000 Finesse:0");
    println!("    lgo tm:450000 cr:350000 fn:0");
    println!("    lgo --character Thalya tm:450000 oh:100000");
    println!();
    println!("Stat names (case-insensitive, full name or two-letter abbreviation):");
    println!("  am  Armor               cr  CriticalRating");
    println!("  fn  Finesse             pm  PhysicalMastery");
    println!("  tm  TacticalMastery     oh  OutgoingHealing");
    println!("  rs  Resistance          cd  CriticalDefense");
    println!("  ih  IncomingHealing     bl  Block");
    println!("  pa  Parry               ev  Evade");
    println!("  pt  PhysicalMitigation  tt  TacticalMitigation");
}
