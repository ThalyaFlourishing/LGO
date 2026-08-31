# LGO Stat Reference

## Tracked Stats (16 total, in canonical order)

These are the only stats used by the optimizer and stats file.
The JSON key column remains relevant for serialized stat maps,
and the TOML key is what appears in `gearStats.toml` / `gearReady.toml`.
The enum variant and display name are defined in `src/stat.rs`.
The `Stat` enum uses `#[serde(rename_all = "snake_case")]`.

| Abbreviation | Enum variant         | JSON key              | TOML key             | Display name        |
|--------------|----------------------|-----------------------|----------------------|---------------------|
| `ml`         | `Morale`             | `morale`              | `Morale`             | Morale              |
| `pw`         | `Power`              | `power`               | `Power`              | Power               |
| `am`         | `Armor`              | `armor`               | `Armor`              | Armour              |
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

## Base Stats (5 total, derivation inputs only)

The five raw Base stats appear in `gearReady.toml` after the 16 tracked
stats: in top-level `[InnateStats]`, in each `[[item]]` block, and in
`[item.EssenceTotals]`. Top-level `[InnateStats]` is **Base-stats-only**:
`Might`, `Agility`, `Vitality`, `Will`, and `Fate` belong there, while tracked
stats such as `Morale` or `CriticalRating` do not.

These Base stats are **derivation inputs only** — they are never added raw to
any tracked total, are never valid as optimizer goals, and have no CLI
abbreviations. At optimize time, `lgo` converts them into tracked-stat
contributions using the per-class coefficients in
`data/base_stat_derivations.json` (`src/base_stats.rs`).

| Enum variant | TOML key   | Display name |
|--------------|------------|--------------|
| `Might`      | `Might`    | Might        |
| `Agility`    | `Agility`  | Agility      |
| `Vitality`   | `Vitality` | Vitality     |
| `Will`       | `Will`     | Will         |
| `Fate`       | `Fate`     | Fate         |

### Rounding rule

Each product `coefficient × base_stat_value` rounds **up** via
`f64::ceil()`, per item, per stat (empirically confirmed in-game). Example: a
Lore-master +9 Might item contributes ceil(9 × 1.5) = 14 Critical Rating.
Negative values follow plain `ceil()` semantics (round toward zero).

## Virtues

`gearReady.toml` also carries a top-level `[Virtues]` block immediately after
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
- `data/lgo_virtues.json` uses LGO's current tracked stat vocabulary, so
  unsupported Virtue-only stats such as In-Combat Morale Regen are omitted.

Selected Virtues are fixed stat sources. Their tracked stats contribute
directly to the fixed baseline totals, and any raw Base stats they contain are
merged into the same Base-stat pool as `[InnateStats]` before class derivation.
Virtues are not optimizer goals, and the final report format is unchanged.
