//! Gear set optimizer.
//!
//! ## Correctness of the per-slot greedy approach
//!
//! Because gear stats are strictly additive across independent slots, the
//! total of any stat S across a gear set is:
//!
//!     total(S) = ? item[slot].stat(S)   for all slots
//!
//! This means the global maximum of total(S) is achieved by independently
//! maximising item[slot].stat(S) in every slot. The slots do not interact.
//!
//! Consequence: the lexicographic optimum can be found by processing one
//! stat at a time, narrowing the candidate set per slot at each step:
//!
//!   Step 1: For each slot, keep only items achieving the maximum value of
//!           Stat1 among all candidates for that slot. (May be >1 item if tied.)
//!   Step 2: Among the survivors, keep only those achieving the maximum of
//!           Stat2. Etc.
//!
//! This is O(goals × slots × candidates_per_slot) — effectively free.
//!
//! ## Feasibility
//!
//! A result is *feasible* if every goal stat's total across all slots meets
//! its user-supplied minimum.
//!
//! Feasibility is checked at the gear-set level, not per item. We use a
//! two-phase approach:
//!
//! Phase 1 — attempt feasible optimisation:
//!   For each slot and each goal stat, a candidate is *compatible* if it
//!   allows every minimum to potentially be met when the other slots each
//!   contribute their best. Formally, candidate C in slot K is compatible if:
//!
//!     C.stat(S) + best_of_other_slots(S, K) >= minimum(S)   for all goals S
//!
//!   where best_of_other_slots(S, K) is the sum of per-slot maxima for S
//!   across all slots except K.
//!
//!   After filtering to compatible candidates only, run lexicographic
//!   narrowing. If every slot still has at least one candidate, the result
//!   is feasible.
//!
//! Phase 2 — infeasible fallback:
//!   If Phase 1 empties any slot's pool (meaning no combination can satisfy
//!   all minima), run lexicographic narrowing on the full unfiltered pools
//!   and report which minima were missed.
//!
//! ## Paired slots (Wrist, Finger, Ear)
//!
//! Items tagged Wrist1/Finger1/Ear1 by the wiki are candidates for either
//! slot in the pair. We resolve pairs before optimisation: given the candidate
//! pool for a paired type, we enumerate all ordered pairs (a, b) with a ? b
//! (using dummy zero-items to fill when only one real candidate exists) and
//! treat the pair as a single combined "super-candidate" whose stats are the
//! sum of a and b. The super-candidate is then handled like any other slot.

use std::collections::HashMap;

use crate::cache::CachedItem;
use crate::gear::{GearItem, GearSet, Slot};
use crate::stat::{Stat, StatGoal};

// ?? Constants ?????????????????????????????????????????????????????????????????

/// Maximum candidates considered per slot. Excess items are dropped with a
/// warning. Keeps the paired-slot enumeration bounded (max 6×5 = 30 pairs).
pub const MAX_CANDIDATES_PER_SLOT: usize = 6;

// ?? Public types ??????????????????????????????????????????????????????????????

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

// ?? Internal types ????????????????????????????????????????????????????????????

/// A resolved item ready for the optimizer: name + stats only.
/// Slot information has already been used to place it in the right pool.
#[derive(Debug, Clone)]
struct Candidate {
    name: String,
    stats: HashMap<Stat, i64>,
}

impl Candidate {
    fn stat(&self, s: &Stat) -> i64 {
        self.stats.get(s).copied().unwrap_or(0)
    }

    fn zero(name: impl Into<String>) -> Self {
        Candidate { name: name.into(), stats: HashMap::new() }
    }
}

