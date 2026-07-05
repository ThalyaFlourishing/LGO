# Model Guidance — Choosing an AI Model for LGO Edits

> **Purpose.** This note recommends *which class of AI coding model* to use when
> editing each part of LGO, based on a per-file review of algorithmic complexity
> and Rust borrow-checker friction. It is advice, not a rule — use judgement.

## TL;DR

- **Default to a cheap, fast model** (e.g. Claude Haiku 4.5) for most of the
  codebase. The CLI, formatting, data tables, and the Lua plugin are
  well within its reach.
- **Escalate to a frontier model** (e.g. GPT-5.4, GPT-5.5, or a Claude Opus tier)
  for the two genuinely subtle files — `optimizer.rs` and `slot_resolver.rs` —
  and for control-flow / parser edits in `build_db.rs` and `plugindata.rs`.
- The decision is **task-based, not just file-based**: editing a *lookup table*
  in a hard file is easy; editing its *control flow, ownership, or a documented
  invariant* is what demands the stronger model.

## The rule of thumb

Ask: **"Am I touching control flow, ownership/lifetimes, or a documented
invariant?"**

- **Yes** → use a frontier model.
- **No** (it's a lookup table, a string, a match arm, a constant) → a cheap
  model is fine, even in an otherwise hard file.

## Per-file guidance

| File | Friction | Recommended model | Why |
|---|---|---|---|
| `src/optimizer.rs` | 🔴 High | **Frontier** | Exact branch-and-bound search, dominance pruning, paired-slot identity, and a multi-stage comparator all have to agree with the locked semantics in `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md`. A "natural" refactor can silently produce wrong gear. |
| `src/slot_resolver.rs` | 🔴 High | **Frontier** | `toml_edit` mutation via `std::mem::replace`; `&mut force` threaded through a loop that mutates four things at once; `Box<dyn Prompter>` trait objects with a hand-written `Debug`; decor/position bookkeeping that underpins an idempotency guarantee. |
| `src/build_db.rs` | 🟠 Moderate | Frontier *for control-flow edits*; cheap for lookup-table edits | Streaming XML state machine with `&mut` accumulators and `ref mut` bindings into `Option`s that are `.take()`n elsewhere. Adding a stat/slot mapping is trivial; reorganising the state machine is not. |
| `src/plugindata.rs` | 🟠 Moderate | Frontier *for parser edits*; cheap for lookup-table edits | Hand-written recursive-descent parser whose functions return `(_, &str)` slices threaded through the whole recursion — an implicit lifetime contract. Editing the `method_to_name` table is easy; editing parser control flow fights the borrow checker. |
| `src/gearstats.rs` | 🟢 Very low | Cheap | Iterator `.filter().collect()` chains over owned `PathBuf`s; short-lived `.get()` borrows. Clean. |
| `src/main.rs` | 🟢 Low | Cheap | Lots of `&` / `PathBuf` passing, but an index-based arg loop sidesteps iterator-borrow issues. Mostly mechanical. |
| `src/stat.rs` | 🟢 None | Cheap | Enums + `FromStr` / `Display` match arms. |
| `src/gear.rs` | 🟢 None | Cheap | Struct/enum definitions, `Display`, a simple `.values().map().sum()`. |
| `src/report.rs` | 🟢 None | Cheap | Read-only borrows for printing; owned `String` building. |
| `src/lib.rs` | 🟢 None | Cheap | Just `pub mod` declarations. |

**Non-Rust files** (`src/lgo.lua`, `src/lgo.plugin`, `bookmarklet/`, `data/`,
`*.toml`, docs) have no borrow checker and are safe for a cheap model. Reserve
stronger models for genuinely tricky Lua logic if it arises.

## Why two files stand out

Both `optimizer.rs` and `slot_resolver.rs` carry **large, dense test suites**
relative to their size. That ratio is itself a signal: the author has been
bitten by subtle regressions before, and several tests exist specifically to
pin down non-obvious invariants, for example:

- `optimizer.rs` — the branch-and-bound search, comparator worked examples, and
  differential fuzzer/oracle tests must continue to agree on exact results.
- `slot_resolver.rs` — the merge must be **idempotent**: running it three times
  in a row must produce byte-identical output (see
  `merge_idempotent_when_nothing_changes`).

Code that *compiles* can still violate these. That is exactly the territory
where a stronger reasoning model earns its cost.

## A note on cost vs. capability

The cheapest model is *dramatically* cheaper than the top tier, so the savings
on easy edits are real — capture them. But "handles it well" is the binding
constraint on the hard files: a wrong-but-compiling change to the optimizer or
the resolver can cost far more time (and re-prompting) than one clean pass with
a capable model. Spend accordingly.

> Model names and pricing change over time; treat the *tiers* (cheap / frontier)
> as the durable advice and pick whatever current model fits each tier.
