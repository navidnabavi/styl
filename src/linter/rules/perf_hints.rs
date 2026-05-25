use crate::diagnostic::Diagnostic;
use crate::linter::LintRule;
use crate::style::{expression::is_legacy_filter, layer::LayerType, Style};

/// W007: text-field is empty string
pub struct EmptyTextField;

impl LintRule for EmptyTextField {
    fn code(&self) -> &'static str {
        "W007"
    }

    fn is_fixable(&self) -> bool {
        true
    }

    fn fix(&self, value: &mut serde_json::Value) {
        if let Some(layers) = value.get_mut("layers").and_then(|l| l.as_array_mut()) {
            for layer in layers.iter_mut() {
                if let Some(layout) = layer.get_mut("layout").and_then(|l| l.as_object_mut()) {
                    if layout.get("text-field").and_then(|v| v.as_str()) == Some("") {
                        layout.remove("text-field");
                    }
                }
            }
        }
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if let Some(layout) = &layer.layout {
                if let Some(tf) = layout.get("text-field") {
                    if tf.as_str() == Some("") {
                        diags.push(
                            Diagnostic::warning(
                                "W007",
                                format!("layers[{}].layout.text-field", i),
                                format!("layer \"{}\" has an empty text-field", layer.id),
                            )
                            .with_hint(
                                "remove text-field or provide a value such as [\"get\", \"name\"]",
                            ),
                        );
                    }
                }
            }
        }
        diags
    }
}

/// W008: icon-image references a sprite name that looks like a placeholder
pub struct PlaceholderIconImage;

const PLACEHOLDER_PATTERNS: &[&str] = &[
    "TODO",
    "FIXME",
    "placeholder",
    "example",
    "test-icon",
    "my-icon",
];

impl LintRule for PlaceholderIconImage {
    fn code(&self) -> &'static str {
        "W008"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if let Some(layout) = &layer.layout {
                if let Some(icon) = layout.get("icon-image") {
                    if let Some(name) = icon.as_str() {
                        if PLACEHOLDER_PATTERNS.iter().any(|p| name.contains(p)) {
                            diags.push(
                                Diagnostic::warning(
                                    "W008",
                                    format!("layers[{}].layout.icon-image", i),
                                    format!("icon-image \"{}\" looks like a placeholder", name),
                                )
                                .with_hint(
                                    "replace with the actual sprite name from your sprite sheet",
                                ),
                            );
                        }
                    }
                }
            }
        }
        diags
    }
}

/// W009: Layer count exceeds 200 (performance hint)
pub struct LayerCountHint;

impl LintRule for LayerCountHint {
    fn code(&self) -> &'static str {
        "W009"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        if style.layers.len() > 200 {
            vec![Diagnostic::info(
                "W009",
                "layers",
                format!(
                    "style has {} layers (>200); consider merging layers for better performance",
                    style.layers.len()
                ),
            )]
        } else {
            vec![]
        }
    }
}

/// W010: line-dasharray contains a zero-length segment
pub struct ZeroDasharray;

impl LintRule for ZeroDasharray {
    fn code(&self) -> &'static str {
        "W010"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if let Some(paint) = &layer.paint {
                if let Some(da) = paint.get("line-dasharray") {
                    if let Some(arr) = da.as_array() {
                        if arr.iter().any(|v| v.as_f64() == Some(0.0)) {
                            diags.push(
                                Diagnostic::warning(
                                    "W010",
                                    format!("layers[{}].paint.line-dasharray", i),
                                    "line-dasharray contains a zero-length segment",
                                )
                                .with_hint("all segments in line-dasharray must be > 0"),
                            );
                        }
                    }
                }
            }
        }
        diags
    }
}

/// W011: Deprecated legacy filter syntax (array-based, not expression-based)
pub struct LegacyFilter;

