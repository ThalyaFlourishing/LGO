> **HISTORICAL — superseded.** This document describes the pre-rewrite optimizer and/or the analysis or PR planning that led to it. The optimizer rewrite landed in PRs #44/#45 (with PR #46 cleanup); see `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` for the current objective and search design. Retained for historical context only; do not treat its algorithm/objective descriptions as current.

## 1. Summary judgment — **unsound**

The current optimizer in `src/optimizer.rs` is **not provably correct** for the intended semantics.

More specifically:

- The paired-slot distinctness logic is **mostly conceptually correct**: it builds legal two-item “super-candidates” and does not self-pair a single owned instance.
- The current compatibility filtering is a **necessary-condition filter**, so it is mostly safe as a pruning step.
- But the algorithm then treats per-stat/per-slot maxima as if they imply the existence of a single combination satisfying all minima. That implication is false.
- The “safe lexicographic narrowing” is therefore **not actually safe in general**. It preserves independent per-stat maxima, not feasible multi-stat combinations.
- The hard cap `MAX_CANDIDATES_PER_SLOT = 8`, combined with candidate order coming from `HashMap` iteration in `main.rs`, is independently unsound and can discard the only feasible item before optimization begins.

My recommendation is **B/C leaning C**:

> The current pruning/narrowing strategy should not be trusted as an exact optimizer. Replace it with a complete exact search method — brute force for small cases, then branch-and-bound / dynamic programming / Pareto-frontier search for production cases.

---

## 2. How the current algorithm works

### Input path

`src/main.rs` reads the gear TOML, builds a `HashMap<String, GearItem>`, and passes all keys to `optimizer::optimize`.

Important detail: candidate identities are per TOML instance, using `optimizer_candidate_key(idx, &item)`, so duplicate owned copies can be represented distinctly. That part supports the intended paired-slot semantics.

However, `main.rs` then does:

```rust
let candidate_names: Vec<String> = resolved.keys().cloned().collect();
```

That means candidate order comes from `HashMap` iteration, which is not stable or semantically meaningful.

### Pool construction

`optimizer.rs` groups items by slot.

For paired slots:

- `Wrist1` and `Wrist2` are canonicalized into one wrist pool.
- `Finger1` and `Finger2` into one finger pool.
- `Ear1` and `Ear2` into one ear pool.

Then it builds pair “super-candidates” from two distinct entries:

```rust
for i in 0..pool.len() {
    for j in (i + 1)..pool.len() {
        pairs.push(PairCandidate::new(pool[i].clone(), pool[j].clone()));
    }
}
```

So the same candidate instance is not paired with itself.

If only one owned item exists for a paired family, the optimizer pairs it with an empty placeholder, not itself. That matches your intended rule.

### Candidate truncation

Before pairing, each per-slot pool is truncated:

```rust
pub const MAX_CANDIDATES_PER_SLOT: usize = 8;
...
pool.truncate(MAX_CANDIDATES_PER_SLOT);
```

This is a correctness problem. The first eight candidates are not necessarily the best eight, and in CLI use the order is derived from a `HashMap`.

So even before the narrowing algorithm runs, the optimizer may have discarded the only item needed for feasibility.

### Phase 1: compatibility filtering

For each candidate `C` in slot `K`, it checks each goal stat independently:

```text
C.stat(S) + best_of_other_slots(S, K) >= minimum(S)
```

If this fails for any goal stat, the candidate is removed.

This is a **necessary condition** for a candidate to be part of a feasible solution: if candidate `C` cannot meet a minimum even with the best possible contribution from every other slot for that same stat, then no feasible solution can use `C`.

So far, this filter is mostly safe.

But the reverse is not true:

> Passing this test does **not** mean the candidate can appear in any feasible complete gear set.

And:

> Every slot having at least one compatible candidate does **not** imply that any feasible gear set exists.

That is the first major proof failure.

### Phase 1 viability check

The code declares Phase 1 viable if every filtered pool is nonempty:

