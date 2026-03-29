//! Fetch item data from lotro-wiki.com via the MediaWiki `action=parse` API.
//!
//! Endpoint:
//!   https://lotro-wiki.com/api.php?action=parse&page=Item:<name>&prop=wikitext&format=json
//!
//! The response contains wikitext with an `{{Item Tooltip}}` template.
//! We extract:
//!   - slot     — plain string e.g. "Chest"
//!   - attrib   — <br>-separated stat strings e.g. "+1,713 Vitality"
//!   - armour   — separate numeric field e.g. "10,742"
//!
//! All fetched items are stored in the caller-supplied Cache.

use std::collections::HashMap;

use crate::cache::{Cache, CachedItem};
use crate::gear::Slot;
use crate::stat::Stat;

// ?? Public entry point ????????????????????????????????????????????????????????

/// Resolve a list of item names to CachedItems, consulting the cache first
/// and falling back to the wiki for any misses.
///
/// Returns a map of name ? CachedItem for every item that could be resolved.
/// Items that could not be resolved are logged to stderr and omitted.
/// The cache is marked dirty for any new fetches; the caller must flush it.
pub fn resolve_items(names: &[String], cache: &mut Cache) -> HashMap<String, CachedItem> {
    let mut out = HashMap::new();

    for name in names {
        // 1. Cache hit
        if let Some(cached) = cache.get(name) {
            out.insert(name.clone(), cached.clone());
            continue;
        }

        // 2. Wiki fetch
        eprintln!("[wiki] Fetching: {}", name);
        match fetch_item(name) {
            Ok(item) => {
                cache.insert(item.clone());
                out.insert(name.clone(), item);
            }
            Err(e) => {
                eprintln!("[wiki] WARN: Could not resolve '{}': {}", name, e);
            }
        }
    }

    out
}

// ?? HTTP fetch + parse ?????????????????????????????????????????????????????????

fn fetch_item(name: &str) -> Result<CachedItem, String> {
    // LotRO wiki item pages are under "Item:<name>" with spaces as underscores.
    let page_title = format!("Item:{}", name.replace(' ', "_"));
    let url = format!(
        "https://lotro-wiki.com/api.php?action=parse&page={}&prop=wikitext&format=json",
        url_encode(&page_title)
    );

    let response = ureq::get(&url)
        .set("User-Agent", "lgo-optimizer/1.0 (github.com/ThalyaFlourishing/LGO)")
        .call()
        .map_err(|e| format!("HTTP error for '{}': {}", name, e))?;

    let body = response
        .into_string()
        .map_err(|e| format!("Failed to read response body for '{}': {}", name, e))?;

    parse_wiki_response(name, &body)
}

fn parse_wiki_response(name: &str, json: &str) -> Result<CachedItem, String> {
    // Check for API-level errors first.
    if json.contains(r#""error""#) {
        // Extract the info field if possible for a cleaner message.
        let info = extract_json_string(json, "info")
            .unwrap_or_else(|| "unknown API error".to_string());
        return Err(format!("Wiki API error: {}", info));
    }

    // Pull out the wikitext value (the "*" key inside "wikitext").
    let wikitext = extract_wikitext(json)
        .ok_or_else(|| format!("No wikitext found in response for '{}'", name))?;

    // Find the {{Item Tooltip}} template block.
    let template = extract_template(&wikitext, "Item Tooltip")
        .ok_or_else(|| format!("No {{{{Item Tooltip}}}} template found for '{}'", name))?;

    // Extract fields from the template.
    let slot_str = template_field(&template, "slot")
        .ok_or_else(|| format!("No 'slot' field in template for '{}'", name))?;
    let slot = parse_slot(&slot_str)
        .ok_or_else(|| format!("Unrecognised slot '{}' for '{}'", slot_str, name))?;

    let mut stats: HashMap<Stat, i64> = HashMap::new();

    // Parse the `attrib` field: "+1,713 Vitality<br> +1,050 Will<br> ..."
    if let Some(attrib) = template_field(&template, "attrib") {
        for fragment in attrib.split("<br>") {
            let fragment = fragment.trim();
            if fragment.is_empty() { continue; }
            if let Some((stat, value)) = parse_attrib_fragment(fragment) {
                // If the same stat appears twice (rare), sum the values.
                *stats.entry(stat).or_insert(0) += value;
            } else {
                eprintln!("[wiki] WARN: Could not parse attrib fragment '{}' for '{}'", fragment, name);
            }
        }
    }

    // Parse the separate `armour` field: "10,742"
    if let Some(armour_str) = template_field(&template, "armour") {
        let armour_str = armour_str.replace(',', "");
        if let Ok(v) = armour_str.trim().parse::<i64>() {
            *stats.entry(Stat::Armour).or_insert(0) += v;
        }
    }

    Ok(CachedItem {
        name: name.to_string(),
        slot,
        stats,
    })
}

// ?? Template parsing ??????????????????????????????????????????????????????????

