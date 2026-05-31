# Copilot Instructions for LGO

These instructions apply to all Copilot-assisted work in this repository
(chat, coding agent, IDE completions). Read them before any task.

## Required reading

Before doing anything else, read:
1. `docs/AGENT_CONTEXT.md` — project brief, conventions, known bugs, and
   methodology rules. This is the authoritative starting point.
2. Any design doc relevant to the current task (e.g.
   `docs/RESOLVER_DESIGN.md` for resolver work).

If `AGENT_CONTEXT.md` and these instructions disagree, `AGENT_CONTEXT.md`
wins — it is the more frequently maintained document.

## Working branch

Active development happens on `The-Browser-Method`, not `main`.
All PRs and edits should target `The-Browser-Method` unless the user
explicitly says otherwise.

## File handling

- **Do not paste large files into chat.** Use repository tools (`getfile`,
  code search) to read files in place. Pasting puts file contents into
  conversation context, which is finite and shared across the whole
  session.
- **Never ingest `SSG_U25_LuaDocumentation/`** or its contents in chat.
  See `AGENT_CONTEXT.md` §7 for the rationale. Reference filenames if
  needed, but do not read or quote contents.
- **`data/lgo_items.json` is ~8 MB.** Do not load it wholesale into
  conversation. If you need to inspect its schema, read the first few
  entries only. The schema is documented in `docs/RESOLVER_DESIGN.md` §3.

## Project conventions

- **Language:** Rust 2021 edition. Single binary crate (`lgo`).
- **Style:** `cargo fmt` defaults; no rustfmt overrides. `cargo clippy`
  warnings should be addressed unless explicitly suppressed with a
  comment explaining why.
- **Testing:** `cargo test` must be green before any PR. Tests live
  next to the code they exercise (`#[cfg(test)] mod tests`).
- **Comments:** prefer doc comments (`///`) on public items;
  module-level comments (`//!`) describing the module's purpose.
  Comments should explain *why*, not *what*.

## Methodology rules

These mirror `AGENT_CONTEXT.md` §7 but are repeated here because they
apply across all sessions:

- **Do not reconstruct from inference.** If you don't know something,
  ask the user or read the relevant file. Speculative reconstruction
  is the leading cause of subtle bugs in this project's history.
- **Verify schemas at implementation time.** Documented schemas
  (especially in `RESOLVER_DESIGN.md`) come from deleted code or prior
  conversation, not from the current source of truth. Probe and verify
  before relying.
- **Bug status lives in `AGENT_CONTEXT.md` §6.** Update it when fixing
  or deferring a bug.

## CLI surface

The `lgo` binary uses an explicit subcommand verb:

- `lgo resolve-slots [--file PATH]` — resolves slot fields against
  `data/lgo_items.json` (see `RESOLVER_DESIGN.md`).
- `lgo -Optimize <stat:minimum> [...]` — runs the gear optimizer.

When updating CLI behaviour, also update:
- `docs/AGENT_CONTEXT.md` (workflow examples)
- `docs/User Workflow.txt` (user-facing instructions)
- `print_usage()` in `src/main.rs`

## Commit and PR conventions

- Commit messages: imperative mood, optional scope prefix (e.g.
  `slot_resolver: fix table position`). First line ?72 chars; body
  explains *why* if non-obvious.
- PR descriptions should reference the relevant section of
  `AGENT_CONTEXT.md` or a design doc when applicable.
- `Cargo.lock` is committed (this is a binary crate). Commit it
  alongside any `Cargo.toml` change that resolves new versions.