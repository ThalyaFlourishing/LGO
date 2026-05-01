//! Gear set optimizer.
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
//!   For each goal stat S in priority order, for each slot K, find the
//!   highest threshold T such that retaining only candidates with stat(S) >= T
//!   still allows all minima to be met (i.e. the resulting global maximum for
//!   every goal stat still reaches its minimum). This prevents greedily
//!   maximising stat1 from eliminating the candidates needed to meet stat2.
//!
//! Phase 2 — infeasible fallback:
//!   Run standard greedy lexicographic narrowing on the full unfiltered pools.
//!   Reports which minima were missed.
//!
//! ## Paired slots (Wrist, Finger, Ear)
//!
//! Items for a paired slot type are combined into super-candidates whose stats
//! are the sum of both items. All ordered pairs (including self-pairs, to
//! support two items with the same name) are enumerated.

use std::collections::HashMap;

use crate::cache::CachedItem;
use crate::gear::{GearItem, GearSet, Slot};
use crate::stat::{Stat, StatGoal};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum candidates considered per slot. Excess items are dropped with a
/// warning. Keeps the paired-slot enumeration bounded (max 6×6 = 36 pairs).
pub const MAX_CANDIDATES_PER_SLOT: usize = 6;

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
        Candidate { name: name.into(), stats: HashMap::new(), original_slot: slot }
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
    equipped: &[String],
    candidates: &[String],
    goals: &[StatGoal],
) -> OptimizeResult {
    let mut warnings: Vec<String> = Vec::new();

    // ── 1. Build per-slot candidate pools ─────────────────────────────────────

    let all_names: Vec<String> = {
        let mut v: Vec<String> = equipped.to_vec();
        for n in candidates {
            if !v.contains(n) { v.push(n.clone()); }
        }
        v
    };

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
                slot_display(*slot), pool.len(), MAX_CANDIDATES_PER_SLOT,
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
            vec![Candidate::zero(format!("[empty {}]", slot_display(slot)), canonical)]
        });
    }

    // ── 2. Build super-candidates for paired slots ────────────────────────────

    let paired_canonicals = [Slot::Wrist1, Slot::Finger1, Slot::Ear1];

    let mut single_pools: HashMap<Slot, Vec<Candidate>> = HashMap::new();
    let mut pair_pools:   HashMap<Slot, Vec<PairCandidate>> = HashMap::new();

    for (&slot, pool) in &pools {
        if paired_canonicals.contains(&slot) {
            pair_pools.insert(slot, build_pairs(pool, slot, paired_slot2(slot)));
        } else {
            single_pools.insert(slot, pool.clone());
        }
    }

    // ── 3. Compute per-slot, per-stat maxima ──────────────────────────────────

    let single_slot_maxima = compute_single_maxima(&single_pools, goals);
    let pair_slot_maxima   = compute_pair_maxima(&pair_pools, goals);
    let global_max = compute_global_max(&single_slot_maxima, &pair_slot_maxima, goals);

    // ── 4. Phase 1: filter to feasibility-compatible candidates ──────────────

    let mut feasible_single = filter_compatible_single(
        &single_pools, &single_slot_maxima, &global_max, goals,
    );
    let mut feasible_pair = filter_compatible_pair(
        &pair_pools, &pair_slot_maxima, &global_max, goals,
    );

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
        fallback_pair   = pair_pools.clone();
        (&mut fallback_single, &mut fallback_pair)
    };

    // ── 6. Lexicographic narrowing ────────────────────────────────────────────
    //
    // Feasible path: safe narrowing — never drop candidates if doing so would
    // make any minimum unreachable.
    //
    // Infeasible path: standard greedy narrowing — minima cannot be met anyway,
    // so we simply maximise stats in priority order.

    for goal in goals {
        if phase1_viable {
            safe_narrow_single(working_single, working_pair, &goal.stat, goals);
            safe_narrow_pair(working_pair, working_single, &goal.stat, goals);
        } else {
            narrow_single(working_single, &goal.stat);
            narrow_pair(working_pair, &goal.stat);
        }
    }

    // ── 7. Assemble the final GearSet ─────────────────────────────────────────

    let mut gear_set = GearSet::new();

    for (slot, pool) in working_single.iter() {
        let chosen = pool.first().expect("pool must not be empty after narrowing");
        gear_set.items.insert(*slot, candidate_to_gear_item(chosen, *slot));
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

        gear_set.items.insert(slot1, candidate_to_gear_item(item_for_slot1, slot1));
        gear_set.items.insert(slot2, candidate_to_gear_item(item_for_slot2, slot2));
    }

    // ── 8. Compute actual feasibility from achieved totals ────────────────────

    let failed_minima: Vec<(Stat, i64, i64)> = goals.iter()
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

    OptimizeResult { gear_set, feasible, failed_minima, warnings }
}

