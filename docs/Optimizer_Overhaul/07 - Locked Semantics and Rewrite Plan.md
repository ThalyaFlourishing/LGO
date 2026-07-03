# Optimizer Overhaul — Locked Semantics & Rewrite Plan

**Status:** Awaiting sign-off. No code changes yet (method A).
**Supersedes:** the "guard + oracle + fuzz, defer redesign" staging in docs 05/06,
which was written before the objective function was redefined. With the new
objective (below), the search core is not salvageable and this is now a
**rewrite of the objective + search core**, keeping the surrounding scaffolding.
**Working branch:** `main`.

---

## 1. The objective function (LOCKED)

The optimizer chooses the single best complete gear set (one item per slot;
paired families use two distinct owned instances) under the following total
order. Only **goal stats** participate; non-goal stats never enter the search
or the objective.

### 1.1 Per-goal clamped score
For each goal stat with minimum `M` and achieved total `v`:

- If `M > 0`: `score = min(v / M, 1.0)`  ? capped at 100%; overshoot is worthless.
- If `M == 0`: the goal is "maximize, no floor." It is **always considered met**
  for satisfaction purposes (score treated as 1.0), and its raw value is used
  only in the min-max polish stage (§1.4). (See open question Q-A.)

### 1.2 The comparator (how to compare two complete builds X and Y)
Apply these stages in order; the first stage that distinguishes X and Y decides.

**Stage 1 — Priority-ordered satisfaction with a "met" ratchet.**
Walk goals in priority order. For each goal `g`:
- Let `met_X` = (X meets g's minimum), `met_Y` = (Y meets g's minimum).
- **Ratchet rule:** a build may not win a higher-priority goal by dropping an
  **already-met** lower-priority goal to unmet *unless* doing so **newly meets**
  a higher-priority goal. Concretely, the comparator is defined so that:
  - Meeting a higher-priority goal is always worth sacrificing a lower one
    (**crossing into "met" justifies dropping a lower goal**).
  - Merely raising an **unmet** higher-priority goal that **still won't be met**
    does **not** justify dropping an already-met lower goal.

  The clean formalization that produces exactly this behavior:

  > **Compare the "met-vector" lexicographically in priority order**, where each
  > goal contributes `1` if met, `0` if not. The build whose met-vector is
  > lexicographically greater wins Stage 1.

  This single rule yields all the required cases:
  - `(met, unmet)` beats `(unmet, met)` — priority-1 met wins. ?
  - A build that newly meets goal 1 by dropping goal 2 ? met-vector `(1,0)`
    beats `(0,1)`. ? (crossing into met justifies the drop)
  - A build that keeps goal 2 met while goal 1 stays unmet ? `(0,1)` beats a
    rival `(0,0)` that sacrificed goal 2 to inch unmet goal 1 upward. ?
    (no benefit ? ratchet protects the met goal)

**Stage 2 — Closeness of the unmet goals (priority-ordered clamped scores).**
Among builds tied on the met-vector, compare the **clamped score vector**
lexicographically in priority order. Because met goals are clamped at `1.0`,
this stage only differentiates on goals that are **unmet in both** builds, and
it gets the higher-priority unmet goal as close to 100% as possible first.
- This is where "94%?95% at the expense of an already-doomed 97%?96%" is chosen
  correctly (both goal-2 values < 100%, so no ratchet; higher clamped score on
  goal 1 wins).

**Stage 3 — Min-max polish (raw values, priority order).**
Among builds tied on the entire clamped-score vector (i.e. they satisfy exactly
the same goals to exactly the same clamped degree — in practice, all goals met),
prefer the higher **raw** value, compared in priority order (goal 1 raw, then
goal 2 raw, …). This is the "min-maxer's dopamine" stage: it never sacrifices
satisfaction (Stages 1–2 already settled that), it only picks the biggest
headline numbers among already-equivalent builds.

