# LGO Agent Context

**Purpose:** A durable, in-repo brief so an AI coding agent can resume work on LGO without losing context across sessions. Read this first at the start of every session.

**Working branch:** `main`. (The previous `The-Browser-Method` working branch has been merged in and retired.)

---

## 1. What LGO is

LGO (LOTRO Gear Optimizer) is a two-part personal tool for the MMO *Lord of the Rings Online*:

1. A **Lua in-game plugin** (`src/lgo.lua`) that exports the player's equipped gear plus the contents of a Shared Storage chest named `lgo`, writing two files to `Documents\The Lord of the Rings Online\PluginData\<account>\AllServers\`:
   - `lgo_export_<character>_<timestamp>.plugindata` — equipped + chest items with slot/category data.
   - `lgo_itemnames_<character>_<timestamp>.plugindata` — flat list of all item names (input for the bookmarklet).
2. A **Rust CLI optimizer** (`src/main.rs` etc.) that reads the plugindata + a stats `.toml` file and finds the best gear combination for a set of stat goals (lexicographic priority).

Item stats cannot be fetched programmatically from lotro-wiki.com — Cloudflare blocks the Rust binary. The workaround is a **bookmarklet** (`bookmarklet/lgo_bookmarklet.html`): the user opens lotro-wiki.com, clicks the bookmarklet, pastes the itemnames data, and gets back a `.toml` they save into the AllServers directory.

---

## 2. User workflow (current, end-to-end)

1. Put candidate gear in the in-game Shared Storage chest named `lgo`.
2. Run `/lgo export` in-game → two `.plugindata` files are written.
3. Open https://lotro-wiki.com in a browser.
4. Click the **LGO Stats** bookmarklet.
5. Paste the contents of `lgo_itemnames_*.plugindata` when prompted.
6. The bookmarklet builds a `.toml`. The summary panel categorises items into three groups: directly resolved, auto-picked from disambiguation variants (informational — the chosen wiki page is listed for audit), and needs-hand-edit. Items in the last group are written with all stats `= 0` and a typed `# UNRESOLVED: ...` comment explaining why; the user fills these in by inspecting their gear in-game.
7. Save the `.toml` to the `AllServers` directory.
8. Run `lgo resolve-slots` — reads the most recent `lgo_stats_*.toml`, looks each item up in `data/lgo_items.json`, and writes a sibling `lgo_stats_*_resolved.toml` with canonical slots and slot-grouped output.
9. Run `lgo optimize tm:450000 cr:350000 fn:0` (etc.) — Rust auto-detects the most recent `.toml` (the `_resolved` one) and `.plugindata`, runs the optimizer, prints the result.

---

## 3. Repo layout (relevant)

- `src/main.rs` — CLI entry: find plugindata → find `.toml` → optimize → report.
- `src/plugindata.rs` — hand-written recursive-descent Lua parser; produces `PluginExport { character, class, base_stats, equipped, candidates }`.
- `src/gearstats.rs` — TOML reader (`read_stats_file`) + `find_latest_stats_file`; `parse_slot_str` enforces the canonical 19-string slot allow-list.
- `src/optimizer.rs` — two-phase compatibility-filter + safe lexicographic narrowing; super-candidates for paired Wrist/Finger/Ear slots; `MAX_CANDIDATES_PER_SLOT = 8`; infeasible-greedy fallback.
- `src/stat.rs` — `Stat` enum, `TRACKED_STATS` (14, canonical order), CLI abbrev parsing, `StatGoal`.
- `src/gear.rs` — `Slot` enum (19 variants; `CraftItem`/`Bridle` excluded), `from_plugin_index`, `from_json_variant`, `Display` impl, `GearItem`, `GearSet`.
- `src/report.rs` — terminal report formatter.
- `src/lgo.lua`, `src/Main.lua`, `src/lgo.plugin` — in-game plugin (tested, working).
- `bookmarklet/lgo_bookmarklet.html` — the bookmarklet HTML page; handles direct lookups, disambiguation auto-pick (via MediaWiki `prefixsearch`), and outcome-typed reporting (see Bug 9).
- `data/items.xml` (~71 MB), `data/lgo_items.json` (~8 MB), `data/progressions.xml` (~3.6 MB) — canonical game data dumps.
- `TestData/` — committed test fixtures: bookmarklet input (`lgo_itemnames_Thalya_*.plugindata`), bookmarklet output (`lgo_stats_Thalya_*.toml`), and a one-off plugin-API probe dump (`lgo_probe_Thalya_20260607_205655.plugindata` — historical reference for what the Turbine API exposes per item; see Bug 9 and §7).
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

