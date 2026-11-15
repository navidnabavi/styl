use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub path: String,
    pub hint: Option<String>,
    /// Source location of `path`, when it could be resolved against the original
    /// text. Rules never set this; it is filled in afterwards by
    /// [`crate::span::resolve_ranges`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<TextRange>,
}

/// A zero-based line and UTF-16 column, matching LSP's default position encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// A half-open range of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TextRange {
    pub start: Position,
    pub end: Position,
}

impl Position {
    /// One-based line and column, for display to humans.
    pub fn one_based(&self) -> (u32, u32) {
        (self.line + 1, self.character + 1)
    }
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
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
            path: path.into(),
            hint: None,
            range: None,
        }
    }

    pub fn warning(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            code,
            message: message.into(),
            path: path.into(),
            hint: None,
            range: None,
        }
    }

    pub fn info(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            code,
            message: message.into(),
            path: path.into(),
            hint: None,
            range: None,
        }
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

// Re-export renderers
pub use github::render_github;
pub use html::render_html;
pub use human::render_human;
pub use json::render_json;

// Module declarations
pub mod github;
pub mod html;
pub mod human;
pub mod json;

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
}