impl LintRule for LegacyFilter {
    fn code(&self) -> &'static str {
        "W011"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if let Some(filter) = &layer.filter {
                if is_legacy_filter(filter) {
                    diags.push(
                        Diagnostic::warning(
                            "W011",
                            format!("layers[{}].filter", i),
                            format!("layer \"{}\" uses deprecated legacy filter syntax", layer.id),
                        )
                        .with_hint("migrate to expression-based filters: https://maplibre.org/maplibre-style-spec/expressions/"),
                    );
                }
            }
        }
        diags
    }
}

/// W012: raster-resampling not set on raster layers (defaults may cause blur)
pub struct RasterResampling;

impl LintRule for RasterResampling {
    fn code(&self) -> &'static str {
        "W012"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if layer.layer_type == Some(LayerType::Raster) {
                let has_resampling = layer
                    .paint
                    .as_ref()
                    .and_then(|p| p.get("raster-resampling"))
                    .is_some();
                if !has_resampling {
                    diags.push(
                        Diagnostic::info(
                            "W012",
                            format!("layers[{}].paint", i),
                            format!(
                                "raster layer \"{}\" does not set raster-resampling",
                                layer.id
                            ),
                        )
                        .with_hint(
                            "set \"raster-resampling\": \"nearest\" to avoid blur when overzooming",
                        ),
                    );
                }
            }
        }
        diags
    }
}

/// W013: Symbol layer has neither text-field nor icon-image — renders nothing
pub struct SymbolNoContent;

impl LintRule for SymbolNoContent {
    fn code(&self) -> &'static str {
        "W013"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if layer.layer_type == Some(LayerType::Symbol) {
                let layout = layer.layout.as_ref();
                let has_text = layout.and_then(|l| l.get("text-field")).is_some();
                let has_icon = layout.and_then(|l| l.get("icon-image")).is_some();
                if !has_text && !has_icon {
                    diags.push(
                        Diagnostic::warning(
                            "W013",
                            format!("layers[{}]", i),
                            format!(
                                "symbol layer \"{}\" has neither text-field nor icon-image and renders nothing",
                                layer.id
                            ),
                        )
                        .with_hint("add text-field or icon-image to make this layer visible"),
                    );
                }
            }
        }
        diags
    }
}

/// W014: Symbol layer uses text-field without text-font
pub struct SymbolMissingFont;

impl LintRule for SymbolMissingFont {
    fn code(&self) -> &'static str {
        "W014"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if layer.layer_type == Some(LayerType::Symbol) {
                if let Some(layout) = &layer.layout {
                    let has_text = layout.get("text-field").is_some();
                    let has_font = layout.get("text-font").is_some();
                    if has_text && !has_font {
                        diags.push(
                            Diagnostic::info(
                                "W014",
                                format!("layers[{}].layout", i),
                                format!(
                                    "symbol layer \"{}\" uses text-field without text-font",
                                    layer.id
                                ),
                            )
                            .with_hint(
                                "set text-font to an explicit font stack; omitting falls back to renderer default",
                            ),
                        );
                    }
                }
            }
        }
        diags
    }
}

/// W015: Background layer sets both background-pattern and background-color (pattern wins)
pub struct BackgroundPatternOverridesColor;

impl LintRule for BackgroundPatternOverridesColor {
    fn code(&self) -> &'static str {
        "W015"
    }

    fn is_fixable(&self) -> bool {
        true
    }

    fn fix(&self, value: &mut serde_json::Value) {
        if let Some(layers) = value.get_mut("layers").and_then(|l| l.as_array_mut()) {
            for layer in layers.iter_mut() {
                if layer.get("type").and_then(|t| t.as_str()) == Some("background") {
                    if let Some(paint) = layer.get_mut("paint").and_then(|p| p.as_object_mut()) {
                        if paint.contains_key("background-pattern") {
                            paint.remove("background-color");
                        }
                    }
                }
            }
        }
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if layer.layer_type == Some(LayerType::Background) {
                if let Some(paint) = &layer.paint {
                    let has_pattern = paint.get("background-pattern").is_some();
                    let has_color = paint.get("background-color").is_some();
                    if has_pattern && has_color {
                        diags.push(
                            Diagnostic::warning(
                                "W015",
                                format!("layers[{}].paint", i),
                                format!(
                                    "background layer \"{}\" sets both background-pattern and background-color; pattern takes precedence",
                                    layer.id
                                ),
                            )
                            .with_hint("remove background-color — it has no effect when background-pattern is set"),
                        );
                    }
                }
            }
        }
        diags
    }
}

