# LGO Stat Reference

## Tracked Stats (13 total, in canonical order)

These are the only stats used by the optimizer and stats file.
The JSON key column remains relevant for serialized stat maps,
and the TOML key is what appears in `gearStats.toml` / `gearReady.toml`.
The enum variant and display name are defined in `src/stat.rs`.
The `Stat` enum uses `#[serde(rename_all = "snake_case")]`.

| Abbreviation | Enum variant         | JSON key              | TOML key             | Display name        |
|--------------|----------------------|-----------------------|----------------------|---------------------|
| `cr`         | `CriticalRating`     | `critical_rating`     | `CriticalRating`     | Critical Rating     |
| `fn`         | `Finesse`            | `finesse`             | `Finesse`            | Finesse             |
| `pm`         | `PhysicalMastery`    | `physical_mastery`    | `PhysicalMastery`    | Physical Mastery    |
| `tm`         | `TacticalMastery`    | `tactical_mastery`    | `TacticalMastery`    | Tactical Mastery    |
| `oh`         | `OutgoingHealing`    | `outgoing_healing`    | `OutgoingHealing`    | Outgoing Healing    |
| `rs`         | `Resistance`         | `resistance`          | `Resistance`         | Resistance          |
| `cd`         | `CriticalDefense`    | `critical_defense`    | `CriticalDefense`    | Critical Defense    |
| `ih`         | `IncomingHealing`    | `incoming_healing`    | `IncomingHealing`    | Incoming Healing    |
| `bl`         | `Block`              | `block`               | `Block`              | Block               |
| `pa`         | `Parry`              | `parry`               | `Parry`              | Parry               |
| `ev`         | `Evade`              | `evade`               | `Evade`              | Evade               |
| `pt`         | `PhysicalMitigation` | `physical_mitigation` | `PhysicalMitigation` | Physical Mitigation |
| `tt`         | `TacticalMitigation` | `tactical_mitigation` | `TacticalMitigation` | Tactical Mitigation |

A stat value in `gearReady.toml` may be a single integer or a non-empty TOML
array of integers (e.g. `CriticalRating = [4917, 3222]`, one element per
socket); LGO sums the array on read and `resolve-slots` preserves it verbatim.
Array values are accepted everywhere except `[InnateStats]`, which is
generated from the plugin export and stays integer-only. Any other value type
(float, string, bool, empty/mixed/nested array, ...) is a hard error naming
the item and key.

## Base Stats (6 total, derivation inputs only)

The six raw Base stats appear in `gearReady.toml` after the 13 tracked
stats: in top-level `[InnateStats]`, in each `[[item]]` block, and in
`[item.EssenceTotals]`. Top-level `[InnateStats]` is **Base-stats-only** and
holds exactly the five plugin-exported Base stats: `Might`, `Agility`,
`Vitality`, `Will`, and `Fate` belong there, while tracked stats such as
`CriticalRating` do not. `Armour` is **omitted** from
`[InnateStats]` (innate/character Armour is always zero), but it is still a
Base stat everywhere else — on items, in `[item.EssenceTotals]`, and from the
three Armour Virtues.

These Base stats are **derivation inputs only** — they are never added raw to
any tracked total, are never valid as optimizer goals, and have no CLI
abbreviations. At optimize time, `lgo` converts them into tracked-stat
contributions using the per-class coefficients in
`data/base_stat_derivations.json` (`src/base_stats.rs`).

| Enum variant | TOML key   | Display name | Derives (all classes)                             |
|--------------|------------|--------------|---------------------------------------------------|
| `Might`      | `Might`    | Might        | class-dependent                                   |
| `Agility`    | `Agility`  | Agility      | class-dependent                                   |
| `Vitality`   | `Vitality` | Vitality     | none                                              |
| `Will`       | `Will`     | Will         | class-dependent                                   |
| `Fate`       | `Fate`     | Fate         | none                                              |
| `Armor`      | `Armour`   | Armour       | PhysicalMitigation × 1.0, TacticalMitigation × 0.2 |

`Armour` comes only from gear items and from the three Armour Virtues
(Compassion, Empathy, Loyalty); innate/character Armour is always zero and
Essences never carry it. Its PhysicalMitigation (1.0) and TacticalMitigation
(0.2) coefficients are **class-independent** — every class row is identical.
Each per-item product still rounds up via `f64::ceil()` (see below), so a
±1 drift versus a whole-total computation is acceptable.