/// Extract the raw wikitext string from the MediaWiki JSON response.
/// The value lives at .parse.wikitext["*"].
fn extract_wikitext(json: &str) -> Option<String> {
    // We use a minimal string search rather than a full JSON parser,
    // since the structure is fixed and serde_json is not yet a dependency here.
    // (serde_json IS a dependency via cache.rs, so we could use it — see below.)
    extract_json_string(json, "\\*")
        .or_else(|| extract_json_string(json, "*"))
}

/// Extract a named `{{Template Name ... }}` block from wikitext.
/// Returns the content between the outer {{ and }}.
fn extract_template<'a>(wikitext: &'a str, name: &str) -> Option<String> {
    // Match "{{Item Tooltip" or "{{item tooltip" case-insensitively.
    let needle_lower = format!("{{{{{}", name.to_lowercase());
    let wikitext_lower = wikitext.to_lowercase();
    let start = wikitext_lower.find(&needle_lower)?;

    // Walk forward counting brace depth to find the matching }}.
    let slice = &wikitext[start..];
    let mut depth = 0usize;
    let mut chars = slice.char_indices().peekable();
    let mut end = slice.len();

    while let Some((i, c)) = chars.next() {
        if c == '{' {
            if chars.peek().map(|&(_, nc)| nc) == Some('{') {
                depth += 1;
                chars.next(); // consume second '{'
            }
        } else if c == '}' {
            if chars.peek().map(|&(_, nc)| nc) == Some('}') {
                depth -= 1;
                chars.next(); // consume second '}'
                if depth == 0 {
                    end = i + 2; // include the closing }}
                    break;
                }
            }
        }
    }

    Some(wikitext[start..start + end].to_string())
}

/// Extract the value of a named field from inside a template block.
/// Handles `| field = value` with optional whitespace and multiline values.
fn template_field(template: &str, field: &str) -> Option<String> {
    // Build a pattern: "| field =" allowing flexible whitespace.
    // We search line by line since fields are typically on their own line.
    let field_lower = field.to_lowercase();

    for line in template.lines() {
        let line_trimmed = line.trim();
        if !line_trimmed.starts_with('|') { continue; }
        let rest = line_trimmed[1..].trim_start();
        // Check if the line starts with our field name followed by '='
        let eq_pos = rest.find('=')?;
        let key = rest[..eq_pos].trim().to_lowercase();
        if key == field_lower {
            let value = rest[eq_pos + 1..].trim().to_string();
            return Some(value);
        }
    }
    None
}

// ?? Stat parsing ??????????????????????????????????????????????????????????????

/// Parse a single attrib fragment such as "+1,713 Vitality" or "+6140 Tactical Mitigation".
/// Returns (Stat, value) or None if unrecognised.
fn parse_attrib_fragment(s: &str) -> Option<(Stat, i64)> {
    // Strip leading '+' or '-', remove commas, split on first space sequence
    // after the number.
    let s = s.trim();

    // Find where the numeric part ends and the stat name begins.
    // Format is always: [+-]digits[,digits]* <stat name>
    let sign: i64 = if s.starts_with('-') { -1 } else { 1 };
    let s = s.trim_start_matches('+').trim_start_matches('-').trim_start();

    // Find the end of the number (digits and commas).
    let num_end = s.find(|c: char| !c.is_ascii_digit() && c != ',')
        .unwrap_or(s.len());
    if num_end == 0 { return None; }

    let num_str = s[..num_end].replace(',', "");
    let value: i64 = num_str.parse().ok()?;
    let stat_name = s[num_end..].trim();

    // Map wiki stat name to our Stat enum.
    let stat = parse_wiki_stat_name(stat_name)?;

    Some((stat, sign * value))
}

/// Map the human-readable stat name from the wiki to our Stat enum.
/// These names come directly from the `attrib` field of {{Item Tooltip}}.
fn parse_wiki_stat_name(s: &str) -> Option<Stat> {
    match s.to_lowercase().replace([' ', '-'], "").as_str() {
        "vitality"                                  => Some(Stat::Vitality),
        "will"                                      => Some(Stat::Will),
        "might"                                     => Some(Stat::Might),
        "agility"                                   => Some(Stat::Agility),
        "fate"                                      => Some(Stat::Fate),
        "morale"                                    => Some(Stat::Morale),
        "power"                                     => Some(Stat::Power),
        "criticalrating" | "critrating"             => Some(Stat::CritRating),
        "devastatingcriticalrating" | "devrating"   => Some(Stat::DevRating),
        "finesserating" | "finesse"                 => Some(Stat::FinesseRating),
        "tacticalmastery"                           => Some(Stat::TacticalMastery),
        "physicalmastery"                           => Some(Stat::PhysicalMastery),
        "offensiveoverpower"                        => Some(Stat::OffensiveOverpower),
        "resistance"                                => Some(Stat::Resistance),
        "criticaldefencerating" |
        "criticaldefense" |
        "critdefence"                               => Some(Stat::CritDefence),
        "tacticalmitigation"                        => Some(Stat::TacticalMitigation),
        "physicalmitigation"                        => Some(Stat::PhysicalMitigation),
        "incominghealing"                           => Some(Stat::IncomingHealing),
        "outgoinghealing"                           => Some(Stat::OutgoingHealing),
        "incomingmitigations" |
        "incomitigations" |
        "incmitigations"                            => Some(Stat::IncMitigations),
        // Armour handled separately via the `armour` field, not attrib.
        // Unknown stats are silently skipped.
        _                                           => None,
    }
}

