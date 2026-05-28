use crate::diagnostic::Diagnostic;
use crate::style::Style;
use crate::validator::Validator;

pub struct RootValidator;
impl Validator for RootValidator {
    fn validate(&self, style: &crate::style::Style) -> Vec<crate::diagnostic::Diagnostic> {
        validate_root(style)
    }
}

pub fn validate_root(style: &Style) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // version must be 8
    if style.version != 8 {
        diags.push(
            Diagnostic::error(
                "E001",
                "version",
                format!("version must be 8, got {}", style.version),
            )
            .with_hint("set \"version\": 8 at the root of your style"),
        );
    }

    // center: [lng, lat] bounds
    if let Some(center) = &style.center {
        let lng = center[0];
        let lat = center[1];
        if !(-180.0..=180.0).contains(&lng) {
            diags.push(Diagnostic::error(
                "E002",
                "center[0]",
                format!("longitude {} is out of range [-180, 180]", lng),
            ));
        }
        if !(-90.0..=90.0).contains(&lat) {
            diags.push(Diagnostic::error(
                "E002",
                "center[1]",
                format!("latitude {} is out of range [-90, 90]", lat),
            ));
        }
    }

    // zoom: 0..=24
    if let Some(zoom) = style.zoom {
        if !(0.0..=24.0).contains(&zoom) {
            diags.push(Diagnostic::error(
                "E002",
                "zoom",
                format!("zoom {} is out of range [0, 24]", zoom),
            ));
        }
    }

    // bearing: spec normalizes automatically; no strict range check needed

    // pitch: 0..=85
    if let Some(pitch) = style.pitch {
        if !(0.0..=85.0).contains(&pitch) {
            diags.push(Diagnostic::error(
                "E002",
                "pitch",
                format!("pitch {} is out of range [0, 85]", pitch),
            ));
        }
    }

    // glyphs must contain {fontstack} and {range}
    if let Some(glyphs) = &style.glyphs {
        if !glyphs.contains("{fontstack}") {
            diags.push(
                Diagnostic::error(
                    "E003",
                    "glyphs",
                    "glyphs URL must contain {fontstack} placeholder",
                )
                .with_hint("example: \"https://example.com/fonts/{fontstack}/{range}.pbf\""),
            );
        }
        if !glyphs.contains("{range}") {
            diags.push(
                Diagnostic::error(
                    "E003",
                    "glyphs",
                    "glyphs URL must contain {range} placeholder",
                )
                .with_hint("example: \"https://example.com/fonts/{fontstack}/{range}.pbf\""),
            );
        }
    }

    // E025: terrain.source is required; exaggeration must be >= 0
    if let Some(terrain) = &style.terrain {
        if let Some(obj) = terrain.as_object() {
            match obj.get("source") {
                None => {
                    diags.push(
                        Diagnostic::error(
                            "E025",
                            "terrain.source",
                            "terrain must have a \"source\" referencing a raster-dem source",
                        )
                        .with_hint(
                            "add \"source\": \"<raster-dem-source-id>\" to the terrain object",
                        ),
                    );
                }
                Some(src) if !src.is_string() => {
                    diags.push(Diagnostic::error(
                        "E025",
                        "terrain.source",
                        "terrain.source must be a string",
                    ));
                }
                _ => {}
            }
            if let Some(exag) = obj.get("exaggeration") {
                // expressions (arrays starting with a string) are valid — only validate literals
                let is_expr = exag
                    .as_array()
                    .and_then(|a| a.first())
                    .map(|v| v.is_string())
                    .unwrap_or(false);
                if !is_expr {
                    match exag.as_f64() {
                        Some(n) if n < 0.0 => {
                            diags.push(Diagnostic::error(
                                "E025",
                                "terrain.exaggeration",
                                format!("terrain.exaggeration must be >= 0, got {}", n),
                            ));
                        }
                        None => {
                            diags.push(Diagnostic::error(
                                "E025",
                                "terrain.exaggeration",
                                "terrain.exaggeration must be a number",
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // E027: light property validation
    if let Some(light) = &style.light {
        if let Some(obj) = light.as_object() {
            if let Some(anchor) = obj.get("anchor") {
                match anchor.as_str() {
                    Some("map") | Some("viewport") => {}
                    Some(s) => {
                        diags.push(Diagnostic::error(
                            "E027",
                            "light.anchor",
                            format!(
                                "light.anchor must be \"map\" or \"viewport\", got \"{}\"",
                                s
                            ),
                        ));
                    }
                    None => {
                        diags.push(Diagnostic::error(
                            "E027",
                            "light.anchor",
                            "light.anchor must be a string",
                        ));
                    }
                }
            }
            if let Some(intensity) = obj.get("intensity") {
                let is_expr = intensity
                    .as_array()
                    .and_then(|a| a.first())
                    .map(|v| v.is_string())
                    .unwrap_or(false);
                if !is_expr {
                    match intensity.as_f64() {
                        Some(n) if !(0.0..=1.0).contains(&n) => {
                            diags.push(Diagnostic::error(
                                "E027",
                                "light.intensity",
                                format!("light.intensity must be in [0, 1], got {}", n),
                            ));
                        }
                        None => {
                            diags.push(Diagnostic::error(
                                "E027",
                                "light.intensity",
                                "light.intensity must be a number",
                            ));
                        }
                        _ => {}
                    }
                }
            }
            if let Some(color) = obj.get("color") {
                if let Some(s) = color.as_str() {
                    if csscolorparser::parse(s).is_err() {
                        diags.push(Diagnostic::error(
                            "E027",
                            "light.color",
                            format!("light.color invalid color: \"{}\"", s),
                        ));
                    }
                } else if !color.is_array() {
                    diags.push(Diagnostic::error(
                        "E027",
                        "light.color",
                        "light.color must be a CSS color string",
                    ));
                }
            }
            if let Some(position) = obj.get("position") {
                let is_expr = position
                    .as_array()
                    .and_then(|a| a.first())
                    .map(|v| v.is_string())
                    .unwrap_or(false);
                if !is_expr {
                    match position.as_array() {
                        Some(arr) if arr.len() != 3 => {
                            diags.push(Diagnostic::error(
                                "E027",
                                "light.position",
                                format!(
                                    "light.position must have 3 elements [radial, azimuthal, polar], got {}",
                                    arr.len()
                                ),
                            ));
                        }
                        Some(arr) => {
                            for (i, v) in arr.iter().enumerate() {
                                if !v.is_number() {
                                    diags.push(Diagnostic::error(
                                        "E027",
                                        format!("light.position[{}]", i),
                                        format!("light.position[{}] must be a number", i),
                                    ));
                                }
                            }
                        }
                        None => {
                            diags.push(Diagnostic::error(
                                "E027",
                                "light.position",
                                "light.position must be an array of 3 numbers",
                            ));
                        }
                    }
                }
            }
        }
    }

    // E028: transition.duration and transition.delay must be >= 0
    if let Some(transition) = &style.transition {
        if let Some(obj) = transition.as_object() {
            for field in &["duration", "delay"] {
                if let Some(v) = obj.get(*field) {
                    let is_expr = v
                        .as_array()
                        .and_then(|a| a.first())
                        .map(|f| f.is_string())
                        .unwrap_or(false);
                    if !is_expr {
                        match v.as_f64() {
                            Some(n) if n < 0.0 => {
                                diags.push(Diagnostic::error(
                                    "E028",
                                    format!("transition.{}", field),
                                    format!("transition.{} must be >= 0 (ms), got {}", field, n),
                                ));
                            }
                            None => {
                                diags.push(Diagnostic::error(
                                    "E028",
                                    format!("transition.{}", field),
                                    format!("transition.{} must be a number (ms)", field),
                                ));
                            }
                            _ => {}
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
    use crate::style::Style;

    fn parse(json: &str) -> Style {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_valid_root() {
        let style = parse(r#"{"version":8,"sources":{},"layers":[]}"#);
        assert!(validate_root(&style).is_empty());
    }

    #[test]
    fn test_wrong_version() {
        let style = parse(r#"{"version":7,"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags.iter().any(|d| d.code == "E001"));
    }

    #[test]
    fn test_invalid_center_lng() {
        let style = parse(r#"{"version":8,"center":[200,0],"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E002" && d.path.contains("center[0]")));
    }

    #[test]
    fn test_invalid_center_lat() {
        let style = parse(r#"{"version":8,"center":[0,100],"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E002" && d.path.contains("center[1]")));
    }

    #[test]
    fn test_invalid_zoom() {
        let style = parse(r#"{"version":8,"zoom":30,"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags.iter().any(|d| d.code == "E002" && d.path == "zoom"));
    }

    #[test]
    fn test_negative_bearing_accepted() {
        // Spec normalizes bearing; -90 is equivalent to 270 and must not error
        let style = parse(r#"{"version":8,"bearing":-90,"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(!diags.iter().any(|d| d.path == "bearing"));
    }

    #[test]
    fn test_invalid_pitch() {
        let style = parse(r#"{"version":8,"pitch":90,"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags.iter().any(|d| d.code == "E002" && d.path == "pitch"));
    }

    #[test]
    fn test_glyphs_missing_placeholders() {
        let style =
            parse(r#"{"version":8,"glyphs":"https://example.com/fonts","sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags.iter().any(|d| d.code == "E003" && d.path == "glyphs"));
    }

    #[test]
    fn test_glyphs_valid() {
        let style = parse(
            r#"{"version":8,"glyphs":"https://example.com/{fontstack}/{range}.pbf","sources":{},"layers":[]}"#,
        );
        let diags = validate_root(&style);
        assert!(diags.iter().filter(|d| d.path == "glyphs").count() == 0);
    }

    #[test]
    fn test_terrain_missing_source_e025() {
        let style =
            parse(r#"{"version":8,"terrain":{"exaggeration":1.5},"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E025" && d.path.contains("source")));
    }

    #[test]
    fn test_terrain_valid() {
        let style = parse(
            r#"{"version":8,"terrain":{"source":"dem","exaggeration":1.5},"sources":{},"layers":[]}"#,
        );
        assert!(validate_root(&style).iter().all(|d| d.code != "E025"));
    }

    #[test]
    fn test_terrain_negative_exaggeration_e025() {
        let style = parse(
            r#"{"version":8,"terrain":{"source":"dem","exaggeration":-1},"sources":{},"layers":[]}"#,
        );
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E025" && d.path.contains("exaggeration")));
    }

    #[test]
    fn test_light_invalid_anchor_e027() {
        let style = parse(r#"{"version":8,"light":{"anchor":"sky"},"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E027" && d.path == "light.anchor"));
    }

    #[test]
    fn test_light_valid() {
        let style = parse(
            r#"{"version":8,"light":{"anchor":"map","intensity":0.5,"color":"white","position":[1.15,210,30]},"sources":{},"layers":[]}"#,
        );
        assert!(validate_root(&style).iter().all(|d| d.code != "E027"));
    }

    #[test]
    fn test_light_intensity_out_of_range_e027() {
        let style = parse(r#"{"version":8,"light":{"intensity":1.5},"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E027" && d.path == "light.intensity"));
    }

    #[test]
    fn test_light_invalid_color_e027() {
        let style =
            parse(r#"{"version":8,"light":{"color":"notacolor"},"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E027" && d.path == "light.color"));
    }

    #[test]
    fn test_light_position_wrong_length_e027() {
        let style = parse(r#"{"version":8,"light":{"position":[1,2]},"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E027" && d.path == "light.position"));
    }

    #[test]
    fn test_transition_negative_duration_e028() {
        let style =
            parse(r#"{"version":8,"transition":{"duration":-100},"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E028" && d.path == "transition.duration"));
    }

    #[test]
    fn test_transition_valid() {
        let style = parse(
            r#"{"version":8,"transition":{"duration":300,"delay":0},"sources":{},"layers":[]}"#,
        );
        assert!(validate_root(&style).iter().all(|d| d.code != "E028"));
    }

    #[test]
    fn test_transition_negative_delay_e028() {
        let style = parse(r#"{"version":8,"transition":{"delay":-50},"sources":{},"layers":[]}"#);
        let diags = validate_root(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E028" && d.path == "transition.delay"));
    }
}
