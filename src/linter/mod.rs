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
    ];

    rules
        .iter()
        .filter(|r| {
            r.spec_affinity()
                .map_or(true, |a| a.conflicts_with(spec))
        })
        .flat_map(|r| r.check(style))
        .collect()
}
