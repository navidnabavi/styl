use super::Diagnostic;

/// Render diagnostics in human-readable format
pub fn render_human(diagnostics: &[Diagnostic], filename: &str) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let location = match &d.range {
            Some(r) => {
                let (line, col) = r.start.one_based();
                format!("{}:{}:{}", filename, line, col)
            }
            None => filename.to_string(),
        };
        out.push_str(&format!(
            "{}[{}] {}: {}\n  --> {}\n",
            d.severity, d.code, d.path, d.message, location
        ));
        if let Some(hint) = &d.hint {
            out.push_str(&format!("  hint: {}\n", hint));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_human() {
        let d = Diagnostic::error("E001", "layers[0]", "test error");
        let out = render_human(&[d], "style.json");
        assert!(out.contains("error[E001]"));
        assert!(out.contains("style.json"));
    }

    #[test]
    fn test_render_human_with_position() {
        use crate::diagnostic::{Position, TextRange};
        let mut d = Diagnostic::error("E001", "layers[0]", "test error");
        d.range = Some(TextRange {
            start: Position {
                line: 11,
                character: 4,
            },
            end: Position {
                line: 11,
                character: 9,
            },
        });
        let out = render_human(&[d], "style.json");
        assert!(out.contains("style.json:12:5"), "got: {}", out);
    }
}
