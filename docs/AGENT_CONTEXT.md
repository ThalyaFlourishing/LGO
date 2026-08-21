# LGO Agent Context

**Purpose:** A durable, in-repo brief so an AI coding agent can resume work on LGO without losing context across sessions. Read this first at the start of every session.

**Default base branch:** `main`.

**Model guidance:** Before editing Rust source, see `docs/MODEL_GUIDANCE.md` for which class of AI model suits each file. For current optimizer semantics and search design, see `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md`. `src/optimizer.rs` and `src/slot_resolver.rs` need a frontier model for non-trivial edits; most other files are fine with a cheap one.

---

## 1. What LGO is

LGO (LOTRO Gear Optimizer) is a two-part personal tool for the MMO *Lord of the Rings Online*. The target user base is a small group of personal acquaintances of the author of this project. There will be no general public release, and there are no commercial needs or long-term engineering best-practices needs. It has not yet been released at all, in fact, so there are currently no concerns about backwards compatibility.

1. A **Lua in-game plugin** (`src/lgo.lua`) that exports the player's equipped gear plus the contents of a Shared Storage chest named `lgo`, writing one file to `Documents\The Lord of the Rings Online\PluginData\<account>\AllServers\`:
   - `lgo_<character-name>_gearNames_<timestamp>.plugindata` — a flat list of equipped + chest item names, plus the character's class and base stats (input for the bookmarklet).
2. A **Rust CLI optimizer** (`src/main.rs` etc.) that reads the plugindata-derived canonical gear `.toml` file and finds the best gear combination for a set of priority-ordered stat goals using the clamped-satisfaction objective in `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md`.

Item stats cannot be fetched programmatically from lotro-wiki.com — Cloudflare blocks the Rust binary. The workaround is a **bookmarklet** (`bookmarklet/lgo_bookmarklet.html`): the user opens lotro-wiki.com, clicks the bookmarklet, pastes the plugindata, and the bookmarklet either opens a Save As dialog (Chromium browsers, via `showSaveFilePicker`) or saves to the browser's Downloads folder (other browsers).

---

## 2. User workflow (current, end-to-end)

See 'docs/User Workflow.txt' for the full step-by-step.
The short form:
- User exports a file from the in-game plug-in: `lgo_<character-name>_gearNames_<timestamp>.plugindata`.plugindata.
- User uses the bookmarklet, pasting in the .plugindata contents, which
then fetches each item's stats from lotro-wiki.com to create a a TOML
file containing each item's name, slot, and stats.
- User clicks **Save TOML...** to save the bookmarklet's output as: lgo_`<character>`_gearStats.toml.
- User invokes 'lgo resolve-slots' to merge that list into a file named: lgo_`<character>`_gearReady.toml.
- User hand-edits persistent corrections, including per-item essence totals, in `gearReady.toml` only.
- User invokes 'lgo optimize' which reads the canonical file and provides
a final optimization report according to user's specified stats of interest.

---

## 3. Repo layout (relevant)

- `src/main.rs` — CLI entry: find plugindata → find `.toml` → optimize → report.
- `src/plugindata.rs` — hand-written recursive-descent Lua parser; produces `PluginExport { character, class, base_stats }`.
- `src/gearstats.rs` — TOML reader (`read_stats_file`) + `find_latest_stats_file`; `parse_slot_str` enforces the canonical 19-string slot allow-list. `read_stats_file` **skips** items with non-canonical slots rather than erroring: silently for `slot = "Unknown"`, with a stderr warning for any other unrecognised value (Bridle, tool, Tool, hand-edited typos, etc.). See Bug 2 / Bug 10 in `BUG_HISTORY.md`.
- `src/optimizer.rs` — exact optimizer: clamped-satisfaction comparator, per-pool dominance filtering, branch-and-bound search, paired-slot super-candidates for Wrist/Finger/Ear, and a hard `MAX_CANDIDATES_PER_SLOT = 8` refusal contract. Verified against a brute-force oracle by a differential fuzzer.
- `src/stat.rs` — `Stat` enum, `TRACKED_STATS` (16, canonical order), CLI abbrev parsing, `StatGoal`.
- `src/gear.rs` — `Slot` enum (19 variants; `CraftItem`/`Bridle` excluded), `from_json_variant`, `Display` impl, `GearItem`, `GearSet`.
- `src/report.rs` — terminal report formatter.
- `src/lgo.lua`, `src/Main.lua`, `src/lgo.plugin` — in-game plugin (tested, working).
- `bookmarklet/lgo_bookmarklet.html` — the bookmarklet HTML page; handles direct lookups, disambiguation auto-pick (via MediaWiki `prefixsearch`), outcome-typed reporting (see Bug 9), a pinned-top-right progress panel, and a programmatic Save TOML... button (`showSaveFilePicker` on Chromium, Blob/`<a download>` fallback elsewhere). `mapSlot()` returns `"Unknown"` for any wiki vocabulary not in `SLOT_MAP` (Bug 2 fix).
- `data/items.xml` (~71 MB), `data/lgo_items.json` (~8 MB), `data/progressions.xml` (~3.6 MB) — canonical game data dumps.
- `src/build_db.rs` — offline database builder, exposed as `lgo build-db [options]`. Reads `data/items.xml` + `data/progressions.xml`, writes `data/lgo_items.json`. Run via `cargo run --release -- build-db` (dev) or `lgo build-db` (user). Always overwrites the output file.
- `TestData/` — committed test fixtures, all for character Thalya:
  - `lgo_<character-name>_gearNames_<timestamp>.plugindata` — fresh in-game plugin export (input for the bookmarklet).
  - `lgo_Thalya_gearStats.toml` — bookmarklet's TOML output (input for `resolve-slots`); contains a mix of canonical slots, `slot = "Unknown"` entries, and pre-Bug-2-fix wiki-vocabulary slots (`"Shoulder"`, `"Gloves"`).
  - `lgo_Thalya_gearReady.toml` — already-resolved canonical gear file (input for `optimize`).
