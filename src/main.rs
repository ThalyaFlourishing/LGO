mod optimizer;
mod report;

use std::env;
use std::process;

fn print_usage() {
    println!("Usage:");
    println!("  lgo <command> <input-file>");
    println!("Commands:");
    println!("  optimize   Run optimizer on the input LGO file");
    println!("  report     Generate a report from the input LGO file");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];
    let input_file = &args[2];

    match command.as_str() {
        "optimize" => {
            if let Err(e) = optimizer::run(input_file) {
                eprintln!("Optimizer error: {}", e);
                process::exit(1);
            }
        }
        "report" => {
            if let Err(e) = report::run(input_file) {
                eprintln!("Report error: {}", e);
                process::exit(1);
            }
        }
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}