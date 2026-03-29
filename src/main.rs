//! LGO — LOTRO Gear Optimizer
//!
//! Usage:
//!   lgo [--character <name>] [--cache <path>] <stat:minimum> [<stat:minimum> ...]
//!
//! Examples:
//!   lgo CritRating:450 TacticalMastery:450 FinesseRating:300 TacticalMitigation:200
//!   lgo --character Thalya Vitality:500 Morale:800
//!
//! Stat names are case-insensitive and accept common aliases (e.g. TactMast,
//! physmit, critrating). See stat.rs for the full alias list.
//!
//! The plugin data file is discovered automatically from:
//!   Documents\The Lord of the Rings Online\PluginData\<character>\AllServers\
//! The most recent lgo_export_*.plugindata file is used.

mod cache;
mod gear;
mod optimizer;
mod plugindata;
mod report;
mod stat;
mod wiki;

use std::path::{Path, PathBuf};
use std::process;

use cache::Cache;
use stat::StatGoal;

// ?? Entry point ???????????????????????????????????????????????????????????????

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

    if cli.goals.is_empty() {
        eprintln!("Error: at least one stat goal is required.");
        eprintln!("Run with --help for usage.");
        process::exit(1);
    }

    // Discover the plugindata file.
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

    // Load the cache.
    let cache_path = cli.cache_path
        .clone()
        .unwrap_or_else(|| cache::default_cache_path(plugindata_path.parent()));
    let mut item_cache = match Cache::load(&cache_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: could not load cache ({}); starting empty.", e);
            Cache::load(&cache_path).unwrap_or_else(|_| {
                // If the path is unreadable and non-existent, create fresh.
                Cache::load(Path::new("/dev/null")).unwrap_or_else(|_| {
                    panic!("Failed to initialise cache")
                })
            })
        }
    };

    // Collect all item names that need wiki resolution.
    let equipped_names: Vec<String> = export.equipped.iter()
        .map(|i| i.name.clone())
        .collect();
    let candidate_names: Vec<String> = export.candidates.iter()
        .map(|i| i.name.clone())
        .collect();

    let mut all_names = equipped_names.clone();
    for n in &candidate_names {
        if !all_names.contains(n) {
            all_names.push(n.clone());
        }
    }

    // Resolve items via cache + wiki.
    let resolved = wiki::resolve_items(&all_names, &mut item_cache);

    // Report any items that could not be resolved.
    for name in &all_names {
        if !resolved.contains_key(name) {
            eprintln!("[lgo] WARN: '{}' could not be resolved — skipped.", name);
        }
    }

    // Flush cache after wiki lookups.
    if let Err(e) = item_cache.flush() {
        eprintln!("[lgo] Warning: could not save cache: {}", e);
    }

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

// ?? CLI parsing ???????????????????????????????????????????????????????????????

struct Cli {
    character:  Option<String>,
    cache_path: Option<PathBuf>,
    goals:      Vec<StatGoal>,
}

fn parse_args(args: &[String]) -> Result<Cli, String> {
    let mut character  = None;
    let mut cache_path = None;
    let mut goals      = Vec::new();
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

    Ok(Cli { character, cache_path, goals })
}

// ?? File discovery ????????????????????????????????????????????????????????????

fn resolve_plugindata(cli: &Cli) -> Result<(PathBuf, String), String> {
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

    // Determine character name.
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

    // Find the most recent lgo_export_*.plugindata file.
    let path = find_latest_export(&char_dir)?;

    Ok((path, character))
}

/// Find the most recent `lgo_export_*.plugindata` file in `dir`.
/// Since the filename contains a timestamp in YYYYMMDD_HHMMSS format,
/// lexicographic ordering gives chronological ordering.
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

    // Sort lexicographically — timestamp format means latest is last.
    entries.sort();
    Ok(entries.into_iter().last().unwrap())
}

/// If no character is specified, scan the PluginData directory.
/// If exactly one character subdirectory exists, use it.
/// If multiple exist, list them and ask the user to specify.
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

/// Return the path to the user's Documents folder.
/// On Windows this is typically C:\Users\<name>\Documents.
fn documents_dir() -> Result<PathBuf, String> {
    // Try the USERPROFILE environment variable first (Windows).
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let docs = PathBuf::from(profile).join("Documents");
        if docs.exists() {
            return Ok(docs);
        }
    }

    // Try HOME (for cross-platform development / WSL testing).
    if let Ok(home) = std::env::var("HOME") {
        let docs = PathBuf::from(home).join("Documents");
        if docs.exists() {
            return Ok(docs);
        }
    }

    // Last resort: current directory (useful for testing).
    std::env::current_dir()
        .map_err(|e| format!("Cannot determine working directory: {}", e))
}

// ?? Usage ?????????????????????????????????????????????????????????????????????

fn print_usage() {
    println!("LGO — LOTRO Gear Optimizer");
    println!();
    println!("Usage:");
    println!("  lgo [options] <stat:minimum> [<stat:minimum> ...]");
    println!();
    println!("Options:");
    println!("  --character <name>   Character name (auto-detected if only one exists)");
    println!("  --cache <path>       Path to the item cache JSON file");
    println!("  --help               Show this message");
    println!();
    println!("Stat goals:");
    println!("  Each goal is a stat name and a minimum value, separated by ':'.");
    println!("  Goals are listed in priority order — the first stat is maximised");
    println!("  first, with later stats used only as tiebreakers.");
    println!("  A minimum of 0 means 'maximise but no floor required'.");
    println!();
    println!("  Examples:");
    println!("    lgo CritRating:450 TacticalMastery:450 FinesseRating:300");
    println!("    lgo --character Thalya Vitality:500 Morale:800 CritRating:0");
    println!();
    println!("Stat name aliases (case-insensitive):");
    println!("  TactMast / TacticalMastery    PhysMast / PhysicalMastery");
    println!("  TactMit  / TacticalMitigation PhysMit  / PhysicalMitigation");
    println!("  CritRating                    DevRating");
    println!("  Finesse  / FinesseRating      Armour / Armor");
    println!("  CritDefence / CritDefense     IncMit / IncMitigations");
    println!("  Vitality  Morale  Power  Might  Agility  Will  Fate");
    println!("  IncomingHealing / IncHeal     OutgoingHealing / OutHeal");
    println!();
    println!("Workflow:");
    println!("  1) Place candidate items in a Shared Storage chest named 'lgo'");
    println!("  2) Run /lgo export in-game");
    println!("  3) Run this program with your stat goals");
}