# LGO Agent Context

**Purpose:** A durable, in-repo brief so an AI coding agent can resume work on LGO without losing context across sessions. Read this first at the start of every session.

**Default base branch:** `main`.

**Model guidance:** Before editing Rust source, see `docs/MODEL_GUIDANCE.md` for which class of AI model suits each file. For current optimizer semantics and search design, see `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md`. `src/optimizer.rs` and `src/slot_resolver.rs` need a frontier model for non-trivial edits; most other files are fine with a cheap one.

---

## 1. What LGO is

LGO (LOTRO Gear Optimizer) is a two-part personal tool for the MMO *Lord of the Rings Online*. The target user base is a small group of personal acquaintances of the author of this project. There will be no general public release, and there are no commercial needs or long-term engineering best-practices needs. It has not yet been released at all, in fact, so there are currently no concerns about backwards compatibility. Also, it is meant to function exclusively in the Windows OS. No concerns regarding other OS's is warranted.

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
- User hand-edits persistent corrections, including top-level `[Virtues]` and
  per-item essence totals, in `gearReady.toml` only.
- User may optionally save named build-goal profiles in the sibling file
  `lgo_<character>_builds.toml` via `lgo optimize --save-build <name> ...`,
  then re-run them later with `lgo optimize --build <name>`.
- User invokes 'lgo optimize' which reads the canonical file, derives the raw
Base stats (Might/Agility/Vitality/Will/Fate) into tracked-stat contributions
via the per-class coefficients in `data/base_stat_derivations.json`, folds in
the fixed stats from any selected Virtues in `[Virtues]` via
`data/lgo_virtues.json`, and provides a final optimization report according to
user's specified stats of interest. Derivation happens at optimize time, before
candidates enter the optimizer; `optimize` must be run from a directory
containing `data/`.
- User may invoke `lgo scrap-gear`, which re-runs the live optimizer once per
  saved build and lists items not used in any saved build; it never parses old
  report files.

---

## 3. Repo layout (relevant)

- `src/main.rs` — CLI entry: find plugindata → find `.toml` → optimize → report.
- `src/build_profiles.rs` — saved-build profile reader/writer for
  `lgo_<character>_builds.toml`; pure user-data TOML, strict validation on read.
- `src/plugindata.rs` — hand-written recursive-descent Lua parser; produces `PluginExport { character, class, base_stats }`.
- `src/gearstats.rs` — TOML reader (`read_stats_file`) + `find_latest_stats_file`; `gear::parse_slot_display` enforces the canonical 19-string slot allow-list. `read_stats_file` **skips** items with non-canonical slots rather than erroring: silently for `slot = "Unknown"`, with a stderr warning for any other unrecognised value (Bridle, tool, Tool, hand-edited typos, etc.). See Bug 2 / Bug 10 in `BUG_HISTORY.md`.
- `src/virtues.rs` — top-level `[Virtues]` parsing plus `data/lgo_virtues.json`
  loading/validation and fixed-stat application before optimization.
- `src/optimizer.rs` — exact optimizer: clamped-satisfaction comparator, per-pool dominance filtering, branch-and-bound search, and paired-slot super-candidates for Wrist/Finger/Ear. Verified against a brute-force oracle by a differential fuzzer.
- `src/stat.rs` — `Stat` enum, `TRACKED_STATS` (16, canonical order), CLI abbrev parsing, `StatGoal`.
- `src/gear.rs` — `Slot` enum (19 variants; `CraftItem`/`Bridle` excluded), the single canonical slot string table, `parse_slot_display`, `Display` impl, `GearItem`, `GearSet`.
- `src/report.rs` — terminal + HTML optimize report formatter and base-stats formatter.
- `src/report.rs` also formats the terminal-only `scrap-gear` output.
- `src/report_files.rs` — optimize report file naming and writing (`LGO_Reports` beside the canonical gear TOML).
- `src/lgo.lua`, `src/Main.lua`, `src/lgo.plugin` — in-game plugin (tested, working).
- `bookmarklet/lgo_bookmarklet.html` — the bookmarklet HTML page; handles direct lookups, disambiguation auto-pick (via MediaWiki `prefixsearch`), outcome-typed reporting (see Bug 9), a pinned-top-right status panel, a Cloudflare warm-up probe + fetch-error circuit breaker (see Bug 11), and a programmatic Save TOML... button (`showSaveFilePicker` on Chromium, Blob/`<a download>` fallback elsewhere). It always emits `slot = "Unknown"`; `resolve-slots` replaces that placeholder from `data/lgo_items.json`.
- `data/items.xml` (~71 MB), `data/lgo_items.json` (~5 MB), `data/lgo_virtues.json`
  — canonical game data / fixed-stat data files used by the CLI.
