use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Rule severity override
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    Error,
    Warn,
    Off,
}

/// Parsed .mapboxlintrc configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    /// Spec variant: "maplibre" or "mapbox"
    #[serde(default)]
    pub spec: Option<String>,

    /// Per-rule severity overrides
    #[serde(default)]
    pub rules: HashMap<String, RuleSeverity>,

    /// Formatting options
    #[serde(default)]
    pub format: FormatConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormatConfig {
    #[serde(default = "default_indent")]
    pub indent: usize,
}

fn default_indent() -> usize {
    2
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self { indent: 2 }
    }
}

/// Walk up the directory tree from `start` to find `.mapboxlintrc`.
/// Returns the path if found, None otherwise.
pub fn discover_config(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(".mapboxlintrc");
        if candidate.exists() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Load and parse a `.mapboxlintrc` TOML file.
pub fn load_config(path: &Path) -> Result<Config, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read config {}: {}", path.display(), e))?;
    toml::from_str(&content).map_err(|e| format!("invalid config {}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"spec = "maplibre""#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.spec, Some("maplibre".to_string()));
        assert!(config.rules.is_empty());
        assert_eq!(config.format.indent, 2);
    }

    #[test]
    fn test_parse_rule_overrides() {
        let toml = r#"
spec = "mapbox"

[rules]
W001 = "error"
W009 = "off"
W012 = "warn"

[format]
indent = 4
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.rules.get("W001"), Some(&RuleSeverity::Error));
        assert_eq!(config.rules.get("W009"), Some(&RuleSeverity::Off));
        assert_eq!(config.rules.get("W012"), Some(&RuleSeverity::Warn));
        assert_eq!(config.format.indent, 4);
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.format.indent, 2);
        assert!(config.rules.is_empty());
        assert!(config.spec.is_none());
    }

    #[test]
    fn test_discover_config_not_found() {
        // Use a temp dir that won't have .mapboxlintrc
        let tmp = std::env::temp_dir().join("no-config-here-xyz");
        std::fs::create_dir_all(&tmp).ok();
        // Should not find one (no .mapboxlintrc in /tmp or above on most systems)
        // Just check it doesn't panic
        let _ = discover_config(&tmp);
    }

    #[test]
    fn test_discover_config_finds_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join(".mapboxlintrc");
        std::fs::write(&config_path, r#"spec = "maplibre""#).unwrap();
        let sub = tmp.path().join("subdir");
        std::fs::create_dir_all(&sub).unwrap();
        let found = discover_config(&sub);
        assert_eq!(found, Some(config_path));
    }
}
