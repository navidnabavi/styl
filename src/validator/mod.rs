use crate::diagnostic::Diagnostic;
use crate::style::Style;

pub mod layers;
pub mod prop_values;
pub mod refs;
pub mod root;
pub mod sources;

pub trait Validator {
    fn validate(&self, style: &Style) -> Vec<Diagnostic>;
}

/// Run all validators and collect diagnostics
pub fn run_all(style: &Style) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(root::validate_root(style));
    diags.extend(sources::validate_sources(style));
    diags.extend(layers::validate_layers(style));
    diags.extend(refs::validate_refs(style));
    diags
}