- `src/build_db.rs` — offline database builder, exposed as `lgo build-db [options]`. Reads `data/items.xml`, writes `data/lgo_items.json` — a name → slot (+ `two_handed` and `either_hand` flags) index; no stats (those come from the bookmarklet). Handles both paired-tag `<item>...</item>` and self-closing `<item/>` XML forms. Run via `cargo run --release -- build-db` (dev) or `lgo build-db` (user). Always overwrites the output file.
- `TestData/` — committed test fixtures, all for character Thalya:
  - `lgo_<character-name>_gearNames_<timestamp>.plugindata` — fresh in-game plugin export (input for the bookmarklet).
  - `lgo_Thalya_gearStats.toml` — historical bookmarklet-output fixture (input for `resolve-slots`); contains a mix of canonical slots, `slot = "Unknown"` entries, and pre-Bug-2-fix wiki-vocabulary slots (`"Shoulder"`, `"Gloves"`).
  - `lgo_Thalya_gearReady.toml` — already-resolved canonical gear file (input for `optimize`).
- `docs/` — live docs: `User Workflow.txt`, `BUG_HISTORY.md`, `lgo_reference_slots.md`, `lgo_reference_stats.md`, `Command Line Reference.txt`, `MODEL_GUIDANCE.md`, and `Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` (authoritative optimizer spec). The optimizer audit chain and PR prompt docs under `docs/Optimizer_Overhaul/` are historical records retained for traceability; do not treat them as the live optimizer spec.
- Saved build profiles live beside the canonical gear file as
  `lgo_<character>_builds.toml`, discovered with the same case-insensitive
  character-name filename convention as `gearReady.toml`.

**Recent structural changes (2026-08):** `data/progressions.xml` removed from the repo and from `build-db` entirely. `data/lgo_items.json` no longer carries item stats — it is a slot + `two_handed` + `either_hand` index only; item stats come exclusively from the bookmarklet's lotro-wiki lookups. `CachedItem` (DB entry: name/slot/two_handed/either_hand) and `GearItem` (TOML-derived, with stats) are now distinct types. [PRs #53, #55]

---

## 4. Canonical reference data

### 4.1 Canonical external slot strings (from `src/gear.rs` Display + `parse_slot_display`)

```
Head            Wrist          Pocket
Chest           Neck           Main-hand
Legs            Finger         Off-hand
Hands           Ear            Ranged
Feet                           Class Item
Shoulders
Back
```

The TOML loader exact-matches against this list. Items with any other slot string are **skipped** by `read_stats_file` (silently for `"Unknown"`, with a stderr warning otherwise). The optimizer continues with the items that did match. The internal `Slot` enum still has 19 variants, including `Wrist1`/`Wrist2`, `Finger1`/`Finger2`, and `Ear1`/`Ear2`; the external strings for those pooled families are intentionally unnumbered.

### 4.2 The 16 tracked stats (canonical order)

`Morale`, `Power`, `Armor`, `CriticalRating`, `Finesse`, `PhysicalMastery`, `TacticalMastery`, `OutgoingHealing`, `Resistance`, `CriticalDefense`, `IncomingHealing`, `Block`, `Parry`, `Evade`, `PhysicalMitigation`, `TacticalMitigation`.

