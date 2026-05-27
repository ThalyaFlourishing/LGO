# LGO Agent Context

**Purpose:** A durable, in-repo brief so an AI coding agent can resume work on LGO without losing context across sessions. Read this first at the start of every session.

**Working branch:** `The-Browser-Method`

---

## 1. What LGO is

LGO (LOTRO Gear Optimizer) is a two-part personal tool for the MMO *Lord of the Rings Online*:

1. A **Lua in-game plugin** (`src/lgo.lua`) that exports the player's equipped gear plus the contents of a Shared Storage chest named `lgo`, writing two files to `Documents\The Lord of the Rings Online\PluginData\<character>\AllServers\`:
   - `lgo_export_<character>_<timestamp>.plugindata` — equipped + chest items with slot/category data.
   - `lgo_itemnames_<character>_<timestamp>.plugindata` — flat list of all item names (input for the bookmarklet).
2. A **Rust CLI optimizer** (`src/main.rs` etc.) that reads the plugindata + a stats `.toml` file and finds the best gear combination for a set of stat goals (lexicographic priority).

Item stats cannot be fetched programmatically from lotro-wiki.com — Cloudflare blocks the Rust binary. The workaround is a **bookmarklet** (`bookmarklet/lgo_bookmarklet.html`): the user opens lotro-wiki.com in a browser, clicks the bookmarklet, pastes the contents of `lgo_itemnames_*.plugindata`, and the bookmarklet fetches each item via the wiki API, parses `{{Item Tooltip}}`, and produces a `.toml` of stats.

---

## 2. User workflow (current, end-to-end)

1. Put candidate gear in the in-game Shared Storage chest named `lgo`.
2. Run `/lgo export` in-game → two `.plugindata` files are written.
3. Open https://lotro-wiki.com in a browser.
4. Click the **LGO Stats** bookmarklet.
5. Paste the contents of `lgo_itemnames_*.plugindata` when prompted.
6. The bookmarklet builds a `.toml`. Unresolvable items (legendary/renamed) are written with all stats `= 0` for the user to fill in by hand.
7. Save the `.toml` to the `AllServers` directory.
8. Run `lgo tm:450000 cr:350000 fn:0` (etc.) — Rust auto-detects the most recent `.toml` and `.plugindata`, runs the optimizer, prints the result.

---

## 3. Repo layout (relevant)

- `src/main.rs` — CLI entry: find plugindata → find `.toml` → optimize → report.
- `src/plugindata.rs` — hand-written recursive-descent Lua parser; produces `PluginExport { character, class, base_stats, equipped, candidates }`.
- `src/gearstats.rs` — TOML reader (`read_stats_file`) + `find_latest_stats_file`; `parse_slot_str` enforces the canonical 19-string slot allow-list.
- `src/optimizer.rs` — two-phase compatibility-filter + safe lexicographic narrowing; super-candidates for paired Wrist/Finger/Ear slots; `MAX_CANDIDATES_PER_SLOT = 8`; infeasible-greedy fallback.
- `src/stat.rs` — `Stat` enum, `TRACKED_STATS` (14, canonical order), CLI abbrev parsing, `StatGoal`.
- `src/gear.rs` — `Slot` enum (19 variants; `CraftItem`/`Bridle` excluded), `from_plugin_index`, `Display` impl, `GearItem`, `GearSet`.
- `src/report.rs` — terminal report formatter.
- `src/lgo.lua`, `src/Main.lua`, `src/lgo.plugin` — in-game plugin (tested, working).
- `bookmarklet/lgo_bookmarklet.html` — the bookmarklet HTML page (under active debugging).
- `data/items.xml` (~71 MB), `data/lgo_items.json` (~8 MB), `data/progressions.xml` (~3.6 MB) — canonical game data dumps.
- `docs/` — design notes (`Merge Coding Prompt.txt`, `TOML Analysis.txt`, `User Story & Hand-Edit-Tracking Approach.txt`, `User Workflow.txt`, `lgo_reference_slots.md`, `lgo_reference_stats.md`).
- `SSG_U25_LuaDocumentation/` — **DO NOT ingest in chat.** Large UTF-16 HTML dumps that blow up the model's context window and cause mid-session amnesia. If the Turbine Lua API needs investigating, use a CLI script or a separate scraping pass and commit a distilled UTF-8 Markdown summary instead. This is a hard rule, not a guideline.
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

