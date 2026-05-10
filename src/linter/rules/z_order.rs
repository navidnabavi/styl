use crate::diagnostic::Diagnostic;
use crate::linter::LintRule;
use crate::style::{layer::LayerType, Style};

/// W005: fill-extrusion layer rendered before a background layer (background will occlude it)
pub struct FillExtrusionBelowBackground;

impl LintRule for FillExtrusionBelowBackground {
    fn code(&self) -> &'static str { "W005" }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // Track fill-extrusion layers seen so far (index, id)
        let mut extrusion_layers: Vec<(usize, &str)> = Vec::new();

        for (i, layer) in style.layers.iter().enumerate() {
            if layer.layer_type == LayerType::FillExtrusion {
                extrusion_layers.push((i, &layer.id));
            }
            // A background at index i will occlude any fill-extrusion at index < i
            if layer.layer_type == LayerType::Background {
                for &(ex_idx, ex_id) in &extrusion_layers {
                    diags.push(
                        Diagnostic::warning(
                            "W005",
                            format!("layers[{}]", ex_idx),
                            format!(
                                "fill-extrusion layer \"{}\" is occluded by background layer \"{}\" at index {}",
                                ex_id, layer.id, i
                            ),
                        )
                        .with_hint("move fill-extrusion layers above all background layers"),
                    );
                }
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
    fn test_extrusion_above_background_ok() {
        // background first, extrusion after → extrusion renders on top → fine
        let style = parse(r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[
            {"id":"bg","type":"background"},
            {"id":"ex","type":"fill-extrusion","source":"s"}
        ]}"#);
        assert!(FillExtrusionBelowBackground.check(&style).is_empty());
    }

    #[test]
    fn test_extrusion_before_background_warns() {
        // extrusion first, background after → background renders on top, occludes extrusion
        let style = parse(r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[
            {"id":"ex","type":"fill-extrusion","source":"s"},
            {"id":"bg","type":"background"}
        ]}"#);
        let diags = FillExtrusionBelowBackground.check(&style);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "W005");
        assert!(diags[0].path.contains("layers[0]"));
    }

    #[test]
    fn test_no_layers_no_warning() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[]}"#);
        assert!(FillExtrusionBelowBackground.check(&style).is_empty());
    }

    #[test]
    fn test_two_extrusions_before_background_warns_both() {
        let style = parse(r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[
            {"id":"ex1","type":"fill-extrusion","source":"s"},
            {"id":"ex2","type":"fill-extrusion","source":"s"},
            {"id":"bg","type":"background"}
        ]}"#);
        let diags = FillExtrusionBelowBackground.check(&style);
        assert_eq!(diags.len(), 2);
    }
}
