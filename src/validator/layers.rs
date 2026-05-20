use crate::diagnostic::Diagnostic;
use crate::style::{
    expression::{is_legacy_filter, validate_expression},
    layer::LayerType,
    types::Source,
    Style,
};
use crate::validator::prop_values::validate_prop_value;
use crate::validator::Validator;

pub struct LayersValidator;
impl Validator for LayersValidator {
    fn validate(&self, style: &crate::style::Style) -> Vec<crate::diagnostic::Diagnostic> {
        validate_layers(style)
    }
}

/// Paint properties valid per layer type
fn valid_paint_props(lt: Option<&LayerType>) -> &'static [&'static str] {
    let lt = match lt {
        Some(lt) => lt,
        None => return &[],
    };
    match lt {
        LayerType::Background => &[
            "background-color",
            "background-pattern",
            "background-opacity",
        ],
        LayerType::Fill => &[
            "fill-antialias",
            "fill-opacity",
            "fill-color",
            "fill-outline-color",
            "fill-translate",
            "fill-translate-anchor",
            "fill-pattern",
        ],
        LayerType::FillExtrusion => &[
            "fill-extrusion-opacity",
            "fill-extrusion-color",
            "fill-extrusion-translate",
            "fill-extrusion-translate-anchor",
            "fill-extrusion-pattern",
            "fill-extrusion-height",
            "fill-extrusion-base",
            "fill-extrusion-vertical-gradient",
        ],
        LayerType::Line => &[
            "line-opacity",
            "line-color",
            "line-translate",
            "line-translate-anchor",
            "line-width",
            "line-gap-width",
            "line-offset",
            "line-blur",
            "line-dasharray",
            "line-pattern",
            "line-gradient",
        ],
        LayerType::Symbol => &[
            "icon-opacity",
            "icon-color",
            "icon-halo-color",
            "icon-halo-width",
            "icon-halo-blur",
            "icon-translate",
            "icon-translate-anchor",
            "text-opacity",
            "text-color",
            "text-halo-color",
            "text-halo-width",
            "text-halo-blur",
            "text-translate",
            "text-translate-anchor",
        ],
        LayerType::Raster => &[
            "raster-opacity",
            "raster-hue-rotate",
            "raster-brightness-min",
            "raster-brightness-max",
            "raster-saturation",
            "raster-contrast",
            "raster-resampling",
            "raster-fade-duration",
        ],
        LayerType::Circle => &[
            "circle-radius",
            "circle-color",
            "circle-blur",
            "circle-opacity",
            "circle-translate",
            "circle-translate-anchor",
            "circle-pitch-scale",
            "circle-pitch-alignment",
            "circle-stroke-width",
            "circle-stroke-color",
            "circle-stroke-opacity",
        ],
        LayerType::Heatmap => &[
            "heatmap-radius",
            "heatmap-weight",
            "heatmap-intensity",
            "heatmap-color",
            "heatmap-opacity",
        ],
        LayerType::Hillshade => &[
            "hillshade-illumination-direction",
            "hillshade-illumination-anchor",
            "hillshade-exaggeration",
            "hillshade-shadow-color",
            "hillshade-highlight-color",
            "hillshade-accent-color",
        ],
        LayerType::Sky => &[
            "sky-type",
            "sky-atmosphere-sun",
            "sky-atmosphere-sun-intensity",
            "sky-gradient-center",
            "sky-gradient-radius",
            "sky-gradient",
            "sky-atmosphere-halo-color",
            "sky-atmosphere-color",
            "sky-opacity",
        ],
        LayerType::ColorRelief => &["color-relief-color-range", "color-relief-opacity"],
    }
}

/// Layout properties valid per layer type
fn valid_layout_props(lt: Option<&LayerType>) -> &'static [&'static str] {
    let lt = match lt {
        Some(lt) => lt,
        None => return &[],
    };
    match lt {
        LayerType::Background
        | LayerType::Hillshade
        | LayerType::Sky
        | LayerType::Raster
        | LayerType::FillExtrusion
        | LayerType::Heatmap
        | LayerType::ColorRelief => &["visibility"],
        LayerType::Fill => &["visibility", "fill-sort-key"],
        LayerType::Circle => &["visibility", "circle-sort-key"],
        LayerType::Line => &[
            "visibility",
            "line-cap",
            "line-join",
            "line-miter-limit",
            "line-round-limit",
            "line-sort-key",
        ],
        LayerType::Symbol => &[
            "visibility",
            "symbol-placement",
            "symbol-spacing",
            "symbol-avoid-edges",
            "symbol-sort-key",
            "symbol-z-order",
            "icon-allow-overlap",
            "icon-ignore-placement",
            "icon-optional",
            "icon-rotation-alignment",
            "icon-size",
            "icon-text-fit",
            "icon-text-fit-padding",
            "icon-image",
            "icon-rotate",
            "icon-padding",
            "icon-keep-upright",
            "icon-offset",
            "icon-anchor",
            "icon-pitch-alignment",
            "text-pitch-alignment",
            "text-rotation-alignment",
            "text-field",
            "text-font",
            "text-size",
            "text-max-width",
            "text-line-height",
            "text-letter-spacing",
            "text-justify",
            "text-radial-offset",
            "text-variable-anchor",
            "text-anchor",
            "text-max-angle",
            "text-writing-mode",
            "text-rotate",
            "text-padding",
            "text-keep-upright",
            "text-transform",
            "text-offset",
            "text-allow-overlap",
            "text-ignore-placement",
            "text-optional",
        ],
    }
}

