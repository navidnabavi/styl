use crate::diagnostic::Diagnostic;
use crate::style::Style;

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
}
