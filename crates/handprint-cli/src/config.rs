//! `handprint.toml` — per-project defaults.
//!
//! Modeled on Vale's `.vale.ini`: a project declares which reference and which
//! packs it uses and which threshold profile it holds itself to, so that
//! `handprint critique` in a repository needs no flags and every contributor
//! gets the same answer.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// The file name searched for.
pub const CONFIG_FILE: &str = "handprint.toml";

/// A project configuration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Project-level defaults.
    #[serde(default)]
    pub project: Project,
    /// Declared packs.
    #[serde(default, rename = "pack")]
    pub packs: Vec<Pack>,
}

/// Project-level defaults.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// Path to the fitted reference, relative to the config file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<PathBuf>,
    /// Path to the "away" reference for contrast mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub away: Option<PathBuf>,
    /// Threshold profile: `lenient`, `balanced` or `strict`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Findings reported per iteration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_findings: Option<usize>,
    /// Minimum severity reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_severity: Option<String>,
}

/// A declared pack: a versioned lexicon or fitted reference.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    /// Pack name.
    pub name: String,
    /// Pinned version. Packs are dated artifacts; an unpinned lexicon will
    /// quietly change what your project considers normal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Path to the pack file, relative to the config file.
    pub path: PathBuf,
}

impl Config {
    /// Find and load the nearest `handprint.toml`, walking up from `start`.
    ///
    /// Returns the config and the directory it was found in, so relative paths
    /// can be resolved against the right place.
    pub fn discover(start: &Path) -> Result<Option<(Config, PathBuf)>> {
        let mut dir = if start.is_dir() {
            start.to_path_buf()
        } else {
            start.parent().unwrap_or(Path::new(".")).to_path_buf()
        };
        loop {
            let candidate = dir.join(CONFIG_FILE);
            if candidate.is_file() {
                return Ok(Some((Config::load(&candidate)?, dir)));
            }
            if !dir.pop() {
                return Ok(None);
            }
        }
    }

    /// Load a config from an explicit path.
    pub fn load(path: &Path) -> Result<Config> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    /// Resolve a configured path against the config's directory.
    pub fn resolve(root: &Path, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[project]
reference = "refs/hn-me.json"
profile = "strict"
max_findings = 5

[[pack]]
name = "ai-slop"
version = "2026.07"
path = "packs/ai-slop.json"
"#;

    #[test]
    fn parses_a_project_file() {
        let config: Config = toml::from_str(SAMPLE).unwrap();
        assert_eq!(
            config.project.reference.as_deref(),
            Some(Path::new("refs/hn-me.json"))
        );
        assert_eq!(config.project.profile.as_deref(), Some("strict"));
        assert_eq!(config.project.max_findings, Some(5));
        assert_eq!(config.packs.len(), 1);
        assert_eq!(config.packs[0].version.as_deref(), Some("2026.07"));
    }

    #[test]
    fn unknown_keys_are_rejected_rather_than_ignored() {
        // A silently ignored typo in a threshold setting is worse than an
        // error, because the run still produces plausible numbers.
        let bad = "[project]\nrefrence = \"x\"\n";
        assert!(toml::from_str::<Config>(bad).is_err());
    }

    #[test]
    fn an_empty_config_is_valid() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn discovery_walks_up_the_tree() {
        let root = std::env::temp_dir().join(format!("handprint-config-{}", std::process::id()));
        let nested = root.join("a").join("b");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.join(CONFIG_FILE), SAMPLE).unwrap();

        let (config, dir) = Config::discover(&nested).unwrap().unwrap();
        assert_eq!(config.project.profile.as_deref(), Some("strict"));
        assert_eq!(dir, root);
        assert_eq!(
            Config::resolve(&dir, Path::new("refs/hn-me.json")),
            root.join("refs/hn-me.json")
        );
        std::fs::remove_dir_all(&root).unwrap();
    }
}
