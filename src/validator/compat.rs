use crate::diagnostic::Diagnostic;
use crate::style::spec::SpecAffinity;
use crate::style::Style;
use crate::validator::Validator;

pub struct SkyCompatValidator;
pub struct TerrainCompatValidator;
pub struct FogCompatValidator;
pub struct ExpressionCompatValidator;

impl Validator for SkyCompatValidator {
    fn spec_affinity(&self) -> Option<SpecAffinity> { Some(SpecAffinity::MaplibreOnly) }
    fn validate(&self, _style: &Style) -> Vec<Diagnostic> { vec![] }
}
impl Validator for TerrainCompatValidator {
    fn spec_affinity(&self) -> Option<SpecAffinity> { Some(SpecAffinity::MaplibreOnly) }
    fn validate(&self, _style: &Style) -> Vec<Diagnostic> { vec![] }
}
impl Validator for FogCompatValidator {
    fn spec_affinity(&self) -> Option<SpecAffinity> { Some(SpecAffinity::MaplibreOnly) }
    fn validate(&self, _style: &Style) -> Vec<Diagnostic> { vec![] }
}
impl Validator for ExpressionCompatValidator {
    fn spec_affinity(&self) -> Option<SpecAffinity> { Some(SpecAffinity::MaplibreOnly) }
    fn validate(&self, _style: &Style) -> Vec<Diagnostic> { vec![] }
}
