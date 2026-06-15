# LGO Agent Context

**Purpose:** A durable, in-repo brief so an AI coding agent can resume work on LGO without losing context across sessions. Read this first at the start of every session.

**Working branch:** `main`.

---

## 1. What LGO is

LGO (LOTRO Gear Optimizer) is a two-part personal tool for the MMO *Lord of the Rings Online*. The target user base is a small group of personal acquaintances of the author of this project. There will be no general public release, and there are no commercial needs or long-term engineering best-practices needs.

1. A **Lua in-game plugin** (`src/lgo.lua`) that exports the player's equipped gear plus the contents of a Shared Storage chest named `lgo`, writing one file to `Documents\The Lord of the Rings Online\PluginData\<account>\AllServers\`:
   - `lgo_gearlist_<character>_<timestamp>.plugindata` — a flat list of equipped + chest item names, plus the character's class and base stats (input for the bookmarklet).
2. A **Rust CLI optimizer** (`src/main.rs` etc.) that reads the plugindata + a stats `.toml` file and finds the best gear combination for a set of stat goals (lexicographic priority).

Item stats cannot be fetched programmatically from lotro-wiki.com — Cloudflare blocks the Rust binary. The workaround is a **bookmarklet** (`bookmarklet/lgo_bookmarklet.html`): the user opens lotro-wiki.com, clicks the bookmarklet, pastes the itemnames data, and gets back a `.toml` they save into the AllServers directory.

---

## 2. User workflow (current, end-to-end)

See 'docs/User Workflow.txt' for the full step-by-step.
The short form:
- User exports a file from the in-game plug-in: lgo_gearlist_`<character>`_`<timestamp>`.plugindata.
- User uses the bookmarklet, pasting in the .plugindata contents, which
then fetches each item's stats from lotro-wiki.com to create a a TOML
file containing each item's name, slot, and stats.
- User saves the bookmarklet's output as a file named: lgo_`<character>`_stats.toml.
- User invokes 'lgo resolve-slots' to merge that list into a file named: lgo_`<character>`_gear.toml.
- User invokes 'lgo optimize' which reads the canonical file and provides
a final optimization report according to user's specified stats of interest.

---

## 3. Repo layout (relevant)

- `src/main.rs` — CLI entry: find plugindata → find `.toml` → optimize → report.
- `src/plugindata.rs` — hand-written recursive-descent Lua parser; produces `PluginExport { character, class, base_stats }`.
- `src/gearstats.rs` — TOML reader (`read_stats_file`) + `find_latest_stats_file`; `parse_slot_str` enforces the canonical 19-string slot allow-list.
- `src/optimizer.rs` — two-phase compatibility-filter + safe lexicographic narrowing; super-candidates for paired Wrist/Finger/Ear slots; `MAX_CANDIDATES_PER_SLOT = 8`; infeasible-greedy fallback.
- `src/stat.rs` — `Stat` enum, `TRACKED_STATS` (14, canonical order), CLI abbrev parsing, `StatGoal`.
- `src/gear.rs` — `Slot` enum (19 variants; `CraftItem`/`Bridle` excluded), `from_json_variant`, `Display` impl, `GearItem`, `GearSet`.
- `src/report.rs` — terminal report formatter.
- `src/lgo.lua`, `src/Main.lua`, `src/lgo.plugin` — in-game plugin (tested, working).
- `bookmarklet/lgo_bookmarklet.html` — the bookmarklet HTML page; handles direct lookups, disambiguation auto-pick (via MediaWiki `prefixsearch`), and outcome-typed reporting (see Bug 9).
- `data/items.xml` (~71 MB), `data/lgo_items.json` (~8 MB), `data/progressions.xml` (~3.6 MB) — canonical game data dumps.
- `src/build_db.rs` — offline database builder, exposed as `lgo build-db [options]`. Reads `data/items.xml` + `data/progressions.xml`, writes `data/lgo_items.json`. Run via `cargo run --release -- build-db` (dev) or `lgo build-db` (user). Always overwrites the output file.
- `TestData/` — committed test fixtures: bookmarklet input (`lgo_gearlist_Thalya_*.plugindata`), bookmarklet output with an outdated-format name (`lgo_stats_Thalya_*.toml`), and a one-off plugin-API probe dump (`lgo_probe_Thalya_20260607_205655.plugindata` — historical reference for what the Turbine API exposes per item; see Bug 9 and §7).
- `docs/` — live docs: `User Workflow.txt`, `BUG_HISTORY.md`, `RESOLVER_DESIGN.md`, `lgo_reference_slots.md`, `lgo_reference_stats.md`, `TOML Analysis.txt`. Historical design docs (kept for traceability, do not treat as live spec): `Merge Coding Prompt.txt`, `User Story & Hand-Edit-Tracking Approach.txt`. See §10 for the rejection rationale.
- `docs/` — design notes (`Merge Coding Prompt.txt`, `TOML Analysis.txt`, `User Story & Hand-Edit-Tracking Approach.txt`, `User Workflow.txt`, `lgo_reference_slots.md`, `lgo_reference_stats.md`, `Command Line Reference.txt`, `Test_Output_01.txt`, `RESOLVER_DESIGN.md`, plus `probes/` for one-off diagnostic plugin-data inputs).
- `SSG_U25_LuaDocumentation/` — **DO NOT ingest in chat.** Large UTF-16 HTML dumps that blow up the model's context window and cause mid-session amnesia. If the Turbine Lua API needs investigation, ask the user to paste a representative snippet.
- `GaranStuff/` — ignore for now.

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