/// A "super-candidate" for a paired slot: holds the two constituent items
/// (either of which may be the zero placeholder) and their combined stats.
#[derive(Debug, Clone)]
struct PairCandidate {
    a: Candidate, // goes into slot1
    b: Candidate, // goes into slot2
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

// ?? Entry point ???????????????????????????????????????????????????????????????

/// Run the optimizer.
///
/// `resolved`:   map of item name ? CachedItem (from wiki/cache lookup).
/// `equipped`:   names of currently-equipped items.
/// `candidates`: names of items in the 'lgo' chest.
/// `goals`:      ordered list of stat goals with minima.
pub fn optimize(
    resolved: &HashMap<String, CachedItem>,
    equipped: &[String],
    candidates: &[String],
    goals: &[StatGoal],
) -> OptimizeResult {
    let mut warnings: Vec<String> = Vec::new();

    // ?? 1. Build per-slot candidate pools ?????????????????????????????????????

    let all_names: Vec<String> = {
        let mut v: Vec<String> = equipped.to_vec();
        for n in candidates {
            if !v.contains(n) { v.push(n.clone()); }
        }
        v
    };

    // pools: canonical_slot ? Vec<Candidate>
    let mut pools: HashMap<Slot, Vec<Candidate>> = HashMap::new();

    for name in &all_names {
        let cached = match resolved.get(name) {
            Some(c) => c,
            None => continue,
        };
        let canonical = canonical_slot(cached.slot);
        let cand = Candidate { name: name.clone(), stats: cached.stats.clone() };
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
            vec![Candidate::zero(format!("[empty {}]", slot_display(slot)))]
        });
    }

    // ?? 2. Build super-candidates for paired slots ????????????????????????????

    let paired_canonicals = [Slot::Wrist1, Slot::Finger1, Slot::Ear1];

    let mut single_pools: HashMap<Slot, Vec<Candidate>> = HashMap::new();
    let mut pair_pools:   HashMap<Slot, Vec<PairCandidate>> = HashMap::new();

    for (&slot, pool) in &pools {
        if paired_canonicals.contains(&slot) {
            pair_pools.insert(slot, build_pairs(pool));
        } else {
            single_pools.insert(slot, pool.clone());
        }
    }

    // ?? 3. Compute per-slot, per-stat maxima (used for compatibility checks) ??

    // slot_max(slot, stat) = max stat value achievable in that slot alone.
    let single_slot_maxima = compute_single_maxima(&single_pools, goals);
    let pair_slot_maxima   = compute_pair_maxima(&pair_pools, goals);

    // global_max(stat) = sum of slot maxima across all slots.
    let global_max = compute_global_max(&single_slot_maxima, &pair_slot_maxima, goals);

    // ?? 4. Phase 1: filter to feasibility-compatible candidates ???????????????
    //
    // A candidate C in slot K is compatible if, for every goal stat S:
    //   C.stat(S) + (global_max(S) - slot_max(K, S)) >= minimum(S)
    //
    // i.e. even if every other slot contributes its absolute best for S,
    // C still allows the minimum to be reached.

    let mut feasible_single = filter_compatible_single(
        &single_pools, &single_slot_maxima, &global_max, goals,
    );
    let mut feasible_pair = filter_compatible_pair(
        &pair_pools, &pair_slot_maxima, &global_max, goals,
    );

    // Check whether Phase 1 produced a viable set (all slots non-empty).
    let phase1_viable = feasible_single.values().all(|p| !p.is_empty())
        && feasible_pair.values().all(|p| !p.is_empty());

    // ?? 5. Choose working pools and feasibility flag ???????????????????????????

    let feasible;
    let working_single_ref: &mut HashMap<Slot, Vec<Candidate>>;
    let working_pair_ref:   &mut HashMap<Slot, Vec<PairCandidate>>;

    // We need owned working copies for narrowing.
    let mut fallback_single;
    let mut fallback_pair;

    if phase1_viable {
        feasible = true;
        working_single_ref = &mut feasible_single;
        working_pair_ref   = &mut feasible_pair;
    } else {
        feasible = false;
        // Fall back to full pools.
        fallback_single = single_pools.clone();
        fallback_pair   = pair_pools.clone();
        working_single_ref = &mut fallback_single;
        working_pair_ref   = &mut fallback_pair;
    }

    // ?? 6. Lexicographic narrowing ????????????????????????????????????????????

    for goal in goals {
        narrow_single(working_single_ref, &goal.stat);
        narrow_pair(working_pair_ref, &goal.stat);
    }

    // ?? 7. Assemble the final GearSet ?????????????????????????????????????????

    let mut gear_set = GearSet::new();

    for (slot, pool) in working_single_ref.iter() {
        let chosen = pool.first().expect("pool must not be empty after narrowing");
        gear_set.items.insert(*slot, candidate_to_gear_item(chosen, *slot));
    }

    for (canonical, pairs) in working_pair_ref.iter() {
        let chosen = pairs.first().expect("pair pool must not be empty");
        let slot1 = *canonical;
        let slot2 = paired_slot2(slot1);
        gear_set.items.insert(slot1, candidate_to_gear_item(&chosen.a, slot1));
        gear_set.items.insert(slot2, candidate_to_gear_item(&chosen.b, slot2));
    }

    // ?? 8. Compute failed minima (actual achieved values) ?????????????????????

    let failed_minima: Vec<(Stat, i64, i64)> = if feasible {
        vec![]
    } else {
        goals.iter()
            .filter(|g| g.minimum > 0)
            .filter_map(|g| {
                let achieved = gear_set.total(&g.stat);
                if achieved < g.minimum {
                    Some((g.stat, g.minimum, achieved))
                } else {
                    None
                }
            })
            .collect()
    };

    OptimizeResult { gear_set, feasible, failed_minima, warnings }
}

// ?? Feasibility filtering ?????????????????????????????????????????????????????

/// For each (slot, stat), the maximum stat value any candidate in that slot
/// can contribute. Used in the compatibility formula.
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

/// Filter single-slot pools to only candidates compatible with all minima.
/// A candidate C in slot K is compatible if:
///   C.stat(S) + global_max(S) - slot_max(K, S) >= minimum(S)  for all S
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
                    .and_then(|m| m.get(&g.stat))
                    .copied()
                    .unwrap_or(0);
                let global_best = global_max.get(&g.stat).copied().unwrap_or(0);
                let best_without_this_slot = global_best - slot_best;
                c.stat(&g.stat) + best_without_this_slot >= g.minimum
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
                    .and_then(|m| m.get(&g.stat))
                    .copied()
                    .unwrap_or(0);
                let global_best = global_max.get(&g.stat).copied().unwrap_or(0);
                let best_without_this_slot = global_best - slot_best;
                p.stat(&g.stat) + best_without_this_slot >= g.minimum
            })
        }).cloned().collect();
        out.insert(slot, filtered);
    }
    out
}

