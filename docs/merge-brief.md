# Add merge-on-iteration behaviour to `lgo resolve-slots`

**Suggested base branch:** `main`

## Background

Today `lgo resolve-slots` reads the latest `lgo_stats_*.toml` produced by the bookmarklet, rewrites each item's `slot` field via `data/lgo_items.json` lookups, regroups items by canonical slot family, and writes a sibling `*_resolved.toml`. Hand-edits the user makes to the resolved file (legendary item stats, essence sums, slot fixes for items not in the items DB, corrections to bookmarklet zeros) are lost on the next iteration: the next bookmarklet run produces a brand-new `.toml`, `resolve-slots` writes a brand-new `*_resolved.toml`, and the previous hand-edited file is orphaned. The optimizer's `find_latest_stats_file` picks the new file by lexicographic sort; nothing reads the old one.

The pre-pivot codebase addressed this via a `[__user_edits__]` metadata section and per-stat prompts. **That design has been rejected.** The replacement is simpler:

- **Preserve all existing items by default.** When the new bookmarklet output contains an item whose `name` matches an item in the canonical file, the new data is silently discarded and the existing block is kept verbatim.
- **Remove items that disappeared from the new export.** Items present in the canonical file but absent from the new export are removed from the canonical file.
- **`--force` (alias `-f`) opts into overwriting.** When `--force` is passed, the user is prompted per item before destructive changes; identical-data items remain a no-op even under `--force`.
- **No `[user_edits]` / `[__user_edits__]` section. No baseline file. No sidecar.** The canonical file is the only state.

## File-naming changes

Two per-character files now live in the AllServers directory:

- **`lgo_<character>_stats.toml`** — the bookmarklet's output. The user saves the bookmarklet's TOML here. Transient input to `resolve-slots`. **No timestamps in the name** (most users are European and write dates day-first; getting timestamps right manually is a usability landmine).
- **`lgo_<character>_gear.toml`** — the canonical merged file. Sole output of `resolve-slots`. Sole input the optimizer reads. The `_resolved.toml` suffix is retired.

`find_latest_stats_file` (in `src/gearstats.rs`) needs two distinct callers, so it needs two distinct behaviours:

1. **For the optimizer** ("what file should I read?"): prefer `lgo_<character>_gear.toml` if it exists. Fall back to the existing lexicographic scan only for backward compatibility with users who haven't run the new resolver yet.
2. **For the resolver** ("what is the new bookmarklet output?"): read `lgo_<character>_stats.toml` exactly. Do not scan, do not match the canonical file.

Add a second function (e.g. `find_bookmarklet_output(dir, character) -> Option<PathBuf>`) rather than overloading the existing one.

The character name comes from the `--character` flag if provided. Otherwise, use the existing `discover_character` logic in `main.rs` (auto-detect when only one character directory exists; error out listing options when multiple exist). Do not extract the character name from filename parsing — the new filenames embed it but the existing PluginData directory structure is the authoritative source.

## Merge semantics (precise)

Inputs:

- **`previous`**: parsed `lgo_<character>_gear.toml`, or `None` if absent.
- **`incoming`**: parsed and slot-resolved `lgo_<character>_stats.toml`.

Output: a `DocumentMut` written back to `lgo_<character>_gear.toml`, plus a `MergeOutcome` summary.

Algorithm, in order:

1. **First run (no `previous`).** Take `incoming` verbatim. The canonical file is created fresh.
2. **Subsequent runs.** Iterate over `previous.items`:
   - If the item's `name` is present in `incoming`: **keep the previous block**. Discard the incoming block for that name.
   - If the item's `name` is absent from `incoming`: **remove the item** (do not include it in the output).
3. Iterate over `incoming.items`:
   - If the item's `name` is absent from `previous`: **add the incoming block**.
   - If the item's `name` is present in `previous`: already handled by step 2; skip.