## 6. Confirmed bug list (bookmarklet — current focus)

### Bug 1 — `Shoulder` vs `Shoulders` ✅ FIXED

`SLOT_MAP` had no entry for the wiki's `Shoulder`/`Shoulders` text; `mapSlot()` fell through and emitted the raw string `"Shoulder"`, which `parse_slot_str` rejects. Fixed by adding both `"shoulder"` and `"shoulders"` (lower-cased keys) to `SLOT_MAP`.

### Bug 2 — `mapSlot()` fallback leaks raw wiki text ⏸ DEFERRED

`mapSlot()` line 157: `return mapped || cleaned;` — when a slot string isn't in `SLOT_MAP`, returns the raw wiki value, which will fail `parse_slot_str` later. **Currently latent**: no observed example of raw wiki text reaching the TOML output.

*(Largely moot once the resolver lands: the bookmarklet stops being trusted for slots at all, so even raw wiki text leaking through `mapSlot` will be overwritten by `resolve-slots`.)*

### Bug 3 — Items with parsed stats emit `slot = "Unknown"` ✅ FIXED

Fixed by the `resolve-slots` subcommand (see `docs/RESOLVER_DESIGN.md`). Wired into the CLI in PR #18 and integration-tested against the 66-item bookmarklet fixture in PR #19.

Observed examples:
- `Faded Watcher's Bracers` — Armor 8631, Finesse 6583 — slot Unknown.
- `Resolute Sword of Old Eregion` — Finesse 4210 — slot Unknown.

**Root cause** (confirmed from the Item_Tooltip template documentation):
- `slot=` is documented as "used only for equip-able items (not weapons)".
- Weapons (and some other categories) carry their slot info in `type=` instead (e.g. `"One-Handed Axe"`, `"Heavy Armour"`, `"Resource"`).
- The bookmarklet only reads `slot=`, so for weapons it gets an empty string and `mapSlot("")` returns `"Unknown"` via the empty-input early return on line 155.

**Decided fix:** the `resolve-slots` subcommand reads `data/lgo_items.json` and looks up the slot by item name. See `docs/RESOLVER_DESIGN.md` for the full design. Implementation underway:

