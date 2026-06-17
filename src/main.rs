//! LGO - LOTRO Gear Optimizer

#![allow(dead_code)]

mod build_db;
mod gear;
mod gearstats;
mod optimizer;
mod plugindata;
mod report;
mod slot_resolver;
mod stat;

use std::collections::HashMap;
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
        Command::Optimize(cli) => run_optimize(&cli),
        Command::ResolveSlots(cli) => run_resolve_slots(&cli),
        Command::BuildDb(cli) => run_build_db(&cli),
    }
}

fn run_optimize(cli: &OptimizeCli) {
    if cli.goals.is_empty() {
        eprintln!("Error: at least one stat goal is required.");
        eprintln!("Run with --help for usage.");
        process::exit(1);
    }

    let (plugindata_path, character) = match resolve_plugindata(cli) {
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

    let stats_file = match gearstats::find_latest_stats_file(&char_dir, &character) {
        Ok(Some(path)) => path,
        Ok(None) => {
            eprintln!(
                "No lgo_{}_gear.toml file found in {}",
                export.character,
                char_dir.display()
            );
            eprintln!("\nThis file is created by running 'lgo resolve-slots'.");
            eprintln!("Please complete the workflow:");
            eprintln!("  1) Place candidate items in a Shared Storage chest named 'lgo'");
            eprintln!("  2) Run /lgo export in-game");
            eprintln!("  3) Navigate to https://lotro-wiki.com in your browser");
            eprintln!("  4) Click the LGO bookmarklet");
            eprintln!("  5) Paste lgo_gearlist_*.plugindata when prompted");
            eprintln!(
                "  6) Save the generated TOML as lgo_{}_stats.toml in your AllServers directory",
                export.character
            );
            eprintln!("  7) Run: lgo resolve-slots   (creates lgo_{}_gear.toml)", export.character);
            eprintln!("  8) Run: lgo optimize <stat:min> [<stat:min> ...]");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
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
    let result = optimizer::optimize(&resolved, &candidate_names, &cli.goals);

    report::print_report(
        &result,
        &cli.goals,
        &character,
        &export.class,
        &stats_file.display().to_string(),
    );
}

fn run_resolve_slots(cli: &ResolveSlotsCli) {
    let (char_dir, character) = match resolve_character_allservers(cli.character.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let db = match slot_resolver::ItemsDb::load_default() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to load items DB (data/lgo_items.json): {}", e);
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

    let report = match slot_resolver::resolve_stats_file(&char_dir, &character, &db, force) {
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
    if let Err(e) = build_db::build(&cli.items, &cli.progressions, &cli.out) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

#[derive(Debug)]
enum Command {
    Help,
    Optimize(OptimizeCli),
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
    goals: Vec<StatGoal>,
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
    progressions: PathBuf,
    out: PathBuf,
}

fn parse_command(args: &[String]) -> Result<Command, CliParseError> {
    let verb = args[0].to_ascii_lowercase();
    match verb.as_str() {
        "--help" | "-h" | "help" => Ok(Command::Help),
        "optimize" | "--optimize" | "-o" => parse_optimize_args(&args[1..])
            .map(Command::Optimize)
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

    Ok(OptimizeCli {
        character,
        file,
        goals,
    })
}

fn parse_resolve_slots_args(args: &[String]) -> Result<ResolveSlotsCli, String> {
    let mut character = None;
    let mut force = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--character" | "-c" => {
                i += 1;
                character = Some(args.get(i).ok_or("--character requires a value")?.clone());
            }
            "--force" | "-f" => {
                force = true;
            }
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option: '{}'", arg));
            }
            _ => return Err("'resolve-slots' takes no positional arguments".to_string()),
        }
        i += 1;
    }

    Ok(ResolveSlotsCli { character, force })
}

fn parse_build_db_args(args: &[String]) -> Result<BuildDbCli, String> {
    let mut items = PathBuf::from("data/items.xml");
    let mut progressions = PathBuf::from("data/progressions.xml");
    let mut out = PathBuf::from("data/lgo_items.json");
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--items" => {
                i += 1;
                items = PathBuf::from(args.get(i).ok_or("--items requires a path")?);
            }
            "--progressions" => {
                i += 1;
                progressions = PathBuf::from(args.get(i).ok_or("--progressions requires a path")?);
            }
            "--out" => {
                i += 1;
                out = PathBuf::from(args.get(i).ok_or("--out requires a path")?);
            }
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option: '{}'", arg));
            }
            _ => return Err("'build-db' takes no positional arguments".to_string()),
        }
        i += 1;
    }

    Ok(BuildDbCli {
        items,
        progressions,
        out,
    })
}