Two-letter CLI abbreviations: `ml pw am cr fn pm tm oh rs cd ih bl pa ev pt tt`.

### 4.3 Slot vocabularies (important — easy to confuse)

| # | Where it lives | Style | Example |
|---|---|---|---|
| 1 | `data/items.xml` (game data dump) | UPPERCASE enum values | `HEAD`, `SHOULDER`, `HAND`, `FEET`, `CHEST`, `LEGS` |
| 2 | lotro-wiki.com `{{Item Tooltip}}` `slot=` / slot-bearing `type=` fields | Free-text, editor-typed, no enforced allow-list | `Gloves`, `Wrist`, `Ear`, `Back`, `Feet`, `Shoulder`/`Shoulders`, `One-Handed Axe` |
| 3 | `src/gear.rs` slot table, TOML, reports, and `data/lgo_items.json` `slot` values (canonical) | Curated display strings | `Wrist`, `Main-hand`, `Class Item` |

The Rust code is vocabulary #3. `data/items.xml` (#1) is the game's source of truth for which slot an item goes in. The bookmarklet no longer reads wiki slot vocabulary #2 at all; every emitted `[[item]]` block carries the literal placeholder `slot = "Unknown"`. The `resolve-slots` subcommand then uses `data/lgo_items.json` entries whose `slot` values already use vocabulary #3 (`Head`, `Wrist`, `Main-hand`, `Class Item`, etc.), mapping pooled family strings such as `Wrist` to the first internal variant for optimizer input.

---

## 5. `.toml` format expected by `gearstats::read_stats_file`

