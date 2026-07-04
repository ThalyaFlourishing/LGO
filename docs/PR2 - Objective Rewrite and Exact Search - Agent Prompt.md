# PR 2 — Objective Rewrite + Exact Search + Oracle + Fuzzer (coding-agent prompt)

**Repo:** ThalyaFlourishing/LGO — cut a feature branch from `main`. PR 1
(refuse-on-overflow guard) is already merged; this builds on it.

**Model guidance:** This is the hardest work in the codebase. `src/optimizer.rs`
is frontier-model territory (`docs/MODEL_GUIDANCE.md`) — control flow,
ownership, AND documented invariants all change. `src/slot_resolver.rs` (touched
only for a small clippy cleanup, Task E) is also frontier-sensitive; keep those
edits mechanical. Use a frontier model.

**Spec of record:** `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite
Plan.md` §1 is the authoritative objective. If anything here is ambiguous, that
document wins.

## Why this PR exists
The merged optimizer still uses the OLD engine: it reasons from independent
per-slot/per-stat maxima (see its own file-header claim, `src/optimizer.rs`
lines 5–13: "the slots do not interact") and implements the WRONG objective —
raw-lexicographic (maximize stat 1, then stat 2). The audit chain
(`docs/Optimizer_Overhaul/01`–`06`) established the independent-maxima model is
unsound. The correct objective is a **clamped-satisfaction** model (below).
Replace both the search core and the objective. This is a rewrite, not a bug
fix — be liberal deleting the old search code and the tests that encode the old
semantics.

## The objective function (authoritative — implement EXACTLY)
Only **goal stats** participate in the search or objective. Non-goal stats are
summed/stored on chosen items for display but never influence the search.

For each goal with minimum `M` and achieved total `v`, define a **clamped
score**: if `M > 0`, `score = min(v / M, 1.0)`; if `M == 0`, no floor — treat as
always "met" for satisfaction, use raw value only in Stage 3.

Compare two complete builds X and Y by these stages, in order; the first stage
that distinguishes them decides:

1. **Met-vector, lexicographic in priority order.** Each goal contributes `1` if
   its minimum is met (`M == 0` counts as met), else `0`. Compare met-vectors as
   tuples in priority order; greater wins. This yields the "ratchet": crossing a
   higher-priority goal into *met* justifies dropping a lower goal, but merely
   raising an unmet-and-still-unmet higher goal does not.
2. **Clamped-score vector, lexicographic in priority order.** Among builds tied
   on the met-vector, compare clamped scores in priority order; greater wins.
   (Met goals clamp to 1.0, so this only differentiates goals unmet in both,
   getting the higher-priority unmet goal closest to 100%.)
3. **Raw-value min-max polish, lexicographic in priority order.** Among builds
   tied on the entire clamped-score vector, prefer higher RAW goal totals in
   priority order. Never sacrifices satisfaction (Stages 1–2 settled that); only
   picks the biggest headline numbers among equivalent builds.
4. **Deterministic final tiebreak.** Exact ties above are astronomically rare;
   break by a stable arbitrary key (e.g. sorted chosen-candidate instance keys)
   so output is reproducible. No product meaning.

**Feasibility flag:** feasible iff every goal with `M > 0` is met. Reported
separately; the optimizer always returns its best build under the comparator,
feasible or not. There is NO separate "infeasible fallback" path — the
comparator handles feasible and infeasible uniformly.

### Worked examples — add each as a deterministic unit test
| Goals (priority order) | Build X | Build Y | Winner | Reason |
|---|---|---|---|---|
| CR:100k, TM:100k | CR 120k / TM 90k | CR 100k / TM 89,999 | **X** | Both meet CR; Stage 2 on TM: 0.90 > 0.89999 |
| A:100 (p1), B:100 (p2); A reachable to at most 96 | A 94 / B 100 (met-vec 0,1) | A 96 / B 80 (met-vec 0,0) | **X** | Ratchet: Y drops *met* B to inch *unmet, doomed* A → (0,1) > (0,0) |
| A:100 (p1), B:100 (p2) | A 98 / B 100 (met-vec 0,1) | A 100 / B 70 (met-vec 1,0) | **Y** | Crossing into met on p1 justifies dropping p2 |
| A:100, B:100 (both unmet either way) | A 95 / B 96 (0,0) | A 94 / B 97 (0,0) | **X** | Stage 2: goal-1 clamped 0.95 > 0.94 |
| CR:100k, TM:100k (both met) | CR 250k / TM 100k | CR 100k / TM 100k | **X** | Stages 1–2 tie (all clamped 1.0); Stage 3 raw CR: 250k > 100k |

## Task A — `src/optimizer.rs`: delete the old search core
Remove (they encode the false independent-maxima model and/or wrong objective):
- The file-header correctness claim (lines ~5–48, "the slots do not interact",
  the two-phase description). Replace with a short doc-comment describing the new
  clamped-satisfaction objective + exact search, referencing doc 07.
- `compute_global_max`, `compute_single_maxima`, `compute_pair_maxima` **as
  search drivers**. (Per-slot per-stat maxima may be reintroduced ONLY as
  admissible branch-and-bound bounds — Task C — but the old feasibility use is
  gone.)
