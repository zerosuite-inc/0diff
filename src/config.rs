use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub watch: WatchConfig,
    pub filter: FilterConfig,
    pub git: GitConfig,
    pub history: HistoryConfig,
    pub agents: AgentConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct WatchConfig {
    #[serde(default = "WatchConfig::default_paths")]
    pub paths: Vec<String>,
    #[serde(default = "WatchConfig::default_ignore")]
    pub ignore: Vec<String>,
    #[serde(default = "WatchConfig::default_extensions")]
    pub extensions: Vec<String>,
    #[serde(default = "WatchConfig::default_debounce_ms")]
    pub debounce_ms: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct FilterConfig {
    #[serde(default = "FilterConfig::default_ignore_whitespace")]
    pub ignore_whitespace: bool,
    #[serde(default = "FilterConfig::default_min_lines_changed")]
    pub min_lines_changed: usize,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct GitConfig {
    #[serde(default = "GitConfig::default_enabled")]
    pub enabled: bool,
    #[serde(default = "GitConfig::default_track_author")]
    pub track_author: bool,
    #[serde(default = "GitConfig::default_track_branch")]
    pub track_branch: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct HistoryConfig {
    #[serde(default = "HistoryConfig::default_max_size_mb")]
    pub max_size_mb: u64,
    #[serde(default = "HistoryConfig::default_max_days")]
    pub max_days: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct AgentConfig {
    #[serde(default = "AgentConfig::default_detect_patterns")]
    pub detect_patterns: Vec<String>,
    #[serde(default = "AgentConfig::default_tag_non_human")]
    pub tag_non_human: bool,
}

// --- Default implementations ---

impl Default for Config {
    fn default() -> Self {
        Self {
            watch: WatchConfig::default(),
            filter: FilterConfig::default(),
            git: GitConfig::default(),
            history: HistoryConfig::default(),
            agents: AgentConfig::default(),
        }
    }
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            paths: Self::default_paths(),
            ignore: Self::default_ignore(),
            extensions: Self::default_extensions(),
            debounce_ms: Self::default_debounce_ms(),
        }
    }
}

impl WatchConfig {
    fn default_paths() -> Vec<String> {
        vec![
            "src/".to_string(),
            "app/".to_string(),
            "entities/".to_string(),
        ]
    }

    fn default_ignore() -> Vec<String> {
        vec![
            "target/".to_string(),
            "node_modules/".to_string(),
            ".git/".to_string(),
            "*.log".to_string(),
            ".flindb/".to_string(),
        ]
    }

    fn default_extensions() -> Vec<String> {
        vec![
            "rs".to_string(),
            "flin".to_string(),
            "ts".to_string(),
            "js".to_string(),
            "py".to_string(),
            "go".to_string(),
            "java".to_string(),
        ]
    }

    fn default_debounce_ms() -> u64 {
        500
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            ignore_whitespace: Self::default_ignore_whitespace(),
            min_lines_changed: Self::default_min_lines_changed(),
        }
    }
}

impl FilterConfig {
    fn default_ignore_whitespace() -> bool {
        true
    }

    fn default_min_lines_changed() -> usize {
        1
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            track_author: Self::default_track_author(),
            track_branch: Self::default_track_branch(),
        }
    }
}

impl GitConfig {
    fn default_enabled() -> bool {
        true
    }

    fn default_track_author() -> bool {
        true
    }

    fn default_track_branch() -> bool {
        true
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_size_mb: Self::default_max_size_mb(),
            max_days: Self::default_max_days(),
        }
    }
}

impl HistoryConfig {
    fn default_max_size_mb() -> u64 {
        10
    }

    fn default_max_days() -> u64 {
        30
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            detect_patterns: Self::default_detect_patterns(),
            tag_non_human: Self::default_tag_non_human(),
        }
    }
}

impl AgentConfig {
    fn default_detect_patterns() -> Vec<String> {
        vec![
            "Claude".to_string(),
            "Cursor".to_string(),
            "Copilot".to_string(),
            "Windsurf".to_string(),
            "Devin".to_string(),
        ]
    }

    fn default_tag_non_human() -> bool {
        true
    }
}

// --- Config methods ---

