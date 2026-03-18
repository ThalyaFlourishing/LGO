# LGO: Rust Command-Line Optimizer & Reporter

LGO is a Rust command-line tool for optimizing and reporting on `.lgo` data files.

## Features

- **Optimizer:** Runs custom optimizations on input LGO files.
- **Report:** Generates and prints analysis reports.

## Usage

```sh
# Build the project
cargo build --release

# Run the CLI with a sample file
cargo run -- data/red.lgo

# For optimizer or report functionality (customize as needed)
cargo run -- optimize data/red.lgo
cargo run -- report data/red.lgo
```

## Project Structure

```
LGO/
??? .gitignore
??? README.md
??? src/
?   ??? main.rs
?   ??? optimizer.rs
?   ??? report.rs
??? data/
    ??? red.lgo
```

## Extending

- Add more optimization rules in `src/optimizer.rs`
- Add further report types in `src/report.rs`
- Sample data files go in the `data/` directory

---

**Note:**  
Do not edit `Cargo.toml` unless changing dependencies or metadata.
