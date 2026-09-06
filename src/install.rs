//! Install-directory anchoring and character-folder discovery.
//!
//! Every input and output path resolves from the *install directory* — the
//! directory that contains `lgo.exe` — never the current working directory.
//! The dev/test escape hatch `LGO_HOME` overrides the install directory so
//! `cargo run` and `cargo test` work without a `data/` folder under `target/`.
//!
//! Install tree layout:
//!
//! ```text
//! <install>\
//!   lgo.exe
//!   data\                         (items.xml, lgo_items.json, ...)
//!   <CharacterName>_Gear\         (one folder per character)
//!     lgo_<char>_gearStats.toml
//!     lgo_<char>_gearReady.toml
//!     lgo_<char>_builds.toml
//!     <CharacterName>_Reports\    (all optimize/scrap reports)
//! ```

use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{env, fs, io};

/// Dev/test-only environment variable that overrides the install directory.
pub const LGO_HOME_ENV: &str = "LGO_HOME";

const GEAR_SUFFIX: &str = "_Gear";
const REPORTS_SUFFIX: &str = "_Reports";

/// A discovered `<character>_Gear` folder directly under the install directory.
#[derive(Debug, Clone)]
pub struct GearFolder {
    /// The `<character>` portion, with the folder's on-disk casing preserved.
    pub character: String,
    /// Absolute path to `<install>/<character>_Gear`.
    pub path: PathBuf,
}

/// The character selected for a command, plus an optional note to print.
#[derive(Debug, Clone)]
pub struct Selection {
    /// Directory-derived character name (authoritative for discovery).
    pub character: String,
    /// Path to the character's `<character>_Gear` folder.
    pub gear_dir: PathBuf,
    /// Message the caller should print (e.g. the auto-selection notice).
    pub note: Option<String>,
}

/// The install directory: `LGO_HOME` when set and non-empty, otherwise the
/// directory that contains the running executable; an empty `LGO_HOME` is
/// treated as unset, and `.cargo/config.toml` deliberately relies on a
/// non-empty relative `LGO_HOME` during dev/test so `cargo run` and
/// `cargo test` anchor at the repo root instead. Never the working directory.
pub fn install_dir() -> io::Result<PathBuf> {
    if let Some(home) = env::var_os(LGO_HOME_ENV) {
        if !home.is_empty() {
            return Ok(PathBuf::from(home));
        }
    }
    let exe = env::current_exe()?;
    exe.parent().map(Path::to_path_buf).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "executable path has no parent directory",
        )
    })
}

/// `<install>/data`.
pub fn data_dir() -> io::Result<PathBuf> {
    Ok(install_dir()?.join("data"))
}

/// `<install>/data/<file_name>`.
pub fn data_path(file_name: &str) -> io::Result<PathBuf> {
    Ok(data_dir()?.join(file_name))
}

/// `<install>/<character>_Gear`.
pub fn gear_dir(install: &Path, character: &str) -> PathBuf {
    install.join(format!("{}{}", character, GEAR_SUFFIX))
}

/// `<install>/<character>_Gear/<character>_Reports`.
pub fn reports_dir(install: &Path, character: &str) -> PathBuf {
    gear_dir(install, character).join(format!("{}{}", character, REPORTS_SUFFIX))
}

/// If `name` is `<character>_Gear` (suffix matched case-insensitively), return
/// the `<character>` portion with its original casing.
fn strip_gear_suffix(name: &str) -> Option<&str> {
    let lower = name.to_ascii_lowercase();
    let suffix_lower = GEAR_SUFFIX.to_ascii_lowercase();
    if lower.len() > suffix_lower.len() && lower.ends_with(&suffix_lower) {
        Some(&name[..name.len() - GEAR_SUFFIX.len()])
    } else {
        None
    }
}

/// List all `<character>_Gear` folders directly under `install`, sorted by
/// character name (case-insensitively). A missing install directory yields an
/// empty list rather than an error.
pub fn list_gear_folders(install: &Path) -> io::Result<Vec<GearFolder>> {
    let mut folders = Vec::new();
    let read_dir = match fs::read_dir(install) {
        Ok(rd) => rd,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(folders),
        Err(e) => return Err(e),
    };
    for entry in read_dir {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if let Some(character) = strip_gear_suffix(&name) {
            if character.is_empty() {
                continue;
            }
            folders.push(GearFolder {
                character: character.to_string(),
                path: entry.path(),
            });
        }
    }
    folders.sort_by(|a, b| {
        a.character
            .to_ascii_lowercase()
            .cmp(&b.character.to_ascii_lowercase())
    });
    Ok(folders)
}

