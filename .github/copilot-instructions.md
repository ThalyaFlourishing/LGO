# Copilot Coding Agent Instructions for LGO

## Project Overview

**LGO** (LotRO Gear Optimizer) is a Rust CLI tool that reads gear data files and
produces optimized gear recommendations for Lord of the Rings Online characters.

## Repository Layout

```
LGO/
├── Cargo.toml            # Rust package manifest — DO NOT MODIFY unless the user explicitly asks
├── .gitignore
├── src/
│   ├── main.rs           # CLI entry point
│   ├── optimizer.rs      # Optimization logic
│   └── report.rs         # Output/report formatting
├── data/
│   └── *.lgo             # Gear data files (plain-text, one item per line)
└── SSG_U25_LuaDocumentation/  # Reference documentation from SSG (read-only)
```

## Language & Toolchain

- **Language**: Rust (edition 2021)
- **Build**: `cargo build`
- **Run**: `cargo run -- <args>`
- **Test**: `cargo test`
- **Dependencies** (see Cargo.toml): `anyhow`, `clap` (derive feature)

## Key Conventions

- **Never modify `Cargo.toml`** unless the user explicitly requests a dependency change.
- New source files go under `src/`.
- Gear data files use the `.lgo` extension and live in `data/`.
- Prefer `anyhow::Result` for error propagation.
- Use `clap` derive macros for CLI argument parsing.

## Copilot Coding Agent Notes

- This repo has **no branch protections** — the coding agent may push to any branch
  and open pull requests freely.
- GitHub Actions are **enabled** and the Copilot SWE agent workflow runs correctly.
- To add new files, create a feature branch and open a PR against `main`.
