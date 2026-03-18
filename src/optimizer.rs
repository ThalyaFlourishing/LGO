use std::collections::HashMap;
use crate::gear::{GearItem, GearSet, Slot};
use crate::stat::StatWeights;

/// Given a list of available items and a stat-weight profile, select the
/// highest-scoring item for each slot to produce an optimized gear set.
pub fn optimize(items: &[GearItem], weights: &StatWeights) -> GearSet {
    // Group items by slot.
    let mut by_slot: HashMap<Slot, Vec<&GearItem>> = HashMap::new();
    for item in items {
        by_slot.entry(item.slot).or_default().push(item);
    }

    let mut set = GearSet::default();

    // For each slot, pick the item with the highest weighted score.
    for (slot, candidates) in &by_slot {
        if let Some(best) = candidates
            .iter()
            .max_by(|a, b| {
                a.score(weights)
                    .partial_cmp(&b.score(weights))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            set.items.insert(*slot, (*best).clone());
        }
    }

    set
}

/// Score an existing gear set and return the weighted total.
pub fn score_set(set: &GearSet, weights: &StatWeights) -> f64 {
    set.score(weights)
}