```rust
let phase1_viable = feasible_single.values().all(|p| !p.is_empty())
    && feasible_pair.values().all(|p| !p.is_empty());
```

This does **not** prove feasibility. It only proves that every slot has at least one candidate that individually passes a necessary compatibility test.

This is weaker than the required condition:

```text
∃ one candidate per slot such that all goal minima are met simultaneously
```

### “Safe” lexicographic narrowing

If Phase 1 is considered viable, the optimizer processes goals in reverse priority order.

For each stat, each slot is narrowed to candidates above the highest threshold that still:

1. Keeps every minimum “reachable” according to per-stat maxima.
2. Preserves the per-slot best value of higher-priority stats.

The intended idea is:

- First ensure lower-priority minima remain possible.
- Then narrow later by higher-priority stats.
- End with a lexicographic optimum.

But the reachability check again uses independent per-stat maxima, not actual combinations.

This means it can preserve:

```text
max A is still reachable somewhere
max B is still reachable somewhere
max C is still reachable somewhere
```

without preserving:

```text
A, B, and C are reachable together in one gear set
```

That is the core unsoundness.

### Phase 2 fallback

If Phase 1 is not viable, it does a simpler greedy lexicographic narrowing over the full pools.

But because Phase 1 viability is not a true feasibility test, the optimizer can take the “feasible” path even when no feasible solution exists, or after narrowing has destroyed all feasible solutions.

The final `feasible` flag is computed from the actual final gear set totals, so the program may still honestly report infeasible. But that is too late: the algorithm may not have searched correctly, and it may not have used the intended infeasible fallback policy.

---

## 3. Likely failure points

### Failure point A — per-stat maxima do not compose

The optimizer repeatedly assumes that this kind of computation is meaningful:

```text
global_max(S) = Σ per_slot_max(S)
```

For one stat alone, that is fine.

For multiple stat minima, it is not enough. The candidate that maximizes `CriticalRating` in a slot may be different from the candidate that maximizes `TacticalMastery`, and both may be incompatible with the candidate needed to meet a third stat.

This invalidates both:

- Phase 1 viability.
- Safe narrowing.

### Failure point B — preserving per-slot best higher-priority stat is insufficient

During safe narrowing, the code tries to avoid damaging higher-priority goals by checking that each slot’s best higher-priority stat value remains available.

But for multiple higher-priority stats, preserving each stat’s individual best does not preserve the feasible frontier.

Example:

```text
Candidate X: A=4, B=0
Candidate Y: A=0, B=4
Candidate Z: A=4, B=4
```

If a pruning step removes `Z` but keeps `X` and `Y`, then:

```text
best A is still 4
best B is still 4
```

But the combined candidate with both `A=4` and `B=4` is gone.

The optimizer’s check cannot distinguish those cases.

### Failure point C — Phase 1 viability is not feasibility

A slot can have a “compatible” candidate for each stat only because the algorithm imagines different other-slot choices for different stats.

That is an admissible upper bound, not a feasibility proof.

### Failure point D — candidate truncation is unsound

`MAX_CANDIDATES_PER_SLOT = 8` is a hard correctness break.

A feasible solution might require the 9th item in a slot.

Worse, in the CLI path, candidate order is derived from a `HashMap`, so “first 8” is arbitrary. This can produce unstable behavior and can explain results where changing goals does not meaningfully improve the secondary stat.

### Failure point E — fallback is only reached when a weak condition fails

The intended semantics say:

> If no feasible solution exists, return the best infeasible result only after correctly determining that no feasible solution exists.

The current optimizer does not correctly determine that no feasible solution exists. It only checks whether compatible pools are nonempty.

So fallback behavior is not semantically reliable.

---

## 4. Minimal counterexamples / test cases

### Counterexample 1 — Phase 1 says “viable” when no feasible solution exists

Use three stats: `A`, `B`, `C`.

In LOTRO stat terms, you could map:

```text
A = CriticalRating
B = TacticalMastery
C = Finesse
```