- `filter_compatible_single`, `filter_compatible_pair`.
- `safe_narrow_single`, `safe_narrow_pair`, `narrow_single`, `narrow_pair`.
- The `phase1_viable` two-path branch (lines ~217–260) and all reverse-priority
  narrowing.

**Keep** (scaffolding is sound; do NOT rewrite): `Candidate`, `PairCandidate`,
`canonical_slot`, `paired_slot2`, `build_pairs`, `candidate_to_gear_item`,
`slot_display`, `validate_candidate_pool_sizes` (the PR-1 guard, lines ~707–730),
`OptimizeError`, per-instance candidate identity.

## Task B — `src/optimizer.rs`: the comparator
Implement the Stage 1–4 comparator as a single total ordering over a build's
goal-stat totals, e.g.
`fn compare_builds(x_totals: &[i64], y_totals: &[i64], goals: &[StatGoal]) -> Ordering`
(operate on goal totals in goal order). **Unit-test it directly with the
worked-examples table BEFORE wiring it into search**, so the objective is
verified in isolation.
- Use **exact integer/rational** comparison for clamped scores — NO floating
  point (avoid `f64` tie/precision bugs). To compare `min(v_x,M)/M` vs
  `min(v_y,M)/M`, since the denominator `M` is common per goal, just compare
  `min(v_x, M)` vs `min(v_y, M)` as integers. `met = v >= M`. Guard `M == 0`
  (no division; treat as met; raw value used only in Stage 3).

## Task C — `src/optimizer.rs`: the production search (exact)
Replace the old search with an **exact** search returning the comparator-maximal
complete build. Two components:

**(C1) Per-slot dominance pre-filter.** Within each slot pool (single slots AND
paired-family pools of pair super-candidates), discard any candidate that is
`<=` another candidate in the SAME pool on EVERY goal stat. Keep exactly one
survivor among exact ties (equal on all goal stats). Rationale: stats are
additive and the objective is monotonic non-decreasing in each goal total up to
its clamp, so a dominated candidate is never uniquely required. ONLY goal stats
matter for dominance. This must be proven safe by the fuzzer (Task D).