The file begins with a top header containing `character`, `class`,
`[InnateStats]`, and `[Virtues]` as the last pre-items blocks. `[InnateStats]`
holds only the five raw Base stats (`Might`, `Agility`, `Vitality`, `Will`,
`Fate`), passed through verbatim from the plugindata by `resolve-slots`.
`[Virtues]` holds five user-maintained string slots (`Virtue1` ... `Virtue5`)
whose non-empty values are matched case-insensitively against the top-level keys
in `data/lgo_virtues.json`. After that, the format per item is:

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
# ...then the 5 raw Base stats...
Might              = 0
Agility            = 0
Vitality           = 0
Will               = 0
Fate               = 0
[item.EssenceTotals]
Morale             = 0
# ...same 21 keys: 16 tracked stats then 5 Base stats...
Fate               = 0
```

`gearReady.toml` is the canonical hand-edited file. Each item has an attached
`[item.EssenceTotals]` child table for user-maintained per-item essence overlays.
The loader immediately adds those values to the base item stats and discards the
base-vs-essence separation in the runtime model. Unknown stat keys in either the
base item block or `EssenceTotals` are hard errors; omitted stats are treated as
zero. Raw Base stats are carried separately from tracked stats and are never
added raw to any tracked total; at optimize time they are derived into
tracked-stat contributions per class (per-product `f64::ceil()` rounding — see
`docs/lgo_reference_stats.md`). Selected Virtues behave like additional fixed
stat sources before that derivation step: tracked Virtue stats add directly to
the fixed tracked baseline, while Virtue Base stats join the `[InnateStats]`
Base-stat pool first and are then derived normally.

### Stat-line alignment (canonical, enforced)

Every stat assignment line in `gearReady.toml` — the 16 tracked stats and the
5 Base stats, in `[[item]]` blocks, `[item.EssenceTotals]`, and
`[InnateStats]` — plus the five `Virtue1` ... `Virtue5` lines in `[Virtues]`,
is written with its `=` at column 20 (key padded to width 19). This is
normalized, not preserved: `resolve-slots` re-aligns spacing on every
resolve/merge regardless of incoming decor, so hand-edited spacing does not
survive, while values and comments do. Any change that lets non-conforming
spacing survive a merge will break the merge-idempotency invariant (output
decor feeds the next run's input decor).

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
  When the chosen main hand is two-handed, the report renders the off-hand
  line as `(2-handed item)` instead of an empty-slot placeholder.

### `either_hand` generated metadata

Items usable in either hand carry `either_hand = true` between `name` and the
stat block, while keeping their slot as `Off-hand`:

```toml
[[item]]
slot        = "Off-hand"
name        = "Example Versatile Mace"
either_hand = true
# ...stats and [item.EssenceTotals] as usual...
```

- **Modeled as a flag, not a slot:** an Either-hand item still occupies the
  Main-hand or Off-hand slot in a gear set; there is no "Either-hand slot" and
  the 19-slot `Slot` enum is unchanged. `two_handed` and `either_hand` are
  mutually exclusive on real items.
- **Source of truth:** the `EITHER_HAND` slot value in `data/items.xml`.
  `build-db` maps such items to `Slot::OffHand` and emits `either_hand: true`
  into `data/lgo_items.json`; `resolve-slots` carries it into `gearReady.toml`.
  It follows the same generated/refresh/preserve rules as `two_handed`: it is
  refreshed from the DB for DB-known items and preserved verbatim for unknown
  (legendary/renamed) items, omitted unless true, and rejected under
  `[item.EssenceTotals]`.
- **Optimizer effect:** Either-hand items are eligible in both hand positions
  (main and off). A single owned instance fills only one hand; two owned
  copies may dual-wield. Real items are required for a hand position: the
  empty placeholder is offered only when that position has no eligible real
  item, and a real item wins any exact tie against the placeholder.

If any slot's candidate pool is empty, the optimizer does not halt; the report
shows a `NO ITEMS` placeholder for that slot (a safety net for bad input).

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

- Group `[[item]]` entries by slot, in canonical `Slot::all()` order.
- Insert a visible divider comment between slot groups.

The bookmarklet emits items in fetch order; `resolve-slots` re-groups them.

---

## 6. There exists no need for any kind of backward compatibility. Ever. Just don't even bring it up. No.

---

## 7. Honest tool / methodology notes for the agent

- The `bing-search` tool returns LLM-summarized results, not raw page content. It is the **wrong instrument** for "what does this wiki template actually say." For wiki source, ask the user to open the page in a browser and view source.
- `data/items.xml` is too large for the code-search index (~384 KB threshold) and too large for `getfile` to be useful. To inspect it, ask the user to paste a representative snippet.
- `data/lgo_items.json` (~5 MB) is at the edge of `getfile`'s comfort zone. The first ~125 entries are reliably retrievable via `getfile`, which is plenty for schema verification. For deeper questions (collisions, counts, name lookups), ask the user to run a `grep` / `Select-String` command locally and paste the output.
- `SSG_U25_LuaDocumentation/*.html` files are UTF-16 with BOM. Pulling several into chat blows past the model's context window and causes mid-session amnesia. **Do not ingest them in chat — hard rule.**
- Slot strings, stat names, and TOML field formatting must round-trip exactly through `parse_slot_display` and the canonical 16-stat list. Do not invent or paraphrase.
- Filename discovery for `lgo_<character>_gearStats.toml` and `lgo_<character>_gearReady.toml` is case-insensitive on the character segment. On Windows, names differing only by case are the same file, so case-only "collisions" are not a real runtime condition in LGO's target environment.
- The bookmarklet does **not** trust or parse the wiki's free-text `slot=` / slot-bearing `type=` vocabulary. It always writes `slot = "Unknown"` and leaves slot resolution to `resolve-slots` and `data/lgo_items.json`.
- **The in-game Turbine plugin API cannot distinguish player-crafted items from non-crafted items.** Verified empirically via a temporary `/lgo probe` subcommand (since removed) that dumped every callable on `Item` and `ItemInfo`. Don't waste a session re-investigating this — crafted-item handling lives in the bookmarklet (see Bug 9).
- **`GetDescription()` on `ItemInfo` returns `<string table error; tableDID [...] token [...]>` for *all* gear items** on the current client. This is a long-standing wiki-side or engine-side string-table failure, not something the plugin can fix. The probe confirmed it succeeds for non-gear items (e.g. fireworks) but fails uniformly across gear.
- **`info.__implementation` is engine-private userdata:** no enumerable metatable methods, no addressable fields. Don't try to use it.
- **`showSaveFilePicker()` requires a *fresh* user activation.** The bookmarklet's original click is consumed by the multi-second wiki fetch loop, so the Save TOML... button is necessary — calling the picker after `await` boundaries throws `SecurityError`. The Blob/`<a download>` fallback path has the same activation requirement. Verified empirically; don't try to "auto-save" without a button.
- When the agent finds itself unsure what was previously decided, **ask the user** rather than reconstructing from inference. Reconstruction from inference is what produced the speculative "Cloakroom of Dol Amroth" episode in earlier sessions; the user's tolerance for it is low and rightly so.
- **`toml_edit` serializes top-level tables by internal `position` index, not map insertion order.** `doc.remove(key)` + `doc.insert(key, ...)` does NOT move a table in the rendered output. To relocate a table you must call `set_position()` on it *and* renumber its sibling tables consistently (see `reorder_resolved_header_before_items` in `src/slot_resolver.rs`, and Bug 10 in `docs/BUG_HISTORY.md`). Note `push_group` renumbers every `[[item]]` table to `0..n` — any header table must be positioned relative to that. Root-level *values* (`character`, `class`) are unaffected; they always render before all tables.
- **`gearReady.toml` is both output and next-run input.** Any layout/decor bug in the merge path compounds across runs (each run's output seeds the next run's parse positions), producing drift that convincingly masquerades as a race condition or nondeterminism. It never is — re-running from an identical file snapshot reproduces byte-identically. Diagnose by diffing consecutive outputs, and guard with bit-identical idempotency tests (modulo the `# gearReady.toml updated:` timestamp line).
- **The Copilot coding agent cannot push to an existing PR's branch from a new task.** A new task always branches from `main` — a prompt instruction to "work on branch X" will be garbled into a fresh branch off `main`, silently producing code against the wrong base (this burned a full agent run as PR #54). To amend an existing agent PR, comment on that PR mentioning `@copilot` instead of starting a new task.
- **CI now runs these gates automatically. A green check is expected and a red X on your own PR means to fix it.
- **"ItemInfo:GetCategory() maps 1:1 to slot for armour, but returns a single Jewelry (49) category for all Ear/Finger/Wrist/Neck/Pocket items, and weapon categories encode type not hand. Per-family overflow bucketing in the Lua plugin is therefore impossible for the slots that matter. Verified empirically 2026-08 via a temporary slot probe (since removed)."
- **The bookmarklet must never contain `//` line comments — block comments (`/* */`) only.** The harness serializes `runBookmarklet.toString()` into a `javascript:` URL; browsers collapse newlines in bookmark URLs, so a single `//` comments out the entire remainder of the script and the bookmarklet silently fails to parse. Every existing comment in the file is a block comment for this reason.
- **After any edit to `bookmarklet/lgo_bookmarklet.html`, the user must reload the harness page and re-drag the link to the bookmarks bar.** The bookmarks-bar copy is a snapshot of the generated URL; editing the file does nothing to already-installed bookmarks. When debugging "my edit had no effect," check this first.
- **Cloudflare challenges on lotro-wiki.com can return HTTP 200 with an HTML interstitial body, not a 403.** `resp.ok` checks do NOT detect them; only attempting `resp.json()` does. This produced the "first run speeds through and everything is a fetch-error" symptom (Bug 10). The bookmarklet's warm-up probe therefore verifies the body parses as JSON (`j && j.query`), not just the status code.

