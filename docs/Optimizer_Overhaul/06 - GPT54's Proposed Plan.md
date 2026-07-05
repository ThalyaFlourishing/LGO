> **HISTORICAL — superseded.** This document describes the pre-rewrite optimizer and/or the analysis or PR planning that led to it. The optimizer rewrite landed in PRs #44/#45 (with PR #46 cleanup); see `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` for the current objective and search design. Retained for historical context only; do not treat its algorithm/objective descriptions as current.

--- --------------------------------------------------------------------------------
THE ACTUAL PLAN - TO BE EXECUTED BY INSTANCE OF GPT55 (can cross-check on Claude O8)
--- --------------------------------------------------------------------------------
Yes — pushing the item-limit check upstream into the Lua plugin is a good idea.

Best design is probably:

- **authoritative limit lives in one place**
- Lua export enforces it early and explains what to remove
- Rust also defensively checks and refuses if overflow somehow still gets through

That avoids wasted work and keeps the CLI honest.

## “fuzz” / “fuzzy” here
In this context:

### Fuzz testing
Automatically generate lots of small test inputs and run the code on them looking for:
- crashes
- assertion failures
- wrong answers
- mismatches against a known-correct oracle

Here, the important kind is **differential fuzzing**:
- generate many random tiny gear sets
- run both:
  - current optimizer
  - brute-force oracle
- compare outputs
- if they differ, save the case as a counterexample

### “Fuzzy”
Usually just means:
- approximate
- heuristic
- non-exact
- or randomly varied

In your case, “fuzzy” should **not** mean approximate optimization.  
You want:
- exact oracle
- random test-case generation

---

# Concrete implementation plan

I’ll break this into three parts:

1. **refuse-on-overflow guard**
2. **brute-force oracle**
3. **fuzz comparison harness**

---

## 1. Refuse-on-overflow guard

### Goal
If any slot exceeds the allowed candidate count:
- do **not** optimize
- emit a clear error
- instruct the user to remove items and re-export

And ideally:
- the Lua plugin catches it first
- Rust re-checks defensively

---

### 1A. Centralize the candidate-limit constant
You do **not** want this hard-coded in multiple places.

#### Add one shared Rust-side constant
Likely in `src/optimizer.rs` or a small shared config module:

```rust
pub const MAX_CANDIDATES_PER_SLOT: usize = 8;
```

You already have this constant in the optimizer; the key change is semantic:
- it becomes a **supported input limit**
- not a truncation limit

#### Add a Lua-side exposed value
Because Lua cannot import Rust constants directly, you need a duplication strategy.

Practical options:

##### Option 1 — duplicate for now
- Rust constant: authoritative for CLI
- Lua constant: same numeric value
- keep them manually synchronized

##### Option 2 — generate plugin file/config later
Not necessary immediately.

For now I’d use **Option 1**, but make the constant name loud and documented in both places.

---

### 1B. Rust: replace truncate-and-warn with validate-and-error
Current behavior in `optimizer.rs`:
- pools are truncated to 8
- warning emitted

New behavior:
- detect overflow before any truncation/pairing
- return an error instead of continuing

### Required design change
Right now `optimizer::optimize(...)` likely returns a result directly, not a `Result<_, _>`.

You will need to decide where the error should surface.

#### Best design
Change optimizer entrypoint to return something like:

```rust
Result<OptimizationResult, OptimizeError>
```

Add an error variant like:

```rust
TooManyCandidates {
    slot_label: String,
    count: usize,
    max: usize,
}
```

For paired families:
- `Wrist`
- `Finger`
- `Ear`

For single slots:
- `Head`
- `Chest`
- etc.

### Validation timing
Validate **after grouping by canonical slot family**, but **before**:
- truncation
- pairing
- narrowing
- optimization

Because paired families must be validated on the **family pool size**, not post-pair count.

### Error message
Make it explicit and actionable, e.g.:

> Too many candidates for slot family `Finger`: 11 provided, maximum allowed is 8.  
> Remove some items from the `lgo` chest and re-export before running `lgo optimize`.

### Main.rs changes
`run_optimize()` must handle the new optimizer error and print the friendly guidance.

---

### 1C. Lua plugin: refuse export upstream
This is a separate but aligned improvement.

