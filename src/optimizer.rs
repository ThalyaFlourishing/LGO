use anyhow::{bail, Result};
use std::collections::HashMap;

/// A single gear item parsed from an `.lgo` file.
#[derive(Debug, Clone)]
pub struct Item {
    pub slot: String,
    pub name: String,
    pub stats: HashMap<String, f64>,
}

/// The result produced by the optimizer for a single equipment slot.
#[derive(Debug, Clone)]
pub struct SlotResult {
    pub slot: String,
    pub best: Item,
}

/// Parse the raw text of an `.lgo` file into a list of [`Item`]s.
///
/// Lines starting with `#` or that are blank are ignored.
/// Each data line has the format:
/// ```text
/// <slot>,<item_name>,<stat>=<value>[,<stat>=<value>...]
/// ```
pub fn parse(raw: &str) -> Result<Vec<Item>> {
    let mut items = Vec::new();

    for (line_no, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, ',').collect();
        let (slot_raw, name_raw, stat_part) = match parts.as_slice() {
            [s, n, p] => (*s, *n, *p),
            _ => bail!(
                "Line {}: expected '<slot>,<name>,<stat>=<value>...', got: {:?}",
                line_no + 1,
                line
            ),
        };

        let slot = slot_raw.trim().to_string();
        let name = name_raw.trim().to_string();

        let mut stats = HashMap::new();
        for kv in stat_part.split(',') {
            let kv = kv.trim();
            if kv.is_empty() {
                continue;
            }
            let mut iter = kv.splitn(2, '=');
            let key = iter
                .next()
                .map(str::trim)
                .unwrap_or("")
                .to_string();
            let val_str = iter
                .next()
                .map(str::trim)
                .unwrap_or("0");
            let val: f64 = val_str.parse().unwrap_or(0.0);
            stats.insert(key, val);
        }

        items.push(Item { slot, name, stats });
    }

    Ok(items)
}

/// Choose the best item for each equipment slot using a simple sum-of-stats heuristic.
pub fn optimize(items: &[Item]) -> Vec<SlotResult> {
    let mut best_by_slot: HashMap<String, &Item> = HashMap::new();

    for item in items {
        let score: f64 = item.stats.values().sum();
        let current_score = best_by_slot
            .get(&item.slot)
            .map(|b| b.stats.values().sum::<f64>())
            .unwrap_or(f64::NEG_INFINITY);

        if score > current_score {
            best_by_slot.insert(item.slot.clone(), item);
        }
    }

    let mut results: Vec<SlotResult> = best_by_slot
        .into_values()
        .map(|item| SlotResult {
            slot: item.slot.clone(),
            best: item.clone(),
        })
        .collect();

    results.sort_by(|a, b| a.slot.cmp(&b.slot));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_item() {
        let raw = "head,Iron Helm,Vitality=200,Morale=400\n";
        let items = parse(raw).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].slot, "head");
        assert_eq!(items[0].name, "Iron Helm");
        assert_eq!(items[0].stats["Vitality"], 200.0);
        assert_eq!(items[0].stats["Morale"], 400.0);
    }

    #[test]
    fn optimize_picks_best() {
        let raw = "head,Helm A,Morale=100\nhead,Helm B,Morale=200\n";
        let items = parse(raw).unwrap();
        let results = optimize(&items);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].best.name, "Helm B");
    }

    #[test]
    fn comments_and_blanks_are_ignored() {
        let raw = "# comment\n\nhead,Iron Helm,Vitality=50\n";
        let items = parse(raw).unwrap();
        assert_eq!(items.len(), 1);
    }
}