---

## 8. Character context (test data)

- Character: **Thalya**
- Class: **Lore-master**
- Base stats: Might 5300, Agility 2650, Vitality 10200, Will 7950, Fate 4000.
- Plugindata fixture: `TestData/lgo_Thalya_gearNames_<time stamp>.plugindata` — fresh in-game plugin export (input for the bookmarklet).
- Bookmarklet-output fixture: `TestData/lgo_Thalya_gearStats.toml` — historical 66-item TOML fixture used by `tests/resolve_slots_integration.rs`. It intentionally contains a mix of canonical slots, `slot = "Unknown"` entries, and pre-Bug-2-fix wiki-vocabulary slots like `"Shoulder"` and `"Gloves"` so the resolver still exercises name-based slot canonicalisation against legacy input.
- Canonical-gear fixture: `TestData/lgo_Thalya_gearReady.toml` — already-resolved gear file (input for `optimize`).

---

## 9. Likely next features

- Identify items which can be removed from pool
- Construct HTML reports

---

## 10. Deferred work (don't lose track of these)

These are known, decided-but-not-urgent items. Do **not** silently fold them into other PRs; track and address explicitly.

- **Bookmarklet test harness.** The bookmarklet currently has no automated tests. Adding one would mean introducing a JS test runner and mocking `fetch()` of the wiki API. Decision: don't bother unless a regression slips through manual testing badly enough to make it worth the setup cost.
- **Hand-edit preservation across re-runs:** implemented via preserve-by-default merge in `resolve-slots`. The `[__user_edits__]` design from `docs/Merge Coding Prompt.txt` and `docs/User Story & Hand-Edit-Tracking Approach.txt` was rejected in favour of the simpler preserve-by-default model. Those two design docs are now historical.
- **Rename detection.** The merge step matches items by NFC-normalized name. If the wiki renames an item between exports, the merge will treat the renamed item as a removal-and-add pair rather than the same item, silently dropping the user's hand-edits. Accepted risk; revisit if it becomes a real problem.

