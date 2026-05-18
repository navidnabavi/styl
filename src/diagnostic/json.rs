use super::Diagnostic;

/// Render diagnostics as JSON array
pub fn render_json(diagnostics: &[Diagnostic]) -> String {
    serde_json::to_string_pretty(diagnostics).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_json() {
        let d = Diagnostic::warning("W001", "layers[0]", "duplicate id");
        let out = render_json(&[d]);
        assert!(out.contains("\"severity\""));
        assert!(out.contains("warning"));
    }
}
