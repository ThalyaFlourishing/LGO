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

| Plugin index | Name       | Reason          |
|--------------|------------|-----------------|
| 19           | CraftItem  | Not gear        |
| 21           | Bridle     | Mount equipment |

## XML slot names (items.xml → Slot mapping, used in db_build)

| XML slot value  | Maps to enum variant |
|-----------------|----------------------|
| `HEAD`          | `Head`               |
| `CHEST`         | `Chest`              |
| `LEGS`          | `Legs`               |
| `HAND`          | `Hands`              |
| `FEET`          | `Feet`               |
| `SHOULDER`      | `Shoulders`          |
| `BACK`          | `Back`               |
| `WRIST`         | `Wrist1`             |
| `LEFT_WRIST`    | `Wrist1`             |
| `RIGHT_WRIST`   | `Wrist1`             |
| `NECK`          | `Neck`               |
| `FINGER`        | `Finger1`            |
| `LEFT_FINGER`   | `Finger1`            |
| `RIGHT_FINGER`  | `Finger1`            |
| `EAR`           | `Ear1`               |
| `LEFT_EAR`      | `Ear1`               |
| `RIGHT_EAR`     | `Ear1`               |
| `POCKET`        | `Pocket`             |
| `MAIN_HAND`     | `MainHand`           |
| `EITHER_HAND`   | `OffHand`            |
| `OFF_HAND`      | `OffHand`            |
| `RANGED_ITEM`   | `Ranged`             |
| `CLASS_SLOT`    | `ClassItem`          |

Note: All wrist, finger, and ear XML slots map to the `1` variant
(canonical). The optimizer's paired-slot logic handles assignment
to slot 1 vs slot 2 at runtime.