fn resolve_plugindata(cli: &OptimizeCli) -> Result<(PathBuf, String), String> {
    if let Some(path) = &cli.file {
        if !path.exists() {
            return Err(format!("File not found: {}", path.display()));
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Could not read filename: {}", path.display()))?;
        // Strip the prefix case-insensitively but preserve the original
        // character-segment casing for downstream display purposes.
        const PREFIX: &str = "lgo_gearlist_";
        let stem_lower = stem.to_ascii_lowercase();
        let character = stem_lower
            .strip_prefix(PREFIX)
            .and_then(|s| {
                // Find the last '_' separating char from timestamp.
                s.rfind('_').map(|i| &stem[PREFIX.len()..PREFIX.len() + i])
            })
            .ok_or_else(|| {
                format!(
                    "Filename '{}' does not match expected pattern lgo_gearlist_{{character}}_{{timestamp}}",
                    stem
                )
            })?
            .to_string();
        return Ok((path.clone(), character));
    }

    let (char_dir, character) = resolve_character_allservers(cli.character.as_deref())?;
    let path = find_latest_export(&char_dir)?;
    Ok((path, character))
}

fn resolve_character_allservers(character_opt: Option<&str>) -> Result<(PathBuf, String), String> {
    let docs = documents_dir()?;
    let plugin_root = docs.join("The Lord of the Rings Online").join("PluginData");

    if !plugin_root.exists() {
        return Err(format!(
            "PluginData directory not found: {}",
            plugin_root.display()
        ));
    }

    let character = match character_opt {
        Some(c) => c.to_string(),
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
    Ok((char_dir, character))
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
            let name = match p.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => return false,
            };
            let name_lower = name.to_ascii_lowercase();
            name_lower.ends_with(".plugindata") && name_lower.starts_with("lgo_gearlist_")
        })
        .collect();

    if entries.is_empty() {
        return Err(format!(
            "No lgo_gearlist_*.plugindata files found in {}",
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
    println!("  lgo optimize      [options] <stat:min> [<stat:min> ...]");
    println!("  lgo resolve-slots [options]");
    println!("  lgo build-db      [options]");
    println!("  lgo --help | -h | help");
    println!();
    println!("Options (optimize):");
    println!("  --character <name>  Character name (auto-detected if only one exists)");
    println!("  --file      <path>  Plugindata export to read instead of auto-detect");
    println!("  --help              Show this message");
    println!();
    println!("Options (resolve-slots):");
    println!("  --character <name>  Character name (auto-detected if only one exists)");
    println!("  --force, -f         Prompt per item before overwriting or removing entries");
    println!("                      in the canonical gear file. Without --force, existing");
    println!("                      entries are preserved on every iteration.");
    println!();
    println!("Options (build-db):");
    println!("  --items        <path>  Items XML  (default: data/items.xml)");
    println!("  --progressions <path>  Progressions XML  (default: data/progressions.xml)");
    println!("  --out          <path>  Output JSON  (default: data/lgo_items.json)");
    println!();
    println!("Workflow:");
    println!("  1) Place candidate items in a Shared Storage chest named 'lgo'");
    println!("  2) Run /lgo export in-game");
    println!("  3) Navigate to https://lotro-wiki.com in your browser");
    println!("  4) Click the LGO bookmarklet");
    println!("  5) Paste the contents of lgo_gearlist_*.plugindata when prompted");
    println!(
        "  6) Save the generated TOML as lgo_<character>_stats.toml in your AllServers directory"
    );
    println!("  7) Run: lgo resolve-slots   (writes lgo_<character>_gear.toml)");
    println!("  8) Run: lgo optimize <stat:min> [<stat:min> ...]");
    println!();
    println!("Stat goals:");
    println!("  Each goal is a stat name and a minimum value, separated by ':'.");
    println!("  Goals are listed in priority order — the first stat is maximised");
    println!("  first, with later stats used only as tiebreakers.");
    println!("  A minimum of 0 means 'maximise but no floor required'.");
    println!();
    println!("  Examples:");
    println!("    lgo optimize TacticalMastery:450000 CriticalRating:350000 Finesse:0");
    println!("    lgo optimize tm:450000 cr:350000 fn:0");
    println!("    lgo optimize --character Thalya tm:450000 oh:100000");
    println!("    lgo resolve-slots");
    println!("    lgo resolve-slots --force");
    println!("    lgo build-db");
    println!("    lgo build-db --items data/items.xml --progressions data/progressions.xml --out data/lgo_items.json");
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
                assert_eq!(cli.items, PathBuf::from("data/items.xml"));
                assert_eq!(cli.progressions, PathBuf::from("data/progressions.xml"));
                assert_eq!(cli.out, PathBuf::from("data/lgo_items.json"));
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
            "--progressions",
            "my/prog.xml",
            "--out",
            "my/out.json",
        ]))
        .expect("build-db path flags should parse");
        match cmd {
            Command::BuildDb(cli) => {
                assert_eq!(cli.items, PathBuf::from("my/items.xml"));
                assert_eq!(cli.progressions, PathBuf::from("my/prog.xml"));
                assert_eq!(cli.out, PathBuf::from("my/out.json"));
            }
            _ => panic!("expected build-db command"),
        }
    }
}