// ?? Slot parsing ??????????????????????????????????????????????????????????????

/// Map the wiki slot string to our Slot enum.
fn parse_slot(s: &str) -> Option<Slot> {
    match s.trim().to_lowercase().as_str() {
        "head"                          => Some(Slot::Head),
        "chest"                         => Some(Slot::Chest),
        "legs"                          => Some(Slot::Legs),
        "hands" | "gloves"              => Some(Slot::Hands),
        "feet" | "boots"                => Some(Slot::Feet),
        "shoulders"                     => Some(Slot::Shoulders),
        "back" | "cloak"                => Some(Slot::Back),
        "wrist" | "wrist1" | "bracelet" => Some(Slot::Wrist1),
        "neck" | "necklace"             => Some(Slot::Neck),
        "finger" | "ring"               => Some(Slot::Finger1),
        "ear" | "earring"               => Some(Slot::Ear1),
        "pocket"                        => Some(Slot::Pocket),
        "off-hand" | "offhand"          => Some(Slot::OffHand),
        "ranged"                        => Some(Slot::Ranged),
        _                               => None,
    }
}

// ?? Minimal JSON string extraction ???????????????????????????????????????????

/// Extract the string value of a JSON key by simple pattern matching.
/// Handles the common case: `"key": "value"` where value may contain
/// escaped characters. Not a full JSON parser — used only for the
/// small, well-known fields we need.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let start = json.find(&needle)?;
    let after_key = &json[start + needle.len()..];
    let colon_pos = after_key.find(':')? ;
    let after_colon = after_key[colon_pos + 1..].trim_start();
    if !after_colon.starts_with('"') { return None; }
    let content = &after_colon[1..];

    let mut result = String::new();
    let mut chars = content.char_indices().peekable();
    loop {
        match chars.next()? {
            (_, '"')  => return Some(result),
            (_, '\\') => match chars.next()?.1 {
                'n'  => result.push('\n'),
                'r'  => result.push('\r'),
                't'  => result.push('\t'),
                '"'  => result.push('"'),
                '\\' => result.push('\\'),
                '/'  => result.push('/'),
                c    => { result.push('\\'); result.push(c); }
            },
            (_, c) => result.push(c),
        }
    }
}

// ?? URL encoding ?????????????????????????????????????????????????????????????

/// Percent-encode a string for use in a URL query parameter.
/// Only encodes characters that are not safe in URLs.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' | b':' | b'/' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(char::from_digit((b >> 4) as u32, 16).unwrap().to_ascii_uppercase());
                out.push(char::from_digit((b & 0xf) as u32, 16).unwrap().to_ascii_uppercase());
            }
        }
    }
    out
}

// ?? Tests ?????????????????????????????????????????????????????????????????????

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_attrib_fragment_simple() {
        let (stat, val) = parse_attrib_fragment("+1,713 Vitality").unwrap();
        assert_eq!(stat, Stat::Vitality);
        assert_eq!(val, 1713);
    }

    #[test]
    fn test_parse_attrib_fragment_no_comma() {
        let (stat, val) = parse_attrib_fragment("+6140 Tactical Mitigation").unwrap();
        assert_eq!(stat, Stat::TacticalMitigation);
        assert_eq!(val, 6140);
    }

    #[test]
    fn test_parse_attrib_fragment_will() {
        let (stat, val) = parse_attrib_fragment("+1,050 Will").unwrap();
        assert_eq!(stat, Stat::Will);
        assert_eq!(val, 1050);
    }

    #[test]
    fn test_parse_slot() {
        assert_eq!(parse_slot("Chest"),   Some(Slot::Chest));
        assert_eq!(parse_slot("Head"),    Some(Slot::Head));
        assert_eq!(parse_slot("Pocket"),  Some(Slot::Pocket));
        assert_eq!(parse_slot("Unknown"), None);
    }

    #[test]
    fn test_template_field() {
        let template = "{{Item Tooltip\n| slot = Chest\n| attrib = +100 Vitality\n}}";
        assert_eq!(template_field(template, "slot"),   Some("Chest".to_string()));
        assert_eq!(template_field(template, "attrib"), Some("+100 Vitality".to_string()));
        assert_eq!(template_field(template, "missing"), None);
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("Item:Umbari Robe of Beasts"),
                   "Item:Umbari+Robe+of+Beasts");
    }
}