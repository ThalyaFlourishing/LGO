//! Interactive merge of a freshly-exported gear stats file with an existing,
//! potentially hand-edited, stats file.
//!
//! # How it works
//!
//! Each gear stats file (`.toml`) may contain a `[__user_edits__]` section at
//! the bottom.  That section records which `"ItemName.StatKey"` pairs the user
//! has deliberately hand-edited.  LGO writes and maintains this section
//! automatically; users never need to touch it.
//!
//! When `lgo --gearlist` finds an existing stats file it runs a merge:
//!
//! 1. **Exporter gap** (`new_val == 0`, `old_val != 0`): the wiki/DB could not
//!    supply a value but the user filled one in (e.g. a Legendary Item stat).
//!    The old value is always preserved silently and the key is flagged.
//!
//! 2. **No change** (`old_val == new_val`): nothing to do; any existing flag is
//!    carried forward.
//!
//! 3. **Values differ, field is flagged** (present in `[__user_edits__]`):
//!    prompt the user — keep theirs or accept the exporter value.
//!
//! 4. **Values differ, field is NOT flagged, file has no `[__user_edits__]`
//!    section yet**: this is the first merge; every difference could be a user
//!    edit, so the user is prompted for each one.
//!
//! 5. **Values differ, field is NOT flagged, file already has a
//!    `[__user_edits__]` section**: the user previously accepted the exporter
//!    value for this field; overwrite automatically.
//!
//! Before individual prompts the user is offered a batch option:
//! keep all theirs (`k`), accept all exporter values (`a`), or prompt each
//! field (`p`, the default).

use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::cache::CachedItem;
use crate::gearstats;
use crate::stat::TRACKED_STATS;

// -- Public types --------------------------------------------------------------

/// The set of `"ItemName.StatKey"` strings the user has hand-edited.
pub type UserEdits = HashSet<String>;

/// Everything read from an existing stats file that is needed for merging.
pub struct MergeContext {
    pub old_items: Vec<CachedItem>,
    /// Keys that have been hand-edited (from `[__user_edits__]`).
    pub user_edits: UserEdits,
    /// True if the file contained a `[__user_edits__]` table (even if empty).
    /// Used to distinguish "first ever merge" from "subsequent merge".
    pub had_user_edits_section: bool,
}

// -- Public API ----------------------------------------------------------------

/// Load item data and user-edit metadata from an existing stats file.
pub fn read_merge_context(path: &Path) -> Result<MergeContext, String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;

    let old_items = gearstats::read_stats_file(path)?;
    let (user_edits, had_user_edits_section) = parse_user_edits(&src)?;

    Ok(MergeContext {
        old_items,
        user_edits,
        had_user_edits_section,
    })
}

/// Merge freshly-exported items with the user's existing hand-edited file.
///
/// Prompts the user interactively for any conflicts (unless a batch option is
/// chosen up front).  Returns the merged item list and the updated
/// `UserEdits` set that should be written back to the new stats file.
pub fn merge_stats(new_items: Vec<CachedItem>, ctx: &MergeContext) -> (Vec<CachedItem>, UserEdits) {
    // Build name → item lookup for the old file.
    let old_map: HashMap<&str, &CachedItem> =
        ctx.old_items.iter().map(|i| (i.name.as_str(), i)).collect();

    // Count fields that need user input.
    let prompt_count = count_prompts(
        &new_items,
        &old_map,
        &ctx.user_edits,
        ctx.had_user_edits_section,
    );

    // When nothing needs prompting we still need to carry forward preserved
    // and flagged fields, but we can skip the batch-option question.
    let batch = if prompt_count == 0 {
        BatchOption::PromptEach // won't actually be used
    } else {
        ask_batch_option(prompt_count)
    };

    let mut new_edits = UserEdits::new();

    let merged = new_items
        .into_iter()
        .map(|mut item| {
            if let Some(old_item) = old_map.get(item.name.as_str()) {
                for (stat, key) in TRACKED_STATS {
                    let new_val = item.stats.get(stat).copied().unwrap_or(0);
                    let old_val = old_item.stats.get(stat).copied().unwrap_or(0);
                    let ekey = edit_key(&item.name, key);

                    // ── Case 1: exporter gap / Legendary Item stat ──────────────
                    // The wiki/DB could not supply this value but the user has one.
                    // Always preserve silently.
                    if new_val == 0 && old_val != 0 {
                        item.stats.insert(*stat, old_val);
                        new_edits.insert(ekey);
                        continue;
                    }

                    // ── Case 2: no change ───────────────────────────────────────
                    if old_val == new_val {
                        // Carry the flag forward if it was set.
                        if ctx.user_edits.contains(&ekey) {
                            new_edits.insert(ekey);
                        }
                        continue;
                    }

                    // ── Cases 3-5: values differ ────────────────────────────────
                    let is_flagged = ctx.user_edits.contains(&ekey);
                    let should_prompt = is_flagged || !ctx.had_user_edits_section;

                    if should_prompt {
                        let keep = match batch {
                            BatchOption::KeepAll => true,
                            BatchOption::AcceptAll => false,
                            BatchOption::PromptEach => {
                                prompt_field(&item.name, key, old_val, new_val)
                            }
                        };
                        if keep {
                            item.stats.insert(*stat, old_val);
                            new_edits.insert(ekey);
                        }
                        // If not kept: new_val already in item.stats; no flag.
                    } else {
                        // File has a user_edits section and this field is not in it
                        // → the user previously accepted exporter data here.
                        eprintln!(
                            "[lgo]   [{}] {} not manually edited — updating to \
                         exporter value ({})",
                            item.name, key, new_val
                        );
                    }
                }
            }
            item
        })
        .collect();

    (merged, new_edits)
}