4. After the merge, **regroup the resulting items by canonical slot family** using the existing `slot_resolver.rs::push_group` / `slot_family_order` logic. Do not segregate preserved vs. new items.

Name comparison is byte-for-byte exact match on the `name` field. No case folding, no Unicode normalisation, no fuzzy matching.

## `--force` semantics

When `--force` (alias `-f`) is passed:

1. For each item in `previous` whose `name` is in `incoming`:
   - **Compute whether the incoming block differs from the previous block.** Comparison is on the canonical stat fields and the `slot` field. Comments, whitespace, and other decor do not count as differences. If only those differ, treat the blocks as identical. If they are identical, **the item is a no-op under `--force`** — do not prompt, keep the previous block, do not count it toward overwrite totals. This must hold *strictly*: a `--force` run on a canonical file with no actual changes in `incoming` must produce zero prompts and a bit-identical output (modulo regrouping if the previous file was somehow out of order).
   - If they differ, **prompt**:

     ```
     Overwrite stats for "<name>"? (y/n/a)
     ```

     `y` accepts this overwrite, `n` rejects (keep previous), `a` means "yes to all *remaining overwrites*" (does not affect removal prompts).

2. For each item in `previous` whose `name` is **not** in `incoming`:
   - **Prompt**:

     ```
     Remove "<name>" (no longer in export)? (y/n/a)
     ```

     `y` accepts the removal, `n` rejects (keep previous, item retained), `a` means "yes to all *remaining removals*" (does not affect overwrite prompts).

3. Items in `incoming` but not in `previous` are added without prompt (adding is never destructive).

The two "yes to all" states are independent and per-category. Track them as two separate booleans.

Prompting reads from `stdin` and writes to `stderr` (so prompt text does not pollute redirected stdout). Accept `y`/`Y`/`n`/`N`/`a`/`A` followed by Enter. Reject other input with a re-prompt rather than crashing.

Make the prompting logic injectable for tests. Define a `Prompter` trait with one method `fn prompt(&mut self, category: PromptCategory, item_name: &str) -> PromptAnswer;` plus enums `PromptCategory { Overwrite, Remove }`, `PromptAnswer { Yes, No, YesToAll }`. Provide `StdinPrompter` for production and `ScriptedPrompter` for tests.

**`--force` with non-TTY stdin must fail.** When stdin is not a terminal (piped or redirected), `--force` errors out with `--force requires interactive stdin for prompts.` and exits nonzero. The whole point of `--force` is interactivity; silently auto-accepting on a piped run is exactly the failure mode that would lose a user's hand-edits. Use `std::io::IsTerminal` (stable since Rust 1.70) to detect.

## Terminal output (summary)

After the merge, regardless of whether `--force` was used, print a per-character summary to stdout:

- `Added: <count>` items added from the new export.
- Default mode: `Preserved: <count>` items kept from the previous file.
- Force mode: `Overwritten: <X> / Preserved: <Y>`.
- `Removed: <count>` items removed.
- One line per removed item: `Removed (no longer in export): <name>`. The audience is friends of the author who do not read source code; they need to see what disappeared.
- One line per item the resolver could not look up in `data/lgo_items.json` (slot still `Unknown` after resolution): `Unknown slot (may need hand-edit): <name>`. Same as today.
- `Previous: <canonical-path or "(none — first run)">`.
- `New export: <bookmarklet-output-path>`.
- `Wrote: <canonical-path>`.

## Idempotency

Running `lgo resolve-slots` twice in a row with no new bookmarklet output must produce a bit-identical canonical file the second time. Add a test that exercises this: write a canonical file, run the merge, assert output is byte-identical to input.

## Error handling for distribution

The audience is a small group of personal acquaintances of the author (per `AGENT_CONTEXT.md` §1). Errors reachable from user input must be returned as `Result::Err` with a human-readable message — never `panic!` or `unwrap()`. Audit the existing `resolve-slots` path and the new merge code for `unwrap()` and `panic!()` on user-reachable paths; convert each to a proper error return. Leave `unwrap`s in test code and in genuinely-unreachable internal invariants alone.