Goals:

```text
A >= 10
B >= 10
C >= 10
```

Two slots:

| Slot | Item | A | B | C |
|---|---:|---:|---:|---:|
| Head | H_A | 10 | 0 | 0 |
| Head | H_B | 0 | 10 | 0 |
| Chest | C_B | 0 | 10 | 0 |
| Chest | C_C | 0 | 0 | 10 |

All possible combinations:

| Combo | A | B | C | Feasible? |
|---|---:|---:|---:|---|
| H_A + C_B | 10 | 10 | 0 | no |
| H_A + C_C | 10 | 0 | 10 | no |
| H_B + C_B | 0 | 20 | 0 | no |
| H_B + C_C | 0 | 10 | 10 | no |

No feasible solution exists.

But the compatibility logic can keep:

```text
H_A, because B can supposedly come from Chest and C can supposedly come from Chest.
C_C, because A can supposedly come from Head and B can supposedly come from Head.
```

Those “supposedly” choices are mutually inconsistent.

So Phase 1 can be considered viable even though the feasible set is empty.

This proves the Phase 1 viability check is not a feasibility proof.

---

### Counterexample 2 — safe narrowing can destroy the only feasible solution

This is the more serious counterexample: a feasible solution exists, but the narrowing strategy can remove it.

Goals, in priority order:

```text
A >= 10
B >= 10
C >= 10
```

Map if desired:

```text
A = CriticalRating
B = TacticalMastery
C = Finesse
```

Two slots:

| Slot | Item | A | B | C | Role |
|---|---:|---:|---:|---:|---|
| Head | H_combo | 6 | 6 | 10 | required |
| Head | H_A | 10 | 0 | 0 | decoy |
| Head | H_B | 0 | 10 | 0 | decoy |
| Chest | C_combo | 4 | 4 | 0 | required |
| Chest | C_A_highC | 4 | 0 | 10 | decoy |
| Chest | C_B_highC | 0 | 4 | 10 | decoy |

There is exactly one obvious feasible combination:

```text
H_combo + C_combo = A=10, B=10, C=10
```

Now consider what the current narrowing can do.

During reverse-priority narrowing on `C`, the Chest slot may choose threshold `C >= 10`.

That keeps:

```text
C_A_highC: A=4, B=0, C=10
C_B_highC: A=0, B=4, C=10
```

and removes:

```text
C_combo: A=4, B=4, C=0
```

The optimizer’s checks can still pass because:

```text
Chest best A remains 4
Chest best B remains 4
Chest best C improves to 10
```

But the one candidate that had both `A=4` and `B=4` is gone.

After that, no feasible solution remains:

```text
H_combo + C_A_highC = A=10, B=6,  C=20  -> fails B
H_combo + C_B_highC = A=6,  B=10, C=20  -> fails A
```

So the algorithm can delete the only feasible solution while believing all minima remain reachable.

That directly disproves correctness of the “safe” narrowing strategy.

---

### Counterexample 3 — hard candidate cap can drop the only feasible item

One slot with nine candidates; cap is eight.

Goals:

```text
CriticalRating >= 10
TacticalMastery >= 10
```

Head candidates:

| Item | CR | TM |
|---|---:|---:|
| H1 | 100 | 0 |
| H2 | 90 | 0 |
| H3 | 80 | 0 |
| H4 | 70 | 0 |
| H5 | 60 | 0 |
| H6 | 50 | 0 |
| H7 | 40 | 0 |
| H8 | 30 | 0 |
| H9 | 10 | 10 |

Correct result:

```text
H9 is feasible.
```

Current optimizer may truncate to the first eight items and conclude no feasible item exists.

In real CLI use, the order is not reliable because it comes from a `HashMap`.

This is not a performance tradeoff; it is a correctness violation.

---

### Counterexample 4 — paired-slot distinctness oracle test

This is a positive test the current implementation mostly handles correctly and should be kept in any replacement.

Goals:

```text
CriticalRating >= 900
```

