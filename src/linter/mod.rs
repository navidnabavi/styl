use crate::diagnostic::Diagnostic;

pub mod rules;

pub trait LintRule {
    fn code(&self) -> &'static str;
    fn check(&self, style: &serde_json::Value) -> Vec<Diagnostic>;
}
