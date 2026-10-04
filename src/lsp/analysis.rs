//! Turning a text buffer into LSP diagnostics and edits.

use std::path::PathBuf;

use lsp_types::{
    CodeDescription, Diagnostic as LspDiagnostic, DiagnosticSeverity, NumberOrString,
    Position as LspPosition, Range as LspRange, TextEdit, Uri,
};

use super::config_cache::ConfigCache;
use crate::cli::Spec;
use crate::diagnostic::{Diagnostic, Severity, TextRange};
use crate::linter::config::Config;
use crate::span::{self, SourceMap};
use crate::{formatter, linter, style::parse_style, style::ShapeError, validator};

const SOURCE: &str = "styl";
const DOCS_BASE: &str = "https://github.com/navidnabavi/styl/blob/main/docs";

/// Whether a buffer looks like a GL style document.
///
/// Deliberately content-based. A filename test would misfire on the `styles.json`
/// design-token files common in web projects, and attaching to every `.json` file
/// would put squiggles on `package.json`. Callers make the result sticky per
/// document so a style stays recognized while it is mid-edit and unparseable.
pub fn looks_like_style(text: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    object.get("version").and_then(serde_json::Value::as_u64) == Some(8)
        && (object.contains_key("layers") || object.contains_key("sources"))
}

/// Run the validators and linter over `text` and convert the result to LSP.
///
/// `spec_override` comes from editor settings and wins over `.stylrc`.
pub fn diagnose(
    text: &str,
    uri: &Uri,
    spec_override: Option<Spec>,
    configs: &mut ConfigCache,
) -> Vec<LspDiagnostic> {
    // Check that it is JSON at all first, so a syntax error is reported as one
    // rather than as a schema complaint about whatever serde reached first.
    if let Err(error) = serde_json::from_str::<serde_json::Value>(text) {
        return vec![syntax_diagnostic(&error, text)];
    }
    let style = match parse_style(text) {
        Ok(style) => style,
        Err(error) => return vec![shape_diagnostic(&error, text)],
    };

    let config = resolve_config(uri, configs).cloned().unwrap_or_default();
    let spec = spec_override
        .or_else(|| config.resolved_spec())
        .unwrap_or(Spec::Both);

    let mut diagnostics = validator::run_all(&style, &spec);
    diagnostics.extend(linter::run_all(&style, &spec));
    config.apply_severity(&mut diagnostics);
    span::resolve_ranges(&mut diagnostics, &SourceMap::parse(text));

    diagnostics.iter().map(to_lsp).collect()
}

/// Format the whole document. `None` when the buffer is not valid JSON, or when
/// it is already formatted.
pub fn format(
    text: &str,
    uri: &Uri,
    tab_size: Option<u32>,
    configs: &mut ConfigCache,
) -> Option<Vec<TextEdit>> {
    let value = serde_json::from_str::<serde_json::Value>(text).ok()?;

    // A project carrying a `.stylrc` has opted into styl's own formatting, so it
    // outranks the editor's tab size. `FormatConfig::indent` cannot distinguish
    // "set to 2" from "unset", which is why this keys off the file existing.
    let indent = match (resolve_config(uri, configs), tab_size) {
        (Some(config), _) => config.format.indent,
        (None, Some(size)) => size as usize,
        (None, None) => 2,
    };

    let formatted = formatter::format_style(&value, indent);
    if formatted == text {
        return Some(Vec::new());
    }

    let map = SourceMap::parse(text);
    let end = map.position(text.len());
    Some(vec![TextEdit {
        range: LspRange {
            start: LspPosition {
                line: 0,
                character: 0,
            },
            end: LspPosition {
                line: end.line,
                character: end.character,
            },
        },
        new_text: formatted,
    }])
}

/// The `.stylrc` governing `uri`, if there is one.
fn resolve_config<'a>(uri: &Uri, configs: &'a mut ConfigCache) -> Option<&'a Config> {
    let path = uri_to_path(uri)?;
    let directory = path.parent()?.to_path_buf();
    configs.get(&directory)
}