Specific error cases:

- Canonical file exists but is malformed TOML → `Cannot parse '<path>': <toml_edit error>`.
- No bookmarklet output and no canonical file → `No lgo_<character>_stats.toml or lgo_<character>_gear.toml found in <dir>`.
- Canonical file exists but no bookmarklet output → **not an error.** Emit `No new export found; canonical file is unchanged.` and exit 0.
- `--force` passed when stdin is not a TTY → `--force requires interactive stdin for prompts.`, exit nonzero.

## Required code changes

### `src/slot_resolver.rs`

Add a merge layer atop `resolve_toml_str`:

- Keep `resolve_toml_str(src, db)` exactly as it is. Pure function, no merge logic.
- Add `merge_into_canonical(previous: Option<&str>, incoming_resolved: &str, force: ForceMode) -> Result<MergeOutcome, ResolveError>`. `ForceMode` is `NoForce | Force { prompter: Box<dyn Prompter> }`.
- `MergeOutcome` carries: `added: Vec<String>`, `preserved: Vec<String>`, `overwritten: Vec<String>`, `removed: Vec<String>`, `unknown_slot: Vec<String>`, and the merged `String` to write.

Update `resolve_stats_file` (the file-level wrapper):

1. Determine the character name (passed in by `main.rs`).
2. Compute the canonical path (`lgo_<character>_gear.toml`) and bookmarklet path (`lgo_<character>_stats.toml`).
3. Read the canonical file if it exists.
4. Read and slot-resolve the bookmarklet output.
5. Call `merge_into_canonical`.
6. Write the result to the canonical path.
7. Return a `Report` containing the `MergeOutcome` for `main.rs` to display.

### `src/gearstats.rs`

- Add `find_bookmarklet_output(dir: &Path, character: &str) -> Option<PathBuf>` returning the path to `lgo_<character>_stats.toml` if it exists.
- Update `find_latest_stats_file` to prefer `lgo_<character>_gear.toml` when present, falling back to the existing lexicographic scan otherwise.

### `src/main.rs`

- Extend `ResolveSlotsCli` with a `force: bool` field.
- Extend `parse_resolve_slots_args` to recognise `--force` and `-f`.
- Update `run_resolve_slots` to pass the character name and force flag through.
- Update terminal output to print the new summary lines.
- Update `print_usage` to document `--force` / `-f` under `resolve-slots`.

Add CLI parsing tests mirroring the existing pattern:

```rust
#[test]
fn resolve_slots_accepts_force_flag() {
    for token in ["--force", "-f"] {
        let cmd = parse_command(&s(&["resolve-slots", token])).expect("must parse");
        match cmd {
            Command::ResolveSlots(cli) => assert!(cli.force),
            _ => panic!("expected resolve-slots"),
        }
    }
}
```

### Tests

Unit tests in `src/slot_resolver.rs` for the merge layer:

- First run (no canonical file): output equals resolved incoming verbatim.
- Subsequent run, no changes: bit-identical output.
- Subsequent run, item added in incoming: present in output.
- Subsequent run, item removed in incoming: absent from output.
- Subsequent run, item present in both with different stats, no force: previous block kept.
- Force mode, scripted prompter answering `y`: incoming block wins.
- Force mode, scripted prompter answering `n`: previous block kept.
- Force mode, scripted prompter answering `a` to overwrites: subsequent overwrite prompts not invoked; removal prompts still invoked.
- Force mode, identical data: prompter never invoked, even under force.
- Force mode, item disappeared, scripted prompter answering `y`: item removed.
- Force mode, item disappeared, scripted prompter answering `n`: item retained.

An integration test in `tests/` exercising the file-level merge against synthetic TOML files in a temp directory.

## Documentation updates

### `docs/AGENT_CONTEXT.md`