// ── Feasibility filtering ─────────────────────────────────────────────────────

type SlotMaxima = HashMap<Slot, HashMap<Stat, i64>>;

fn compute_single_maxima(
    pools: &HashMap<Slot, Vec<Candidate>>,
    goals: &[StatGoal],
) -> SlotMaxima {
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
        let filtered: Vec<Candidate> = pool.iter().filter(|c| {
            goals.iter().all(|g| {
                if g.minimum == 0 { return true; }
                let slot_best = this_slot_max
                    .and_then(|m| m.get(&g.stat)).copied().unwrap_or(0);
                let global_best = global_max.get(&g.stat).copied().unwrap_or(0);
                c.stat(&g.stat) + (global_best - slot_best) >= g.minimum
            })
        }).cloned().collect();
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
        let filtered: Vec<PairCandidate> = pool.iter().filter(|p| {
            goals.iter().all(|g| {
                if g.minimum == 0 { return true; }
                let slot_best = this_slot_max
                    .and_then(|m| m.get(&g.stat)).copied().unwrap_or(0);
                let global_best = global_max.get(&g.stat).copied().unwrap_or(0);
                p.stat(&g.stat) + (global_best - slot_best) >= g.minimum
            })
        }).cloned().collect();
        out.insert(slot, filtered);
    }
    out
}

// ── Safe lexicographic narrowing (feasible path) ──────────────────────────────

/// Narrow single-slot pools on `stat` without breaking feasibility.
///
/// For each slot in canonical order, recomputes the global max from the
/// current state of all pools (reflecting any narrowing already done this
/// round), then finds the highest threshold T for `stat` that keeps all
/// minima reachable.
fn safe_narrow_single(
    single_pools: &mut HashMap<Slot, Vec<Candidate>>,
    pair_pools:   &HashMap<Slot, Vec<PairCandidate>>,
    stat: &Stat,
    goals: &[StatGoal],
) {
    for &slot in Slot::ALL {
        if !single_pools.contains_key(&slot) { continue; }
        // Recompute after each slot's narrowing so later slots see updated maxima.
        let single_maxima = compute_single_maxima(single_pools, goals);
        let pair_maxima   = compute_pair_maxima(pair_pools, goals);
        let global_max    = compute_global_max(&single_maxima, &pair_maxima, goals);
        let current_slot_max = single_maxima.get(&slot).cloned();

        let pool = single_pools.get(&slot).unwrap();
        if pool.is_empty() { continue; }

        // Distinct values of `stat` in descending order.
        let mut thresholds: Vec<i64> = pool.iter().map(|c| c.stat(stat)).collect();
        thresholds.sort_unstable_by(|a, b| b.cmp(a));
        thresholds.dedup();

        // Find the highest threshold that keeps all minima reachable.
        let chosen = thresholds.iter().copied().find(|&t| {
            let tentative: Vec<&Candidate> = pool.iter()
                .filter(|c| c.stat(stat) >= t)
                .collect();

            goals.iter().all(|g| {
                if g.minimum == 0 { return true; }
                let old_best = current_slot_max.as_ref()
                    .and_then(|m| m.get(&g.stat)).copied().unwrap_or(0);
                let new_best = tentative.iter()
                    .map(|c| c.stat(&g.stat)).max().unwrap_or(0);
                let new_global = global_max.get(&g.stat).copied().unwrap_or(0)
                    - old_best + new_best;
                new_global >= g.minimum
            })
        });

        if let Some(t) = chosen {
            single_pools.get_mut(&slot).unwrap().retain(|c| c.stat(stat) >= t);
        }
    }
}