**(C2) Branch-and-bound over slots.** DFS one slot at a time over the
dominance-filtered pools, tracking running goal totals; keep the best complete
build so far under the comparator. Prune a partial build when its **optimistic
completion** (running totals + each remaining slot's per-goal maximum) cannot
beat the best-so-far under the comparator. The bound must be **admissible**
(never underestimate achievable goal totals). Iterate slots in `Slot::ALL` order
and candidates in a stable sorted order for reproducibility.

**Empty/placeholder slots and singleton paired families:** preserve current
behavior (zero placeholders for missing slots; a singleton paired item pairs
with an empty placeholder, never itself — `build_pairs` already does this).

**Assembly & result:** assemble the winning build into a `GearSet` exactly as
today (one item per slot; paired families place both constituents into
slot1/slot2). Recompute `feasible` and `failed_minima` from assembled totals
(keep this honest final check). `OptimizeResult` shape unchanged unless a field
is genuinely needed.

## Task D — brute-force oracle (test-only) + differential fuzzer
**Oracle** (`#[cfg(test)]`): reuse the SAME pool construction and `build_pairs`
as production (do NOT reimplement slot/pair semantics); enumerate the full
Cartesian product of complete builds, streaming (keep only best-so-far, never
store all); score each with the SAME `compare_builds`; return the
comparator-maximal build. Never run on large inputs.

**Fuzzer** (deterministic seeded RNG): compare production `optimize` vs. oracle.
- Inputs within contract: 2–4 slot families (sometimes a paired family), 1–4
  candidates per slot/family (≤ cap), 1–3 goals, small stat ranges (e.g.
  `0..=15`, some zeros, occasional negatives), minima small (include some `:0`).
- Compare feasibility flag AND clamped-score vector (i.e. `compare_builds`
  judges them equal). Do NOT require identical item identity (ties may pick
  different-but-equivalent builds).
- On mismatch: print goals, all pools+candidates, both results, and a
  copy-pasteable fixture; fail. Promote any discovered mismatch to a permanent
  deterministic regression test.
- Volume: ~250 cases in a normal `#[test]`; a deeper run (5,000+) behind
  `#[ignore]`. Document the deep run in `docs/Command Line Reference.txt` §4
  (`cargo test -- --ignored`).

## Task E — clippy cleanup (get `cargo clippy` green on the whole crate)
`cargo clippy --all-targets` currently fails on PRE-EXISTING warnings in files
untouched by the optimizer work. Fix them, mechanically and minimally:
- **`result_large_err`** (introduced by PR 1's `Result<_, OptimizeError>`): box
  the error. Change the signature to
  `pub fn optimize(...) -> Result<OptimizeResult, Box<OptimizeError>>`, wrap
  error construction in `Box::new(...)`, and update `src/main.rs`'s `run_optimize`
  match arm and all optimizer tests accordingly.
- **`needless_return`** in `src/slot_resolver.rs` and `src/report.rs`: remove the
  flagged `return` keywords (tail-expression form). Do NOT alter any logic.
- **`type_complexity`** in `src/slot_resolver.rs` test helpers (e.g. the
  `&[(&str, &str, &[(&str, i64)])]` signatures): introduce a `type` alias to
  satisfy the lint. Test-only; no behavior change.
- Do NOT change `slot_resolver.rs` logic beyond these mechanical lint fixes; its
  idempotency invariant (`merge_idempotent_*` tests) must still hold.
- If any *new* clippy warning appears from PR-2 code, fix it too.

## Task F — `src/report.rs`: align wording with the new objective
- Header doc-comment (line ~8) and `print_infeasible_banner` (lines ~185–186)
  currently say the result reflects "the priority order of your stat list" (old
  framing). Reword to reflect clamped-satisfaction, e.g.: "The result shown gets
  your highest-priority goals as close to their targets as possible; once a goal
  is met, extra points in it are not pursued at the expense of lower-priority
  goals still short of target." No structural report changes; no non-goal-stat
  columns.

## Tests to DELETE (encode old semantics)
- The `test_safe_narrowing_*` tests whose EXPECTED VALUES assume raw-lexicographic
  maximization: `test_safe_narrowing_does_not_sacrifice_stat2_for_stat1`,
  `test_safe_narrowing_feasibility_flag_correct`,
  `test_safe_narrowing_preserves_higher_priority_max_when_no_minimum`,
  `test_safe_narrowing_paired_preserves_higher_priority_max_when_no_minimum`.
  Re-express any still-valid INTENT under the new objective as new tests; do not
  keep assertions that contradict the clamped-satisfaction rules.
- `test_spec_run1_c2_wins`, `test_spec_run2_c6_wins_infeasible`, `test_c5_over_c4_same_slot`:
  re-derive the expected winner under the NEW comparator and rewrite, or delete
  if redundant with the worked-examples tests. (Do NOT assume the old expected
  winners still hold.)

## Tests to KEEP and re-verify (rules unchanged by the new objective)
Update all to the new `Result`/`Box` signature; re-confirm each expected value
under the new objective (most are single-goal or feasibility checks, unaffected):
- Paired-slot identity: `test_no_self_pair_for_tight_minimum_is_infeasible`,
  `test_two_distinct_same_name_instances_can_fill_paired_slots`,
  `test_one_same_name_instance_alone_is_infeasible_but_two_are_feasible`,
  `test_pair_family_consistency_for_ear_singleton_and_duplicate_copy`,
  `test_pair_infeasible_when_minimum_exceeds_best_legal_pair`,
  `test_paired_slots_use_two_distinct_instances_and_sum_once_each`,
  `test_single_paired_instance_cannot_fill_both_slots`.
- Feasibility boundary: `test_single_candidate_meets_minimum_exactly`,
  `test_single_candidate_one_below_minimum`.
- Negative-stat: `test_negative_stat_compensated_across_slots`,
  `test_negative_stat_causes_infeasibility`,
  `test_negative_stat_on_non_goal_stat_does_not_crash`.
- Placeholder/missing-slot: `test_missing_slot_emits_placeholder_warning`,
  `test_no_goals_returns_first_candidates`.
- PR-1 overflow guard: `test_too_many_single_slot_candidates_is_refused`,
  `test_too_many_paired_family_candidates_is_refused`,
  `test_exactly_eight_candidates_is_allowed`, `test_eight_per_family_paired_is_allowed`
  (update to `.unwrap_err()` returning `Box<OptimizeError>` — deref as needed).

## Tests to ADD
- Direct comparator tests: the five worked-examples rows, one test each.
- Dominance-safety test: a small case where the naive best uses an item a buggy
  dominance filter might wrongly discard (guards C1).
- Branch-and-bound-exactness test: a small case with a tempting high-priority
  overshoot that must be rejected to meet a lower goal (guards C2).
- The differential fuzzer (normal + `#[ignore]` deep run).

## Acceptance criteria
- `cargo build`, `cargo test`, and **`cargo clippy --all-targets` (0 warnings)**,
  `cargo fmt --check` all clean.
- The comparator matches all five worked examples.
- Production `optimize` matches the oracle on the fuzzer (250-case normal run and
  the deep `#[ignore]` run).
- The five real observed cases from `docs/Optimizer_Overhaul/01` (e.g.
  `cr:100000 tm:70000`) behave per the new objective: a lower-priority goal is
  NOT missed while a higher-priority goal sits far above its met threshold when a
  build meeting more/closer goals exists. **Document the new outputs in the PR
  description** (they are expected to differ from the old ones; exact numbers
  depend on fixture data).
- No candidate is ever silently dropped (overflow refuses via PR-1 guard;
  dominance only drops provably-dominated items).

## Out of scope
- Any change to `resolve-slots` LOGIC, `build-db`, plugindata parsing, the
  bookmarklet, or the Lua export format. (`slot_resolver.rs` clippy fixes in
  Task E are mechanical-only.)
- Lowering the cap (stays 8; speed comes from dominance + B&B).
- New CLI flags or report columns.