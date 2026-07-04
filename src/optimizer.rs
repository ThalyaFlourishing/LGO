//! Gear set optimizer.
//!
//! Model guidance: high borrow-checker/algorithmic friction — see `docs/MODEL_GUIDANCE.md` before non-trivial edits.
//!
//! Implements the clamped-satisfaction objective from
//! `docs/Optimizer_Overhaul/07 - Locked Semantics and Rewrite Plan.md` §1.
//! The search is exact: candidates are first dominance-filtered within each
//! slot/family, then a branch-and-bound DFS returns the comparator-maximal
//! complete build. Only goal stats participate in search; all selected item
//! stats are retained for display.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::gear::{CachedItem, GearItem, GearSet, Slot};
use crate::stat::{Stat, StatGoal};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum supported candidates per canonical slot or paired-slot family.
/// Keeps paired-slot enumeration bounded (max 8×7/2 = 28 real pairs, plus
/// singleton empty-placeholder pairs).
pub const MAX_CANDIDATES_PER_SLOT: usize = 8;

// ── Public types ──────────────────────────────────────────────────────────────

/// The result returned by the optimizer.
#[derive(Debug)]
pub struct OptimizeResult {
    pub gear_set: GearSet,
    pub feasible: bool,
    /// For each goal stat that failed its minimum: (stat, minimum, achieved).
    pub failed_minima: Vec<(Stat, i64, i64)>,
    /// Warning messages (e.g. empty-slot placeholders).
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizeError {
    TooManyCandidates {
        slot_label: String,
        count: usize,
        max: usize,
    },
}

impl fmt::Display for OptimizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptimizeError::TooManyCandidates {
                slot_label,
                count,
                max,
            } => write!(
                f,
                "Too many candidates for slot \"{}\": {} provided, maximum allowed is {}. Remove items from the 'lgo' chest and re-export before running 'lgo optimize'.",
                slot_label, count, max
            ),
        }
    }
}

impl std::error::Error for OptimizeError {}

// ── Internal types ────────────────────────────────────────────────────────────

/// A resolved item ready for the optimizer: instance key, name, stats, and slot.
#[derive(Debug, Clone)]
struct Candidate {
    /// Stable per-owned-instance optimizer key; distinguishes duplicate items
    /// with the same display name and feeds the deterministic final tiebreak.
    key: String,
    name: String,
    stats: HashMap<Stat, i64>,
    original_slot: Slot,
}

impl Candidate {
    fn stat(&self, s: &Stat) -> i64 {
        self.stats.get(s).copied().unwrap_or(0)
    }

