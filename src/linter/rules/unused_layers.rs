use crate::diagnostic::Diagnostic;
use crate::linter::LintRule;
use crate::style::Style;

/// W003: Unused source (defined but not referenced by any layer)
pub struct UnusedSource;

impl LintRule for UnusedSource {
    fn code(&self) -> &'static str {
        "W003"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let used: std::collections::HashSet<&str> = style
            .layers
            .iter()
            .filter_map(|l| l.source.as_deref())
            .collect();

        style
            .sources
            .keys()
            .filter(|id| !used.contains(id.as_str()))
            .map(|id| {
                Diagnostic::warning(
                    "W003",
                    format!("sources.{}", id),
                    format!("source \"{}\" is defined but not used by any layer", id),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(json: &str) -> Style {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_used_source() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[{"id":"l","type":"fill","source":"s"}]}"#,
        );
        assert!(UnusedSource.check(&style).is_empty());
    }

    #[test]
    fn test_unused_source() {
        let style =
            parse(r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[]}"#);
        let diags = UnusedSource.check(&style);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "W003");
    }
}