- `docs/` — live docs: `User Workflow.txt`, `BUG_HISTORY.md`, `lgo_reference_slots.md`, `lgo_reference_stats.md`, `Command Line Reference.txt`, `MODEL_GUIDANCE.md`, and `Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` (authoritative optimizer spec). The optimizer audit chain and PR prompt docs under `docs/Optimizer_Overhaul/` are historical records retained for traceability; do not treat them as the live optimizer spec.

---

## 4. Canonical reference data

### 4.1 The 19 canonical slot strings (from `src/gear.rs` Display + `src/gearstats.rs::parse_slot_str`)

```
Head            Wrist (1)       Pocket
Chest           Wrist (2)       Main-hand
Legs            Neck            Off-hand
Hands           Finger (1)      Ranged
Feet            Finger (2)      Class Item
Shoulders       Ear (1)
Back            Ear (2)
```

The TOML loader exact-matches against this list. Items with any other slot string are **skipped** by `read_stats_file` (silently for `"Unknown"`, with a stderr warning otherwise). The optimizer continues with the items that did match.

### 4.2 The 16 tracked stats (canonical order)

`Morale`, `Power`, `Armor`, `CriticalRating`, `Finesse`, `PhysicalMastery`, `TacticalMastery`, `OutgoingHealing`, `Resistance`, `CriticalDefense`, `IncomingHealing`, `Block`, `Parry`, `Evade`, `PhysicalMitigation`, `TacticalMitigation`.

Two-letter CLI abbreviations: `ml pw am cr fn pm tm oh rs cd ih bl pa ev pt tt`.

### 4.3 The three slot vocabularies (important — easy to confuse)

| # | Where it lives | Style | Example |
|---|---|---|---|
| 1 | `data/items.xml` (game data dump) | UPPERCASE enum values | `HEAD`, `SHOULDER`, `HAND`, `FEET`, `CHEST`, `LEGS` |
| 2 | lotro-wiki.com `{{Item Tooltip}}` `slot=` field | Free-text, editor-typed, no enforced allow-list | `Gloves`, `Wrist`, `Ear`, `Back`, `Feet`, `Shoulder`/`Shoulders` |
| 3 | `src/gear.rs` `Slot::Display` (canonical) | Curated display strings | `Wrist (1)`, `Main-hand`, `Class Item` |

The Rust code is vocabulary #3. The bookmarklet translates #2 → #3 via a hand-maintained `SLOT_MAP` in `lgo_bookmarklet.html`; anything not in `SLOT_MAP` is converged to the literal string `"Unknown"` (Bug 2 fix). `data/items.xml` (#1) is the game's source of truth for which slot an item goes in.

**A fourth representation** also exists: `data/lgo_items.json`'s `slot` values use bare PascalCase variant names (`Head`, `Wrist1`, `MainHand`, `ClassItem`, etc.) — this is Rust's *default* enum serialization. The `resolve-slots` subcommand bridges #4 → #3 via `Slot::from_json_variant`.

---

## ## 5. `.toml` format expected by `gearstats::read_stats_file` begins with a top header containing `character`, `class`, and then `[InnateStats]` as the last pre-items block. After that, the format for each item is as follows:

