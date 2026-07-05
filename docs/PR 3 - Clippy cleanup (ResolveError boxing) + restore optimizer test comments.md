> **HISTORICAL — superseded.** This document is a point-in-time optimizer rewrite PR prompt/record. The optimizer rewrite landed in PRs #44/#45 (with PR #46 cleanup); see `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` for the current objective and search design. Retained for historical context only; do not treat its planning text as the live spec.

# PR 3 — Clippy cleanup (ResolveError boxing) + restore optimizer test comments

**Repo:** ThalyaFlourishing/LGO — cut a feature branch from `main`. PRs 1 and 2
are merged. This is a small, mechanical cleanup PR. **No behavior changes.**

**Model guidance:** `src/slot_resolver.rs` is frontier-sensitive
(`docs/MODEL_GUIDANCE.md`) and carries a documented idempotency invariant. All
edits here are mechanical (error boxing + comment restoration); do NOT alter any
resolver logic, control flow, or merge semantics. Use a frontier model given the
file's sensitivity, but keep the diff surgical.

## Context
`cargo clippy --all-targets` currently emits 6 `result_large_err` warnings, all
from `src/slot_resolver.rs`. They are NOT about `OptimizeError` (PR 2 already
boxed that correctly). They are about `ResolveError`, whose `ParseToml` variant
embeds a large `toml_edit::TomlError` (~128 bytes), making every
`Result<_, ResolveError>` return type oversized. Separately, PR 2's reformat
stripped explanatory `//` comments from many optimizer tests; restore them.

## Task A — box the fat field in `ResolveError` (clears all 6 warnings)
In `src/slot_resolver.rs`, the `ResolveError` enum is defined at ~lines 231–263.
Its `ParseToml` variant (~lines 241–244) is:

```rust
ParseToml {
    path: PathBuf,
    source: toml_edit::TomlError,
},
```

**Box the fat field** so the enum shrinks without changing any return-type
signatures:

```rust
ParseToml {
    path: PathBuf,
    source: Box<toml_edit::TomlError>,
},
```

Then fix every construction and destructuring site of `ParseToml`. There are
several `.map_err(|e| ResolveError::ParseToml { path: ..., source: e })` sites
(in `resolve_toml_str` ~line 362, `merge_into_canonical` ~lines 652 and 656–662,
`item_names` ~line 941, `collect_unknown_slot_names` ~line 959, and
`take_item_tables` returns a different variant — check each). At each
construction site, wrap the source: `source: Box::new(e)`.

**Also fix the two remap sites in `resolve_stats_file`** (~lines 1076–1086 and
~1099–1115) that destructure and rebuild `ParseToml { source, .. }`. After
boxing, `source` is already a `Box<TomlError>`; rebuild as
`ParseToml { path: ..., source }` (it is already boxed — do NOT double-box).
Verify the `path == Path::new("<previous>")` / `"<incoming>"` guard arms still
compile and behave identically.

Update `ResolveError`'s `Display` impl (~lines 274–276) if needed — `source` is
now a `Box`, but `write!(f, "... {}", source)` works unchanged via `Deref`, so
likely no change required. Confirm.

**Acceptance for Task A:** `cargo clippy --all-targets` emits **0 warnings**.
Do NOT change any other `ResolveError` variant, any function signature, or any
logic. This is purely moving `TomlError` behind a `Box`.

## Task B — restore explanatory comments on optimizer tests
PR 2's reformat removed many `//` explanatory comments from the tests in
`src/optimizer.rs`. Restore concise explanatory comments to the optimizer test
module, matching the style of the pre-PR-2 tests (a short comment stating what
each test proves and, where relevant, the arithmetic). Priority tests to
re-comment (they previously had explanatory comments that were dropped):
- `test_spec_run1_c2_wins`, `test_spec_run2_c6_wins_infeasible`,
  `test_c5_over_c4_same_slot`
- `test_paired_slots_use_two_distinct_instances_and_sum_once_each`
- `test_single_paired_instance_cannot_fill_both_slots`
- `test_no_self_pair_for_tight_minimum_is_infeasible`
- `test_two_distinct_same_name_instances_can_fill_paired_slots`
- `test_pair_infeasible_when_minimum_exceeds_best_legal_pair`
- `test_single_candidate_meets_minimum_exactly`,
  `test_single_candidate_one_below_minimum`
- the three `test_negative_stat_*` tests
- the new `dominance_safety_*`, `branch_and_bound_exactness_*`, and the
  `comparator_worked_example_*` tests (briefly note which objective stage each
  exercises)
- `differential_fuzzer_matches_oracle_smoke` / `_deep` (note they compare
  production search vs. the brute-force oracle)

Comments only — do NOT change any test logic, inputs, or assertions.

## Out of scope
- Any behavior change, signature change, or logic change anywhere.
- Boxing `OptimizeError` (already done in PR 2).
- Any change to the optimizer search, objective, report, or Lua.
- Touching `ResolveError` variants other than `ParseToml`.

## Acceptance criteria
- `cargo build`, `cargo test`, `cargo test -- --ignored` all pass.
- `cargo clippy --all-targets` emits **0 warnings**.
- `cargo fmt --check` clean.
- `slot_resolver.rs` idempotency tests (`merge_idempotent_*`) still pass
  unchanged — the boxing must not alter merge behavior.
- Diff is limited to `src/slot_resolver.rs` (Task A) and `src/optimizer.rs`
  test comments (Task B).