use super::{Diagnostic, Severity};

/// Render diagnostics as GitHub Actions annotations
pub fn render_github(diagnostics: &[Diagnostic], filename: &str) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let level = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "notice",
        };
        // `col`/`endColumn` are only meaningful to GitHub on a single-line
        // annotation, so they are emitted only when the range does not wrap.
        let position = match &d.range {
            Some(r) if r.start.line == r.end.line => format!(
                ",line={},endLine={},col={},endColumn={}",
                r.start.line + 1,
                r.end.line + 1,
                r.start.character + 1,
                r.end.character + 1
            ),
            Some(r) => format!(",line={},endLine={}", r.start.line + 1, r.end.line + 1),
            None => String::new(),
        };
        out.push_str(&format!(
            "::{} file={}{},title={}::{} — {}\n",
            level, filename, position, d.code, d.path, d.message
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_github() {
        let d = Diagnostic::error("E001", "layers[0].source", "missing source");
        let out = render_github(&[d], "style.json");
        assert!(out.starts_with("::error"));
        assert!(out.contains("file=style.json"));
    }

    #[test]
    fn test_render_github_with_position() {
        use crate::diagnostic::{Position, TextRange};
        let mut d = Diagnostic::error("E001", "layers[0]", "missing source");
        d.range = Some(TextRange {
            start: Position {
                line: 4,
                character: 2,
            },
            end: Position {
                line: 4,
                character: 8,
            },
        });
        let out = render_github(&[d], "style.json");
        assert!(
            out.contains("line=5,endLine=5,col=3,endColumn=9"),
            "got: {}",
            out
        );
    }
}
