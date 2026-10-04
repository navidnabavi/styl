//! Deserializing a `Style` from text, keeping track of *where* it went wrong.
//!
//! `serde_json::from_value` discards all position information, so a type error
//! anywhere in the document surfaced as a bare `invalid type: string "five",
//! expected f64` with no indication of which field. In a style with hundreds of
//! layers that is close to unusable.
//!
//! Deserializing through `serde_path_to_error` recovers the field path, which
//! happens to use exactly the syntax diagnostic paths already use — so it feeds
//! straight into [`crate::span::SourceMap`] for a precise range.

use super::Style;

/// A document that parsed as JSON but does not match the style schema.
#[derive(Debug, Clone)]
pub struct ShapeError {
    /// Diagnostic path of the offending field, e.g. `layers[0].minzoom`. Empty
    /// when the fault is the document root itself.
    pub path: String,
    /// What serde objected to.
    pub message: String,
}

impl std::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "at {}: {}", self.path, self.message)
        }
    }
}

/// Deserialize a style from JSON text, reporting the path of any type error.
pub fn parse_style(text: &str) -> Result<Style, ShapeError> {
    let deserializer = &mut serde_json::Deserializer::from_str(text);
    serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let path = error.path().to_string();
        ShapeError {
            // serde_path_to_error renders a root-level fault as ".", which is
            // not a path any rule would emit.
            path: if path == "." { String::new() } else { path },
            message: strip_serde_position(&error.inner().to_string()),
        }
    })
}

/// Drop the ` at line N column M` serde appends.
///
/// Callers render their own location from the field path, which points at the
/// key rather than at the byte serde happened to stop on. Two positions in one
/// message, disagreeing slightly, is worse than one.
fn strip_serde_position(message: &str) -> String {
    match message.rfind(" at line ") {
        Some(cut) if message[cut..].contains(" column ") => message[..cut].to_string(),
        _ => message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_the_field_path_for_a_type_error() {
        let error = parse_style(
            r#"{"version":8,"sources":{},"layers":[{"id":"a","type":"fill","minzoom":"five"}]}"#,
        )
        .expect_err("should not deserialize");
        assert_eq!(error.path, "layers[0].minzoom");
        assert!(error.message.contains("invalid type"));
    }

    #[test]
    fn path_syntax_matches_what_rules_emit() {
        // The whole point: the path must resolve against a `SourceMap`.
        let text = "{\n  \"version\": 8,\n  \"sources\": {},\n  \"layers\": [\n    { \"id\": \"a\", \"type\": \"fill\", \"minzoom\": \"five\" }\n  ]\n}";
        let error = parse_style(text).expect_err("should not deserialize");
        let map = crate::span::SourceMap::parse(text);
        let range = map
            .range_for_path(&error.path)
            .expect("path resolves against the source map");
        assert_eq!(range.start.line, 4);
    }

    #[test]
    fn reports_root_level_faults_without_a_path() {
        let error = parse_style("[]").expect_err("an array is not a style");
        assert!(error.path.is_empty(), "got {:?}", error.path);
    }

    #[test]
    fn strips_serdes_own_position_from_the_message() {
        let error = parse_style(
            r#"{"version":8,"sources":{},"layers":[{"id":"a","type":"fill","minzoom":"five"}]}"#,
        )
        .expect_err("should not deserialize");
        assert_eq!(error.message, "invalid type: string \"five\", expected f64");
    }

    #[test]
    fn accepts_a_valid_style() {
        let style = parse_style(r#"{"version":8,"sources":{},"layers":[]}"#).expect("valid");
        assert_eq!(style.version, 8);
    }
}
