# LGO — LOTRO Gear Optimizer

LGO is a command-line optimizer for **Lord of the Rings Online** gear. It picks the best item combination by lexicographic stat priority, using a user-edited `lgo_stats_*.toml` file as input.

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- The **LGO** in-game plugin (`src/lgo.lua` + `src/lgo.plugin`)

## Build

```bash
cargo build --release
```

Binary output:
- Windows: `target\release\lgo.exe`
- Linux/macOS: `target/release/lgo`

## Workflow

See [`docs/User Workflow.txt`](docs/User%20Workflow.txt) for the full step-by-step. In brief:

1. Put candidate gear in an in-game shared-storage chest named `lgo` and run `/lgo export`. Equipped items are exported automatically; equipped craft tools and bridles are excluded (no need to unequip them).
2. On lotro-wiki.com, click the **LGO Stats** bookmarklet, paste the `lgo_gearNames_*.plugindata`, and save the result as `lgo_<character>_gearStats.toml`.
3. Run `lgo resolve-slots` to merge into `lgo_<character>_gearReady.toml` (hand-edit legendaries / `Unknown` slots as needed).
4. Run `lgo optimize <stat:minimum> …` and read the report.

## Stat goal syntax

Each goal is `stat:minimum`.

- Goals are ordered by priority (left to right)
- Minimum `0` means “maximise this stat but require no floor”

Examples:

```bash
lgo TacticalMastery:450000 CriticalRating:350000 Finesse:0
lgo tm:450000 cr:350000 fn:0
lgo --character Thalya tm:450000 oh:100000
```

## Stat abbreviations (tracked stats)

| Abbrev | Stat key | Abbrev | Stat key |
|---|---|---|---|
| `am` | `Armor` | `cd` | `CriticalDefense` |
| `cr` | `CriticalRating` | `ih` | `IncomingHealing` |
| `fn` | `Finesse` | `bl` | `Block` |
| `pm` | `PhysicalMastery` | `pa` | `Parry` |
| `tm` | `TacticalMastery` | `ev` | `Evade` |
| `oh` | `OutgoingHealing` | `pt` | `PhysicalMitigation` |
| `rs` | `Resistance` | `tt` | `TacticalMitigation` |

## Optimization logic

LGO optimizes by strict lexicographic priority:

1. Maximise the first stat.
2. Break ties using the second stat.
3. Continue through the goal list.

A solution is **feasible** only if all minima are met.

If no feasible set exists, LGO still returns the best available lexicographic result and reports which minima were missed.

## Contributing / AI model guidance

If you edit LGO with the help of an AI coding model, see
[`docs/MODEL_GUIDANCE.md`](docs/MODEL_GUIDANCE.md) for a per-file
recommendation on which class of model to use. Short version: most of the
codebase is safe for a cheap, fast model, but `src/optimizer.rs` and
`src/slot_resolver.rs` carry enough algorithmic and borrow-checker
subtlety to warrant a frontier model for non-trivial edits.