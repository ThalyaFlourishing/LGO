//! Offline item database loader.
//!
//! At startup, LGO checks for `lgo_items.json` (produced by the `db_build`
//! binary). If found, it is loaded and used as the primary item source,
//! bypassing wiki lookups for all items it covers.
//!
//! The file format is identical to `lgo_cache.json` — a JSON object mapping
//! item name (String) ? CachedItem — so no separate parsing logic is needed.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::cache::CachedItem;

/// Load the offline item database from `path`, if it exists.
///
/// - Returns `Ok(Some(map))` on success.
/// - Returns `Ok(None)` if the file does not exist (not an error; wiki fallback applies).
/// - Returns `Err(String)` if the file exists but cannot be read or parsed.
pub fn load_item_db(path: &Path) -> Result<Option<HashMap<String, CachedItem>>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read item db '{}': {}", path.display(), e))?;

    let map: HashMap<String, CachedItem> = serde_json::from_str(&json)
        .map_err(|e| format!("Cannot parse item db '{}': {}", path.display(), e))?;

    eprintln!(
        "[db] Loaded {} items from offline database '{}'.",
        map.len(),
        path.display()
    );
    Ok(Some(map))
}