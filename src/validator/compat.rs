use crate::diagnostic::Diagnostic;
use crate::style::layer::LayerType;
use crate::style::spec::{SpecAffinity, MAPLIBRE_ONLY_EXPRESSIONS};
use crate::style::Style;
use crate::validator::Validator;

pub struct SkyCompatValidator;
pub struct TerrainCompatValidator;
pub struct FogCompatValidator;
pub struct ExpressionCompatValidator;

impl Validator for SkyCompatValidator {
    fn spec_affinity(&self) -> Option<SpecAffinity> { Some(SpecAffinity::MaplibreOnly) }
    fn validate(&self, style: &Style) -> Vec<Diagnostic> {
        style
            .layers
            .iter()
            .enumerate()
            .filter(|(_, l)| l.layer_type == Some(LayerType::Sky))
            .map(|(i, _)| {
                Diagnostic::error(
                    "E023",
                    format!("layers[{}].type", i),
                    "sky layer is not supported in Mapbox spec",
                )
                .with_hint("remove this layer or switch to --spec maplibre")
            })
            .collect()
    }
}

impl Validator for TerrainCompatValidator {
    fn spec_affinity(&self) -> Option<SpecAffinity> { Some(SpecAffinity::MaplibreOnly) }
    fn validate(&self, style: &Style) -> Vec<Diagnostic> {
        if style.terrain.is_some() {
            vec![Diagnostic::error(
                "E023",
                "terrain",
                "\"terrain\" is not supported in Mapbox spec",
            )
            .with_hint("remove \"terrain\" or switch to --spec maplibre")]
        } else {
            vec![]
        }
    }
}

impl Validator for FogCompatValidator {
    fn spec_affinity(&self) -> Option<SpecAffinity> { Some(SpecAffinity::MaplibreOnly) }
    fn validate(&self, style: &Style) -> Vec<Diagnostic> {
        if style.fog.is_some() {
            vec![Diagnostic::error(
                "E023",
                "fog",
                "\"fog\" is not supported in Mapbox spec",
            )
            .with_hint("remove \"fog\" or switch to --spec maplibre")]
        } else {
            vec![]
        }
    }
}

impl Validator for ExpressionCompatValidator {
    fn spec_affinity(&self) -> Option<SpecAffinity> { Some(SpecAffinity::MaplibreOnly) }
    fn validate(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            let base = format!("layers[{}]", i);
            if let Some(paint) = &layer.paint {
                diags.extend(check_expr_compat(paint, &format!("{}.paint", base)));
            }
            if let Some(layout) = &layer.layout {
                diags.extend(check_expr_compat(layout, &format!("{}.layout", base)));
            }
        }
        diags
    }
}

fn check_expr_compat(value: &serde_json::Value, path: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    match value {
        serde_json::Value::Array(arr) if !arr.is_empty() && arr[0].is_string() => {
            let op = arr[0].as_str().unwrap();
            if MAPLIBRE_ONLY_EXPRESSIONS.contains(&op) {
                diags.push(
                    Diagnostic::error(
                        "E023",
                        path,
                        format!(
                            "expression operator \"{}\" is not supported in Mapbox spec",
                            op
                        ),
                    )
                    .with_hint("remove this expression or switch to --spec maplibre"),
                );
            }
            for (j, arg) in arr[1..].iter().enumerate() {
                diags.extend(check_expr_compat(arg, &format!("{}[{}]", path, j + 1)));
            }
        }
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                diags.extend(check_expr_compat(v, &format!("{}.{}", path, k)));
            }
        }
        _ => {}
    }
    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::Validator;

    fn parse(json: &str) -> crate::style::Style {
        serde_json::from_str(json).unwrap()
    }

    // SkyCompatValidator
    #[test]
    fn sky_compat_emits_e023_for_sky_layer() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[{"id":"s","type":"sky"}]}"#);
        let diags = SkyCompatValidator.validate(&style);
        assert!(diags.iter().any(|d| d.code == "E023" && d.path.contains("layers[0]")));
    }

    #[test]
    fn sky_compat_clean_for_non_sky_layer() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[{"id":"bg","type":"background"}]}"#);
        assert!(SkyCompatValidator.validate(&style).is_empty());
    }

    // TerrainCompatValidator
    #[test]
    fn terrain_compat_emits_e023_when_terrain_present() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[],"terrain":{"source":"dem"}}"#);
        let diags = TerrainCompatValidator.validate(&style);
        assert!(diags.iter().any(|d| d.code == "E023" && d.path == "terrain"));
    }

    #[test]
    fn terrain_compat_clean_without_terrain() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[]}"#);
        assert!(TerrainCompatValidator.validate(&style).is_empty());
    }

    // FogCompatValidator
    #[test]
    fn fog_compat_emits_e023_when_fog_present() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[],"fog":{"color":"white"}}"#);
        let diags = FogCompatValidator.validate(&style);
        assert!(diags.iter().any(|d| d.code == "E023" && d.path == "fog"));
    }

    #[test]
    fn fog_compat_clean_without_fog() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[]}"#);
        assert!(FogCompatValidator.validate(&style).is_empty());
    }

    // ExpressionCompatValidator
    #[test]
    fn expression_compat_emits_e023_for_distance_from_center() {
        let style = parse(r#"{
            "version":8,
            "sources":{},
            "layers":[{
                "id":"c",
                "type":"circle",
                "paint":{"circle-radius":["distance-from-center"]}
            }]
        }"#);
        let diags = ExpressionCompatValidator.validate(&style);
        assert!(diags.iter().any(|d| d.code == "E023" && d.message.contains("distance-from-center")));
    }

    #[test]
    fn expression_compat_clean_for_standard_expression() {
        let style = parse(r#"{
            "version":8,
            "sources":{},
            "layers":[{
                "id":"c",
                "type":"circle",
                "paint":{"circle-radius":["get","zoom"]}
            }]
        }"#);
        assert!(ExpressionCompatValidator.validate(&style).is_empty());
    }
}
