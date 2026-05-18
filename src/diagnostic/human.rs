use super::Diagnostic;

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
}
