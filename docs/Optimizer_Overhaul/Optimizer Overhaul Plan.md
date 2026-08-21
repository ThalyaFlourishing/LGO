> **HISTORICAL — superseded.** This document describes the pre-rewrite optimizer and/or the analysis or PR planning that led to it. The optimizer rewrite landed in PRs #44/#45 (with PR #46 cleanup); see `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` for the current objective and search design. Retained for historical context only; do not treat its algorithm/objective descriptions as current.

I need help implementing a correctness-first investigation and guardrail plan for my LOTRO gear optimizer repository.

## Current situation
We have already done an audit pass on the optimizer logic. The main conclusion is:

- the current optimizer is **not proven correct**
- it reasons heavily from independent per-slot / per-stat maxima
- it does **not** evaluate complete gear-set combinations as part of its core search logic
- Phase 1 viability is **not** equivalent to true feasibility
- however, we do **not yet have a fully validated concrete counterexample** proving that the current narrowing logic produces wrong answers on legal inputs
- therefore, the next step is **not** immediate redesign, but:
  1. add a proper overflow guard
  2. build an exact brute-force oracle for small cases
  3. fuzz-test the current optimizer against that oracle
  4. only then decide whether to replace the current search core

## Important design clarification: candidate cap
There is a cap, currently `MAX_CANDIDATES_PER_SLOT = 8`.

This cap is **not** meant to be an internal truncation heuristic.

It is intended as a **supported input contract**:

- if any single slot has more than the allowed number of candidates, optimization should be refused
- if any paired family (e.g. rings / ears / wrists) has more than the allowed number of owned items, optimization should be refused
- the user should be instructed to remove items and re-export
- the Lua plugin should ideally refuse export upstream for the same reason
- the Rust CLI should also defensively refuse if overflow somehow gets through

So:
- **do not silently truncate**
- **do not approximate**
- **refuse with a clear message**

Also:
- the value 8 is a starting point, not sacred
- it should be easy to adjust later
- in practice, 8 is already a high number from the user's point of view

## What I want implemented/planned
Please help with a concrete implementation plan, and then the code changes if requested, for these three items:

### 1. Refuse-on-overflow guard
Replace any truncate-and-warn behavior with proper overflow detection and refusal.

Requirements:
- validate candidate counts per single slot and per paired family
- fail clearly and instructively
- preserve the intended paired-slot semantics
- if useful, introduce a proper error type instead of continuing silently

### 2. Brute-force oracle
Create an exact exhaustive oracle for small/bounded cases, initially for tests.

Requirements:
- reuse the same slot grouping and paired-family construction rules as the main optimizer
- enumerate legal complete combinations
- evaluate feasibility on complete builds
- among feasible builds, choose the lexicographically best goal-total vector in CLI goal order
- if no feasible build exists, use an explicit documented fallback policy rather than an implicit one
- do not store every combination if streaming evaluation is simpler

### 3. Fuzz comparison harness
Create a deterministic random small-case generator that compares:
- current optimizer result
vs
- brute-force oracle result

Requirements:
- stay within the supported input contract
- generate small legal instances
- compare primarily on feasibility and goal-total vectors, not exact item identity if ties exist
- print or preserve good failure diagnostics so mismatches can be promoted into fixed regression tests

## Very important constraints
- Favor correctness over speed.
- Do not redesign the whole optimizer yet unless the evidence from oracle/fuzzing justifies it.
- Keep existing correct pieces if possible:
  - item parsing
  - slot grouping
  - paired-slot super-candidate construction
  - report formatting
- If a future redesign is needed, I would prefer replacing only the search/narrowing core.

## Terms / semantics
By “fuzzing” here, I mean differential testing with many small randomly generated legal cases, comparing the current optimizer to an exact oracle.

## What I want from you first
Please start by:
1. identifying the exact files/functions that need to change
2. proposing the safest implementation order
3. calling out any semantic decisions that must be fixed before coding
4. then, if the plan looks sound, we can proceed to implementation