- **§2 (User workflow)**: replace the existing 9-step `_resolved.toml` description with the new merge-aware flow. Mention `lgo_<character>_stats.toml` (bookmarklet sink), `lgo_<character>_gear.toml` (canonical), the preserve-by-default rule, and `--force`. Keep terse — this section is a summary, not a tutorial.
- **§10 (Deferred work)**: replace the last bullet (the one referencing pre-pivot `src/merge.rs` and `[__user_edits__]`) with: "Hand-edit preservation across re-runs: implemented in PR #<this-PR> via preserve-by-default merge in `resolve-slots`. The `[__user_edits__]` design from `docs/Merge Coding Prompt.txt` and `docs/User Story & Hand-Edit-Tracking Approach.txt` was rejected in favour of the simpler preserve-by-default model. Those two design docs are now historical."
- **§10**: add a new bullet: "Rename detection. The merge step matches items by exact byte-for-byte name. If the wiki renames an item between exports, or if a Unicode encoding glitch alters a character, the merge will treat the renamed item as a removal-and-add pair rather than the same item, silently dropping the user's hand-edits. Accepted risk; revisit if it becomes a real problem."
- Do not modify §1, §3, §4, §5, §6, §7, §8, or §9.

### `docs/User Workflow.txt`

The user is hand-editing this file. Make only the strictly-necessary technical corrections; do not redraft prose.

- **Step 7**: change the suggested filename to `lgo_<character>_stats.toml`. The bookmarklet's output is the resolver's input; the canonical merged file is the resolver's output. The user should not save the bookmarklet output to the canonical name — that would destroy hand-edits before the merge can preserve them.
- **Step 8**: the existing text *"If there is a pre-existing .toml file, it will not overwrite any items already present."* is correct and matches the new behaviour. Keep it. Add a sentence describing what `--force` prompts for (overwrite per item, remove per item, with `a` for "yes to all" within each category).
- **TODO block at lines 1–4**: now satisfied by step 8a. Remove it.
- **Iteration block, last bullet**: filename reference is already `lgo_<character-name>_gear.toml`, which is correct.

Do not touch the rest of the file (steps 1–6, 8a, 9–11) for style or wording.

### `docs/Merge Coding Prompt.txt` and `docs/User Story & Hand-Edit-Tracking Approach.txt`

These are historical design docs for the rejected `[__user_edits__]` approach. Do **not** modify them. The §10 update in `AGENT_CONTEXT.md` notes that they are historical.

## Out of scope

Per `AGENT_CONTEXT.md` §10's "do not silently fold them into other PRs" rule, do not include any of:

- Rename detection (added to §10 as a deferred-work note above; no code change).
- The `--toml-file` optimizer flag from `AGENT_CONTEXT.md` §9.
- Any change to the bookmarklet, the in-game plugin, the optimizer, or `build-db`.
- The unknown-slot regression in §9 ("Optimizer is barfing on the 'unknown' slot names again: Pickaxe / Bridle").
- Any change to the items DB, the slot vocabulary, or the stat vocabulary.

## Constraints

- **Do not introduce `clap` or any other CLI library.** The existing hand-written parser remains.
- **Match the existing CLI conventions exactly.** Case-insensitive verb. `--long-form` and `-x` short alias. Shared options keep their existing semantics.
- **Use `toml_edit` for all TOML manipulation in the merge.** Do not round-trip through `toml` + serde; that discards comments and decor.
- **Comments inside preserved `[[item]]` blocks must survive the merge.** Hand-written notes the user added (e.g. `# essence: +1500 tactical mastery`) must round-trip. The existing `toml_edit` discipline in `slot_resolver.rs::push_group` is the model.
- **The canonical file must be the only TOML file the optimizer ever needs to read** once the merge has run at least once. No silent reliance on `lgo_<character>_stats.toml` for optimizer input.
- **No new dependencies** unless absolutely required. State the case in the PR description if one is added.
- **Final verification:** `cargo build`, `cargo build --release`, and `cargo test` all succeed with no new warnings.