/// Select a character for a read-only command (`optimize`, `scrap-gear`,
/// `base-stats`) using the discovery rules. Errors clearly on no match / zero
/// folders.
pub fn select_character(install: &Path, requested: Option<&str>) -> Result<Selection, String> {
    let folders = list_gear_folders(install)
        .map_err(|e| format!("Cannot read install directory {}: {}", install.display(), e))?;

    if let Some(req) = requested {
        return match folders
            .iter()
            .find(|f| f.character.eq_ignore_ascii_case(req))
        {
            Some(f) => Ok(Selection {
                character: f.character.clone(),
                gear_dir: f.path.clone(),
                note: None,
            }),
            None => Err(no_matching_folder_message(install, req)),
        };
    }

    if folders.is_empty() {
        return Err(zero_folder_message(install));
    }
    Ok(select_among(&folders))
}

/// Resolve-slots preparation: relocate any stray `lgo_<char>_gearStats.toml`
/// files from the install root into their `<char>_Gear` folder, then select or
/// create the target folder. Prints move and auto-selection messages.
pub fn prepare_resolve_slots(install: &Path, requested: Option<&str>) -> Result<Selection, String> {
    relocate_stray_gear_stats(install).map_err(|e| {
        format!(
            "Cannot relocate stray gear files in {}: {}",
            install.display(),
            e
        )
    })?;

    let folders = list_gear_folders(install)
        .map_err(|e| format!("Cannot read install directory {}: {}", install.display(), e))?;

    if let Some(req) = requested {
        if let Some(f) = folders
            .iter()
            .find(|f| f.character.eq_ignore_ascii_case(req))
        {
            return Ok(Selection {
                character: f.character.clone(),
                gear_dir: f.path.clone(),
                note: None,
            });
        }
        let dir = gear_dir(install, req);
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Cannot create gear folder {}: {}", dir.display(), e))?;
        return Ok(Selection {
            character: req.to_string(),
            gear_dir: dir,
            note: None,
        });
    }

    if folders.is_empty() {
        return Err(resolve_slots_zero_message(install));
    }
    Ok(select_among(&folders))
}

/// Select among one-or-more folders: single → that one; multiple → the one
/// whose `lgo_<char>_gearReady.toml` is most recently modified, with a note.
/// Panics if `folders` is empty (callers guard against that).
fn select_among(folders: &[GearFolder]) -> Selection {
    if folders.len() == 1 {
        let f = &folders[0];
        return Selection {
            character: f.character.clone(),
            gear_dir: f.path.clone(),
            note: None,
        };
    }
    let chosen = most_recently_updated(folders);
    Selection {
        character: chosen.character.clone(),
        gear_dir: chosen.path.clone(),
        note: Some(format!(
            "Using character: {} (most recently updated)",
            chosen.character
        )),
    }
}

fn gear_ready_mtime(folder: &GearFolder) -> Option<SystemTime> {
    let path =
        crate::gearstats::find_canonical_gear_file(&folder.path, &folder.character).ok()??;
    fs::metadata(&path).and_then(|m| m.modified()).ok()
}

fn most_recently_updated(folders: &[GearFolder]) -> &GearFolder {
    folders
        .iter()
        .max_by(|a, b| {
            let name_order = || {
                a.character
                    .to_ascii_lowercase()
                    .cmp(&b.character.to_ascii_lowercase())
            };
            match (gear_ready_mtime(a), gear_ready_mtime(b)) {
                (Some(x), Some(y)) => x.cmp(&y).then_with(name_order),
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (None, None) => name_order(),
            }
        })
        .expect("most_recently_updated requires a non-empty slice")
}

/// Move any loose `lgo_<char>_gearStats.toml` files in the install root into
/// their `<char>_Gear` folder (creating it), printing a message for each move.
/// Returns the characters whose files were moved.
pub fn relocate_stray_gear_stats(install: &Path) -> io::Result<Vec<String>> {
    let mut moved = Vec::new();
    let read_dir = match fs::read_dir(install) {
        Ok(rd) => rd,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(moved),
        Err(e) => return Err(e),
    };

    let mut strays: Vec<(String, PathBuf)> = Vec::new();
    for entry in read_dir {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if let Some(character) = stray_gear_stats_character(&name) {
            strays.push((character, entry.path()));
        }
    }

    for (character, src) in strays {
        let dest_dir = gear_dir(install, &character);
        fs::create_dir_all(&dest_dir)?;
        let file_name = src
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("lgo_{}_gearStats.toml", character)));
        let dest = dest_dir.join(&file_name);
        fs::rename(&src, &dest)?;
        println!(
            "Moved stray {} into {}",
            file_name.display(),
            dest_dir.display()
        );
        moved.push(character);
    }
    Ok(moved)
}