// -- Internals -----------------------------------------------------------------

/// Parse the `[__user_edits__]` table from TOML source text.
fn parse_user_edits(src: &str) -> Result<(UserEdits, bool), String> {
    let doc: toml::Value = src.parse().map_err(|e| format!("Malformed TOML: {}", e))?;

    match doc.get("__user_edits__") {
        None => Ok((UserEdits::new(), false)),
        Some(toml::Value::Table(t)) => {
            let mut edits = UserEdits::new();
            for (k, v) in t {
                match v.as_bool() {
                    Some(true) => {
                        edits.insert(k.clone());
                    }
                    Some(false) => { /* false = not edited; skip */ }
                    None => {
                        eprintln!(
                            "[lgo] Warning: unexpected value type for [__user_edits__] \
                             key '{}' (expected boolean); ignoring.",
                            k
                        );
                    }
                }
            }
            Ok((edits, true))
        }
        Some(_) => Ok((UserEdits::new(), true)), // unexpected type — treat as empty
    }
}

/// Return an `"ItemName.StatKey"` composite key.
fn edit_key(item_name: &str, stat_key: &str) -> String {
    format!("{}.{}", item_name, stat_key)
}

/// Count how many fields will need an interactive prompt.
fn count_prompts(
    new_items: &[CachedItem],
    old_map: &HashMap<&str, &CachedItem>,
    edits: &UserEdits,
    had_section: bool,
) -> usize {
    let mut n = 0;
    for item in new_items {
        if let Some(old) = old_map.get(item.name.as_str()) {
            for (stat, key) in TRACKED_STATS {
                let nv = item.stats.get(stat).copied().unwrap_or(0);
                let ov = old.stats.get(stat).copied().unwrap_or(0);
                if nv == 0 && ov != 0 {
                    continue;
                } // auto-preserved
                if nv == ov {
                    continue;
                } // no change
                if edits.contains(&edit_key(&item.name, key)) || !had_section {
                    n += 1;
                }
            }
        }
    }
    n
}

enum BatchOption {
    KeepAll,
    AcceptAll,
    PromptEach,
}

fn ask_batch_option(count: usize) -> BatchOption {
    eprintln!(
        "[lgo] {} field(s) changed that may have been hand-edited.",
        count
    );
    eprint!(
        "[lgo] Keep all yours (k), accept all exporter values (a), \
         or prompt each (p)? [k/a/p, default=p]: "
    );
    io::stderr().flush().ok();

    let stdin = io::stdin();
    let mut line = String::new();
    match stdin.lock().read_line(&mut line) {
        Ok(_) => match line.trim().to_lowercase().as_str() {
            "a" | "accept" | "accept-all" => BatchOption::AcceptAll,
            "k" | "keep" | "keep-all" => BatchOption::KeepAll,
            _ => BatchOption::PromptEach,
        },
        Err(_) => BatchOption::PromptEach,
    }
}

fn prompt_field(item_name: &str, stat_key: &str, old_val: i64, new_val: i64) -> bool {
    eprint!(
        "  [{}] {}: yours ({}) vs exporter ({}). Keep yours? [y/n, default=y]: ",
        item_name, stat_key, old_val, new_val
    );
    io::stderr().flush().ok();

    let stdin = io::stdin();
    let mut line = String::new();
    match stdin.lock().read_line(&mut line) {
        Ok(_) => !matches!(line.trim().to_lowercase().as_str(), "n" | "no"),
        Err(_) => true, // default: keep old value on read failure
    }
}