### Goal
Before writing `.plugindata`, the plugin should:
- count candidate items by optimizer slot family
- if any family exceeds the configured max:
  - refuse export
  - print a clear message listing the overflowing slot(s)
  - tell the user to remove items from the `lgo` chest

### Important subtlety
This requires the Lua plugin’s slot classification to match optimizer slot-family semantics closely enough.

You do **not** necessarily need exact final Rust-side canonical slot resolution.  
You just need export-time family bucketing based on equipped/storage slot categories:
- Head
- Chest
- Wrist family
- Finger family
- Ear family
- etc.

### Lua behavior
Suggested flow:
1. scan chest contents
2. bucket by slot/family
3. compare each count to configured limit
4. if any overflow:
   - print summary
   - do not export
5. else export normally

### Configurability
Do **not** hard-code the number invisibly.

Good immediate approach:
- define a named Lua constant near top of file, e.g. `MAX_CANDIDATES_PER_SLOT = 8`
- print it in the refusal message

Later, if needed:
- make it configurable in plugin settings

---

## 2. Brute-force oracle

### Goal
Create a **known-correct exact solver** for small/bounded cases to use in tests.

This is not initially about performance.
It is about:
- correctness
- proving/refuting current algorithm behavior
- generating real counterexamples

---

### 2A. Scope
Put this under `#[cfg(test)]` first.

Likely best location:
- `src/optimizer.rs` test module
or
- a dedicated test helper module if it gets large

I’d lean toward:
- small helper functions in `src/optimizer.rs` tests first
- promote later if needed

---

### 2B. Reuse existing pool construction logic
Do **not** duplicate slot-family semantics from scratch if avoidable.

Reuse the same machinery for:
- grouping items into slot pools
- building paired-slot super-candidates

That way the oracle differs only in **search strategy**, not in preprocessing assumptions.

If existing functions are private, expose/internalize only what tests need.

### Desired shared pipeline
For both:
- current optimizer
- oracle

Use identical:
- slot grouping
- paired-slot pool creation
- candidate identity rules

Then diverge only at:
- current narrowing/search
vs
- exhaustive combination enumeration

---

### 2C. Oracle semantics
The oracle should implement your intended semantics exactly:

1. enumerate every legal full gear combination
2. compute total stats
3. partition combinations into:
   - feasible
   - infeasible

Then:

### If feasible combinations exist
Choose the one with the best **lexicographic vector of goal totals** in input order.

Example:
- goals `[CR, TM, FN]`
- compare vectors:
  - `(210000, 95000, 30000)`
  - `(205000, 120000, 50000)`

The first wins if `CR` is priority 1.

### If no feasible combination exists
You need an explicit fallback definition.

This is important: do **not** leave oracle fallback vague.

If your intended fallback is currently unclear, define one before comparing current optimizer to the oracle.

Possible fallback options:
- lexicographically maximize goal totals even if minima fail
- maximize number of minima met, then lexicographic totals
- minimize deficit vector first, then lexicographic totals

You need one fixed rule.

---

### 2D. Enumeration method
For tests, recursive depth-first enumeration is fine.

Pseudo-shape:

```rust
fn enumerate_slot_combinations(
    pools: &[Vec<CandidateLike>],
    idx: usize,
    current: &mut Vec<CandidateLike>,
    out: &mut Vec<Vec<CandidateLike>>,
)
```

But you do **not** need to store all combinations.

Better:
- enumerate incrementally
- score each full combination on the fly
- keep only the best-so-far feasible and best-so-far infeasible

This avoids memory blowup even in tests.

### Better pattern
Maintain:

```rust
best_feasible: Option<ScoredBuild>
best_infeasible: Option<ScoredBuild>
```

At each leaf:
- compute totals
- score
- update best

This is closer to what production exact search would also do.

---

### 2E. Comparison representation
Define a compact comparable score:

```rust
struct OracleScore {
    feasible: bool,
    goal_totals: Vec<i64>,
    // optional fallback metadata
}
```

For feasible:
- lexicographic compare over `goal_totals`

For infeasible:
- use your chosen fallback ordering

Also preserve:
- chosen item keys / names
- totals by stat

So failing tests can print a real counterexample.

---

### 2F. First deterministic oracle-backed tests
Before fuzzing, add hand-written tests:

#### Test 1 — corrected 10/10 hypothetical
Two slots:
- A12/B0
- A5/B5
- A5/B5

