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
stats: in `[InnateStats]`, in each `[[item]]` block, and in
`[item.EssenceTotals]`. They are **derivation inputs only** — they are never
added raw to any tracked total, are never valid as optimizer goals, and have
no CLI abbreviations. At optimize time, `lgo` converts them into tracked-stat
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
