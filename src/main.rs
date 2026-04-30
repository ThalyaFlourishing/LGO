//! LGO - LOTRO Gear Optimizer
//!
//! Usage:
//!   lgo --gearlist [options]               Generate editable gear stats file
//!   lgo [options] <stat:minimum> [...]     Run optimizer
//!   lgo --test [<filename>] [options] <stat:minimum> [...]
//!                                          Run optimizer using a test .toml from
//!                                          AllServers\lgo\test data\
//!
//! The plugin export file and gear stats file are discovered automatically from:
//!   Documents\The Lord of the Rings Online\PluginData\<character>\AllServers\

mod cache;
mod db;
mod gear;
mod gearstats;
mod merge;
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

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h" || a == "help") {
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

    if cli.write_stats {
        eprintln!("[lgo] Reading plugindata : {}", plugindata_path.display());
    }

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
    // If --gearlist was passed: resolve items, optionally merge with an
    // existing stats file to preserve hand-edits, write the result, then exit.
    if cli.write_stats {
        let new_items = resolve_to_cached_items(&export, &char_dir, &cli);
        let timestamp = gearstats::now_timestamp();
        let out_path  = gearstats::stats_file_path(&char_dir, &character, &timestamp);

        // Try to merge with the most recent existing stats file.
        let (final_items, maybe_edits) =
            if !cli.forget_edits {
                if let Some(old_path) = gearstats::find_latest_stats_file(&char_dir, &character) {
                    eprintln!("[lgo] Found existing stats file — merging to preserve hand-edits.");
                    eprintln!("[lgo] Existing file: {}", old_path.display());
                    match merge::read_merge_context(&old_path) {
                        Ok(ctx) => {
                            let (merged, edits) = merge::merge_stats(new_items, &ctx);
                            (merged, Some(edits))
                        }
                        Err(e) => {
                            eprintln!("[lgo] Warning: could not read existing stats file \
                                       for merge ({}); generating fresh file.", e);
                            (new_items, None)
                        }
                    }
                } else {
                    (new_items, None)
                }
            } else {
                eprintln!("[lgo] --forget-edits: generating fresh file, ignoring hand-edits.");
                (new_items, None)
            };

        match gearstats::write_stats_file(
            &final_items, &out_path, &character, maybe_edits.as_ref()
        ) {
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
    //   1. --test <file>  (test mode — bypasses all auto-detection)
    //   2. Explicit --stats-file path
    //   3. Auto-detected most recent lgo_stats_*.toml in AllServers
    //   4. db / wiki / cache pipeline

    // Validate test file exists before proceeding.
    if let Some(ref tf_name) = cli.test_stats_file {
        let tf = char_dir.join("lgo").join("test data").join(tf_name);
        if !tf.exists() {
            eprintln!(
                "Error: test stats file not found: {}",
                tf.display()
            );
            process::exit(1);
        }
    }

    let maybe_stats_file = stats_file_to_use(&cli, &char_dir, &character);

    // Bug 2 fix: emit the test-mode warning using the resolved path,
    // not a separately-recomputed one (which had an 'llgo' typo).
    if cli.test_stats_file.is_some() {
        if let Some(ref p) = maybe_stats_file {
            eprintln!(
                "TEST MODE: using {}. Results are based on explicit test file, not latest export.",
                p.file_name().and_then(|n| n.to_str()).unwrap_or("(unknown)")
            );
        }
    }

    let resolved: std::collections::HashMap<String, cache::CachedItem> =
        if let Some(ref sf_path) = maybe_stats_file {
            match gearstats::read_stats_file(sf_path) {
                Ok(items) => items.into_iter().map(|i| (format!("{}::{}", i.slot, i.name), i)).collect(),
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

    // Bug 1 fix: when a stats file is in use, all items in it are candidates.
    // The equipped/candidate distinction only applies to the live export path.
    let (equipped_names, candidate_names): (Vec<String>, Vec<String>) =
        if maybe_stats_file.is_some() {
            (vec![], resolved.keys().cloned().collect())
        } else {
            (
                export.equipped.iter().map(|i| i.name.clone()).collect(),
                export.candidates.iter().map(|i| i.name.clone()).collect(),
            )
        };

    // Run the optimizer.
    let result = optimizer::optimize(
        &resolved,
        &equipped_names,
        &candidate_names,
        &cli.goals,
    );

    // Choose the file label shown in the report: use the stats TOML when one is
    // in use (the normal optimizer path).  When no stats file was found the
    // optimizer falls back to the live db/wiki/cache pipeline, which derives
    // items directly from the plugindata export — so reference that instead.
    let report_input = maybe_stats_file
        .as_ref()
        .map(|p| {
            let s = p.to_string_lossy();
            if let Some(idx) = s.find("lgo\\") {
                s[idx..].to_string()
            } else if let Some(idx) = s.find("lgo/") {
                s[idx..].to_string()
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| plugindata_path.display().to_string());

    // Print the report.
    report::print_report(
        &result,
        &cli.goals,
        &character,
        &report_input,
    );

    // Exit with a non-zero code if infeasible, so shell scripts can detect it.
    // if !result.feasible {
    //    process::exit(2);
    //}
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
/// Priority: --test flag > explicit --stats-file flag > auto-detected most recent file.
fn stats_file_to_use(cli: &Cli, char_dir: &Path, character: &str) -> Option<PathBuf> {
    if let Some(ref p) = cli.test_stats_file {
        return Some(char_dir.join("lgo").join("test data").join(p));
    }
    if let Some(ref p) = cli.stats_file {
        return Some(p.clone());
    }
    gearstats::find_latest_stats_file(char_dir, character)
}

// -- CLI parsing ---------------------------------------------------------------

struct Cli {
    character:       Option<String>,
    cache_path:      Option<PathBuf>,
    file:            Option<PathBuf>,
    stats_file:      Option<PathBuf>,
    test_stats_file: Option<PathBuf>,
    write_stats:     bool,
    forget_edits:    bool,
    goals:           Vec<StatGoal>,
}

fn parse_args(args: &[String]) -> Result<Cli, String> {
    let mut character       = None;
    let mut cache_path      = None;
    let mut file            = None;
    let mut stats_file      = None;
    let mut test_stats_file = None;
    let mut write_stats     = false;
    let mut forget_edits    = false;
    let mut goals           = Vec::new();
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
            "--test" => {
                // Optional filename argument: consume the next token only if it
                // looks like a .toml filename (no leading '-' and ends with
                // ".toml"), to avoid consuming stat goals such as "tm:450000".
                let next = args.get(i + 1);
                let filename = match next {
                    Some(n) if !n.starts_with('-') && n.to_lowercase().ends_with(".toml") => {
                        i += 1;
                        n.clone()
                    }
                    _ => "lgo_stats_TEST.toml".to_string(),
                };
                test_stats_file = Some(PathBuf::from(filename));
            }
            "--gearlist" | "-gl" => {
                write_stats = true;
            }
            "--forget-edits" => {
                forget_edits = true;
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

    Ok(Cli { character, cache_path, file, stats_file, test_stats_file, write_stats, forget_edits, goals })
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

fn ensure_lgo_dir(char_dir: &Path) -> Result<(), String> {
    let lgo_dir = char_dir.join("lgo");
    std::fs::create_dir_all(&lgo_dir).map_err(|e| {
        format!(
            "Cannot create lgo directory {}: {}",
            lgo_dir.display(),
            e
        )
    })?;
    let test_data_dir = lgo_dir.join("test data");
    std::fs::create_dir_all(&test_data_dir).map_err(|e| {
        format!(
            "Cannot create test data directory {}: {}",
            test_data_dir.display(),
            e
        )
    })?;
    Ok(())
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
    println!("LGO - Thalya's LOTRO Gear Optimizer");
    println!();
    println!("Usage:");
    println!("  lgo --gearlist [options]               Generate/update editable gear stats file");
    println!("  lgo [options] <stat:minimum> [...]     Run optimizer");
    println!("  lgo --test [<file>] [options] <stat:minimum> [...]");
    println!("                                         Run optimizer with a test .toml from lgo\\test data\\");
    println!();
    println!("Options:");
    println!("  --character   <name>  Character name (auto-detected if only one exists)");
    println!("  --file        <path>  Explicit path to lgo_export_*.plugindata");
    println!("  --stats-file  <path>  Explicit path to lgo_stats_*.toml");
    println!("  --test       [<file>] Use a test stats .toml from lgo\\test data\\");
    println!("                        (default filename: lgo_stats_TEST.toml)");
    println!("  --cache       <path>  Path to the item cache JSON file");
    println!("  --forget-edits        Ignore hand-edit history; generate a fresh stats file");
    println!("  --help                Show this message");
    println!();
    println!("Workflow:");
    println!("  1) Place candidate items in a Shared Storage chest named 'lgo'");
    println!("  2) Run /lgo export in-game");
    println!("  3) Run: lgo --gearlist");
    println!("  4) Edit the generated lgo_stats_*.toml file as needed");
    println!("  5) Run: lgo <stat:minimum> [<stat:minimum> ...]");
    println!();
    println!("Hand-edit preservation (--gearlist):");
    println!("  When a previous lgo_stats_*.toml exists, lgo merges it with the fresh");
    println!("  export rather than overwriting it.  Fields you have hand-edited are");
    println!("  detected and you are prompted to keep your value or accept the new");
    println!("  exporter value.  Legendary Item stats (unknown to the exporter) are");
    println!("  always preserved automatically with no prompt.");
    println!("  Use --forget-edits to skip the merge and generate a clean slate.");
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