// -- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::CachedItem;
    use crate::gear::Slot;
    use crate::stat::Stat;
    use std::collections::HashMap;

    fn make_item(name: &str, stats: &[(Stat, i64)]) -> CachedItem {
        CachedItem {
            name: name.to_string(),
            slot: Slot::Finger1,
            stats: stats.iter().cloned().collect(),
        }
    }

    #[test]
    fn exporter_gap_always_preserved() {
        // new_val == 0, old_val != 0 → always keep old
        let new_items = vec![make_item("Ring", &[(Stat::Block, 0)])];
        let ctx = MergeContext {
            old_items: vec![make_item("Ring", &[(Stat::Block, 500)])],
            user_edits: UserEdits::new(),
            had_user_edits_section: true,
        };

        // We can't easily test the interactive path, but we can test the
        // exporter-gap branch by ensuring the old value is used when the
        // batch is AcceptAll (which would normally overwrite).
        // Instead, call the internal count_prompts to verify gap is excluded.
        let old_map: HashMap<&str, &CachedItem> =
            ctx.old_items.iter().map(|i| (i.name.as_str(), i)).collect();
        let prompts = count_prompts(
            &new_items,
            &old_map,
            &ctx.user_edits,
            ctx.had_user_edits_section,
        );
        // Gap fields are excluded from prompts (auto-preserved).
        assert_eq!(prompts, 0);
    }

    #[test]
    fn no_change_no_prompt() {
        let item = make_item("Ring", &[(Stat::Block, 500)]);
        let new_items = vec![item.clone()];
        let ctx = MergeContext {
            old_items: vec![item],
            user_edits: UserEdits::new(),
            had_user_edits_section: true,
        };
        let old_map: HashMap<&str, &CachedItem> =
            ctx.old_items.iter().map(|i| (i.name.as_str(), i)).collect();
        assert_eq!(
            count_prompts(
                &new_items,
                &old_map,
                &ctx.user_edits,
                ctx.had_user_edits_section
            ),
            0
        );
    }

    #[test]
    fn flagged_field_triggers_prompt() {
        let new_items = vec![make_item("Ring", &[(Stat::Block, 600)])];
        let mut edits = UserEdits::new();
        edits.insert("Ring.Block".to_string());
        let ctx = MergeContext {
            old_items: vec![make_item("Ring", &[(Stat::Block, 500)])],
            user_edits: edits,
            had_user_edits_section: true,
        };
        let old_map: HashMap<&str, &CachedItem> =
            ctx.old_items.iter().map(|i| (i.name.as_str(), i)).collect();
        assert_eq!(
            count_prompts(
                &new_items,
                &old_map,
                &ctx.user_edits,
                ctx.had_user_edits_section
            ),
            1
        );
    }

    #[test]
    fn first_merge_no_section_all_diffs_prompt() {
        // had_user_edits_section = false → every diff counts as a prompt
        let new_items = vec![make_item("Ring", &[(Stat::Block, 600)])];
        let ctx = MergeContext {
            old_items: vec![make_item("Ring", &[(Stat::Block, 500)])],
            user_edits: UserEdits::new(),
            had_user_edits_section: false,
        };
        let old_map: HashMap<&str, &CachedItem> =
            ctx.old_items.iter().map(|i| (i.name.as_str(), i)).collect();
        assert_eq!(
            count_prompts(
                &new_items,
                &old_map,
                &ctx.user_edits,
                ctx.had_user_edits_section
            ),
            1
        );
    }

    #[test]
    fn unflagged_diff_with_section_no_prompt() {
        // had_user_edits_section = true, field NOT flagged → auto-overwrite, no prompt
        let new_items = vec![make_item("Ring", &[(Stat::Block, 600)])];
        let ctx = MergeContext {
            old_items: vec![make_item("Ring", &[(Stat::Block, 500)])],
            user_edits: UserEdits::new(),
            had_user_edits_section: true,
        };
        let old_map: HashMap<&str, &CachedItem> =
            ctx.old_items.iter().map(|i| (i.name.as_str(), i)).collect();
        assert_eq!(
            count_prompts(
                &new_items,
                &old_map,
                &ctx.user_edits,
                ctx.had_user_edits_section
            ),
            0
        );
    }

    #[test]
    fn parse_user_edits_empty_section() {
        let src = r#"
[[item]]
slot = "Head"
name = "Helm"
Block = 0

[__user_edits__]
"#;
        let (edits, had) = parse_user_edits(src).unwrap();
        assert!(had);
        assert!(edits.is_empty());
    }

    #[test]
    fn parse_user_edits_with_entries() {
        let src = r#"
[[item]]
slot = "Head"
name = "Helm"
Block = 0

[__user_edits__]
"Helm.Block" = true
"OtherRing.Parry" = true
"#;
        let (edits, had) = parse_user_edits(src).unwrap();
        assert!(had);
        assert!(edits.contains("Helm.Block"));
        assert!(edits.contains("OtherRing.Parry"));
        assert_eq!(edits.len(), 2);
    }

    #[test]
    fn parse_user_edits_no_section() {
        let src = r#"
[[item]]
slot = "Head"
name = "Helm"
Block = 0
"#;
        let (edits, had) = parse_user_edits(src).unwrap();
        assert!(!had);
        assert!(edits.is_empty());
    }
}
