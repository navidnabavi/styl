use crate::diagnostic::Diagnostic;
use crate::style::Style;

pub fn validate_refs(style: &Style) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (i, layer) in style.layers.iter().enumerate() {
        let path = format!("layers[{}]", i);

        // layer.source must reference a key in sources
        if let Some(source_id) = &layer.source {
            if !style.sources.contains_key(source_id) {
                diags.push(
                    Diagnostic::error(
                        "E003",
                        format!("{}.source", path),
                        format!("source \"{}\" is not defined in sources", source_id),
                    )
                    .with_hint(format!(
                        "add \"{}\" to the top-level sources object, or correct the layer source reference",
                        source_id
                    )),
                );
            }
        }
    }

    // sprite must be a non-empty string (format validated only)
    if let Some(sprite) = &style.sprite {
        if sprite.is_empty() {
            diags.push(Diagnostic::error("E008", "sprite", "sprite must be a non-empty URL string"));
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
    fn test_missing_source_ref() {
        let style = parse(r#"{
            "version": 8,
            "sources": {},
            "layers": [{"id": "roads", "type": "line", "source": "missing"}]
        }"#);
        let diags = validate_refs(&style);
        assert!(diags.iter().any(|d| d.code == "E003"));
    }

    #[test]
    fn test_valid_source_ref() {
        let style = parse(r#"{
            "version":8,
            "sources":{"s":{"type":"geojson","data":null}},
            "layers":[{"id":"l","type":"fill","source":"s","source-layer":"water"}]
        }"#);
        let diags = validate_refs(&style);
        assert!(!diags.iter().any(|d| d.code == "E003"));
    }

    #[test]
    fn test_empty_sprite() {
        let style = parse(r#"{"version":8,"sprite":"","sources":{},"layers":[]}"#);
        let diags = validate_refs(&style);
        assert!(diags.iter().any(|d| d.code == "E008"));
    }
}
