use super::Diagnostic;
use std::collections::BTreeMap;

/// Render diagnostics as a self-contained HTML document
pub fn render_html(diagnostics: &[Diagnostic], filename: &str) -> String {
    // Count by severity
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;

    for d in diagnostics {
        match d.severity {
            super::Severity::Error => error_count += 1,
            super::Severity::Warning => warning_count += 1,
            super::Severity::Info => info_count += 1,
        }
    }

    // Group by severity, then by code
    let mut by_severity: BTreeMap<String, BTreeMap<String, Vec<&Diagnostic>>> = BTreeMap::new();
    for d in diagnostics {
        let severity_key = match d.severity {
            super::Severity::Error => "error".to_string(),
            super::Severity::Warning => "warning".to_string(),
            super::Severity::Info => "info".to_string(),
        };
        by_severity
            .entry(severity_key)
            .or_default()
            .entry(d.code.to_string())
            .or_default()
            .push(d);
    }

    // Render HTML
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n");
    html.push_str("<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("  <title>styl Report</title>\n");
    html.push_str(&render_html_styles());
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    html.push_str("<div class=\"styl-report\">\n");

    // Header
    html.push_str("  <header>\n");
    html.push_str(&format!(
        "    <h1>{}</h1>\n",
        html_escape::encode_text(filename)
    ));
    let now = chrono::Utc::now();
    html.push_str(&format!(
        "    <p>{} error{}, {} warning{}, {} info | {}</p>\n",
        error_count,
        if error_count == 1 { "" } else { "s" },
        warning_count,
        if warning_count == 1 { "" } else { "s" },
        info_count,
        now.format("%Y-%m-%d %H:%M UTC")
    ));
    html.push_str("  </header>\n");

    // Severity groups
    for (severity, codes) in &by_severity {
        let (emoji, label, count, class_name, summary_class) = match severity.as_str() {
            "error" => ("🔴", "Errors", error_count, "error-group", "error-summary"),
            "warning" => (
                "🟡",
                "Warnings",
                warning_count,
                "warning-group",
                "warning-summary",
            ),
            "info" => ("🔵", "Info", info_count, "info-group", "info-summary"),
            _ => ("❓", "Unknown", 0, "unknown-group", "unknown-summary"),
        };

        html.push_str(&format!(
            "  <details open class=\"{}\">\n    <summary class=\"{}\">{} {} ({})</summary>\n",
            class_name, summary_class, emoji, label, count
        ));
        html.push_str("    <div class=\"severity-group\">\n");

        // Code groups within severity
        for (code, items) in codes {
            html.push_str(&format!(
                "      <details>\n        <summary class=\"code-summary\">{} ({} occurrence{})</summary>\n",
                html_escape::encode_text(code),
                items.len(),
                if items.len() == 1 { "" } else { "s" }
            ));
            html.push_str("        <div class=\"code-group\">\n");

            for item in items {
                html.push_str("          <div class=\"diagnostic\">\n");
                html.push_str(&format!(
                    "            <p><code class=\"path\">{}</code> {}</p>\n",
                    html_escape::encode_text(&item.path),
                    html_escape::encode_text(&item.message)
                ));
                if let Some(hint) = &item.hint {
                    html.push_str("            <details>\n");
                    html.push_str("              <summary>Hint</summary>\n");
                    html.push_str(&format!(
                        "              <p>{}</p>\n",
                        html_escape::encode_text(hint)
                    ));
                    html.push_str("            </details>\n");
                }
                html.push_str("          </div>\n");
            }

            html.push_str("        </div>\n");
            html.push_str("      </details>\n");
        }

        html.push_str("    </div>\n");
        html.push_str("  </details>\n");
    }

    html.push_str("</div>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");

    html
}

/// Render inline CSS styles for HTML output
fn render_html_styles() -> String {
    r#"  <style>
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }

    body {
      background-color: #1e1e1e;
      color: #e0e0e0;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 14px;
      line-height: 1.6;
      padding: 20px;
    }

    .styl-report {
      max-width: 900px;
      margin: 0 auto;
    }

    header {
      margin-bottom: 30px;
      border-bottom: 2px solid #333333;
      padding-bottom: 15px;
    }

    header h1 {
      font-size: 24px;
      margin-bottom: 8px;
      color: #ffffff;
    }

    header p {
      font-size: 13px;
      color: #999999;
    }

    details {
      margin: 12px 0;
      border-left: 3px solid #333333;
      padding-left: 15px;
    }

    details.error-group {
      border-left-color: #ff6b6b;
    }

    details.warning-group {
      border-left-color: #ffd93d;
    }

    details.info-group {
      border-left-color: #6bcaff;
    }

    summary {
      font-weight: bold;
      cursor: pointer;
      user-select: none;
      padding: 6px 0;
    }

    summary:hover {
      color: #ffffff;
    }

    summary.error-summary {
      color: #ff6b6b;
    }

    summary.warning-summary {
      color: #ffd93d;
    }

    summary.info-summary {
      color: #6bcaff;
    }

    .severity-group {
      margin-left: 20px;
    }

    .code-summary {
      color: #b0b0b0;
      font-size: 13px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    .code-group {
      margin-left: 20px;
    }

    .diagnostic {
      margin: 10px 0;
      padding: 10px;
      background-color: #252525;
      border-radius: 4px;
    }

    .diagnostic p {
      margin: 0 0 6px 0;
    }

    .diagnostic code.path {
      background-color: #1a1a1a;
      padding: 2px 4px;
      border-radius: 2px;
      font-family: 'Courier New', monospace;
      color: #6bcaff;
    }

    .diagnostic code {
      font-family: 'Courier New', monospace;
      padding: 2px 4px;
      border-radius: 2px;
    }
  </style>
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_html_empty() {
        let diags: Vec<Diagnostic> = vec![];
        let html = render_html(&diags, "style.json");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("style.json"));
        assert!(html.contains("0 errors"));
        assert!(html.contains("0 warnings"));
        assert!(html.contains("0 info"));
    }

    #[test]
    fn test_render_html_errors() {
        let diags = vec![
            Diagnostic::error("E001", "root.version", "version must be 8"),
            Diagnostic::error("E001", "root.name", "name is required"),
            Diagnostic::error("E002", "sources[0]", "url is required"),
        ];
        let html = render_html(&diags, "test.json");
        assert!(html.contains("3 errors"));
        assert!(html.contains("0 warnings"));
        assert!(html.contains("🔴 Errors"));
        assert!(html.contains("E001 (2 occurrence"));
        assert!(html.contains("E002 (1 occurrence"));
        assert!(html.contains("root.version"));
    }

    #[test]
    fn test_render_html_mixed() {
        let diags = vec![
            Diagnostic::error("E001", "root.version", "version must be 8"),
            Diagnostic::warning("W001", "layers[0]", "duplicate id"),
            Diagnostic::info("I001", "sources", "unused source"),
        ];
        let html = render_html(&diags, "style.json");
        assert!(html.contains("1 error"));
        assert!(html.contains("1 warning"));
        assert!(html.contains("1 info"));
        assert!(html.contains("🔴 Errors"));
        assert!(html.contains("🟡 Warnings"));
        assert!(html.contains("🔵 Info"));
    }

    #[test]
    fn test_render_html_with_hints() {
        let diags = vec![
            Diagnostic::error("E001", "root.version", "version must be 8")
                .with_hint("MapLibre v8 spec requires version: 8"),
        ];
        let html = render_html(&diags, "style.json");
        assert!(html.contains("Hint"));
        assert!(html.contains("MapLibre v8 spec requires version: 8"));
    }

    #[test]
    fn test_render_html_escaping() {
        let diags = vec![Diagnostic::error(
            "E001",
            "layers[0].paint",
            "invalid <operator> in expression",
        )
        .with_hint("use \"property\" & \"value\"")];
        let html = render_html(&diags, "style.json");
        assert!(html.contains("&lt;operator&gt;"));
        assert!(html.contains("&amp;"));
        assert!(!html.contains("<operator>"));
    }
}
