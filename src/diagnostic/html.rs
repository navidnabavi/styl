use super::Diagnostic;

/// Render diagnostics as a self-contained HTML document
pub fn render_html(_diagnostics: &[Diagnostic], filename: &str) -> String {
    // TODO: Implement in Task 3
    format!("<html><body>HTML renderer for {}</body></html>", filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_html_empty() {
        let diags: Vec<Diagnostic> = vec![];
        let html = render_html(&diags, "style.json");
        assert!(html.contains("<html"));
    }
}
