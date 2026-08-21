# PR 4 — Documentation Reconciliation After the Optimizer Rewrite

**Repo:** ThalyaFlourishing/LGO — cut a feature branch from `main`.
**Prerequisite:** PRs #44 (overflow guard), #45 (objective rewrite), and #46
(clippy cleanup + test comments) should all be merged first. If #46 is not yet
merged, STOP and wait — this audit must reflect the final merged code.

**Nature of this PR:** Documentation + user-facing help text only. **No logic
changes, no test changes.** The one code file it may touch is `src/main.rs`, and
only the `print_usage` help-text strings (see Task C).

## Why this PR exists
The optimizer was just rewritten (PRs #44/#45). The change was large enough that
existing docs likely describe code and behavior that **no longer exist**. Stale
docs that read as current are actively harmful — they mislead future readers and
AI agents. This PR brings all documentation into agreement with the current
code.

## Ground truth (what the code does NOW — verify against `src/optimizer.rs`)
The old optimizer used an **independent-per-slot-maxima** search with a
**raw-lexicographic** objective ("maximize stat 1, then stat 2 as a tiebreaker")
and a two-phase feasible/infeasible fallback. **All of that was deleted.**

The current optimizer implements a **clamped-satisfaction objective** and an
**exact search**. Authoritative spec:
`docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` §1. In brief,
two complete builds are compared by this total order (goal stats only; non-goal
stats never influence the search):
1. **Met-vector**, lexicographic by priority — for each goal, met (`1`) vs unmet
   (`0`); `minimum == 0` counts as always met. (This yields the "ratchet":
   crossing a higher-priority goal into *met* justifies dropping a lower goal;
   merely raising a still-unmet higher goal does not.)
2. **Clamped-score vector**, lexicographic by priority — `min(total, minimum)`
   per goal (integer comparison; equivalent to `min(total/minimum, 1.0)`; no
   floats). Only differentiates goals unmet in both builds.
3. **Raw goal totals**, lexicographic by priority — the "min-max polish": among
   otherwise-equivalent builds, prefer the biggest headline numbers.
4. **Deterministic tiebreak** by sorted candidate instance keys.

Key behavioral facts that docs must reflect:
- Overshoot above a goal's minimum is worthless *except* as the Stage-3 polish
  tiebreak among builds that are otherwise equal; it is never pursued at the
  expense of a lower-priority goal still short of target.
- Feasible and infeasible cases use the SAME comparator — there is no separate
  "infeasible fallback" path anymore.
- The search is **exact** (dominance pre-filter + branch-and-bound), verified in
  tests against a brute-force **oracle** via a differential **fuzzer**.
- `MAX_CANDIDATES_PER_SLOT = 8` is a hard input contract: more than 8 candidates
  in any single slot or paired family (Wrist/Finger/Ear) causes `lgo optimize`
  to **refuse** with an error (not truncate). (PR #44.)
- Negative per-item stat values are supported (a stat can go negative on one item
  and be compensated across slots); the optimizer must not reject them.

## Task A — audit and produce a findings report
Read the entire `docs/` directory (and any `README*`, plus `src/main.rs`
`print_usage`). For EACH document, classify it:
- **CURRENT** — accurate, no change needed.
- **STALE** — describes the old optimizer engine/objective as if current; needs
  correction.
- **HISTORICAL** — a point-in-time record (e.g. the audit chain) that should be
  *preserved* but clearly marked as superseded, not rewritten.

Write the findings as a short table at the top of the PR description: doc path ?
classification ? one-line reason ? planned action.

Known starting points to check (confirm; do not assume this list is complete):
- `docs/Optimizer_Overhaul/01`–`06` — the audit chain describing the OLD
  algorithm, the unproven "Claim C", and the "defer the redesign" staging that
  PR #45 superseded. Very likely **HISTORICAL**.
- `docs/Optimizer_Overhaul/07` — the locked spec. Should be **CURRENT** (it is
  the source of truth); update only if it still says "no code yet / pending
  sign-off" so it reflects that the rewrite is now implemented.
- The `PR1`–`PR3` (#44–#46) agent-prompt docs in `docs/Optimizer_Overhaul/` —
  **HISTORICAL** records of completed work; leave as-is or lightly banner.
- `docs/AGENT_CONTEXT.md` — the durable cold-start brief. Likely **STALE**; must
  point at `07` as the authoritative objective and state the rewrite is done.
- `docs/MODEL_GUIDANCE.md` — likely references the old optimizer structure; the
  "high friction, use a frontier model" guidance stays, but any specifics about
  the old two-phase/narrowing design are now wrong.
- Any optimizer/resolver design doc (e.g. `docs/RESOLVER_DESIGN.md`, a
  `docs/merge-brief.md`, or an optimizer design note) — check for the phrase
  "slots do not interact" / "independent maxima" / "greedy narrowing" and any
  "maximize stat 1 then stat 2" objective description.
- `docs/Command Line Reference.txt` — verify `optimize` behavior is described
  correctly, including the overflow-refusal error and the deep fuzzer command
  (`cargo test -- --ignored`).

## Task B — correct the STALE docs
Rewrite the stale portions to match the Ground Truth above. Specifically hunt
for and fix any text that:
- Describes the objective as "maximize the first stat, then later stats as
  tiebreakers" (the OLD, rejected objective).
- Describes "independent per-slot maxima", "slots do not interact", "safe
  narrowing", "compatibility filtering", or a "two-phase feasible/infeasible
  fallback" as the current design.
- Describes candidate overflow as "truncated with a warning" (now: refused).
- Implies overshoot above a goal is valuable in itself.
Replace with concise, accurate descriptions of the clamped-satisfaction
objective, the exact dominance+branch-and-bound search, the oracle/fuzzer
verification, and the refuse-on-overflow contract. Reference
`docs/Optimizer_Overhaul/07` as the authoritative objective rather than
duplicating its full detail.

## Task C — fix the `src/main.rs` usage help text
`print_usage` in `src/main.rs` (around the "Stat goals:" section) contains
user-facing help that describes the OLD objective, e.g. wording like *"the first
stat is maximised first, with later stats used only as tiebreakers."* Update
these lines to describe the current behavior accurately and briefly, e.g.:
- Goals are listed in priority order.
- Each goal has a minimum; the optimizer gets every goal as close to its minimum
  as possible, honoring priority, and does not pursue overshoot on a higher
  goal at the expense of a lower goal still short of its target.
- A minimum of 0 means "no floor, but maximize" (used for the polish/tiebreak).
Keep it concise — this is CLI help, not a spec. Do NOT change any parsing logic,
flags, or other `main.rs` code — only the printed help strings.

## Task D — mark HISTORICAL docs
For docs classified HISTORICAL (the `01`–`06` audit chain in particular), add a
short banner at the very top of each, e.g.:

> **HISTORICAL — superseded.** This document describes the pre-rewrite optimizer
> and/or the analysis that led to it. The optimizer was rewritten in PRs #44/#45
> (see `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` for
> the current objective and design). Retained for historical context only; do
> not treat its algorithm/objective descriptions as current.

Do NOT rewrite their bodies — they are a record. The banner is enough.

## Out of scope
- Any change to optimizer/resolver/report/Lua logic or tests.
- Rewriting the historical audit docs' bodies (banner only).
- New features or new docs beyond what's needed for accuracy.

## Acceptance criteria
- The PR description contains the Task A findings table (every doc classified).
- No doc remaining in the repo describes the OLD objective or OLD search engine
  as current behavior.
- `src/main.rs` `print_usage` describes the current objective; `cargo build`,
  `cargo test`, `cargo clippy --all-targets`, `cargo fmt --check` all still pass
  (the help-text edit must not break the build or the `print_usage`-adjacent
  tests).
- Historical docs are preserved with a superseded banner, not deleted or
  rewritten.
- `docs/AGENT_CONTEXT.md` accurately points a cold-start reader/agent at the
  current objective (`07`) and the completed rewrite.