fn to_lsp(diagnostic: &Diagnostic) -> LspDiagnostic {
    LspDiagnostic {
        range: diagnostic.range.map(to_lsp_range).unwrap_or(LspRange {
            start: LspPosition {
                line: 0,
                character: 0,
            },
            end: LspPosition {
                line: 0,
                character: 0,
            },
        }),
        severity: Some(match diagnostic.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
        }),
        code: Some(NumberOrString::String(diagnostic.code.to_string())),
        code_description: doc_href(diagnostic.code).map(|href| CodeDescription { href }),
        source: Some(SOURCE.to_string()),
        message: match &diagnostic.hint {
            Some(hint) => format!("{}\n\nhint: {}", diagnostic.message, hint),
            None => diagnostic.message.clone(),
        },
        related_information: None,
        tags: None,
        data: None,
    }
}

fn to_lsp_range(range: TextRange) -> LspRange {
    LspRange {
        start: LspPosition {
            line: range.start.line,
            character: range.start.character,
        },
        end: LspPosition {
            line: range.end.line,
            character: range.end.character,
        },
    }
}

/// Link each code to its documentation section, which clients surface as a
/// "view rule" affordance next to the diagnostic.
fn doc_href(code: &str) -> Option<Uri> {
    let page = match code.as_bytes().first()? {
        b'E' => "validators.md",
        b'W' => "linter.md",
        _ => return None,
    };
    format!("{}/{}#{}", DOCS_BASE, page, code.to_ascii_lowercase())
        .parse()
        .ok()
}

