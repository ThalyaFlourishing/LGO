# LGO — LotRO Gear Optimizer

A command-line gear optimizer for **Lord of the Rings Online** (LOTRO), paired with an in-game Lua plugin that exports your character's equipment and stats.

---

## Features

| Component | Description |
|-----------|-------------|
| **`lgo` CLI** | Rust binary — optimizes a gear set for a chosen role (DPS / Tank / Healer) or custom stat weights |
| **Lua plugin** | In-game LOTRO plugin — exports your equipped items as JSON with `/lgo export` |

---

## Quick Start

### Build the CLI

```bash
cargo build --release
# binary is at target/release/lgo
```

### Find your Best-in-Slot gear (DPS)

```bash
lgo optimize --items data/items.json --preset dps
```

### Score an existing gear set (Tank)

```bash
lgo score --set data/gear_set.json --preset tank
```

### Use custom stat weights

```bash
lgo optimize --items data/items.json --weights data/weights_healer.json
```

---

## CLI Reference

```
USAGE:
    lgo <COMMAND>

COMMANDS:
    optimize    Find the best-in-slot combination from a list of items
    score       Score and summarize a fixed gear set

OPTIONS (optimize / score):
    -i, --items <PATH>     Items JSON file   [default: data/items.json]
    -s, --set   <PATH>     Gear-set JSON file [default: data/gear_set.json]
    -w, --weights <PATH>   Stat-weights JSON file (overrides --preset)
    -p, --preset <NAME>    Built-in role preset: dps | tank | healer  [default: dps]
```

---

## Data Formats

### Items file (`data/items.json`)

A JSON array of gear items:

```json
[
  {
    "name": "Helm of the Wanderer",
    "slot": "head",
    "item_level": 475,
    "stats": {
      "vitality": 1200,
      "might":    950,
      "crit_rating": 800,
      "morale":   3500
    }
  }
]
```

**Valid slot names:** `head`, `chest`, `legs`, `hands`, `feet`, `shoulders`, `back`, `neck`,
`ear1`, `ear2`, `finger1`, `finger2`, `wrist1`, `wrist2`, `main_hand`, `off_hand`, `pocketed`

**Valid stat names:** `might`, `agility`, `vitality`, `will`, `fate`, `armor`, `resistance`,
`crit_defense`, `inc_mitigations`, `phys_mitigation`, `tact_mitigation`, `crit_rating`,
`dev_rating`, `finesse_rating`, `offensive_overpower`, `morale`, `power`,
`incoming_healing`, `outgoing_healing`

### Weights file

A JSON object with a `weights` array of `[stat, value]` pairs (value 0.0 – 1.0):

```json
{
  "weights": [
    ["will",             1.0],
    ["fate",             0.9],
    ["outgoing_healing", 0.85]
  ]
}
```

---

## LOTRO Lua Plugin

### Installation

1. Copy the `plugin/` folder to your LOTRO plugins directory:
   ```
   %LOCALAPPDATA%\Turbine\LotroLauncher\LGO\
   ```
2. In-game, load the plugin via the Plugin Manager or:
   ```
   /plugins load LGO
   ```

### In-game Commands

| Command | Description |
|---------|-------------|
| `/lgo export` | Prints your equipped items as JSON — paste into `data/gear_set.json` |
| `/lgo stats`  | Shows your current primary stats |
| `/lgo help`   | Shows help text |

---

## Built-in Role Presets

| Preset | Top stats |
|--------|-----------|
| `dps`    | Might → Crit Rating → Dev Rating → Finesse → Offensive Overpower |
| `tank`   | Vitality → Armor → Phys. Mitigation → Tact. Mitigation → Crit Defense |
| `healer` | Will → Fate → Outgoing Healing → Vitality → Crit Rating |

---

## Project Layout

```
├── Cargo.toml
├── src/
│   ├── main.rs         # CLI entry point (clap)
│   ├── stat.rs         # Stat enum & StatWeights
│   ├── gear.rs         # GearItem, GearSet, Slot enum
│   └── optimizer.rs    # Optimize & score functions
├── data/
│   ├── items.json      # Sample item pool
│   ├── gear_set.json   # Sample fixed gear set
│   └── weights_healer.json  # Example custom weights
└── plugin/
    ├── LGO.lua         # LOTRO Lua plugin
    └── plugin.xml      # Plugin manifest
```

---

## License

MIT
