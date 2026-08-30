# LGO — LOTRO Gear Optimizer

LGO is a command-line optimizer for **Lord of the Rings Online** gear. It picks the best item combination for priority-ordered stat goals, using the canonical `lgo_<character>_gearReady.toml` file produced by `lgo resolve-slots`.

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
2. On lotro-wiki.com, click the **LGO Stats** bookmarklet, paste the `lgo_*_gearNames_<timestamp>.plugindata` content you copied, and save the result as `lgo_<character>_gearStats.toml`.
3. Run `lgo resolve-slots` to merge into `lgo_<character>_gearReady.toml` (hand-edit legendaries / `Unknown` slots as needed).
4. Run `lgo optimize <stat:minimum> …` and read the report.

## Stat goal syntax

Each goal is `stat:minimum`.

- Goals are ordered by priority (left to right)
- Minimum `0` means “no floor, but maximise this stat only as a later polish/tiebreak”

Examples:

```bash
lgo optimize TacticalMastery:450000 CriticalRating:350000 Finesse:0
lgo optimize tm:450000 cr:350000 fn:0
lgo optimize --character Thalya tm:450000 oh:100000
```

## Stat abbreviations (tracked stats)

| Abbrev | Stat key | Abbrev | Stat key |
|---|---|---|---|
| `am` | `Armour` | `cd` | `CriticalDefense` |
| `ml` | `Morale` | `pw` | `Power` |
| `cr` | `CriticalRating` | `ih` | `IncomingHealing` |
| `fn` | `Finesse` | `bl` | `Block` |
| `pm` | `PhysicalMastery` | `pa` | `Parry` |
| `tm` | `TacticalMastery` | `ev` | `Evade` |
| `oh` | `OutgoingHealing` | `pt` | `PhysicalMitigation` |
| `rs` | `Resistance` | `tt` | `TacticalMitigation` |

## Optimization logic

LGO compares complete builds using the clamped-satisfaction objective from
[`docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md`](docs/Optimizer_Overhaul/07%20-%20Locked%20Semantics%20and%20Rewrite%20Plan.md):

1. Prefer builds that meet higher-priority goals.
2. Among ties, get still-unmet goals as close to their minima as possible in priority order.
3. Only after that, use extra raw totals as a polish/tiebreak.

The search is exact (dominance pre-filter + branch-and-bound). A solution is
**feasible** only if all positive minima are met, but feasible and infeasible
results are compared with the same objective.

## Contributing / AI model guidance

If you edit LGO with the help of an AI coding model, see
[`docs/MODEL_GUIDANCE.md`](docs/MODEL_GUIDANCE.md) for a per-file
recommendation on which class of model to use. Short version: most of the
codebase is safe for a cheap, fast model, but `src/optimizer.rs` and
`src/slot_resolver.rs` carry enough algorithmic and borrow-checker
subtlety to warrant a frontier model for non-trivial edits.