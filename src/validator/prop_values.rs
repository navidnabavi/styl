use serde_json::Value;
use crate::diagnostic::Diagnostic;
use crate::style::expression::validate_expression;

#[derive(Debug)]
pub enum PropType {
    Number { min: f64, max: f64 },
    NumberMin { min: f64 },
    Color,
    Boolean,
    Enum(&'static [&'static str]),
    StringType,
    Array,
    Any,
}

pub fn get_prop_type(name: &str) -> Option<PropType> {
    match name {
        // --- Shared ---
        "visibility" => Some(PropType::Enum(&["visible", "none"])),

        // --- Background paint ---
        "background-color"   => Some(PropType::Color),
        "background-opacity" => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "background-pattern" => Some(PropType::StringType),

        // --- Fill paint ---
        "fill-antialias"        => Some(PropType::Boolean),
        "fill-opacity"          => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "fill-color"            => Some(PropType::Color),
        "fill-outline-color"    => Some(PropType::Color),
        "fill-translate"        => Some(PropType::Array),
        "fill-translate-anchor" => Some(PropType::Enum(&["map", "viewport"])),
        "fill-pattern"          => Some(PropType::StringType),

        // --- Fill layout ---
        "fill-sort-key" => Some(PropType::Any),

        // --- Fill-extrusion paint ---
        "fill-extrusion-opacity"           => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "fill-extrusion-color"             => Some(PropType::Color),
        "fill-extrusion-translate"         => Some(PropType::Array),
        "fill-extrusion-translate-anchor"  => Some(PropType::Enum(&["map", "viewport"])),
        "fill-extrusion-pattern"           => Some(PropType::StringType),
        "fill-extrusion-height"            => Some(PropType::NumberMin { min: 0.0 }),
        "fill-extrusion-base"              => Some(PropType::NumberMin { min: 0.0 }),
        "fill-extrusion-vertical-gradient" => Some(PropType::Boolean),

        // --- Line layout ---
        "line-cap"         => Some(PropType::Enum(&["butt", "round", "square"])),
        "line-join"        => Some(PropType::Enum(&["bevel", "round", "miter"])),
        "line-miter-limit" => Some(PropType::NumberMin { min: 0.0 }),
        "line-round-limit" => Some(PropType::NumberMin { min: 0.0 }),
        "line-sort-key"    => Some(PropType::Any),

        // --- Line paint ---
        "line-opacity"          => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "line-color"            => Some(PropType::Color),
        "line-translate"        => Some(PropType::Array),
        "line-translate-anchor" => Some(PropType::Enum(&["map", "viewport"])),
        "line-width"            => Some(PropType::NumberMin { min: 0.0 }),
        "line-gap-width"        => Some(PropType::NumberMin { min: 0.0 }),
        "line-offset"           => Some(PropType::Any),
        "line-blur"             => Some(PropType::NumberMin { min: 0.0 }),
        "line-dasharray"        => Some(PropType::Array),
        "line-pattern"          => Some(PropType::StringType),
        "line-gradient"         => Some(PropType::Color),

        // --- Symbol layout ---
        "symbol-placement"   => Some(PropType::Enum(&["point", "line", "line-center"])),
        "symbol-spacing"     => Some(PropType::NumberMin { min: 1.0 }),
        "symbol-avoid-edges" => Some(PropType::Boolean),
        "symbol-sort-key"    => Some(PropType::Any),
        "symbol-z-order"     => Some(PropType::Enum(&["auto", "viewport-y", "source"])),

        "icon-allow-overlap"      => Some(PropType::Boolean),
        "icon-ignore-placement"   => Some(PropType::Boolean),
        "icon-optional"           => Some(PropType::Boolean),
        "icon-rotation-alignment" => Some(PropType::Enum(&["map", "viewport", "auto"])),
        "icon-size"               => Some(PropType::NumberMin { min: 0.0 }),
        "icon-text-fit"           => Some(PropType::Enum(&["none", "width", "height", "both"])),
        "icon-text-fit-padding"   => Some(PropType::Array),
        "icon-image"              => Some(PropType::StringType),
        "icon-rotate"             => Some(PropType::Any),
        "icon-padding"            => Some(PropType::NumberMin { min: 0.0 }),
        "icon-keep-upright"       => Some(PropType::Boolean),
        "icon-offset"             => Some(PropType::Array),
        "icon-anchor"             => Some(PropType::Enum(&["center", "left", "right", "top", "bottom", "top-left", "top-right", "bottom-left", "bottom-right"])),
        "icon-pitch-alignment"    => Some(PropType::Enum(&["map", "viewport", "auto"])),

        "text-pitch-alignment"    => Some(PropType::Enum(&["map", "viewport", "auto"])),
        "text-rotation-alignment" => Some(PropType::Enum(&["map", "viewport", "auto"])),
        "text-field"              => Some(PropType::Any),
        "text-font"               => Some(PropType::Array),
        "text-size"               => Some(PropType::NumberMin { min: 0.0 }),
        "text-max-width"          => Some(PropType::NumberMin { min: 0.0 }),
        "text-line-height"        => Some(PropType::Any),
        "text-letter-spacing"     => Some(PropType::Any),
        "text-justify"            => Some(PropType::Enum(&["auto", "left", "center", "right"])),
        "text-radial-offset"      => Some(PropType::Any),
        "text-variable-anchor"    => Some(PropType::Array),
        "text-anchor"             => Some(PropType::Enum(&["center", "left", "right", "top", "bottom", "top-left", "top-right", "bottom-left", "bottom-right"])),
        "text-max-angle"          => Some(PropType::Any),
        "text-writing-mode"       => Some(PropType::Array),
        "text-rotate"             => Some(PropType::Any),
        "text-padding"            => Some(PropType::NumberMin { min: 0.0 }),
        "text-keep-upright"       => Some(PropType::Boolean),
        "text-transform"          => Some(PropType::Enum(&["none", "uppercase", "lowercase"])),
        "text-offset"             => Some(PropType::Array),
        "text-allow-overlap"      => Some(PropType::Boolean),
        "text-ignore-placement"   => Some(PropType::Boolean),
        "text-optional"           => Some(PropType::Boolean),

        // --- Symbol paint ---
        "icon-opacity"          => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "icon-color"            => Some(PropType::Color),
        "icon-halo-color"       => Some(PropType::Color),
        "icon-halo-width"       => Some(PropType::NumberMin { min: 0.0 }),
        "icon-halo-blur"        => Some(PropType::NumberMin { min: 0.0 }),
        "icon-translate"        => Some(PropType::Array),
        "icon-translate-anchor" => Some(PropType::Enum(&["map", "viewport"])),
        "text-opacity"          => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "text-color"            => Some(PropType::Color),
        "text-halo-color"       => Some(PropType::Color),
        "text-halo-width"       => Some(PropType::NumberMin { min: 0.0 }),
        "text-halo-blur"        => Some(PropType::NumberMin { min: 0.0 }),
        "text-translate"        => Some(PropType::Array),
        "text-translate-anchor" => Some(PropType::Enum(&["map", "viewport"])),

        // --- Raster paint ---
        "raster-opacity"        => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "raster-hue-rotate"     => Some(PropType::Any),
        "raster-brightness-min" => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "raster-brightness-max" => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "raster-saturation"     => Some(PropType::Number { min: -1.0, max: 1.0 }),
        "raster-contrast"       => Some(PropType::Number { min: -1.0, max: 1.0 }),
        "raster-resampling"     => Some(PropType::Enum(&["linear", "nearest"])),
        "raster-fade-duration"  => Some(PropType::NumberMin { min: 0.0 }),

        // --- Circle layout ---
        "circle-sort-key" => Some(PropType::Any),

        // --- Circle paint ---
        "circle-radius"           => Some(PropType::NumberMin { min: 0.0 }),
        "circle-color"            => Some(PropType::Color),
        "circle-blur"             => Some(PropType::Any),
        "circle-opacity"          => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "circle-translate"        => Some(PropType::Array),
        "circle-translate-anchor" => Some(PropType::Enum(&["map", "viewport"])),
        "circle-pitch-scale"      => Some(PropType::Enum(&["map", "viewport"])),
        "circle-pitch-alignment"  => Some(PropType::Enum(&["map", "viewport"])),
        "circle-stroke-width"     => Some(PropType::NumberMin { min: 0.0 }),
        "circle-stroke-color"     => Some(PropType::Color),
        "circle-stroke-opacity"   => Some(PropType::Number { min: 0.0, max: 1.0 }),

        // --- Heatmap paint ---
        "heatmap-radius"    => Some(PropType::NumberMin { min: 1.0 }),
        "heatmap-weight"    => Some(PropType::NumberMin { min: 0.0 }),
        "heatmap-intensity" => Some(PropType::NumberMin { min: 0.0 }),
        "heatmap-color"     => Some(PropType::Color),
        "heatmap-opacity"   => Some(PropType::Number { min: 0.0, max: 1.0 }),

        // --- Hillshade paint ---
        "hillshade-illumination-direction" => Some(PropType::Number { min: 0.0, max: 359.0 }),
        "hillshade-illumination-anchor"    => Some(PropType::Enum(&["map", "viewport"])),
        "hillshade-exaggeration"           => Some(PropType::Number { min: 0.0, max: 1.0 }),
        "hillshade-shadow-color"           => Some(PropType::Color),
        "hillshade-highlight-color"        => Some(PropType::Color),
        "hillshade-accent-color"           => Some(PropType::Color),

        // --- Sky paint ---
        "sky-type"                     => Some(PropType::Enum(&["gradient", "atmosphere"])),
        "sky-atmosphere-sun"           => Some(PropType::Array),
        "sky-atmosphere-sun-intensity" => Some(PropType::NumberMin { min: 0.0 }),
        "sky-gradient-center"          => Some(PropType::Array),
        "sky-gradient-radius"          => Some(PropType::Number { min: 0.0, max: 180.0 }),
        "sky-gradient"                 => Some(PropType::Color),
        "sky-atmosphere-halo-color"    => Some(PropType::Color),
        "sky-atmosphere-color"         => Some(PropType::Color),
        "sky-opacity"                  => Some(PropType::Number { min: 0.0, max: 1.0 }),

        // --- Color-relief paint ---
        "color-relief-color-range" => Some(PropType::Array),
        "color-relief-opacity"     => Some(PropType::Number { min: 0.0, max: 1.0 }),

        _ => None,
    }
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub fn validate_prop_value(name: &str, value: &Value, path: &str) -> Vec<Diagnostic> {
    if let Some(arr) = value.as_array() {
        if arr.first().and_then(|v| v.as_str()).is_some() {
            return validate_expression(value, path, 0);
        }
    }

    let Some(prop_type) = get_prop_type(name) else {
        return vec![];
    };

    match prop_type {
        PropType::Any => vec![],
        PropType::Array => {
            if !value.is_array() {
                vec![Diagnostic::error("E018", path,
                    format!("\"{}\" expects an array, got {}", name, value_type_name(value)))]
            } else {
                vec![]
            }
        }
        PropType::Boolean => {
            if !value.is_boolean() {
                vec![Diagnostic::error("E018", path,
                    format!("\"{}\" expects boolean, got {}", name, value_type_name(value)))]
            } else {
                vec![]
            }
        }
        PropType::StringType => {
            if !value.is_string() {
                vec![Diagnostic::error("E018", path,
                    format!("\"{}\" expects a string, got {}", name, value_type_name(value)))]
            } else {
                vec![]
            }
        }
        PropType::Enum(variants) => {
            match value.as_str() {
                Some(s) if variants.contains(&s) => vec![],
                Some(s) => {
                    let quoted: Vec<String> = variants.iter().map(|v| format!("\"{}\"", v)).collect();
                    let listed = if quoted.len() == 1 {
                        quoted[0].clone()
                    } else if quoted.len() == 2 {
                        format!("{} or {}", quoted[0], quoted[1])
                    } else {
                        format!("{}, or {}", quoted[..quoted.len()-1].join(", "), quoted[quoted.len()-1])
                    };
                    vec![Diagnostic::error("E018", path,
                        format!("\"{}\" expects one of {}, got \"{}\"", name, listed, s))]
                }
                None => vec![Diagnostic::error("E018", path,
                    format!("\"{}\" expects a string, got {}", name, value_type_name(value)))],
            }
        }
        PropType::Number { min, max } => {
            match value.as_f64() {
                Some(n) if n >= min && n <= max => vec![],
                Some(n) => vec![Diagnostic::error("E018", path,
                    format!("\"{}\" expects number in [{}, {}], got {}", name, min, max, n))],
                None => vec![Diagnostic::error("E018", path,
                    format!("\"{}\" expects number in [{}, {}], got {}", name, min, max, value_type_name(value)))],
            }
        }
        PropType::NumberMin { min } => {
            match value.as_f64() {
                Some(n) if n >= min => vec![],
                Some(n) => vec![Diagnostic::error("E018", path,
                    format!("\"{}\" expects number >= {}, got {}", name, min, n))],
                None => vec![Diagnostic::error("E018", path,
                    format!("\"{}\" expects number >= {}, got {}", name, min, value_type_name(value)))],
            }
        }
        PropType::Color => {
            match value.as_str() {
                Some(s) => {
                    if csscolorparser::parse(s).is_err() {
                        vec![Diagnostic::error("E018", path,
                            format!("\"{}\" invalid color: \"{}\"", name, s))]
                    } else {
                        vec![]
                    }
                }
                None => vec![Diagnostic::error("E018", path,
                    format!("\"{}\" expects a CSS color string, got {}", name, value_type_name(value)))],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_opacity_in_range_valid() {
        let diags = validate_prop_value("fill-opacity", &json!(0.5), "layers[0].paint.fill-opacity");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_opacity_out_of_range() {
        let diags = validate_prop_value("fill-opacity", &json!(1.5), "layers[0].paint.fill-opacity");
        assert!(diags.iter().any(|d| d.code == "E018" && d.message.contains("fill-opacity")));
    }

    #[test]
    fn test_opacity_wrong_type() {
        let diags = validate_prop_value("fill-opacity", &json!("high"), "layers[0].paint.fill-opacity");
        assert!(diags.iter().any(|d| d.code == "E018"));
    }

    #[test]
    fn test_enum_valid() {
        let diags = validate_prop_value("line-cap", &json!("round"), "layers[0].layout.line-cap");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_enum_invalid() {
        let diags = validate_prop_value("line-cap", &json!("flat"), "layers[0].layout.line-cap");
        assert!(diags.iter().any(|d| d.code == "E018" && d.message.contains("\"butt\", \"round\", or \"square\"")));
    }

    #[test]
    fn test_color_valid() {
        let diags = validate_prop_value("fill-color", &json!("#ff0000"), "layers[0].paint.fill-color");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_color_invalid_string() {
        let diags = validate_prop_value("fill-color", &json!("notacolor"), "layers[0].paint.fill-color");
        assert!(diags.iter().any(|d| d.code == "E018" && d.message.contains("fill-color")));
    }

    #[test]
    fn test_color_wrong_type() {
        let diags = validate_prop_value("fill-color", &json!(42), "layers[0].paint.fill-color");
        assert!(diags.iter().any(|d| d.code == "E018"));
    }

    #[test]
    fn test_expression_skips_literal_validation() {
        let diags = validate_prop_value("fill-opacity", &json!(["get", "opacity"]), "layers[0].paint.fill-opacity");
        assert!(!diags.iter().any(|d| d.code == "E018"));
    }

    #[test]
    fn test_unknown_prop_no_diagnostic() {
        let diags = validate_prop_value("unknown-prop", &json!(42), "layers[0].paint.unknown-prop");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_number_min_valid() {
        let diags = validate_prop_value("line-width", &json!(5.0), "layers[0].paint.line-width");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_number_min_negative() {
        let diags = validate_prop_value("line-width", &json!(-1.0), "layers[0].paint.line-width");
        assert!(diags.iter().any(|d| d.code == "E018" && d.message.contains("line-width")));
    }

    #[test]
    fn test_boolean_valid() {
        let diags = validate_prop_value("fill-antialias", &json!(true), "layers[0].paint.fill-antialias");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_boolean_invalid() {
        let diags = validate_prop_value("fill-antialias", &json!("yes"), "layers[0].paint.fill-antialias");
        assert!(diags.iter().any(|d| d.code == "E018"));
    }

    #[test]
    fn test_visibility_valid() {
        let diags = validate_prop_value("visibility", &json!("visible"), "layers[0].layout.visibility");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_visibility_invalid() {
        let diags = validate_prop_value("visibility", &json!("hidden"), "layers[0].layout.visibility");
        assert!(diags.iter().any(|d| d.code == "E018" && d.message.contains("\"visible\" or \"none\"")));
    }
}