Goals:
- A10
- B10

Expected:
- feasible
- picks A5/B5 + A5/B5, not the two A12/B0 items

#### Test 2 — no-feasible-but-phase1-compatible
Use the valid three-stat structure from prior audit / Claude’s accepted Counterexample 1.

Expected:
- oracle says infeasible
- current optimizer may still enter viable path
- compare final output behavior carefully

#### Test 3 — paired-slot singleton cannot self-pair
Already mostly covered, but useful oracle cross-check.

#### Test 4 — two distinct same-name instances can pair
Same.

#### Test 5 — lower-priority minimum requires sacrificing excess primary stat
A deliberately small case where:
- maximizing stat1 too hard causes stat2 miss
- exact solver must sacrifice stat1 surplus to meet stat2 minimum

This is probably the most important hand-built test after the 10/10 hypothetical.

---

## 3. Fuzz comparison harness

### Goal
Automatically generate many small legal test cases and compare:
- current optimizer
- brute-force oracle

This is how you find a **real** counterexample against the actual code.

---

### 3A. What to fuzz
Generate small random cases within the supported input contract.

Keep them tiny enough that exhaustive oracle is trivial.

Suggested bounds:
- 2 to 4 slot families total
- 1 to 4 candidates per slot/family
- 1 to 3 goal stats
- stat values in a small range, e.g. `0..15`
- include paired families sometimes, not always

This is plenty to shake out logic bugs.

---

### 3B. Fuzz structure
For each generated case:

1. build candidate items
2. run current optimizer
3. run oracle
4. compare:
   - feasibility
   - chosen lexicographic goal totals
   - possibly chosen item set if ties are deterministic

### Important comparison rule
Do **not** require exact item identity match if multiple builds are equally optimal.

Compare primarily:
- feasibility classification
- final goal-total vector
- fallback score if infeasible

That avoids false failures on tie-equivalent builds.

---

### 3C. Number of cases
Initial target:
- 1,000 random cases in one ignored test
or
- 100 small cases in a normal test
- larger run as ignored/manual

Suggested split:
- fast smoke fuzz: 100–250 cases in ordinary test
- deep fuzz: 5,000+ cases under `#[ignore]`

---

### 3D. Failure reporting
When a mismatch occurs, print:
- goals
- slot families
- all candidates
- optimizer result
- oracle result

And ideally emit a copy-pasteable fixture format.

Best possible outcome:
- a fuzz test discovers the first true counterexample
- then you convert it into a permanent deterministic regression test

---

### 3E. Optional property-testing crate
You can do this manually with a deterministic RNG, or with a property-testing library.

#### Simpler first version
Use:
- deterministic seeded RNG
- plain loop in a test

Advantages:
- no extra framework
- easier to control and print

That is what I’d recommend first.

---

# Suggested implementation order

## Phase 1 — guard cleanup
1. Replace truncation in Rust with overflow detection + refusal
2. Add clear error type/message
3. Later add matching Lua-side refusal before export

## Phase 2 — oracle
4. Build exact exhaustive oracle in tests
5. Add deterministic toy tests:
   - corrected 10/10 case
   - valid no-feasible-but-compatible case
   - paired-slot rules
   - primary-stat sacrifice case

## Phase 3 — fuzz
6. Add seeded random small-case generator
7. Compare optimizer vs oracle on hundreds/thousands of legal inputs
8. Promote first found mismatch into fixed regression test

## Phase 4 — decision
9. If mismatches appear:
   - current algorithm is concretely disproven
   - replace narrowing/search core
10. If no mismatches appear after substantial fuzzing:
   - the search may be more sound than feared
   - remaining issues may be narrower than a full redesign

---

# My recommendation on architecture
If fuzzing finds real mismatches, my likely next recommendation would be:

- keep:
  - item parsing
  - slot grouping
  - pair construction
  - report formatting
- replace only:
  - the actual search/narrowing core

That is much smaller than “rewrite the optimizer.”

If you want, next I can turn this into a **developer-ready task checklist by file**, e.g.:
- `src/optimizer.rs`
- `src/main.rs`
- `src/lgo.lua`
- tests to add and in what order.


  . Use GPT-5.5 for the new session
  . Start with the prompt above
  . Attach the repo
  . If possible, also mention the old JS optimizer exists as behavioral context, but don’t make it central unless the new session asks for it