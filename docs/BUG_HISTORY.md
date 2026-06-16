### Bug 1 — `Shoulder` vs `Shoulders` ? FIXED

`SLOT_MAP` had no entry for the wiki's `Shoulder`/`Shoulders` text; `mapSlot()` fell through and emitted the raw string `"Shoulder"`, which `parse_slot_str` rejects. Fixed by adding both `"shoulder"` and `"shoulders"` (lower-cased keys) to `SLOT_MAP`.

### Bug 2 — `mapSlot()` fallback leaks raw wiki text ✅ FIXED

`mapSlot()` line 171 previously returned the raw cleaned string for any slot value not in `SLOT_MAP`, allowing free-text wiki vocabulary like `"tool"`, `"bridle"`, `"Shoulder"`, or `"Gloves"` to reach the TOML output. The resolver's name-based lookup canonicalised these for items it recognised, but for items not in `data/lgo_items.json` (Bridle, Craft Tool, legendaries, recently-added items) the raw wiki string would propagate all the way to the optimizer's TOML reader and crash it with `unrecognised slot 'X'`.

**Fix:** `mapSlot` now returns `"Unknown"` instead of the raw cleaned string. Combined with the `gearstats::read_stats_file` change to skip items with non-canonical slots (silently for `"Unknown"`, with a stderr warning for any other unrecognised string), the optimizer no longer chokes on these items.

### Bug 3 — Items with parsed stats emit `slot = "Unknown"` ? FIXED

Fixed by the `resolve-slots` subcommand (see `docs/RESOLVER_DESIGN.md`). Wired into the CLI in PR #18 and integration-tested against the 66-item bookmarklet fixture in PR #19.

Observed examples:
- `Faded Watcher's Bracers` — Armor 8631, Finesse 6583 — slot Unknown.
- `Resolute Sword of Old Eregion` — Finesse 4210 — slot Unknown.

**Root cause** (confirmed from the Item_Tooltip template documentation):
- `slot=` is documented as "used only for equip-able items (not weapons)".
- Weapons (and some other categories) carry their slot info in `type=` instead (e.g. `"One-Handed Axe"`, `"Heavy Armour"`, `"Resource"`).
- The bookmarklet only reads `slot=`, so for weapons it gets an empty string and `mapSlot("")` returns `"Unknown"` via the empty-input early return on line 155.

**Decided fix:** the `resolve-slots` subcommand reads `data/lgo_items.json` and looks up the slot by item name. See `docs/RESOLVER_DESIGN.md` for the full design. Implementation underway:

