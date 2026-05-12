use crate::diagnostic::Diagnostic;
use crate::linter::LintRule;
use crate::style::{expression::validate_expression, Style};

/// W006: Expression depth exceeds 10 (performance)
pub struct ExpressionDepth;

impl LintRule for ExpressionDepth {
    fn code(&self) -> &'static str {
        "W006"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            let path = format!("layers[{}]", i);
            if let Some(paint) = &layer.paint {
                if let Some(obj) = paint.as_object() {
                    for (key, val) in obj {
                        let p = format!("{}.paint.{}", path, key);
                        diags.extend(
                            validate_expression(val, &p, 0)
                                .into_iter()
                                .filter(|d| d.code == "W006"),
                        );
                    }
                }
            }
            if let Some(layout) = &layer.layout {
                if let Some(obj) = layout.as_object() {
                    for (key, val) in obj {
                        let p = format!("{}.layout.{}", path, key);
                        diags.extend(
                            validate_expression(val, &p, 0)
                                .into_iter()
                                .filter(|d| d.code == "W006"),
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
    use serde_json::json;

    fn style_with_paint(paint_val: serde_json::Value) -> Style {
        let style_json = json!({
            "version": 8,
            "sources": {},
            "layers": [{"id": "bg", "type": "background", "paint": {"background-opacity": paint_val}}]
        });
        serde_json::from_value(style_json).unwrap()
    }

    #[test]
    fn test_shallow_expression_no_warning() {
        let style = style_with_paint(json!([
            "interpolate",
            ["linear"],
            ["zoom"],
            0,
            0.0,
            10,
            1.0
        ]));
        assert!(ExpressionDepth.check(&style).is_empty());
    }

    #[test]
    fn test_deep_expression_warns() {
        let mut expr = json!(["get", "x"]);
        for _ in 0..12 {
            expr = json!(["get", expr]);
        }
        let style = style_with_paint(expr);
        let diags = ExpressionDepth.check(&style);
        assert!(diags.iter().any(|d| d.code == "W006"));
    }
}