/// Extract `<char>` from a loose `lgo_<char>_gearStats.toml` filename.
fn stray_gear_stats_character(name: &str) -> Option<String> {
    embedded_character(name, "_gearstats.toml")
}

/// Warn (to stderr) when a `<character>_Gear` folder holds gear files named for
/// a different character. Discovery still proceeds on the folder name.
pub fn warn_on_name_mismatch(gear_dir: &Path, character: &str) {
    let read_dir = match fs::read_dir(gear_dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    let mut mismatched: Vec<String> = Vec::new();
    for entry in read_dir.flatten() {
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if let Some(embedded) = embedded_character_any(&name) {
            if !embedded.eq_ignore_ascii_case(character)
                && !mismatched.iter().any(|m| m.eq_ignore_ascii_case(&embedded))
            {
                mismatched.push(embedded);
            }
        }
    }
    if !mismatched.is_empty() {
        mismatched.sort();
        eprintln!(
            "WARNING: gear folder {} is for character '{}' but contains files named for: {}. \
Proceeding with the folder name '{}'; rename the folder or the files if this is wrong.",
            gear_dir.display(),
            character,
            mismatched.join(", "),
            character
        );
    }
}

/// Resolve the character used to route reports for a (possibly out-of-tree)
/// gear TOML: the top-level `character = "..."` field first, then the canonical
/// `lgo_<char>_gearReady.toml` filename convention.
pub fn resolve_report_character(
    gear_file: &Path,
    gear_doc_character: Option<&str>,
) -> Result<String, String> {
    if let Some(c) = gear_doc_character.map(str::trim).filter(|c| !c.is_empty()) {
        return Ok(c.to_string());
    }
    if let Some(name) = gear_file.file_name().and_then(|n| n.to_str()) {
        if let Some(c) = embedded_character(name, "_gearready.toml") {
            return Ok(c);
        }
    }
    Err(format!(
        "Cannot determine the character for report routing from {}. \
Add a top-level `character = \"...\"` field or use a canonical `lgo_<character>_gearReady.toml` filename.",
        gear_file.display()
    ))
}

/// Extract `<char>` from `lgo_<char><suffix>` (suffix given lowercase).
fn embedded_character(name: &str, suffix_lower: &str) -> Option<String> {
    let prefix = "lgo_";
    let lower = name.to_ascii_lowercase();
    if lower.starts_with(prefix)
        && lower.ends_with(suffix_lower)
        && lower.len() > prefix.len() + suffix_lower.len()
    {
        Some(name[prefix.len()..name.len() - suffix_lower.len()].to_string())
    } else {
        None
    }
}

/// Extract `<char>` from any recognised gear filename.
fn embedded_character_any(name: &str) -> Option<String> {
    for suffix in ["_gearstats.toml", "_gearready.toml", "_builds.toml"] {
        if let Some(c) = embedded_character(name, suffix) {
            return Some(c);
        }
    }
    None
}

fn no_matching_folder_message(install: &Path, requested: &str) -> String {
    format!(
        "No gear folder for character '{}' found in {}.\n\
Expected a folder named '{}{}'. Save the bookmarklet output as \
lgo_{}_gearStats.toml and run 'lgo resolve-slots' to create it.",
        requested,
        install.display(),
        requested,
        GEAR_SUFFIX,
        requested
    )
}

fn zero_folder_message(install: &Path) -> String {
    format!(
        "Install directory:\n{}\nNo character gear folders found.\nExpected one \
'<CharacterName>_Gear' folder per character, each containing \
lgo_<character>_gearReady.toml.\nRun 'lgo resolve-slots' after saving the \
bookmarklet output (lgo_<character>_gearStats.toml) to create it.",
        install.display()
    )
}

fn resolve_slots_zero_message(install: &Path) -> String {
    format!(
        "Install directory:\n{}\nNo gear folders or loose gearStats files found.\n\
Save the bookmarklet output as lgo_<character>_gearStats.toml into that \
directory (or into a '<CharacterName>_Gear' folder), then re-run \
'lgo resolve-slots'.",
        install.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "lgo_install_test_{}_{}",
            tag,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write file");
    }

    #[test]
    fn install_dir_prefers_lgo_home() {
        // The process-wide LGO_HOME (set via .cargo/config.toml in dev/test)
        // must win over the exe directory.
        let got = install_dir().expect("install dir");
        let expected = PathBuf::from(env::var_os(LGO_HOME_ENV).expect("LGO_HOME set in tests"));
        assert_eq!(got, expected);
    }

    #[test]
    fn single_folder_auto_selects() {
        let install = temp_dir("single");
        fs::create_dir_all(install.join("Thalya_Gear")).unwrap();
        let sel = select_character(&install, None).expect("select");
        assert_eq!(sel.character, "Thalya");
        assert_eq!(sel.gear_dir, install.join("Thalya_Gear"));
        assert!(sel.note.is_none());
        fs::remove_dir_all(&install).ok();
    }

    #[test]
    fn requested_matches_case_insensitively() {
        let install = temp_dir("ci_match");
        fs::create_dir_all(install.join("Thalya_Gear")).unwrap();
        let sel = select_character(&install, Some("THALYA")).expect("select");
        assert_eq!(sel.character, "Thalya");
        fs::remove_dir_all(&install).ok();
    }

    #[test]
    fn requested_missing_errors() {
        let install = temp_dir("missing");
        fs::create_dir_all(install.join("Thalya_Gear")).unwrap();
        let err = select_character(&install, Some("Legolas")).unwrap_err();
        assert!(err.contains("Legolas_Gear"), "got: {err}");
        fs::remove_dir_all(&install).ok();
    }

    #[test]
    fn zero_folders_errors() {
        let install = temp_dir("zero");
        let err = select_character(&install, None).unwrap_err();
        let expected_prefix = format!("Install directory:\n{}\n", install.display());
        assert!(err.starts_with(&expected_prefix), "got: {err}");
        assert!(err.contains("No character gear folders"), "got: {err}");
        fs::remove_dir_all(&install).ok();
    }

    #[test]
    fn resolve_slots_zero_message_starts_with_install_directory() {
        let install = temp_dir("resolve_zero");
        let err = prepare_resolve_slots(&install, None).unwrap_err();
        let expected_prefix = format!("Install directory:\n{}\n", install.display());
        assert!(err.starts_with(&expected_prefix), "got: {err}");
        assert!(
            err.contains("No gear folders or loose gearStats files found"),
            "got: {err}"
        );
        fs::remove_dir_all(&install).ok();
    }

    /// Rewrite `path` until its mtime is strictly greater than `floor`, so the
    /// recency comparison is deterministic regardless of filesystem mtime
    /// granularity (and without pulling in an extra crate).
    fn bump_mtime_past(path: &Path, floor: SystemTime) {
        loop {
            fs::write(path, "bump\n").expect("rewrite");
            let m = fs::metadata(path)
                .and_then(|m| m.modified())
                .expect("mtime");
            if m > floor {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn multiple_folders_pick_recent_gear_ready() {
        let install = temp_dir("multi");
        let older = install.join("Alpha_Gear/lgo_Alpha_gearReady.toml");
        let newer = install.join("Beta_Gear/lgo_Beta_gearReady.toml");
        write(&older, "character = \"Alpha\"\n");
        write(&newer, "character = \"Beta\"\n");
        // Force Beta to be strictly newer than Alpha.
        let older_mtime = fs::metadata(&older)
            .and_then(|m| m.modified())
            .expect("older mtime");
        bump_mtime_past(&newer, older_mtime);
        let sel = select_character(&install, None).expect("select");
        assert_eq!(sel.character, "Beta");
        assert_eq!(
            sel.note.as_deref(),
            Some("Using character: Beta (most recently updated)")
        );
        fs::remove_dir_all(&install).ok();
    }

    #[test]
    fn stray_gear_stats_is_relocated() {
        let install = temp_dir("stray");
        let stray = install.join("lgo_Thalya_gearStats.toml");
        write(&stray, "character = \"Thalya\"\n");
        let moved = relocate_stray_gear_stats(&install).expect("relocate");
        assert_eq!(moved, vec!["Thalya".to_string()]);
        assert!(!stray.exists(), "stray file should be moved");
        assert!(install
            .join("Thalya_Gear/lgo_Thalya_gearStats.toml")
            .exists());
        fs::remove_dir_all(&install).ok();
    }

    #[test]
    fn resolve_report_character_prefers_field_then_filename() {
        assert_eq!(
            resolve_report_character(Path::new("whatever.toml"), Some("Thalya")).unwrap(),
            "Thalya"
        );
        assert_eq!(
            resolve_report_character(Path::new("lgo_Bilbo_gearReady.toml"), None).unwrap(),
            "Bilbo"
        );
        assert!(resolve_report_character(Path::new("mystery.toml"), None).is_err());
    }

    #[test]
    fn reports_dir_is_nested_in_gear_folder() {
        let install = Path::new("/install");
        assert_eq!(
            reports_dir(install, "Thalya"),
            install.join("Thalya_Gear").join("Thalya_Reports")
        );
    }
}
