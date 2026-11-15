use crate::cli::Spec;
use crate::diagnostic::Diagnostic;
use crate::style::spec::SpecAffinity;
use crate::style::Style;

pub mod config;
pub mod rules;

pub trait LintRule {
    fn code(&self) -> &'static str;
    /// Spec affinity for compat rules. `None` = always runs.
    /// `Some(MaplibreOnly)` = runs when spec is Mapbox or Both (checks MapLibre-only features).
    fn spec_affinity(&self) -> Option<SpecAffinity> {
        None
    }
    fn check(&self, style: &Style) -> Vec<Diagnostic>;
    /// Apply an in-place fix to the raw JSON value. Only called when
    /// `is_fixable()` is true. Need not report whether it changed anything —
    /// `run_fixes` determines that by comparing the document.
    fn fix(&self, _value: &mut serde_json::Value) {}
    /// Whether this rule can automatically fix the issues it detects.
    fn is_fixable(&self) -> bool {
        false
    }
}

/// Run all lint rules, filtered by spec compatibility.
pub fn run_all(style: &Style, spec: &Spec) -> Vec<Diagnostic> {
    let rules: Vec<Box<dyn LintRule>> = vec![
        Box::new(rules::duplicate_ids::DuplicateIds),
        Box::new(rules::visibility::PermanentlyInvisible),
        Box::new(rules::unused_layers::UnusedSource),
        Box::new(rules::stop_order::StopOrder),
        Box::new(rules::z_order::FillExtrusionBelowBackground),
        Box::new(rules::expression_depth::ExpressionDepth),
        Box::new(rules::perf_hints::EmptyTextField),
        Box::new(rules::perf_hints::PlaceholderIconImage),
        Box::new(rules::perf_hints::LayerCountHint),
        Box::new(rules::perf_hints::ZeroDasharray),
        Box::new(rules::perf_hints::LegacyFilter),
        Box::new(rules::perf_hints::RasterResampling),
        Box::new(rules::perf_hints::SymbolNoContent),
        Box::new(rules::perf_hints::SymbolMissingFont),
        Box::new(rules::perf_hints::BackgroundPatternOverridesColor),
        Box::new(rules::perf_hints::FillPatternOverridesColor),
        Box::new(rules::perf_hints::LinePatternOverridesColor),
        Box::new(rules::perf_hints::HeatmapMissingColor),
        Box::new(rules::perf_hints::MissingGlyphs),
        Box::new(rules::perf_hints::FogMissingColor),
        Box::new(rules::perf_hints::FogMissingRange),
        Box::new(rules::perf_hints::IconImageWithoutSprite),
    ];

    rules
        .iter()
        .filter(|r| r.spec_affinity().is_none_or(|a| a.conflicts_with(spec)))
        .flat_map(|r| r.check(style))
        .collect()
}

/// Apply all fixable rules to the raw JSON value in-place.
/// Returns a list of rule codes that were applied.
pub fn run_fixes(value: &mut serde_json::Value, spec: &crate::cli::Spec) -> Vec<&'static str> {
    let rules: Vec<Box<dyn LintRule>> = vec![
        Box::new(rules::stop_order::StopOrder),
        Box::new(rules::perf_hints::EmptyTextField),
        Box::new(rules::perf_hints::ZeroDasharray),
        Box::new(rules::perf_hints::LegacyFilter),
        Box::new(rules::perf_hints::BackgroundPatternOverridesColor),
        Box::new(rules::perf_hints::FillPatternOverridesColor),
        Box::new(rules::perf_hints::LinePatternOverridesColor),
    ];

    rules
        .into_iter()
        .filter(|r| r.spec_affinity().is_none_or(|a| a.conflicts_with(spec)) && r.is_fixable())
        .filter_map(|r| {
            // Report what actually changed rather than what ran. Comparing the
            // document before and after keeps every rule honest without asking
            // each `fix` to track its own mutations, where one forgetting the
            // flag would silently misreport.
            //
            // Caveat: `serde_json::Map` compares order-insensitively under
            // `preserve_order`, so a fix that only reordered object keys would
            // look like a no-op. None do; array reordering is caught, because
            // arrays are `Vec`.
            let before = value.clone();
            r.fix(value);
            (*value != before).then_some(r.code())
        })
        .collect()
}

#[cfg(test)]
mod fix_reporting_tests {
    use crate::cli::Spec;

    /// A document with one fixable problem must report that one code, not every
    /// fixable rule that happened to run.
    #[test]
    fn run_fixes_reports_only_rules_that_changed_something() {
        let mut value: serde_json::Value = serde_json::json!({
            "version": 8,
            "sources": {},
            "layers": [{
                "id": "l",
                "type": "line",
                "source": "s",
                "paint": { "line-dasharray": [0, 0] }
            }]
        });

        let fixed = super::run_fixes(&mut value, &Spec::Both);
        assert_eq!(fixed, vec!["W010"], "expected only the dasharray fix");
    }

    #[test]
    fn run_fixes_reports_nothing_for_a_clean_document() {
        let mut value: serde_json::Value = serde_json::json!({
            "version": 8,
            "sources": {},
            "layers": []
        });
        assert!(super::run_fixes(&mut value, &Spec::Both).is_empty());
    }
}

#[cfg(test)]
mod trait_tests {
    use super::*;
    use crate::style::Style;

    struct AlwaysFixable;
    impl LintRule for AlwaysFixable {
        fn code(&self) -> &'static str {
            "W999"
        }
        fn check(&self, _style: &Style) -> Vec<crate::diagnostic::Diagnostic> {
            vec![]
        }
        fn is_fixable(&self) -> bool {
            true
        }
        fn fix(&self, value: &mut serde_json::Value) {
            value["__fixed"] = serde_json::json!(true);
        }
    }

    #[test]
    fn test_fixable_rule_trait() {
        let rule = AlwaysFixable;
        assert!(rule.is_fixable());
        let mut v = serde_json::json!({});
        rule.fix(&mut v);
        assert_eq!(v["__fixed"], serde_json::json!(true));
    }

    #[test]
    fn test_default_not_fixable() {
        struct NeverFix;
        impl LintRule for NeverFix {
            fn code(&self) -> &'static str {
                "W998"
            }
            fn check(&self, _style: &Style) -> Vec<crate::diagnostic::Diagnostic> {
                vec![]
            }
        }
        let rule = NeverFix;
        assert!(!rule.is_fixable());
        let mut v = serde_json::json!({"x": 1});
        rule.fix(&mut v); // should be no-op
        assert_eq!(v["x"], serde_json::json!(1));
    }
}