- ? Step 1 — JSON schema verified (matches `RESOLVER_DESIGN.md` §3).
- ? Step 2 — `Slot::from_json_variant` added to `src/gear.rs` with full round-trip + rejection tests.
- ? Step 3 — `ItemsDb::load_default` + `lookup`.
- ? Step 4 — `resolve_stats_file` (uses `toml_edit` to preserve comments; emits slot-grouped output, fixing Bug 5 as a side effect).
- ? Step 5 — wired `optimize` / `resolve-slots` subcommands into `main.rs` (bare verbs, case-insensitive, with `--optimize`/`-o` and `--resolve-slots`/`-r` aliases). [PR #18]
- ? Step 6 — integration test against the 66-item bookmarklet output (`tests/resolve_slots_integration.rs`, 7 tests). [PR #19]
- ? Step 7 — synced `docs/AGENT_CONTEXT.md` and `docs/RESOLVER_DESIGN.md` to the new CLI. `docs/User Workflow.txt` was updated in PR #28.

### Bug 4 — Many wiki pages fail to resolve ? FIXED

The bookmarklet wrote `# WARNING: all stats unknown` for items whose pages *do* exist on the wiki (e.g. `Ornate Ordâkhai Necklace`, `Keen Pristine Madáshi Ring`). Two confounding causes:

1. The page-name builder only encoded spaces and `'`. Non-ASCII characters (`á`, `â`, `û`, `ó`) went into the URL raw.
2. The API call was missing `&redirects=1`. lotro-wiki uses redirects heavily for item-page aliases; without that flag, the API returned the redirect page itself (no `{{Item Tooltip}}`) instead of following to the canonical target.

**Fix (PR #21):** `encodeURIComponent` the title portion (after the `Item:` prefix), and add `&redirects=1` to the API call. Both `bookmarklet/lgo_bookmarklet.html` lines 263 and 269 reflect the fix.

### Bug 5 — TOML output is no longer slot-grouped ? FIXED

Fixed as a side effect of the `resolve-slots` subcommand: the resolver re-emits items grouped by canonical slot order with divider comments between groups. The bookmarklet's raw output is no longer expected to be slot-grouped.

The bookmarklet's `buildToml()` emits items in fetch order with blank-line separators. The previously-agreed format (slot groups in canonical order, with divider comments between groups) is now produced downstream.

### Bug 6 — Bookmarklet drops most stats from successfully-fetched pages ? FIXED

**Symptom:** for some items whose wiki page was fetched successfully (page exists, no Bug 4 redirect/encoding issue), the bookmarklet emitted the `.toml` with only a subset of stats populated and silently dropped the rest.

**Root cause:** the bookmarklet's `STAT_MAP` had aliases for some "X Rating" suffix forms but not others (`finesse rating` and `critical defence rating` were present, but `tactical mastery rating` was missing, etc.).

**Fix:** `parseAttribs()` now strips a trailing `" rating"` from the lowercased stat name before `STAT_MAP` lookup. `STAT_MAP` simplified to bare forms only; the `Armor`/`Armour` and `Defence`/`Defense` spelling pairs remain listed explicitly because they aren't suffix variants.

**Also added:** an unrecognised-stat-name diagnostic. Any attrib line whose stat name does not resolve through `STAT_MAP` after the rating-strip is recorded in a deduped `Set` and shown in the result panel. Most entries will be base stats (Vitality, Will, Might, Agility, Fate) the bookmarklet intentionally does not track; genuine `STAT_MAP` misses also surface here.

**Probe input** (cross-class items chosen to exercise stats the 66-item Lore-master fixture cannot reach): `docs/probes/lgo_itemnames_StatProbe_20260603_000000.plugindata`. Manual diagnostic — does not have an automated test.

### Bug 7 — `# WARNING: all stats unknown` is misleading ? FIXED

Resolved as a side effect of Bug 9. The bookmarklet now tags every item with an `outcome` (`resolved`, `auto-picked`, `needs-pick`, `no-tooltip`, `missing`, `fetch-error`) and emits a distinct TOML comment for each non-`resolved` outcome (see §5). The generic "WARNING: all stats unknown" line is gone; the user can now tell at a glance *why* any given item needs hand-editing.

### Bug 8 — `//` line comments inside `runBookmarklet` break the `javascript:` URL ? FIXED

**Symptom:** after PR #24 (Bug 6 fix) merged, clicking the bookmarklet on lotro-wiki.com produced no dialog. The browser console showed `Uncaught SyntaxError: Unexpected end of input`.

**Root cause:** the bookmarklet wiring at the bottom of `bookmarklet/lgo_bookmarklet.html` serialises `runBookmarklet.toString()` and puts the result into the link's `href` as a `javascript:` URL. The browser collapses that string into a single line; any `//` line comment inside swallows everything that follows.

**Fix:** all `//` line comments inside `runBookmarklet` converted to `/* ... */` block comments. Block comments survive newline collapse intact.

**? Lesson for future agents editing `bookmarklet/lgo_bookmarklet.html`:** every comment inside `runBookmarklet` MUST use `/* ... */` form. **Never use `//` line comments inside that function.** A second offence will be much harder to notice in review.

### Bug 9 — Crafted items always emit `# WARNING: all stats unknown` ? FIXED

**Symptom:** items like `Keen Pristine Madáshi Earring`, the three `Pristine Mûrai Stickpin of ...` variants, `Elegant Blade of the Adventurer`, `Grove-tender's Robe`, and `Kinta Sword of the Herbalist` consistently failed to resolve through the bookmarklet, even though they exist on lotro-wiki. Roughly one item per crafted recipe was affected.

**Root cause:** lotro-wiki disambiguates near-duplicate item pages by suffixing the URL. `Keen Pristine Madáshi Earring` exists in two forms (Item Level 561 and Item Level 563) and lives at `Item:Keen_Pristine_Madáshi_Earring_(Item_Level_561)` / `_(Item_Level_563)`. The bare `Item:Keen_Pristine_Madáshi_Earring` page is a non-existent stub. Same pattern for nearly all player-crafted items. Weapon variants like `Elegant Blade of the Adventurer` are disambiguated by class/role (`_(DPS)`, `_(Heal)`) rather than item level.

**Investigated and rejected:** distinguishing crafted from non-crafted at plugin-extract time. A temporary `/lgo probe` subcommand was added to `src/lgo.lua` to dump the full Turbine API surface for selected items, and the recorded data (`TestData/lgo_probe_Thalya_20260607_205655.plugindata`) confirmed the API exposes no field that distinguishes the two — `GetCategory`, `GetQuality`, `IsUnique`, `GetMaxStackSize` are byte-identical between a crafted earring and a non-crafted earring; `GetItemClass`, `GetItemLevel`, `GetLevel`, `IsBound`, etc. simply don't exist on this API version; `GetDescription` returns engine error tokens for all gear; `__implementation` is opaque userdata with no enumerable methods. The probe code was removed once the investigation was complete; the data file is kept as historical reference.

**Fix:** the bookmarklet now uses MediaWiki's `prefixsearch` API (in namespace 100, the wiki's `Item` namespace) as a fallback when a direct page lookup yields `missing-page` or `no-tooltip`. The fallback behaviour:

- If prefixsearch returns variants and *all* are tagged with `_(Item_Level_NNN)`, the bookmarklet parses out the integer levels, sorts descending, and auto-picks the highest — the equip-target for an end-game character. The chosen variant is recorded in the item record as `pickedTitle` + `pickedItemLevel` and surfaced in the result panel's "Auto-resolved via disambiguation (informational)" list and as an `# AUTO-PICKED ...` TOML comment so the user can audit the choice.
- If any variant is non-numeric (`_(DPS)`, `_(Heal)`, `_(Burglar)`, etc.) — even if other variants are numeric — the bookmarklet declines to auto-pick. The item is reported as `needs-pick`, included in the "Multiple variants exist; auto-pick declined" sub-list of the result panel, and emitted into the TOML with all-zero stats and a `# UNRESOLVED: ...` comment.
- If prefixsearch returns no variants, the item is reported as `no-tooltip` (bare page existed but had no tooltip — the legendary case) or `missing` (bare page didn't exist either).

The new code lives in `fetchByTitle`, `findDisambigVariants`, and the refactored `fetchItem` in `bookmarklet/lgo_bookmarklet.html`. `buildToml` and `renderResult` switch on the per-item `outcome` field to produce the typed TOML comments and the three-sub-list summary panel.

---