Owned finger items:

| Instance | Name | CR |
|---|---|---:|
| RingA instance 1 | Same Ring | 500 |

Only one owned copy exists.

Correct:

```text
Infeasible. Total CR must be 500, not 1000.
```

Then add a second distinct owned instance:

| Instance | Name | CR |
|---|---|---:|
| RingA instance 1 | Same Ring | 500 |
| RingA instance 2 | Same Ring | 500 |

Correct:

```text
Feasible. Total CR = 1000.
```

This tests the intended rule:

> Same display name may appear twice only if represented as two distinct owned input items.

---

### Proposed brute-force oracle tests

For tiny test cases, add a brute-force reference implementation under `#[cfg(test)]` that:

1. Builds the same slot pools.
2. Builds legal paired-slot super-candidates.
3. Enumerates the full Cartesian product.
4. Computes totals.
5. Partitions into feasible and infeasible combinations.
6. If feasible combinations exist:
   - choose the lexicographically greatest vector of goal totals.
7. If none exist:
   - apply an explicitly documented fallback ordering.

Then compare `optimize(...)` to the brute-force oracle.

The most valuable deterministic test classes are:

1. **Single-slot lexicographic feasible**
   - Confirms priority order.
2. **Two-slot minima tradeoff**
   - Confirms lower-priority minima can force sacrificing excess primary stat.
3. **Three-stat cross-candidate dependency**
   - The `H_combo` / `C_combo` counterexample above.
4. **No-feasible-but-compatible-pools-nonempty**
   - Confirms the optimizer does not confuse necessary compatibility with feasibility.
5. **Paired-slot no-self-pair**
   - One owned ring cannot fill two ring slots.
6. **Paired-slot duplicate-owned-copies**
   - Two distinct same-name rings can fill both slots.
7. **Candidate-cap regression**
   - Either remove the cap for exact mode or assert that capped mode is explicitly approximate.

---

## 5. Are the real observed examples consistent with these failure points?

Yes. Your observed examples are strongly consistent with the current bug pattern.

### Example 1

```text
lgo optimize cr:200000 tm:200000

CR = 250,389, met
TM = 58,117, failed
```

This is consistent with the optimizer over-prioritizing CR and failing to find or preserve TM-supporting combinations.

It could be caused by:

- infeasible actual search space,
- unsafe narrowing,
- candidate truncation,
- or some combination of those.

But the fact that TM remains very low while CR is far above the minimum is exactly the shape I would expect from an optimizer that does not correctly solve constrained multi-stat feasibility.

### Example 2

```text
lgo optimize cr:150000 tm:150000

CR = 214,503, met
TM = 58,117, failed
```

The fact that lowering the CR minimum drops CR but does not improve TM suggests the optimizer is not navigating the true feasible frontier. It may be getting trapped in the same narrowed pool or same truncated candidate set.

### Example 3

```text
lgo optimize tm:100000

TM = 105,303, met
```

This proves that high-TM gear exists somewhere in the candidate space.

It does not prove that `cr:150000 tm:150000` or `cr:200000 tm:200000` are feasible, but it does prove the optimizer can access higher-TM items when TM is the only priority.

That makes the multi-goal failures suspicious.

### Example 4

```text
lgo optimize cr:100000 tm:70000

CR = 243,078, met
TM = 67,224, failed
```

This is especially suspicious.

If a small sacrifice in CR could meet TM, the current algorithm is exactly the kind of algorithm that might miss that because it preserves independent maxima rather than feasible stat vectors.

### Example 5

```text
lgo optimize cr:70000 tm:100000

CR = 215,091, met
TM = 99,675, failed
```

This also matches the likely failure pattern: CR is wildly above its minimum while TM narrowly misses. A correct feasible-first optimizer should first determine whether any set meets both minima. It should not keep excess CR merely because CR is priority 1 if that prevents meeting a stated TM minimum.

Important qualification:

