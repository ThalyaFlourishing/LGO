use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

mod optimizer;
mod report;

/// LotRO Gear Optimizer — reads an .lgo gear file and prints an optimized gear report.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the .lgo gear data file
    #[arg(short, long)]
    input: PathBuf,

    /// Write the report to a file instead of stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let raw = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read input file: {}", args.input.display()))?;

    let items = optimizer::parse(&raw)
        .with_context(|| "Failed to parse gear data")?;

    let result = optimizer::optimize(&items);

    let report_text = report::render(&result);

    match args.output {
        Some(path) => {
            fs::write(&path, &report_text)
                .with_context(|| format!("Failed to write report to: {}", path.display()))?;
            println!("Report written to {}", path.display());
        }
        None => print!("{}", report_text),
    }

    Ok(())
}
