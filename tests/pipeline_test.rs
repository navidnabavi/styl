use std::path::Path;
use mapbox_style_tool_lib::*;

/// Helper: load fixture, run full pipeline, return diagnostics
fn check_fixture(name: &str) -> Vec<Diagnostic> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {}", name, e));
    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("invalid JSON in {}: {}", name, e));
    let style: Style = serde_json::from_value(value)
        .unwrap_or_else(|e| panic!("style parse failed for {}: {}", name, e));
    let mut diags = validator::run_all(&style);
    diags.extend(linter::run_all(&style));
    diags
}

#[test]
fn valid_minimal_has_no_diagnostics() {
    let diags = check_fixture("valid_minimal.json");
    assert!(
        diags.is_empty(),
        "expected no diagnostics for valid_minimal.json, got: {:?}",
        diags.iter().map(|d| format!("[{}] {}", d.code, d.message)).collect::<Vec<_>>()
    );
}

#[test]
fn valid_complex_has_no_errors() {
    let diags = check_fixture("valid_complex.json");
    let errors: Vec<_> = diags.iter().filter(|d| d.severity == Severity::Error).collect();
    assert!(
        errors.is_empty(),
        "expected no errors for valid_complex.json, got: {:?}",
        errors.iter().map(|d| format!("[{}] {}: {}", d.code, d.path, d.message)).collect::<Vec<_>>()
    );
}

#[test]
fn invalid_missing_source_has_e003() {
    let diags = check_fixture("invalid_missing_source.json");
    assert!(
        diags.iter().any(|d| d.code == "E003"),
        "expected E003 in invalid_missing_source.json"
    );
}

#[test]
fn invalid_bad_layer_props_has_e006() {
    let diags = check_fixture("invalid_bad_layer_props.json");
    assert!(
        diags.iter().any(|d| d.code == "E006"),
        "expected E006 (invalid paint prop) in invalid_bad_layer_props.json, got: {:?}",
        diags.iter().map(|d| d.code).collect::<Vec<_>>()
    );
}

#[test]
fn lint_warnings_triggers_w001_w002_w003_w004_w007_w008_w010() {
    let diags = check_fixture("lint_warnings.json");
    let codes: Vec<&str> = diags.iter().map(|d| d.code).collect();
    for expected in &["W001", "W002", "W003", "W004", "W007", "W008", "W010"] {
        assert!(
            codes.contains(expected),
            "expected {} in lint_warnings.json, got: {:?}", expected, codes
        );
    }
}

#[test]
fn legacy_filters_triggers_w011() {
    let diags = check_fixture("legacy_filters.json");
    assert!(
        diags.iter().any(|d| d.code == "W011"),
        "expected W011 in legacy_filters.json"
    );
}