> I cannot prove from these output numbers alone that a feasible solution exists for the real fixture goals. But I can say the current implementation is not a reliable witness either way.

A brute-force or exact oracle is needed for the fixture to answer that definitively.

---

## 6. Comparison against the older JS optimizer

The old `docs/Old_LGO_Version/LGO.js` implementation is crude but important because it does the one thing the current Rust optimizer does not do:

> It recursively enumerates complete combinations and then checks complete-combination totals.

The relevant structure is:

- `generateCombinations(...)` recursively picks one item per slot.
- `processCombination(...)` sums the full gear set.
- Then it checks all minima together:

```javascript
if ((crit >= critMinimum) && (tact >= tactMinimum) && (tmit >= tmitMinimum)) {
    ...
}
```

That is semantically much closer to the intended feasible-first rule.

However, the old JS optimizer is not ideal:

- It appears hard-coded around `crit`, `tact`, and `tmit`.
- It stores/report-lists all feasible permutations, which can blow memory.
- It rejects duplicate item names, not just duplicate object references, which is stricter than your current intended paired-slot semantics.
- It does not implement a general lexicographic objective over arbitrary CLI goals.

So I would not copy it blindly.

But as a behavioral reference, it is useful because it treats feasibility as a property of a **complete combination**, not as a sum of independent per-slot maxima.

---

## 7. Recommended next step — redesign around an exact method

My recommendation:

> **C. The current approach should be abandoned in favor of a different exact search method**, or at least replaced with a correctness-first exact core and optional approximate pruning later.

A good path would be:

### Step 1 — define a brute-force oracle

For unit tests and small cases, implement exhaustive enumeration.

This becomes the reference for correctness.

### Step 2 — remove or quarantine `MAX_CANDIDATES_PER_SLOT`

For correctness mode, no candidate should be silently dropped.

If a cap remains, it should be explicitly approximate and reported as such.

### Step 3 — replace “safe narrowing” with an exact search

Reasonable exact approaches:

#### Option 1: Branch-and-bound over slots

Search one slot at a time.

Maintain:

- current totals,
- remaining per-stat upper bounds,
- current lexicographic best feasible solution.

Prune only when it is mathematically safe:

- If current totals plus remaining maxima cannot meet a required minimum, prune.
- If current totals plus remaining maxima cannot beat current best lexicographically, prune.

This is likely the most straightforward Rust implementation.

#### Option 2: Pareto-frontier dynamic programming

For each processed slot, maintain nondominated partial totals over goal stats.

A partial total vector dominates another if it is at least as good in all relevant stats and better in at least one, subject to preserving enough information for lexicographic comparison.

This can be exact and efficient when the number of goal stats is small.

#### Option 3: ILP-style formulation

Model each candidate as a binary variable:

```text
choose exactly one candidate per normal slot
choose exactly one pair candidate per paired family
meet all minima
maximize lexicographic objective
```

Lexicographic objective can be solved by sequential optimization:

1. Maximize goal 1 among feasible solutions.
2. Add equality constraint for optimum goal 1.
3. Maximize goal 2.
4. Continue.

This is conceptually clean but adds solver complexity unless implemented manually.

### Step 4 — use the old JS optimizer only as a test oracle clue

Do not port the old algorithm directly, but preserve its core correctness principle:

> enumerate/evaluate complete combinations, not isolated per-slot stat maxima.

---

## Final answer

The current optimizer is **unsound** for the intended semantics.

The proof fails because it treats independent per-stat maxima as if they compose into feasible complete gear sets. They do not. The compatibility filter is only a necessary condition, not a feasibility proof, and the “safe” narrowing can remove candidates required for the only feasible solution.

The observed real examples are consistent with this bug pattern, especially the cases where CR remains far above its minimum while TM narrowly or badly misses its minimum.

Recommended next step: add a brute-force oracle for tiny deterministic tests, then replace the current narrowing strategy with an exact search method. The current paired-slot instance model is worth preserving, but the pruning/narrowing strategy should not be the basis of correctness.
