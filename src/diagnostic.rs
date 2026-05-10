use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub path: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Diagnostic {
    pub fn error(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { severity: Severity::Error, code, message: message.into(), path: path.into(), hint: None }
    }

    pub fn warning(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { severity: Severity::Warning, code, message: message.into(), path: path.into(), hint: None }
    }

    pub fn info(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { severity: Severity::Info, code, message: message.into(), path: path.into(), hint: None }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// Render diagnostics in human-readable format
pub fn render_human(diagnostics: &[Diagnostic], filename: &str) -> String {
    let mut out = String::new();
    for d in diagnostics {
        out.push_str(&format!(
            "{}[{}] {}: {}\n  --> {}\n",
            d.severity, d.code, d.path, d.message, filename
        ));
        if let Some(hint) = &d.hint {
            out.push_str(&format!("  hint: {}\n", hint));
        }
        out.push('\n');
    }
    out
}

/// Render diagnostics as JSON array
pub fn render_json(diagnostics: &[Diagnostic]) -> String {
    serde_json::to_string_pretty(diagnostics).unwrap_or_else(|_| "[]".to_string())
}

/// Render diagnostics as GitHub Actions annotations
pub fn render_github(diagnostics: &[Diagnostic], filename: &str) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let level = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "notice",
        };
        out.push_str(&format!(
            "::{} file={},title={}::{} — {}\n",
            level, filename, d.code, d.path, d.message
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder() {
        let d = Diagnostic::error("E001", "layers[0].source", "source not found")
            .with_hint("add the source to the sources object");
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.code, "E001");
        assert_eq!(d.path, "layers[0].source");
        assert!(d.hint.is_some());
    }

    #[test]
    fn test_render_human() {
        let d = Diagnostic::error("E001", "layers[0]", "test error");
        let out = render_human(&[d], "style.json");
        assert!(out.contains("error[E001]"));
        assert!(out.contains("style.json"));
    }

    #[test]
    fn test_render_json() {
        let d = Diagnostic::warning("W001", "layers[0]", "duplicate id");
        let out = render_json(&[d]);
        assert!(out.contains("\"severity\""));
        assert!(out.contains("warning"));
    }

    #[test]
    fn test_render_github() {
        let d = Diagnostic::error("E001", "layers[0].source", "missing source");
        let out = render_github(&[d], "style.json");
        assert!(out.starts_with("::error"));
        assert!(out.contains("file=style.json"));
    }
}
