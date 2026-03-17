use std::collections::BTreeMap;
use crate::optimizer::SlotResult;

/// Render a list of [`SlotResult`]s as a human-readable text report.
pub fn render(results: &[SlotResult]) -> String {
    if results.is_empty() {
        return "No gear data found.\n".to_string();
    }

    let mut out = String::new();
    out.push_str("=== LGO Optimized Gear Report ===\n\n");

    for sr in results {
        out.push_str(&format!("[{}]\n", sr.slot.to_uppercase()));
        out.push_str(&format!("  Best item : {}\n", sr.best.name));

        let stats: BTreeMap<&String, &f64> = sr.best.stats.iter().collect();

        for (stat, value) in &stats {
            out.push_str(&format!("  {:20} {:.0}\n", stat, value));
        }
        out.push('\n');
    }

    let total: f64 = results
        .iter()
        .flat_map(|sr| sr.best.stats.values())
        .sum();

    out.push_str(&format!("Total stat score: {:.0}\n", total));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::{Item, SlotResult};
    use std::collections::HashMap;

    fn make_result(slot: &str, name: &str, stats: &[(&str, f64)]) -> SlotResult {
        let mut map = HashMap::new();
        for (k, v) in stats {
            map.insert(k.to_string(), *v);
        }
        SlotResult {
            slot: slot.to_string(),
            best: Item {
                slot: slot.to_string(),
                name: name.to_string(),
                stats: map,
            },
        }
    }

    #[test]
    fn render_empty() {
        assert_eq!(render(&[]), "No gear data found.\n");
    }

    #[test]
    fn render_single_item() {
        let results = vec![make_result("head", "Iron Helm", &[("Morale", 400.0)])];
        let out = render(&results);
        assert!(out.contains("[HEAD]"));
        assert!(out.contains("Iron Helm"));
        assert!(out.contains("400"));
    }
}
