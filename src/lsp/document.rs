//! Per-buffer state, and the editor-provided settings that govern it.

use lsp_types::Uri;

use super::analysis;
use crate::cli::Spec;
use crate::linter::config::Config;

/// Editor-provided settings, which outrank `.stylrc`.
pub(super) struct Settings {
    pub(super) enabled: bool,
    pub(super) spec: Option<Spec>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            spec: None,
        }
    }
}

impl Settings {
    /// Read `enable` and `spec`, accepting them either bare or nested under a
    /// `styl` key, which is how VS Code delivers a configuration section.
    pub(super) fn apply(&mut self, value: Option<&serde_json::Value>) {
        let Some(value) = value else { return };
        let scope = value.get("styl").unwrap_or(value);

        if let Some(enabled) = scope.get("enable").and_then(serde_json::Value::as_bool) {
            self.enabled = enabled;
        }
        if let Some(name) = scope.get("spec").and_then(serde_json::Value::as_str) {
            self.spec = Config::parse_spec(name);
        }
    }
}

pub(super) struct Document {
    pub(super) uri: Uri,
    pub(super) text: String,
    /// Whether this buffer is a style. Sticky once true, so a recognized style
    /// keeps its diagnostics while it is mid-edit and momentarily unparseable.
    pub(super) is_style: bool,
}

impl Document {
    pub(super) fn new(uri: Uri, text: String) -> Self {
        Self {
            uri,
            is_style: analysis::looks_like_style(&text),
            text,
        }
    }

    pub(super) fn update(&mut self, text: String) {
        self.is_style |= analysis::looks_like_style(&text);
        self.text = text;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_accept_bare_or_nested_shapes() {
        let mut settings = Settings::default();
        settings.apply(Some(&serde_json::json!({ "enable": false })));
        assert!(!settings.enabled);

        let mut settings = Settings::default();
        settings.apply(Some(&serde_json::json!({ "styl": { "spec": "mapbox" } })));
        assert_eq!(settings.spec, Some(Spec::Mapbox));
    }

    #[test]
    fn settings_ignore_absent_and_unrecognized_values() {
        let mut settings = Settings::default();
        settings.apply(None);
        settings.apply(Some(&serde_json::json!({ "spec": "nonsense" })));
        assert!(settings.enabled);
        assert_eq!(settings.spec, None);
    }

    #[test]
    fn style_recognition_is_sticky_across_edits() {
        let uri: Uri = "file:///s.json".parse().unwrap();
        let mut document = Document::new(uri, r#"{"version": 8, "layers": []}"#.to_string());
        assert!(document.is_style);

        // An edit that leaves the buffer unparseable must not deselect it.
        document.update("{".to_string());
        assert!(document.is_style);
    }

    #[test]
    fn unrelated_json_is_never_claimed() {
        let uri: Uri = "file:///package.json".parse().unwrap();
        let document = Document::new(uri, r#"{"name": "app"}"#.to_string());
        assert!(!document.is_style);
    }
}
