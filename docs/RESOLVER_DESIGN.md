# `resolve-slots` Subcommand — Design / Scoping Doc

**Status (now historical):** This design has shipped. The `resolve-slots` subcommand and the `optimize` / `resolve-slots` CLI split are live on `The-Browser-Method` (CLI wiring in PR #18, integration tests in PR #19). This document is preserved for reference but is no longer an active spec.

**Original status:** Approved scope, not yet implemented.
**Branch:** `The-Browser-Method`
**Related:** `docs/AGENT_CONTEXT.md` (project-wide context — read first).

This document is written to be **self-contained for any future agent session**. If you are an AI agent picking this up cold, read `docs/AGENT_CONTEXT.md` first, then this file. You should not need to reconstruct prior conversation.

---

## 1. Problem statement

The bookmarklet (`bookmarklet/lgo_bookmarklet.html`) reads each item's stats from lotro-wiki.com's free-text `{{Item Tooltip}}` template. Stats work well; **slots do not**:

- The wiki has no enforced allow-list for the `slot=` field — editors type whatever convention they remember (`Shoulder`/`Shoulders`, `Gloves`/`Hands`, etc.).
- Weapons (and some other categories) carry their slot info in a different field (`type=`) — not in `slot=` at all.
- The result: many items emit `slot = "Unknown"` in the `.toml`, even when their stats parsed correctly. This breaks the optimizer because `parse_slot_str` in `src/gearstats.rs` rejects anything outside the canonical 19 slot strings.

This is **Bug 3** in `docs/AGENT_CONTEXT.md`.

## 2. Decided approach

Stop asking the wiki for slots at all. Instead, **look the slot up by item name** in `data/lgo_items.json`, which is a canonical game data dump (see §3). This is implemented as a new Rust CLI subcommand `resolve-slots` that runs between the bookmarklet step and the optimizer step.

### Architectural rationale (one-paragraph version)

The wiki is authoritative for **stats**. The game data dump is authoritative for **slots**. The plugin is authoritative for **names**. The current design asks the wiki for both stats and slots, which is the wrong oracle for half its job. The new design assigns each data source to what it's actually good at.

## 3. Pre-existing data: `data/lgo_items.json`

Critically important context, easy to miss otherwise:

- This file was **produced by a now-deleted `src/bin/db_build.rs`** which read `data/items.xml` + `data/progressions.xml` and wrote the JSON.
- That builder, along with `src/db.rs` (the loader) and `src/cache.rs`, `src/wiki.rs`, `src/merge.rs`, was removed in the "dead code" PR when the project pivoted to the bookmarklet approach.
- `data/lgo_items.json` itself was **left in place** and is the surviving output of the deleted pipeline.
- It is roughly 8 MB.

### Known JSON schema (from the deleted `src/db.rs` and `src/bin/db_build.rs`)

```json
{
  "<item name>": {
    "name":  "<item name>",
    "slot":  "<Slot variant — PascalCase enum, e.g. Head, Wrist1, MainHand, ClassItem>",
    "stats": { "armor": 6181, "critical_rating": 22335, "...": "..." }
  },
  "<next item name>": { ... }
}
```

- The top-level value is a JSON object keyed by item name (a `HashMap<String, CachedItem>` in the old code).
- The `slot` value uses the Rust `Slot` enum's default serialization: bare PascalCase variant names. **Crucially this is `"Wrist1"` not `"Wrist (1)"`** — close to but **not** identical to the canonical display strings in §4 below. A small translation is required.
- The `stats` map uses snake_case keys (via `#[serde(rename_all = "snake_case")]` on the old `Stat` enum). The resolver does **not** care about stats — only slots — so this section is irrelevant here.

### Verification step required at implementation time

The schema above is taken from the deleted code, not from the actual file. **The very first implementation task is to read the first ~3 entries from `data/lgo_items.json` and confirm the schema matches.** If it has drifted, stop and surface the difference before writing more code.

## 4. Canonical slot vocabulary the resolver must emit

These are the **19 strings** that `parse_slot_str` in `src/gearstats.rs` accepts. The resolver must translate the JSON's PascalCase variants into these exact strings:

| JSON value (PascalCase) | Canonical string (what the resolver writes) |
|---|---|
| `Head`        | `Head` |
| `Chest`       | `Chest` |
| `Legs`        | `Legs` |
| `Hands`       | `Hands` |
| `Feet`        | `Feet` |
| `Shoulders`   | `Shoulders` |
| `Back`        | `Back` |
| `Wrist1`      | `Wrist (1)` |
| `Wrist2`      | `Wrist (2)` |
| `Neck`        | `Neck` |
| `Finger1`     | `Finger (1)` |
| `Finger2`     | `Finger (2)` |
| `Ear1`        | `Ear (1)` |
| `Ear2`        | `Ear (2)` |
| `Pocket`      | `Pocket` |
| `MainHand`    | `Main-hand` |
| `OffHand`     | `Off-hand` |
| `Ranged`      | `Ranged` |
| `ClassItem`   | `Class Item` |

This translation should ideally live on the `Slot` type itself in `src/gear.rs` (e.g. a `From<&str>` impl or a parser method) rather than being hand-rolled inside the resolver, so it doesn't drift if `Slot::Display` ever changes.

## 5. Paired-slot handling

Rings, earrings, bracelets, and main-hand/off-hand items fit two canonical slots each (e.g. a ring fits both `Finger (1)` and `Finger (2)`).

**Decision:** The resolver writes whichever single slot `lgo_items.json` says (typically the `*1` variant, e.g. `Finger1` → `Finger (1)`). It does **not** attempt to write multiple slot entries or comma-separated lists.

This works because `src/optimizer.rs` already builds **super-candidates** for paired slots — it picks pairs of items from the full candidate pool *regardless* of whether each item's slot tag is `(1)` or `(2)`. So writing every ring as `Finger (1)` does not prevent the optimizer from filling both finger slots.

**Verification checkbox before shipping:** Re-read the super-candidate / paired-slot section of `src/optimizer.rs` and confirm in writing that it draws from the union pool, not from the `(1)`-tagged or `(2)`-tagged subsets specifically. If it filters by the tag, this whole assumption breaks and the resolver must emit something else.

**Asymmetric edge case to leave to the optimizer, not the resolver:** Main-hand/off-hand is not symmetric in the game — some items are off-hand only. That is a candidate-eligibility question, not a slot-resolution question. The resolver simply writes whatever `lgo_items.json` says is the item's primary slot.

## 6. Handling of items not in `lgo_items.json`

Two categories, same behaviour:

1. **Legendary / player-renamed items.** "Lore-master's Staff of Legends", etc. The bookmarklet already writes these with all-zero stats and a `# WARNING: all stats unknown` comment for manual entry.
2. **Stale dump.** New items added to the game after the most recent `lgo_items.json` snapshot.

**Behaviour:** Leave the `slot` value as it was (typically `"Unknown"` from the bookmarklet) and emit a per-item warning to stderr. The user fixes slot and stats together by hand.

## 7. Subcommand / CLI design

### Argument structure change

Currently `lgo` takes stat goals as positional args (`lgo tm:450000 cr:350000`). To add subcommands cleanly, **introduce an explicit verb in front**:

- `lgo -Optimize tm:450000 cr:350000 ...` — runs the optimizer (formerly the default behaviour).
- `lgo resolve-slots [--file PATH]` — runs the resolver.

The user has approved requiring `-Optimize` to keep the two uses consistent. This is a **breaking change** to the CLI surface; all existing usage examples in `docs/AGENT_CONTEXT.md` and `docs/User Workflow.txt` must be updated.

> **Implementation note (post-ship):** the actual CLI uses bare verbs `optimize` and `resolve-slots`, case-insensitive, with `--optimize`/`-o` and `--resolve-slots`/`-r` as aliases. The `-Optimize` form shown above is not what shipped. See `src/main.rs::parse_command`.

(If we ever want a third subcommand later, this scales cleanly.)

### Input file

`resolve-slots`:

- With no `--file` argument, calls `find_latest_stats_file` from `src/gearstats.rs` to locate the most recent `lgo_stats_*.toml` in the conventional `AllServers` directory (same logic the optimizer uses today).
- With `--file PATH`, reads that file instead.

### Output file

**Writes a new timestamped file rather than overwriting in place.** Suggested name: `lgo_stats_<original-timestamp>_resolved.toml` (or similar). Rationale:

- Preserves the bookmarklet's raw output for debugging.
- The optimizer's `find_latest_stats_file` then automatically picks up the resolved version on the next run (lexicographic timestamp sort — the `_resolved` suffix should sort after the original; **verify this assumption at implementation time** by checking how `find_latest_stats_file` sorts).
- If sort order is wrong, alternative: use a fresh `YYYYMMDD_HHMMSS` timestamp on the resolved file.

### Comment preservation

The bookmarklet's `.toml` contains header comments and per-item `# WARNING` lines. A plain `serde`/`toml` round-trip would discard them. Use **`toml_edit`** instead, which preserves formatting and comments.

User has approved adding `toml_edit` as a dependency provided it does not significantly complicate the code. If it turns out to be heavyweight or to require major restructuring, surface that before pulling it in.

### Side effect: Bug 5 (TOML formatting) gets fixed here

The resolver should re-emit items grouped by canonical slot order with divider comments between groups (per `docs/AGENT_CONTEXT.md` §5). Doing this in the resolver step is the natural place — the resolver already knows the canonical slot of each item and must rewrite the file anyway.

## 8. Code layout

### New module: `src/slot_resolver.rs`

Single module owns:
- Loading `data/lgo_items.json`.
- Building the in-memory `name → Slot` index.
- The end-to-end "read a .toml, rewrite it with resolved slots, write the new file" workflow.

User has approved this as a single new module rather than splitting items-DB loading into a separate `src/items_db.rs`. Rationale for the single-module decision: the loaded data is **not currently reused elsewhere** (stats come from the bookmarklet, not from the JSON). If a future feature wants to reuse the items DB, *then* split it out — premature splitting adds boilerplate without benefit.

### Approved public API sketch

```rust
// In src/slot_resolver.rs

pub struct ItemsDb {
    by_name: HashMap<String, Vec<DbItem>>,
}

pub struct DbItem {
    pub name: String,
    pub slot: Slot,             // canonical, already translated from PascalCase JSON
    // (item_level / quality fields omitted unless verification shows we need them
    // for name-collision disambiguation — see §9)
}

impl ItemsDb {
    /// Loads from data/lgo_items.json (default path).
    pub fn load_default() -> Result<Self, ItemsDbError>;

    /// Returns the primary slot for an item name, if known.
    /// Hides the multi-item Vec from callers; if disambiguation is ever needed,
    /// add a second method then.
    pub fn lookup(&self, name: &str) -> Option<Slot>;
}

pub enum ResolutionOutcome {
    Resolved { name: String, slot: Slot },
    Unknown  { name: String, reason: UnknownReason },
}

pub enum UnknownReason {
    NotInDb,           // legendary / renamed / stale dump
    // room here for future categories
}

pub struct Report {
    pub outcomes: Vec<ResolutionOutcome>,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

/// End-to-end: read .toml at `path`, look up each item via `db`,
/// write the resolved .toml to a new file, return summary.
pub fn resolve_stats_file(path: &Path, db: &ItemsDb) -> Result<Report, ResolveError>;
```

Decisions baked in (all user-approved):
- The DB pre-translates JSON PascalCase → canonical `Slot` **once at load time**, not on every lookup.
- `lookup` returns `Option<Slot>`. The `Vec<DbItem>` is internal because name collisions for slot purposes don't matter (see §9).
- `resolve_stats_file` returns a `Report` so `main.rs` can print a summary (e.g. *"47 resolved, 3 unknown"*) instead of silently writing the file.

### Wiring in `src/main.rs`

`main()` needs to branch on the first CLI argument:
- `resolve-slots` → call into `slot_resolver`.
- `-Optimize` → existing optimizer flow.
- Anything else → print usage and exit non-zero.

Keep the branching shallow and obvious. Resist the urge to introduce a heavy CLI framework (`clap`, etc.) for two subcommands.

## 9. Name collisions

Some items share names across tiers (same name, different `item_level`/`quality`). For slot purposes this is almost never a problem because the game's data model essentially never reuses a name across slot categories — re-used names virtually always resolve to the same slot.

**Decision:** First match wins. Do not load `item_level` / `quality` into `DbItem` for v1. If collisions ever cause a wrong slot in practice, add a tie-breaker (e.g. highest `item_level`) at that point.

## 10. Implementation order

1. **Verify the JSON schema.** Write a tiny one-off probe that loads `data/lgo_items.json`, prints the first 3 entries, and stops. Confirm the schema described in §3. If it has drifted, halt and report.
2. **Add the JSON-PascalCase → canonical-Slot translation** to `src/gear.rs`. Add tests covering all 19 variants, especially the four with punctuation differences (`Wrist1` → `Wrist (1)`, `MainHand` → `Main-hand`, etc.).
3. **Build `ItemsDb::load_default` + `lookup`.** Unit-test against a tiny synthetic JSON fixture (a few items covering at least one paired slot and one weapon).
4. **Build `resolve_stats_file`.** Use `toml_edit` so comments survive. Group output by canonical slot order with divider comments (kills Bug 5 here).
5. **Wire the subcommand into `main.rs`** with the `-Optimize` / `resolve-slots` split.
6. **Integration test** against the existing test `.toml` from the user's bookmarklet run (66 items including weapons, jewelry, and legendary unresolvables).
7. **Update `docs/User Workflow.txt` and `docs/AGENT_CONTEXT.md`** to reflect the new CLI surface and the new workflow step.

## 11. Open items / things still to verify at implementation time

A short list of "do not assume; check" items pulled together from the body of this doc:

- [x] Actual JSON schema of `data/lgo_items.json` matches §3 — verified by `slot_resolver`'s real-DB tests.
- [x] `src/optimizer.rs` super-candidate logic for paired slots really does draw from the union pool, not the `(1)`/`(2)`-tagged subsets — verified: `canonical_slot()` in `src/optimizer.rs` collapses `Wrist2`/`Finger2`/`Ear2` into their `*1` counterparts before the per-slot `pools` HashMap is keyed, and `build_pairs` enumerates all `i ≤ j` pairings over that union pool; `original_slot` is used only as a sort tiebreaker (preferring `a→slot1, b→slot2`), never as a filter.
- [x] `find_latest_stats_file` sort order puts `_resolved` files after their originals — verified by the `resolved_path_sorts_after_original_lexicographically` test in `src/slot_resolver.rs`.
- [x] `toml_edit` is not heavyweight or invasive enough to merit pulling in — verified: used cleanly throughout `src/slot_resolver.rs`.
- [x] No reused-name-across-different-slots cases exist in `data/lgo_items.json` that would invalidate the first-match-wins decision in §9 — verified by `no_item_name_maps_to_multiple_slots_in_lgo_items_json` in `tests/resolve_slots_integration.rs`.

## 12. Out of scope for this work

Explicitly **not** part of this change, to avoid scope creep:

- Re-resurrecting the deleted `src/wiki.rs`, `src/cache.rs`, `src/merge.rs`, etc. The bookmarklet replaces them; only slot lookup is being restored.
- Moving slot detection into the in-game Lua plugin. This was raised as a future architectural question in `docs/AGENT_CONTEXT.md` §6 Bug 3 and explicitly deferred — investigating the Turbine Lua API in chat is the very thing that caused the context-loss event motivating `AGENT_CONTEXT.md`.
- HTML report output (a separate planned feature, see `docs/AGENT_CONTEXT.md` §9).
- Bug 2 (the `mapSlot()` fallback). Once the bookmarklet stops being trusted for slots at all, Bug 2 becomes even more theoretical than it is now. Defer indefinitely.
- Bug 4 (URL encoding / `&redirects=1`). Independent of slot resolution; should still be fixed, but on its own schedule.

## 13. Definition of done

- `cargo build` clean, `cargo test` green.
- Running `lgo resolve-slots --file <bookmarklet-output.toml>` on the existing 66-item test input produces a new `.toml` where:
  - All 20 equipped items have a correct canonical slot.
  - All non-legendary candidate items have a correct canonical slot.
  - Legendary / renamed items retain `slot = "Unknown"` with their existing `# WARNING` comments preserved.
  - Items are grouped by canonical slot order with divider comments (Bug 5 fixed as a side effect).
  - Header comments from the bookmarklet output are preserved.
- Running `lgo -Optimize tm:450000 cr:350000 ...` against the resolved file completes without `unrecognised slot` errors.
- `docs/User Workflow.txt` and `docs/AGENT_CONTEXT.md` updated; `docs/AGENT_CONTEXT.md` Bug 3 marked ✅ FIXED.
