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

/// Parsed .stylrc configuration
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

impl Config {
    /// Apply severity overrides to diagnostics. Rules mapped to "off" are removed,
    /// "error" overrides upgrade warnings/info to errors, "warn" downgrades errors to warnings.
    pub fn apply_severity(&self, diags: &mut Vec<crate::diagnostic::Diagnostic>) {
        if self.rules.is_empty() {
            return;
        }
        diags.retain(|d| {
            self.rules
                .get(d.code)
                .is_none_or(|s| *s != RuleSeverity::Off)
        });
        for d in diags.iter_mut() {
            if let Some(severity) = self.rules.get(d.code) {
                match severity {
                    RuleSeverity::Error => d.severity = crate::diagnostic::Severity::Error,
                    RuleSeverity::Warn => d.severity = crate::diagnostic::Severity::Warning,
                    RuleSeverity::Off => {} // already filtered out above
                }
            }
        }
    }
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

/// Walk up the directory tree from `start` to find `.stylrc`.
/// Returns the path if found, None otherwise.
pub fn discover_config(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(".stylrc");
        if candidate.exists() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Load and parse a `.stylrc` TOML file.
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
        // Use a temp dir that won't have .stylrc
        let tmp = std::env::temp_dir().join("no-config-here-xyz");
        std::fs::create_dir_all(&tmp).ok();
        // Should not find one (no .stylrc in /tmp or above on most systems)
        // Just check it doesn't panic
        let _ = discover_config(&tmp);
    }

    #[test]
    fn test_discover_config_finds_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join(".stylrc");
        std::fs::write(&config_path, r#"spec = "maplibre""#).unwrap();
        let sub = tmp.path().join("subdir");
        std::fs::create_dir_all(&sub).unwrap();
        let found = discover_config(&sub);
        assert_eq!(found, Some(config_path));
    }

    use crate::diagnostic::{Diagnostic, Severity};

    #[test]
    fn test_apply_severity_upgrades_to_error() {
        let config = Config {
            rules: {
                let mut m = HashMap::new();
                m.insert("W001".to_string(), RuleSeverity::Error);
                m
            },
            ..Config::default()
        };
        let mut diags = vec![Diagnostic::warning("W001", "p", "duplicate")];
        config.apply_severity(&mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Error);
    }

    #[test]
    fn test_apply_severity_off_removes() {
        let config = Config {
            rules: {
                let mut m = HashMap::new();
                m.insert("W001".to_string(), RuleSeverity::Off);
                m
            },
            ..Config::default()
        };
        let mut diags = vec![Diagnostic::warning("W001", "p", "duplicate")];
        config.apply_severity(&mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_apply_severity_warn_downgrades() {
        let config = Config {
            rules: {
                let mut m = HashMap::new();
                m.insert("E001".to_string(), RuleSeverity::Warn);
                m
            },
            ..Config::default()
        };
        let mut diags = vec![Diagnostic::error("E001", "p", "wrong version")];
        config.apply_severity(&mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Warning);
    }

    #[test]
    fn test_apply_severity_empty_rules_noop() {
        let config = Config::default();
        let mut diags = vec![
            Diagnostic::warning("W001", "p", "dup"),
            Diagnostic::error("E001", "p", "version"),
        ];
        let original = diags.clone();
        config.apply_severity(&mut diags);
        assert_eq!(diags.len(), original.len());
    }
}