    fn zero(name: impl Into<String>, slot: Slot) -> Self {
        let name = name.into();
        Candidate {
            key: format!("0000::{}::{}", slot, name),
            name,
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

#[derive(Debug, Clone)]
enum Choice {
    Single {
        slot: Slot,
        candidate: Candidate,
    },
    Pair {
        slot1: Slot,
        slot2: Slot,
        pair: PairCandidate,
    },
}

impl Choice {
    fn stat(&self, stat: &Stat) -> i64 {
        match self {
            Choice::Single { candidate, .. } => candidate.stat(stat),
            Choice::Pair { pair, .. } => pair.stat(stat),
        }
    }

    fn goal_totals(&self, goals: &[StatGoal]) -> Vec<i64> {
        goals.iter().map(|goal| self.stat(&goal.stat)).collect()
    }

    fn instance_keys(&self) -> Vec<String> {
        match self {
            Choice::Single { candidate, .. } => vec![candidate.key.clone()],
            Choice::Pair { pair, .. } => {
                let mut keys = vec![pair.a.key.clone(), pair.b.key.clone()];
                keys.sort();
                keys
            }
        }
    }

    fn sort_key(&self) -> Vec<String> {
        self.instance_keys()
    }

    fn insert_into(&self, gear_set: &mut GearSet) {
        match self {
            Choice::Single { slot, candidate } => {
                gear_set
                    .items
                    .insert(*slot, candidate_to_gear_item(candidate, *slot));
            }
            Choice::Pair { slot1, slot2, pair } => {
                let (item_for_slot1, item_for_slot2) =
                    if pair.a.original_slot == *slot1 && pair.b.original_slot == *slot2 {
                        (&pair.a, &pair.b)
                    } else if pair.a.original_slot == *slot2 && pair.b.original_slot == *slot1 {
                        (&pair.b, &pair.a)
                    } else {
                        (&pair.a, &pair.b)
                    };

                gear_set
                    .items
                    .insert(*slot1, candidate_to_gear_item(item_for_slot1, *slot1));
                gear_set
                    .items
                    .insert(*slot2, candidate_to_gear_item(item_for_slot2, *slot2));
            }
        }
    }
}

#[derive(Debug, Clone)]
struct SearchPool {
    choices: Vec<Choice>,
}

#[derive(Debug, Clone)]
struct SearchBuild {
    totals: Vec<i64>,
    tiebreak_key: Vec<String>,
    choices: Vec<Choice>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn optimize(
    resolved: &HashMap<String, CachedItem>,
    candidates: &[String],
    goals: &[StatGoal],
) -> Result<OptimizeResult, Box<OptimizeError>> {
    let (pools, warnings) = build_search_pools(resolved, candidates, goals, true)?;
    let best = exact_search(&pools, goals).unwrap_or_else(|| SearchBuild {
        totals: vec![0; goals.len()],
        tiebreak_key: Vec::new(),
        choices: Vec::new(),
    });

    let mut gear_set = GearSet::new();
    for choice in &best.choices {
        choice.insert_into(&mut gear_set);
    }

    let failed_minima = failed_minima(&gear_set, goals);
    let feasible = failed_minima.is_empty();

    Ok(OptimizeResult {
        gear_set,
        feasible,
        failed_minima,
        warnings,
    })
}

// ── Objective comparator ─────────────────────────────────────────────────────

fn compare_builds(x_totals: &[i64], y_totals: &[i64], goals: &[StatGoal]) -> Ordering {
    debug_assert_eq!(x_totals.len(), goals.len());
    debug_assert_eq!(y_totals.len(), goals.len());

    for ((&x, &y), goal) in x_totals.iter().zip(y_totals).zip(goals) {
        let x_met = goal.minimum <= 0 || x >= goal.minimum;
        let y_met = goal.minimum <= 0 || y >= goal.minimum;
        match x_met.cmp(&y_met) {
            Ordering::Equal => {}
            non_equal => return non_equal,
        }
    }

    for ((&x, &y), goal) in x_totals.iter().zip(y_totals).zip(goals) {
        if goal.minimum <= 0 {
            continue;
        }
        match x.min(goal.minimum).cmp(&y.min(goal.minimum)) {
            Ordering::Equal => {}
            non_equal => return non_equal,
        }
    }

    for (&x, &y) in x_totals.iter().zip(y_totals) {
        match x.cmp(&y) {
            Ordering::Equal => {}
            non_equal => return non_equal,
        }
    }

    Ordering::Equal
}

fn compare_search_builds(x: &SearchBuild, y: &SearchBuild, goals: &[StatGoal]) -> Ordering {
    match compare_builds(&x.totals, &y.totals, goals) {
        // Earlier sorted instance keys win final ties. `cmp` returns Greater
        // when the right-hand key is later, so compare in reverse order.
        Ordering::Equal => y.tiebreak_key.cmp(&x.tiebreak_key),
        non_equal => non_equal,
    }
}

fn should_replace_best(
    candidate: &SearchBuild,
    best: Option<&SearchBuild>,
    goals: &[StatGoal],
) -> bool {
    best.is_none_or(|best_build| compare_search_builds(candidate, best_build, goals).is_gt())
}

// ── Exact production search ──────────────────────────────────────────────────

fn build_search_pools(
    resolved: &HashMap<String, CachedItem>,
    candidates: &[String],
    goals: &[StatGoal],
    apply_dominance: bool,
) -> Result<(Vec<SearchPool>, Vec<String>), Box<OptimizeError>> {
    let mut warnings: Vec<String> = Vec::new();
    let mut pools: HashMap<Slot, Vec<Candidate>> = HashMap::new();

    let mut all_names: Vec<String> = candidates.to_vec();
    all_names.sort();

    for key in &all_names {
        let cached = match resolved.get(key) {
            Some(c) => c,
            None => continue,
        };
        let canonical = canonical_slot(cached.slot);
        let cand = Candidate {
            key: key.clone(),
            name: cached.name.clone(),
            stats: cached.stats.clone(),
            original_slot: cached.slot,
        };
        pools.entry(canonical).or_default().push(cand);
    }

    validate_candidate_pool_sizes(&pools)?;

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

    let paired_canonicals = [Slot::Wrist1, Slot::Finger1, Slot::Ear1];
    let mut seen = HashSet::new();
    let mut search_pools = Vec::new();

    for &slot in Slot::ALL {
        let canonical = canonical_slot(slot);
        if !seen.insert(canonical) {
            continue;
        }

        let mut choices: Vec<Choice> = if paired_canonicals.contains(&canonical) {
            build_pairs(
                pools.get(&canonical).map(Vec::as_slice).unwrap_or(&[]),
                canonical,
                paired_slot2(canonical),
            )
            .into_iter()
            .map(|pair| Choice::Pair {
                slot1: canonical,
                slot2: paired_slot2(canonical),
                pair,
            })
            .collect()
        } else {
            pools
                .get(&canonical)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|candidate| Choice::Single {
                    slot: canonical,
                    candidate,
                })
                .collect()
        };

        choices.sort_by_key(Choice::sort_key);
        if apply_dominance {
            choices = dominance_filter(choices, goals);
        }
        search_pools.push(SearchPool { choices });
    }

    Ok((search_pools, warnings))
}

fn dominance_filter(choices: Vec<Choice>, goals: &[StatGoal]) -> Vec<Choice> {
    if choices.len() <= 1 {
        return choices;
    }

    let goal_vectors: Vec<Vec<i64>> = choices
        .iter()
        .map(|choice| choice.goal_totals(goals))
        .collect();
    let mut keep = vec![true; choices.len()];

    for i in 0..choices.len() {
        if !keep[i] {
            continue;
        }
        for j in 0..choices.len() {
            if i == j {
                continue;
            }
            let i_le_j = goal_vectors[i]
                .iter()
                .zip(&goal_vectors[j])
                .all(|(i_value, j_value)| i_value <= j_value);
            if !i_le_j {
                continue;
            }
            let i_lt_j = goal_vectors[i]
                .iter()
                .zip(&goal_vectors[j])
                .any(|(i_value, j_value)| i_value < j_value);
            if i_lt_j || i > j {
                keep[i] = false;
                break;
            }
        }
    }

    choices
        .into_iter()
        .enumerate()
        .filter_map(|(idx, choice)| keep[idx].then_some(choice))
        .collect()
}

fn exact_search(pools: &[SearchPool], goals: &[StatGoal]) -> Option<SearchBuild> {
    if pools.iter().any(|pool| pool.choices.is_empty()) {
        return None;
    }

    let suffix_maxima = suffix_goal_maxima(pools, goals);
    let mut best: Option<SearchBuild> = None;
    let mut running_totals = vec![0; goals.len()];
    let mut current_choices: Vec<Choice> = Vec::with_capacity(pools.len());
    let mut current_keys: Vec<String> = Vec::new();

    dfs_search(
        0,
        pools,
        goals,
        &suffix_maxima,
        &mut running_totals,
        &mut current_choices,
        &mut current_keys,
        &mut best,
    );

    best
}

fn suffix_goal_maxima(pools: &[SearchPool], goals: &[StatGoal]) -> Vec<Vec<i64>> {
    let mut suffix = vec![vec![0; goals.len()]; pools.len() + 1];
    for idx in (0..pools.len()).rev() {
        suffix[idx] = suffix[idx + 1].clone();
        for (goal_idx, goal) in goals.iter().enumerate() {
            let pool_max = pools[idx]
                .choices
                .iter()
                .map(|choice| choice.stat(&goal.stat))
                .max()
                .unwrap_or(0);
            suffix[idx][goal_idx] += pool_max;
        }
    }
    suffix
}

#[allow(clippy::too_many_arguments)]
fn dfs_search(
    pool_idx: usize,
    pools: &[SearchPool],
    goals: &[StatGoal],
    suffix_maxima: &[Vec<i64>],
    running_totals: &mut [i64],
    current_choices: &mut Vec<Choice>,
    current_keys: &mut Vec<String>,
    best: &mut Option<SearchBuild>,
) {
    if let Some(best_build) = best.as_ref() {
        let optimistic: Vec<i64> = running_totals
            .iter()
            .zip(&suffix_maxima[pool_idx])
            .map(|(running, remaining_max)| running + remaining_max)
            .collect();
        if compare_builds(&optimistic, &best_build.totals, goals) == Ordering::Less {
            return;
        }
    }

    if pool_idx == pools.len() {
        let mut tiebreak_key = current_keys.clone();
        tiebreak_key.sort();
        let candidate = SearchBuild {
            totals: running_totals.to_vec(),
            tiebreak_key,
            choices: current_choices.clone(),
        };
        if should_replace_best(&candidate, best.as_ref(), goals) {
            *best = Some(candidate);
        }
        return;
    }

    for choice in &pools[pool_idx].choices {
        let choice_totals = choice.goal_totals(goals);
        let choice_keys = choice.instance_keys();

        for (running, add) in running_totals.iter_mut().zip(&choice_totals) {
            *running += add;
        }
        current_keys.extend(choice_keys.iter().cloned());
        current_choices.push(choice.clone());

        dfs_search(
            pool_idx + 1,
            pools,
            goals,
            suffix_maxima,
            running_totals,
            current_choices,
            current_keys,
            best,
        );

        current_choices.pop();
        for _ in 0..choice_keys.len() {
            current_keys.pop();
        }
        for (running, add) in running_totals.iter_mut().zip(&choice_totals) {
            *running -= add;
        }
    }
}

fn failed_minima(gear_set: &GearSet, goals: &[StatGoal]) -> Vec<(Stat, i64, i64)> {
    goals
        .iter()
        .filter(|g| g.minimum > 0)
        .filter_map(|g| {
            let achieved = gear_set.total(&g.stat);
            (achieved < g.minimum).then_some((g.stat, g.minimum, achieved))
        })
        .collect()
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
    if pool.is_empty() {
        return vec![PairCandidate::new(
            Candidate::zero("[empty]", slot1),
            Candidate::zero("[empty]", slot2),
        )];
    }
    if pool.len() == 1 {
        if pool[0].original_slot == slot2 {
            return vec![PairCandidate::new(
                Candidate::zero(format!("[empty {}]", slot_display(slot1)), slot1),
                pool[0].clone(),
            )];
        }
        return vec![PairCandidate::new(
            pool[0].clone(),
            Candidate::zero(format!("[empty {}]", slot_display(slot2)), slot2),
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

fn validate_candidate_pool_sizes(
    pools: &HashMap<Slot, Vec<Candidate>>,
) -> Result<(), Box<OptimizeError>> {
    let mut seen = HashSet::new();

    for &slot in Slot::ALL {
        let canonical = canonical_slot(slot);
        if !seen.insert(canonical) {
            continue;
        }

        if let Some(pool) = pools.get(&canonical) {
            if pool.len() > MAX_CANDIDATES_PER_SLOT {
                return Err(Box::new(OptimizeError::TooManyCandidates {
                    slot_label: slot_display(canonical).to_string(),
                    count: pool.len(),
                    max: MAX_CANDIDATES_PER_SLOT,
                }));
            }
        }
    }

    Ok(())
}

// ────────── TESTS ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Lcg(u64);

    impl Lcg {
        fn new(seed: u64) -> Self {
            Self(seed)
        }

        fn next_u32(&mut self) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 32) as u32
        }

        fn range_usize(&mut self, start: usize, end_inclusive: usize) -> usize {
            start + (self.next_u32() as usize % (end_inclusive - start + 1))
        }

        fn range_i64(&mut self, start: i64, end_inclusive: i64) -> i64 {
            start + (self.next_u32() as i64).rem_euclid(end_inclusive - start + 1)
        }

        fn chance(&mut self, numerator: u32, denominator: u32) -> bool {
            self.next_u32() % denominator < numerator
        }
    }

    fn make_cached(name: &str, slot: Slot, stats: &[(Stat, i64)]) -> CachedItem {
        CachedItem {
            name: name.to_string(),
            slot,
            stats: stats.iter().copied().collect(),
        }
    }

    fn instance_key(idx: usize, slot: Slot, name: &str) -> String {
        crate::gear::optimizer_candidate_key(
            idx,
            &CachedItem {
                name: name.to_string(),
                slot,
                stats: HashMap::new(),
            },
        )
    }

    fn goal(stat: Stat, minimum: i64) -> StatGoal {
        StatGoal { stat, minimum }
    }

    fn optimize_ok(
        resolved: &HashMap<String, CachedItem>,
        candidates: &[String],
        goals: &[StatGoal],
    ) -> OptimizeResult {
        optimize(resolved, candidates, goals).expect("optimization should succeed")
    }

    fn single_slot_result(
        resolved: &HashMap<String, CachedItem>,
        names: &[&str],
        goals: Vec<StatGoal>,
        slot: Slot,
    ) -> String {
        let name_strings: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        let result = optimize_ok(resolved, &name_strings, &goals);
        result
            .gear_set
            .items
            .get(&slot)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| "[missing]".to_string())
    }

    fn result_goal_totals(result: &OptimizeResult, goals: &[StatGoal]) -> Vec<i64> {
        goals
            .iter()
            .map(|goal| result.gear_set.total(&goal.stat))
            .collect()
    }

    fn oracle_optimize(
        resolved: &HashMap<String, CachedItem>,
        candidates: &[String],
        goals: &[StatGoal],
    ) -> OptimizeResult {
        let (pools, warnings) = build_search_pools(resolved, candidates, goals, false)
            .expect("oracle inputs should be within candidate cap");
        let best = oracle_best_build(&pools, goals).expect("oracle pools should not be empty");
        let mut gear_set = GearSet::new();
        for choice in &best.choices {
            choice.insert_into(&mut gear_set);
        }
        let failed_minima = failed_minima(&gear_set, goals);
        OptimizeResult {
            gear_set,
            feasible: failed_minima.is_empty(),
            failed_minima,
            warnings,
        }
    }

    fn oracle_best_build(pools: &[SearchPool], goals: &[StatGoal]) -> Option<SearchBuild> {
        fn rec(
            pool_idx: usize,
            pools: &[SearchPool],
            goals: &[StatGoal],
            running_totals: &mut [i64],
            current_choices: &mut Vec<Choice>,
            current_keys: &mut Vec<String>,
            best: &mut Option<SearchBuild>,
        ) {
            if pool_idx == pools.len() {
                let mut tiebreak_key = current_keys.clone();
                tiebreak_key.sort();
                let candidate = SearchBuild {
                    totals: running_totals.to_vec(),
                    tiebreak_key,
                    choices: current_choices.clone(),
                };
                if should_replace_best(&candidate, best.as_ref(), goals) {
                    *best = Some(candidate);
                }
                return;
            }

            for choice in &pools[pool_idx].choices {
                let choice_totals = choice.goal_totals(goals);
                let choice_keys = choice.instance_keys();
                for (running, add) in running_totals.iter_mut().zip(&choice_totals) {
                    *running += add;
                }
                current_keys.extend(choice_keys.iter().cloned());
                current_choices.push(choice.clone());

                rec(
                    pool_idx + 1,
                    pools,
                    goals,
                    running_totals,
                    current_choices,
                    current_keys,
                    best,
                );

                current_choices.pop();
                for _ in 0..choice_keys.len() {
                    current_keys.pop();
                }
                for (running, add) in running_totals.iter_mut().zip(&choice_totals) {
                    *running -= add;
                }
            }
        }

        if pools.iter().any(|pool| pool.choices.is_empty()) {
            return None;
        }
        let mut best = None;
        rec(
            0,
            pools,
            goals,
            &mut vec![0; goals.len()],
            &mut Vec::new(),
            &mut Vec::new(),
            &mut best,
        );
        best
    }

    fn assert_x_beats_y(x: &[i64], y: &[i64], goals: &[StatGoal]) {
        assert_eq!(compare_builds(x, y, goals), Ordering::Greater);
        assert_eq!(compare_builds(y, x, goals), Ordering::Less);
    }

    #[test]
    fn comparator_worked_example_both_meet_first_goal_stage2_second_goal() {
        let goals = vec![
            goal(Stat::CriticalRating, 100_000),
            goal(Stat::TacticalMastery, 100_000),
        ];
        assert_x_beats_y(&[120_000, 90_000], &[100_000, 89_999], &goals);
    }

    #[test]
    fn comparator_worked_example_ratchet_protects_met_lower_goal() {
        let goals = vec![
            goal(Stat::CriticalRating, 100),
            goal(Stat::TacticalMastery, 100),
        ];
        assert_x_beats_y(&[94, 100], &[96, 80], &goals);
    }

    #[test]
    fn comparator_worked_example_meeting_priority_one_justifies_drop() {
        let goals = vec![
            goal(Stat::CriticalRating, 100),
            goal(Stat::TacticalMastery, 100),
        ];
        assert_x_beats_y(&[100, 70], &[98, 100], &goals);
    }

    #[test]
    fn comparator_worked_example_both_unmet_stage2_priority_order() {
        let goals = vec![
            goal(Stat::CriticalRating, 100),
            goal(Stat::TacticalMastery, 100),
        ];
        assert_x_beats_y(&[95, 96], &[94, 97], &goals);
    }

    #[test]
    fn comparator_worked_example_all_met_stage3_raw_polish() {
        let goals = vec![
            goal(Stat::CriticalRating, 100_000),
            goal(Stat::TacticalMastery, 100_000),
        ];
        assert_x_beats_y(&[250_000, 100_000], &[100_000, 100_000], &goals);
    }

    #[test]
    fn comparator_zero_minimum_is_met_and_uses_stage3_raw_value() {
        let goals = vec![
            goal(Stat::CriticalRating, 0),
            goal(Stat::TacticalMastery, 100),
        ];
        assert_x_beats_y(&[200, 100], &[100, 100], &goals);
    }

    #[test]
    fn test_spec_run1_c2_wins() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        for (name, cr, tm, fnv, tt) in [
            ("C1", 480, 420, 310, 190),
            ("C2", 500, 450, 310, 200),
            ("C3", 490, 450, 310, 230),
            ("C4", 520, 430, 310, 230),
            ("C5", 460, 450, 310, 230),
        ] {
            resolved.insert(
                name.into(),
                make_cached(
                    name,
                    Slot::Chest,
                    &[
                        (Stat::CriticalRating, cr),
                        (Stat::TacticalMastery, tm),
                        (Stat::Finesse, fnv),
                        (Stat::TacticalMitigation, tt),
                    ],
                ),
            );
        }
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
        assert_eq!(winner, "C2");
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
        let result = optimize_ok(&resolved, &["C2".to_string()], &goals);
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
        let result = optimize_ok(&resolved, &["C6".to_string(), "C7".to_string()], &goals);
        assert!(!result.feasible);
        assert_eq!(result.gear_set.items[&Slot::Chest].name, "C6");
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
        assert_eq!(winner, "C5");
    }

    #[test]
    fn dominance_safety_keeps_lower_priority_met_ratchet_candidate() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Balanced".into(),
            make_cached(
                "Balanced",
                Slot::Head,
                &[(Stat::CriticalRating, 9), (Stat::TacticalMastery, 9)],
            ),
        );
        resolved.insert(
            "HighCR".into(),
            make_cached(
                "HighCR",
                Slot::Head,
                &[(Stat::CriticalRating, 10), (Stat::TacticalMastery, 0)],
            ),
        );
        resolved.insert(
            "Filler".into(),
            make_cached("Filler", Slot::Chest, &[(Stat::TacticalMastery, 1)]),
        );
        let names = vec![
            "Balanced".to_string(),
            "HighCR".to_string(),
            "Filler".to_string(),
        ];
        let goals = vec![
            goal(Stat::CriticalRating, 11),
            goal(Stat::TacticalMastery, 10),
        ];
        let production = optimize_ok(&resolved, &names, &goals);
        let oracle = oracle_optimize(&resolved, &names, &goals);
        assert_eq!(production.gear_set.items[&Slot::Head].name, "Balanced");
        assert_eq!(
            compare_builds(
                &result_goal_totals(&production, &goals),
                &result_goal_totals(&oracle, &goals),
                &goals,
            ),
            Ordering::Equal
        );
    }

