use crate::diagnostic::Diagnostic;
use crate::style::{types::SpriteValue, Style};
use crate::validator::Validator;

pub struct RefsValidator;
impl Validator for RefsValidator {
    fn validate(&self, style: &crate::style::Style) -> Vec<crate::diagnostic::Diagnostic> {
        validate_refs(style)
    }
}

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

    // sprite: string must be non-empty; array entries must each have non-empty id and url
    match &style.sprite {
        Some(SpriteValue::String(s)) if s.is_empty() => {
            diags.push(Diagnostic::error(
                "E008",
                "sprite",
                "sprite must be a non-empty URL string",
            ));
        }
        Some(SpriteValue::Array(entries)) => {
            if entries.is_empty() {
                diags.push(Diagnostic::error(
                    "E008",
                    "sprite",
                    "sprite array must not be empty",
                ));
            }
            for (i, entry) in entries.iter().enumerate() {
                if entry.id.is_empty() {
                    diags.push(Diagnostic::error(
                        "E008",
                        format!("sprite[{}].id", i),
                        "sprite entry id must be non-empty",
                    ));
                }
                if entry.url.is_empty() {
                    diags.push(Diagnostic::error(
                        "E008",
                        format!("sprite[{}].url", i),
                        "sprite entry url must be non-empty",
                    ));
                }
            }
        }
        _ => {}
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
        let style = parse(
            r#"{
            "version": 8,
            "sources": {},
            "layers": [{"id": "roads", "type": "line", "source": "missing"}]
        }"#,
        );
        let diags = validate_refs(&style);
        assert!(diags.iter().any(|d| d.code == "E003"));
    }

    #[test]
    fn test_valid_source_ref() {
        let style = parse(
            r#"{
            "version":8,
            "sources":{"s":{"type":"geojson","data":"https://example.com/data.geojson"}},
            "layers":[{"id":"l","type":"fill","source":"s"}]
        }"#,
        );
        let diags = validate_refs(&style);
        assert!(!diags.iter().any(|d| d.code == "E003"));
    }

    #[test]
    fn test_empty_sprite() {
        let style = parse(r#"{"version":8,"sprite":"","sources":{},"layers":[]}"#);
        let diags = validate_refs(&style);
        assert!(diags.iter().any(|d| d.code == "E008"));
    }

    #[test]
    fn test_sprite_string_valid() {
        let style = parse(
            r#"{"version":8,"sprite":"https://example.com/sprite","sources":{},"layers":[]}"#,
        );
        assert!(validate_refs(&style).iter().all(|d| d.code != "E008"));
    }

    #[test]
    fn test_sprite_array_valid() {
        let style = parse(
            r#"{"version":8,"sprite":[{"id":"default","url":"https://example.com/sprite"}],"sources":{},"layers":[]}"#,
        );
        assert!(validate_refs(&style).iter().all(|d| d.code != "E008"));
    }

    #[test]
    fn test_sprite_array_empty() {
        let style = parse(r#"{"version":8,"sprite":[],"sources":{},"layers":[]}"#);
        let diags = validate_refs(&style);
        assert!(diags.iter().any(|d| d.code == "E008"));
    }

    #[test]
    fn test_sprite_array_empty_id() {
        let style = parse(
            r#"{"version":8,"sprite":[{"id":"","url":"https://example.com/sprite"}],"sources":{},"layers":[]}"#,
        );
        let diags = validate_refs(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E008" && d.path.contains("sprite[0].id")));
    }
}
