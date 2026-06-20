//! Gear set optimizer.
//!
//! Model guidance: high borrow-checker/algorithmic friction — see `docs/MODEL_GUIDANCE.md` before non-trivial edits.
//!
//! ## Correctness of the per-slot greedy approach
//!
//! Because gear stats are strictly additive across independent slots, the
//! total of any stat S across a gear set is:
//!
//!     total(S) = Σ item[slot].stat(S)   for all slots
//!
//! This means the global maximum of total(S) is achieved by independently
//! maximising item[slot].stat(S) in every slot. The slots do not interact.
//!
//! ## Feasibility
//!
//! A result is *feasible* if every goal stat's total across all slots meets
//! its user-supplied minimum.
//!
//! We use a two-phase approach:
//!
//! Phase 1 — compatibility filtering:
//!   For each slot K, a candidate C is *compatible* if it cannot prevent any
//!   minimum from being met, i.e. for every goal stat S:
//!
//!     C.stat(S) + best_of_other_slots(S, K) >= minimum(S)
//!
//!   Candidates failing this test are dropped. If any slot becomes empty,
//!   no feasible solution exists and we fall through to Phase 2.
//!
//! Phase 1 narrowing — safe lexicographic narrowing:
//!   For each goal stat S in reverse priority order, for each slot K, find the
//!   highest threshold T such that retaining only candidates with stat(S) >= T
//!   still allows all minima to be met (i.e. the resulting global maximum for
//!   every goal stat still reaches its minimum). Processing in reverse priority
//!   order ensures lower-priority stats are maximised first, so higher-priority
//!   stats narrow last and do not eliminate candidates needed by lower-priority
//!   minima.
//!
//! Phase 2 — infeasible fallback:
//!   Run standard greedy lexicographic narrowing on the full unfiltered pools.
//!   Reports which minima were missed.
//!
//! ## Paired slots (Wrist, Finger, Ear)
//!
//! Items for a paired slot type are combined into super-candidates whose stats
//! are the sum of both items. All unordered pairs (including same-item pairs, to
//! support two items with the same name) are enumerated.

use std::collections::HashMap;

use crate::gear::{CachedItem, GearItem, GearSet, Slot};
use crate::stat::{Stat, StatGoal};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum candidates considered per slot. Excess items are dropped with a
/// warning. Keeps the paired-slot enumeration bounded (max 8×9/2 = 36 pairs).
pub const MAX_CANDIDATES_PER_SLOT: usize = 8;

// ── Public types ──────────────────────────────────────────────────────────────

/// The result returned by the optimizer.
#[derive(Debug)]
pub struct OptimizeResult {
    pub gear_set: GearSet,
    pub feasible: bool,
    /// For each goal stat that failed its minimum: (stat, minimum, achieved).
    pub failed_minima: Vec<(Stat, i64, i64)>,
    /// Warning messages (e.g. candidate pool truncation).
    pub warnings: Vec<String>,
}

// ── Internal types ────────────────────────────────────────────────────────────

/// A resolved item ready for the optimizer: name + stats only.
#[derive(Debug, Clone)]
struct Candidate {
    name: String,
    stats: HashMap<Stat, i64>,
    original_slot: Slot,
}

impl Candidate {
    fn stat(&self, s: &Stat) -> i64 {
        self.stats.get(s).copied().unwrap_or(0)
    }

    fn zero(name: impl Into<String>, slot: Slot) -> Self {
        Candidate {
            name: name.into(),
            stats: HashMap::new(),
            original_slot: slot,
        }
    }
}

/// A "super-candidate" for a paired slot: holds the two constituent items
/// and their combined stats.
#[derive(Debug, Clone)]
struct PairCandidate {
    a: Candidate,
    b: Candidate,
    combined: HashMap<Stat, i64>,
}

impl PairCandidate {
    fn new(a: Candidate, b: Candidate) -> Self {
        let mut combined: HashMap<Stat, i64> = a.stats.clone();
        for (s, v) in &b.stats {
            *combined.entry(*s).or_insert(0) += v;
        }
        PairCandidate { a, b, combined }
    }

