use std::collections::HashSet;
use crate::diagnostic::Diagnostic;
use crate::linter::LintRule;
use crate::style::Style;

/// W001: Duplicate layer ID
pub struct DuplicateIds;

impl LintRule for DuplicateIds {
    fn code(&self) -> &'static str { "W001" }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut seen = HashSet::new();
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if !seen.insert(layer.id.as_str()) {
                diags.push(Diagnostic::warning(
                    "W001",
                    format!("layers[{}].id", i),
                    format!("duplicate layer id \"{}\"", layer.id),
                ));
            }
        }
        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(json: &str) -> Style { serde_json::from_str(json).unwrap() }

    #[test]
    fn test_no_duplicates() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[{"id":"a","type":"background"},{"id":"b","type":"background"}]}"#);
        assert!(DuplicateIds.check(&style).is_empty());
    }

    #[test]
    fn test_duplicate() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[{"id":"a","type":"background"},{"id":"a","type":"background"}]}"#);
        let diags = DuplicateIds.check(&style);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "W001");
    }
}