/// The buffer is not valid JSON. Common while typing, so the position matters.
fn syntax_diagnostic(error: &serde_json::Error, text: &str) -> LspDiagnostic {
    // serde_json reports a 1-based line and a 1-based *byte* column, and 0 for
    // both when it has no position. Route through `SourceMap` so the column is
    // converted to the UTF-16 units LSP expects.
    let map = SourceMap::parse(text);
    let line = error.line().saturating_sub(1);
    let column = error.column().saturating_sub(1);
    let position = match map.line_start(line) {
        Some(start) => map.position(start + column),
        None => crate::diagnostic::Position {
            line: 0,
            character: 0,
        },
    };

    LspDiagnostic {
        range: LspRange {
            start: LspPosition {
                line: position.line,
                character: position.character,
            },
            end: LspPosition {
                line: position.line,
                character: position.character,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("syntax".to_string())),
        code_description: None,
        source: Some(SOURCE.to_string()),
        message: format!("invalid JSON: {}", error),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// The JSON is valid but does not match the style schema.
fn shape_diagnostic(error: &ShapeError, text: &str) -> LspDiagnostic {
    // The field path comes from `serde_path_to_error` in exactly the syntax the
    // rules use, so it resolves against the source map like any other
    // diagnostic. A root-level fault has no path and lands on the document.
    let range = SourceMap::parse(text)
        .range_for_path(&error.path)
        .map(to_lsp_range)
        .unwrap_or(LspRange {
            start: LspPosition {
                line: 0,
                character: 0,
            },
            end: LspPosition {
                line: 0,
                character: 0,
            },
        });

    LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("shape".to_string())),
        code_description: None,
        source: Some(SOURCE.to_string()),
        message: format!("style does not match the schema: {}", error.message),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// A `file:` URI as a local path. `None` for any other scheme, so an unsaved or
/// remote buffer simply gets no `.stylrc`.
fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    // Local file URIs have an empty authority, so what follows is the path.
    let path = percent_decode(uri.as_str().strip_prefix("file://")?);

    // `file:///C:/x` yields `/C:/x` on Windows; drop the leading separator.
    #[cfg(windows)]
    let path = match path.strip_prefix('/') {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => rest.to_string(),
        _ => path,
    };

    Some(PathBuf::from(path))
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push(high * 16 + low);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(raw: &str) -> Uri {
        raw.parse().expect("valid uri")
    }

    #[test]
    fn detects_a_style_by_content() {
        assert!(looks_like_style(r#"{"version": 8, "layers": []}"#));
        assert!(looks_like_style(r#"{"version": 8, "sources": {}}"#));
    }

    #[test]
    fn does_not_claim_unrelated_json() {
        // The files this would otherwise squiggle.
        assert!(!looks_like_style(r#"{"name": "app", "version": "1.0.0"}"#));
        assert!(!looks_like_style(
            r#"{"compilerOptions": {"strict": true}}"#
        ));
        // Right shape, wrong spec version.
        assert!(!looks_like_style(r#"{"version": 7, "layers": []}"#));
        // Version 8 alone is not enough.
        assert!(!looks_like_style(r#"{"version": 8}"#));
        assert!(!looks_like_style("[]"));
        assert!(!looks_like_style("{"));
    }

    #[test]
    fn reports_syntax_errors_with_a_position() {
        let diagnostics = diagnose(
            "{\n  \"version\": 8,\n  oops\n}",
            &uri("file:///s.json"),
            None,
            &mut ConfigCache::default(),
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start.line, 2);
        assert!(diagnostics[0].message.starts_with("invalid JSON"));
    }

    #[test]
    fn reports_rule_diagnostics_with_ranges_and_doc_links() {
        let text = "{\n  \"version\": 8,\n  \"sources\": {},\n  \"layers\": [\n    {\n      \"id\": \"a\",\n      \"type\": \"line\",\n      \"source\": \"nope\"\n    }\n  ]\n}";
        let diagnostics = diagnose(
            text,
            &uri("file:///s.json"),
            None,
            &mut ConfigCache::default(),
        );

        let missing = diagnostics
            .iter()
            .find(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "E009"))
            .expect("E009 for the undefined source");
        assert_eq!(missing.range.start.line, 7);
        assert_eq!(missing.severity, Some(DiagnosticSeverity::ERROR));
        assert!(missing.code_description.is_some());
    }

    #[test]
    fn shape_errors_point_at_the_offending_field() {
        let text = "{\n  \"version\": 8,\n  \"sources\": {},\n  \"layers\": [\n    { \"id\": \"a\", \"type\": \"fill\", \"minzoom\": \"five\" }\n  ]\n}";
        let diagnostics = diagnose(
            text,
            &uri("file:///s.json"),
            None,
            &mut ConfigCache::default(),
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            Some(NumberOrString::String("shape".to_string()))
        );
        // The bad `minzoom` is on line 5, not line 1.
        assert_eq!(diagnostics[0].range.start.line, 4);
        assert!(diagnostics[0].message.contains("expected f64"));
    }

    #[test]
    fn format_returns_no_edit_for_already_formatted_text() {
        let text = "{\n  \"version\": 8,\n  \"sources\": {},\n  \"layers\": []\n}\n";
        let edits = format(
            text,
            &uri("file:///s.json"),
            Some(2),
            &mut ConfigCache::default(),
        )
        .expect("formattable");
        assert!(edits.is_empty(), "got: {:?}", edits);
    }

    #[test]
    fn format_rewrites_the_whole_document() {
        let edits = format(
            r#"{"layers":[],"version":8,"sources":{}}"#,
            &uri("file:///s.json"),
            Some(2),
            &mut ConfigCache::default(),
        )
        .expect("formattable");
        assert_eq!(edits.len(), 1);
        // Key order is canonicalized, so `version` leads.
        assert!(edits[0].new_text.starts_with("{\n  \"version\": 8"));
    }

    #[test]
    fn format_declines_invalid_json() {
        assert!(format(
            "{not json",
            &uri("file:///s.json"),
            Some(2),
            &mut ConfigCache::default(),
        )
        .is_none());
    }

    #[test]
    fn decodes_file_uris() {
        assert_eq!(
            uri_to_path(&uri("file:///tmp/my%20styles/style.json")),
            Some(PathBuf::from("/tmp/my styles/style.json"))
        );
        // Non-file schemes have no path, and so no `.stylrc`.
        assert_eq!(uri_to_path(&uri("untitled:Untitled-1")), None);
    }
}
