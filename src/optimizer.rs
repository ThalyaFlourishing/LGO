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

use crate::gear::{GearItem, GearSet, Slot};
use crate::stat::{Stat, StatGoal};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum supported candidates per canonical slot or paired-slot family.
/// Keeps paired-slot enumeration bounded (max 8×7/2 = 28 real pairs, plus
/// singleton empty-placeholder pairs).
pub const MAX_CANDIDATES_PER_SLOT: usize = 8;

/// Maximum supported combined real-item candidates across both hand slots:
/// Main-hand-only + Off-hand-only + Either-hand items. The two hand slots are
/// searched as one combined pool, so they share this single cap instead of the
/// per-slot cap that applies to every other slot and paired family.
pub const MAX_HAND_CANDIDATES_COMBINED: usize = 12;

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
    /// The two hand slots share one combined candidate cap. `count` is the
    /// number of Main-hand-only + Off-hand-only + Either-hand real items.
    TooManyHandCandidates { count: usize, max: usize },
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
            OptimizeError::TooManyHandCandidates { count, max } => write!(
                f,
                "Too many hand candidates: {} provided, maximum allowed is {}. The two hand slots share one combined cap, counting Main-hand-only, Off-hand-only, and Either-hand items together. Remove hand items from the 'lgo' chest and re-export before running 'lgo optimize'.",
                count, max
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
    /// True for two-handed `MainHand` weapons: legal only with an empty
    /// off-hand in the combined hand pool. Sourced from `GearItem.two_handed`
    /// (the optimizer never consults the items DB directly).
    two_handed: bool,
    /// True for Either-hand items: equippable in either the `MainHand` or the
    /// `OffHand` position. Sourced from `GearItem.either_hand`. Such items are
    /// resolved with `original_slot == OffHand`; the flag adds main-hand
    /// eligibility inside `build_hand_choices`.
    either_hand: bool,
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
            two_handed: false,
            either_hand: false,
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

/// A "super-candidate" for the combined hand pool: one legal
/// `MainHand` + `OffHand` configuration and its combined stats. Two-handed
/// main hands only ever appear here paired with the empty off-hand, which
/// is how off-hand suppression is enforced structurally rather than by a
/// search-time constraint.
#[derive(Debug, Clone)]
struct HandsCandidate {
    main: Candidate,
    off: Candidate,
    combined: HashMap<Stat, i64>,
}

impl HandsCandidate {
    fn new(main: Candidate, off: Candidate) -> Self {
        let mut combined: HashMap<Stat, i64> = main.stats.clone();
        for (s, v) in &off.stats {
            *combined.entry(*s).or_insert(0) += v;
        }
        HandsCandidate {
            main,
            off,
            combined,
        }
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
    /// One combined `MainHand` + `OffHand` configuration; slots are implied.
    Hands {
        hands: HandsCandidate,
    },
}

impl Choice {
    fn stat(&self, stat: &Stat) -> i64 {
        match self {
            Choice::Single { candidate, .. } => candidate.stat(stat),
            Choice::Pair { pair, .. } => pair.stat(stat),
            Choice::Hands { hands } => hands.stat(stat),
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
            Choice::Hands { hands } => {
                let mut keys = vec![hands.main.key.clone(), hands.off.key.clone()];
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
            Choice::Hands { hands } => {
                // A two-handed main hand carries the `[empty Off-hand]` zero
                // placeholder as its off component. The gear set stores that
                // placeholder as usual; the report renders the off-hand line
                // as `(2-handed item)` when the main hand is two-handed.
                gear_set.items.insert(
                    Slot::MainHand,
                    candidate_to_gear_item(&hands.main, Slot::MainHand),
                );
                gear_set.items.insert(
                    Slot::OffHand,
                    candidate_to_gear_item(&hands.off, Slot::OffHand),
                );
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
    resolved: &HashMap<String, GearItem>,
    candidates: &[String],
    goals: &[StatGoal],
    innate_stats: &HashMap<Stat, i64>,
) -> Result<OptimizeResult, Box<OptimizeError>> {
    let (pools, warnings) = build_search_pools(resolved, candidates, goals, true)?;
    let innate_goal_totals = innate_goal_totals(innate_stats, goals);
    let best = exact_search(&pools, goals, &innate_goal_totals).unwrap_or_else(|| SearchBuild {
        totals: innate_goal_totals.clone(),
        tiebreak_key: Vec::new(),
        choices: Vec::new(),
    });

    let mut gear_set = GearSet::new(innate_stats.clone());
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
    resolved: &HashMap<String, GearItem>,
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
            two_handed: cached.two_handed,
            either_hand: cached.either_hand,
        };
        pools.entry(canonical).or_default().push(cand);
    }

    validate_candidate_pool_sizes(&pools)?;

    for slot in Slot::all() {
        let canonical = canonical_slot(slot);
        // Hand slots get their empty candidates inside build_hand_choices
        // (pre-filling a zero here would duplicate them in the combined
        // pool); only the missing-candidates warning is kept.
        if canonical == Slot::MainHand || canonical == Slot::OffHand {
            if !pools.contains_key(&canonical) {
                warnings.push(format!(
                    "Slot {}: no candidates found; using zero placeholder.",
                    slot_display(slot)
                ));
            }
            continue;
        }
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

    for slot in Slot::all() {
        let canonical = canonical_slot(slot);
        if !seen.insert(canonical) {
            continue;
        }
        // The two hand slots form one combined pool, built when MainHand is
        // reached (it precedes OffHand in Slot::all()). The per-slot candidate
        // cap was already enforced on the source pools above; the combined
        // pool is deliberately allowed to exceed it pre-dominance.
        if canonical == Slot::OffHand {
            continue;
        }

        // Keep benchmark_candidate_caps in sync: its test-only harness mirrors this structural pipeline below the cap validation gate.
        let mut choices: Vec<Choice> = if canonical == Slot::MainHand {
            build_hand_choices(
                pools.get(&Slot::MainHand).map(Vec::as_slice).unwrap_or(&[]),
                pools.get(&Slot::OffHand).map(Vec::as_slice).unwrap_or(&[]),
            )
            .into_iter()
            .map(|hands| Choice::Hands { hands })
            .collect()
        } else if paired_canonicals.contains(&canonical) {
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

fn exact_search(
    pools: &[SearchPool],
    goals: &[StatGoal],
    initial_totals: &[i64],
) -> Option<SearchBuild> {
    if pools.iter().any(|pool| pool.choices.is_empty()) {
        return None;
    }

    let suffix_maxima = suffix_goal_maxima(pools, goals);
    let mut best: Option<SearchBuild> = None;
    let mut running_totals = initial_totals.to_vec();
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

fn innate_goal_totals(innate_stats: &HashMap<Stat, i64>, goals: &[StatGoal]) -> Vec<i64> {
    goals
        .iter()
        .map(|goal| innate_stats.get(&goal.stat).copied().unwrap_or(0))
        .collect()
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

/// Enumerate every legal `MainHand` + `OffHand` configuration under the
/// hand-slot semantics.
///
/// Item eligibility by position:
/// - Main-hand position: main-hand-only items (incl. two-handed) ∪ Either-hand
///   items.
/// - Off-hand position: off-hand-only items ∪ Either-hand items.
///
/// Either-hand items arrive in `off_pool` with `either_hand = true` (their
/// resolved slot is `Off-hand`); they are also eligible for the main position.
///
/// Rules enforced:
/// - A single owned instance can never fill both hands at once (compared by
///   `key`); two owned copies of the same Either-hand item may dual-wield.
/// - A two-handed main occupies both positions and pairs only with the empty
///   off-hand placeholder.
/// - **Real items required:** the empty-hand placeholder for a position is
///   emitted only when no real item is available for that position given the
///   other hand's assignment. On an exact stat tie a real item therefore
///   always beats the placeholder, because the placeholder is not offered
///   whenever a real item can fill the hand.
fn build_hand_choices(main_pool: &[Candidate], off_pool: &[Candidate]) -> Vec<HandsCandidate> {
    let empty_main = Candidate::zero(
        format!("[empty {}]", slot_display(Slot::MainHand)),
        Slot::MainHand,
    );
    let empty_off = Candidate::zero(
        format!("[empty {}]", slot_display(Slot::OffHand)),
        Slot::OffHand,
    );

    // Main-hand-eligible items: everything in the main pool plus Either-hand
    // items (which live in the off pool). Off-hand-eligible items: the off
    // pool as-is (off-hand-only items plus Either-hand items).
    let main_eligible: Vec<&Candidate> = main_pool
        .iter()
        .chain(off_pool.iter().filter(|c| c.either_hand))
        .collect();
    let off_eligible: Vec<&Candidate> = off_pool.iter().collect();

    let mut hands = Vec::new();

    // Configurations that place a real item in the main hand.
    for main in &main_eligible {
        if main.two_handed {
            // A two-handed weapon occupies both hands: only the structural
            // empty off-hand pairs with it.
            hands.push(HandsCandidate::new((*main).clone(), empty_off.clone()));
            continue;
        }
        // Real off-hand items usable alongside this main (not the same owned
        // instance).
        let mut paired_any = false;
        for off in &off_eligible {
            if off.key == main.key {
                continue;
            }
            hands.push(HandsCandidate::new((*main).clone(), (*off).clone()));
            paired_any = true;
        }
        // Empty off-hand is offered only when no real off-hand item is
        // available for this main (real-items-required).
        if !paired_any {
            hands.push(HandsCandidate::new((*main).clone(), empty_off.clone()));
        }
    }

    // Configurations that leave the main hand empty: legal only when no
    // main-eligible item is available given the off-hand's assignment.
    for off in &off_eligible {
        let main_available = main_eligible.iter().any(|m| m.key != off.key);
        if !main_available {
            hands.push(HandsCandidate::new(empty_main.clone(), (*off).clone()));
        }
    }

    // Both hands empty: only when neither position has any real item.
    if main_eligible.is_empty() && off_eligible.is_empty() {
        hands.push(HandsCandidate::new(empty_main, empty_off));
    }

    hands
}

fn candidate_to_gear_item(c: &Candidate, slot: Slot) -> GearItem {
    GearItem {
        name: c.name.clone(),
        slot,
        two_handed: c.two_handed,
        either_hand: c.either_hand,
        stats: c.stats.clone(),
    }
}

fn slot_display(slot: Slot) -> &'static str {
    slot.display_name()
}

fn validate_candidate_pool_sizes(
    pools: &HashMap<Slot, Vec<Candidate>>,
) -> Result<(), Box<OptimizeError>> {
    let mut seen = HashSet::new();

    for slot in Slot::all() {
        let canonical = canonical_slot(slot);
        if !seen.insert(canonical) {
            continue;
        }

        // The two hand slots share a single combined cap enforced below; the
        // per-slot cap does not apply to them.
        if canonical == Slot::MainHand || canonical == Slot::OffHand {
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

    // Combined hand cap: Main-hand-only + Off-hand-only + Either-hand real
    // items across both hand slots. Either-hand items are counted once, in the
    // Off-hand pool where they are resolved.
    let hand_count = pools.get(&Slot::MainHand).map_or(0, Vec::len)
        + pools.get(&Slot::OffHand).map_or(0, Vec::len);
    if hand_count > MAX_HAND_CANDIDATES_COMBINED {
        return Err(Box::new(OptimizeError::TooManyHandCandidates {
            count: hand_count,
            max: MAX_HAND_CANDIDATES_COMBINED,
        }));
    }

    Ok(())
}

// ────────── TESTS ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stat::TRACKED_STATS;
    use std::any::Any;
    use std::env;
    use std::panic::AssertUnwindSafe;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    // benchmark_candidate_caps:
    // Edit this single constant to raise or lower the per-run cutoff. The limit
    // applies to each individual optimizer search, not to the whole harness.
    const TIME_LIMIT_SECS: u64 = 100;
    const MAX_N: usize = 10;

    const BENCHMARK_GOALS: [(Stat, i64); 9] = [        
        (Stat::Morale, 20_000),
        (Stat::CriticalRating, 280_000),
        (Stat::TacticalMastery, 20_000),
        (Stat::PhysicalMastery, 120_000),
        (Stat::TacticalMitigation, 200_000),
        (Stat::Evade, 200_000),
        (Stat::Block, 200_000),
        (Stat::PhysicalMitigation, 200_000),
        (Stat::OutgoingHealing, 200_000),
    ];

    const BENCHMARK_SINGLETON_SLOTS: [Slot; 11] = [
        Slot::Head,
        Slot::Chest,
        Slot::Legs,
        Slot::Hands,
        Slot::Feet,
        Slot::Shoulders,
        Slot::Back,
        Slot::Neck,
        Slot::Pocket,
        Slot::Ranged,
        Slot::ClassItem,
    ];

    const BENCHMARK_PAIRED_FAMILIES: [(Slot, Slot); 3] = [
        (Slot::Wrist1, Slot::Wrist2),
        (Slot::Finger1, Slot::Finger2),
        (Slot::Ear1, Slot::Ear2),
    ];

    #[derive(Clone, Copy, Debug)]
    enum BenchmarkProfile {
        Uniform,
        SinglesAtN,
        PairsAtN,
        HandsAtN,
    }

    impl BenchmarkProfile {
        fn all() -> [Self; 4] {
            [
                Self::Uniform,
                Self::SinglesAtN,
                Self::PairsAtN,
                Self::HandsAtN,
            ]
        }

        fn label(self) -> &'static str {
            match self {
                Self::Uniform => "Uniform",
                Self::SinglesAtN => "Singles-at-N",
                Self::PairsAtN => "Pairs-at-N",
                Self::HandsAtN => "Hands-at-N",
            }
        }

        fn env_label(self) -> &'static str {
            match self {
                Self::Uniform => "Uniform",
                Self::SinglesAtN => "Singles",
                Self::PairsAtN => "Pairs",
                Self::HandsAtN => "Hands",
            }
        }

        fn singleton_count(self, n: usize) -> usize {
            match self {
                Self::Uniform | Self::SinglesAtN => n,
                Self::PairsAtN | Self::HandsAtN => MAX_CANDIDATES_PER_SLOT,
            }
        }

        fn paired_count(self, n: usize) -> usize {
            match self {
                Self::Uniform | Self::PairsAtN => n,
                Self::SinglesAtN | Self::HandsAtN => MAX_CANDIDATES_PER_SLOT,
            }
        }

        fn hand_count(self, n: usize) -> usize {
            match self {
                Self::Uniform | Self::HandsAtN => n,
                Self::SinglesAtN | Self::PairsAtN => MAX_HAND_CANDIDATES_COMBINED,
            }
        }
    }

    #[derive(Debug)]
    struct BenchmarkRun {
        raw_pool_sizes: Vec<(String, usize)>,
        post_pool_sizes: Vec<(String, usize)>,
        pair_super_counts: Vec<(String, usize)>,
        hand_configuration_count: usize,
        wall_time: BenchmarkWallTime,
    }

    #[derive(Debug)]
    enum BenchmarkWallTime {
        Completed(Duration),
        TimedOut,
    }

    enum BenchmarkWorkerResult {
        Completed {
            elapsed: Duration,
            found_build: bool,
        },
        Panicked(String),
    }

    impl BenchmarkWallTime {
        fn completed_within_limit(&self) -> bool {
            matches!(self, Self::Completed(_))
        }

        fn display(&self) -> String {
            match self {
                Self::Completed(elapsed) => format!("{:.3}s", elapsed.as_secs_f64()),
                Self::TimedOut => format!(">{TIME_LIMIT_SECS}s"),
            }
        }
    }

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

    fn make_cached(name: &str, slot: Slot, stats: &[(Stat, i64)]) -> GearItem {
        GearItem {
            name: name.to_string(),
            slot,
            two_handed: false,
            either_hand: false,
            stats: stats.iter().copied().collect(),
        }
    }

    fn make_cached_2h(name: &str, stats: &[(Stat, i64)]) -> GearItem {
        GearItem {
            name: name.to_string(),
            slot: Slot::MainHand,
            two_handed: true,
            either_hand: false,
            stats: stats.iter().copied().collect(),
        }
    }

    // An Either-hand item: it lives in the Off-hand slot but may be worn in
    // either hand position.
    fn make_cached_either(name: &str, stats: &[(Stat, i64)]) -> GearItem {
        GearItem {
            name: name.to_string(),
            slot: Slot::OffHand,
            two_handed: false,
            either_hand: true,
            stats: stats.iter().copied().collect(),
        }
    }

    fn instance_key(idx: usize, slot: Slot, name: &str) -> String {
        crate::gear::optimizer_candidate_key(
            idx,
            &GearItem {
                name: name.to_string(),
                slot,
                two_handed: false,
                either_hand: false,
                stats: HashMap::new(),
            },
        )
    }

    fn goal(stat: Stat, minimum: i64) -> StatGoal {
        StatGoal { stat, minimum }
    }

    fn optimize_ok(
        resolved: &HashMap<String, GearItem>,
        candidates: &[String],
        goals: &[StatGoal],
    ) -> OptimizeResult {
        optimize(resolved, candidates, goals, &HashMap::new()).expect("optimization should succeed")
    }

    fn single_slot_result(
        resolved: &HashMap<String, GearItem>,
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
        resolved: &HashMap<String, GearItem>,
        candidates: &[String],
        goals: &[StatGoal],
    ) -> OptimizeResult {
        let (pools, warnings) = build_search_pools(resolved, candidates, goals, false)
            .expect("oracle inputs should be within candidate cap");
        let best = oracle_best_build(&pools, goals).expect("oracle pools should not be empty");
        let mut gear_set = GearSet::new(HashMap::new());
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
        // Stage 2: once both builds meet CR, compare the clamped TM contribution.
        let goals = vec![
            goal(Stat::CriticalRating, 100_000),
            goal(Stat::TacticalMastery, 100_000),
        ];
        assert_x_beats_y(&[120_000, 90_000], &[100_000, 89_999], &goals);
    }

    #[test]
    fn comparator_worked_example_ratchet_protects_met_lower_goal() {
        // Stage 1 ratchet: a build that already meets TM beats one that drops it, even with less CR.
        let goals = vec![
            goal(Stat::CriticalRating, 100),
            goal(Stat::TacticalMastery, 100),
        ];
        assert_x_beats_y(&[94, 100], &[96, 80], &goals);
    }

    #[test]
    fn comparator_worked_example_meeting_priority_one_justifies_drop() {
        // Stage 1: meeting the higher-priority CR minimum outranks missing it to keep more TM.
        let goals = vec![
            goal(Stat::CriticalRating, 100),
            goal(Stat::TacticalMastery, 100),
        ];
        assert_x_beats_y(&[100, 70], &[98, 100], &goals);
    }

    #[test]
    fn comparator_worked_example_both_unmet_stage2_priority_order() {
        // Stage 2: with both minima unmet, the earlier goal still wins 95/100 over 94/100.
        let goals = vec![
            goal(Stat::CriticalRating, 100),
            goal(Stat::TacticalMastery, 100),
        ];
        assert_x_beats_y(&[95, 96], &[94, 97], &goals);
    }

    #[test]
    fn comparator_worked_example_all_met_stage3_raw_polish() {
        // Stage 3: after all minima are met, raw overflow decides the winner.
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
        // C2 is the first candidate to meet all four minima exactly enough; extra CR/TT elsewhere does not matter.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // Neither candidate is feasible, so the comparator prefers the stronger first-goal progress on CR.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
    fn innate_stats_seed_search_goal_totals() {
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "A".into(),
            make_cached("A", Slot::Chest, &[(Stat::CriticalRating, 50)]),
        );
        resolved.insert(
            "B".into(),
            make_cached("B", Slot::Chest, &[(Stat::TacticalMastery, 90)]),
        );
        let goals = vec![
            goal(Stat::CriticalRating, 100),
            goal(Stat::TacticalMastery, 100),
        ];
        let innate_stats = [(Stat::CriticalRating, 100)].into_iter().collect();

        let result = optimize(
            &resolved,
            &["A".to_string(), "B".to_string()],
            &goals,
            &innate_stats,
        )
        .expect("optimization should succeed");

        assert_eq!(
            result
                .gear_set
                .items
                .get(&Slot::Chest)
                .map(|item| item.name.as_str()),
            Some("B")
        );
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 100);
        assert_eq!(result.gear_set.total(&Stat::TacticalMastery), 90);
    }

    #[test]
    fn test_c5_over_c4_same_slot() {
        // Both meet CR/FN/TT, so hitting the TM minimum with C5 beats C4's extra CR.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // Dominance filtering must keep the build that preserves the later-goal ratchet, then match the brute-force oracle.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // Exact search must honor Stage 1 minima before Stage 3 overflow, so 10/10 beats 100/0.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // Paired slots consume two distinct wrists and total 100 + 80 = 180 exactly once each.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // One wrist item may satisfy the minimum alone, but it cannot be duplicated into the second wrist slot.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // The best legal ring pair is 500 + 300 = 800, so a 900 minimum stays infeasible without self-pairing.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // Duplicate display names are fine when the instance keys are distinct; both 500-value rings can be worn together.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        let mut one: HashMap<String, GearItem> = HashMap::new();
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
        let mut one: HashMap<String, GearItem> = HashMap::new();
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
        // Raising the minimum from 800 to 801 keeps the same best pair but flips feasibility.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // Equality counts as satisfied: 300 TMit meets a 300 minimum.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // Falling short by one point should report the exact 300 vs 299 deficit.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        let mut names: Vec<String> = Vec::new();
        for i in 1..=9usize {
            let name = format!("Head{}", i);
            resolved.insert(
                name.clone(),
                make_cached(&name, Slot::Head, &[(Stat::CriticalRating, i as i64 * 10)]),
            );
            names.push(name);
        }
        let err = optimize(
            &resolved,
            &names,
            &[goal(Stat::CriticalRating, 0)],
            &HashMap::new(),
        )
        .unwrap_err();
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
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        let err = optimize(
            &resolved,
            &names,
            &[goal(Stat::CriticalRating, 0)],
            &HashMap::new(),
        )
        .unwrap_err();
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
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        let mut names: Vec<String> = Vec::new();
        for i in 1..=8usize {
            let name = format!("Head{}", i);
            resolved.insert(
                name.clone(),
                make_cached(&name, Slot::Head, &[(Stat::CriticalRating, i as i64 * 10)]),
            );
            names.push(name);
        }
        assert!(optimize(
            &resolved,
            &names,
            &[goal(Stat::CriticalRating, 0)],
            &HashMap::new()
        )
        .is_ok());
    }

    #[test]
    fn test_eight_per_family_paired_is_allowed() {
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        assert!(optimize(
            &resolved,
            &names,
            &[goal(Stat::CriticalRating, 0)],
            &HashMap::new()
        )
        .is_ok());
    }

    #[test]
    fn test_missing_slot_emits_placeholder_warning() {
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // Negative contributions still add algebraically: -50 + 300 = 250, so another slot can compensate.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // A negative goal stat may drive the achieved total below zero and must surface as an unmet minimum.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
        // Off-goal negatives are ignored by the objective but must still pass through the optimizer safely.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
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

    // ── Two-handed hand-pool tests ────────────────────────────────────────────

    #[test]
    fn test_two_handed_main_suppresses_attractive_off_hand() {
        // The shield's 999 CR would beat the greatsword's lone 1000 if it
        // could combine with it (1999); a two-handed main must never take a
        // real off-hand, so the best legal build is greatsword + empty off.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Greatsword".into(),
            make_cached_2h("Greatsword", &[(Stat::CriticalRating, 1000)]),
        );
        resolved.insert(
            "Shield".into(),
            make_cached("Shield", Slot::OffHand, &[(Stat::CriticalRating, 999)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Greatsword".to_string(), "Shield".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(result.gear_set.items[&Slot::MainHand].name, "Greatsword");
        assert_eq!(
            result.gear_set.items[&Slot::OffHand].name,
            "[empty Off-hand]",
            "two-handed main must block the off-hand slot"
        );
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 1000);
    }

    #[test]
    fn test_one_handed_main_combines_with_off_hand() {
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Sword".into(),
            make_cached("Sword", Slot::MainHand, &[(Stat::CriticalRating, 100)]),
        );
        resolved.insert(
            "Shield".into(),
            make_cached("Shield", Slot::OffHand, &[(Stat::CriticalRating, 50)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Sword".to_string(), "Shield".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(result.gear_set.items[&Slot::MainHand].name, "Sword");
        assert_eq!(result.gear_set.items[&Slot::OffHand].name, "Shield");
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 150);
    }

    #[test]
    fn test_prefers_one_handed_plus_off_hand_when_stats_better() {
        // 1H (100) + off (50) = 150 beats 2H (120) + blocked off.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Greatsword".into(),
            make_cached_2h("Greatsword", &[(Stat::CriticalRating, 120)]),
        );
        resolved.insert(
            "Sword".into(),
            make_cached("Sword", Slot::MainHand, &[(Stat::CriticalRating, 100)]),
        );
        resolved.insert(
            "Shield".into(),
            make_cached("Shield", Slot::OffHand, &[(Stat::CriticalRating, 50)]),
        );
        let result = optimize_ok(
            &resolved,
            &[
                "Greatsword".to_string(),
                "Sword".to_string(),
                "Shield".to_string(),
            ],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(result.gear_set.items[&Slot::MainHand].name, "Sword");
        assert_eq!(result.gear_set.items[&Slot::OffHand].name, "Shield");
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 150);
    }

    #[test]
    fn test_prefers_two_handed_when_stats_better() {
        // 2H (200) beats 1H (100) + off (50) = 150.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Greatsword".into(),
            make_cached_2h("Greatsword", &[(Stat::CriticalRating, 200)]),
        );
        resolved.insert(
            "Sword".into(),
            make_cached("Sword", Slot::MainHand, &[(Stat::CriticalRating, 100)]),
        );
        resolved.insert(
            "Shield".into(),
            make_cached("Shield", Slot::OffHand, &[(Stat::CriticalRating, 50)]),
        );
        let result = optimize_ok(
            &resolved,
            &[
                "Greatsword".to_string(),
                "Sword".to_string(),
                "Shield".to_string(),
            ],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(result.gear_set.items[&Slot::MainHand].name, "Greatsword");
        assert_eq!(
            result.gear_set.items[&Slot::OffHand].name,
            "[empty Off-hand]"
        );
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 200);
    }

    #[test]
    fn test_real_main_required_even_when_it_hurts_the_goal() {
        // Real-items-required: the empty main-hand placeholder is selectable
        // only when the main position has no eligible real item. Here the sole
        // main candidate is a two-handed weapon that hurts the goal, but it
        // must still be equipped (blocking the off-hand) rather than left empty
        // in favor of the shield. (Updated for Task 2: the old behavior let the
        // main hand go empty so the shield could be worn.)
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Cursed Greatsword".into(),
            make_cached_2h("Cursed Greatsword", &[(Stat::CriticalRating, -10)]),
        );
        resolved.insert(
            "Shield".into(),
            make_cached("Shield", Slot::OffHand, &[(Stat::CriticalRating, 100)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Cursed Greatsword".to_string(), "Shield".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(
            result.gear_set.items[&Slot::MainHand].name,
            "Cursed Greatsword"
        );
        assert_eq!(
            result.gear_set.items[&Slot::OffHand].name,
            "[empty Off-hand]",
            "a two-handed main blocks the off-hand even when a shield exists"
        );
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), -10);
    }

    #[test]
    fn test_one_handed_with_negative_off_hand_still_requires_real_off() {
        // Real-items-required: when a real off-hand item exists it must be worn
        // rather than leaving the off-hand empty, even if it hurts the goal.
        // (Updated for Task 2: the old behavior preferred the empty off-hand.)
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Sword".into(),
            make_cached("Sword", Slot::MainHand, &[(Stat::CriticalRating, 100)]),
        );
        resolved.insert(
            "Cursed Shield".into(),
            make_cached(
                "Cursed Shield",
                Slot::OffHand,
                &[(Stat::CriticalRating, -40)],
            ),
        );
        let result = optimize_ok(
            &resolved,
            &["Sword".to_string(), "Cursed Shield".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(result.gear_set.items[&Slot::MainHand].name, "Sword");
        assert_eq!(
            result.gear_set.items[&Slot::OffHand].name,
            "Cursed Shield",
            "a real off-hand must be worn rather than left empty"
        );
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 60);
    }

    #[test]
    fn test_combined_hand_pool_of_twelve_is_allowed() {
        // The two hand slots share one combined cap of 12. Six main-hand items
        // plus six off-hand items is exactly at the cap and must be accepted.
        // (Updated for Task 3: the old per-slot cap of 8 no longer applies to
        // the hand slots.)
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        let mut names: Vec<String> = Vec::new();
        for i in 1..=6usize {
            let main = format!("Main{}", i);
            let cached = if i % 2 == 0 {
                make_cached_2h(&main, &[(Stat::CriticalRating, i as i64 * 10)])
            } else {
                make_cached(
                    &main,
                    Slot::MainHand,
                    &[(Stat::CriticalRating, i as i64 * 10)],
                )
            };
            resolved.insert(main.clone(), cached);
            names.push(main);
            let off = format!("Off{}", i);
            resolved.insert(
                off.clone(),
                make_cached(&off, Slot::OffHand, &[(Stat::CriticalRating, i as i64)]),
            );
            names.push(off);
        }
        let result = optimize(
            &resolved,
            &names,
            &[goal(Stat::CriticalRating, 0)],
            &HashMap::new(),
        );
        assert!(
            result.is_ok(),
            "12 combined hand candidates must be within the cap"
        );
        // Best: Main6 (60, two-handed) beats Main5 (50, 1H) + Off6 (6) = 56.
        let result = result.unwrap();
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 60);
    }

    #[test]
    fn test_thirteen_combined_hand_candidates_is_refused() {
        // Thirteen real hand items (7 main-hand + 6 off-hand) exceed the
        // combined cap of 12 and must be refused with a message stating the
        // count, the cap, and the counted categories. (Updated for Task 3: the
        // old test refused 9 main-hand items under the per-slot cap.)
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        let mut names: Vec<String> = Vec::new();
        for i in 1..=7usize {
            let name = format!("Main{}", i);
            resolved.insert(
                name.clone(),
                make_cached(&name, Slot::MainHand, &[(Stat::CriticalRating, i as i64)]),
            );
            names.push(name);
        }
        for i in 1..=6usize {
            let name = format!("Off{}", i);
            resolved.insert(
                name.clone(),
                make_cached(&name, Slot::OffHand, &[(Stat::CriticalRating, i as i64)]),
            );
            names.push(name);
        }
        let err = optimize(
            &resolved,
            &names,
            &[goal(Stat::CriticalRating, 0)],
            &HashMap::new(),
        )
        .unwrap_err();
        assert_eq!(
            *err,
            OptimizeError::TooManyHandCandidates { count: 13, max: 12 }
        );
        let message = err.to_string();
        assert!(
            message.contains("13"),
            "message states the count: {message}"
        );
        assert!(message.contains("12"), "message states the cap: {message}");
        assert!(
            message.contains("Main-hand-only")
                && message.contains("Off-hand-only")
                && message.contains("Either-hand"),
            "message names the counted categories: {message}"
        );
    }

    #[test]
    fn test_missing_hand_slots_emit_placeholder_warnings() {
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Helm".into(),
            make_cached("Helm", Slot::Head, &[(Stat::CriticalRating, 100)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Helm".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        for label in ["Main-hand", "Off-hand"] {
            assert!(
                result
                    .warnings
                    .iter()
                    .any(|w| w.contains(label) && w.contains("no candidates found")),
                "expected placeholder warning for {}: {:?}",
                label,
                result.warnings
            );
        }
        assert_eq!(
            result.gear_set.items[&Slot::MainHand].name,
            "[empty Main-hand]"
        );
        assert_eq!(
            result.gear_set.items[&Slot::OffHand].name,
            "[empty Off-hand]"
        );
    }

    // ── Either-hand hand-pool tests (Task 2) ──────────────────────────────────

    #[test]
    fn test_either_hand_item_selectable_in_main_hand() {
        // An Either-hand item paired with an off-hand-only item is best worn in
        // the main hand so both real items can be equipped together.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Versatile Mace".into(),
            make_cached_either("Versatile Mace", &[(Stat::CriticalRating, 100)]),
        );
        resolved.insert(
            "Shield".into(),
            make_cached("Shield", Slot::OffHand, &[(Stat::CriticalRating, 50)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Versatile Mace".to_string(), "Shield".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(
            result.gear_set.items[&Slot::MainHand].name,
            "Versatile Mace"
        );
        assert_eq!(result.gear_set.items[&Slot::OffHand].name, "Shield");
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 150);
    }

    #[test]
    fn test_either_hand_item_selectable_in_off_hand() {
        // A main-hand-only item alongside an Either-hand item pushes the
        // Either-hand item into the off-hand so both are equipped.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Sword".into(),
            make_cached("Sword", Slot::MainHand, &[(Stat::CriticalRating, 100)]),
        );
        resolved.insert(
            "Versatile Mace".into(),
            make_cached_either("Versatile Mace", &[(Stat::CriticalRating, 50)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Sword".to_string(), "Versatile Mace".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(result.gear_set.items[&Slot::MainHand].name, "Sword");
        assert_eq!(result.gear_set.items[&Slot::OffHand].name, "Versatile Mace");
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 150);
    }

    #[test]
    fn test_one_either_hand_instance_cannot_dual_wield_but_two_copies_can() {
        // A single owned Either-hand instance fills only one hand, so a goal
        // that needs both hands' worth is infeasible; two owned copies with
        // distinct instance keys may dual-wield to reach it.
        let goals = vec![goal(Stat::CriticalRating, 200)];

        let mut one: HashMap<String, GearItem> = HashMap::new();
        one.insert(
            instance_key(1, Slot::OffHand, "Versatile Mace"),
            make_cached_either("Versatile Mace", &[(Stat::CriticalRating, 100)]),
        );
        let one_names = vec![instance_key(1, Slot::OffHand, "Versatile Mace")];
        let one_result = optimize_ok(&one, &one_names, &goals);
        assert!(!one_result.feasible, "one instance cannot fill both hands");
        assert_eq!(one_result.gear_set.total(&Stat::CriticalRating), 100);

        let mut two = one;
        two.insert(
            instance_key(2, Slot::OffHand, "Versatile Mace"),
            make_cached_either("Versatile Mace", &[(Stat::CriticalRating, 100)]),
        );
        let two_names = vec![
            instance_key(1, Slot::OffHand, "Versatile Mace"),
            instance_key(2, Slot::OffHand, "Versatile Mace"),
        ];
        let two_result = optimize_ok(&two, &two_names, &goals);
        assert!(two_result.feasible, "two copies may dual-wield");
        assert_eq!(
            two_result.gear_set.items[&Slot::MainHand].name,
            "Versatile Mace"
        );
        assert_eq!(
            two_result.gear_set.items[&Slot::OffHand].name,
            "Versatile Mace"
        );
        assert_eq!(two_result.gear_set.total(&Stat::CriticalRating), 200);
    }

    #[test]
    fn test_all_zero_real_main_hand_beats_empty_placeholder_on_tie() {
        // A real main-hand item contributing nothing still beats the empty
        // placeholder: real-items-required means the placeholder is not even
        // offered while a real main exists, so the build shows the real item.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Blank Sword".into(),
            make_cached("Blank Sword", Slot::MainHand, &[(Stat::CriticalRating, 0)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Blank Sword".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(result.gear_set.items[&Slot::MainHand].name, "Blank Sword");
        assert_eq!(
            result.gear_set.items[&Slot::OffHand].name,
            "[empty Off-hand]"
        );
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 0);
    }

    #[test]
    fn test_empty_placeholder_selected_when_hand_pool_is_truly_empty() {
        // With no main-hand-eligible item at all, the empty main-hand
        // placeholder becomes selectable so a real off-hand item can be worn.
        let mut resolved: HashMap<String, GearItem> = HashMap::new();
        resolved.insert(
            "Shield".into(),
            make_cached("Shield", Slot::OffHand, &[(Stat::CriticalRating, 40)]),
        );
        let result = optimize_ok(
            &resolved,
            &["Shield".to_string()],
            &[goal(Stat::CriticalRating, 0)],
        );
        assert_eq!(
            result.gear_set.items[&Slot::MainHand].name,
            "[empty Main-hand]"
        );
        assert_eq!(result.gear_set.items[&Slot::OffHand].name, "Shield");
        assert_eq!(result.gear_set.total(&Stat::CriticalRating), 40);
    }

    fn benchmark_goal_set() -> Vec<StatGoal> {
        BENCHMARK_GOALS
            .iter()
            .map(|(stat, minimum)| goal(*stat, *minimum))
            .collect()
    }

    fn benchmark_pool_label(slot: Slot) -> String {
        match slot {
            Slot::Head => "Hd".to_string(),
            Slot::Chest => "Ch".to_string(),
            Slot::Legs => "Lg".to_string(),
            Slot::Hands => "Ha".to_string(),
            Slot::Feet => "Ft".to_string(),
            Slot::Shoulders => "Sh".to_string(),
            Slot::Back => "Bk".to_string(),
            Slot::Wrist1 => "Wr".to_string(),
            Slot::Wrist2 => "Wr".to_string(),
            Slot::Neck => "Nk".to_string(),
            Slot::Finger1 => "Fi".to_string(),
            Slot::Finger2 => "Fi".to_string(),
            Slot::Ear1 => "Er".to_string(),
            Slot::Ear2 => "Er".to_string(),
            Slot::Pocket => "Po".to_string(),
            Slot::MainHand => "MH+OH".to_string(),
            Slot::OffHand => "OH".to_string(),
            Slot::Ranged => "Ra".to_string(),
            Slot::ClassItem => "Ci".to_string(),
        }
    }

    fn make_benchmark_candidate(
        family: &str,
        frontier_idx: usize,
        original_slot: Slot,
        two_handed: bool,
        either_hand: bool,
    ) -> Candidate {
        let mut stats: HashMap<Stat, i64> = HashMap::new();
        let primary_stat = TRACKED_STATS[frontier_idx % TRACKED_STATS.len()].0;

        for (stat_idx, (stat, _)) in TRACKED_STATS.iter().enumerate() {
            let mut value = ((frontier_idx + stat_idx * 5) % 5) as i64;
            if *stat == Stat::CriticalRating {
                value += 20_000 - 200 * frontier_idx as i64;
            }
            if *stat == Stat::TacticalMastery {
                value += 2_000 + 200 * frontier_idx as i64;
            }
            if *stat == Stat::Finesse {
                value += 10_000 + (frontier_idx % 17) as i64;
            }
            if *stat == primary_stat {
                value += if matches!(
                    primary_stat,
                    Stat::CriticalRating | Stat::TacticalMastery | Stat::Finesse
                ) {
                    50
                } else {
                    8_000
                };
            }
            if value != 0 {
                stats.insert(*stat, value);
            }
        }

        let name = format!("{family}-{frontier_idx:03}");
        Candidate {
            key: format!(
                "bench::{family}::{frontier_idx:04}::{}::{}",
                original_slot.display_name(),
                name
            ),
            name,
            stats,
            original_slot,
            two_handed,
            either_hand,
        }
    }

    fn make_benchmark_pool(family: &str, slot: Slot, count: usize) -> Vec<Candidate> {
        (0..count)
            .map(|idx| make_benchmark_candidate(family, idx, slot, false, false))
            .collect()
    }

    fn make_benchmark_paired_pool(
        family: &str,
        slot1: Slot,
        slot2: Slot,
        count: usize,
    ) -> Vec<Candidate> {
        (0..count)
            .map(|idx| {
                let slot = if idx % 2 == 0 { slot1 } else { slot2 };
                make_benchmark_candidate(family, idx, slot, false, false)
            })
            .collect()
    }

    fn make_benchmark_hand_pools(count: usize) -> (Vec<Candidate>, Vec<Candidate>) {
        assert!(
            count >= 8,
            "hand benchmark starts at N = 8 and needs enough items to mix main-hand-only, off-hand-only, and Either-hand categories"
        );

        let main_only = usize::max(2, count / 4);
        let off_only = usize::max(2, count / 4);
        let either = count.checked_sub(main_only + off_only).unwrap_or_else(|| {
            panic!(
                "hand benchmark category split exceeded total candidate count: count={count}, main_only={main_only}, off_only={off_only}"
            )
        });
        assert!(
            either >= 2,
            "hand benchmark needs Either-hand items to exercise the current hand semantics"
        );

        let main_pool: Vec<Candidate> = (0..main_only)
            .map(|idx| make_benchmark_candidate("main", idx, Slot::MainHand, idx % 2 == 0, false))
            .collect();

        let off_only_pool = (0..off_only)
            .map(|idx| {
                make_benchmark_candidate("off", main_only + idx, Slot::OffHand, false, false)
            })
            .collect::<Vec<_>>();

        let either_pool = (0..either)
            .map(|idx| {
                make_benchmark_candidate(
                    "either",
                    main_only + off_only + idx,
                    Slot::OffHand,
                    false,
                    true,
                )
            })
            .collect::<Vec<_>>();

        let mut off_pool = off_only_pool;
        off_pool.extend(either_pool);
        assert_eq!(main_pool.len() + off_pool.len(), count);

        (main_pool, off_pool)
    }

    fn assert_frontier_survives_dominance(label: &str, pool: &[Candidate], goals: &[StatGoal]) {
        let raw_choices = pool
            .iter()
            .cloned()
            .map(|candidate| Choice::Single {
                slot: canonical_slot(candidate.original_slot),
                candidate,
            })
            .collect();
        let kept = dominance_filter(raw_choices, goals);
        assert_eq!(
            kept.len(),
            pool.len(),
            "benchmark pool {label} collapsed under dominance; the generated frontier is invalid"
        );
    }

    fn benchmark_raw_pools(profile: BenchmarkProfile, n: usize) -> HashMap<Slot, Vec<Candidate>> {
        let mut pools = HashMap::new();

        for slot in BENCHMARK_SINGLETON_SLOTS {
            pools.insert(
                slot,
                make_benchmark_pool(
                    benchmark_pool_label(slot).as_str(),
                    slot,
                    profile.singleton_count(n),
                ),
            );
        }

        for (slot1, slot2) in BENCHMARK_PAIRED_FAMILIES {
            pools.insert(
                slot1,
                make_benchmark_paired_pool(
                    benchmark_pool_label(slot1).as_str(),
                    slot1,
                    slot2,
                    profile.paired_count(n),
                ),
            );
        }

        let (main_pool, off_pool) = make_benchmark_hand_pools(profile.hand_count(n));
        pools.insert(Slot::MainHand, main_pool);
        pools.insert(Slot::OffHand, off_pool);

        pools
    }

    fn format_benchmark_counts(entries: &[(String, usize)]) -> String {
        entries
            .iter()
            .map(|(label, count)| format!("{label}:{count}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn benchmark_panic_message(payload: Box<dyn Any + Send>) -> String {
        match payload.downcast::<String>() {
            Ok(message) => *message,
            Err(payload) => match payload.downcast::<&'static str>() {
                Ok(message) => (*message).to_string(),
                Err(_) => "non-string panic payload".to_string(),
            },
        }
    }

    fn benchmark_profiles_from_env() -> Vec<BenchmarkProfile> {
        match env::var("LGO_BENCH_PROFILE") {
            Ok(value) => {
                let profile = match value.as_str() {
                    "Uniform" => BenchmarkProfile::Uniform,
                    "Singles" => BenchmarkProfile::SinglesAtN,
                    "Pairs" => BenchmarkProfile::PairsAtN,
                    "Hands" => BenchmarkProfile::HandsAtN,
                    _ => panic!(
                        "invalid LGO_BENCH_PROFILE={value:?}; expected one of Uniform|Singles|Pairs|Hands"
                    ),
                };
                vec![profile]
            }
            Err(env::VarError::NotPresent) => BenchmarkProfile::all().to_vec(),
            Err(env::VarError::NotUnicode(_)) => {
                panic!("LGO_BENCH_PROFILE must be valid Unicode")
            }
        }
    }

    fn run_benchmark_profile(profile: BenchmarkProfile, n: usize) -> BenchmarkRun {
        let goals = benchmark_goal_set();
        let pools = benchmark_raw_pools(profile, n);

        for slot in BENCHMARK_SINGLETON_SLOTS {
            let label = benchmark_pool_label(slot);
            let pool = pools
                .get(&slot)
                .unwrap_or_else(|| panic!("missing singleton benchmark pool {label}"));
            assert_frontier_survives_dominance(&label, pool, &goals);
        }

        for (slot1, _) in BENCHMARK_PAIRED_FAMILIES {
            let label = benchmark_pool_label(slot1);
            let pool = pools
                .get(&slot1)
                .unwrap_or_else(|| panic!("missing paired benchmark pool {label}"));
            assert_frontier_survives_dominance(&label, pool, &goals);
        }

        let main_pool = pools
            .get(&Slot::MainHand)
            .expect("missing main-hand benchmark pool");
        let off_pool = pools
            .get(&Slot::OffHand)
            .expect("missing off-hand benchmark pool");
        let mut combined_hands = main_pool.clone();
        combined_hands.extend(off_pool.iter().cloned());
        assert_frontier_survives_dominance("MH+OH", &combined_hands, &goals);

        let mut raw_pool_sizes = Vec::new();
        let mut post_pool_sizes = Vec::new();
        let mut pair_super_counts = Vec::new();
        let mut search_pools = Vec::new();
        let mut seen = HashSet::new();
        let mut hand_configuration_count = 0usize;

        for slot in Slot::all() {
            let canonical = canonical_slot(slot);
            if !seen.insert(canonical) || canonical == Slot::OffHand {
                continue;
            }

            let raw_count = if canonical == Slot::MainHand {
                main_pool.len() + off_pool.len()
            } else {
                pools
                    .get(&canonical)
                    .unwrap_or_else(|| {
                        panic!("missing benchmark pool {}", canonical.display_name())
                    })
                    .len()
            };
            raw_pool_sizes.push((benchmark_pool_label(canonical), raw_count));

            let pre_choices: Vec<Choice> = if canonical == Slot::MainHand {
                let hands = build_hand_choices(main_pool, off_pool);
                hand_configuration_count = hands.len();
                hands
                    .into_iter()
                    .map(|hands| Choice::Hands { hands })
                    .collect()
            } else if matches!(canonical, Slot::Wrist1 | Slot::Finger1 | Slot::Ear1) {
                let pairs = build_pairs(
                    pools.get(&canonical).unwrap_or_else(|| {
                        panic!("missing paired benchmark pool {}", canonical.display_name())
                    }),
                    canonical,
                    paired_slot2(canonical),
                );
                pair_super_counts.push((benchmark_pool_label(canonical), pairs.len()));
                pairs
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
                    .unwrap_or_else(|| {
                        panic!(
                            "missing singleton benchmark pool {}",
                            canonical.display_name()
                        )
                    })
                    .iter()
                    .cloned()
                    .map(|candidate| Choice::Single {
                        slot: canonical,
                        candidate,
                    })
                    .collect()
            };

            let post_choices = dominance_filter(pre_choices, &goals);
            post_pool_sizes.push((benchmark_pool_label(canonical), post_choices.len()));
            search_pools.push(SearchPool {
                choices: post_choices,
            });
        }

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let started = Instant::now();
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                exact_search(&search_pools, &goals, &[0_i64; BENCHMARK_GOALS.len()])
            }));
            let _ = match result {
                Ok(best) => tx.send(BenchmarkWorkerResult::Completed {
                    elapsed: started.elapsed(),
                    found_build: best.is_some(),
                }),
                Err(payload) => tx.send(BenchmarkWorkerResult::Panicked(benchmark_panic_message(
                    payload,
                ))),
            };
        });

        let wall_time = match rx.recv_timeout(Duration::from_secs(TIME_LIMIT_SECS)) {
            Ok(BenchmarkWorkerResult::Completed {
                elapsed,
                found_build,
            }) => {
                assert!(
                    found_build,
                    "benchmark search unexpectedly returned no build"
                );
                BenchmarkWallTime::Completed(elapsed)
            }
            Ok(BenchmarkWorkerResult::Panicked(message)) => {
                panic!("benchmark worker thread panicked: {message}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => BenchmarkWallTime::TimedOut,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("benchmark worker thread disconnected before reporting")
            }
        };

        BenchmarkRun {
            raw_pool_sizes,
            post_pool_sizes,
            pair_super_counts,
            hand_configuration_count,
            wall_time,
        }
    }

    #[test]
    #[ignore]
    fn benchmark_candidate_caps() {
        /*
        benchmark_candidate_caps is an ignored empirical harness for tuning the
        optimizer's candidate caps from data rather than guesses.

        Run only this benchmark with:
            LGO_RUN_BENCH=1 cargo test --release benchmark_candidate_caps -- --ignored --nocapture

        Optional isolation:
            LGO_BENCH_PROFILE=Uniform|Singles|Pairs|Hands

        Debug-build timings are meaningless. The benchmark also returns
        immediately unless LGO_RUN_BENCH=1 is set. Edit TIME_LIMIT_SECS above to
        change the per-run cutoff and MAX_N above to cap the escalation range.
        Each N is timed independently; escalation stops at the first run that
        does not finish within TIME_LIMIT_SECS, prints that breaching row as
        >TIME_LIMIT_SECS, and reports the last completed N for each profile. If
        any run times out, that worker thread keeps burning a core; later
        profiles are contaminated and should be re-run individually with
        LGO_BENCH_PROFILE. The pre column is the raw real-item pool size before
        pair/hand expansion; the post column is the search-pool size after the
        optimizer's dominance filter.
        */
        let mut should_return = false;
        if cfg!(debug_assertions) {
            println!(
                "WARNING: benchmark_candidate_caps should be run with --release; debug timings are meaningless."
            );
            should_return = true;
        }
        if env::var("LGO_RUN_BENCH").ok().as_deref() != Some("1") {
            println!("benchmark_candidate_caps skipped: set LGO_RUN_BENCH=1 to run it.");
            should_return = true;
        }
        if should_return {
            return;
        }

        let profiles = benchmark_profiles_from_env();
        println!("benchmark_candidate_caps");
        println!(
            "goals: {:?}",
            BENCHMARK_GOALS
                .iter()
                .map(|(stat, minimum)| format!("{stat}:{minimum}"))
                .collect::<Vec<_>>()
        );
        println!("time limit: {TIME_LIMIT_SECS}s per optimization run");
        println!("max N: {MAX_N}");
        if profiles.len() == 1 {
            println!("profile filter: {}", profiles[0].env_label());
        }
        println!("| profile | N | pre pools | post pools | pair supers | hand cfgs | wall time |");

        let mut summaries = Vec::new();
        let mut warned_contamination = false;
        for profile in profiles {
            let mut last_completed = None;
            let mut breach_at = None;

            for n in 8usize..=MAX_N {
                let run = run_benchmark_profile(profile, n);
                println!(
                    "| {} | {} | {} | {} | {} | {} | {} |",
                    profile.label(),
                    n,
                    format_benchmark_counts(&run.raw_pool_sizes),
                    format_benchmark_counts(&run.post_pool_sizes),
                    format_benchmark_counts(&run.pair_super_counts),
                    run.hand_configuration_count,
                    run.wall_time.display(),
                );

                if run.wall_time.completed_within_limit() {
                    last_completed = Some(n);
                } else {
                    breach_at = Some(n);
                    if !warned_contamination {
                        println!(
                            "WARNING: benchmark_candidate_caps timed out; the runaway worker thread is still consuming a CPU core, so all later profiles are timing-contaminated and should be re-run individually with LGO_BENCH_PROFILE."
                        );
                        warned_contamination = true;
                    }
                    break;
                }
            }

            summaries.push((profile.label(), last_completed, breach_at));
        }

        println!();
        println!("benchmark_candidate_caps summary");
        for (profile, last_completed, breach_at) in summaries {
            match (last_completed, breach_at) {
                (Some(last), Some(breach)) => {
                    println!("{profile}: last N within {TIME_LIMIT_SECS}s = {last}; breached at N = {breach}");
                }
                (None, Some(breach)) => {
                    println!("{profile}: no run completed within {TIME_LIMIT_SECS}s; first breach at N = {breach}");
                }
                (Some(last), None) => {
                    println!("{profile}: completed through N = {last} without breaching {TIME_LIMIT_SECS}s");
                }
                (None, None) => {
                    println!("{profile}: no runs executed");
                }
            }
        }
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
            Slot::MainHand,
            Slot::OffHand,
        ];
        let mut rng = Lcg::new(seed);

        for case_idx in 0..case_count {
            let mut resolved: HashMap<String, GearItem> = HashMap::new();
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
                    // Half of the main-hand candidates are two-handed and
                    // half of the off-hand candidates are Either-hand so the
                    // fuzzer exercises every legal hand combination against
                    // the oracle.
                    let cached = if family == Slot::MainHand && rng.chance(1, 2) {
                        make_cached_2h(&name, &item_stats)
                    } else if slot == Slot::OffHand && rng.chance(1, 2) {
                        make_cached_either(&name, &item_stats)
                    } else {
                        make_cached(&name, slot, &item_stats)
                    };
                    resolved.insert(key.clone(), cached);
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
        // Smoke-check the production search against the brute-force oracle on a modest randomized corpus.
        run_fuzzer_cases(250, 0x5eed_1234_abcd_9876);
    }

    #[test]
    #[ignore]
    fn differential_fuzzer_matches_oracle_deep() {
        // Deeper randomized differential run against the same brute-force oracle.
        run_fuzzer_cases(5_000, 0x0dd5_ea51_5eed_f00d);
    }
}
