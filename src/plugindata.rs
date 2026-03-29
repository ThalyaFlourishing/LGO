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
//! Output: a `PluginExport` containing two flat lists of `PartialItem`:
//!   - equipped:    items currently worn; slot resolved from plugin index
//!   - candidates:  items from the 'lgo' chest; slot resolved later by wiki

use std::fs;
use std::path::Path;

use crate::gear::Slot;

// ?? Public types ??????????????????????????????????????????????????????????????

/// All item data extracted from the plugin export file, before wiki lookup.
#[derive(Debug)]
pub struct PluginExport {
    pub character: String,
    /// Items currently worn by the character. Slot is known from plugin index.
    pub equipped: Vec<PartialItem>,
    /// Candidate items from the 'lgo' shared storage chest.
    /// Slot is None here — filled in by wiki lookup.
    pub candidates: Vec<PartialItem>,
}

/// One item as it comes out of the plugin file, before stat resolution.
#[derive(Debug, Clone)]
pub struct PartialItem {
    pub name: String,
    /// Some for equipped items (resolved from plugin slot index).
    /// None for storage candidates (resolved later from wiki data).
    pub slot: Option<Slot>,
}

// ?? Entry point ???????????????????????????????????????????????????????????????

pub fn load(path: &Path) -> Result<PluginExport, String> {
    let src = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;

    // Files start with "return " followed by the table literal.
    let src = src.trim();
    let src = src.strip_prefix("return").unwrap_or(src).trim_start();

    let (val, _) = parse_value(src, 0)
        .map_err(|e| format!("Parse error in {}: {}", path.display(), e))?;

    extract_export(val)
}

// ?? Extraction ????????????????????????????????????????????????????????????????

fn extract_export(val: LuaVal) -> Result<PluginExport, String> {
    let root = expect_table(val, "root")?;

    let character = match table_get(&root, "character") {
        Some(LuaVal::Str(s)) => s,
        _ => "Unknown".to_string(),
    };

    let equipped_val = table_get(&root, "equipped")
        .ok_or_else(|| "Missing 'equipped' key in export".to_string())?;
    let ss_val = table_get(&root, "sharedStorage")
        .ok_or_else(|| "Missing 'sharedStorage' key in export".to_string())?;

    let equipped   = extract_item_list(equipped_val,  false)?;
    let candidates = extract_item_list(ss_val,        true)?;

    Ok(PluginExport { character, equipped, candidates })
}

fn extract_item_list(val: LuaVal, is_storage: bool) -> Result<Vec<PartialItem>, String> {
    let tbl = expect_table(val, "equipped/sharedStorage block")?;

    let items_val = table_get(&tbl, "items")
        .ok_or_else(|| "Missing 'items' key in block".to_string())?;
    let items_tbl = expect_table(items_val, "items array")?;

    let mut out = Vec::new();

    for (_, item_val) in items_tbl {
        let item_tbl = match item_val {
            LuaVal::Table(t) => t,
            _ => continue,
        };

        // Name is required.
        let name = match table_get(&item_tbl, "name") {
            Some(LuaVal::Str(s)) if !s.is_empty() => s,
            _ => continue,
        };

        let slot = if is_storage {
            // Storage items: the 'slot' field is the storage array index,
            // not the equipment slot. Actual slot comes from wiki lookup.
            None
        } else {
            // Equipped items: 'slot' is the LotRO equipment slot index (1–21),
            // stored as a float (e.g. 12.000000).
            match table_get(&item_tbl, "slot") {
                Some(LuaVal::Num(n)) => {
                    let idx = n as u32;
                    match Slot::from_plugin_index(idx) {
                        Some(s) => Some(s),
                        None => continue, // excluded slot — skip silently
                    }
                }
                _ => continue,
            }
        };

        out.push(PartialItem { name, slot });
    }

    Ok(out)
}

// ?? Raw Lua value types ???????????????????????????????????????????????????????

#[derive(Debug, Clone)]
enum LuaVal {
    Str(String),
    Num(f64),
    Bool(bool),
    Table(LuaTable),
    Nil,
}

type LuaTable = Vec<(LuaKey, LuaVal)>;

#[derive(Debug, Clone)]
enum LuaKey {
    Str(String),
    Num(f64),
}

// ?? Table helpers ?????????????????????????????????????????????????????????????

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

// ?? Recursive descent parser ??????????????????????????????????????????????????

const MAX_DEPTH: usize = 64;

