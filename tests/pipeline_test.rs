use std::path::Path;
use styl::*;

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
    let mut diags = validator::run_all(&style, &styl::cli::Spec::Both);
    diags.extend(linter::run_all(&style, &styl::cli::Spec::Both));
    diags
}

#[test]
fn valid_minimal_has_no_diagnostics() {
    let diags = check_fixture("valid_minimal.json");
    assert!(
        diags.is_empty(),
        "expected no diagnostics for valid_minimal.json, got: {:?}",
        diags
            .iter()
            .map(|d| format!("[{}] {}", d.code, d.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn valid_complex_has_no_errors() {
    let diags = check_fixture("valid_complex.json");
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "expected no errors for valid_complex.json, got: {:?}",
        errors
            .iter()
            .map(|d| format!("[{}] {}: {}", d.code, d.path, d.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn invalid_missing_source_has_e009() {
    let diags = check_fixture("invalid_missing_source.json");
    assert!(
        diags.iter().any(|d| d.code == "E009"),
        "expected E009 in invalid_missing_source.json"
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
            "expected {} in lint_warnings.json, got: {:?}",
            expected,
            codes
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

#[test]
fn test_lint_fix_removes_shadowed_color_and_sorts_stops() {
    use std::fs;

    let input = serde_json::json!({
        "version": 8,
        "sources": {},
        "layers": [
            {
                "id": "bg",
                "type": "background",
                "paint": {
                    "background-color": "#fff",
                    "background-pattern": "dots",
                    "background-opacity": {
                        "stops": [[20, 1.0], [0, 0.0]]
                    }
                }
            }
        ]
    });

    let tmp = std::env::temp_dir().join("styl_fix_test.json");
    fs::write(&tmp, serde_json::to_string_pretty(&input).unwrap()).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_styl"))
        .args(["lint", "--fix", tmp.to_str().unwrap()])
        .output()
        .expect("failed to run styl");

    // All issues were fixed → exit 0
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let fixed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&tmp).unwrap()).unwrap();

    // W015 fix: background-color removed, pattern preserved
    assert!(fixed["layers"][0]["paint"]
        .get("background-color")
        .is_none());
    assert_eq!(
        fixed["layers"][0]["paint"]["background-pattern"],
        serde_json::json!("dots")
    );

    // W004 fix: stops sorted ascending
    let stops = &fixed["layers"][0]["paint"]["background-opacity"]["stops"];
    assert_eq!(stops[0][0], serde_json::json!(0));
    assert_eq!(stops[1][0], serde_json::json!(20));

    fs::remove_file(&tmp).unwrap();
}

mod spec_compat {
    use styl::{cli::Spec, diagnostic::Diagnostic, linter, style::Style, validator};

    fn run(json: &str, spec: Spec) -> Vec<Diagnostic> {
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let style: Style = serde_json::from_value(value).unwrap();
        let mut diags = validator::run_all(&style, &spec);
        diags.extend(linter::run_all(&style, &spec));
        diags
    }

    fn has_e023(diags: &[Diagnostic]) -> bool {
        diags.iter().any(|d| d.code == "E023")
    }

    #[test]
    fn sky_layer_flagged_by_both() {
        let style = r#"{"version":8,"sources":{},"layers":[{"id":"s","type":"sky"}]}"#;
        assert!(has_e023(&run(style, Spec::Both)));
    }

    #[test]
    fn sky_layer_flagged_by_mapbox() {
        let style = r#"{"version":8,"sources":{},"layers":[{"id":"s","type":"sky"}]}"#;
        assert!(has_e023(&run(style, Spec::Mapbox)));
    }

    #[test]
    fn sky_layer_clean_for_maplibre() {
        let style = r#"{"version":8,"sources":{},"layers":[{"id":"s","type":"sky"}]}"#;
        assert!(!has_e023(&run(style, Spec::Maplibre)));
    }

    #[test]
    fn terrain_flagged_by_both() {
        let style = r#"{"version":8,"sources":{},"layers":[],"terrain":{"source":"dem"}}"#;
        assert!(has_e023(&run(style, Spec::Both)));
    }

    #[test]
    fn terrain_clean_for_maplibre() {
        let style = r#"{"version":8,"sources":{},"layers":[],"terrain":{"source":"dem"}}"#;
        assert!(!has_e023(&run(style, Spec::Maplibre)));
    }

    #[test]
    fn fog_flagged_by_mapbox() {
        let style = r#"{"version":8,"sources":{},"layers":[],"fog":{"color":"white"}}"#;
        assert!(has_e023(&run(style, Spec::Mapbox)));
    }

    #[test]
    fn fog_clean_for_maplibre() {
        let style = r#"{"version":8,"sources":{},"layers":[],"fog":{"color":"white"}}"#;
        assert!(!has_e023(&run(style, Spec::Maplibre)));
    }

    #[test]
    fn clean_style_passes_all_specs() {
        let style = r#"{"version":8,"sources":{},"layers":[{"id":"bg","type":"background"}]}"#;
        assert!(!has_e023(&run(style, Spec::Both)));
        assert!(!has_e023(&run(style, Spec::Maplibre)));
        assert!(!has_e023(&run(style, Spec::Mapbox)));
    }

    #[test]
    fn distance_from_center_flagged_by_mapbox() {
        let style = r#"{"version":8,"sources":{},"layers":[{"id":"c","type":"circle","paint":{"circle-radius":["distance-from-center"]}}]}"#;
        assert!(has_e023(&run(style, Spec::Mapbox)));
    }

    #[test]
    fn distance_from_center_clean_for_maplibre() {
        let style = r#"{"version":8,"sources":{},"layers":[{"id":"c","type":"circle","paint":{"circle-radius":["distance-from-center"]}}]}"#;
        assert!(!has_e023(&run(style, Spec::Maplibre)));
    }
}
