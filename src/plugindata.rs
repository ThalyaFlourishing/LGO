//! Parser for the Lua table format written by Turbine.PluginData.Save.
//!
//! The file begins with `return ` followed by a Lua table literal.
//! Keys are always in one of these forms:
//!   ["string_key"]   — bracketed string key
//!   [1.000000]       — bracketed numeric key (array index)
//!   bare_ident       — unquoted identifier key (rare in PluginData but present)
//!
//! Values are strings, numbers, booleans, nil, or nested tables.
//!
//! We use a hand-written recursive descent parser. No full Lua runtime is
//! needed because the PluginData subset is small and well-defined.
//!
//! Output: a `PluginExport` with character metadata from
//! `lgo_<character-name>_gearNames_<timestamp>.plugindata`
//! (`character`, `class`, `baseStats`, and — for `lgo-gearlist-2` exports —
//! `level`, `maxMorale`, `maxPower`, `activeEffects`, and `equipped`).

use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ── Public types ──────────────────────────────────────────────────────────────

/// All item data extracted from the plugin export file, before wiki lookup.
#[derive(Debug, PartialEq, Eq)]
pub struct PluginExport {
    #[allow(dead_code)]
    pub character: String,
    /// Character class string, e.g. "Lore-master" or "Guardian".
    pub class: String,
    /// Base primary stats (Might/Agility/Vitality/Will/Fate) before gear bonuses.
    pub base_stats: HashMap<String, i64>,
    /// Character level, if the export carries one.
    pub level: Option<u32>,
    /// Max Morale as the in-game panel showed it at export time — dressed,
    /// including gear, virtues and whatever buffs were active.
    pub max_morale: Option<i64>,
    /// Max Power, measured the same way as `max_morale`.
    pub max_power: Option<i64>,
    /// Number of effects active on the character at export time. Non-zero
    /// means the measured Max Morale/Power may include buffs.
    pub active_effects: Option<u32>,
    /// Equipped item names in slot order, duplicates preserved. Empty when
    /// the export does not carry the field.
    pub equipped: Vec<String>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn load(path: &Path) -> Result<PluginExport, String> {
    let src =
        fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;

    // Files start with "return " followed by the table literal.
    let src = src.trim();
    let src = src.strip_prefix("return").unwrap_or(src).trim_start();

    let (val, _) =
        parse_value(src, 0).map_err(|e| format!("Parse error in {}: {}", path.display(), e))?;

    extract_export(val)
}

// ── Extraction ────────────────────────────────────────────────────────────────

fn extract_export(val: LuaVal) -> Result<PluginExport, String> {
    let root = expect_table(val, "root")?;

    let character = match table_get(&root, "character") {
        Some(LuaVal::Str(s)) => s,
        _ => "Unknown".to_string(),
    };

    let class = match table_get(&root, "class") {
        Some(LuaVal::Str(s)) => s,
        _ => "Unknown".to_string(),
    };

    let base_stats = extract_base_stats(&root);

    Ok(PluginExport {
        character,
        class,
        base_stats,
        level: table_get_u32(&root, "level"),
        max_morale: table_get_i64(&root, "maxMorale"),
        max_power: table_get_i64(&root, "maxPower"),
        active_effects: table_get_u32(&root, "activeEffects"),
        equipped: extract_string_array(&root, "equipped"),
    })
}

/// Read a numeric field as `i64`. Plugin numbers arrive as Lua floats
/// (`187342.000000`); truncation is exact for the integral values the plugin
/// writes. A missing or non-numeric field is `None`, never an error — old
/// exports simply lack these keys.
fn table_get_i64(tbl: &LuaTable, key: &str) -> Option<i64> {
    match table_get(tbl, key) {
        Some(LuaVal::Num(n)) if n.is_finite() => Some(n as i64),
        _ => None,
    }
}

/// Read a numeric field as `u32`, dropping negatives and out-of-range values
/// rather than wrapping them.
fn table_get_u32(tbl: &LuaTable, key: &str) -> Option<u32> {
    let n = table_get_i64(tbl, key)?;
    u32::try_from(n).ok()
}

/// Read a Lua array of strings (`[1.000000] = "..."`) in index order,
/// preserving duplicates. Missing field or non-table value ⇒ empty vector.
fn extract_string_array(tbl: &LuaTable, key: &str) -> Vec<String> {
    let Some(LuaVal::Table(entries)) = table_get(tbl, key) else {
        return Vec::new();
    };

    let mut indexed: Vec<(f64, String)> = Vec::new();
    for (entry_key, entry_val) in &entries {
        let (LuaKey::Num(index), LuaVal::Str(name)) = (entry_key, entry_val) else {
            continue;
        };
        indexed.push((*index, name.clone()));
    }
    // Sort by Lua array index so the recorded slot order survives even if the
    // serializer emitted the pairs out of order. The sort is stable, so equal
    // indices (which the plugin never writes) keep file order.
    indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    indexed.into_iter().map(|(_, name)| name).collect()
}

fn extract_base_stats(root: &LuaTable) -> HashMap<String, i64> {
    let mut map = HashMap::new();

    let bs_tbl = match table_get(root, "baseStats") {
        Some(LuaVal::Table(t)) => t,
        _ => return map,
    };

    // Keys are the Lua method names; map them to clean stat names.
    let method_to_name = [
        ("GetBaseMight", "Might"),
        ("GetBaseAgility", "Agility"),
        ("GetBaseVitality", "Vitality"),
        ("GetBaseWill", "Will"),
        ("GetBaseFate", "Fate"),
    ];

    for (method, stat_name) in &method_to_name {
        if let Some(LuaVal::Num(n)) = table_get(&bs_tbl, method) {
            map.insert(stat_name.to_string(), n as i64);
        }
    }

    map
}

// ── Raw Lua value types ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum LuaVal {
    Str(String),
    Num(f64),
    #[allow(dead_code)]
    Bool(bool),
    Table(LuaTable),
    Nil,
}

