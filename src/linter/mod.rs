use crate::diagnostic::Diagnostic;
use crate::style::Style;

pub mod config;
pub mod rules;

pub trait LintRule {
    fn code(&self) -> &'static str;
    fn check(&self, style: &Style) -> Vec<Diagnostic>;
}

/// Run all lint rules and collect diagnostics
pub fn run_all(style: &Style) -> Vec<Diagnostic> {
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
    ];

    rules.iter().flat_map(|r| r.check(style)).collect()
}