fn skip_ws_and_comments(s: &str) -> &str {
    let mut s = s;
    loop {
        s = s.trim_start();
        if s.starts_with("--") {
            s = match s.find('\n') {
                Some(i) => &s[i + 1..],
                None    => "",
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

    if s.starts_with('{') {
        parse_table(&s[1..], depth)
    } else if s.starts_with('"') {
        parse_quoted_string(s)
    } else if s.starts_with("true") {
        Ok((LuaVal::Bool(true), &s[4..]))
    } else if s.starts_with("false") {
        Ok((LuaVal::Bool(false), &s[5..]))
    } else if s.starts_with("nil") {
        Ok((LuaVal::Nil, &s[3..]))
    } else {
        parse_number(s)
    }
}

fn parse_table(s: &str, depth: usize) -> Result<(LuaVal, &str), String> {
    let mut s = skip_ws_and_comments(s);
    let mut entries: LuaTable = Vec::new();

    loop {
        s = skip_ws_and_comments(s);

        if s.starts_with('}') {
            return Ok((LuaVal::Table(entries), &s[1..]));
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
    if s.starts_with('[') {
        let inner = skip_ws_and_comments(&s[1..]);
        if inner.starts_with('"') {
            let (val, rest) = parse_quoted_string(inner)?;
            let rest = skip_ws_and_comments(rest);
            let rest = rest.strip_prefix(']')
                .ok_or_else(|| "Expected ']' after string key".to_string())?;
            if let LuaVal::Str(k) = val {
                return Ok((LuaKey::Str(k), rest));
            }
            unreachable!()
        } else {
            let (val, rest) = parse_number(inner)?;
            let rest = skip_ws_and_comments(rest);
            let rest = rest.strip_prefix(']')
                .ok_or_else(|| "Expected ']' after numeric key".to_string())?;
            if let LuaVal::Num(n) = val {
                return Ok((LuaKey::Num(n), rest));
            }
            unreachable!()
        }
    }

    // Bare identifier key (e.g. `version = ...`)
    let end = s.find(|c: char| !c.is_alphanumeric() && c != '_')
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
                Some((_, 'n'))  => result.push('\n'),
                Some((_, 'r'))  => result.push('\r'),
                Some((_, 't'))  => result.push('\t'),
                Some((_, '"'))  => result.push('"'),
                Some((_, '\\')) => result.push('\\'),
                Some((_, c))    => { result.push('\\'); result.push(c); }
                None => return Err("Unterminated escape sequence".into()),
            },
            Some((_, c)) => result.push(c),
        }
    }
}

fn parse_number(s: &str) -> Result<(LuaVal, &str), String> {
    // Consume an optional leading minus then digits, dot, exponent.
    let end = s.find(|c: char| {
        !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E' && c != '+'
    }).unwrap_or(s.len());

    if end == 0 {
        return Err(format!("Expected number, found: {:?}", &s[..s.len().min(30)]));
    }
    let n: f64 = s[..end].parse()
        .map_err(|_| format!("Invalid number literal: '{}'", &s[..end]))?;
    Ok((LuaVal::Num(n), &s[end..]))
}

// ?? Tests ?????????????????????????????????????????????????????????????????????

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
        assert!(matches!(parse("true"),  LuaVal::Bool(true)));
        assert!(matches!(parse("false"), LuaVal::Bool(false)));
    }

    #[test]
    fn test_simple_table() {
        let src = r#"{ ["name"] = "Umbari Robe of Beasts", ["slot"] = 2.000000 }"#;
        let tbl = match parse(src) {
            LuaVal::Table(t) => t,
            other => panic!("{:?}", other),
        };
        assert_eq!(
            table_get(&tbl, "name"),
            Some(LuaVal::Str("Umbari Robe of Beasts".into()))
                .as_ref().map(|_| ()).map(|_| LuaVal::Str("Umbari Robe of Beasts".into()))
        );
        // Just check it parses without error and has two entries
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
    fn slot_from_index_roundtrip() {
        assert_eq!(Slot::from_plugin_index(1),  Some(Slot::Head));
        assert_eq!(Slot::from_plugin_index(15), Some(Slot::Pocket));
        assert_eq!(Slot::from_plugin_index(17), Some(Slot::OffHand));
        assert_eq!(Slot::from_plugin_index(18), Some(Slot::Ranged));
        assert_eq!(Slot::from_plugin_index(16), None); // MainHand excluded
        assert_eq!(Slot::from_plugin_index(19), None); // CraftItem excluded
    }
}