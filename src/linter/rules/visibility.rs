use crate::diagnostic::Diagnostic;
use crate::linter::LintRule;
use crate::style::Style;

/// W002: Layer is permanently invisible (visibility: none)
pub struct PermanentlyInvisible;

impl LintRule for PermanentlyInvisible {
    fn code(&self) -> &'static str {
        "W002"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if let Some(layout) = &layer.layout {
                if let Some(vis) = layout.get("visibility") {
                    if vis.as_str() == Some("none") {
                        diags.push(
                            Diagnostic::warning(
                                "W002",
                                format!("layers[{}].layout.visibility", i),
                                format!("layer \"{}\" is permanently invisible", layer.id),
                            )
                            .with_hint("remove the layer or set visibility to \"visible\" if it should be shown"),
                        );
                    }
                }
            }
        }
        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(json: &str) -> Style {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_visible_layer() {
        let style = parse(
            r#"{"version":8,"sources":{},"layers":[{"id":"a","type":"background","layout":{"visibility":"visible"}}]}"#,
        );
        assert!(PermanentlyInvisible.check(&style).is_empty());
    }

    #[test]
    fn test_invisible_layer() {
        let style = parse(
            r#"{"version":8,"sources":{},"layers":[{"id":"a","type":"background","layout":{"visibility":"none"}}]}"#,
        );
        let diags = PermanentlyInvisible.check(&style);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "W002");
    }
}