**Stage 4 — Deterministic final tiebreak.**
Exact ties in all of the above are astronomically rare. Break them by a stable,
arbitrary key (e.g. the chosen candidates' instance keys) so results are
reproducible run-to-run. No product meaning is assigned to this stage.

### 1.3 Feasibility flag
A build is **feasible** iff every goal with `M > 0` meets its minimum (met-vector
is all 1s on the floored goals). Reported separately from the objective; the
optimizer always returns its best build under the comparator, feasible or not.

### 1.4 Worked examples (regression targets)
| Goals (priority order) | Build X | Build Y | Winner | Why |
|---|---|---|---|---|
| CR:100k, TM:100k | CR120k/TM90k | CR100k/TM89,999 | **X** | Both meet CR; Stage 2 on TM: 0.90 > 0.89999 |


| CR:100 (p1), TM:100 (p2), A can't reach 100 | A:96/B:40 | A:80/B:100 | **X** | (assuming B is a *met* goal for X) ratchet: Y drops met B to inch unmet A ? forbidden. See Q-B. |
  A:100 (p1), B:100 (p2); A can reach at most 96	A:94 / B:100 → met-vec (0,1)	A:96 / B:80 → met-vec (0,0)	X	Ratchet: Y drops met B to inch unmet, doomed A → forbidden. (0,1) > (0,0).
| A:100 (p1), B:100 (p2) | A:98/B:100 ? met-vec (0,1) | A:100/B:70 ? met-vec (1,0) | **Y** | Crossing into met on p1 justifies dropping p2 |
| A:100, B:100 (both unmet either way) | A:95/B:96 ? (0,0) | A:94/B:97 ? (0,0) | **X** | Stage 2: goal-1 clamped 0.95 > 0.94 |
| CR:100k, TM:100k (both met) | CR250k/TM100k | CR100k/TM100k | **X** | Stages 1–2 tie (all clamped 1.0); Stage 3 raw CR: 250k > 100k |

---

## 2. Architecture decision (LOCKED, pending sign-off)

This is a **rewrite of the objective + search core**, not a bug fix.

### 2.1 Keep (scaffolding is sound)
- CLI parsing, character/file discovery, report plumbing (`src/main.rs`).
- `Candidate` / `PairCandidate` types; `canonical_slot`, `paired_slot2`,
  `build_pairs` (paired-slot super-candidate construction — judged correct by
  both audits and re-confirmed here).
- Per-instance candidate identity (`optimizer_candidate_key`) — this is what
  makes "two distinct same-name rings" work; keep it.
- `report::print_report` (may need minor signature tweaks for the new result).

### 2.2 Throw out (encodes the false independent-maxima premise)
- The file header's correctness claim (lines 5–13) — it is the wrong model.
- `compute_global_max`, `compute_single_maxima`, `compute_pair_maxima`
  (as *search* drivers; per-stat maxima may survive only as B&B bounds — §2.3).
- `filter_compatible_single` / `filter_compatible_pair`.
- `safe_narrow_single` / `safe_narrow_pair` / `narrow_single` / `narrow_pair`.
- The `phase1_viable` two-path branch and reverse-priority narrowing.
- `truncate(8)`-and-warn (lines 152–162) — replaced by refuse-on-overflow (§3).

### 2.3 Build (new search core)
Two implementations that must agree:

1. **Naive oracle (test-only, `#[cfg(test)]`).** Full Cartesian enumeration of
   complete builds; evaluate each with the §1 comparator; keep best-so-far
   (streaming, never materialize the list). *Obviously correct by construction.*
   Only ever run on **tiny** inputs (?3 slots, ?4 items/slot) — see §4 math.

2. **Production search: dominance-pruned branch-and-bound.**
   - **Dominance pre-filter (the big win):** within each slot pool, discard any
     candidate that is `?` another candidate on **every goal stat** (ties broken
     so one survivor remains). Additivity + clamping make dominated items
     never uniquely useful. Real gear dominates heavily ? pools usually collapse
     to 1–2 effective candidates per goal set.
   - **Branch-and-bound over slots:** DFS one slot at a time, tracking running
     goal totals; prune a partial build when its optimistic completion (running
     totals + remaining per-slot maxima) cannot beat the best-so-far under the
     §1 comparator. This is where the old per-stat maxima are legitimately
     reused — as **admissible bounds**, not as a feasibility proxy.
   - **Exactness:** no candidate silently dropped; the cap (§3) bounds the input,
     dominance + B&B bound the *work*.

### 2.4 Why not naive-enumeration-as-product
At cap 5, worst case ? **1.2 × 10¹²** complete builds (§4) — far past any
tolerable budget. Naive enumeration is viable only as the *oracle on tiny
inputs*. Dominance + B&B is what ships.

---

## 3. Refuse-on-overflow guard (LOCKED)

- `MAX_CANDIDATES_PER_SLOT = 8` becomes a **supported-input contract**, enforced
  by **refusal**, not truncation. Applies to each **single slot** and each
  **paired family** (family pool size, before pairing).
- **Rust:** validate after grouping by canonical slot/family, before pairing/
  search. Return a proper error (e.g. `OptimizeError::TooManyCandidates {
  slot_label, count, max }`); `optimize` becomes `Result<_, OptimizeError>`;
  `run_optimize` prints an actionable message and exits non-zero.
- **Lua plugin (`src/lgo.lua`):** bucket chest+equipped items by optimizer slot
  family at export time; if any family exceeds the cap, **refuse to export**,
  list the offending families/counts, and tell the user to remove items. Use a
  named constant `MAX_CANDIDATES_PER_SLOT = 8` at the top of the file, printed
  in the refusal message. (Rust remains the defensive backstop.)
- Constant is authoritative in Rust; Lua value is manually kept in sync
  (documented in both places). Value is a knob, not sacred.

**Note:** performance tractability comes from **dominance pruning**, not from the
cap — 13 single slots dominate the combinatorics, not the 3 paired families.
So the cap stays user-generous at 8; we do **not** lower it for speed.

---

## 4. Worst-case size (cap sensitivity)

Layout: 13 single slots + 3 paired families. Single slot ? `k` choices;
paired family ? `C(k,2)` legal distinct-pair super-candidates.

`builds(k) = k^13 × C(k,2)^3`

| Cap k | k^13 | C(k,2)^3 | Total builds |
|---:|---:|---:|---:|
| 2 | 8,192 | 1 | ~8.2 × 10³ |
| 3 | 1,594,323 | 27 | ~4.3 × 10? |
| 4 | 67,108,864 | 216 | ~1.4 × 10¹? |
| **5** | 1,220,703,125 | 1,000 | **~1.2 × 10¹²** |
| 8 | ~5.5 × 10¹¹ | 21,952 | ~1.2 × 10¹? |

**Practical tolerance for a *naive* evaluate-every-build loop:** ~10? builds
(sub-second in Rust, keeps oracle code trivial). ? naive enumeration is safe
only up to cap ? 3, and is therefore **test-only**. Production must prune.

---

## 5. Test strategy

- **Delete** tests that encode old semantics:
  `test_truncation_warning_emitted_for_oversized_pool` (asserts truncation — now
  forbidden), and the `test_safe_narrowing_*` tests whose *expected values*
  assume raw-lexicographic maximization. Re-express any still-valid intent
  under the new objective.
- **Keep & re-verify** (rules unchanged by the new objective): all paired-slot
  identity tests (`test_no_self_pair_...`, `test_two_distinct_same_name_...`,
  `test_one_same_name_instance_alone_...`, `test_pair_family_consistency_...`,
  `test_pair_infeasible_when_minimum_exceeds_best_legal_pair`), feasibility
  boundary (`..._meets_minimum_exactly`, `..._one_below_minimum`), and the
  negative-stat tests.
- **Add deterministic objective tests** = the §1.4 table, one test each.
- **Add refuse-on-overflow tests** (single slot >8, paired family >8, both).
- **Add the oracle** (`#[cfg(test)]`), then **differential fuzz**: seeded RNG
  generates tiny legal cases (2–4 slots, 1–4 items, 1–3 goals, small stat
  ranges, sometimes paired); compare production search vs. oracle on
  **feasibility + clamped-score vector** (not exact item identity). Any mismatch
  is auto-formatted as a copy-pasteable fixture and promoted to a permanent
  regression test. Fast smoke run (~250 cases) in the normal suite; deep run
  (5,000+) under `#[ignore]`.

---

## 6. Open questions for sign-off

- **Q-A (M==0 goals in Stage 3):** For a `stat:0` goal, confirm the intended
  behavior is "no floor, but maximize its raw value as part of the min-max
  polish, in its priority position." (This matches today's `:0` = "maximize, no
  floor," and matches the existing test
  `test_safe_narrowing_preserves_higher_priority_max_when_no_minimum`, which
  expects the priority-1 `:0` stat to be maximized.) Assumed **yes**.
- **Q-B (mixed met/unmet in the ratchet):** Confirm the met-vector-lexicographic
  formalization in §1.2 captures your intent in *all* mixed cases, not just the
  ones tabulated. I believe it does, but it is the subtlest rule; a quick review
  of the §1.4 table is the fastest way to confirm.
- **Q-C (cap stays 8):** Confirm keeping the cap at 8 (user-generous) and relying
  on dominance+B&B for speed, rather than lowering the cap.
- **Q-D (scope of first PR):** One PR for objective+search+guard+oracle+fuzz, or
  split (e.g. guard first, then search-core rewrite, then fuzz)? Recommendation:
  land the **guard** as a small standalone PR first (low-risk, unblocks the Lua
  change), then the **search-core rewrite + oracle + fuzz** as the main PR.

---

## 7. Execution note
Implementation will be delegated to a coding-agent session (per Thalya's plan),
prompted from this document. Per `docs/MODEL_GUIDANCE.md`, the search core is
frontier-model territory (control flow + ownership + documented invariants).