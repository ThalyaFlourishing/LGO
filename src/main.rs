//! LGO - LOTRO Gear Optimizer

mod gear;
mod gearstats;
mod optimizer;
mod plugindata;
mod report;
mod stat;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

use stat::StatGoal;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_usage();
        process::exit(0);
    }

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

    let (plugindata_path, character) = match resolve_plugindata(&cli) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let export = match plugindata::load(&plugindata_path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading plugin data: {}", e);
            process::exit(1);
        }
    };

    let char_dir = plugindata_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    let stats_file = match gearstats::find_latest_stats_file(&char_dir) {
        Some(path) => path,
        None => {
            eprintln!("No lgo_stats_*.toml file found in {}", char_dir.display());
            eprintln!("\nGenerate one with the bookmarklet workflow:");
            eprintln!("  1) Place candidate items in a Shared Storage chest named 'lgo'");
            eprintln!("  2) Run /lgo export in-game");
            eprintln!("  3) Navigate to https://lotro-wiki.com in your browser");
            eprintln!("  4) Click the LGO bookmarklet");
            eprintln!("  5) Paste lgo_itemnames_*.plugindata when prompted");
            eprintln!("  6) Save the generated .toml to your AllServers directory");
            eprintln!("  7) Run: lgo <stat:minimum> [<stat:minimum> ...]");
            process::exit(1);
        }
    };

    let stats_items = match gearstats::read_stats_file(&stats_file) {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Error reading gear stats file: {}", e);
            process::exit(1);
        }
    };

    let resolved: HashMap<String, gear::GearItem> = stats_items
        .into_iter()
        .enumerate()
        .map(|(idx, item)| (format!("{:04}::{}::{}", idx, item.slot, item.name), item))
        .collect();

    let candidate_names: Vec<String> = resolved.keys().cloned().collect();
    let result = optimizer::optimize(&resolved, &[], &candidate_names, &cli.goals);

    report::print_report(
        &result,
        &cli.goals,
        &character,
        &export.class,
        &stats_file.display().to_string(),
    );
}

struct Cli {
    character: Option<String>,
    file: Option<PathBuf>,
    goals: Vec<StatGoal>,
}

fn parse_args(args: &[String]) -> Result<Cli, String> {
    let mut character = None;
    let mut file = None;
    let mut goals = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--character" | "-c" => {
                i += 1;
                character = Some(args.get(i).ok_or("--character requires a value")?.clone());
            }
            "--file" | "-f" => {
                i += 1;
                file = Some(PathBuf::from(args.get(i).ok_or("--file requires a path")?));
            }
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option: '{}'", arg));
            }
            arg => {
                let goal: StatGoal = arg
                    .parse()
                    .map_err(|e| format!("Invalid stat goal '{}': {}", arg, e))?;
                goals.push(goal);
            }
        }
        i += 1;
    }

    Ok(Cli {
        character,
        file,
        goals,
    })
}

fn resolve_plugindata(cli: &Cli) -> Result<(PathBuf, String), String> {
    if let Some(path) = &cli.file {
        if !path.exists() {
            return Err(format!("File not found: {}", path.display()));
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Could not read filename: {}", path.display()))?;
        let character = stem
            .strip_prefix("lgo_export_")
            .and_then(|s| s.rsplitn(2, '_').nth(1))
            .ok_or_else(|| {
                format!(
                    "Filename '{}' does not match expected pattern lgo_export_{{character}}_{{timestamp}}",
                    stem
                )
            })?
            .to_string();
        return Ok((path.clone(), character));
    }

    let docs = documents_dir()?;
    let plugin_root = docs.join("The Lord of the Rings Online").join("PluginData");

    if !plugin_root.exists() {
        return Err(format!(
            "PluginData directory not found: {}",
            plugin_root.display()
        ));
    }

    let character = match &cli.character {
        Some(c) => c.clone(),
        None => discover_character(&plugin_root)?,
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
    std::fs::create_dir_all(&lgo_dir)
        .map_err(|e| format!("Cannot create lgo directory {}: {}", lgo_dir.display(), e))?;
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
            let mut msg =
                String::from("Multiple characters found. Specify one with --character:\n");
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
    std::env::current_dir().map_err(|e| format!("Cannot determine working directory: {}", e))
}

fn print_usage() {
    println!("LGO - Thalya's LOTRO Gear Optimizer");
    println!();
    println!("Usage:");
    println!("  lgo [options] <stat:minimum> [...]     Run optimizer");
    println!();
    println!("Options:");
    println!("  --character <name>  Character name (auto-detected if only one exists)");
    println!("  --file      <path>  Explicit path to lgo_export_*.plugindata");
    println!("  --help              Show this message");
    println!();
    println!("Workflow:");
    println!("  1) Place candidate items in a Shared Storage chest named 'lgo'");
    println!("  2) Run /lgo export in-game");
    println!("  3) Navigate to https://lotro-wiki.com in your browser");
    println!("  4) Click the LGO bookmarklet");
    println!("  5) Paste the contents of lgo_itemnames_*.plugindata when prompted");
    println!("  6) Copy the generated .toml and save it to your AllServers directory");
    println!("  7) Run: lgo <stat:minimum> [<stat:minimum> ...]");
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
}