/// Narrow pair-slot pools on `stat` without breaking feasibility.
/// Same logic as `safe_narrow_single`.
fn safe_narrow_pair(
    pair_pools:   &mut HashMap<Slot, Vec<PairCandidate>>,
    single_pools: &HashMap<Slot, Vec<Candidate>>,
    stat: &Stat,
    goals: &[StatGoal],
) {
    for &slot in Slot::ALL {
        if !pair_pools.contains_key(&slot) { continue; }
        // Recompute after each slot's narrowing so later slots see updated maxima.
        let single_maxima = compute_single_maxima(single_pools, goals);
        let pair_maxima   = compute_pair_maxima(pair_pools, goals);
        let global_max    = compute_global_max(&single_maxima, &pair_maxima, goals);
        let current_slot_max = pair_maxima.get(&slot).cloned();

        let pool = pair_pools.get(&slot).unwrap();
        if pool.is_empty() { continue; }

        let mut thresholds: Vec<i64> = pool.iter().map(|p| p.stat(stat)).collect();
        thresholds.sort_unstable_by(|a, b| b.cmp(a));
        thresholds.dedup();

        let chosen = thresholds.iter().copied().find(|&t| {
            let tentative: Vec<&PairCandidate> = pool.iter()
                .filter(|p| p.stat(stat) >= t)
                .collect();

            goals.iter().all(|g| {
                if g.minimum == 0 { return true; }
                let old_best = current_slot_max.as_ref()
                    .and_then(|m| m.get(&g.stat)).copied().unwrap_or(0);
                let new_best = tentative.iter()
                    .map(|p| p.stat(&g.stat)).max().unwrap_or(0);
                let new_global = global_max.get(&g.stat).copied().unwrap_or(0)
                    - old_best + new_best;
                new_global >= g.minimum
            })
        });

        if let Some(t) = chosen {
            pair_pools.get_mut(&slot).unwrap().retain(|p| p.stat(stat) >= t);
        }
    }
}

// ── Standard greedy narrowing (infeasible fallback path) ──────────────────────

fn narrow_single(pools: &mut HashMap<Slot, Vec<Candidate>>, stat: &Stat) {
    for pool in pools.values_mut() {
        if pool.is_empty() { continue; }
        let best = pool.iter().map(|c| c.stat(stat)).max().unwrap_or(0);
        pool.retain(|c| c.stat(stat) >= best);
        debug_assert!(!pool.is_empty());
    }
}

fn narrow_pair(pools: &mut HashMap<Slot, Vec<PairCandidate>>, stat: &Stat) {
    for pool in pools.values_mut() {
        if pool.is_empty() { continue; }
        let best = pool.iter().map(|p| p.stat(stat)).max().unwrap_or(0);
        pool.retain(|p| p.stat(stat) >= best);
        debug_assert!(!pool.is_empty());
    }
}

// ── Other helpers ─────────────────────────────────────────────────────────────

fn canonical_slot(slot: Slot) -> Slot {
    match slot {
        Slot::Wrist2  => Slot::Wrist1,
        Slot::Finger2 => Slot::Finger1,
        Slot::Ear2    => Slot::Ear1,
        other         => other,
    }
}

fn paired_slot2(slot1: Slot) -> Slot {
    match slot1 {
        Slot::Wrist1  => Slot::Wrist2,
        Slot::Finger1 => Slot::Finger2,
        Slot::Ear1    => Slot::Ear2,
        other         => other,
    }
}

fn build_pairs(pool: &[Candidate], slot1: Slot, slot2: Slot) -> Vec<PairCandidate> {
    if pool.is_empty() {
        return vec![PairCandidate::new(
            Candidate::zero("[empty]", slot1),
            Candidate::zero("[empty]", slot2),
        )];
    }
    let mut pairs = Vec::new();
    for i in 0..pool.len() {
        for j in 0..pool.len() {
            pairs.push(PairCandidate::new(pool[i].clone(), pool[j].clone()));
        }
    }
    // Prefer natural slot assignments as a tiebreaker: a→slot1, b→slot2.
    pairs.sort_by_key(|p| {
        if p.a.original_slot == slot1 && p.b.original_slot == slot2 { 0 } else { 1 }
    });
    pairs
}

