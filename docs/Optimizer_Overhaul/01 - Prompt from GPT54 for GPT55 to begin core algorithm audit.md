> **HISTORICAL — superseded.** This document describes the pre-rewrite optimizer and/or the analysis or PR planning that led to it. The optimizer rewrite landed in PRs #44/#45 (with PR #46 cleanup); see `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` for the current objective and search design. Retained for historical context only; do not treat its algorithm/objective descriptions as current.

I need a **correctness audit** of the optimizer in this repository, especially `src/optimizer.rs`. (Repo address: https://github.com/ThalyaFlourishing/LGO )

## Context
This project is a LOTRO gear optimizer.

The intended optimization semantics are:

1. Each CLI goal is of the form `<stat>:<minimum>`.
2. A solution is **feasible** if **all** stated minima are met.
3. If one or more feasible solutions exist:
   - choose among feasible solutions by **strict lexicographic priority order**
   - maximize goal 1 first
   - break ties with goal 2
   - then goal 3, etc.
4. If **no** feasible solution exists:
   - return the best infeasible result according to the intended fallback policy,
   - but only after correctly determining that no feasible solution exists.
5. For paired slots (e.g. rings/ears/wrists), two distinct owned instances must be used; the same instance may not fill both paired slots unless there are actually two owned copies represented as distinct input items.

## My concern
I do not trust the current “greedy” / narrowing / pruning approach. It may be incorrectly maximizing earlier stats beyond their minima at the expense of later minima.

I have observed behavior that appears wrong.

## Real observed examples
Using the current fixture test data:

### Example 1
`lgo optimize cr:200000 tm:200000`

produces:

- Critical Rating = 250,389 (minimum 200,000, met)
- Tactical Mastery = 58,117 (minimum 200,000, failed)

### Example 2
`lgo optimize cr:150000 tm:150000`

produces:

- Critical Rating = 214,503 (minimum 150,000, met)
- Tactical Mastery = 58,117 (minimum 150,000, failed)

In example 2, CR dropped by ~36k, but TM did not improve at all.

### Example 3
`lgo optimize tm:100000`

produces:

- Tactical Mastery = 105,303 (minimum 100,000, met)

So >100k TM is demonstrably available somewhere in the search space.

### Example 4
`lgo optimize cr:100000 tm:70000`

produces:

- Critical Rating = 243,078 (minimum 100,000, met)
- Tactical Mastery = 67,224 (minimum 70,000, failed)

This suggests CR is being kept far above its minimum while TM misses its minimum.

### Example 5
`lgo optimize cr:70000 tm:100000`

produces:

- Critical Rating = 215,091 (minimum 70,000, met)
- Tactical Mastery = 99,675 (minimum 100,000, failed)

This suggests the primary stat is being maximized rather than merely kept above its minimum, at the expense of meeting the secondary minimum.

## Hypothetical counterexample structure
Consider this small case:

- Requirements: `StatA:10`, `StatB:10`

- Slot 1:
  - Item 1: A=12, B=0
  - Item 2: A=5, B=5
  - Item 3: A=5, B=5

- Slot 2:
  - Item 1: A=12, B=0
  - Item 2: A=5, B=5
  - Item 3: A=5, B=5

A correct optimizer should not simply pick the two A=12/B=0 items and then report StatB infeasible, because there exists a feasible solution meeting both minima: choose the two A=5/B=5 items for totals A=10, B=10.

This is the sort of failure I am worried the current algorithm may permit.

## What I want from you
Please perform a **correctness audit**, not an immediate code rewrite.

### Phase 1 — understand current implementation
Read the existing optimizer and explain:
- what algorithm it is actually using,
- what pruning/narrowing assumptions it relies on,
- and what correctness claims would need to hold for it to be valid.

Important files likely include:
- `src/optimizer.rs`
- `src/gear.rs`
- `src/gearstats.rs`
- `src/main.rs`
- any relevant tests

### Phase 2 — determine whether the current algorithm is sound
I want you to answer explicitly:

1. Is the current algorithm **provably correct** for the intended semantics above?
2. If not, where exactly does the proof fail?
3. Can you construct minimal counterexamples?
4. Do the observed examples above fit a likely bug pattern in the current implementation?

### Phase 3 — use brute-force on tiny cases as an oracle
Please create or propose **small exhaustive tests** that compare:
- current optimizer output
against
- brute-force enumeration of all combinations

I want toy cases that are small enough to enumerate completely and that test:
- minima satisfaction,
- lexicographic priority,
- paired-slot distinctness,
- and cases where meeting a lower-priority minimum requires sacrificing excess in a higher-priority stat.

### Phase 4 — compare against older optimizer
I am also attaching an older, cruder optimizer implementation that behaves more as expected in practice.
Use it only as:
- a behavioral comparison reference,
- a clue to expected semantics,
- and possibly a source of test ideas.

Do **not** assume its algorithm is ideal or copy it blindly.

### Phase 5 — recommend fix scope
At the end, tell me which of these is true:

A. The current algorithm is conceptually sound and only needs a localized bug fix.

B. The current pruning/narrowing strategy is unsound in general, but can be replaced with another still-efficient exact method.

C. The current approach should be abandoned in favor of a different exact search method (e.g. branch-and-bound, dynamic programming, ILP-style formulation, or another complete search).


## Additional resource:
The repo has a folder named 'docs\Old_LGO_Version'. In it is a previous and much simpler incarnation of this same idea written in JavaScript. It has no data-extraction whatsoever; it is just the bare algorithm. It was known to work as expected; please refer to it as an example of the type of algorithm we now need.

## Constraints
- Favor correctness over speed.
- Be explicit when making claims.
- If you assert correctness, sketch the proof.
- If you assert incorrectness, provide a concrete counterexample.
- Do not silently change the intended semantics.
- Do not implement fixes yet unless I ask; first produce the audit findings and recommended path.

## Deliverable format
Please respond with:

1. **Summary judgment** — sound / unsound / likely bug.
2. **How the current algorithm works**.
3. **Likely failure points**.
4. **Minimal counterexamples / test cases**.
5. **Whether the real observed examples are consistent with those failure points**.
6. **Recommended next step** — small fix or redesign.

## Optional add-on
**Repository note:** this project already has substantial tests. If you add proposed audit tests, prefer small deterministic unit/integration tests over large fixture-dependent behavioral claims.