---

### Bug 11 — Bookmarklet first run in a fresh session speeds through, marking every item `fetch-error` ✅ FIXED

**Symptom:** on the first run in a fresh browser session (typically noticed right
after updating the bookmarklet), the fetch loop completed near-instantly and every
item was emitted as `slot = "Unknown"` with the `fetch-error` outcome comment. The
second and all subsequent runs worked normally. Looked like a server-side glitch;
it was Cloudflare.

**Root cause:** Cloudflare bot mitigation on lotro-wiki.com answers `api.php`
fetches from a not-yet-cleared session with an **HTTP 200 HTML challenge
interstitial**, not a 403. `fetchByTitle` passed its `resp.ok` check, then threw
inside `resp.json()` — a plain `SyntaxError` with no `.code` — which `fetchItem`
mapped to `fetch-error`. Because the failure is instant (no wikitext download, no
prefixsearch fallback), the whole list burned in milliseconds. The first run's
requests let the challenge resolve and set the `cf_clearance` cookie, so run two
succeeded. The correlation with "just updated the bookmarklet" was incidental —
updating is simply when a fresh session starts.

**Fix:** two additions to `run()` in `bookmarklet/lgo_bookmarklet.html`:

1. **Warm-up probe** before the fetch loop: fetch
   `action=query&meta=siteinfo` and require the body to *parse as JSON with a
   `query` key* — `resp.ok` alone cannot detect the 200-HTML challenge. On
   failure, wait 3 s and retry once (the challenge usually auto-clears after the
   first request); if still failing, abort with an instructive status message
   before burning the item list.
2. **Circuit breaker** in the loop: if the first 3 items all come back
   `fetch-error`, abort with a "reload the wiki page and re-run" message instead
   of emitting a garbage TOML.

During the fix, Bug 8 struck again: the first draft of the warm-up used a `//`
line comment, which broke the entire serialized `javascript:` URL. Block comments
only — see Bug 8's lesson.

**⚠ Lesson for future agents:** a Cloudflare-fronted API can fail with
**HTTP 200 + HTML body**; status-code checks are not sufficient liveness checks —
verify the payload parses. And an instant, uniform failure across all items in a
fetch loop is the signature of a transport/session problem, not a data problem:
fail fast and loud rather than emitting plausible-looking all-zero output.