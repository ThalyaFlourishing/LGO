# LGO Stat Reference

## Tracked Stats (14 total, in canonical order)

These are the only stats used by the optimizer and stats file.
The JSON key column remains relevant for serialized stat maps,
and the TOML key is what appears in `lgo_stats_*.toml`.
The enum variant is defined in `src/stat.rs`.
The `Stat` enum uses `#[serde(rename_all = "snake_case")]`.

| Abbreviation | Enum variant         | JSON key              | TOML key             | Display name        |
|--------------|----------------------|-----------------------|----------------------|---------------------|
| `am`         | `Armor`              | `armor`               | `Armor`              | Armor               |
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
