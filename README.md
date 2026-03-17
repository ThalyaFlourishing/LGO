# LGO — LotRO Gear Optimizer

A command-line tool that reads gear data from `.lgo` files and produces an optimized gear report for Lord of the Rings Online characters.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later (stable toolchain)

## Building

```bash
cargo build --release
```

The compiled binary is placed at `target/release/lgo`.

## Running

```bash
# Run with the sample Red data file
cargo run -- --input data/red.lgo

# Or after a release build
./target/release/lgo --input data/red.lgo
```

### Options

| Flag | Description |
|------|-------------|
| `--input <FILE>` | Path to the `.lgo` gear data file |
| `--output <FILE>` | Write the report to a file instead of stdout (optional) |
| `--help` | Print usage information |

## Project Layout

```
LGO/
├── Cargo.toml           # Rust package manifest
├── data/
│   └── red.lgo          # Sample gear data (Red line)
└── src/
    ├── main.rs          # CLI entry point
    ├── optimizer.rs     # Core optimisation logic
    └── report.rs        # Report generation / formatting
```

## Data Format

`.lgo` files are plain-text, line-oriented gear definitions.  Each non-blank, non-comment line describes one item:

```
# <comment>
<slot>,<item_name>,<stat1>=<value1>[,<stat2>=<value2>...]
```

Example:
```
# Red line starter gear
head,Iron Helm,Vitality=200,Morale=400
chest,Iron Breastplate,Vitality=250,Morale=500
```

## License

This project is provided as-is for personal use.