// ?? Lexicographic narrowing ????????????????????????????????????????????????????

/// For each slot, retain only candidates achieving the slot-maximum for
/// this stat. Preserves at least one candidate (the pool cannot become empty
/// because the maximum was drawn from the pool itself).
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

// ?? Other helpers ?????????????????????????????????????????????????????????????

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

fn build_pairs(pool: &[Candidate]) -> Vec<PairCandidate> {
    if pool.is_empty() {
        return vec![PairCandidate::new(
            Candidate::zero("[empty]"), Candidate::zero("[empty]"),
        )];
    }
    if pool.len() == 1 {
        return vec![PairCandidate::new(
            pool[0].clone(), Candidate::zero("[empty]"),
        )];
    }
    let mut pairs = Vec::new();
    for i in 0..pool.len() {
        for j in 0..pool.len() {
            if i != j {
                pairs.push(PairCandidate::new(pool[i].clone(), pool[j].clone()));
            }
        }
    }
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
        Slot::OffHand   => "Off-hand",
        Slot::Ranged    => "Ranged",
    }
}

// ?? Tests ?????????????????????????????????????????????????????????????????????

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
        // Run 1: all minima can be met.
        // C2 should win: highest CritRating (500) among feasible candidates.
        // C4 is excluded (TactMast 430 < 450).
        // C1 is excluded (TactMast 420 < 450, TactMit 190 < 200).
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert("C1".into(), make_cached("C1", Slot::Chest, &[
            (Stat::CritRating, 480), (Stat::TacticalMastery, 420),
            (Stat::FinesseRating, 310), (Stat::TacticalMitigation, 190),
        ]));
        resolved.insert("C2".into(), make_cached("C2", Slot::Chest, &[
            (Stat::CritRating, 500), (Stat::TacticalMastery, 450),
            (Stat::FinesseRating, 310), (Stat::TacticalMitigation, 200),
        ]));
        resolved.insert("C3".into(), make_cached("C3", Slot::Chest, &[
            (Stat::CritRating, 490), (Stat::TacticalMastery, 450),
            (Stat::FinesseRating, 310), (Stat::TacticalMitigation, 230),
        ]));
        resolved.insert("C4".into(), make_cached("C4", Slot::Chest, &[
            (Stat::CritRating, 520), (Stat::TacticalMastery, 430),
            (Stat::FinesseRating, 310), (Stat::TacticalMitigation, 230),
        ]));
        resolved.insert("C5".into(), make_cached("C5", Slot::Chest, &[
            (Stat::CritRating, 460), (Stat::TacticalMastery, 450),
            (Stat::FinesseRating, 310), (Stat::TacticalMitigation, 230),
        ]));

        let goals = vec![
            goal(Stat::CritRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::FinesseRating, 300),
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
            (Stat::CritRating, 500), (Stat::TacticalMastery, 450),
            (Stat::FinesseRating, 310), (Stat::TacticalMitigation, 200),
        ]));
        let goals = vec![
            goal(Stat::CritRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::FinesseRating, 300),
            goal(Stat::TacticalMitigation, 200),
        ];
        let result = optimize(&resolved, &[], &["C2".to_string()], &goals);
        assert!(result.feasible);
        assert!(result.failed_minima.is_empty());
    }

    #[test]
    fn test_spec_run2_c6_wins_infeasible() {
        // Run 2: no combination meets all minima.
        // C6 wins because CritRating 440 > C7's 400.
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert("C6".into(), make_cached("C6", Slot::Chest, &[
            (Stat::CritRating, 440), (Stat::TacticalMastery, 200),
            (Stat::FinesseRating, 200), (Stat::TacticalMitigation, 100),
        ]));
        resolved.insert("C7".into(), make_cached("C7", Slot::Chest, &[
            (Stat::CritRating, 400), (Stat::TacticalMastery, 440),
            (Stat::FinesseRating, 290), (Stat::TacticalMitigation, 190),
        ]));

        let goals = vec![
            goal(Stat::CritRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::FinesseRating, 300),
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
        // C5 (feasible, CritRating=460) should beat C4 (infeasible, CritRating=520).
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert("C4".into(), make_cached("C4", Slot::Chest, &[
            (Stat::CritRating, 520), (Stat::TacticalMastery, 430),
            (Stat::FinesseRating, 310), (Stat::TacticalMitigation, 230),
        ]));
        resolved.insert("C5".into(), make_cached("C5", Slot::Chest, &[
            (Stat::CritRating, 460), (Stat::TacticalMastery, 450),
            (Stat::FinesseRating, 310), (Stat::TacticalMitigation, 230),
        ]));

        let goals = vec![
            goal(Stat::CritRating, 450),
            goal(Stat::TacticalMastery, 450),
            goal(Stat::FinesseRating, 300),
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
}