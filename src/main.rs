mod gear;
mod optimizer;
mod stat;

use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use gear::{GearItem, GearSet};
use stat::{Stat, StatWeights};

#[derive(Parser)]
#[command(name = "lgo", version, about = "LotRO Gear Optimizer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Find the best-in-slot gear combination from a list of available items.
    Optimize {
        /// Path to a JSON file containing available gear items.
        #[arg(short, long, default_value = "data/items.json")]
        items: PathBuf,

        /// Path to a JSON file containing stat weights (defaults to DPS profile).
        #[arg(short, long)]
        weights: Option<PathBuf>,

        /// Class preset to use for stat weights: dps, tank, or healer.
        #[arg(short, long, default_value = "dps")]
        preset: String,
    },
    /// Score a fixed gear set and display stat totals.
    Score {
        /// Path to a JSON file containing the gear set to evaluate.
        #[arg(short, long, default_value = "data/gear_set.json")]
        set: PathBuf,

        /// Path to a JSON file containing stat weights (defaults to DPS profile).
        #[arg(short, long)]
        weights: Option<PathBuf>,

        /// Class preset to use for stat weights: dps, tank, or healer.
        #[arg(short, long, default_value = "dps")]
        preset: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Optimize { items, weights, preset } => {
            let items: Vec<GearItem> = load_json(&items)
                .with_context(|| format!("Failed to load items from {:?}", items))?;
            let stat_weights = resolve_weights(weights, &preset)?;

            let gear_set = optimizer::optimize(&items, &stat_weights);
            print_gear_set(&gear_set, &stat_weights);
        }
        Command::Score { set, weights, preset } => {
            let items: Vec<GearItem> = load_json(&set)
                .with_context(|| format!("Failed to load gear set from {:?}", set))?;
            let stat_weights = resolve_weights(weights, &preset)?;

            // Build a GearSet from the flat item list.
            let mut gear_set = GearSet::default();
            for item in items {
                gear_set.items.insert(item.slot, item);
            }

            print_gear_set(&gear_set, &stat_weights);
        }
    }

    Ok(())
}

fn load_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    let value = serde_json::from_str(&content)?;
    Ok(value)
}

fn resolve_weights(path: Option<PathBuf>, preset: &str) -> Result<StatWeights> {
    if let Some(p) = path {
        return load_json(&p).with_context(|| format!("Failed to load weights from {:?}", p));
    }
    Ok(preset_weights(preset))
}

/// Built-in stat-weight presets for common roles.
fn preset_weights(preset: &str) -> StatWeights {
    let weights = match preset {
        "tank" => vec![
            (Stat::Vitality, 1.0),
            (Stat::Armor, 0.9),
            (Stat::PhysMitigation, 0.85),
            (Stat::TactMitigation, 0.8),
            (Stat::CritDefense, 0.75),
            (Stat::Resistance, 0.7),
            (Stat::Morale, 0.65),
            (Stat::Might, 0.4),
        ],
        "healer" => vec![
            (Stat::Will, 1.0),
            (Stat::Fate, 0.9),
            (Stat::OutgoingHealing, 0.85),
            (Stat::Vitality, 0.7),
            (Stat::CritRating, 0.6),
            (Stat::Power, 0.5),
            (Stat::IncomingHealing, 0.3),
        ],
        _ => vec![
            // default: dps
            (Stat::Might, 1.0),
            (Stat::CritRating, 0.9),
            (Stat::DevRating, 0.85),
            (Stat::FinesseRating, 0.8),
            (Stat::OffensiveOverpower, 0.75),
            (Stat::Agility, 0.5),
            (Stat::Vitality, 0.3),
        ],
    };
    StatWeights(weights)
}

fn print_gear_set(set: &GearSet, weights: &StatWeights) {
    println!("\n=== Optimized Gear Set ===\n");

    let mut slots: Vec<_> = set.items.keys().collect();
    slots.sort_by_key(|s| format!("{}", s));

    for slot in slots {
        let item = &set.items[slot];
        println!(
            "  {:15} {:>6} ilvl  {}",
            format!("[{}]", slot),
            item.item_level,
            item.name
        );
    }

    let total_score = optimizer::score_set(set, weights);
    println!("\n  Total weighted score: {:.1}", total_score);

    println!("\n=== Stat Totals ===\n");

    let totals = set.total_stats();
    let mut stats: Vec<_> = totals.iter().collect();
    stats.sort_by_key(|(s, _)| format!("{}", s));

    for (stat, value) in stats {
        println!("  {:25} {:>8}", format!("{}", stat), value);
    }

    println!();
}