    #[test]
    fn branch_and_bound_exactness_rejects_high_priority_overshoot_for_lower_goal() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Overshoot".into(),
            make_cached(
                "Overshoot",
                Slot::Head,
                &[(Stat::CriticalRating, 100), (Stat::TacticalMastery, 0)],
            ),
        );
        resolved.insert(
            "Satisfied".into(),
            make_cached(
                "Satisfied",
                Slot::Head,
                &[(Stat::CriticalRating, 10), (Stat::TacticalMastery, 10)],
            ),
        );
        let goals = vec![
            goal(Stat::CriticalRating, 10),
            goal(Stat::TacticalMastery, 10),
        ];
        let winner = single_slot_result(&resolved, &["Overshoot", "Satisfied"], goals, Slot::Head);
        assert_eq!(winner, "Satisfied");
    }

    #[test]
    fn test_paired_slots_use_two_distinct_instances_and_sum_once_each() {
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
        let result = optimize_ok(
            &resolved,
            &["WristA".to_string(), "WristB".to_string()],
            &goals,
        );
        assert_eq!(result.gear_set.total(&Stat::Vitality), 180);
        let mut chosen = [
            result.gear_set.items[&Slot::Wrist1].name.as_str(),
            result.gear_set.items[&Slot::Wrist2].name.as_str(),
        ];
        chosen.sort_unstable();
        assert_eq!(chosen, ["WristA", "WristB"]);
    }

    #[test]
    fn test_no_goals_returns_first_candidates() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "ItemA".into(),
            make_cached("ItemA", Slot::Head, &[(Stat::Vitality, 50)]),
        );
        let result = optimize_ok(&resolved, &["ItemA".to_string()], &[]);
        assert!(result.feasible);
        assert!(result.failed_minima.is_empty());
    }

    #[test]
    fn test_single_paired_instance_cannot_fill_both_slots() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "WristX".into(),
            make_cached("WristX", Slot::Wrist1, &[(Stat::CriticalRating, 500)]),
        );
        let goals = vec![goal(Stat::CriticalRating, 400)];
        let result = optimize_ok(&resolved, &["WristX".to_string()], &goals);
        assert!(result.feasible);
        assert_eq!(result.gear_set.items[&Slot::Wrist1].name, "WristX");
        assert!(result.gear_set.items[&Slot::Wrist2]
            .name
            .starts_with("[empty"));
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 500);
    }

    #[test]
    fn test_no_self_pair_for_tight_minimum_is_infeasible() {
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
        let result = optimize_ok(
            &resolved,
            &["RingA".to_string(), "RingB".to_string()],
            &goals,
        );
        assert!(!result.feasible);
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 800);
    }

    #[test]
    fn test_two_distinct_same_name_instances_can_fill_paired_slots() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            instance_key(1, Slot::Finger1, "Same Ring"),
            make_cached("Same Ring", Slot::Finger1, &[(Stat::CriticalRating, 500)]),
        );
        resolved.insert(
            instance_key(2, Slot::Finger1, "Same Ring"),
            make_cached("Same Ring", Slot::Finger1, &[(Stat::CriticalRating, 500)]),
        );
        let goals = vec![goal(Stat::CriticalRating, 900)];
        let names = vec![
            instance_key(1, Slot::Finger1, "Same Ring"),
            instance_key(2, Slot::Finger1, "Same Ring"),
        ];
        let result = optimize_ok(&resolved, &names, &goals);
        assert!(result.feasible);
        assert_eq!(result.gear_set.items[&Slot::Finger1].name, "Same Ring");
        assert_eq!(result.gear_set.items[&Slot::Finger2].name, "Same Ring");
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 1000);
    }

    #[test]
    fn test_one_same_name_instance_alone_is_infeasible_but_two_are_feasible() {
        let goals = vec![goal(Stat::CriticalRating, 900)];
        let mut one: HashMap<String, CachedItem> = HashMap::new();
        one.insert(
            instance_key(1, Slot::Finger1, "Same Ring"),
            make_cached("Same Ring", Slot::Finger1, &[(Stat::CriticalRating, 500)]),
        );
        let one_name = vec![instance_key(1, Slot::Finger1, "Same Ring")];
        let one_result = optimize_ok(&one, &one_name, &goals);
        assert!(!one_result.feasible);
        assert_eq!(one_result.gear_set.total(&Stat::CriticalRating), 500);

        let mut two = one;
        two.insert(
            instance_key(2, Slot::Finger1, "Same Ring"),
            make_cached("Same Ring", Slot::Finger1, &[(Stat::CriticalRating, 500)]),
        );
        let two_names = vec![
            instance_key(1, Slot::Finger1, "Same Ring"),
            instance_key(2, Slot::Finger1, "Same Ring"),
        ];
        let two_result = optimize_ok(&two, &two_names, &goals);
        assert!(two_result.feasible);
        assert_eq!(two_result.gear_set.total(&Stat::CriticalRating), 1000);
    }

    #[test]
    fn test_pair_family_consistency_for_ear_singleton_and_duplicate_copy() {
        let goals = vec![goal(Stat::TacticalMitigation, 700)];
        let mut one: HashMap<String, CachedItem> = HashMap::new();
        one.insert(
            "EarOnly".into(),
            make_cached(
                "Same Earring",
                Slot::Ear1,
                &[(Stat::TacticalMitigation, 400)],
            ),
        );
        let one_result = optimize_ok(&one, &["EarOnly".to_string()], &goals);
        assert!(!one_result.feasible);
        assert_eq!(one_result.gear_set.total(&Stat::TacticalMitigation), 400);

        let mut two = one;
        two.insert(
            "EarCopy".into(),
            make_cached(
                "Same Earring",
                Slot::Ear1,
                &[(Stat::TacticalMitigation, 400)],
            ),
        );
        let two_result = optimize_ok(
            &two,
            &["EarOnly".to_string(), "EarCopy".to_string()],
            &goals,
        );
        assert!(two_result.feasible);
        assert_eq!(two_result.gear_set.total(&Stat::TacticalMitigation), 800);
        assert_eq!(two_result.gear_set.items[&Slot::Ear1].name, "Same Earring");
        assert_eq!(two_result.gear_set.items[&Slot::Ear2].name, "Same Earring");
    }

    #[test]
    fn test_pair_infeasible_when_minimum_exceeds_best_legal_pair() {
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
        let result = optimize_ok(
            &resolved,
            &["RingA".to_string(), "RingB".to_string()],
            &goals,
        );
        assert!(!result.feasible);
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 800);
    }

    #[test]
    fn test_single_candidate_meets_minimum_exactly() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Helm".into(),
            make_cached("Helm", Slot::Head, &[(Stat::TacticalMitigation, 300)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Helm".to_string()],
            &[goal(Stat::TacticalMitigation, 300)],
        );
        assert!(result.feasible);
    }

    #[test]
    fn test_single_candidate_one_below_minimum() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Helm".into(),
            make_cached("Helm", Slot::Head, &[(Stat::TacticalMitigation, 299)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Helm".to_string()],
            &[goal(Stat::TacticalMitigation, 300)],
        );
        assert!(!result.feasible);
        assert!(matches!(
            result
                .failed_minima
                .iter()
                .find(|(s, _, _)| *s == Stat::TacticalMitigation),
            Some((_, 300, 299))
        ));
    }

    #[test]
    fn test_too_many_single_slot_candidates_is_refused() {
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
        let err = optimize(&resolved, &names, &[goal(Stat::CriticalRating, 0)]).unwrap_err();
        assert_eq!(
            *err,
            OptimizeError::TooManyCandidates {
                slot_label: "Head".to_string(),
                count: 9,
                max: 8,
            }
        );
    }

    #[test]
    fn test_too_many_paired_family_candidates_is_refused() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        let mut names: Vec<String> = Vec::new();
        for i in 1..=9usize {
            let name = format!("Ring{}", i);
            let slot = if i % 2 == 0 {
                Slot::Finger1
            } else {
                Slot::Finger2
            };
            resolved.insert(
                name.clone(),
                make_cached(&name, slot, &[(Stat::CriticalRating, i as i64 * 10)]),
            );
            names.push(name);
        }
        let err = optimize(&resolved, &names, &[goal(Stat::CriticalRating, 0)]).unwrap_err();
        assert_eq!(
            *err,
            OptimizeError::TooManyCandidates {
                slot_label: "Finger".to_string(),
                count: 9,
                max: 8,
            }
        );
    }

    #[test]
    fn test_exactly_eight_candidates_is_allowed() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        let mut names: Vec<String> = Vec::new();
        for i in 1..=8usize {
            let name = format!("Head{}", i);
            resolved.insert(
                name.clone(),
                make_cached(&name, Slot::Head, &[(Stat::CriticalRating, i as i64 * 10)]),
            );
            names.push(name);
        }
        assert!(optimize(&resolved, &names, &[goal(Stat::CriticalRating, 0)]).is_ok());
    }

    #[test]
    fn test_eight_per_family_paired_is_allowed() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        let mut names: Vec<String> = Vec::new();
        for i in 1..=8usize {
            let name = format!("Ring{}", i);
            let slot = if i % 2 == 0 {
                Slot::Finger1
            } else {
                Slot::Finger2
            };
            resolved.insert(
                name.clone(),
                make_cached(&name, slot, &[(Stat::CriticalRating, i as i64 * 10)]),
            );
            names.push(name);
        }
        assert!(optimize(&resolved, &names, &[goal(Stat::CriticalRating, 0)]).is_ok());
    }

    #[test]
    fn test_missing_slot_emits_placeholder_warning() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "Helm".into(),
            make_cached("Helm", Slot::Head, &[(Stat::CriticalRating, 100)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Helm".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("no candidates found")));
        assert_eq!(result.gear_set.items[&Slot::Head].name, "Helm");
    }

    #[test]
    fn test_negative_stat_compensated_across_slots() {
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
            make_cached("ItemX", Slot::Chest, &[(Stat::TacticalMitigation, 300)]),
        );
        let goals = vec![goal(Stat::TacticalMitigation, 200)];
        let result = optimize_ok(
            &resolved,
            &["ItemA".to_string(), "ItemX".to_string()],
            &goals,
        );
        assert!(result.feasible);
        assert_eq!(result.gear_set.total(&Stat::TacticalMitigation), 250);
        assert_eq!(result.gear_set.items[&Slot::Head].name, "ItemA");
    }

    #[test]
    fn test_negative_stat_causes_infeasibility() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "ItemA".into(),
            make_cached("ItemA", Slot::Head, &[(Stat::TacticalMitigation, -50)]),
        );
        let result = optimize_ok(
            &resolved,
            &["ItemA".to_string()],
            &[goal(Stat::TacticalMitigation, 1)],
        );
        assert!(!result.feasible);
        assert!(matches!(
            result
                .failed_minima
                .iter()
                .find(|(s, _, _)| *s == Stat::TacticalMitigation),
            Some((_, 1, achieved)) if *achieved < 0
        ));
    }

    #[test]
    fn test_negative_stat_on_non_goal_stat_does_not_crash() {
        let mut resolved: HashMap<String, CachedItem> = HashMap::new();
        resolved.insert(
            "ItemA".into(),
            make_cached(
                "ItemA",
                Slot::Head,
                &[(Stat::CriticalRating, 300), (Stat::Finesse, -999)],
            ),
        );
        let result = optimize_ok(
            &resolved,
            &["ItemA".to_string()],
            &[goal(Stat::CriticalRating, 300)],
        );
        assert!(result.feasible);
        assert_eq!(result.gear_set.items[&Slot::Head].name, "ItemA");
    }

    fn run_fuzzer_cases(case_count: usize, seed: u64) {
        let stats = [
            Stat::CriticalRating,
            Stat::TacticalMastery,
            Stat::Finesse,
            Stat::TacticalMitigation,
        ];
        let families = [
            Slot::Head,
            Slot::Chest,
            Slot::Legs,
            Slot::Wrist1,
            Slot::Finger1,
            Slot::Ear1,
        ];
        let mut rng = Lcg::new(seed);

        for case_idx in 0..case_count {
            let mut resolved: HashMap<String, CachedItem> = HashMap::new();
            let mut names = Vec::new();
            let family_count = rng.range_usize(2, 4);
            let mut selected_families = Vec::new();
            while selected_families.len() < family_count {
                let family = families[rng.range_usize(0, families.len() - 1)];
                if !selected_families.contains(&family) {
                    selected_families.push(family);
                }
            }

            let mut instance_idx = 0usize;
            for family in selected_families {
                let candidate_count = rng.range_usize(1, 4);
                for local_idx in 0..candidate_count {
                    let slot = match family {
                        Slot::Wrist1 if rng.chance(1, 2) => Slot::Wrist2,
                        Slot::Finger1 if rng.chance(1, 2) => Slot::Finger2,
                        Slot::Ear1 if rng.chance(1, 2) => Slot::Ear2,
                        other => other,
                    };
                    let name = format!("case{}_item{}_{}", case_idx, instance_idx, local_idx);
                    let key = instance_key(instance_idx, slot, &name);
                    instance_idx += 1;
                    let item_stats: Vec<(Stat, i64)> = stats
                        .iter()
                        .map(|stat| (*stat, rng.range_i64(-3, 15)))
                        .collect();
                    resolved.insert(key.clone(), make_cached(&name, slot, &item_stats));
                    names.push(key);
                }
            }

            let goal_count = rng.range_usize(1, 3);
            let mut goals = Vec::new();
            while goals.len() < goal_count {
                let stat = stats[rng.range_usize(0, stats.len() - 1)];
                if goals.iter().any(|goal: &StatGoal| goal.stat == stat) {
                    continue;
                }
                let minimum = if rng.chance(1, 5) {
                    0
                } else {
                    rng.range_i64(1, 20)
                };
                goals.push(goal(stat, minimum));
            }

            let production = optimize_ok(&resolved, &names, &goals);
            let oracle = oracle_optimize(&resolved, &names, &goals);
            let production_totals = result_goal_totals(&production, &goals);
            let oracle_totals = result_goal_totals(&oracle, &goals);
            if production.feasible != oracle.feasible
                || compare_builds(&production_totals, &oracle_totals, &goals) != Ordering::Equal
            {
                panic!(
                    "fuzzer mismatch case {case_idx}\ngoals: {:?}\nnames: {:?}\nresolved: {:#?}\nproduction feasible/totals: {:?} {:?}\noracle feasible/totals: {:?} {:?}",
                    goals,
                    names,
                    resolved,
                    production.feasible,
                    production_totals,
                    oracle.feasible,
                    oracle_totals,
                );
            }
        }
    }

    #[test]
    fn differential_fuzzer_matches_oracle_smoke() {
        run_fuzzer_cases(250, 0x5eed_1234_abcd_9876);
    }

    #[test]
    #[ignore]
    fn differential_fuzzer_matches_oracle_deep() {
        run_fuzzer_cases(5_000, 0x0dd5_ea51_5eed_f00d);
    }
}
