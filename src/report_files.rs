//! Report file output.

use std::fs;
use std::path::{Path, PathBuf};

const STEM_PREFIX: &str = "lgo_GearReport_";
const SCRAP_GEAR_STEM_PREFIX: &str = "lgo_ScrapGearReport_";

#[derive(Debug, Clone)]
pub struct ReportPaths {
    pub text_path: PathBuf,
    pub html_path: PathBuf,
}

/// Write the optimize `.txt`/`.html` reports into `reports_dir`, creating it
/// (and any missing parents, e.g. the enclosing `<char>_Gear` folder) first.
pub fn write_optimize_report_files(
    reports_dir: &Path,
    text_report: &str,
    html_report: &str,
) -> Result<ReportPaths, String> {
    fs::create_dir_all(reports_dir).map_err(|e| {
        format!(
            "cannot create reports directory {}: {}",
            reports_dir.display(),
            e
        )
    })?;

    let stem = choose_report_stem(reports_dir)?;
    let text_path = reports_dir.join(format!("{}.txt", stem));
    let html_path = reports_dir.join(format!("{}.html", stem));

    fs::write(&text_path, text_report)
        .map_err(|e| format!("cannot write {}: {}", text_path.display(), e))?;
    fs::write(&html_path, html_report)
        .map_err(|e| format!("cannot write {}: {}", html_path.display(), e))?;

    Ok(ReportPaths {
        text_path,
        html_path,
    })
}

/// Write the scrap-gear `.txt` report into `reports_dir`, creating it (and any
/// missing parents) first.
pub fn write_scrap_gear_report_file(
    reports_dir: &Path,
    text_report: &str,
) -> Result<PathBuf, String> {
    fs::create_dir_all(reports_dir).map_err(|e| {
        format!(
            "cannot create reports directory {}: {}",
            reports_dir.display(),
            e
        )
    })?;

    let serial = highest_report_serial(reports_dir)?.unwrap_or(0);
    let text_path = choose_scrap_gear_report_path(reports_dir, serial)?;

    fs::write(&text_path, text_report)
        .map_err(|e| format!("cannot write {}: {}", text_path.display(), e))?;

    Ok(text_path)
}

fn choose_scrap_gear_report_path(reports_dir: &Path, serial: u16) -> Result<PathBuf, String> {
    let base_stem = format!("{SCRAP_GEAR_STEM_PREFIX}{serial:03}");
    let base_path = reports_dir.join(format!("{base_stem}.txt"));
    if !base_path.exists() {
        return Ok(base_path);
    }

    for suffix in 1..=9 {
        let suffixed_path = reports_dir.join(format!("{base_stem}-{suffix}.txt"));
        if !suffixed_path.exists() {
            return Ok(suffixed_path);
        }
    }

    Err(format!(
        "all filenames from {base_stem}.txt through {base_stem}-9.txt already exist"
    ))
}

fn choose_report_stem(reports_dir: &Path) -> Result<String, String> {
    let serial = next_serial(reports_dir)?;
    let base_stem = format!("{STEM_PREFIX}{serial:03}");
    if stem_is_available(reports_dir, &base_stem) {
        return Ok(base_stem);
    }

    for suffix in 1..=9 {
        let suffixed = format!("{base_stem}-{suffix}");
        if stem_is_available(reports_dir, &suffixed) {
            return Ok(suffixed);
        }
    }

    Err(format!(
        "all filenames from {base_stem}.txt/.html through {base_stem}-9.txt/.html already exist"
    ))
}

fn next_serial(reports_dir: &Path) -> Result<u16, String> {
    let highest = highest_report_serial(reports_dir)?;
    Ok(match highest {
        Some(999) => 0,
        Some(serial) => serial + 1,
        None => 0,
    })
}

fn highest_report_serial(reports_dir: &Path) -> Result<Option<u16>, String> {
    let mut highest: Option<u16> = None;
    let entries = fs::read_dir(reports_dir).map_err(|e| {
        format!(
            "cannot read reports directory {}: {}",
            reports_dir.display(),
            e
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "cannot read entry in reports directory {}: {}",
                reports_dir.display(),
                e
            )
        })?;
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        let Some(serial) = parse_report_serial(name) else {
            continue;
        };
        highest = Some(highest.map_or(serial, |current| current.max(serial)));
    }

    Ok(highest)
}

fn stem_is_available(reports_dir: &Path, stem: &str) -> bool {
    !reports_dir.join(format!("{}.txt", stem)).exists()
        && !reports_dir.join(format!("{}.html", stem)).exists()
}

