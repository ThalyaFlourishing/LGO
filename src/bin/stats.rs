//! `stats` — standalone binary that generates the editable TOML gear stats file.
//!
//! Equivalent to running `lgo --stats`.
//!
//! Usage:
//!   stats [--character <name>] [--file <path>]
//!
//! The generated file is written to the character's AllServers directory
//! alongside the plugin export file.

fn main() {
    // Prepend --stats to the user's arguments and delegate to lgo.
    // This works because stats and lgo are built to the same target directory.
    //
    // TODO: once shared logic is extracted to src/lib.rs, call it directly
    // instead of re-exec.
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    args.insert(0, "--stats".to_string());

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("lgo")))
        .unwrap_or_else(|| std::path::PathBuf::from("lgo"));

    let status = std::process::Command::new(&exe)
        .args(&args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Error: could not run lgo: {}", e);
            std::process::exit(1);
        });

    std::process::exit(status.code().unwrap_or(1));
}