The Rust code is vocabulary #3. The bookmarklet currently translates #2 → #3 via a hand-maintained `SLOT_MAP` in `lgo_bookmarklet.html`. `data/items.xml` (#1) is the game's source of truth for which slot an item belongs to. Names in `items.xml` and `lgo_items.json` match the in-game item names exactly (same source as the plugin's name export).

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

### Formatting decisions the bookmarklet must honour (regressed — see Bug 5)

- Group `[[item]]` entries by slot, in canonical `Slot::ALL` order.
- Insert a visible divider comment between slot groups.

---

## 6. Confirmed bug list (bookmarklet — current focus)

### Bug 1 — `Shoulder` vs `Shoulders` ✅ FIXED

`SLOT_MAP` had no entry for the wiki's `Shoulder`/`Shoulders` text; `mapSlot()` fell through and emitted the raw string `"Shoulder"`, which `parse_slot_str` rejects. Fixed by adding both `"shoulder": "Shoulders"` and `"shoulders": "Shoulders"` to `SLOT_MAP`. Verified: all six shoulder items in the test run now emit `slot = "Shoulders"`.

### Bug 2 — `mapSlot()` fallback leaks raw wiki text ⏸ DEFERRED

`mapSlot()` line 157: `return mapped || cleaned;` — when a slot string isn't in `SLOT_MAP`, returns the raw wiki value, which will fail `parse_slot_str` later. **Currently latent**: no observed failures via this code path in the test TOML. The wiki has no enforced slot allow-list ({{Item Tooltip}} doc treats `slot=` as free-text), so misses *can* happen, but we have no observational evidence yet. Defer until/unless a real symptom appears. If we do fix it, the change is one line: return `"Unknown"` instead of `cleaned`. (Do **not** duplicate the Rust 19-slot allow-list in JS — that creates drift.)

### Bug 3 — Items with parsed stats emit `slot = "Unknown"` 🔴 NEXT

Observed examples:
- `Faded Watcher's Bracers` — Armor 8631, Finesse 6583 — slot Unknown.
- `Resolute Sword of Old Eregion` — Finesse 4210 — slot Unknown.

**Root cause** (confirmed from the Item_Tooltip template documentation):
- `slot=` is documented as "used only for equip-able items (not weapons)".
- Weapons (and some other categories) carry their slot info in `type=` instead (e.g. `"One-Handed Axe"`, `"Heavy Armour"`, `"Resource"`).
- The bookmarklet only reads `slot=`, so for weapons it gets an empty string and `mapSlot("")` returns `"Unknown"` via the empty-input early return on line 155.

**Open question — user's proposal, NOT YET DECIDED:**