fn candidate_to_gear_item(c: &Candidate, slot: Slot) -> GearItem {
    GearItem { name: c.name.clone(), slot, stats: c.stats.clone() }
}

fn slot_display(slot: Slot) -> &'static str {
    match slot {
        Slot::Head      => "Head",
        Slot::Chest     => "Chest",
        Slot::Legs      => "Legs",
        Slot::Hands     => "Hands",
        Slot::Feet      => "Feet",
        Slot::Shoulders => "Shoulders",
        Slot::Back      => "Back",
        Slot::Wrist1 | Slot::Wrist2   => "Wrist",
        Slot::Neck      => "Neck",
        Slot::Finger1 | Slot::Finger2 => "Finger",
        Slot::Ear1 | Slot::Ear2       => "Ear",
        Slot::Pocket    => "Pocket",
        Slot::MainHand  => "Main-hand",
        Slot::OffHand   => "Off-hand",
        Slot::Ranged    => "Ranged",
        Slot::ClassItem => "Class Item",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

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
        let result = optimize(resolved, &[], &name_strings, &goals);
        result.gear_set.items.get(&slot)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| "[missing]".to_string())
    }

    #[test]
    fn test_spec_run1_c2_wins() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert("C1".into(), make_cached("C1", Slot::Chest, &[
            (Stat::CriticalRating, 480), (Stat::TacticalMastery, 420),
            (Stat::Finesse, 310), (Stat::TacticalMitigation, 190),
        ]));
        resolved.insert("C2".into(), make_cached("C2", Slot::Chest, &[
            (Stat::CriticalRating, 500), (Stat::TacticalMastery, 450),
            (Stat::Finesse, 310), (Stat::TacticalMitigation, 200),
        ]));
        resolved.insert("C3".into(), make_cached("C3", Slot::Chest, &[
            (Stat::CriticalRating, 490), (Stat::TacticalMastery, 450),
            (Stat::Finesse, 310), (Stat::TacticalMitigation, 230),
        ]));
        resolved.insert("C4".into(), make_cached("C4", Slot::Chest, &[
            (Stat::CriticalRating, 520), (Stat::TacticalMastery, 430),
            (Stat::Finesse, 310), (Stat::TacticalMitigation, 230),
        ]));
        resolved.insert("C5".into(), make_cached("C5", Slot::Chest, &[
            (Stat::CriticalRating, 460), (Stat::TacticalMastery, 450),
            (Stat::Finesse, 310), (Stat::TacticalMitigation, 230),
        ]));

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
        resolved.insert("C2".into(), make_cached("C2", Slot::Chest, &[
            (Stat::CriticalRating, 500), (Stat::TacticalMastery, 450),
            (Stat::Finesse, 310), (Stat::TacticalMitigation, 200),
        ]));
        let goals = vec![
            goal(Stat::CriticalRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::Finesse, 300),
            goal(Stat::TacticalMitigation, 200),
        ];
        let result = optimize(&resolved, &[], &["C2".to_string()], &goals);
        assert!(result.feasible);
        assert!(result.failed_minima.is_empty());
    }

    #[test]
    fn test_spec_run2_c6_wins_infeasible() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert("C6".into(), make_cached("C6", Slot::Chest, &[
            (Stat::CriticalRating, 440), (Stat::TacticalMastery, 200),
            (Stat::Finesse, 200), (Stat::TacticalMitigation, 100),
        ]));
        resolved.insert("C7".into(), make_cached("C7", Slot::Chest, &[
            (Stat::CriticalRating, 400), (Stat::TacticalMastery, 440),
            (Stat::Finesse, 290), (Stat::TacticalMitigation, 190),
        ]));

        let goals = vec![
            goal(Stat::CriticalRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::Finesse, 300),
            goal(Stat::TacticalMitigation, 200),
        ];

        let name_strings = vec!["C6".to_string(), "C7".to_string()];
        let result = optimize(&resolved, &[], &name_strings, &goals);

        assert!(!result.feasible);
        assert!(!result.failed_minima.is_empty());

        let winner = result.gear_set.items.get(&Slot::Chest)
            .map(|i| i.name.as_str()).unwrap_or("[missing]");
        assert_eq!(winner, "C6", "Expected C6; got {}", winner);
    }

    #[test]
    fn test_c5_over_c4_same_slot() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert("C4".into(), make_cached("C4", Slot::Chest, &[
            (Stat::CriticalRating, 520), (Stat::TacticalMastery, 430),
            (Stat::Finesse, 310), (Stat::TacticalMitigation, 230),
        ]));
        resolved.insert("C5".into(), make_cached("C5", Slot::Chest, &[
            (Stat::CriticalRating, 460), (Stat::TacticalMastery, 450),
            (Stat::Finesse, 310), (Stat::TacticalMitigation, 230),
        ]));

        let goals = vec![
            goal(Stat::CriticalRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::Finesse, 300),
            goal(Stat::TacticalMitigation, 200),
        ];

        let winner = single_slot_result(
            &resolved, &["C4", "C5"], goals, Slot::Chest,
        );
        assert_eq!(winner, "C5", "Expected C5; got {}", winner);
    }

    #[test]
    fn test_paired_slots_both_filled_and_summed() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert("WristA".into(), make_cached("WristA", Slot::Wrist1, &[
            (Stat::Vitality, 100),
        ]));
        resolved.insert("WristB".into(), make_cached("WristB", Slot::Wrist1, &[
            (Stat::Vitality, 80),
        ]));

        let goals = vec![goal(Stat::Vitality, 0)];
        let names = vec!["WristA".to_string(), "WristB".to_string()];
        let result = optimize(&resolved, &[], &names, &goals);

        assert!(result.gear_set.items.contains_key(&Slot::Wrist1));
        assert!(result.gear_set.items.contains_key(&Slot::Wrist2));
        assert_eq!(result.gear_set.total(&Stat::Vitality), 180);
    }

    #[test]
    fn test_no_goals_returns_first_candidates() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert("ItemA".into(), make_cached("ItemA", Slot::Head, &[
            (Stat::Vitality, 50),
        ]));
        let result = optimize(&resolved, &[], &["ItemA".to_string()], &[]);
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
        resolved.insert("Bad".into(),  make_cached("Bad",  Slot::Head, &[
            (Stat::CriticalRating, 400), (Stat::TacticalMitigation, 100),
        ]));
        resolved.insert("Ok".into(),   make_cached("Ok",   Slot::Head, &[
            (Stat::CriticalRating, 300), (Stat::TacticalMitigation, 300),
        ]));
        resolved.insert("Good".into(), make_cached("Good", Slot::Head, &[
            (Stat::CriticalRating, 300), (Stat::TacticalMitigation, 600),
        ]));

        let goals = vec![
            goal(Stat::CriticalRating,    300),
            goal(Stat::TacticalMitigation, 600),
        ];

        let winner = single_slot_result(
            &resolved, &["Bad", "Ok", "Good"], goals, Slot::Head,
        );
        assert_eq!(winner, "Good", "Expected Good; got {}", winner);
    }

    #[test]
    fn test_safe_narrowing_feasibility_flag_correct() {
        // Companion to the above: result must be reported as feasible,
        // not as infeasible with a spurious "all minima met" message.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert("Bad".into(),  make_cached("Bad",  Slot::Head, &[
            (Stat::CriticalRating, 400), (Stat::TacticalMitigation, 100),
        ]));
        resolved.insert("Good".into(), make_cached("Good", Slot::Head, &[
            (Stat::CriticalRating, 300), (Stat::TacticalMitigation, 600),
        ]));

        let goals = vec![
            goal(Stat::CriticalRating,    300),
            goal(Stat::TacticalMitigation, 600),
        ];

        let result = optimize(&resolved, &[], &["Bad".to_string(), "Good".to_string()], &goals);
        assert!(result.feasible, "Result should be feasible");
        assert!(result.failed_minima.is_empty());
    }
}