```toml
[[item]]
slot               = "Head"
name               = "Forgotten Elvish Healer's Hood"
Morale             = 0
Power              = 0
Armor              = 0
CriticalRating     = 12345
# ...all 16 tracked stats, canonical order...
TacticalMitigation = 0
[item.EssenceTotals]
Morale             = 0
Power              = 0
Armor              = 0
CriticalRating     = 0
# ...all 16 tracked stats, canonical order...
TacticalMitigation = 0
```

`gearReady.toml` is the canonical hand-edited file. Each item has an attached
`[item.EssenceTotals]` child table for user-maintained per-item essence overlays.
The loader immediately adds those values to the base item stats and discards the
base-vs-essence separation in the runtime model. Unknown stat keys in either the
base item block or `EssenceTotals` are hard errors; omitted stats are treated as
zero.

### `two_handed` generated metadata

Two-handed `Main-hand` items carry `two_handed = true` between `name` and the
stat block:

```toml
[[item]]
slot       = "Main-hand"
name       = "Example Greatsword"
two_handed = true
# ...stats and [item.EssenceTotals] as usual...
```

- **Source of truth:** the `precludedSlots` attribute in `data/items.xml`
  (present exactly on `MAIN_HAND` items that block `OFF_HAND`). `build-db`
  emits `two_handed: true` into `data/lgo_items.json`, and `resolve-slots`
  carries it into `gearReady.toml`. `optimize` never reads the items DB —
  the TOML flag is its only source.
- **Generated, not hand-edited:** on every merge, `resolve-slots` refreshes
  the flag from the DB for items the DB knows (adding it or removing a stale
  one) while still preserving hand-edited stats/essence totals. For items
  *not* in the DB (legendary/renamed), a user-provided `two_handed = true`
  is preserved.
- The flag is omitted unless true; missing means false. It must never appear
  under `[item.EssenceTotals]`.
- **Optimizer effect:** `MainHand` + `OffHand` form one combined search pool
  of legal hand configurations; a two-handed main hand only ever pairs with
  an empty off-hand, which suppresses real off-hand candidates structurally.
  The final report shows the usual empty-slot placeholder, with no
  "2-Handed" tag.

### Outcome-typed comments emitted by the bookmarklet

The bookmarklet annotates each `[[item]]` with an outcome-typed comment when resolution wasn't a clean direct hit. Five forms exist:

```
# AUTO-PICKED highest-item-level variant: Item:Foo (Item Level 563)
# UNRESOLVED: multiple wiki variants exist — you should hand-edit stats
# UNRESOLVED: wiki page has no Item Tooltip (likely legendary) — you should hand-edit stats
# UNRESOLVED: no wiki page found — you should hand-edit stats
# UNRESOLVED: fetch error — retry or you should hand-edit stats
```

See Bug 7 / Bug 9 for the rationale. The downstream `resolve-slots` step preserves these comments via `toml_edit`.

### Formatting decisions (now handled by `resolve-slots`)

- Group `[[item]]` entries by slot, in canonical `Slot::ALL` order.
- Insert a visible divider comment between slot groups.

The bookmarklet emits items in fetch order; `resolve-slots` re-groups them.

---

## 6. Confirmed bug list has been moved to `docs/BUG_HISTORY.md`

---

## 7. Honest tool / methodology notes for the agent

