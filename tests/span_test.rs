use std::path::Path;

use styl::span::SourceMap;
use styl::*;

const SAMPLE: &str = r##"{
  "version": 8,
  "sources": {
    "openmaptiles.v3": {
      "type": "vector",
      "bounds": [-180, -90, 180, 90]
    }
  },
  "layers": [
    {
      "id": "roads",
      "type": "line",
      "paint": { "line-color": "#fff" }
    }
  ]
}"##;

/// One-based line of a path's resolved range, for terser assertions.
fn line_of(map: &SourceMap, path: &str) -> u32 {
    map.range_for_path(path)
        .unwrap_or_else(|| panic!("path did not resolve: {}", path))
        .start
        .line
        + 1
}

#[test]
fn resolves_every_path_shape() {
    let map = SourceMap::parse(SAMPLE);

    assert_eq!(line_of(&map, "version"), 2);
    assert_eq!(line_of(&map, "sources"), 3);
    assert_eq!(line_of(&map, "sources.openmaptiles.v3.type"), 5);
    assert_eq!(line_of(&map, "sources.openmaptiles.v3.bounds"), 6);
    assert_eq!(line_of(&map, "sources.openmaptiles.v3.bounds[2]"), 6);
    assert_eq!(line_of(&map, "layers[0].id"), 11);
    assert_eq!(line_of(&map, "layers[0].paint.line-color"), 13);
}

/// A source id containing the path delimiter must still resolve, because both the
/// validators and the scanner build the key by plain concatenation.
#[test]
fn resolves_dotted_source_id() {
    let map = SourceMap::parse(SAMPLE);
    let span = map
        .resolve("sources.openmaptiles.v3")
        .expect("dotted source id should resolve exactly");
    assert_eq!(map.position(span.key.start).line + 1, 4);
}

#[test]
fn range_covers_key_through_value() {
    let map = SourceMap::parse(SAMPLE);
    let range = map.range_for_path("layers[0].id").unwrap();
    // `      "id": "roads",` -> opening quote of the key through the closing
    // quote of the value, so the squiggle covers the whole member.
    assert_eq!(range.start.character, 6);
    assert_eq!(range.end.character, 19);
    assert_eq!(range.start.line, range.end.line);
}

/// E004 reports `layers[N].source` for a key that is not in the text at all.
#[test]
fn missing_key_falls_back_to_enclosing_object() {
    let text = "{\n  \"layers\": [\n    {\n      \"id\": \"roads\"\n    }\n  ]\n}";
    let map = SourceMap::parse(text);

    assert!(map.resolve("layers[0].source").is_none());
    // Falls back to the layer object, which opens on line 3.
    assert_eq!(line_of(&map, "layers[0].source"), 3);
    // And an entirely unknown root key falls back to the document.
    assert_eq!(line_of(&map, "nonexistent.deeply.nested"), 1);
}

#[test]
fn columns_are_utf16_code_units() {
    // The emoji is one char, two UTF-16 code units, four UTF-8 bytes.
    let text = "{\n  \"a\": \"\u{1F5FA}\u{1F5FA}\",\n  \"b\": 1\n}";
    let map = SourceMap::parse(text);

    let a = map.range_for_path("a").unwrap();
    assert_eq!(a.start.character, 2);
    // 2 (indent) + 3 ("a") + 2 (": ") + 1 (open quote) + 4 (two emoji) + 1 (close)
    assert_eq!(a.end.character, 13);

    let b = map.range_for_path("b").unwrap();
    assert_eq!(b.start.line, 2);
    assert_eq!(b.start.character, 2);
}

#[test]
fn unescapes_keys_before_matching() {
    let map = SourceMap::parse(r#"{"a\nb": 1, "cAd": 2}"#);
    assert!(map.resolve("a\nb").is_some());
    assert!(map.resolve("cAd").is_some());
}

/// A language server indexes buffers mid-edit, so malformed input must degrade
/// rather than panic or hang.
#[test]
fn tolerates_malformed_input() {
    for text in [
        "",
        "{",
        "{\"version\": 8,",
        "{\"layers\": [{\"id\":",
        "{\"a\": }",
        "[,]",
        "not json at all",
        "{\"a\": \"unterminated",
        &"[".repeat(1000),
    ] {
        let map = SourceMap::parse(text);
        // Must not panic, and must still answer queries.
        let _ = map.range_for_path("version");
        let _ = map.len();
    }

    // Partial recovery: keys scanned before the fault are still indexed.
    let map = SourceMap::parse("{\n  \"version\": 8,\n  \"layers\": [{\"id\":");
    assert!(map.resolve("version").is_some());
}

/// The regression net: every path any rule emits over the real fixtures must
/// resolve *exactly*. If a new rule invents a path shape the scanner cannot
/// produce, this fails instead of silently degrading to a parent range.
/// Codes that deliberately report the *absence* of a key, so their path cannot
/// exist in the text and is expected to fall back to an enclosing range.
const ABSENCE_CODES: &[&str] = &[
    // W019: symbol layers use `text-field` but the style has no `glyphs` URL.
    "W019",
];

#[test]
fn every_fixture_diagnostic_path_resolves_exactly() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0usize;
    let mut unresolved = Vec::new();

    for entry in std::fs::read_dir(&dir).expect("fixtures dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let content = std::fs::read_to_string(&path).expect("read fixture");
        let value: serde_json::Value = serde_json::from_str(&content).expect("fixture is valid");
        let style: Style = serde_json::from_value(value).expect("fixture parses as a style");

        let mut diags = validator::run_all(&style, &styl::cli::Spec::Both);
        diags.extend(linter::run_all(&style, &styl::cli::Spec::Both));

        let map = SourceMap::parse(&content);
        for d in &diags {
            if ABSENCE_CODES.contains(&d.code) {
                continue;
            }
            checked += 1;
            if map.resolve(&d.path).is_none() {
                unresolved.push(format!("{}: [{}] {}", name, d.code, d.path));
            }
        }
    }

    assert!(checked > 0, "fixtures produced no diagnostics to check");
    assert!(
        unresolved.is_empty(),
        "{} of {} diagnostic paths did not resolve exactly:\n  {}",
        unresolved.len(),
        checked,
        unresolved.join("\n  ")
    );
}
