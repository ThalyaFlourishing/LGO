# LGO Stat Reference

## Tracked Stats (14 total, in canonical order)

These are the only stats used by the optimizer and stats file.
The JSON key is what appears in `lgo_items.json` and `lgo_cache.json`.
The enum variant is defined in `src/stat.rs`.
The `Stat` enum uses `#[serde(rename_all = "snake_case")]`.

| Abbreviation | Enum variant         | JSON key              | Display name        |
|--------------|----------------------|-----------------------|---------------------|
| `am`         | `Armor`              | `armor`               | Armor               |
| `cr`         | `CriticalRating`     | `critical_rating`     | Critical Rating     |
| `fn`         | `Finesse`            | `finesse`             | Finesse             |
| `pm`         | `PhysicalMastery`    | `physical_mastery`    | Physical Mastery    |
| `tm`         | `TacticalMastery`    | `tactical_mastery`    | Tactical Mastery    |
| `oh`         | `OutgoingHealing`    | `outgoing_healing`    | Outgoing Healing    |
| `rs`         | `Resistance`         | `resistance`          | Resistance          |
| `cd`         | `CriticalDefense`    | `critical_defense`    | Critical Defense    |
| `ih`         | `IncomingHealing`    | `incoming_healing`    | Incoming Healing    |
| `bl`         | `Block`              | `block`               | Block               |
| `pa`         | `Parry`              | `parry`               | Parry               |
| `ev`         | `Evade`              | `evade`               | Evade               |
| `pt`         | `PhysicalMitigation` | `physical_mitigation` | Physical Mitigation |
| `tt`         | `TacticalMitigation` | `tactical_mitigation` | Tactical Mitigation |

## Internal Stats (not tracked by optimizer, used in db_build only)

These appear in `items.xml` and are parsed by `db_build` but are NOT
written to `lgo_items.json` and are NOT used by the optimizer.

| Enum variant         | JSON key              | XML name in items.xml |
|----------------------|-----------------------|-----------------------|
| `Might`              | `might`               | `MIGHT`               |
| `Agility`            | `agility`             | `AGILITY`             |
| `Vitality`           | `vitality`            | `VITALITY`            |
| `Will`               | `will`                | `WILL`                |
| `Fate`               | `fate`                | `FATE`                |
| `Morale`             | `morale`              | `MORALE`              |
| `Power`              | `power`               | `POWER`               |
| `DevRating`          | `dev_rating`          | `DEVASTATE_RATING`    |
| `OffensiveOverpower` | `offensive_overpower` | `OCMR`                |
| `IncMitigations`     | `inc_mitigations`     | *(none known)*        |

## XML stat names (items.xml → Stat mapping, used in db_build)

| XML name in items.xml | Maps to enum variant |
|-----------------------|----------------------|
| `ARMOUR`              | `Armor`              |
| `CRITICAL_RATING`     | `CriticalRating`     |
| `FINESSE`             | `Finesse`            |
| `PHYSICAL_MASTERY`    | `PhysicalMastery`    |
| `TACTICAL_MASTERY`    | `TacticalMastery`    |
| `OUTGOING_HEALING`    | `OutgoingHealing`    |
| `RESISTANCE`          | `Resistance`         |
| `CRITICAL_DEFENCE`    | `CriticalDefense`    |
| `INCOMING_HEALING`    | `IncomingHealing`    |
| `PHYSICAL_MITIGATION` | `PhysicalMitigation` |
| `TACTICAL_MITIGATION` | `TacticalMitigation` |
| `OCPR`                | `Block`              |
| `PARRY`               | `Parry`              |
| `EVADE`               | `Evade`              |
| `MIGHT`               | `Might`              |
| `AGILITY`             | `Agility`            |
| `VITALITY`            | `Vitality`           |
| `WILL`                | `Will`               |
| `FATE`                | `Fate`               |
| `MORALE`              | `Morale`             |
| `POWER`               | `Power`              |
| `DEVASTATE_RATING`    | `DevRating`          |
| `OCMR`                | `OffensiveOverpower` |