fn parse_report_serial(file_name: &str) -> Option<u16> {
    if !(file_name.ends_with(".txt") || file_name.ends_with(".html")) {
        return None;
    }
    let stem = file_name
        .strip_suffix(".txt")
        .or_else(|| file_name.strip_suffix(".html"))?;
    let rest = stem.strip_prefix(STEM_PREFIX)?;
    let (serial, suffix) = match rest.split_once('-') {
        Some((serial, suffix)) => (serial, Some(suffix)),
        None => (rest, None),
    };
    if serial.len() != 3 || !serial.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if let Some(suffix) = suffix {
        if suffix.len() != 1 || !matches!(suffix.chars().next(), Some('1'..='9')) {
            return None;
        }
    }
    serial.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gear::{GearItem, GearSet, Slot};
    use crate::optimizer::OptimizeResult;
    use crate::report::{format_optimize_report, format_optimize_report_html};
    use crate::stat::{Stat, StatGoal, BASE_STATS, TRACKED_STATS};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn serial_increments_from_highest_existing_report() {
        let temp = TempDir::new();
        let reports_dir = temp.reports_dir();
        fs::create_dir_all(&reports_dir).expect("reports dir");
        fs::write(reports_dir.join("lgo_GearReport_000.txt"), "").expect("seed");
        fs::write(reports_dir.join("lgo_GearReport_000.html"), "").expect("seed");
        fs::write(reports_dir.join("lgo_GearReport_004-2.txt"), "").expect("seed");

        let paths =
            write_optimize_report_files(&reports_dir, "text", "html").expect("write succeeds");

        assert_eq!(
            paths
                .text_path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("utf-8 filename"),
            "lgo_GearReport_005.txt"
        );
        assert_eq!(
            paths
                .html_path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("utf-8 filename"),
            "lgo_GearReport_005.html"
        );
    }

    #[test]
    fn serial_wraps_from_999_to_000() {
        let temp = TempDir::new();
        let reports_dir = temp.reports_dir();
        fs::create_dir_all(&reports_dir).expect("reports dir");
        fs::write(reports_dir.join("lgo_GearReport_999.txt"), "").expect("seed");

        let paths =
            write_optimize_report_files(&reports_dir, "text", "html").expect("write succeeds");

        assert_eq!(
            paths
                .text_path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("utf-8 filename"),
            "lgo_GearReport_000.txt"
        );
    }

    #[test]
    fn collision_uses_single_digit_suffix_with_matching_stems() {
        let temp = TempDir::new();
        let reports_dir = temp.reports_dir();
        fs::create_dir_all(&reports_dir).expect("reports dir");
        fs::write(reports_dir.join("lgo_GearReport_999.txt"), "").expect("seed");
        fs::write(reports_dir.join("lgo_GearReport_000.txt"), "").expect("seed");
        fs::write(reports_dir.join("lgo_GearReport_000.html"), "").expect("seed");

        let paths =
            write_optimize_report_files(&reports_dir, "text", "html").expect("write succeeds");

        assert_eq!(
            paths
                .text_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("utf-8 stem"),
            "lgo_GearReport_000-1"
        );
        assert_eq!(
            paths
                .html_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("utf-8 stem"),
            "lgo_GearReport_000-1"
        );
    }

    #[test]
    fn relative_reports_dir_keeps_plain_relative_report_paths() {
        let temp = TempDir::new_in_current_dir();
        let reports_dir = temp.relative_reports_dir();

        let paths = write_optimize_report_files(&reports_dir, "text", "html")
            .expect("write succeeds");

        let text = paths.text_path.display().to_string();
        let html = paths.html_path.display().to_string();
        assert!(!text.starts_with(r"\\?\"));
        assert!(!html.starts_with(r"\\?\"));
        assert_eq!(
            paths
                .text_path
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str()),
            reports_dir
                .file_name()
                .and_then(|name| name.to_str())
        );
    }

    #[test]
    fn zero_stat_rows_are_written_to_text_report() {
        let temp = TempDir::new();
        let reports_dir = temp.reports_dir();
        let result = sample_optimize_result("Simple Helm");
        let text_report = format_optimize_report(
            &result,
            &[StatGoal {
                stat: Stat::Morale,
                minimum: 1000,
            }],
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &sample_base_stats(),
        );
        let html_report = format_optimize_report_html(
            &result,
            &[StatGoal {
                stat: Stat::Morale,
                minimum: 1000,
            }],
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &sample_base_stats(),
        );

        let paths = write_optimize_report_files(&reports_dir, &text_report, &html_report)
            .expect("write succeeds");
        let written = fs::read_to_string(paths.text_path).expect("read text report");

        for (stat, _) in TRACKED_STATS {
            assert!(
                written.contains(&stat.to_string()),
                "text report must list tracked stat {}",
                stat
            );
        }
        assert!(written.contains("Block"));
        assert!(written.contains("Projected raw Base stats"));
        for (stat, _) in BASE_STATS {
            assert!(
                written.contains(&stat.to_string()),
                "text report must list base stat {}",
                stat
            );
        }
    }

    #[test]
    fn html_report_writes_escaped_item_names() {
        let temp = TempDir::new();
        let reports_dir = temp.reports_dir();
        let result = sample_optimize_result("Shield <&> \"Quote\" 'Single'");
        let text_report = format_optimize_report(
            &result,
            &[StatGoal {
                stat: Stat::Morale,
                minimum: 1000,
            }],
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &sample_base_stats(),
        );
        let html_report = format_optimize_report_html(
            &result,
            &[StatGoal {
                stat: Stat::Morale,
                minimum: 1000,
            }],
            "Thalya",
            "Lore-master",
            "lgo_Thalya_gearReady.toml",
            "2026-08-31 08:00:00 +00:00",
            &sample_base_stats(),
        );

        let paths = write_optimize_report_files(&reports_dir, &text_report, &html_report)
            .expect("write succeeds");
        let written = fs::read_to_string(paths.html_path).expect("read html report");

        assert!(written.contains("Shield &lt;&amp;&gt; &quot;Quote&quot; &#39;Single&#39;"));
        assert!(written.contains("<meta charset=\"utf-8\">"));
    }

    #[test]
    fn fails_when_all_suffixes_for_wrapped_serial_are_taken() {
        let temp = TempDir::new();
        let reports_dir = temp.reports_dir();
        fs::create_dir_all(&reports_dir).expect("reports dir");
        fs::write(reports_dir.join("lgo_GearReport_999.txt"), "").expect("seed");
        fs::write(reports_dir.join("lgo_GearReport_000.txt"), "").expect("seed");
        fs::write(reports_dir.join("lgo_GearReport_000.html"), "").expect("seed");
        for suffix in 1..=9 {
            fs::write(
                reports_dir.join(format!("lgo_GearReport_000-{suffix}.txt")),
                "",
            )
            .expect("seed");
            fs::write(
                reports_dir.join(format!("lgo_GearReport_000-{suffix}.html")),
                "",
            )
            .expect("seed");
        }

        let err =
            write_optimize_report_files(&reports_dir, "text", "html").expect_err("must fail");
        assert!(err.contains("lgo_GearReport_000"));
        assert!(err.contains("-9"));
    }

    #[test]
    fn scrap_gear_report_file_uses_highest_existing_gear_report_serial() {
        let temp = TempDir::new();
        let reports_dir = temp.reports_dir();
        fs::create_dir_all(&reports_dir).expect("reports dir");
        fs::write(reports_dir.join("lgo_GearReport_008.txt"), "").expect("seed optimize txt");
        fs::write(reports_dir.join("lgo_GearReport_008.html"), "").expect("seed optimize html");
        fs::write(reports_dir.join("lgo_ScrapGearReport_099.txt"), "").expect("seed scrap txt");

        let path = write_scrap_gear_report_file(&reports_dir, "scrap report text")
            .expect("write succeeds");

        assert_eq!(
            path.file_name()
                .and_then(|name| name.to_str())
                .expect("utf-8 filename"),
            "lgo_ScrapGearReport_008.txt"
        );
        assert_eq!(
            fs::read_to_string(&path).expect("read scrap report"),
            "scrap report text"
        );
    }

    fn sample_optimize_result(name: &str) -> OptimizeResult {
        let mut innate_stats = HashMap::new();
        innate_stats.insert(Stat::Morale, 1000);
        let mut gear_set = GearSet::new(innate_stats);
        gear_set.items.insert(
            Slot::Head,
            GearItem {
                name: name.to_string(),
                slot: Slot::Head,
                two_handed: false,
                either_hand: false,
                stats: [(Stat::Morale, 250), (Stat::Armor, 300)]
                    .into_iter()
                    .collect(),
            },
        );

        OptimizeResult {
            gear_set,
            feasible: true,
            failed_minima: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn sample_base_stats() -> HashMap<Stat, i64> {
        [
            (Stat::Might, 5300),
            (Stat::Agility, 2650),
            (Stat::Vitality, 10200),
            (Stat::Will, 7950),
            (Stat::Fate, 4000),
        ]
        .into_iter()
        .collect()
    }

    struct TempDir {
        path: PathBuf,
        relative_dir: Option<PathBuf>,
    }

    impl TempDir {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "lgo-report-files-{}-{}-{}",
                std::process::id(),
                nanos,
                unique
            ));
            fs::create_dir_all(&path).expect("temp dir created");
            TempDir {
                path,
                relative_dir: None,
            }
        }

        fn new_in_current_dir() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir_name = format!("tmp-report-files-{}-{}", std::process::id(), unique);
            let path = std::env::current_dir()
                .expect("current dir")
                .join(&dir_name);
            fs::create_dir_all(&path).expect("temp dir created");
            TempDir {
                path,
                relative_dir: Some(PathBuf::from(&dir_name)),
            }
        }

        /// Absolute reports directory mirroring the install-tree layout.
        fn reports_dir(&self) -> PathBuf {
            self.path.join("Thalya_Gear").join("Thalya_Reports")
        }

        /// Relative reports directory (for the plain-relative-path test).
        fn relative_reports_dir(&self) -> PathBuf {
            self.relative_dir
                .clone()
                .expect("relative dir available for this temp dir")
                .join("Thalya_Reports")
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