> Since the bookmarklet is far better at extracting **names** than **slots**, rely on `data/lgo_items.json` (or `data/items.xml`) for slot resolution. The slot names there are consistent and reliable, and the canonical slot list derives from them. Add a post-processing CLI step (after pasting the bookmarklet's `.toml`) that looks up each item's slot by name from the canonical data file.

The user asked for the agent's opinion on this; it has not been committed to. Things to decide before implementing:

- Use `lgo_items.json` (8 MB, JSON, easy to load in Rust) rather than `items.xml` (71 MB, XML).
- Map items.xml vocabulary (`HEAD`, `SHOULDER`, `HAND`, etc.) → canonical vocabulary (`Head`, `Shoulders`, `Hands`, etc.).
- How to handle items that legitimately fit multiple slots (rings, earrings, bracelets, one-hand weapons).
- How to disambiguate name collisions (same name across tiers/variants — use item id if available, else itemlevel/quality, else accept any match).
- Behaviour for unknown names: error / warn-and-skip / write `Unknown` for human fix.
- Build a one-time `name → slot(s)` index (in-memory or precomputed `data/name_to_slots.json`) so the step is O(1) per item.
- A separate, larger architectural question: the in-game plugin already knows the slot of equipped items (via `Slot::from_plugin_index`); the Turbine Lua API *may* also expose slot-type for chest items. If so, slot detection could move entirely into the plugin and the wiki would only be consulted for stats. **Do not pursue this now** — investigating the Turbine Lua API in chat caused the context-loss event that motivated writing this file. Flag for later, investigate only via out-of-chat scripts.

### Bug 4 — Many wiki pages fail to resolve

The bookmarklet writes `# WARNING: all stats unknown` for items whose pages *do* exist on the wiki (e.g. `Ornate Ordâkhai Necklace`, `Keen Pristine Madáshi Ring`, `Keen Pristine Madáshi Earring`, `Pristine Mûrai Stickpin of …`, likely `Grove-tender's Robe`). Two confirmed code defects in `fetchItem()` (lines 228–258):

1. The page-name builder (line 230) only encodes spaces and `'`. Non-ASCII characters (`á`, `â`, `û`, `ó`) go into the URL raw. They should be `encodeURIComponent`'d after the `Item:` prefix and underscore substitution.
2. The API call (line 231) is missing `&redirects=1`. lotro-wiki uses redirects heavily for item-page aliases; without that flag, a redirect page (no `{{Item Tooltip}}`) is returned and the item is wrongly reported as unresolved.

### Bug 5 — TOML output is no longer slot-grouped

The bookmarklet's `buildToml()` (lines 160–182) emits items in fetch order with blank-line separators. The previously-agreed format (slot groups in canonical order, with divider comments between groups) is regressed. Restore it.

---

## 7. Honest tool / methodology notes for the agent

- The `bing-search` tool returns LLM-summarized results, not raw page content. It is the **wrong instrument** for "what does this wiki template actually say." For wiki source, ask the user to open `https://lotro-wiki.com/index.php?title=Template:Item_Tooltip&action=edit` (or `&action=raw`) and paste the source.
- `data/items.xml` is too large for the code-search index (~384 KB threshold) and too large for `getfile` to be useful. To inspect it, ask the user to paste a representative snippet.
- `SSG_U25_LuaDocumentation/*.html` files are UTF-16 with BOM. Pulling several into chat blows past the model's context window and causes mid-session amnesia. **Do not ingest them in chat — hard rule.** If the Turbine Lua API needs surveying, do it via a CLI script that emits a clean UTF-8 Markdown summary committed to the repo.
- Slot strings, stat names, and TOML field formatting must round-trip exactly through `parse_slot_str` and the canonical 14-stat list. Do not invent or paraphrase.
- The bookmarklet's `SLOT_MAP` is a translation table between two free-text vocabularies and a rigid one. There is **no canonical translation table** between the wiki's free-text `slot=`/`type=` values and the Rust 19-slot enum. Do **not** speculate about what wiki pages contain — either inspect the actual page (via the user) or instrument the bookmarklet for one diagnostic run.
- When the agent finds itself unsure what was previously decided, **ask the user** rather than reconstructing from inference. Reconstruction from inference is what produced the speculative "Cloak"/"Shield" claims that wasted a previous session.

---

## 8. Character context (test data)

- Character: **Thalya**
- Class: **Lore-master**
- Base stats: Might 5300, Agility 2650, Vitality 10200, Will 7950, Fate 4000.
- Test input: `lgo_itemnames_Thalya_20260521_221120.plugindata` (66 items: first 20 equipped, items 21–66 from the `lgo` chest; several legendary/renamed items expected to be unresolvable).

---

## 9. Likely next features (after the bookmarklet bugs)

- HTML report output (currently terminal-only).
- Architectural review of where slot detection should live (plugin vs wiki vs canonical data file — see Bug 3 open question).
- Others as they come up.