    fn stat(&self, s: &Stat) -> i64 {
        self.combined.get(s).copied().unwrap_or(0)
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn optimize(
    resolved: &HashMap<String, CachedItem>,
    candidates: &[String],
    goals: &[StatGoal],
) -> OptimizeResult {
    let mut warnings: Vec<String> = Vec::new();

    // ── 1. Build per-slot candidate pools ─────────────────────────────────────

    let all_names: Vec<String> = candidates.to_vec();

    let mut pools: HashMap<Slot, Vec<Candidate>> = HashMap::new();

    for name in &all_names {
        let cached = match resolved.get(name) {
            Some(c) => c,
            None => continue,
        };
        let canonical = canonical_slot(cached.slot);
        let cand = Candidate {
            name: cached.name.clone(),
            stats: cached.stats.clone(),
            original_slot: cached.slot,
        };
        pools.entry(canonical).or_default().push(cand);
    }

    // Enforce per-slot candidate limit.
    for (slot, pool) in pools.iter_mut() {
        if pool.len() > MAX_CANDIDATES_PER_SLOT {
            warnings.push(format!(
                "Slot {}: {} candidates found; only the first {} will be considered.",
                slot_display(*slot),
                pool.len(),
                MAX_CANDIDATES_PER_SLOT,
            ));
            pool.truncate(MAX_CANDIDATES_PER_SLOT);
        }
    }

    // Ensure every slot has a pool entry (zero placeholder if needed).
    for &slot in Slot::ALL {
        let canonical = canonical_slot(slot);
        pools.entry(canonical).or_insert_with(|| {
            warnings.push(format!(
                "Slot {}: no candidates found; using zero placeholder.",
                slot_display(slot)
            ));
            vec![Candidate::zero(
                format!("[empty {}]", slot_display(slot)),
                canonical,
            )]
        });
    }

    // ── 2. Build super-candidates for paired slots ────────────────────────────

    let paired_canonicals = [Slot::Wrist1, Slot::Finger1, Slot::Ear1];

    let mut single_pools: HashMap<Slot, Vec<Candidate>> = HashMap::new();
    let mut pair_pools: HashMap<Slot, Vec<PairCandidate>> = HashMap::new();

    for (&slot, pool) in &pools {
        if paired_canonicals.contains(&slot) {
            pair_pools.insert(slot, build_pairs(pool, slot, paired_slot2(slot)));
        } else {
            single_pools.insert(slot, pool.clone());
        }
    }

    // ── 3. Compute per-slot, per-stat maxima ──────────────────────────────────

    let single_slot_maxima = compute_single_maxima(&single_pools, goals);
    let pair_slot_maxima = compute_pair_maxima(&pair_pools, goals);
    let global_max = compute_global_max(&single_slot_maxima, &pair_slot_maxima, goals);

    // ── 4. Phase 1: filter to feasibility-compatible candidates ──────────────

    let mut feasible_single =
        filter_compatible_single(&single_pools, &single_slot_maxima, &global_max, goals);
    let mut feasible_pair =
        filter_compatible_pair(&pair_pools, &pair_slot_maxima, &global_max, goals);

    let phase1_viable = feasible_single.values().all(|p| !p.is_empty())
        && feasible_pair.values().all(|p| !p.is_empty());

    // ── 5. Choose working pools ───────────────────────────────────────────────

    let mut fallback_single;
    let mut fallback_pair;

    let (working_single, working_pair): (
        &mut HashMap<Slot, Vec<Candidate>>,
        &mut HashMap<Slot, Vec<PairCandidate>>,
    ) = if phase1_viable {
        (&mut feasible_single, &mut feasible_pair)
    } else {
        fallback_single = single_pools.clone();
        fallback_pair = pair_pools.clone();
        (&mut fallback_single, &mut fallback_pair)
    };

    // ── 6. Lexicographic narrowing ────────────────────────────────────────────
    //
    // Feasible path: safe narrowing in reverse priority order — maximises
    // lower-priority stats first, then narrows higher-priority stats last,
    // while never making any minimum unreachable.
    //
    // Infeasible path: standard greedy narrowing in forward priority order —
    // minima cannot be met anyway, so we maximise stats in the user's stated
    // priority order.

    if phase1_viable {
        for idx in (0..goals.len()).rev() {
            safe_narrow_single(working_single, working_pair, idx, goals);
            safe_narrow_pair(working_pair, working_single, idx, goals);
        }
    } else {
        for goal in goals {
            narrow_single(working_single, &goal.stat);
            narrow_pair(working_pair, &goal.stat);
        }
    }

    // ── 7. Assemble the final GearSet ─────────────────────────────────────────

    let mut gear_set = GearSet::new();

    for (slot, pool) in working_single.iter() {
        let chosen = pool
            .first()
            .expect("pool must not be empty after narrowing");
        gear_set
            .items
            .insert(*slot, candidate_to_gear_item(chosen, *slot));
    }

    for (canonical, pairs) in working_pair.iter() {
        let chosen = pairs.first().expect("pair pool must not be empty");
        let slot1 = *canonical;
        let slot2 = paired_slot2(slot1);

        let (item_for_slot1, item_for_slot2) =
            if chosen.a.original_slot == slot1 && chosen.b.original_slot == slot2 {
                (&chosen.a, &chosen.b)
            } else if chosen.a.original_slot == slot2 && chosen.b.original_slot == slot1 {
                (&chosen.b, &chosen.a)
            } else {
                (&chosen.a, &chosen.b)
            };

        gear_set
            .items
            .insert(slot1, candidate_to_gear_item(item_for_slot1, slot1));
        gear_set
            .items
            .insert(slot2, candidate_to_gear_item(item_for_slot2, slot2));
    }

    // ── 8. Compute actual feasibility from achieved totals ────────────────────

    let failed_minima: Vec<(Stat, i64, i64)> = goals
        .iter()
        .filter(|g| g.minimum > 0)
        .filter_map(|g| {
            let achieved = gear_set.total(&g.stat);
            if achieved < g.minimum {
                Some((g.stat, g.minimum, achieved))
            } else {
                None
            }
        })
        .collect();

    let feasible = failed_minima.is_empty();

    OptimizeResult {
        gear_set,
        feasible,
        failed_minima,
        warnings,
    }
}

// ── Feasibility filtering ─────────────────────────────────────────────────────

type SlotMaxima = HashMap<Slot, HashMap<Stat, i64>>;

fn compute_single_maxima(pools: &HashMap<Slot, Vec<Candidate>>, goals: &[StatGoal]) -> SlotMaxima {
    let mut out: SlotMaxima = HashMap::new();
    for (&slot, pool) in pools {
        let mut stat_max: HashMap<Stat, i64> = HashMap::new();
        for goal in goals {
            let best = pool.iter().map(|c| c.stat(&goal.stat)).max().unwrap_or(0);
            stat_max.insert(goal.stat, best);
        }
        out.insert(slot, stat_max);
    }
    out
}

fn compute_pair_maxima(
    pools: &HashMap<Slot, Vec<PairCandidate>>,
    goals: &[StatGoal],
) -> SlotMaxima {
    let mut out: SlotMaxima = HashMap::new();
    for (&slot, pool) in pools {
        let mut stat_max: HashMap<Stat, i64> = HashMap::new();
        for goal in goals {
            let best = pool.iter().map(|p| p.stat(&goal.stat)).max().unwrap_or(0);
            stat_max.insert(goal.stat, best);
        }
        out.insert(slot, stat_max);
    }
    out
}

fn compute_global_max(
    single: &SlotMaxima,
    pair: &SlotMaxima,
    goals: &[StatGoal],
) -> HashMap<Stat, i64> {
    let mut out: HashMap<Stat, i64> = HashMap::new();
    for goal in goals {
        let mut total = 0i64;
        for stat_max in single.values() {
            total += stat_max.get(&goal.stat).copied().unwrap_or(0);
        }
        for stat_max in pair.values() {
            total += stat_max.get(&goal.stat).copied().unwrap_or(0);
        }
        out.insert(goal.stat, total);
    }
    out
}

fn filter_compatible_single(
    pools: &HashMap<Slot, Vec<Candidate>>,
    slot_maxima: &SlotMaxima,
    global_max: &HashMap<Stat, i64>,
    goals: &[StatGoal],
) -> HashMap<Slot, Vec<Candidate>> {
    let mut out: HashMap<Slot, Vec<Candidate>> = HashMap::new();
    for (&slot, pool) in pools {
        let this_slot_max = slot_maxima.get(&slot);
        let filtered: Vec<Candidate> = pool
            .iter()
            .filter(|c| {
                goals.iter().all(|g| {
                    if g.minimum == 0 {
                        return true;
                    }
                    let slot_best = this_slot_max
                        .and_then(|m| m.get(&g.stat))
                        .copied()
                        .unwrap_or(0);
                    let global_best = global_max.get(&g.stat).copied().unwrap_or(0);
                    c.stat(&g.stat) + (global_best - slot_best) >= g.minimum
                })
            })
            .cloned()
            .collect();
        out.insert(slot, filtered);
    }
    out
}

fn filter_compatible_pair(
    pools: &HashMap<Slot, Vec<PairCandidate>>,
    slot_maxima: &SlotMaxima,
    global_max: &HashMap<Stat, i64>,
    goals: &[StatGoal],
) -> HashMap<Slot, Vec<PairCandidate>> {
    let mut out: HashMap<Slot, Vec<PairCandidate>> = HashMap::new();
    for (&slot, pool) in pools {
        let this_slot_max = slot_maxima.get(&slot);
        let filtered: Vec<PairCandidate> = pool
            .iter()
            .filter(|p| {
                goals.iter().all(|g| {
                    if g.minimum == 0 {
                        return true;
                    }
                    let slot_best = this_slot_max
                        .and_then(|m| m.get(&g.stat))
                        .copied()
                        .unwrap_or(0);
                    let global_best = global_max.get(&g.stat).copied().unwrap_or(0);
                    p.stat(&g.stat) + (global_best - slot_best) >= g.minimum
                })
            })
            .cloned()
            .collect();
        out.insert(slot, filtered);
    }
    out
}

// ── Safe lexicographic narrowing (feasible path) ──────────────────────────────

/// Narrow single-slot pools on goals[goal_index].stat without breaking
/// feasibility, and without reducing the achievable global maximum of any
/// higher-priority stat (goals[..goal_index]).
///
/// For each slot, the highest threshold T is chosen such that retaining only
/// candidates with stat >= T:
///   (a) keeps every minimum reachable, AND
///   (b) does not reduce the per-slot best of any higher-priority stat.
///
/// Recomputes global maxima after each individual slot's narrowing so that
/// later slots in the same pass see the updated state.
fn safe_narrow_single(
    single_pools: &mut HashMap<Slot, Vec<Candidate>>,
    pair_pools: &HashMap<Slot, Vec<PairCandidate>>,
    goal_index: usize,
    goals: &[StatGoal],
) {
    let stat = &goals[goal_index].stat;
    let higher_priority = &goals[..goal_index];

    for &slot in Slot::ALL {
        if !single_pools.contains_key(&slot) {
            continue;
        }
        // Recompute after each slot's narrowing so later slots see updated maxima.
        let single_maxima = compute_single_maxima(single_pools, goals);
        let pair_maxima = compute_pair_maxima(pair_pools, goals);
        let global_max = compute_global_max(&single_maxima, &pair_maxima, goals);
        let current_slot_max = single_maxima.get(&slot).cloned();

        let pool = single_pools.get(&slot).unwrap();
        if pool.is_empty() {
            continue;
        }

        // Distinct values of `stat` in descending order.
        let mut thresholds: Vec<i64> = pool.iter().map(|c| c.stat(stat)).collect();
        thresholds.sort_unstable_by(|a, b| b.cmp(a));
        thresholds.dedup();

        // Find the highest threshold that satisfies both checks.
        let chosen = thresholds.iter().copied().find(|&t| {
            let tentative: Vec<&Candidate> = pool.iter().filter(|c| c.stat(stat) >= t).collect();

            // (a) All minima remain reachable.
            let minima_ok = goals.iter().all(|g| {
                if g.minimum == 0 {
                    return true;
                }
                let old_best = current_slot_max
                    .as_ref()
                    .and_then(|m| m.get(&g.stat))
                    .copied()
                    .unwrap_or(0);
                let new_best = tentative.iter().map(|c| c.stat(&g.stat)).max().unwrap_or(0);
                let new_global =
                    global_max.get(&g.stat).copied().unwrap_or(0) - old_best + new_best;
                new_global >= g.minimum
            });
            if !minima_ok {
                return false;
            }

            // (b) Per-slot best of every higher-priority stat is preserved.
            higher_priority.iter().all(|g| {
                let old_best = current_slot_max
                    .as_ref()
                    .and_then(|m| m.get(&g.stat))
                    .copied()
                    .unwrap_or(0);
                let new_best = tentative.iter().map(|c| c.stat(&g.stat)).max().unwrap_or(0);
                new_best >= old_best
            })
        });

        if let Some(t) = chosen {
            single_pools
                .get_mut(&slot)
                .unwrap()
                .retain(|c| c.stat(stat) >= t);
        }
    }
}

/// Narrow pair-slot pools on goals[goal_index].stat without breaking
/// feasibility, and without reducing the achievable global maximum of any
/// higher-priority stat (goals[..goal_index]).
/// Same logic as `safe_narrow_single`.
fn safe_narrow_pair(
    pair_pools: &mut HashMap<Slot, Vec<PairCandidate>>,
    single_pools: &HashMap<Slot, Vec<Candidate>>,
    goal_index: usize,
    goals: &[StatGoal],
) {
    let stat = &goals[goal_index].stat;
    let higher_priority = &goals[..goal_index];

    for &slot in Slot::ALL {
        if !pair_pools.contains_key(&slot) {
            continue;
        }
        // Recompute after each slot's narrowing so later slots see updated maxima.
        let single_maxima = compute_single_maxima(single_pools, goals);
        let pair_maxima = compute_pair_maxima(pair_pools, goals);
        let global_max = compute_global_max(&single_maxima, &pair_maxima, goals);
        let current_slot_max = pair_maxima.get(&slot).cloned();

        let pool = pair_pools.get(&slot).unwrap();
        if pool.is_empty() {
            continue;
        }

        let mut thresholds: Vec<i64> = pool.iter().map(|p| p.stat(stat)).collect();
        thresholds.sort_unstable_by(|a, b| b.cmp(a));
        thresholds.dedup();

        let chosen = thresholds.iter().copied().find(|&t| {
            let tentative: Vec<&PairCandidate> =
                pool.iter().filter(|p| p.stat(stat) >= t).collect();

            // (a) All minima remain reachable.
            let minima_ok = goals.iter().all(|g| {
                if g.minimum == 0 {
                    return true;
                }
                let old_best = current_slot_max
                    .as_ref()
                    .and_then(|m| m.get(&g.stat))
                    .copied()
                    .unwrap_or(0);
                let new_best = tentative.iter().map(|p| p.stat(&g.stat)).max().unwrap_or(0);
                let new_global =
                    global_max.get(&g.stat).copied().unwrap_or(0) - old_best + new_best;
                new_global >= g.minimum
            });
            if !minima_ok {
                return false;
            }

            // (b) Per-slot best of every higher-priority stat is preserved.
            higher_priority.iter().all(|g| {
                let old_best = current_slot_max
                    .as_ref()
                    .and_then(|m| m.get(&g.stat))
                    .copied()
                    .unwrap_or(0);
                let new_best = tentative.iter().map(|p| p.stat(&g.stat)).max().unwrap_or(0);
                new_best >= old_best
            })
        });

        if let Some(t) = chosen {
            pair_pools
                .get_mut(&slot)
                .unwrap()
                .retain(|p| p.stat(stat) >= t);
        }
    }
}

// ── Standard greedy narrowing (infeasible fallback path) ──────────────────────

fn narrow_single(pools: &mut HashMap<Slot, Vec<Candidate>>, stat: &Stat) {
    for pool in pools.values_mut() {
        if pool.is_empty() {
            continue;
        }
        let best = pool.iter().map(|c| c.stat(stat)).max().unwrap_or(0);
        pool.retain(|c| c.stat(stat) >= best);
        debug_assert!(!pool.is_empty());
    }
}

fn narrow_pair(pools: &mut HashMap<Slot, Vec<PairCandidate>>, stat: &Stat) {
    for pool in pools.values_mut() {
        if pool.is_empty() {
            continue;
        }
        let best = pool.iter().map(|p| p.stat(stat)).max().unwrap_or(0);
        pool.retain(|p| p.stat(stat) >= best);
        debug_assert!(!pool.is_empty());
    }
}

// ── Other helpers ─────────────────────────────────────────────────────────────

fn canonical_slot(slot: Slot) -> Slot {
    match slot {
        Slot::Wrist2 => Slot::Wrist1,
        Slot::Finger2 => Slot::Finger1,
        Slot::Ear2 => Slot::Ear1,
        other => other,
    }
}

fn paired_slot2(slot1: Slot) -> Slot {
    match slot1 {
        Slot::Wrist1 => Slot::Wrist2,
        Slot::Finger1 => Slot::Finger2,
        Slot::Ear1 => Slot::Ear2,
        other => other,
    }
}

fn build_pairs(pool: &[Candidate], slot1: Slot, slot2: Slot) -> Vec<PairCandidate> {
    // A single candidate instance may never be assigned to more than one slot.
    // Two distinct instances with the same display name are allowed to occupy
    // both slots of a paired type only because they are separate pool entries.
    // The inner loop starts at j = i+1 (strictly greater) so that (i, i) pairs
    // are never generated — each pair always consists of two distinct instances.
    if pool.is_empty() {
        return vec![PairCandidate::new(
            Candidate::zero("[empty]", slot1),
            Candidate::zero("[empty]", slot2),
        )];
    }
    if pool.len() == 1 {
        // One owned instance: it fills one slot; the other slot is empty.
        return vec![PairCandidate::new(
            pool[0].clone(),
            Candidate::zero("[empty]", slot2),
        )];
    }
    let mut pairs = Vec::new();
    for i in 0..pool.len() {
        for j in (i + 1)..pool.len() {
            pairs.push(PairCandidate::new(pool[i].clone(), pool[j].clone()));
        }
    }
    // Prefer natural slot assignments as a tiebreaker: a→slot1, b→slot2.
    pairs.sort_by_key(|p| {
        if p.a.original_slot == slot1 && p.b.original_slot == slot2 {
            0
        } else {
            1
        }
    });
    pairs
}

fn candidate_to_gear_item(c: &Candidate, slot: Slot) -> GearItem {
    GearItem {
        name: c.name.clone(),
        slot,
        stats: c.stats.clone(),
    }
}

fn slot_display(slot: Slot) -> &'static str {
    match slot {
        Slot::Head => "Head",
        Slot::Chest => "Chest",
        Slot::Legs => "Legs",
        Slot::Hands => "Hands",
        Slot::Feet => "Feet",
        Slot::Shoulders => "Shoulders",
        Slot::Back => "Back",
        Slot::Wrist1 | Slot::Wrist2 => "Wrist",
        Slot::Neck => "Neck",
        Slot::Finger1 | Slot::Finger2 => "Finger",
        Slot::Ear1 | Slot::Ear2 => "Ear",
        Slot::Pocket => "Pocket",
        Slot::MainHand => "Main-hand",
        Slot::OffHand => "Off-hand",
        Slot::Ranged => "Ranged",
        Slot::ClassItem => "Class Item",
    }
}

// ────────── TESTS ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cached(name: &str, slot: Slot, stats: &[(Stat, i64)]) -> CachedItem {
        CachedItem {
            name: name.to_string(),
            slot,
            stats: stats.iter().copied().collect(),
        }
    }

    fn goal(stat: Stat, minimum: i64) -> StatGoal {
        StatGoal { stat, minimum }
    }

    fn single_slot_result(
        resolved: &HashMap<String, CachedItem>,
        names: &[&str],
        goals: Vec<StatGoal>,
        slot: Slot,
    ) -> String {
        let name_strings: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        let result = optimize(resolved, &name_strings, &goals);
        result
            .gear_set
            .items
            .get(&slot)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| "[missing]".to_string())
    }

    #[test]
    fn test_spec_run1_c2_wins() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "C1".into(),
            make_cached(
                "C1",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 480),
                    (Stat::TacticalMastery, 420),
                    (Stat::Finesse, 310),
                    (Stat::TacticalMitigation, 190),
                ],
            ),
        );
        resolved.insert(
            "C2".into(),
            make_cached(
                "C2",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 500),
                    (Stat::TacticalMastery, 450),
                    (Stat::Finesse, 310),
                    (Stat::TacticalMitigation, 200),
                ],
            ),
        );
        resolved.insert(
            "C3".into(),
            make_cached(
                "C3",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 490),
                    (Stat::TacticalMastery, 450),
                    (Stat::Finesse, 310),
                    (Stat::TacticalMitigation, 230),
                ],
            ),
        );
        resolved.insert(
            "C4".into(),
            make_cached(
                "C4",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 520),
                    (Stat::TacticalMastery, 430),
                    (Stat::Finesse, 310),
                    (Stat::TacticalMitigation, 230),
                ],
            ),
        );
        resolved.insert(
            "C5".into(),
            make_cached(
                "C5",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 460),
                    (Stat::TacticalMastery, 450),
                    (Stat::Finesse, 310),
                    (Stat::TacticalMitigation, 230),
                ],
            ),
        );

        let goals = vec![
            goal(Stat::CriticalRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::Finesse, 300),
            goal(Stat::TacticalMitigation, 200),
        ];

        let winner = single_slot_result(
            &resolved,
            &["C1", "C2", "C3", "C4", "C5"],
            goals,
            Slot::Chest,
        );
        assert_eq!(winner, "C2", "Expected C2; got {}", winner);
    }

    #[test]
    fn test_spec_run1_feasible() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "C2".into(),
            make_cached(
                "C2",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 500),
                    (Stat::TacticalMastery, 450),
                    (Stat::Finesse, 310),
                    (Stat::TacticalMitigation, 200),
                ],
            ),
        );
        let goals = vec![
            goal(Stat::CriticalRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::Finesse, 300),
            goal(Stat::TacticalMitigation, 200),
        ];
        let result = optimize(&resolved, &["C2".to_string()], &goals);
        assert!(result.feasible);
        assert!(result.failed_minima.is_empty());
    }

    #[test]
    fn test_spec_run2_c6_wins_infeasible() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "C6".into(),
            make_cached(
                "C6",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 440),
                    (Stat::TacticalMastery, 200),
                    (Stat::Finesse, 200),
                    (Stat::TacticalMitigation, 100),
                ],
            ),
        );
        resolved.insert(
            "C7".into(),
            make_cached(
                "C7",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 400),
                    (Stat::TacticalMastery, 440),
                    (Stat::Finesse, 290),
                    (Stat::TacticalMitigation, 190),
                ],
            ),
        );

        let goals = vec![
            goal(Stat::CriticalRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::Finesse, 300),
            goal(Stat::TacticalMitigation, 200),
        ];

        let name_strings = vec!["C6".to_string(), "C7".to_string()];
        let result = optimize(&resolved, &name_strings, &goals);

        assert!(!result.feasible);
        assert!(!result.failed_minima.is_empty());

        let winner = result
            .gear_set
            .items
            .get(&Slot::Chest)
            .map(|i| i.name.as_str())
            .unwrap_or("[missing]");
        assert_eq!(winner, "C6", "Expected C6; got {}", winner);
    }

    #[test]
    fn test_c5_over_c4_same_slot() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "C4".into(),
            make_cached(
                "C4",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 520),
                    (Stat::TacticalMastery, 430),
                    (Stat::Finesse, 310),
                    (Stat::TacticalMitigation, 230),
                ],
            ),
        );
        resolved.insert(
            "C5".into(),
            make_cached(
                "C5",
                Slot::Chest,
                &[
                    (Stat::CriticalRating, 460),
                    (Stat::TacticalMastery, 450),
                    (Stat::Finesse, 310),
                    (Stat::TacticalMitigation, 230),
                ],
            ),
        );

        let goals = vec![
            goal(Stat::CriticalRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::Finesse, 300),
            goal(Stat::TacticalMitigation, 200),
        ];

        let winner = single_slot_result(&resolved, &["C4", "C5"], goals, Slot::Chest);
        assert_eq!(winner, "C5", "Expected C5; got {}", winner);
    }

    #[test]
    fn test_paired_slots_both_filled_and_summed() {
        // Two distinct candidates: WristA (100) and WristB (80).
        // The only legal pair is (A,B); both wrist slots are filled and
        // stats are summed: 100+80=180.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "WristA".into(),
            make_cached("WristA", Slot::Wrist1, &[(Stat::Vitality, 100)]),
        );
        resolved.insert(
            "WristB".into(),
            make_cached("WristB", Slot::Wrist1, &[(Stat::Vitality, 80)]),
        );

        let goals = vec![goal(Stat::Vitality, 0)];
        let names = vec!["WristA".to_string(), "WristB".to_string()];
        let result = optimize(&resolved, &names, &goals);

        assert!(result.gear_set.items.contains_key(&Slot::Wrist1));
        assert!(result.gear_set.items.contains_key(&Slot::Wrist2));
        // Best distinct pair: (A,B) = 100+80 = 180.
        assert_eq!(result.gear_set.total(&Stat::Vitality), 180);
    }

    #[test]
    fn test_no_goals_returns_first_candidates() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "ItemA".into(),
            make_cached("ItemA", Slot::Head, &[(Stat::Vitality, 50)]),
        );
        let result = optimize(&resolved, &["ItemA".to_string()], &[]);
        assert!(result.feasible);
        assert!(result.failed_minima.is_empty());
    }

    #[test]
    fn test_safe_narrowing_does_not_sacrifice_stat2_for_stat1() {
        // Bad has higher CriticalRating but its low TacticalMitigation would
        // prevent the TM minimum from being met if chosen.
        // Safe narrowing must recognise this and keep Good instead.
        //
        //   Bad:  CR=400, TM=100
        //   Ok:   CR=300, TM=300
        //   Good: CR=300, TM=600
        //
        // Goals: CR≥300, TM≥600  →  Good must win.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Bad".into(),
            make_cached(
                "Bad",
                Slot::Head,
                &[(Stat::CriticalRating, 400), (Stat::TacticalMitigation, 100)],
            ),
        );
        resolved.insert(
            "Ok".into(),
            make_cached(
                "Ok",
                Slot::Head,
                &[(Stat::CriticalRating, 300), (Stat::TacticalMitigation, 300)],
            ),
        );
        resolved.insert(
            "Good".into(),
            make_cached(
                "Good",
                Slot::Head,
                &[(Stat::CriticalRating, 300), (Stat::TacticalMitigation, 600)],
            ),
        );

        let goals = vec![
            goal(Stat::CriticalRating, 300),
            goal(Stat::TacticalMitigation, 600),
        ];

        let winner = single_slot_result(&resolved, &["Bad", "Ok", "Good"], goals, Slot::Head);
        assert_eq!(winner, "Good", "Expected Good; got {}", winner);
    }

    #[test]
    fn test_safe_narrowing_feasibility_flag_correct() {
        // Companion to the above: result must be reported as feasible,
        // not as infeasible with a spurious "all minima met" message.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Bad".into(),
            make_cached(
                "Bad",
                Slot::Head,
                &[(Stat::CriticalRating, 400), (Stat::TacticalMitigation, 100)],
            ),
        );
        resolved.insert(
            "Good".into(),
            make_cached(
                "Good",
                Slot::Head,
                &[(Stat::CriticalRating, 300), (Stat::TacticalMitigation, 600)],
            ),
        );

        let goals = vec![
            goal(Stat::CriticalRating, 300),
            goal(Stat::TacticalMitigation, 600),
        ];

        let result = optimize(&resolved, &["Bad".to_string(), "Good".to_string()], &goals);
        assert!(result.feasible, "Result should be feasible");
        assert!(result.failed_minima.is_empty());
    }

    // ── Instance-per-slot tests ──────────────────────────────────────────────────
    //
    // One owned item instance must never be assigned to more than one slot.
    // Two distinct instances with the same display name are allowed to fill
    // both slots of a paired type because they are separate candidate entries.

    #[test]
    fn test_single_instance_fills_only_one_paired_slot() {
        // Only one wrist item instance exists.  It can legally occupy only one
        // of the two wrist slots; the other slot is left empty (zero stats).
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "WristX".into(),
            make_cached("WristX", Slot::Wrist1, &[(Stat::CriticalRating, 500)]),
        );

        let goals = vec![goal(Stat::CriticalRating, 0)];
        let result = optimize(&resolved, &["WristX".to_string()], &goals);

        assert!(
            result.gear_set.items.contains_key(&Slot::Wrist1),
            "Wrist1 must be filled"
        );
        // The single instance must not also be placed in Wrist2.
        // Wrist2 should either be absent or hold the zero "[empty]" placeholder.
        assert_ne!(
            result.gear_set.items.get(&Slot::Wrist2).map(|i| i.name.as_str()),
            Some("WristX"),
            "WristX must not occupy Wrist2 — one instance cannot fill two slots"
        );
        if let Some(wrist2) = result.gear_set.items.get(&Slot::Wrist2) {
            assert_eq!(
                wrist2.name, "[empty]",
                "Wrist2 should hold the empty placeholder, not a real item"
            );
        }
        // Stats reflect only one copy: 500, not the erroneous doubled 1000.
        assert_eq!(
            result.gear_set.total(&Stat::CriticalRating),
            500,
            "CR must be 500 (one instance, one slot) — not doubled",
        );
    }

    #[test]
    fn test_two_distinct_same_name_instances_fill_both_paired_slots() {
        // Two separate owned instances of the same item name (two copies).
        // Both paired slots should be filled, and stats are summed.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        // Simulate two instances with distinct synthetic keys but same display name,
        // as produced by main.rs's "{idx}::{slot}::{name}" mapping.
        resolved.insert(
            "0000::Wrist (1)::Pristine Bracelet".into(),
            make_cached(
                "Pristine Bracelet",
                Slot::Wrist1,
                &[(Stat::CriticalRating, 500)],
            ),
        );
        resolved.insert(
            "0001::Wrist (1)::Pristine Bracelet".into(),
            make_cached(
                "Pristine Bracelet",
                Slot::Wrist1,
                &[(Stat::CriticalRating, 500)],
            ),
        );

        let goals = vec![goal(Stat::CriticalRating, 0)];
        let names = vec![
            "0000::Wrist (1)::Pristine Bracelet".to_string(),
            "0001::Wrist (1)::Pristine Bracelet".to_string(),
        ];
        let result = optimize(&resolved, &names, &goals);

        assert!(
            result.gear_set.items.contains_key(&Slot::Wrist1),
            "Wrist1 must be filled"
        );
        assert!(
            result.gear_set.items.contains_key(&Slot::Wrist2),
            "Wrist2 must be filled"
        );
        assert_eq!(
            result.gear_set.items[&Slot::Wrist1].name,
            "Pristine Bracelet",
            "Wrist1 should hold Pristine Bracelet"
        );
        assert_eq!(
            result.gear_set.items[&Slot::Wrist2].name,
            "Pristine Bracelet",
            "Wrist2 should hold Pristine Bracelet"
        );
        assert_eq!(
            result.gear_set.total(&Stat::CriticalRating),
            1000,
            "CR must be 500×2=1000 when two separate instances fill both wrist slots",
        );
    }

    #[test]
    fn test_one_instance_per_slot_makes_tight_minimum_infeasible() {
        // RingA: CR=500, RingB: CR=300.
        // Only distinct pairs are legal: (A,B)=800.
        // Minimum CR=900 — no legal pair can meet it.  Must be infeasible.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "RingA".into(),
            make_cached("RingA", Slot::Finger1, &[(Stat::CriticalRating, 500)]),
        );
        resolved.insert(
            "RingB".into(),
            make_cached("RingB", Slot::Finger1, &[(Stat::CriticalRating, 300)]),
        );

        let goals = vec![goal(Stat::CriticalRating, 900)];
        let result = optimize(
            &resolved,
            &["RingA".to_string(), "RingB".to_string()],
            &goals,
        );

        assert!(
            !result.feasible,
            "Best legal pair (A,B)=800 < 900; result must be infeasible"
        );
        let cr_fail = result
            .failed_minima
            .iter()
            .find(|(s, _, _)| *s == Stat::CriticalRating);
        assert!(
            cr_fail.is_some(),
            "CriticalRating must appear in failed_minima"
        );
        // Infeasible greedy picks the best available distinct pair (A,B)=800.
        assert_eq!(
            result.gear_set.total(&Stat::CriticalRating),
            800,
            "Best achievable with two distinct instances is (A,B)=800"
        );
    }

    #[test]
    fn test_two_same_name_instances_meet_tight_minimum() {
        // Two owned copies of RingA (CR=500 each).
        // Together: CR=1000 ≥ 900 — must be feasible.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "0000::Finger (1)::RingA".into(),
            make_cached("RingA", Slot::Finger1, &[(Stat::CriticalRating, 500)]),
        );
        resolved.insert(
            "0001::Finger (1)::RingA".into(),
            make_cached("RingA", Slot::Finger1, &[(Stat::CriticalRating, 500)]),
        );

        let goals = vec![goal(Stat::CriticalRating, 900)];
        let result = optimize(
            &resolved,
            &[
                "0000::Finger (1)::RingA".to_string(),
                "0001::Finger (1)::RingA".to_string(),
            ],
            &goals,
        );

        assert!(
            result.feasible,
            "Two copies of RingA give CR=1000 ≥ 900; must be feasible"
        );
        assert!(result.failed_minima.is_empty());
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 1000);
    }

    #[test]
    fn test_pair_infeasible_when_minimum_exceeds_best_distinct_pair() {
        // RingA: CR=500, RingB: CR=300.
        // Best distinct pair: (A,B)=800 < 801 — infeasible.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "RingA".into(),
            make_cached("RingA", Slot::Finger1, &[(Stat::CriticalRating, 500)]),
        );
        resolved.insert(
            "RingB".into(),
            make_cached("RingB", Slot::Finger1, &[(Stat::CriticalRating, 300)]),
        );

        let goals = vec![goal(Stat::CriticalRating, 801)];
        let result = optimize(
            &resolved,
            &["RingA".to_string(), "RingB".to_string()],
            &goals,
        );

        assert!(
            !result.feasible,
            "Best distinct pair (A,B)=800 < 801; result must be infeasible"
        );
        let cr_fail = result
            .failed_minima
            .iter()
            .find(|(s, _, _)| *s == Stat::CriticalRating);
        assert!(
            cr_fail.is_some(),
            "CriticalRating must appear in failed_minima"
        );
        assert_eq!(
            result.gear_set.total(&Stat::CriticalRating),
            800,
            "Infeasible result should still show the best achievable value (A,B)=800"
        );
    }

    // ── Feasibility boundary ────────────────────────────────────────────────────

    #[test]
    fn test_single_candidate_meets_minimum_exactly() {
        // Stat == minimum exactly: must be feasible (no off-by-one).
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Helm".into(),
            make_cached("Helm", Slot::Head, &[(Stat::TacticalMitigation, 300)]),
        );

        let goals = vec![goal(Stat::TacticalMitigation, 300)];
        let result = optimize(&resolved, &["Helm".to_string()], &goals);

        assert!(result.feasible, "300 ≥ 300 must be feasible");
        assert!(result.failed_minima.is_empty());
    }

    #[test]
    fn test_single_candidate_one_below_minimum() {
        // Stat is one below the minimum: must be infeasible with correct reporting.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Helm".into(),
            make_cached("Helm", Slot::Head, &[(Stat::TacticalMitigation, 299)]),
        );

        let goals = vec![goal(Stat::TacticalMitigation, 300)];
        let result = optimize(&resolved, &["Helm".to_string()], &goals);

        assert!(!result.feasible, "299 < 300 must be infeasible");
        assert!(
            matches!(
                result
                    .failed_minima
                    .iter()
                    .find(|(s, _, _)| *s == Stat::TacticalMitigation),
                Some((_, 300, 299))
            ),
            "failed_minima must report (TacticalMitigation, min=300, achieved=299)"
        );
    }

    // ── Candidate pool truncation ───────────────────────────────────────────────

    #[test]
    fn test_truncation_warning_emitted_for_oversized_pool() {
        // 9 head items — one above the limit of MAX_CANDIDATES_PER_SLOT (8).
        // A truncation warning must be emitted; the result must still be valid.
        // Note: items are added in order Head1…Head9.  Head9 (highest CR=90)
        // is truncated, so Head8 (CR=80) wins — testing that the cap is
        // enforced *and* that the warning message is correct.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        let mut names: Vec<String> = Vec::new();
        for i in 1..=9usize {
            let name = format!("Head{}", i);
            resolved.insert(
                name.clone(),
                make_cached(&name, Slot::Head, &[(Stat::CriticalRating, i as i64 * 10)]),
            );
            names.push(name);
        }

        let goals = vec![goal(Stat::CriticalRating, 0)];
        let result = optimize(&resolved, &names, &goals);

        assert!(
            result.warnings.iter().any(|w| w.contains("9 candidates")),
            "Expected a truncation warning mentioning '9 candidates'; got: {:?}",
            result.warnings,
        );
        // The result must still be a complete, non-panicking gear set.
        assert!(
            result.gear_set.items.contains_key(&Slot::Head),
            "Head slot must be filled even after truncation"
        );
    }

    // ── Placeholder / missing slot ──────────────────────────────────────────────

    #[test]
    fn test_missing_slot_emits_placeholder_warning() {
        // Supply only a Head item; every other slot has no candidates.
        // The optimizer must insert zero placeholders and emit warnings for them.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Helm".into(),
            make_cached("Helm", Slot::Head, &[(Stat::CriticalRating, 100)]),
        );

        let goals = vec![goal(Stat::CriticalRating, 0)];
        let result = optimize(&resolved, &["Helm".to_string()], &goals);

        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("no candidates found")),
            "Expected placeholder warnings for empty slots; got: {:?}",
            result.warnings,
        );
        // The real item must still win for Head.
        assert_eq!(
            result
                .gear_set
                .items
                .get(&Slot::Head)
                .map(|i| i.name.as_str()),
            Some("Helm"),
        );
    }

    #[test]
    fn test_safe_narrowing_preserves_higher_priority_max_when_no_minimum() {
        // ItemA: CR=200, TM=600   → meets TM≥600, better CR
        // ItemB: CR=100, TM=700   → meets TM≥600, worse CR
        //
        // Goals (priority order): CR:0  then  TM:600
        //
        // Both items are feasible.  CR is priority-1; ItemA must win.
        //
        // Bug: safe_narrow processes TM first (reverse order).  With TM min=600,
        // T=700 passes the feasibility check (700≥600), so ItemA is eliminated.
        // CR then picks from {ItemB} only — result is CR=100 instead of 200.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "ItemA".into(),
            make_cached(
                "ItemA",
                Slot::Head,
                &[(Stat::CriticalRating, 200), (Stat::TacticalMitigation, 600)],
            ),
        );
        resolved.insert(
            "ItemB".into(),
            make_cached(
                "ItemB",
                Slot::Head,
                &[(Stat::CriticalRating, 100), (Stat::TacticalMitigation, 700)],
            ),
        );

        let goals = vec![
            goal(Stat::CriticalRating, 0),       // priority 1 — no floor
            goal(Stat::TacticalMitigation, 600), // priority 2 — floor = 600
        ];

        let winner = single_slot_result(&resolved, &["ItemA", "ItemB"], goals, Slot::Head);
        assert_eq!(
            winner, "ItemA",
            "ItemA has higher CR (priority-1) and still meets TM≥600; it must win"
        );
    }

    #[test]
    fn test_safe_narrowing_paired_preserves_higher_priority_max_when_no_minimum() {
        // EarA: CR=500, TM=100.   EarB: CR=100, TM=500.
        // Only distinct pair: (A,B): CR=600, TM=600.
        //
        // Goals (priority order): CR:0  then  TM:600
        //
        // Verifies that safe-narrowing on a secondary stat (TM) does not
        // eliminate the only legal pair (A,B) when it meets the TM floor:
        // CR must be maximised to 600, not degraded by an overly aggressive
        // narrowing pass.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "EarA".into(),
            make_cached(
                "EarA",
                Slot::Ear1,
                &[(Stat::CriticalRating, 500), (Stat::TacticalMitigation, 100)],
            ),
        );
        resolved.insert(
            "EarB".into(),
            make_cached(
                "EarB",
                Slot::Ear1,
                &[(Stat::CriticalRating, 100), (Stat::TacticalMitigation, 500)],
            ),
        );

        let goals = vec![
            goal(Stat::CriticalRating, 0),       // priority 1 — no floor
            goal(Stat::TacticalMitigation, 600), // priority 2 — floor = 600
        ];

        let result = optimize(&resolved, &["EarA".to_string(), "EarB".to_string()], &goals);

        assert!(
            result.feasible,
            "Pair (A,B) gives TM=600 ≥ 600; must be feasible"
        );
        assert_eq!(
            result.gear_set.total(&Stat::CriticalRating),
            600,
            "CR should be 600 (pair A+B), not 200 (pair B+B); (A,B) must win",
        );
        assert_eq!(result.gear_set.total(&Stat::TacticalMitigation), 600);
    }

    // ── Negative stat tests ─────────────────────────────────────────────────────

    #[test]
    fn test_negative_stat_compensated_across_slots() {
        // Head: ItemA has TM=-50 but high CR.
        // Chest: ItemX has TM=300.
        // Combined TM = -50 + 300 = 250 >= 200 — feasible.
        // ItemA must NOT be filtered out just because its individual TM is negative.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "ItemA".into(),
            make_cached(
                "ItemA",
                Slot::Head,
                &[(Stat::CriticalRating, 500), (Stat::TacticalMitigation, -50)],
            ),
        );
        resolved.insert(
            "ItemX".into(),
            make_cached(
                "ItemX",
                Slot::Chest,
                &[(Stat::CriticalRating, 100), (Stat::TacticalMitigation, 300)],
            ),
        );

        let goals = vec![goal(Stat::TacticalMitigation, 200)];
        let result = optimize(
            &resolved,
            &["ItemA".to_string(), "ItemX".to_string()],
            &goals,
        );

        assert!(
            result.feasible,
            "Combined TM = -50+300 = 250 >= 200; must be feasible"
        );
        assert_eq!(result.gear_set.total(&Stat::TacticalMitigation), 250);
        assert_eq!(
            result
                .gear_set
                .items
                .get(&Slot::Head)
                .map(|i| i.name.as_str()),
            Some("ItemA"),
            "ItemA must not be filtered out despite negative TM",
        );
    }

    #[test]
    fn test_negative_stat_causes_infeasibility() {
        // Single Head item with TM=-50.  Minimum TM=1.
        // No other slot can compensate — result must be infeasible,
        // and the achieved value in failed_minima must be negative.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "ItemA".into(),
            make_cached("ItemA", Slot::Head, &[(Stat::TacticalMitigation, -50)]),
        );

        let goals = vec![goal(Stat::TacticalMitigation, 1)];
        let result = optimize(&resolved, &["ItemA".to_string()], &goals);

        assert!(
            !result.feasible,
            "TM=-50 cannot meet minimum=1; must be infeasible"
        );
        assert!(
            matches!(
                result.failed_minima
                      .iter()
                      .find(|(s, _, _)| *s == Stat::TacticalMitigation),
                Some((_, 1, achieved)) if *achieved < 0
            ),
            "failed_minima must report a negative achieved value for TM",
        );
    }

    #[test]
    fn test_negative_stat_on_non_goal_stat_does_not_crash() {
        // ItemA has a negative value on Finesse, which is not a goal stat.
        // The optimizer must not crash and must select ItemA normally.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "ItemA".into(),
            make_cached(
                "ItemA",
                Slot::Head,
                &[(Stat::CriticalRating, 300), (Stat::Finesse, -999)],
            ),
        );

        let goals = vec![goal(Stat::CriticalRating, 300)];
        let result = optimize(&resolved, &["ItemA".to_string()], &goals);

        assert!(result.feasible);
        assert_eq!(
            result
                .gear_set
                .items
                .get(&Slot::Head)
                .map(|i| i.name.as_str()),
            Some("ItemA"),
        );
    }
}
