//! Innate Morale/Power calibration from the measured plugin export.
//!
//! The innate Morale/Power baseline — class base, virtue passives, racials,
//! stat tomes — is **measured, not modelled**: the Turbine API exposes no
//! innate value (`Actor:GetBaseMaxMorale()` returns the *current* maximum),
//! and several contributors are not exposed at all. See Bug 14 in
//! `docs/BUG_HISTORY.md`.
//!
//! So LGO subtracts what it already knows. `[MeasuredStats]` records the
//! dressed character's Max Morale/Power and the equipped item names; this
//! pass matches those names to the gear document's items, subtracts their
//! stats (and the innate tracked stats already folded in from Virtues and
//! Base-stat derivation), and folds the remainder back into the innate map.
//! Whatever LGO cannot account for lands in that residual, which is exactly
//! where the unmodelled contributors belong.
//!
//! Runs after Virtue folding and `BaseStatDerivations::derive_doc`, so every
//! item and the innate map are already in final tracked-stat form, and before
//! the optimizer runs.

use std::collections::HashMap;

use crate::gearstats::GearDoc;
use crate::slot_resolver::nfc_eq;
use crate::stat::Stat;

/// The stats calibrated from the measured export, in canonical order.
const CALIBRATED_STATS: [Stat; 2] = [Stat::Morale, Stat::Power];

/// What one calibration pass measured and concluded, for reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalibrationReport {
    /// Character level from the gear file, when it carries one.
    pub level: Option<u32>,
    /// Effects active on the character at export time. Non-zero means the
    /// measured maxima may include buffs.
    pub active_effects: u32,
    /// Max Morale as measured in game at export time.
    pub measured_morale: i64,
    /// Max Power as measured in game at export time.
    pub measured_power: i64,
    /// Innate Morale after calibration.
    pub innate_morale: i64,
    /// Innate Power after calibration.
    pub innate_power: i64,
    /// Equipped item names with no matching `[[item]]` block, in export
    /// order. Their stats are absorbed into the residual.
    pub unmatched: Vec<String>,
}

/// Calibrate `doc`'s innate Morale and Power from `[MeasuredStats]`.
///
/// For each of Morale and Power: sum what LGO already accounts for (the
/// innate map plus the equipped items' stats), subtract that from the
/// measured maximum, and fold the residual back into the innate map. The
/// innate value ends up as "measured maximum minus the equipped set", which
/// is the baseline any other gear combination starts from.
///
/// Returns `None` (leaving `doc` untouched) when the file carries no
/// `[MeasuredStats]` block. Problems are reported as stderr warnings, never
/// errors: a gear file whose measured block is stale still optimizes.
pub fn calibrate_innate(doc: &mut GearDoc) -> Option<CalibrationReport> {
    let measured = doc.measured.clone()?;

    if measured.active_effects > 0 {
        eprintln!(
            "Warning: {} active effect(s) were present when [MeasuredStats] was exported; \
             the measured Max Morale/Power may include buffs, which LGO treats as permanent. \
             Re-export with no food/hope/fellowship buffs for a clean baseline.",
            measured.active_effects
        );
    }

    let (matched_items, unmatched) = match_equipped_items(&measured.equipped, doc);
    for name in &unmatched {
        eprintln!(
            "Warning: equipped item \"{}\" has no matching [[item]] block in the gear file; \
             its stats are absorbed into the innate Morale/Power baseline.",
            name
        );
    }

    let measured_totals: HashMap<Stat, i64> = [
        (Stat::Morale, measured.max_morale),
        (Stat::Power, measured.max_power),
    ]
    .into_iter()
    .collect();

    for stat in CALIBRATED_STATS {
        let innate = doc.innate_stats.get(&stat).copied().unwrap_or(0);
        let equipped_total: i64 = matched_items
            .iter()
            .map(|index| {
                doc.items[*index]
                    .item
                    .stats
                    .get(&stat)
                    .copied()
                    .unwrap_or(0)
            })
            .fold(innate, |total, value| total + value);
        let measured_value = measured_totals[&stat];
        let residual = measured_value - equipped_total;
        let calibrated = innate + residual;

        if residual < 0 {
            eprintln!(
                "Warning: equipped {stat} ({}) exceeds measured Max {stat} ({}); \
                 the innate {stat} baseline calibrated to {}. Re-run /lgo export and \
                 resolve-slots so [MeasuredStats] matches the item stats in the gear file.",
                equipped_total, measured_value, calibrated
            );
        }

        // Absence means zero in the runtime stat maps.
        if calibrated == 0 {
            doc.innate_stats.remove(&stat);
        } else {
            doc.innate_stats.insert(stat, calibrated);
        }
    }

    Some(CalibrationReport {
        level: doc.level,
        active_effects: measured.active_effects,
        measured_morale: measured.max_morale,
        measured_power: measured.max_power,
        innate_morale: doc.innate_stats.get(&Stat::Morale).copied().unwrap_or(0),
        innate_power: doc.innate_stats.get(&Stat::Power).copied().unwrap_or(0),
        unmatched,
    })
}

