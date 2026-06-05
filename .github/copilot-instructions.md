# Copilot Instructions for LGO

These instructions apply to all Copilot-assisted work in this repository
(chat, coding agent, IDE completions). They cover Rust style, commit
messages, and PR mechanics only. Everything else — project layout,
workflow, slot/stat vocabularies, bug status, file-handling rules,
methodology — lives in `docs/AGENT_CONTEXT.md`.

## Required reading

Before doing anything else, read:

1. `docs/AGENT_CONTEXT.md` — project brief and authoritative starting point.
2. Any design doc relevant to the current task (e.g.
   `docs/RESOLVER_DESIGN.md` for resolver work).

If `AGENT_CONTEXT.md` and these instructions disagree, `AGENT_CONTEXT.md`
wins — it is the more frequently maintained document.

## Rust conventions

- **Edition:** 2021. Single Cargo package: the `lgo` binary
  (`src/main.rs`) plus a thin library (`src/lib.rs`) that exists so
  integration tests under `tests/` can reach internal modules.
- **Style:** `cargo fmt` defaults; no rustfmt overrides.
- **Lints:** `cargo clippy` warnings should be addressed unless
  explicitly suppressed with a comment explaining why.
- **Testing:** `cargo test` must be green before any PR. Unit tests
  live next to the code they exercise (`#[cfg(test)] mod tests`);
  integration tests live in `tests/`.
- **Comments:** prefer doc comments (`///`) on public items;
  module-level comments (`//!`) describing the module's purpose.
  Comments should explain *why*, not *what*.

## Commit and PR conventions

- **Commit messages:** imperative mood, optional scope prefix
  (e.g. `slot_resolver: fix table position`). First line less than or 
  equal to 72 chars; body explains *why* if non-obvious.
- **PR descriptions:** reference the relevant section of
  `AGENT_CONTEXT.md` or a design doc when applicable.
- **`Cargo.lock`** is committed (the crate produces a binary). Commit
  it alongside any `Cargo.toml` change that resolves new versions.