The TOML loader exact-matches against this list. Anything else → fatal `unrecognised slot 'X'` error.

### 4.2 The 14 tracked stats (canonical order)

`Armor`, `CriticalRating`, `Finesse`, `PhysicalMastery`, `TacticalMastery`, `OutgoingHealing`, `Resistance`, `CriticalDefense`, `IncomingHealing`, `Block`, `Parry`, `Evade`, `PhysicalMitigation`, `TacticalMitigation`.

Two-letter CLI abbreviations: `am cr fn pm tm oh rs cd ih bl pa ev pt tt`.

### 4.3 The three slot vocabularies (important — easy to confuse)

| # | Where it lives | Style | Example |
|---|---|---|---|
| 1 | `data/items.xml` (game data dump) | UPPERCASE enum values | `HEAD`, `SHOULDER`, `HAND`, `FEET`, `CHEST`, `LEGS` |
| 2 | lotro-wiki.com `{{Item Tooltip}}` `slot=` field | Free-text, editor-typed, no enforced allow-list | `Gloves`, `Wrist`, `Ear`, `Back`, `Feet`, `Shoulder`/`Shoulders` |
| 3 | `src/gear.rs` `Slot::Display` (canonical) | Curated display strings | `Wrist (1)`, `Main-hand`, `Class Item` |

The Rust code is vocabulary #3. The bookmarklet currently translates #2 → #3 via a hand-maintained `SLOT_MAP` in `lgo_bookmarklet.html`. `data/items.xml` (#1) is the game's source of truth for which slot an item goes in.

**A fourth representation** also exists: `data/lgo_items.json`'s `slot` values use bare PascalCase variant names (`Head`, `Wrist1`, `MainHand`, `ClassItem`, etc.) — this is Rust's *default* enum serialization. The `resolve-slots` subcommand bridges #4 → #3 via `Slot::from_json_variant`.

---

## 5. `.toml` format expected by `gearstats::read_stats_file`