type LuaTable = Vec<(LuaKey, LuaVal)>;

#[derive(Debug, Clone)]
enum LuaKey {
    Str(String),
    #[allow(dead_code)]
    Num(f64),
}

// ── Table helpers ─────────────────────────────────────────────────────────────

fn expect_table(val: LuaVal, ctx: &str) -> Result<LuaTable, String> {
    match val {
        LuaVal::Table(t) => Ok(t),
        other => Err(format!("Expected table at '{}', got {:?}", ctx, other)),
    }
}

/// Return a clone of the value for a string key, or None.
fn table_get(tbl: &LuaTable, key: &str) -> Option<LuaVal> {
    tbl.iter()
        .find(|(k, _)| matches!(k, LuaKey::Str(s) if s == key))
        .map(|(_, v)| v.clone())
}

// ── Recursive descent parser ──────────────────────────────────────────────────

const MAX_DEPTH: usize = 64;

fn skip_ws_and_comments(s: &str) -> &str {
    let mut s = s;
    loop {
        s = s.trim_start();
        if s.starts_with("--") {
            s = match s.find('\n') {
                Some(i) => &s[i + 1..],
                None => "",
            };
        } else {
            break;
        }
    }
    s
}

fn parse_value(s: &str, depth: usize) -> Result<(LuaVal, &str), String> {
    if depth > MAX_DEPTH {
        return Err("Table nesting depth exceeded".into());
    }
    let s = skip_ws_and_comments(s);

    if let Some(rest) = s.strip_prefix('{') {
        parse_table(rest, depth)
    } else if s.starts_with('"') {
        parse_quoted_string(s)
    } else if let Some(rest) = s.strip_prefix("true") {
        Ok((LuaVal::Bool(true), rest))
    } else if let Some(rest) = s.strip_prefix("false") {
        Ok((LuaVal::Bool(false), rest))
    } else if let Some(rest) = s.strip_prefix("nil") {
        Ok((LuaVal::Nil, rest))
    } else {
        parse_number(s)
    }
}

fn parse_table(s: &str, depth: usize) -> Result<(LuaVal, &str), String> {
    let mut s = skip_ws_and_comments(s);
    let mut entries: LuaTable = Vec::new();

    loop {
        s = skip_ws_and_comments(s);

        if let Some(rest) = s.strip_prefix('}') {
            return Ok((LuaVal::Table(entries), rest));
        }
        if s.is_empty() {
            return Err("Unexpected end of input inside table".into());
        }
        // Consume separator between entries.
        if s.starts_with(',') || s.starts_with(';') {
            s = &s[1..];
            continue;
        }

        let (key, rest) = parse_key(s)?;
        s = skip_ws_and_comments(rest);

        if !s.starts_with('=') {
            return Err(format!(
                "Expected '=' after key, found: {:?}",
                &s[..s.len().min(30)]
            ));
        }
        s = skip_ws_and_comments(&s[1..]);

        let (val, rest) = parse_value(s, depth + 1)?;
        s = skip_ws_and_comments(rest);

        if s.starts_with(',') || s.starts_with(';') {
            s = &s[1..];
        }

        entries.push((key, val));
    }
}