/// W016: Fill layer sets both fill-pattern and fill-color (pattern takes precedence)
pub struct FillPatternOverridesColor;

impl LintRule for FillPatternOverridesColor {
    fn code(&self) -> &'static str {
        "W016"
    }

    fn is_fixable(&self) -> bool {
        true
    }

    fn fix(&self, value: &mut serde_json::Value) {
        if let Some(layers) = value.get_mut("layers").and_then(|l| l.as_array_mut()) {
            for layer in layers.iter_mut() {
                if layer.get("type").and_then(|t| t.as_str()) == Some("fill") {
                    if let Some(paint) = layer.get_mut("paint").and_then(|p| p.as_object_mut()) {
                        if paint.contains_key("fill-pattern") {
                            paint.remove("fill-color");
                        }
                    }
                }
            }
        }
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if layer.layer_type == Some(LayerType::Fill) {
                if let Some(paint) = &layer.paint {
                    let has_pattern = paint.get("fill-pattern").is_some();
                    let has_color = paint.get("fill-color").is_some();
                    if has_pattern && has_color {
                        diags.push(
                            Diagnostic::warning(
                                "W016",
                                format!("layers[{}].paint", i),
                                format!(
                                    "fill layer \"{}\" sets both fill-pattern and fill-color; pattern takes precedence",
                                    layer.id
                                ),
                            )
                            .with_hint("remove fill-color — it has no effect when fill-pattern is set"),
                        );
                    }
                }
            }
        }
        diags
    }
}

/// W017: Line layer sets both line-pattern and line-color (pattern takes precedence)
pub struct LinePatternOverridesColor;

impl LintRule for LinePatternOverridesColor {
    fn code(&self) -> &'static str {
        "W017"
    }

    fn is_fixable(&self) -> bool {
        true
    }

    fn fix(&self, value: &mut serde_json::Value) {
        if let Some(layers) = value.get_mut("layers").and_then(|l| l.as_array_mut()) {
            for layer in layers.iter_mut() {
                if layer.get("type").and_then(|t| t.as_str()) == Some("line") {
                    if let Some(paint) = layer.get_mut("paint").and_then(|p| p.as_object_mut()) {
                        if paint.contains_key("line-pattern") {
                            paint.remove("line-color");
                        }
                    }
                }
            }
        }
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if layer.layer_type == Some(LayerType::Line) {
                if let Some(paint) = &layer.paint {
                    let has_pattern = paint.get("line-pattern").is_some();
                    let has_color = paint.get("line-color").is_some();
                    if has_pattern && has_color {
                        diags.push(
                            Diagnostic::warning(
                                "W017",
                                format!("layers[{}].paint", i),
                                format!(
                                    "line layer \"{}\" sets both line-pattern and line-color; pattern takes precedence",
                                    layer.id
                                ),
                            )
                            .with_hint("remove line-color — it has no effect when line-pattern is set"),
                        );
                    }
                }
            }
        }
        diags
    }
}

/// W018: Heatmap layer missing heatmap-color expression (renders monochrome)
pub struct HeatmapMissingColor;

