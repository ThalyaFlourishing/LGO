//! Saved optimize-build profiles (`lgo_<character>_builds.toml`).

use crate::stat::StatGoal;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const BUILDS_TABLE_KEY: &str = "builds";
const GOALS_KEY: &str = "goals";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedBuild {
    pub name: String,
    pub goals: Vec<StatGoal>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SavedBuilds {
    builds: Vec<SavedBuild>,
}

impl SavedBuilds {
    pub fn read_file(path: &Path) -> Result<Self, String> {
        let src = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read saved builds file {}: {}", path.display(), e))?;
        Self::from_toml_str(&src, path)
    }

    pub fn from_toml_str(src: &str, path: &Path) -> Result<Self, String> {
        let doc: toml::Value = src
            .parse()
            .map_err(|e| format!("Malformed TOML in {}: {}", path.display(), e))?;
        let root = doc.as_table().ok_or_else(|| {
            format!(
                "Saved builds file {} must contain a TOML table",
                path.display()
            )
        })?;

        for key in root.keys() {
            if key != BUILDS_TABLE_KEY {
                return Err(format!(
                    "Unknown top-level key `{}` in {}. Expected only `[builds.<name>]` entries.",
                    key,
                    path.display()
                ));
            }
        }

        let Some(builds_value) = root.get(BUILDS_TABLE_KEY) else {
            return Ok(Self::default());
        };
        let builds_table = builds_value.as_table().ok_or_else(|| {
            format!(
                "`{}` in {} must be a TOML table of `[builds.<name>]` entries",
                BUILDS_TABLE_KEY,
                path.display()
            )
        })?;

        let mut seen_lowercase_names = std::collections::HashMap::<String, String>::new();
        let mut builds = Vec::with_capacity(builds_table.len());

        for (build_name, build_value) in builds_table {
            if build_name.trim().is_empty() {
                return Err(format!(
                    "Saved build names in {} must not be empty",
                    path.display()
                ));
            }

            let lowercase_name = build_name.to_ascii_lowercase();
            if let Some(previous_name) =
                seen_lowercase_names.insert(lowercase_name, build_name.clone())
            {
                return Err(format!(
                    "Duplicate build names `{}` and `{}` in {} (build names are case-insensitive).",
                    previous_name,
                    build_name,
                    path.display()
                ));
            }

            let build_table = build_value.as_table().ok_or_else(|| {
                format!(
                    "Build `{}` in {} must be a TOML table",
                    build_name,
                    path.display()
                )
            })?;

            for key in build_table.keys() {
                if key != GOALS_KEY {
                    return Err(format!("Unknown key `{}` in build `{}`.", key, build_name));
                }
            }

            let goals_value = build_table
                .get(GOALS_KEY)
                .ok_or_else(|| format!("Build `{}` is missing `{}`.", build_name, GOALS_KEY))?;
            let goals_array = goals_value.as_array().ok_or_else(|| {
                format!(
                    "Build `{}` field `{}` must be an array.",
                    build_name, GOALS_KEY
                )
            })?;
            if goals_array.is_empty() {
                return Err(format!(
                    "Build `{}` has an empty `{}` array.",
                    build_name, GOALS_KEY
                ));
            }

            let mut goals = Vec::with_capacity(goals_array.len());
            for goal_value in goals_array {
                let Some(goal_str) = goal_value.as_str() else {
                    return Err(format!(
                        "Build `{}` has a non-string goal entry in `{}`.",
                        build_name, GOALS_KEY
                    ));
                };
                let goal = goal_str.parse::<StatGoal>().map_err(|e| {
                    format!(
                        "Invalid goal `{}` in build `{}`: {}",
                        goal_str, build_name, e
                    )
                })?;
                goals.push(goal);
            }

            builds.push(SavedBuild {
                name: build_name.clone(),
                goals,
            });
        }

        Ok(Self { builds })
    }

    pub fn is_empty(&self) -> bool {
        self.builds.is_empty()
    }

    pub fn builds(&self) -> &[SavedBuild] {
        &self.builds
    }

    pub fn find(&self, name: &str) -> Option<&SavedBuild> {
        self.builds
            .iter()
            .find(|build| build.name.eq_ignore_ascii_case(name))
    }

    pub fn upsert(&mut self, name: String, goals: Vec<StatGoal>) {
        if let Some(existing) = self
            .builds
            .iter_mut()
            .find(|build| build.name.eq_ignore_ascii_case(&name))
        {
            existing.name = name;
            existing.goals = goals;
            return;
        }

        self.builds.push(SavedBuild { name, goals });
    }

    pub fn write_file(&self, path: &Path) -> Result<(), String> {
        fs::write(path, self.to_toml_string())
            .map_err(|e| format!("Cannot write saved builds file {}: {}", path.display(), e))
    }

    pub fn to_toml_string(&self) -> String {
        let mut out = String::from(
            "# LGO saved build profiles. Hand-editing is fine.\n# Goals are priority-ordered: first entry = highest priority.\n",
        );

        if self.builds.is_empty() {
            return out;
        }

        out.push('\n');
        for (idx, build) in self.builds.iter().enumerate() {
            let header_key = format_build_header_key(&build.name);
            let goal_list = build
                .goals
                .iter()
                .map(|goal| toml::Value::String(goal.to_string()).to_string())
                .collect::<Vec<_>>()
                .join(", ");

            writeln!(&mut out, "[builds.{}]", header_key).expect("writing to String cannot fail");
            writeln!(&mut out, "goals = [{}]", goal_list).expect("writing to String cannot fail");
            if idx + 1 != self.builds.len() {
                writeln!(&mut out).expect("writing to String cannot fail");
            }
        }

        out
    }
}

fn format_build_header_key(name: &str) -> String {
    if name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        name.to_string()
    } else {
        toml::Value::String(name.to_string()).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stat::Stat;
    use std::path::PathBuf;

    fn fake_path() -> PathBuf {
        PathBuf::from("/tmp/lgo_test_builds.toml")
    }

    #[test]
    fn parses_saved_builds() {
        let builds = SavedBuilds::from_toml_str(
            r#"
[builds.healer]
goals = ["oh:200000", "cr:350000", "ml:0"]

[builds.tank]
goals = ["tt:450000", "ml:300000", "pt:0"]
"#,
            &fake_path(),
        )
        .expect("must parse");

        assert_eq!(builds.builds().len(), 2);
        assert_eq!(builds.builds()[0].name, "healer");
        assert_eq!(builds.builds()[0].goals[0].stat, Stat::OutgoingHealing);
        assert_eq!(builds.builds()[1].goals[2].stat, Stat::PhysicalMitigation);
    }

    #[test]
    fn rejects_duplicate_case_insensitive_build_names() {
        let err = SavedBuilds::from_toml_str(
            r#"
[builds.Healer]
goals = ["oh:1"]

[builds.healer]
goals = ["oh:2"]
"#,
            &fake_path(),
        )
        .expect_err("must reject duplicates");

        assert!(err.contains("Duplicate build names `Healer` and `healer`"));
    }

    #[test]
    fn rejects_invalid_goal_with_build_name_in_message() {
        let err = SavedBuilds::from_toml_str(
            r#"
[builds.healer]
goals = ["bogus:1"]
"#,
            &fake_path(),
        )
        .expect_err("must reject invalid goal");

        assert!(err.contains("Invalid goal `bogus:1` in build `healer`"));
    }

    #[test]
    fn rejects_empty_goals_array() {
        let err = SavedBuilds::from_toml_str(
            r#"
[builds.healer]
goals = []
"#,
            &fake_path(),
        )
        .expect_err("must reject empty goals");

        assert!(err.contains("Build `healer` has an empty `goals` array."));
    }

    #[test]
    fn upsert_overwrites_case_insensitively_and_preserves_new_display_name() {
        let mut builds = SavedBuilds::from_toml_str(
            r#"
[builds.healer]
goals = ["oh:1"]
"#,
            &fake_path(),
        )
        .expect("must parse");

        builds.upsert(
            "Healer".to_string(),
            vec![StatGoal {
                stat: Stat::CriticalRating,
                minimum: 2,
            }],
        );

        assert_eq!(builds.builds().len(), 1);
        assert_eq!(builds.builds()[0].name, "Healer");
        assert_eq!(builds.builds()[0].goals[0].stat, Stat::CriticalRating);
        assert_eq!(builds.builds()[0].goals[0].minimum, 2);
    }

    #[test]
    fn writes_canonical_goal_strings() {
        let builds = SavedBuilds {
            builds: vec![SavedBuild {
                name: "Healer Build".to_string(),
                goals: vec![
                    StatGoal {
                        stat: Stat::OutgoingHealing,
                        minimum: 200000,
                    },
                    StatGoal {
                        stat: Stat::Morale,
                        minimum: 0,
                    },
                ],
            }],
        };

        let written = builds.to_toml_string();
        assert!(written.contains(r#"[builds."Healer Build"]"#));
        assert!(written.contains(r#"goals = ["oh:200000", "ml:0"]"#));
    }

    #[test]
    fn writes_bare_build_name_when_toml_allows_it() {
        let builds = SavedBuilds {
            builds: vec![SavedBuild {
                name: "healer".to_string(),
                goals: vec![StatGoal {
                    stat: Stat::OutgoingHealing,
                    minimum: 200000,
                }],
            }],
        };

        let written = builds.to_toml_string();
        assert!(written.contains("[builds.healer]"));
    }
}
