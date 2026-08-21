# LGO Slot Reference

## Equipment Slots (19 total, in canonical order)

The enum variant is defined in `src/gear.rs`.
The `Slot` enum does NOT use `#[serde(rename_all = "snake_case")]`,
so it serializes exactly as the variant name (PascalCase).

| Enum variant | JSON serialization | Display name | Plugin index |
|--------------|-------------------|--------------|--------------|
| `Head`       | `Head`            | Head         | 1            |
| `Chest`      | `Chest`           | Chest        | 2            |
| `Legs`       | `Legs`            | Legs         | 3            |
| `Hands`      | `Hands`           | Hands        | 4            |
| `Feet`       | `Feet`            | Feet         | 5            |
| `Shoulders`  | `Shoulders`       | Shoulders    | 6            |
| `Back`       | `Back`            | Back         | 7            |
| `Wrist1`     | `Wrist1`          | Wrist (1)    | 8            |
| `Wrist2`     | `Wrist2`          | Wrist (2)    | 9            |
| `Neck`       | `Neck`            | Neck         | 10           |
| `Finger1`    | `Finger1`         | Finger (1)   | 11           |
| `Finger2`    | `Finger2`         | Finger (2)   | 12           |
| `Ear1`       | `Ear1`            | Ear (1)      | 13           |
| `Ear2`       | `Ear2`            | Ear (2)      | 14           |
| `Pocket`     | `Pocket`          | Pocket       | 15           |
| `MainHand`   | `MainHand`        | Main-hand    | 16           |
| `OffHand`    | `OffHand`         | Off-hand     | 17           |
| `Ranged`     | `Ranged`          | Ranged       | 18           |
| `ClassItem`  | `ClassItem`       | Class Item   | 20           |

## Excluded slots (never considered by optimizer)

These slots are excluded at the plugin export stage and are never passed to the optimizer.
Players do not need to unequip these items before running `/lgo export`.

| Plugin index | Name       | Reason          |
|--------------|------------|-----------------|
| 19           | CraftItem  | Not gear        |
| 21           | Bridle     | Mount equipment |

Note: Wrist, Finger, and Ear are paired slots handled by optimizer logic.

## Main-hand / Off-hand special handling (two-handed weapons)

`MainHand` and `OffHand` are optimized as one combined hand pool rather than
two independent single slots. A `Main-hand` item flagged `two_handed = true`
in `gearReady.toml` (sourced from `precludedSlots` in `data/items.xml` via
`build-db` and `resolve-slots`) occupies both hand slots: the optimizer never
combines it with a real `Off-hand` candidate and reports the off-hand as
empty. One-handed main hands combine with off-hand candidates as before, and
either hand slot may also be left empty when that is optimal. Note that
`build_db` maps `EITHER_HAND` items to `Slot::OffHand` (a known modeling
simplification).