impl LintRule for HeatmapMissingColor {
    fn code(&self) -> &'static str {
        "W018"
    }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            if layer.layer_type == Some(LayerType::Heatmap) {
                let has_color = layer
                    .paint
                    .as_ref()
                    .and_then(|p| p.get("heatmap-color"))
                    .is_some();
                if !has_color {
                    diags.push(
                        Diagnostic::info(
                            "W018",
                            format!("layers[{}].paint", i),
                            format!(
                                "heatmap layer \"{}\" does not set heatmap-color and renders monochrome",
                                layer.id
                            ),
                        )
                        .with_hint(
                            "set heatmap-color to an interpolate expression for a meaningful color ramp",
                        ),
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
    fn parse(json: &str) -> Style {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_empty_text_field() {
        let style = parse(
            r#"{"version":8,"sources":{},"layers":[{"id":"s","type":"symbol","source":"x","layout":{"text-field":""}}]}"#,
        );
        assert!(EmptyTextField
            .check(&style)
            .iter()
            .any(|d| d.code == "W007"));
    }

    #[test]
    fn test_non_empty_text_field_ok() {
        let style = parse(
            r#"{"version":8,"sources":{},"layers":[{"id":"s","type":"symbol","source":"x","layout":{"text-field":"hello"}}]}"#,
        );
        assert!(EmptyTextField.check(&style).is_empty());
    }

    #[test]
    fn test_placeholder_icon() {
        let style = parse(
            r#"{"version":8,"sources":{},"layers":[{"id":"s","type":"symbol","source":"x","layout":{"icon-image":"my-icon-24"}}]}"#,
        );
        assert!(PlaceholderIconImage
            .check(&style)
            .iter()
            .any(|d| d.code == "W008"));
    }

    #[test]
    fn test_zero_dasharray() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[{"id":"l","type":"line","source":"s","paint":{"line-dasharray":[2,0,2]}}]}"#,
        );
        assert!(ZeroDasharray.check(&style).iter().any(|d| d.code == "W010"));
    }

    #[test]
    fn test_valid_dasharray() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[{"id":"l","type":"line","source":"s","paint":{"line-dasharray":[2,4]}}]}"#,
        );
        assert!(ZeroDasharray.check(&style).is_empty());
    }

    #[test]
    fn test_legacy_filter() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[{"id":"l","type":"fill","source":"s","filter":["==","class","road"]}]}"#,
        );
        assert!(LegacyFilter.check(&style).iter().any(|d| d.code == "W011"));
    }

    #[test]
    fn test_expression_filter_ok() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[{"id":"l","type":"fill","source":"s","filter":["match",["get","class"],["road"],true,false]}]}"#,
        );
        assert!(LegacyFilter.check(&style).is_empty());
    }

    #[test]
    fn test_modern_all_filter_not_flagged() {
        // ["all", expr1, expr2] is a valid modern expression filter — must not be flagged
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[{"id":"l","type":"fill","source":"s","filter":["all",["==",["get","class"],"road"],["has","name"]]}]}"#,
        );
        assert!(LegacyFilter.check(&style).is_empty());
    }

    #[test]
    fn test_legacy_all_filter_flagged() {
        // ["all", ["==", "class", "road"], ...] — second arg of inner item is plain string → legacy
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[{"id":"l","type":"fill","source":"s","filter":["all",["==","class","road"],["in","type","primary","secondary"]]}]}"#,
        );
        assert!(LegacyFilter.check(&style).iter().any(|d| d.code == "W011"));
    }

    #[test]
    fn test_raster_missing_resampling() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"raster","url":"mapbox://x"}},"layers":[{"id":"r","type":"raster","source":"s"}]}"#,
        );
        assert!(RasterResampling
            .check(&style)
            .iter()
            .any(|d| d.code == "W012"));
    }

    #[test]
    fn test_raster_with_resampling_ok() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"raster","url":"mapbox://x"}},"layers":[{"id":"r","type":"raster","source":"s","paint":{"raster-resampling":"nearest"}}]}"#,
        );
        assert!(RasterResampling.check(&style).is_empty());
    }

    #[test]
    fn test_symbol_no_content() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://x"}},"layers":[{"id":"l","type":"symbol","source":"s","source-layer":"x"}]}"#,
        );
        assert!(SymbolNoContent
            .check(&style)
            .iter()
            .any(|d| d.code == "W013"));
    }

    #[test]
    fn test_symbol_with_text_field_ok() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://x"}},"layers":[{"id":"l","type":"symbol","source":"s","source-layer":"x","layout":{"text-field":["get","name"]}}]}"#,
        );
        assert!(SymbolNoContent.check(&style).is_empty());
    }

    #[test]
    fn test_symbol_with_icon_image_ok() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://x"}},"layers":[{"id":"l","type":"symbol","source":"s","source-layer":"x","layout":{"icon-image":"marker"}}]}"#,
        );
        assert!(SymbolNoContent.check(&style).is_empty());
    }

    #[test]
    fn test_symbol_missing_font() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://x"}},"layers":[{"id":"l","type":"symbol","source":"s","source-layer":"x","layout":{"text-field":["get","name"]}}]}"#,
        );
        assert!(SymbolMissingFont
            .check(&style)
            .iter()
            .any(|d| d.code == "W014"));
    }

    #[test]
    fn test_symbol_with_font_ok() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://x"}},"layers":[{"id":"l","type":"symbol","source":"s","source-layer":"x","layout":{"text-field":["get","name"],"text-font":["Open Sans Regular"]}}]}"#,
        );
        assert!(SymbolMissingFont.check(&style).is_empty());
    }

    #[test]
    fn test_background_pattern_overrides_color() {
        let style = parse("{\"version\":8,\"sources\":{},\"layers\":[{\"id\":\"bg\",\"type\":\"background\",\"paint\":{\"background-color\":\"#fff\",\"background-pattern\":\"dots\"}}]}");
        assert!(BackgroundPatternOverridesColor
            .check(&style)
            .iter()
            .any(|d| d.code == "W015"));
    }

    #[test]
    fn test_background_pattern_only_ok() {
        let style = parse("{\"version\":8,\"sources\":{},\"layers\":[{\"id\":\"bg\",\"type\":\"background\",\"paint\":{\"background-pattern\":\"dots\"}}]}");
        assert!(BackgroundPatternOverridesColor.check(&style).is_empty());
    }

    #[test]
    fn test_background_color_only_ok() {
        let style = parse("{\"version\":8,\"sources\":{},\"layers\":[{\"id\":\"bg\",\"type\":\"background\",\"paint\":{\"background-color\":\"#fff\"}}]}");
        assert!(BackgroundPatternOverridesColor.check(&style).is_empty());
    }

    #[test]
    fn test_layer_count_hint() {
        // Build 201 layers
        let layers: Vec<serde_json::Value> = (0..201)
            .map(|i| serde_json::json!({"id": format!("l{}", i), "type": "background"}))
            .collect();
        let style_json = serde_json::json!({
            "version": 8, "sources": {}, "layers": layers
        });
        let style: Style = serde_json::from_value(style_json).unwrap();
        assert!(LayerCountHint
            .check(&style)
            .iter()
            .any(|d| d.code == "W009"));
    }

    #[test]
    fn test_fill_pattern_overrides_color() {
        let style = parse("{\"version\":8,\"sources\":{\"s\":{\"type\":\"vector\",\"url\":\"mapbox://x\"}},\"layers\":[{\"id\":\"l\",\"type\":\"fill\",\"source\":\"s\",\"source-layer\":\"x\",\"paint\":{\"fill-color\":\"#ff0\",\"fill-pattern\":\"dots\"}}]}");
        assert!(FillPatternOverridesColor
            .check(&style)
            .iter()
            .any(|d| d.code == "W016"));
    }

    #[test]
    fn test_fill_pattern_only_ok() {
        let style = parse("{\"version\":8,\"sources\":{\"s\":{\"type\":\"vector\",\"url\":\"mapbox://x\"}},\"layers\":[{\"id\":\"l\",\"type\":\"fill\",\"source\":\"s\",\"source-layer\":\"x\",\"paint\":{\"fill-pattern\":\"dots\"}}]}");
        assert!(FillPatternOverridesColor.check(&style).is_empty());
    }

    #[test]
    fn test_line_pattern_overrides_color() {
        let style = parse("{\"version\":8,\"sources\":{\"s\":{\"type\":\"vector\",\"url\":\"mapbox://x\"}},\"layers\":[{\"id\":\"l\",\"type\":\"line\",\"source\":\"s\",\"source-layer\":\"x\",\"paint\":{\"line-color\":\"#ff0\",\"line-pattern\":\"dash\"}}]}");
        assert!(LinePatternOverridesColor
            .check(&style)
            .iter()
            .any(|d| d.code == "W017"));
    }

    #[test]
    fn test_line_pattern_only_ok() {
        let style = parse("{\"version\":8,\"sources\":{\"s\":{\"type\":\"vector\",\"url\":\"mapbox://x\"}},\"layers\":[{\"id\":\"l\",\"type\":\"line\",\"source\":\"s\",\"source-layer\":\"x\",\"paint\":{\"line-pattern\":\"dash\"}}]}");
        assert!(LinePatternOverridesColor.check(&style).is_empty());
    }

    #[test]
    fn test_heatmap_missing_color() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://x"}},"layers":[{"id":"h","type":"heatmap","source":"s","source-layer":"x"}]}"#,
        );
        assert!(HeatmapMissingColor
            .check(&style)
            .iter()
            .any(|d| d.code == "W018"));
    }

    #[test]
    fn test_heatmap_with_color_ok() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://x"}},"layers":[{"id":"h","type":"heatmap","source":"s","source-layer":"x","paint":{"heatmap-color":["interpolate",["linear"],["heatmap-density"],0,"transparent",1,"red"]}}]}"#,
        );
        assert!(HeatmapMissingColor.check(&style).is_empty());
    }

    #[test]
    fn test_fix_empty_text_field() {
        let mut value = serde_json::json!({
            "version": 8,
            "sources": {},
            "layers": [
                {
                    "id": "s",
                    "type": "symbol",
                    "source": "x",
                    "layout": { "text-field": "" }
                }
            ]
        });
        EmptyTextField.fix(&mut value);
        assert!(value["layers"][0]["layout"].get("text-field").is_none());
    }

    #[test]
    fn test_fix_empty_text_field_is_fixable() {
        assert!(EmptyTextField.is_fixable());
    }

    #[test]
    fn test_fix_background_pattern_removes_color() {
        let mut value = serde_json::json!({
            "version": 8, "sources": {}, "layers": [{
                "id": "bg", "type": "background",
                "paint": { "background-color": "#fff", "background-pattern": "dots" }
            }]
        });
        BackgroundPatternOverridesColor.fix(&mut value);
        assert!(value["layers"][0]["paint"].get("background-color").is_none());
        assert!(value["layers"][0]["paint"].get("background-pattern").is_some());
    }

    #[test]
    fn test_fix_background_pattern_is_fixable() {
        assert!(BackgroundPatternOverridesColor.is_fixable());
    }

    #[test]
    fn test_fix_fill_pattern_removes_color() {
        let mut value = serde_json::json!({
            "version": 8,
            "sources": { "s": { "type": "vector", "url": "mapbox://x" } },
            "layers": [{
                "id": "l", "type": "fill", "source": "s", "source-layer": "x",
                "paint": { "fill-color": "#ff0", "fill-pattern": "dots" }
            }]
        });
        FillPatternOverridesColor.fix(&mut value);
        assert!(value["layers"][0]["paint"].get("fill-color").is_none());
        assert!(value["layers"][0]["paint"].get("fill-pattern").is_some());
    }

    #[test]
    fn test_fix_fill_pattern_is_fixable() {
        assert!(FillPatternOverridesColor.is_fixable());
    }

    #[test]
    fn test_fix_line_pattern_removes_color() {
        let mut value = serde_json::json!({
            "version": 8,
            "sources": { "s": { "type": "vector", "url": "mapbox://x" } },
            "layers": [{
                "id": "l", "type": "line", "source": "s", "source-layer": "x",
                "paint": { "line-color": "#ff0", "line-pattern": "dash" }
            }]
        });
        LinePatternOverridesColor.fix(&mut value);
        assert!(value["layers"][0]["paint"].get("line-color").is_none());
        assert!(value["layers"][0]["paint"].get("line-pattern").is_some());
    }

    #[test]
    fn test_fix_line_pattern_is_fixable() {
        assert!(LinePatternOverridesColor.is_fixable());
    }
}