fn parse_key(s: &str) -> Result<(LuaKey, &str), String> {
    if let Some(rest) = s.strip_prefix('[') {
        let inner = skip_ws_and_comments(rest);
        if inner.starts_with('"') {
            let (val, rest) = parse_quoted_string(inner)?;
            let rest = skip_ws_and_comments(rest);
            let rest = rest
                .strip_prefix(']')
                .ok_or_else(|| "Expected ']' after string key".to_string())?;
            if let LuaVal::Str(k) = val {
                return Ok((LuaKey::Str(k), rest));
            }
            unreachable!()
        } else {
            let (val, rest) = parse_number(inner)?;
            let rest = skip_ws_and_comments(rest);
            let rest = rest
                .strip_prefix(']')
                .ok_or_else(|| "Expected ']' after numeric key".to_string())?;
            if let LuaVal::Num(n) = val {
                return Ok((LuaKey::Num(n), rest));
            }
            unreachable!()
        }
    }

    // Bare identifier key (e.g. `version = ...`)
    let end = s
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(s.len());
    if end == 0 {
        return Err(format!("Expected key, found: {:?}", &s[..s.len().min(30)]));
    }
    Ok((LuaKey::Str(s[..end].to_string()), &s[end..]))
}

fn parse_quoted_string(s: &str) -> Result<(LuaVal, &str), String> {
    // s must start with '"'
    debug_assert!(s.starts_with('"'));
    let s = &s[1..];
    let mut result = String::new();
    let mut iter = s.char_indices().peekable();

    loop {
        match iter.next() {
            None => return Err("Unterminated string literal".into()),
            Some((i, '"')) => {
                let remaining = &s[i + '"'.len_utf8()..];
                return Ok((LuaVal::Str(result), remaining));
            }
            Some((_, '\\')) => match iter.next() {
                Some((_, 'n')) => result.push('\n'),
                Some((_, 'r')) => result.push('\r'),
                Some((_, 't')) => result.push('\t'),
                Some((_, '"')) => result.push('"'),
                Some((_, '\\')) => result.push('\\'),
                Some((_, c)) => {
                    result.push('\\');
                    result.push(c);
                }
                None => return Err("Unterminated escape sequence".into()),
            },
            Some((_, c)) => result.push(c),
        }
    }
}