**Double-count caveat:** the PhysicalMitigation / TacticalMitigation values the
bookmarklet scrapes from lotro-wiki tooltips are the item's *explicit*
mitigation lines only and do **not** include the Armour-derived portion; LGO
adds that at optimize time. If you hand-enter mitigation, enter only the
explicit tooltip values, never the Armour-derived ones — otherwise Armour's
mitigation is counted twice.

### Rounding rule

Each product `coefficient × base_stat_value` rounds **up** via
`f64::ceil()`, per item, per stat (empirically confirmed in-game). Example: a
Lore-master +9 Might item contributes ceil(9 × 1.5) = 14 Critical Rating.
Negative values follow plain `ceil()` semantics (round toward zero).

## Tracked-stat links

Some tracked stats feed other tracked stats. Currently there is one rule:

| Source | Target | Coefficient |
|---|---|---|
| `TacticalMastery` | `OutgoingHealing` | × 1 |

Every point of Tactical Mastery adds exactly one point of Outgoing Healing.
The rule is **class-independent** and lives in `base_stats::TRACKED_LINKS`
(not in `data/base_stat_derivations.json`). It is applied after Base
derivation, once to the innate tracked-stat map and once to each item's map,
so the Tactical Mastery it reads already includes explicit tooltip values,
essences, Base-derived Tactical Mastery and Virtue contributions. Because the
coefficient is an integer, applying it per map is exactly equal to applying it
to the final total — no rounding applies. The optimizer never sees this as a
rule; it only sees ordinary tracked-stat maps with the Outgoing Healing
already folded in. `lgo base-stats` prints a *Fixed tracked-stat baseline*
section taken from the same pre-optimize map `optimize` consumes, so its
Outgoing Healing row already includes the link; the *Derived tracked-stat
contributions* table above it is Base-stats-only and excludes the link.

**Double-count caveat:** the Outgoing Healing the bookmarklet scrapes from
lotro-wiki tooltips is the item's *explicit* Outgoing Healing line only; LGO
adds the Tactical Mastery-derived portion at optimize time. Never hand-enter
the TM-derived portion. Note that `[VirtueInnateStats].TacticalMastery`
(default 9275) therefore also contributes 9275 Outgoing Healing — this is
intended, not a bug.

## Virtues

`gearReady.toml` also carries a top-level `[Virtues]` block after generated
`[InnateStats]`:

```toml
[Virtues]
Virtue1            = ""
Virtue2            = ""
Virtue3            = ""
Virtue4            = ""
Virtue5            = ""
```

- Users hand-edit these five string values.
- Empty strings, or strings containing only whitespace, mean "no Virtue
  selected" for that slot.
- Non-empty values are matched case-insensitively against the top-level keys in
  `data/lgo_virtues.json`.
- Unknown names and duplicate non-empty names are hard errors at optimize time.
- `data/lgo_virtues.json` uses only LGO's tracked and Base stat vocabulary;
  unsupported stats are omitted. Justice contributes only Tactical Mitigation.

Selected Virtues are fixed stat sources. Their tracked stats contribute
directly to the fixed baseline totals, and any raw Base stats they contain are
merged into the same Base-stat pool as `[InnateStats]` before class derivation.
Virtues are not optimizer goals, and the final report format is unchanged.

## VirtueInnateStats

`gearReady.toml` carries a top-level `[VirtueInnateStats]` block immediately
after `[Virtues]` and before the first `[[item]]`:

```toml
[VirtueInnateStats]
# Virtue-passive masteries. This assumes you have all of them maxed. If you don't, you can go look at what your actual mastery passives are and correct the number here.
PhysicalMastery    = 9275
TacticalMastery    = 9275
```

- Holds exactly two tracked stats, in canonical `TRACKED_STATS` order:
  `PhysicalMastery` then `TacticalMastery`. Any other key is a hard error.
- These represent virtue-passive masteries the Turbine API cannot expose;
  they are **not derived and not calculated**. Both default to `9275`.
- Like `[Virtues]`, the block is **user-maintained / preserved**: once present,
  existing values survive every `resolve-slots` re-run verbatim. Only missing
  fields are added (seeded to `9275`) and alignment is normalized.
- At optimize time the two values fold **directly** into the fixed tracked-stat
  baseline, exactly like a Virtue's tracked stats — added directly, with no
  Base-stat derivation. An absent block contributes nothing.
