use crate::diagnostic::Diagnostic;
use crate::style::{layer::LayerType, types::Source, Style};

/// Paint properties valid per layer type
fn valid_paint_props(lt: &LayerType) -> &'static [&'static str] {
    match lt {
        LayerType::Background => &[
            "background-color", "background-pattern", "background-opacity",
        ],
        LayerType::Fill => &[
            "fill-antialias", "fill-opacity", "fill-color", "fill-outline-color",
            "fill-translate", "fill-translate-anchor", "fill-pattern",
        ],
        LayerType::FillExtrusion => &[
            "fill-extrusion-opacity", "fill-extrusion-color", "fill-extrusion-translate",
            "fill-extrusion-translate-anchor", "fill-extrusion-pattern",
            "fill-extrusion-height", "fill-extrusion-base", "fill-extrusion-vertical-gradient",
        ],
        LayerType::Line => &[
            "line-opacity", "line-color", "line-translate", "line-translate-anchor",
            "line-width", "line-gap-width", "line-offset", "line-blur",
            "line-dasharray", "line-pattern", "line-gradient",
        ],
        LayerType::Symbol => &[
            "icon-opacity", "icon-color", "icon-halo-color", "icon-halo-width",
            "icon-halo-blur", "icon-translate", "icon-translate-anchor",
            "text-opacity", "text-color", "text-halo-color", "text-halo-width",
            "text-halo-blur", "text-translate", "text-translate-anchor",
        ],
        LayerType::Raster => &[
            "raster-opacity", "raster-hue-rotate", "raster-brightness-min",
            "raster-brightness-max", "raster-saturation", "raster-contrast",
            "raster-resampling", "raster-fade-duration",
        ],
        LayerType::Circle => &[
            "circle-radius", "circle-color", "circle-blur", "circle-opacity",
            "circle-translate", "circle-translate-anchor", "circle-pitch-scale",
            "circle-pitch-alignment", "circle-stroke-width", "circle-stroke-color",
            "circle-stroke-opacity",
        ],
        LayerType::Heatmap => &[
            "heatmap-radius", "heatmap-weight", "heatmap-intensity",
            "heatmap-color", "heatmap-opacity",
        ],
        LayerType::Hillshade => &[
            "hillshade-illumination-direction", "hillshade-illumination-anchor",
            "hillshade-exaggeration", "hillshade-shadow-color", "hillshade-highlight-color",
            "hillshade-accent-color",
        ],
        LayerType::Sky => &[
            "sky-type", "sky-atmosphere-sun", "sky-atmosphere-sun-intensity",
            "sky-gradient-center", "sky-gradient-radius", "sky-gradient",
            "sky-atmosphere-halo-color", "sky-atmosphere-color", "sky-opacity",
        ],
    }
}

/// Layout properties valid per layer type
fn valid_layout_props(lt: &LayerType) -> &'static [&'static str] {
    match lt {
        LayerType::Background | LayerType::Hillshade | LayerType::Sky | LayerType::Raster => &[
            "visibility",
        ],
        LayerType::Fill | LayerType::FillExtrusion | LayerType::Circle | LayerType::Heatmap => &[
            "visibility",
        ],
        LayerType::Line => &[
            "visibility", "line-cap", "line-join", "line-miter-limit",
            "line-round-limit", "line-sort-key",
        ],
        LayerType::Symbol => &[
            "visibility",
            "symbol-placement", "symbol-spacing", "symbol-avoid-edges",
            "symbol-sort-key", "symbol-z-order",
            "icon-allow-overlap", "icon-ignore-placement", "icon-optional",
            "icon-rotation-alignment", "icon-size", "icon-text-fit",
            "icon-text-fit-padding", "icon-image", "icon-rotate", "icon-padding",
            "icon-keep-upright", "icon-offset", "icon-anchor", "icon-pitch-alignment",
            "text-pitch-alignment", "text-rotation-alignment", "text-field",
            "text-font", "text-size", "text-max-width", "text-line-height",
            "text-letter-spacing", "text-justify", "text-radial-offset",
            "text-variable-anchor", "text-anchor", "text-max-angle", "text-writing-mode",
            "text-rotate", "text-padding", "text-keep-upright", "text-transform",
            "text-offset", "text-allow-overlap", "text-ignore-placement", "text-optional",
        ],
    }
}

