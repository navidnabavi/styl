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
    fn test_render_github() {
        let d = Diagnostic::error("E001", "layers[0].source", "missing source");
        let out = render_github(&[d], "style.json");
        assert!(out.starts_with("::error"));
        assert!(out.contains("file=style.json"));
    }
}