- ✅ Step 1 — JSON schema verified (matches `RESOLVER_DESIGN.md` §3).
- ✅ Step 2 — `Slot::from_json_variant` added to `src/gear.rs` with full round-trip + rejection tests.
- ✅ Step 3 — `ItemsDb::load_default` + `lookup`.
- ✅ Step 4 — `resolve_stats_file` (uses `toml_edit` to preserve comments; emits slot-grouped output, fixing Bug 5 as a side effect).
- ✅ Step 5 — wired `optimize` / `resolve-slots` subcommands into `main.rs` (bare verbs, case-insensitive, with `--optimize`/`-o` and `--resolve-slots`/`-r` aliases). [PR #18]
- ✅ Step 6 — integration test against the 66-item bookmarklet output (`tests/resolve_slots_integration.rs`, 7 tests). [PR #19]
- ✅ Step 7 — synced `docs/AGENT_CONTEXT.md` and `docs/RESOLVER_DESIGN.md` to the new CLI. (`docs/User Workflow.txt` requires a full revision and is tracked separately.)

### Bug 4 — Many wiki pages fail to resolve ✅ FIXED

The bookmarklet wrote `# WARNING: all stats unknown` for items whose pages *do* exist on the wiki (e.g. `Ornate Ordâkhai Necklace`, `Keen Pristine Madáshi Ring`). Two confounding causes:

1. The page-name builder only encoded spaces and `'`. Non-ASCII characters (`á`, `â`, `û`, `ó`) went into the URL raw.
2. The API call was missing `&redirects=1`. lotro-wiki uses redirects heavily for item-page aliases; without that flag, the API returned the redirect page itself (no `{{Item Tooltip}}`) instead of following to the canonical target.

**Fix (PR #21):** `encodeURIComponent` the title portion (after the `Item:` prefix), and add `&redirects=1` to the API call. Both `bookmarklet/lgo_bookmarklet.html` lines 263 and 269 reflect the fix.

### Bug 5 — TOML output is no longer slot-grouped ✅ FIXED

Fixed as a side effect of the `resolve-slots` subcommand: the resolver re-emits items grouped by canonical slot order with divider comments between groups. The bookmarklet's raw output is no longer expected to be slot-grouped.

The bookmarklet's `buildToml()` emits items in fetch order with blank-line separators. The previously-agreed format (slot groups in canonical order, with divider comments between groups) is now produced downstream.

### Bug 6 — Bookmarklet drops most stats from successfully-fetched pages ✅ FIXED

**Symptom:** for some items whose wiki page was fetched successfully (page exists, no Bug 4 redirect/encoding issue), the bookmarklet emitted the `.toml` with only a subset of stats populated and silently dropped the rest.

**Root cause:** the bookmarklet's `STAT_MAP` had aliases for some "X Rating" suffix forms but not others (`finesse rating` and `critical defence rating` were present, but `tactical mastery rating` was missing, etc.).

**Fix:** `parseAttribs()` now strips a trailing `" rating"` from the lowercased stat name before `STAT_MAP` lookup. `STAT_MAP` simplified to bare forms only; the `Armor`/`Armour` and `Defence`/`Defense` spelling pairs remain listed explicitly because they aren't suffix variants.

**Also added:** an unrecognised-stat-name diagnostic. Any attrib line whose stat name does not resolve through `STAT_MAP` after the rating-strip is recorded in a deduped `Set` and shown in the result panel. Most entries will be base stats (Vitality, Will, Might, Agility, Fate) the bookmarklet intentionally does not track; genuine `STAT_MAP` misses also surface here.

**Probe input** (cross-class items chosen to exercise stats the 66-item Lore-master fixture cannot reach): `docs/probes/lgo_itemnames_StatProbe_20260603_000000.plugindata`. Manual diagnostic — does not have an automated test.

### Bug 7 — `# WARNING: all stats unknown` is misleading ✅ FIXED

Resolved as a side effect of Bug 9. The bookmarklet now tags every item with an `outcome` (`resolved`, `auto-picked`, `needs-pick`, `no-tooltip`, `missing`, `fetch-error`) and emits a distinct TOML comment for each non-`resolved` outcome (see §5). The generic "WARNING: all stats unknown" line is gone; the user can now tell at a glance *why* any given item needs hand-editing.

### Bug 8 — `//` line comments inside `runBookmarklet` break the `javascript:` URL ✅ FIXED

**Symptom:** after PR #24 (Bug 6 fix) merged, clicking the bookmarklet on lotro-wiki.com produced no dialog. The browser console showed `Uncaught SyntaxError: Unexpected end of input`.

**Root cause:** the bookmarklet wiring at the bottom of `bookmarklet/lgo_bookmarklet.html` serialises `runBookmarklet.toString()` and puts the result into the link's `href` as a `javascript:` URL. The browser collapses that string into a single line; any `//` line comment inside swallows everything that follows.

**Fix:** all `//` line comments inside `runBookmarklet` converted to `/* ... */` block comments. Block comments survive newline collapse intact.

**⚠ Lesson for future agents editing `bookmarklet/lgo_bookmarklet.html`:** every comment inside `runBookmarklet` MUST use `/* ... */` form. **Never use `//` line comments inside that function.** A second offence will be much harder to notice in review.

### Bug 9 — Crafted items always emit `# WARNING: all stats unknown` ✅ FIXED

**Symptom:** items like `Keen Pristine Madáshi Earring`, the three `Pristine Mûrai Stickpin of ...` variants, `Elegant Blade of the Adventurer`, `Grove-tender's Robe`, and `Kinta Sword of the Herbalist` consistently failed to resolve through the bookmarklet, even though they exist on lotro-wiki. Roughly one item per crafted recipe was affected.

**Root cause:** lotro-wiki disambiguates near-duplicate item pages by suffixing the URL. `Keen Pristine Madáshi Earring` exists in two forms (Item Level 561 and Item Level 563) and lives at `Item:Keen_Pristine_Madáshi_Earring_(Item_Level_561)` / `_(Item_Level_563)`. The bare `Item:Keen_Pristine_Madáshi_Earring` page is a non-existent stub. Same pattern for nearly all player-crafted items. Weapon variants like `Elegant Blade of the Adventurer` are disambiguated by class/role (`_(DPS)`, `_(Heal)`) rather than item level.

**Investigated and rejected:** distinguishing crafted from non-crafted at plugin-extract time. A temporary `/lgo probe` subcommand was added to `src/lgo.lua` to dump the full Turbine API surface for selected items, and the recorded data (`TestData/lgo_probe_Thalya_20260607_205655.plugindata`) confirmed the API exposes no field that distinguishes the two — `GetCategory`, `GetQuality`, `IsUnique`, `GetMaxStackSize` are byte-identical between a crafted earring and a non-crafted earring; `GetItemClass`, `GetItemLevel`, `GetLevel`, `IsBound`, etc. simply don't exist on this API version; `GetDescription` returns engine error tokens for all gear; `__implementation` is opaque userdata with no enumerable methods. The probe code was removed once the investigation was complete; the data file is kept as historical reference.

**Fix:** the bookmarklet now uses MediaWiki's `prefixsearch` API (in namespace 100, the wiki's `Item` namespace) as a fallback when a direct page lookup yields `missing-page` or `no-tooltip`. The fallback behaviour:

- If prefixsearch returns variants and *all* are tagged with `_(Item_Level_NNN)`, the bookmarklet parses out the integer levels, sorts descending, and auto-picks the highest — the equip-target for an end-game character. The chosen variant is recorded in the item record as `pickedTitle` + `pickedItemLevel` and surfaced in the result panel's "Auto-resolved via disambiguation (informational)" list and as an `# AUTO-PICKED ...` TOML comment so the user can audit the choice.
- If any variant is non-numeric (`_(DPS)`, `_(Heal)`, `_(Burglar)`, etc.) — even if other variants are numeric — the bookmarklet declines to auto-pick. The item is reported as `needs-pick`, included in the "Multiple variants exist; auto-pick declined" sub-list of the result panel, and emitted into the TOML with all-zero stats and a `# UNRESOLVED: ...` comment.
- If prefixsearch returns no variants, the item is reported as `no-tooltip` (bare page existed but had no tooltip — the legendary case) or `missing` (bare page didn't exist either).

The new code lives in `fetchByTitle`, `findDisambigVariants`, and the refactored `fetchItem` in `bookmarklet/lgo_bookmarklet.html`. `buildToml` and `renderResult` switch on the per-item `outcome` field to produce the typed TOML comments and the three-sub-list summary panel.

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
- Test input: `TestData/lgo_itemnames_Thalya_20260521_221120.plugindata` (66 items: first 20 equipped, items 21–66 from the `lgo` chest; several legendary/renamed items expected to be unresolvable).
- Bookmarklet fixture: `TestData/lgo_stats_Thalya_20260525_215012.toml` — the bookmarklet's TOML output for the test input above; used by `tests/resolve_slots_integration.rs` (6 of 7 tests depend on it).
- Probe data (historical reference, see Bug 9): `TestData/lgo_probe_Thalya_20260607_205655.plugindata` — per-item Turbine API dump for a hand-picked set of 7 items (3 paired crafted/non-crafted comparisons plus one ignorable fireworks).

---

## 9. Likely next features (after the bookmarklet bugs)

- Combine the two exported .plugindata files into one
- Optimizer --toml-file flag (specify input .toml)

---

## 10. Deferred work (don't lose track of these)

These are known, decided-but-not-urgent items. Do **not** silently fold them into other PRs; track and address explicitly.

- **Bookmarklet test harness.** The bookmarklet currently has no automated tests. Adding one would mean introducing a JS test runner and mocking `fetch()` of the wiki API. Decision: don't bother unless a regression slips through manual testing badly enough to make it worth the setup cost.
- **Bug 2 (`mapSlot()` fallback).** Latent, no observed symptom. Largely moot once the resolver overrides slot decisions anyway. Leave alone unless it produces a real failure.
- **Hand-edit preservation across `resolve-slots` re-runs.** The pre-pivot `src/merge.rs` implemented a `[__user_edits__]` metadata section that tracked user hand-edits in the `.toml` and prompted on conflicts. The current `resolve-slots` does not preserve hand-edited stats if the user re-runs it after a fresh bookmarklet export. See `docs/User Story & Hand-Edit-Tracking Approach.txt` for the original design. Address if and when a user actually gets bitten by losing edits.
