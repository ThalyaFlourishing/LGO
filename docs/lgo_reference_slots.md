# LGO Slot Reference

## Internal equipment slots (19 total, in canonical order)

The enum variant is defined in `src/gear.rs`. Wrist, Finger, and Ear keep
numbered internal variants because the optimizer and final gear-set model still
need two positions for each pooled family. External/user-facing formats use the
unnumbered family names.

| Enum variant | Items DB / TOML / report display | Plugin index |
|--------------|-----------------------------------|--------------|
| `Head`       | Head                              | 1            |
| `Chest`      | Chest                             | 2            |
| `Legs`       | Legs                              | 3            |
| `Hands`      | Hands                             | 4            |
| `Feet`       | Feet                              | 5            |
| `Shoulders`  | Shoulders                         | 6            |
| `Back`       | Back                              | 7            |
| `Wrist1`     | Wrist                             | 8            |
| `Wrist2`     | Wrist                             | 9            |
| `Neck`       | Neck                              | 10           |
| `Finger1`    | Finger                            | 11           |
| `Finger2`    | Finger                            | 12           |
| `Ear1`       | Ear                               | 13           |
| `Ear2`       | Ear                               | 14           |
| `Pocket`     | Pocket                            | 15           |
| `MainHand`   | Main-hand                         | 16           |
| `OffHand`    | Off-hand                          | 17           |
| `Ranged`     | Ranged                            | 18           |
| `ClassItem`  | Class Item                        | 20           |

## Excluded slots (never considered by optimizer)

These slots are excluded at the plugin export stage and are never passed to the optimizer.
Players do not need to unequip these items before running `/lgo export`.

| Plugin index | Name       | Reason          |
|--------------|------------|-----------------|
| 19           | CraftItem  | Not gear        |
| 21           | Bridle     | Mount equipment |

Note: Wrist, Finger, and Ear are paired slots handled by optimizer logic.
User-editable TOML should use only `Wrist`, `Finger`, and `Ear`; numbered
forms such as `Wrist (1)` are no longer canonical.

## Main-hand / Off-hand special handling (two-handed and Either-hand items)

`MainHand` and `OffHand` are optimized as one combined hand pool rather than
two independent single slots. A `Main-hand` item flagged `two_handed = true`
in `gearReady.toml` (sourced from `precludedSlots` in `data/items.xml` via
`build-db` and `resolve-slots`) occupies both hand slots: the optimizer never
combines it with a real `Off-hand` candidate, and the report shows the
off-hand line as `(2-handed item)`. One-handed main hands combine with
off-hand candidates as before.

Items usable in either hand carry `either_hand = true` while keeping their
slot as `Off-hand` (sourced from `EITHER_HAND` in `data/items.xml`). This is a
generated-metadata flag, not a new `Slot` variant — there is no "Either-hand
slot". Legal hand configurations are therefore:

- Main-hand position: Main-hand-only items plus Either-hand items.
- Off-hand position: Off-hand-only items plus Either-hand items.
- A single owned item instance can never fill both hands at once; two owned
  copies of the same Either-hand item may dual-wield.
- A two-handed item occupies both positions and pairs with nothing.

Real items are required: the empty-hand placeholder for a hand position is
selectable only when that position has no eligible real item. On an exact
stat tie between a real item and the placeholder, the real item wins.