```toml
[[item]]
slot               = "Head"
name               = "Forgotten Elvish Healer's Hood"
Armor              = 0
CriticalRating     = 12345
# ...all 14 tracked stats, canonical order...
TacticalMitigation = 0
```

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
- Slot strings, stat names, and TOML field formatting must round-trip exactly through `parse_slot_str` and the canonical 14-stat list. Do not invent or paraphrase.
- The bookmarklet's `SLOT_MAP` is a translation table between two free-text vocabularies and a rigid one. There is **no canonical translation table** between the wiki's free-text `slot=`/`type=` and the Rust `Slot` enum — every entry in `SLOT_MAP` was added by hand in response to a discovered mismatch.
- **The in-game Turbine plugin API cannot distinguish player-crafted items from non-crafted items.** Verified empirically via a temporary `/lgo probe` subcommand (since removed) that dumped every callable on `Item` and `ItemInfo`. The resulting `TestData/lgo_probe_Thalya_20260607_205655.plugindata` is the empirical evidence. Don't waste a session re-investigating this — crafted-item handling lives in the bookmarklet (see Bug 9).
- **`GetDescription()` on `ItemInfo` returns `<string table error; tableDID [...] token [...]>` for *all* gear items** on the current client. This is a long-standing wiki-side or engine-side string-table failure, not something the plugin can fix. The probe confirmed it succeeds for non-gear items (e.g. fireworks) but fails uniformly across gear.
- **`info.__implementation` is engine-private userdata:** no enumerable metatable methods, no addressable fields. Don't try to use it.
- When the agent finds itself unsure what was previously decided, **ask the user** rather than reconstructing from inference. Reconstruction from inference is what produced the speculative "Cloakroom of Dol Amroth" episode in earlier sessions; the user's tolerance for it is low and rightly so.

---

## 8. Character context (test data)

- Character: **Thalya**
- Class: **Lore-master**
- Base stats: Might 5300, Agility 2650, Vitality 10200, Will 7950, Fate 4000.
- Bookmarklet fixture: `TestData/lgo_stats_Thalya_20260525_215012.toml` — the bookmarklet's TOML output for a 66-item Lore-master gear set; used by `tests/resolve_slots_integration.rs` (6 of 7 tests depend on it).
- Probe data (historical reference, see Bug 9): `TestData/lgo_probe_Thalya_20260607_205655.plugindata` — per-item Turbine API dump for a hand-picked set of 7 items (3 paired crafted/non-crafted comparisons plus one ignorable fireworks).

---

## 9. Likely next features (after the bookmarklet bugs)

- Optimizer --toml-file flag (specify input .toml)
- Make Wiki look-up stats assume max item level
- Ignore craft-tool and bridle slots. Optimizer is barfing on the "unknown" slot names again:
  . 'Craft Tool'
  . 'Bridle'
  . Possibly others as well
- Change all spellings of 'Armor' to 'Armour'
- Get the plug-in to deposit the .plugindata and .toml files in an 'lgo' sub-folder
  . \\Documents\The Lord of the Rings Online\PluginData\Thalya\AllServers\lgo

---

## 10. Deferred work (don't lose track of these)

These are known, decided-but-not-urgent items. Do **not** silently fold them into other PRs; track and address explicitly.

- **Change CSS properties to ensure the bookmarklet's progress display appears at the top of the window, not the bottom. Per previous coding assistant: "`Add position: fixed; top: 0.5rem; right: 0.5rem; background: white; border: 2px solid #333; padding: 0.5rem; z-index: 9999;` (or similar) so it floats above the wiki content regardless of scroll position."
- **Bookmarklet test harness.** The bookmarklet currently has no automated tests. Adding one would mean introducing a JS test runner and mocking `fetch()` of the wiki API. Decision: don't bother unless a regression slips through manual testing badly enough to make it worth the setup cost.
- **Bug 2 (`mapSlot()` fallback).** Latent, no observed symptom. Largely moot once the resolver overrides slot decisions anyway. Leave alone unless it produces a real failure.
- **Hand-edit preservation across re-runs:** implemented via preserve-by-default merge in `resolve-slots`. The `[__user_edits__]` design from `docs/Merge Coding Prompt.txt` and `docs/User Story & Hand-Edit-Tracking Approach.txt` was rejected in favour of the simpler preserve-by-default model. Those two design docs are now historical.
- **Rename detection.** The merge step matches items by exact byte-for-byte name. If the wiki renames an item between exports, or if a Unicode encoding glitch alters a character, the merge will treat the renamed item as a removal-and-add pair rather than the same item, silently dropping the user's hand-edits. Accepted risk; revisit if it becomes a real problem.