fn parse_number(s: &str) -> Result<(LuaVal, &str), String> {
    // Consume an optional leading minus then digits, dot, exponent.
    let end = s
        .find(|c: char| {
            !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E' && c != '+'
        })
        .unwrap_or(s.len());

    if end == 0 {
        return Err(format!(
            "Expected number, found: {:?}",
            &s[..s.len().min(30)]
        ));
    }
    let n: f64 = s[..end]
        .parse()
        .map_err(|_| format!("Invalid number literal: '{}'", &s[..end]))?;
    Ok((LuaVal::Num(n), &s[end..]))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> LuaVal {
        let (v, rest) = parse_value(s, 0).expect("parse failed");
        assert!(rest.trim().is_empty(), "leftover input: {:?}", rest);
        v
    }

    #[test]
    fn test_string() {
        match parse(r#""hello world""#) {
            LuaVal::Str(s) => assert_eq!(s, "hello world"),
            other => panic!("expected Str, got {:?}", other),
        }
    }

    #[test]
    fn test_string_escape() {
        match parse(r#""line1\nline2""#) {
            LuaVal::Str(s) => assert_eq!(s, "line1\nline2"),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn test_number_float() {
        match parse("12.000000") {
            LuaVal::Num(n) => assert!((n - 12.0).abs() < 1e-9),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn test_bool() {
        assert!(matches!(parse("true"), LuaVal::Bool(true)));
        assert!(matches!(parse("false"), LuaVal::Bool(false)));
    }

    #[test]
    fn test_simple_table() {
        let src = r#"{ ["name"] = "Umbari Robe of Beasts", ["slot"] = 2.000000 }"#;
        let tbl = match parse(src) {
            LuaVal::Table(t) => t,
            other => panic!("{:?}", other),
        };
        assert_eq!(tbl.len(), 2);
    }

    #[test]
    fn test_nested_table() {
        let src = r#"{
            ["character"] = "Thalya",
            ["items"] = {
                [1.000000] = { ["slot"] = 2.000000, ["name"] = "Robe" },
            },
        }"#;
        let tbl = match parse(src) {
            LuaVal::Table(t) => t,
            other => panic!("{:?}", other),
        };
        assert_eq!(tbl.len(), 2);
    }

    #[test]
    fn extract_export_minimal_gearlist() {
        let src = r#"{
            ["version"] = "lgo-gearlist-1",
            ["character"] = "Thalya",
            ["class"] = "Lore-master",
            ["baseStats"] = {
                ["GetBaseMight"] = 5300.000000,
                ["GetBaseAgility"] = 2650.000000,
                ["GetBaseVitality"] = 10200.000000,
                ["GetBaseWill"] = 7950.000000,
                ["GetBaseFate"] = 4000.000000,
            },
            ["names"] = {
                [1.000000] = "Item One",
                [2.000000] = "Item Two",
            },
        }"#;
        let export = extract_export(parse(src)).expect("extract_export should succeed");
        assert_eq!(export.character, "Thalya");
        assert_eq!(export.class, "Lore-master");
        assert_eq!(export.base_stats.get("Might"), Some(&5300));
        assert_eq!(export.base_stats.get("Agility"), Some(&2650));
        assert_eq!(export.base_stats.get("Vitality"), Some(&10200));
        assert_eq!(export.base_stats.get("Will"), Some(&7950));
        assert_eq!(export.base_stats.get("Fate"), Some(&4000));
    }

    /// An `lgo-gearlist-1`-shaped export lacks every measured field. Missing
    /// means `None`/empty, never a parse error.
    #[test]
    fn extract_export_without_measured_fields_yields_none() {
        let src = r#"{
            ["version"] = "lgo-gearlist-1",
            ["character"] = "Thalya",
            ["class"] = "Lore-master",
            ["names"] = {
                [1.000000] = "Item One",
            },
        }"#;
        let export = extract_export(parse(src)).expect("extract_export should succeed");
        assert_eq!(export.level, None);
        assert_eq!(export.max_morale, None);
        assert_eq!(export.max_power, None);
        assert_eq!(export.active_effects, None);
        assert!(export.equipped.is_empty());
    }

    #[test]
    fn extract_export_reads_measured_fields_and_equipped_order() {
        let src = r#"{
            ["version"] = "lgo-gearlist-2",
            ["character"] = "Thalya",
            ["class"] = "Lore-master",
            ["level"] = 160.000000,
            ["maxMorale"] = 187342.000000,
            ["maxPower"] = 21005.000000,
            ["activeEffects"] = 0.000000,
            ["equipped"] = {
                [2.000000] = "Second Item",
                [1.000000] = "First Item",
                [3.000000] = "Second Item",
            },
        }"#;
        let export = extract_export(parse(src)).expect("extract_export should succeed");
        assert_eq!(export.level, Some(160));
        assert_eq!(export.max_morale, Some(187_342));
        assert_eq!(export.max_power, Some(21_005));
        assert_eq!(export.active_effects, Some(0));
        assert_eq!(
            export.equipped,
            vec![
                "First Item".to_string(),
                "Second Item".to_string(),
                "Second Item".to_string(),
            ],
            "equipped must follow Lua array index order and keep duplicates"
        );
    }

    fn current_plugindata_fixture_path() -> std::path::PathBuf {
        let test_data = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData");
        let mut matches: Vec<std::path::PathBuf> = fs::read_dir(&test_data)
            .expect("TestData directory must be readable")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| {
                        name.starts_with("lgo_Thalya_gearNames_") && name.ends_with(".plugindata")
                    })
                    .unwrap_or(false)
            })
            .collect();
        matches.sort();
        matches.pop().expect("a Thalya plugindata fixture exists")
    }

    #[test]
    fn loads_measured_fields_from_committed_fixture() {
        let export = load(&current_plugindata_fixture_path()).expect("fixture must parse");
        assert_eq!(export.character, "Thalya");
        assert_eq!(export.class, "Lore-master");
        assert_eq!(export.level, Some(160));
        assert_eq!(export.max_morale, Some(187_342));
        assert_eq!(export.max_power, Some(21_005));
        assert_eq!(export.active_effects, Some(0));
        assert_eq!(export.equipped.len(), 19);
        assert_eq!(export.equipped[0], "Veteran Sage's Hooded Helm");
        assert_eq!(export.equipped[18], "Lore-master's Book");
        assert_eq!(
            export
                .equipped
                .iter()
                .filter(|name| *name == "Keen Pristine Madáshi Earring")
                .count(),
            2,
            "duplicate equipped names must be preserved"
        );
    }

    #[test]
    fn reordered_fixture_parses_identically() {
        let test_data = Path::new(env!("CARGO_MANIFEST_DIR")).join("TestData");
        let original = load(&test_data.join("lgo_Thalya_gearNames_20260906_025012.plugindata"))
            .expect("original fixture must parse");
        let reordered = load(
            &test_data.join("lgo_Thalya_gearNames_20260906_030000_reordered.plugindata"),
        )
        .expect("reordered fixture must parse");

        assert_eq!(reordered, original);
    }
}