pub fn validate_layers(style: &Style) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (i, layer) in style.layers.iter().enumerate() {
        let path = format!("layers[{}]", i);

        // Validate minzoom/maxzoom ranges
        if let Some(z) = layer.minzoom {
            if !(0.0..=24.0).contains(&z) {
                diags.push(Diagnostic::error(
                    "E014",
                    format!("{}.minzoom", path),
                    format!("minzoom {} is out of range [0, 24]", z),
                ));
            }
        }
        if let Some(z) = layer.maxzoom {
            if !(0.0..=24.0).contains(&z) {
                diags.push(Diagnostic::error(
                    "E014",
                    format!("{}.maxzoom", path),
                    format!("maxzoom {} is out of range [0, 24]", z),
                ));
            }
        }
        if let (Some(min), Some(max)) = (layer.minzoom, layer.maxzoom) {
            if min > max {
                diags.push(Diagnostic::error(
                    "E014",
                    format!("{}.minzoom", path),
                    format!("minzoom ({}) must be <= maxzoom ({})", min, max),
                ));
            }
        }

        // All non-background layers need a source
        if !matches!(
            &layer.layer_type,
            Some(LayerType::Background) | Some(LayerType::Sky)
        ) && layer.source.is_none()
            && layer.layer_ref.is_none()
        {
            let type_str = layer
                .layer_type
                .as_ref()
                .map_or("unknown".to_string(), |lt| lt.to_string());
            diags.push(
                Diagnostic::error(
                    "E004",
                    format!("{}.source", path),
                    format!(
                        "layer \"{}\" of type \"{}\" requires a \"source\"",
                        layer.id, type_str
                    ),
                )
                .with_hint(
                    "add a \"source\" field referencing a key in the top-level sources object",
                ),
            );
        }

        // Vector sources require source-layer
        if let Some(source_id) = &layer.source {
            if let Some(source) = style.sources.get(source_id) {
                if matches!(source, Source::Vector(_)) && layer.source_layer.is_none() {
                    diags.push(
                        Diagnostic::error(
                            "E005",
                            format!("{}.source-layer", path),
                            format!(
                                "layer \"{}\" uses a vector source but has no \"source-layer\"",
                                layer.id
                            ),
                        )
                        .with_hint(
                            "add \"source-layer\" matching the layer name in the vector tile",
                        ),
                    );
                }
            }
        }

        // Validate filter expression (skip legacy filters — W011 handles those)
        if let Some(filter) = &layer.filter {
            if !is_legacy_filter(filter) {
                let filter_path = format!("{}.filter", path);
                diags.extend(validate_expression(filter, &filter_path, 0));
            }
        }

        // Validate paint properties
        if let Some(paint) = &layer.paint {
            if let Some(obj) = paint.as_object() {
                let valid = valid_paint_props(layer.layer_type.as_ref());
                let type_str = layer
                    .layer_type
                    .as_ref()
                    .map_or("unknown".to_string(), |lt| lt.to_string());
                for key in obj.keys() {
                    if !valid.contains(&key.as_str()) {
                        diags.push(Diagnostic::error(
                            "E006",
                            format!("{}.paint.{}", path, key),
                            format!(
                                "\"{}\" is not a valid paint property for \"{}\" layers",
                                key, type_str
                            ),
                        ));
                    }
                }
            }
        }

        // Validate layout properties
        if let Some(layout) = &layer.layout {
            if let Some(obj) = layout.as_object() {
                let valid = valid_layout_props(layer.layer_type.as_ref());
                let type_str = layer
                    .layer_type
                    .as_ref()
                    .map_or("unknown".to_string(), |lt| lt.to_string());
                for key in obj.keys() {
                    if !valid.contains(&key.as_str()) {
                        diags.push(Diagnostic::error(
                            "E007",
                            format!("{}.layout.{}", path, key),
                            format!(
                                "\"{}\" is not a valid layout property for \"{}\" layers",
                                key, type_str
                            ),
                        ));
                    }
                }
            }
        }

        // Validate paint property values (E018) — only for known-valid props
        if let Some(paint) = &layer.paint {
            if let Some(obj) = paint.as_object() {
                let valid = valid_paint_props(layer.layer_type.as_ref());
                for (key, value) in obj {
                    if valid.contains(&key.as_str()) {
                        let prop_path = format!("{}.paint.{}", path, key);
                        diags.extend(validate_prop_value(key, value, &prop_path));
                    }
                }
            }
        }

        // Validate layout property values (E018) — only for known-valid props
        if let Some(layout) = &layer.layout {
            if let Some(obj) = layout.as_object() {
                let valid = valid_layout_props(layer.layer_type.as_ref());
                for (key, value) in obj {
                    if valid.contains(&key.as_str()) {
                        let prop_path = format!("{}.layout.{}", path, key);
                        diags.extend(validate_prop_value(key, value, &prop_path));
                    }
                }
            }
        }
    }

    // Validate ref layers inherit parent's paint/layout
    for (i, layer) in style.layers.iter().enumerate() {
        if let Some(ref_id) = &layer.layer_ref {
            let path = format!("layers[{}]", i);
            // Find parent layer by ID
            if let Some((_parent_idx, parent_layer)) = style
                .layers
                .iter()
                .enumerate()
                .find(|(_, l)| l.id == *ref_id)
            {
                // Validate parent's paint as if on ref layer (inherits parent type)
                if let Some(parent_paint) = &parent_layer.paint {
                    if let Some(obj) = parent_paint.as_object() {
                        let valid = valid_paint_props(parent_layer.layer_type.as_ref());
                        for (key, value) in obj {
                            if valid.contains(&key.as_str()) {
                                let prop_path = format!("{}.paint.{}", path, key);
                                diags.extend(validate_prop_value(key, value, &prop_path));
                            }
                        }
                    }
                }

                // Validate parent's layout as if on ref layer (inherits parent type)
                if let Some(parent_layout) = &parent_layer.layout {
                    if let Some(obj) = parent_layout.as_object() {
                        let valid = valid_layout_props(parent_layer.layer_type.as_ref());
                        for (key, value) in obj {
                            if valid.contains(&key.as_str()) {
                                let prop_path = format!("{}.layout.{}", path, key);
                                diags.extend(validate_prop_value(key, value, &prop_path));
                            }
                        }
                    }
                }
            }
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Style {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_background_layer_no_source_required() {
        let style =
            parse(r#"{"version":8,"sources":{},"layers":[{"id":"bg","type":"background"}]}"#);
        assert!(validate_layers(&style).is_empty());
    }

    #[test]
    fn test_non_background_missing_source() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[{"id":"l","type":"fill"}]}"#);
        let diags = validate_layers(&style);
        assert!(diags.iter().any(|d| d.code == "E004"));
    }

    #[test]
    fn test_vector_layer_missing_source_layer() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{"id":"l","type":"fill","source":"s"}]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(diags.iter().any(|d| d.code == "E005"));
    }

    #[test]
    fn test_vector_layer_with_source_layer_valid() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{"id":"l","type":"fill","source":"s","source-layer":"water"}]
        }"#,
        );
        assert!(validate_layers(&style).is_empty());
    }

    #[test]
    fn test_invalid_paint_prop() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "paint":{"line-width":2}
            }]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E006" && d.path.contains("line-width")));
    }

    #[test]
    fn test_valid_fill_paint() {
        let style = parse(
            r##"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "paint":{"fill-color":"#0000ff","fill-opacity":0.8}
            }]
        }"##,
        );
        assert!(validate_layers(&style).is_empty());
    }

    #[test]
    fn test_layer_invalid_minzoom() {
        let style = parse(
            r#"{"version":8,"sources":{},"layers":[{"id":"bg","type":"background","minzoom":25}]}"#,
        );
        let diags = validate_layers(&style);
        assert!(diags.iter().any(|d| d.code == "E014"));
    }

    #[test]
    fn test_layer_minzoom_gt_maxzoom() {
        let style = parse(
            r#"{"version":8,"sources":{},"layers":[{"id":"bg","type":"background","minzoom":10,"maxzoom":5}]}"#,
        );
        let diags = validate_layers(&style);
        assert!(diags.iter().any(|d| d.code == "E014"));
    }

    #[test]
    fn test_layer_valid_zoom_range() {
        let style = parse(
            r#"{"version":8,"sources":{},"layers":[{"id":"bg","type":"background","minzoom":0,"maxzoom":24}]}"#,
        );
        assert!(validate_layers(&style).is_empty());
    }

    #[test]
    fn test_valid_expression_filter() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{"id":"l","type":"fill","source":"s","source-layer":"water",
                "filter":["==",["get","class"],"lake"]}]
        }"#,
        );
        assert!(validate_layers(&style)
            .iter()
            .all(|d| d.code != "E022" && d.code != "E021"));
    }

    #[test]
    fn test_filter_unknown_operator() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{"id":"l","type":"fill","source":"s","source-layer":"water",
                "filter":["not-an-op","value"]}]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(diags.iter().any(|d| d.code == "E022"));
    }

    #[test]
    fn test_filter_empty_array() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{"id":"l","type":"fill","source":"s","source-layer":"water",
                "filter":[]}]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(diags.iter().any(|d| d.code == "E020"));
    }

    #[test]
    fn test_legacy_filter_not_expression_validated() {
        // Legacy filters should not generate E022 — W011 handles them separately
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{"id":"l","type":"fill","source":"s","source-layer":"water",
                "filter":["==","class","road"]}]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(!diags.iter().any(|d| d.code == "E022"));
    }

    #[test]
    fn test_invalid_layout_prop() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "layout":{"text-field":"hello"}
            }]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E007" && d.path.contains("text-field")));
    }

    #[test]
    fn test_paint_value_opacity_out_of_range() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "paint":{"fill-opacity":2.0}
            }]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E018" && d.path.contains("fill-opacity")));
    }

    #[test]
    fn test_paint_value_enum_invalid() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"line","source":"s","source-layer":"roads",
                "layout":{"line-cap":"flat"}
            }]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E018" && d.path.contains("line-cap")));
    }

    #[test]
    fn test_paint_value_color_invalid() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "paint":{"fill-color":"notacolor"}
            }]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E018" && d.path.contains("fill-color")));
    }

    #[test]
    fn test_paint_value_expression_not_e018() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "paint":{"fill-opacity":["get","opacity"]}
            }]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(!diags.iter().any(|d| d.code == "E018"));
    }

    #[test]
    fn test_paint_value_color_named_valid() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "paint":{"fill-color":"red"}
            }]
        }"#,
        );
        assert!(validate_layers(&style).iter().all(|d| d.code != "E018"));
    }

    #[test]
    fn test_layout_value_visibility_invalid() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{},"layers":[{
                "id":"bg","type":"background",
                "layout":{"visibility":"hidden"}
            }]
        }"#,
        );
        let diags = validate_layers(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E018" && d.path.contains("visibility")));
    }

    fn from_json(v: serde_json::Value) -> Style {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn test_ref_layer_invalid_inherited_paint() {
        // "ref" key requires json!() macro — raw strings trigger Rust lexer bug on r"
        let style = from_json(serde_json::json!({
            "version": 8,
            "sources": {"s": {"type": "vector", "url": "mapbox://x"}},
            "layers": [
                {"id": "base", "type": "fill", "source": "s", "source-layer": "water",
                 "paint": {"fill-opacity": 2.0}},
                {"id": "derived", "type": "fill", "ref": "base"}
            ]
        }));
        let diags = validate_layers(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E018" && d.path.contains("layers[1]")));
    }

    #[test]
    fn test_ref_layer_valid_inherited_paint() {
        let style = from_json(serde_json::json!({
            "version": 8,
            "sources": {"s": {"type": "vector", "url": "mapbox://x"}},
            "layers": [
                {"id": "base", "type": "fill", "source": "s", "source-layer": "water",
                 "paint": {"fill-opacity": 0.8, "fill-color": "#ff0000"}},
                {"id": "derived", "type": "fill", "ref": "base"}
            ]
        }));
        assert!(validate_layers(&style).iter().all(|d| d.code != "E018"));
    }

    #[test]
    fn test_ref_layer_invalid_inherited_layout() {
        let style = from_json(serde_json::json!({
            "version": 8,
            "sources": {"s": {"type": "vector", "url": "mapbox://x"}},
            "layers": [
                {"id": "base", "type": "line", "source": "s", "source-layer": "roads",
                 "layout": {"line-cap": "invalid"}},
                {"id": "derived", "type": "line", "ref": "base"}
            ]
        }));
        let diags = validate_layers(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E018" && d.path.contains("layers[1]")));
    }

    #[test]
    fn test_ref_to_nonexistent_no_crash() {
        let style = from_json(serde_json::json!({
            "version": 8,
            "sources": {},
            "layers": [{"id": "orphan", "type": "fill", "ref": "missing"}]
        }));
        assert!(!validate_layers(&style).iter().any(|d| d.code == "E018"));
    }
}
