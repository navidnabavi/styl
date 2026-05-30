use crate::cli::Spec;
use crate::diagnostic::Diagnostic;
use crate::style::spec::SpecAffinity;
use crate::style::Style;

pub mod compat;
pub mod layers;
pub mod prop_values;
pub mod refs;
pub mod root;
pub mod sources;

pub trait Validator {
    /// Spec affinity for compat validators. `None` = always runs.
    /// `Some(MaplibreOnly)` = runs when spec is Mapbox or Both (checks MapLibre-only features).
    fn spec_affinity(&self) -> Option<SpecAffinity> {
        None
    }
    fn validate(&self, style: &Style) -> Vec<Diagnostic>;
}

/// Run all validators, filtered by spec compatibility.
pub fn run_all(style: &Style, spec: &Spec) -> Vec<Diagnostic> {
    let validators: Vec<Box<dyn Validator>> = vec![
        Box::new(root::RootValidator),
        Box::new(sources::SourcesValidator),
        Box::new(layers::LayersValidator),
        Box::new(refs::RefsValidator),
        Box::new(compat::SkyCompatValidator),
        Box::new(compat::ColorReliefCompatValidator),
        Box::new(compat::TerrainCompatValidator),
        Box::new(compat::FogCompatValidator),
        Box::new(compat::ExpressionCompatValidator),
    ];

    validators
        .iter()
        .filter(|v| v.spec_affinity().is_none_or(|a| a.conflicts_with(spec)))
        .flat_map(|v| v.validate(style))
        .collect()
}
