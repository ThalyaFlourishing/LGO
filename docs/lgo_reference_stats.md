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

## XML stat names (items.xml → Stat mapping, used in db_build)

Only the stats we care about are listed. All other XML stat names
(e.g. `OCPR`, `ICMR`, `FIRE_MITIGATION`, etc.) are silently ignored
by `db_build`.

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
| `BLOCK`               | `Block`              |
| `PARRY`               | `Parry`              |
| `EVADE`               | `Evade`              |