impl Config {
    /// Load config from a TOML file. Missing fields use defaults.
    pub fn load(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Return the full commented default config as a TOML string.
    pub fn template() -> String {
        r#"# 0diff Configuration
# Place this file as .0diff.toml in your project root.

[watch]
# Directories to watch for changes (relative to project root)
paths = ["src/", "app/", "entities/"]

# Patterns to ignore (supports glob syntax)
ignore = ["target/", "node_modules/", ".git/", "*.log", ".flindb/"]

# File extensions to track (without the dot)
extensions = ["rs", "flin", "ts", "js", "py", "go", "java"]

# Debounce interval in milliseconds — changes within this window are batched
debounce_ms = 500

[filter]
# Ignore whitespace-only changes in diffs
ignore_whitespace = true

# Minimum number of lines changed to record a modification
min_lines_changed = 1

[git]
# Enable git integration (branch, author, commit tracking)
enabled = true

# Track the author of each change via git blame
track_author = true

# Track the current branch name
track_branch = true

[history]
# Maximum history storage size in megabytes
max_size_mb = 10

# Maximum age of history entries in days
max_days = 30

[agents]
# Patterns to detect AI agent modifications (matched against commit messages and metadata)
detect_patterns = ["Claude", "Cursor", "Copilot", "Windsurf", "Devin"]

# Automatically tag changes identified as non-human
tag_non_human = true
"#
        .to_string()
    }

    /// Check whether a given file path should be watched based on config rules.
    ///
    /// A path is watched when ALL of these are true:
    /// 1. Its extension is in `watch.extensions`
    /// 2. It starts with at least one of the `watch.paths` prefixes
    /// 3. It does NOT match any `watch.ignore` glob pattern
    pub fn should_watch(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // 1. Check extension
        let ext_match = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.watch.extensions.iter().any(|e| e == ext))
            .unwrap_or(false);
        if !ext_match {
            return false;
        }

        // 2. Check watch paths — at least one prefix must match
        let prefix_match = self
            .watch
            .paths
            .iter()
            .any(|prefix| path_str.starts_with(prefix.as_str()));
        if !prefix_match {
            return false;
        }

        // 3. Check ignore patterns — none must match
        for pattern_str in &self.watch.ignore {
            // Try the pattern as-is against the full path
            if let Ok(pattern) = glob::Pattern::new(pattern_str) {
                if pattern.matches(&path_str) {
                    return false;
                }
            }
            // For directory patterns like "target/", also match without trailing slash
            // against each path component
            let clean = pattern_str.trim_end_matches('/');
            if let Ok(pattern) = glob::Pattern::new(clean) {
                for component in path.components() {
                    let comp = component.as_os_str().to_string_lossy();
                    if pattern.matches(&comp) {
                        return false;
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_config_values() {
        let config = Config::default();

        // Watch defaults
        assert_eq!(config.watch.paths, vec!["src/", "app/", "entities/"]);
        assert_eq!(
            config.watch.ignore,
            vec!["target/", "node_modules/", ".git/", "*.log", ".flindb/"]
        );
        assert_eq!(
            config.watch.extensions,
            vec!["rs", "flin", "ts", "js", "py", "go", "java"]
        );
        assert_eq!(config.watch.debounce_ms, 500);

        // Filter defaults
        assert!(config.filter.ignore_whitespace);
        assert_eq!(config.filter.min_lines_changed, 1);

        // Git defaults
        assert!(config.git.enabled);
        assert!(config.git.track_author);
        assert!(config.git.track_branch);

        // History defaults
        assert_eq!(config.history.max_size_mb, 10);
        assert_eq!(config.history.max_days, 30);

        // Agent defaults
        assert_eq!(
            config.agents.detect_patterns,
            vec!["Claude", "Cursor", "Copilot", "Windsurf", "Devin"]
        );
        assert!(config.agents.tag_non_human);
    }

    #[test]
    fn test_should_watch_matching_path() {
        let config = Config::default();

        // Matches: extension=rs, prefix=src/, no ignore hit
        assert!(config.should_watch(&PathBuf::from("src/main.rs")));
        assert!(config.should_watch(&PathBuf::from("src/config.rs")));
        assert!(config.should_watch(&PathBuf::from("app/index.flin")));
        assert!(config.should_watch(&PathBuf::from("entities/user.flin")));
        assert!(config.should_watch(&PathBuf::from("src/lib/utils.ts")));
    }

    #[test]
    fn test_should_watch_wrong_extension() {
        let config = Config::default();

        // .txt is not in the extensions list
        assert!(!config.should_watch(&PathBuf::from("src/readme.txt")));
        assert!(!config.should_watch(&PathBuf::from("src/image.png")));
    }

    #[test]
    fn test_should_watch_wrong_prefix() {
        let config = Config::default();

        // "docs/" is not a watched path
        assert!(!config.should_watch(&PathBuf::from("docs/guide.rs")));
        assert!(!config.should_watch(&PathBuf::from("tests/main.rs")));
    }

    #[test]
    fn test_should_watch_ignored_patterns() {
        let config = Config::default();

        // target/ directory should be ignored
        assert!(!config.should_watch(&PathBuf::from("src/target/debug.rs")));
        // node_modules/ should be ignored
        assert!(!config.should_watch(&PathBuf::from("app/node_modules/pkg.js")));
        // .git/ should be ignored
        assert!(!config.should_watch(&PathBuf::from("src/.git/config.rs")));
        // *.log glob should be ignored
        assert!(!config.should_watch(&PathBuf::from("src/server.log")));
    }

    #[test]
    fn test_toml_parsing_roundtrip() {
        let template = Config::template();
        let parsed: Config =
            toml::from_str(&template).expect("template should parse as valid TOML");

        // Verify parsed values match defaults
        assert_eq!(parsed.watch.paths, Config::default().watch.paths);
        assert_eq!(parsed.watch.debounce_ms, 500);
        assert!(parsed.filter.ignore_whitespace);
        assert!(parsed.git.enabled);
        assert_eq!(parsed.history.max_size_mb, 10);
        assert_eq!(parsed.history.max_days, 30);
        assert_eq!(parsed.agents.detect_patterns.len(), 5);
        assert!(parsed.agents.tag_non_human);
    }

    #[test]
    fn test_toml_partial_config() {
        // Only specify one section — rest should use defaults
        let partial = r#"
[watch]
debounce_ms = 1000
"#;
        let config: Config = toml::from_str(partial).expect("partial config should parse");
        assert_eq!(config.watch.debounce_ms, 1000);
        // Other watch fields keep defaults
        assert_eq!(config.watch.paths, WatchConfig::default_paths());
        // Other sections keep defaults
        assert!(config.git.enabled);
        assert_eq!(config.history.max_days, 30);
    }

    #[test]
    fn test_toml_empty_config() {
        // Empty string should produce full defaults
        let config: Config = toml::from_str("").expect("empty config should parse");
        assert_eq!(config.watch.debounce_ms, 500);
        assert!(config.filter.ignore_whitespace);
    }
}