- The `bing-search` tool returns LLM-summarized results, not raw page content. It is the **wrong instrument** for "what does this wiki template actually say." For wiki source, ask the user to open the page in a browser and view source.
- `data/items.xml` is too large for the code-search index (~384 KB threshold) and too large for `getfile` to be useful. To inspect it, ask the user to paste a representative snippet.
- `data/lgo_items.json` (~8 MB) is at the edge of `getfile`'s comfort zone. The first ~125 entries are reliably retrievable via `getfile`, which is plenty for schema verification. For deeper questions (collisions, counts, name lookups), ask the user to run a `grep` / `Select-String` command locally and paste the output.
- `SSG_U25_LuaDocumentation/*.html` files are UTF-16 with BOM. Pulling several into chat blows past the model's context window and causes mid-session amnesia. **Do not ingest them in chat — hard rule.**
- Slot strings, stat names, and TOML field formatting must round-trip exactly through `parse_slot_str` and the canonical 16-stat list. Do not invent or paraphrase.
- Filename discovery for `lgo_<character>_gearStats.toml` and `lgo_<character>_gearReady.toml` is case-insensitive on the character segment. On Windows, names differing only by case are the same file, so case-only "collisions" are not a real runtime condition in LGO's target environment.
- The bookmarklet's `SLOT_MAP` is a translation table between two free-text vocabularies and a rigid one. There is **no canonical translation table** between the wiki's free-text `slot=`/`type=` and the Rust `Slot` enum — every entry in `SLOT_MAP` was added by hand in response to a discovered mismatch.
- **The in-game Turbine plugin API cannot distinguish player-crafted items from non-crafted items.** Verified empirically via a temporary `/lgo probe` subcommand (since removed) that dumped every callable on `Item` and `ItemInfo`. Don't waste a session re-investigating this — crafted-item handling lives in the bookmarklet (see Bug 9).
- **`GetDescription()` on `ItemInfo` returns `<string table error; tableDID [...] token [...]>` for *all* gear items** on the current client. This is a long-standing wiki-side or engine-side string-table failure, not something the plugin can fix. The probe confirmed it succeeds for non-gear items (e.g. fireworks) but fails uniformly across gear.
- **`info.__implementation` is engine-private userdata:** no enumerable metatable methods, no addressable fields. Don't try to use it.
- **`showSaveFilePicker()` requires a *fresh* user activation.** The bookmarklet's original click is consumed by the multi-second wiki fetch loop, so the Save TOML... button is necessary — calling the picker after `await` boundaries throws `SecurityError`. The Blob/`<a download>` fallback path has the same activation requirement. Verified empirically; don't try to "auto-save" without a button.
- When the agent finds itself unsure what was previously decided, **ask the user** rather than reconstructing from inference. Reconstruction from inference is what produced the speculative "Cloakroom of Dol Amroth" episode in earlier sessions; the user's tolerance for it is low and rightly so.
- **`toml_edit` serializes top-level tables by internal `position` index, not map insertion order.** `doc.remove(key)` + `doc.insert(key, ...)` does NOT move a table in the rendered output. To relocate a table you must call `set_position()` on it *and* renumber its sibling tables consistently (see `reorder_resolved_header_before_items` in `src/slot_resolver.rs`, and Bug 10 in `docs/BUG_HISTORY.md`). Note `push_group` renumbers every `[[item]]` table to `0..n` — any header table must be positioned relative to that. Root-level *values* (`character`, `class`) are unaffected; they always render before all tables.
- **`gearReady.toml` is both output and next-run input.** Any layout/decor bug in the merge path compounds across runs (each run's output seeds the next run's parse positions), producing drift that convincingly masquerades as a race condition or nondeterminism. It never is — re-running from an identical file snapshot reproduces byte-identically. Diagnose by diffing consecutive outputs, and guard with bit-identical idempotency tests (modulo the `# gearReady.toml updated:` timestamp line).

---

## 8. Character context (test data)

- Character: **Thalya**
- Class: **Lore-master**
- Base stats: Might 5300, Agility 2650, Vitality 10200, Will 7950, Fate 4000.
- Plugindata fixture: `TestData/lgo_Thalya_gearNames_<time stamp>.plugindata` — fresh in-game plugin export (input for the bookmarklet).
- Bookmarklet-output fixture: `TestData/lgo_Thalya_gearStats.toml` — 66-item TOML; used by `tests/resolve_slots_integration.rs`. Contains a mix of canonical slots, `slot = "Unknown"` entries, and (intentionally) some pre-Bug-2-fix wiki-vocabulary slots like `"Shoulder"` and `"Gloves"` — exercises the resolver's name-based slot canonicalisation.
- Canonical-gear fixture: `TestData/lgo_Thalya_gearReady.toml` — already-resolved gear file (input for `optimize`).

---

## 9. Likely next features

- Minor bug fixes
- Identify items which can be removed from pool
- Construct HTML reports

---

## 10. Deferred work (don't lose track of these)

These are known, decided-but-not-urgent items. Do **not** silently fold them into other PRs; track and address explicitly.

- **Bookmarklet test harness.** The bookmarklet currently has no automated tests. Adding one would mean introducing a JS test runner and mocking `fetch()` of the wiki API. Decision: don't bother unless a regression slips through manual testing badly enough to make it worth the setup cost.
- **Hand-edit preservation across re-runs:** implemented via preserve-by-default merge in `resolve-slots`. The `[__user_edits__]` design from `docs/Merge Coding Prompt.txt` and `docs/User Story & Hand-Edit-Tracking Approach.txt` was rejected in favour of the simpler preserve-by-default model. Those two design docs are now historical.
- **Rename detection.** The merge step matches items by exact byte-for-byte name. If the wiki renames an item between exports, or if a Unicode encoding glitch alters a character, the merge will treat the renamed item as a removal-and-add pair rather than the same item, silently dropping the user's hand-edits. Accepted risk; revisit if it becomes a real problem.

---