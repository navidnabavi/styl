use crate::diagnostic::Diagnostic;

pub trait Validator {
    fn validate(&self, value: &serde_json::Value) -> Vec<Diagnostic>;
}