/// Pair each equipped name with one `[[item]]` entry, count-aware: walking
/// `equipped` in order, every name consumes the first not-yet-consumed item
/// with that name (NFC-normalised exact match), so owning two copies of a
/// ring that is equipped twice consumes both, and owning one consumes one.
///
/// Names with no unconsumed match are returned as unmatched rather than
/// erroring — the residual absorbs whatever their stats would have been.
fn match_equipped_items(equipped: &[String], doc: &GearDoc) -> (Vec<usize>, Vec<String>) {
    let mut consumed = vec![false; doc.items.len()];
    let mut matched = Vec::with_capacity(equipped.len());
    let mut unmatched = Vec::new();

    for name in equipped {
        let found = doc
            .items
            .iter()
            .enumerate()
            .position(|(index, doc_item)| !consumed[index] && nfc_eq(&doc_item.item.name, name));
        match found {
            Some(index) => {
                consumed[index] = true;
                matched.push(index);
            }
            None => unmatched.push(name.clone()),
        }
    }

    (matched, unmatched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gear::{GearItem, Slot};
    use crate::gearstats::{DocItem, MeasuredStats};
    use crate::virtues::SelectedVirtues;

    fn doc_item(name: &str, morale: i64, power: i64) -> DocItem {
        let stats: HashMap<Stat, i64> = [(Stat::Morale, morale), (Stat::Power, power)]
            .into_iter()
            .filter(|(_, value)| *value != 0)
            .collect();
        DocItem {
            item: GearItem {
                name: name.to_string(),
                slot: Slot::Head,
                two_handed: false,
                either_hand: false,
                stats,
            },
            base_stats: HashMap::new(),
        }
    }

    fn gear_doc(
        innate: &[(Stat, i64)],
        items: Vec<DocItem>,
        measured: Option<MeasuredStats>,
    ) -> GearDoc {
        GearDoc {
            character: Some("Thalya".to_string()),
            class: Some("Lore-master".to_string()),
            level: Some(160),
            measured,
            innate_stats: innate.iter().copied().collect(),
            innate_base_stats: HashMap::new(),
            selected_virtues: SelectedVirtues::default(),
            items,
        }
    }

    fn measured(max_morale: i64, max_power: i64, equipped: &[&str]) -> MeasuredStats {
        MeasuredStats {
            max_morale,
            max_power,
            active_effects: 0,
            equipped: equipped.iter().map(|name| name.to_string()).collect(),
        }
    }

    #[test]
    fn no_measured_block_leaves_the_doc_untouched() {
        let mut doc = gear_doc(&[(Stat::Morale, 100)], vec![doc_item("Helm", 50, 5)], None);
        assert!(calibrate_innate(&mut doc).is_none());
        assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&100));
    }

    /// The residual absorbs everything LGO cannot account for: the innate
    /// value ends up as measured minus the equipped set's stats.
    #[test]
    fn residual_becomes_the_innate_baseline() {
        let mut doc = gear_doc(
            &[(Stat::Morale, 45_900), (Stat::Power, 6_000)],
            vec![doc_item("Helm", 5_000, 500), doc_item("Cloak", 2_000, 200)],
            Some(measured(187_342, 21_005, &["Helm", "Cloak"])),
        );

        let report = calibrate_innate(&mut doc).expect("measured block present");

        assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&180_342));
        assert_eq!(doc.innate_stats.get(&Stat::Power), Some(&20_305));
        assert_eq!(report.measured_morale, 187_342);
        assert_eq!(report.measured_power, 21_005);
        assert_eq!(report.innate_morale, 180_342);
        assert_eq!(report.innate_power, 20_305);
        assert_eq!(report.level, Some(160));
        assert!(report.unmatched.is_empty());
    }

    /// Re-running the pass on an already-calibrated doc with the same
    /// equipped set is a no-op: the totals still add up to the measurement.
    #[test]
    fn calibration_is_idempotent_for_the_same_equipped_set() {
        let mut doc = gear_doc(
            &[(Stat::Morale, 1_000), (Stat::Power, 100)],
            vec![doc_item("Helm", 5_000, 500)],
            Some(measured(100_000, 9_000, &["Helm"])),
        );

        calibrate_innate(&mut doc).expect("first pass");
        let first = doc.innate_stats.clone();
        calibrate_innate(&mut doc).expect("second pass");

        assert_eq!(doc.innate_stats, first);
        assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&95_000));
    }

    /// Matching is count-aware: two owned copies with one equipped consumes
    /// exactly one of them.
    #[test]
    fn duplicate_names_are_matched_by_count() {
        let mut doc = gear_doc(
            &[],
            vec![doc_item("Ring", 1_000, 0), doc_item("Ring", 1_000, 0)],
            Some(measured(50_000, 5_000, &["Ring"])),
        );

        calibrate_innate(&mut doc).expect("measured block present");
        assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&49_000));

        let mut both = gear_doc(
            &[],
            vec![doc_item("Ring", 1_000, 0), doc_item("Ring", 1_000, 0)],
            Some(measured(50_000, 5_000, &["Ring", "Ring"])),
        );
        calibrate_innate(&mut both).expect("measured block present");
        assert_eq!(both.innate_stats.get(&Stat::Morale), Some(&48_000));
    }

    /// An equipped name LGO owns no item for is reported, not fatal.
    #[test]
    fn unmatched_equipped_names_are_reported_and_absorbed() {
        let mut doc = gear_doc(
            &[],
            vec![doc_item("Helm", 5_000, 0)],
            Some(measured(100_000, 9_000, &["Helm", "Mystery Trinket"])),
        );

        let report = calibrate_innate(&mut doc).expect("measured block present");

        assert_eq!(report.unmatched, vec!["Mystery Trinket".to_string()]);
        assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&95_000));
    }

    /// Names are matched NFC-normalised, so a decomposed export still pairs
    /// with a composed item name (Bug 4's territory).
    #[test]
    fn equipped_names_match_across_unicode_normalization_forms() {
        let composed = "Keen Pristine Mad\u{e1}shi Ring";
        let decomposed = "Keen Pristine Mada\u{301}shi Ring";
        let mut doc = gear_doc(
            &[],
            vec![doc_item(composed, 2_000, 0)],
            Some(measured(100_000, 9_000, &[decomposed])),
        );

        let report = calibrate_innate(&mut doc).expect("measured block present");

        assert!(report.unmatched.is_empty());
        assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&98_000));
    }

    /// A calibrated value of zero is dropped, matching the runtime convention
    /// that an absent stat means zero.
    #[test]
    fn zero_calibrated_value_is_dropped_from_the_innate_map() {
        let mut doc = gear_doc(
            &[(Stat::Power, 500)],
            vec![doc_item("Helm", 5_000, 0)],
            Some(measured(5_000, 0, &["Helm"])),
        );

        let report = calibrate_innate(&mut doc).expect("measured block present");

        assert_eq!(doc.innate_stats.get(&Stat::Morale), None);
        assert_eq!(doc.innate_stats.get(&Stat::Power), None);
        assert_eq!(report.innate_morale, 0);
        assert_eq!(report.innate_power, 0);
    }

    /// A stale measurement can leave the baseline negative. That is a
    /// warning, not an error — the numbers still round-trip.
    #[test]
    fn negative_residual_is_tolerated() {
        let mut doc = gear_doc(
            &[],
            vec![doc_item("Helm", 200_000, 0)],
            Some(measured(100_000, 9_000, &["Helm"])),
        );

        let report = calibrate_innate(&mut doc).expect("measured block present");

        assert_eq!(doc.innate_stats.get(&Stat::Morale), Some(&-100_000));
        assert_eq!(report.innate_morale, -100_000);
    }
}