pub fn validate_layers(style: &Style) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (i, layer) in style.layers.iter().enumerate() {
        let path = format!("layers[{}]", i);

        // All non-background layers need a source
        if layer.layer_type != LayerType::Background
            && layer.layer_type != LayerType::Sky
            && layer.source.is_none()
            && layer.layer_ref.is_none()
        {
            diags.push(
                Diagnostic::error(
                    "E004",
                    format!("{}.source", path),
                    format!("layer \"{}\" of type \"{}\" requires a \"source\"", layer.id, layer.layer_type),
                )
                .with_hint("add a \"source\" field referencing a key in the top-level sources object"),
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
                            format!("layer \"{}\" uses a vector source but has no \"source-layer\"", layer.id),
                        )
                        .with_hint("add \"source-layer\" matching the layer name in the vector tile"),
                    );
                }
            }
        }

        // Validate paint properties
        if let Some(paint) = &layer.paint {
            if let Some(obj) = paint.as_object() {
                let valid = valid_paint_props(&layer.layer_type);
                for key in obj.keys() {
                    if !valid.contains(&key.as_str()) {
                        diags.push(Diagnostic::error(
                            "E006",
                            format!("{}.paint.{}", path, key),
                            format!(
                                "\"{}\" is not a valid paint property for \"{}\" layers",
                                key, layer.layer_type
                            ),
                        ));
                    }
                }
            }
        }

        // Validate layout properties
        if let Some(layout) = &layer.layout {
            if let Some(obj) = layout.as_object() {
                let valid = valid_layout_props(&layer.layer_type);
                for key in obj.keys() {
                    if !valid.contains(&key.as_str()) {
                        diags.push(Diagnostic::error(
                            "E007",
                            format!("{}.layout.{}", path, key),
                            format!(
                                "\"{}\" is not a valid layout property for \"{}\" layers",
                                key, layer.layer_type
                            ),
                        ));
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
        let style = parse(r#"{"version":8,"sources":{},"layers":[{"id":"bg","type":"background"}]}"#);
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
        let style = parse(r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{"id":"l","type":"fill","source":"s"}]
        }"#);
        let diags = validate_layers(&style);
        assert!(diags.iter().any(|d| d.code == "E005"));
    }

    #[test]
    fn test_vector_layer_with_source_layer_valid() {
        let style = parse(r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{"id":"l","type":"fill","source":"s","source-layer":"water"}]
        }"#);
        assert!(validate_layers(&style).is_empty());
    }

    #[test]
    fn test_invalid_paint_prop() {
        let style = parse(r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "paint":{"line-width":2}
            }]
        }"#);
        let diags = validate_layers(&style);
        assert!(diags.iter().any(|d| d.code == "E006" && d.path.contains("line-width")));
    }

    #[test]
    fn test_valid_fill_paint() {
        let style = parse(r##"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "paint":{"fill-color":"#0000ff","fill-opacity":0.8}
            }]
        }"##);
        assert!(validate_layers(&style).is_empty());
    }

    #[test]
    fn test_invalid_layout_prop() {
        let style = parse(r#"{
            "version":8,
            "sources":{"s":{"type":"vector","url":"mapbox://x"}},
            "layers":[{
                "id":"l","type":"fill","source":"s","source-layer":"water",
                "layout":{"text-field":"hello"}
            }]
        }"#);
        let diags = validate_layers(&style);
        assert!(diags.iter().any(|d| d.code == "E007" && d.path.contains("text-field")));
    }
}
