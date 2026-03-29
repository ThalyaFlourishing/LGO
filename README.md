# LGO — LOTRO Gear Optimizer

A command-line tool for Lord of the Rings Online players that finds the
optimal combination of gear items for a given set of stat priorities and
minima.

---

## How it works

1. You place candidate gear items into a Shared Storage chest named **`lgo`**
   in-game.
2. You run the `/lgo export` command in-game, which writes a `.plugindata`
   file to your Documents folder.
3. You run this program from the command line with your stat goals.
4. The optimizer fetches item stats from [lotro-wiki.com](https://lotro-wiki.com)
   (cached locally after the first lookup) and finds the best gear set.

---

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- The **LGO** in-game plugin installed and configured

### Build

```
cargo build --release
```

The binary will be at `target\release\lgo.exe`.

Optionally, copy it somewhere on your `PATH`:

```
copy target\release\lgo.exe C:\Users\<you>\bin\lgo.exe
```

---

## Usage

```
lgo [options] <stat:minimum> [<stat:minimum> ...]
```

Stats are listed in **priority order**. The optimizer maximises the first
stat, uses the second as a tiebreaker, and so on. Each stat has an optional
minimum — the gear set is only considered valid if every minimum is met.

### Options

| Option | Description |
|---|---|
| `--character <name>` | Character name (auto-detected if only one exists) |
| `--cache <path>` | Path to the item cache JSON file |
| `--help` | Show usage information |

### Examples

```
lgo CritRating:450 TacticalMastery:450 FinesseRating:300 TacticalMitigation:200
```

```
lgo --character Thalya Vitality:500 Morale:800 CritRating:0
```

A minimum of `0` means "maximise this stat but impose no floor."

---

## Stat names

Stat names are case-insensitive and accept common aliases.

| Canonical name | Aliases |
|---|---|
| `Vitality` | |
| `Morale` | |
| `Power` | |
| `Might` | |
| `Agility` | |
| `Will` | |
| `Fate` | |
| `CritRating` | |
| `DevRating` | |
| `FinesseRating` | `Finesse` |
| `TacticalMastery` | `TactMast`, `TactMastery` |
| `PhysicalMastery` | `PhysMast`, `PhysMastery` |
| `OffensiveOverpower` | `Overpower` |
| `Armour` | `Armor` |
| `Resistance` | |
| `CritDefence` | `CritDefense` |
| `TacticalMitigation` | `TactMit` |
| `PhysicalMitigation` | `PhysMit` |
| `IncMitigations` | `IncMit` |
| `IncomingHealing` | `IncHeal` |
| `OutgoingHealing` | `OutHeal` |

---

## Optimization logic

### Lexicographic priority

The optimizer selects the gear set that maximises stats in strict priority
order:

- Among all valid gear sets, pick the one with the highest total for Stat 1.
- Among those tied on Stat 1, pick the one with the highest total for Stat 2.
- And so on.

A gear set is **valid** if every stat meets its user-supplied minimum.

### Infeasible results

If no combination of available items can meet all minima simultaneously, the
program returns the best available result anyway (using the same priority
order, ignoring minima) and clearly reports which stats fell short and by how
much. The process exits with code `2` in this case.

### Slots considered

| Slot | Notes |
|---|---|
| Head | |
| Chest | |
| Legs | |
| Hands | |
| Feet | |
| Shoulders | |
| Back | |
| Wrist (×2) | Any wrist item is a candidate for either wrist slot |
| Neck | |
| Finger (×2) | Any ring is a candidate for either finger slot |
| Ear (×2) | Any earring is a candidate for either ear slot |
| Pocket | |
| Off-hand | |
| Ranged | |

The following slots are **not** considered: Main-hand, Craft item,
Class item, Bridle.

### Candidate limit

No more than **6 candidates per slot** are considered. If your `lgo` chest
contains more than 6 items for a single slot, only the first 6 will be
used and a warning will be shown.

---

## Item data

Item stats are fetched from [lotro-wiki.com](https://lotro-wiki.com) via
its MediaWiki API. Fetched data is cached locally in `lgo_cache.json`
(in the same directory as your `.plugindata` file) so that subsequent
runs do not need to re-query the wiki for items already seen.

To force a fresh lookup for all items, delete `lgo_cache.json`.

---

## Workflow summary

```
[In-game]
  1. Put candidate items in a Shared Storage chest named 'lgo'
  2. /lgo export

[Command line]
  3. lgo CritRating:450 TactMast:450 Finesse:300 TactMit:200

[Output]
  Slot            Recommended Item
  ????????????????????????????????????????????????????????????????
  Head            Umbari Hat of Beasts
  Chest           Umbari Robe of Beasts
  ...

  Stat                    Total    Minimum  Met?
  ????????????????????????????????????????????????
  CritRating             12,480     10,000   ?
  TacticalMastery        11,200     10,000   ?
  FinesseRating           8,300      8,000   ?
  TacticalMitigation     15,400     14,000   ?
```

---

## License

MIT