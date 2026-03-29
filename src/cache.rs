//! Local disk cache for item data fetched from the wiki.
//!
//! Cache file: `lgo_cache.json` in the same directory as the plugindata file,
//! or in the current working directory if no path is specified.
//!
//! Format: a JSON object mapping item name (String) to CachedItem.
//!
//! The cache is loaded once at startup, consulted before any wiki request,
//! and written back to disk after any new items are added.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::gear::Slot;
use crate::stat::Stat;

// ?? Types ?????????????????????????????????????????????????????????????????????

/// One item's resolved data as stored in the cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedItem {
    pub name: String,
    pub slot: Slot,
    /// Stat values keyed by stat name. Only stats present on the item are
    /// stored; missing stats are treated as 0 by the optimizer.
    pub stats: HashMap<Stat, i64>,
}

/// The in-memory cache, backed by a JSON file on disk.
#[derive(Debug)]
pub struct Cache {
    path: PathBuf,
    items: HashMap<String, CachedItem>,
    dirty: bool,
}

// ?? Public API ????????????????????????????????????????????????????????????????

impl Cache {
    /// Create a new empty cache at the given path without reading from disk.
    pub fn empty(path: &Path) -> Self {
        Cache { path: path.to_path_buf(), items: HashMap::new(), dirty: false }

    }    /// Load the cache from `path`, or start empty if the file does not exist.
    pub fn load(path: &Path) -> Result<Self, String> {
        if path.exists() {
            let src = fs::read_to_string(path)
                .map_err(|e| format!("Cannot read cache {}: {}", path.display(), e))?;
            let items: HashMap<String, CachedItem> = serde_json::from_str(&src)
                .map_err(|e| format!("Malformed cache {}: {}", path.display(), e))?;
            eprintln!(
                "[cache] Loaded {} item(s) from {}",
                items.len(),
                path.display()
            );
            Ok(Cache { path: path.to_path_buf(), items, dirty: false })
        } else {
            eprintln!("[cache] No cache file at {} — starting empty", path.display());
            Ok(Cache { path: path.to_path_buf(), items: HashMap::new(), dirty: false })
        }
    }

    /// Look up an item by name. Returns None if not cached.
    pub fn get(&self, name: &str) -> Option<&CachedItem> {
        self.items.get(name)
    }

    /// Insert or replace an item. Marks the cache dirty.
    pub fn insert(&mut self, item: CachedItem) {
        self.items.insert(item.name.clone(), item);
        self.dirty = true;
    }

    /// Write the cache to disk if it has been modified since last save.
    /// Safe to call unconditionally — does nothing if not dirty.
    pub fn flush(&self) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        let json = serde_json::to_string_pretty(&self.items)
            .map_err(|e| format!("Cannot serialise cache: {}", e))?;
        // Write to a temp file then rename for atomicity.
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, &json)
            .map_err(|e| format!("Cannot write cache temp file {}: {}", tmp.display(), e))?;
        fs::rename(&tmp, &self.path)
            .map_err(|e| format!("Cannot rename cache file: {}", e))?;
        eprintln!(
            "[cache] Saved {} item(s) to {}",
            self.items.len(),
            self.path.display()
        );
        Ok(())
    }

    /// How many items are currently cached.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Return the standard cache path given the directory containing the
/// plugindata file (or the current directory if None).
pub fn default_cache_path(plugindata_dir: Option<&Path>) -> PathBuf {
    let dir = plugindata_dir.unwrap_or_else(|| Path::new("."));
    dir.join("lgo_cache.json")
}