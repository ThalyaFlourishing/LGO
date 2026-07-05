> **HISTORICAL — superseded.** This document describes the pre-rewrite optimizer and/or the analysis that led to it. The optimizer was rewritten in PRs #44/#45 (see `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` for the current objective and design). Retained for historical context only; do not treat its algorithm/objective descriptions as current.

This new framing materially changes the assessment of **Claim D**, but it does **not** rescue the audit's core problem. Let me re-review with the corrected understanding, and then answer your meta-question about whether to re-run the audit.

## How your clarification changes things

**The cap is now a precondition, not an algorithm parameter.** Under your intended design:

- `MAX_CANDIDATES_PER_SLOT = 8` is a **user-facing input contract**, enforced (ideally) by a hard refusal: if any slot or paired family exceeds 8 owned candidates, the program declines to optimize and tells the user to trim and re-export.
- Within the contract (? 8 per slot/family), **no truncation ever occurs**, so "the 9th item is silently dropped" is outside the supported input domain — it's user error that the program should *reject*, not silently approximate.

This is a reasonable and defensible design. From the user's point of view 8 swappable items for a single slot is already extravagant, and refusing-with-instructions is cleaner than truncating-with-a-warning.

## Re-assessment of each claim under the corrected model

### Claim D — now **largely INVALID / a design mismatch, not a correctness flaw**

This is where your clarification lands hardest, and it **further weakens** the audit (which already overstated D).

- The current code's actual behavior — `truncate(8)` + a *warning* + continue (`optimizer.rs:151-162`) — is simply **the wrong enforcement mechanism for your intended contract.** It should be a hard **refuse-to-optimize**, not a silent truncate. So Counterexample 3 (the "9th item is the only feasible one") describes an input that, under your design, **must never reach the optimizer at all** — the program should have bailed out before optimization with a "too many candidates for slot X" error.
- Therefore Claim D does **not** describe a soundness bug in the *optimizer*. It describes:
  1. a **missing precondition guard** (the program should refuse > 8, currently it truncates), and
  2. a **call-site nondeterminism** (`main.rs:143` `HashMap` key order) that becomes **entirely moot** once the > 8 case is a hard error, because at ? 8 nothing is ever dropped and order only affects tie-break *presentation*, not feasibility.

So under the corrected model: **the cap is not a correctness violation at all.** The only legitimate residue of Claim D is "the current code truncates-and-warns instead of refusing," which is a **5-line input-validation fix**, not anything about the search algorithm. The audit's framing of D as "independently a correctness violation" was overstated before and is now essentially **wrong for your design intent**.

One caveat worth keeping: the nondeterministic `HashMap` ordering still has a *cosmetic* effect — among equally-optimal candidates, which one is reported can vary run-to-run. If you care about reproducible output you may still want to sort at the call site, but that is polish, not correctness.

### Claims A, B, C — **unchanged**

Crucially, **your clarification does not touch the central finding.** A, B, and C are all about what happens *inside* the supported input domain (? 8 per slot). The independent-maxima problem exists just as much with 3 candidates per slot as with 8:

- **Claim A (independent per-stat maxima don't compose):** still **correct**. Counterexample 1 uses only **2 candidates per real slot** — comfortably inside your contract — and still defeats Phase 1 viability. The disease is fully present at legal input sizes.
- **Claim B (viability ? feasibility):** still **correct and demonstrated** (Counterexample 1).
- **Claim C (safe narrowing may destroy the only feasible set):** still **conceptually credible but undemonstrated**, and still the key open question. The audit's Counterexample 2 remains **invalid against the real `safe_narrow_*`** for exactly the reasons in my prior review (it mis-models guards (a)/(b), the highest-passing-threshold rule, and the per-slot recompute) — and your cap clarification doesn't help it, since that counterexample also uses only 2–3 candidates per slot.

So the corrected bottom line shifts only on D:

> The prior audit is **partially correct**. Its central thesis (A/B) is right and demonstrated. Its scariest claim (C) is plausible but unproven, and its key counterexample (2) is invalid. Its Claim D, under your intended design, **collapses to a missing input-validation guard** and is **not** an optimizer-correctness issue.

## What this means for the recommendation

Your clarification makes the **redesign question even more clearly "not yet justified," and the real fix even smaller**:

1. **Add the precondition guard** (this is now the *correct* resolution of all of "Claim D"): before optimizing, count candidates per slot *and per paired family*; if any exceeds the cap, **refuse and instruct the user to trim and re-export.** Replace `truncate()` accordingly. This is independent of the search algorithm and should go in regardless.

2. **The cap makes brute force trivially viable as a real option, not just an oracle.** This is the most important consequence. With a hard ceiling of 8 per slot/family:
   - Single slots: ? 8 choices each.
   - Paired families: ? 8 owned items ? at most 8·7/2 = **28** legal pair-candidates each (and the *placeholder* cases are tiny).
   
   The worst-case product across 13 normal slots + 3 paired families is bounded and constant. Exhaustive enumeration of complete combinations — exactly what the old `LGO.js` did and what the optimizer fails to do — may be **directly tractable** here, which would let you delete the entire Phase-1/safe-narrowing apparatus rather than repair it. That is a *simplification*, not a heavyweight redesign. **(This should be checked with an actual worst-case multiplication against the real slot set before committing — the product of sixteen factors up to 8–28 can still be large, so branch-and-bound with bounds remains the safe fallback if naive enumeration is too big.)**

3. **Still build the brute-force oracle first** to settle Claim C. Under the cap, the oracle is cheap *and* may double as the production algorithm.

So your two clarifications combine to **shrink the problem**, not grow it: D becomes a guard, and the cap turns exhaustive evaluation from "infeasible at 19 slots" (the old JS's O(n!) wall) into "bounded and possibly shippable."

## Your closing computational question, revisited

You no longer need to *lower* 8 for soundness reasons (soundness is about A/B/C, which are size-independent). 8 is a **performance/UX knob** only. The right move is:

- Keep 8 as the **enforced** ceiling for both single and paired slots (refuse above it).
- Compute the **worst-case combination count** under that ceiling against your actual slot list. If naive enumeration is comfortably fast, ship brute force and retire the narrowing logic. If the paired-family multiplication pushes it too high, drop to **branch-and-bound** (admissible bound, prune on min-infeasibility and lexicographic dominance), which reuses your existing pool/pair builders.
- Only lower 8 if *both* exact methods prove too slow on real data — which, at these bounds, is unlikely.

## Your meta-question: re-run the audit, or not?

**My recommendation: do not re-run the full audit yet. Do one targeted thing instead.**

Reasoning:
- The omission contaminated **only Claim D**. Re-running the whole audit risks re-litigating A/B/C (which are already correctly settled) and re-generating the same invalid Counterexample 2, wasting the effort.
- The two things that actually need doing are both *narrow*:
  1. **Re-scope Claim D** as "missing input-validation guard; refuse > 8 per slot/family" — which I've just done above; you don't need a model round-trip for that.
  2. **Resolve Claim C empirically** with a brute-force oracle + fuzz. This is the one genuinely open question, and *no amount of re-auditing by prose will settle it* — it needs executable verification.

So the high-value next action is **not another audit pass** but **writing the oracle and fuzzing `optimize()` against it within the ? 8 contract.** That will either produce the valid Counterexample 2 the first audit failed to find (proving C and justifying the rewrite) or fail to find one after heavy fuzzing (suggesting C is false-in-practice and the narrowing can stay, pending a proof). Either outcome is more decisive than a re-audit.

If you'd like, I can **kick off a coding-agent session** to: (a) add the per-slot/per-paired-family precondition guard that refuses > 8 and replaces the silent truncation, and (b) add a `#[cfg(test)]` brute-force oracle plus a fuzz harness comparing it to `optimize()` — which would directly settle the one remaining uncertainty. Just confirm and I'll start it against `ThalyaFlourishing/LGO`.