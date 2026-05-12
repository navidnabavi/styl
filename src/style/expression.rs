use crate::diagnostic::Diagnostic;
use serde_json::Value;

/// The expected output type of an expression
#[derive(Debug, Clone, PartialEq)]
pub enum ExprType {
    Number,
    String,
    Boolean,
    Color,
    Array,
    Object,
    Any,
    Formatted,
    ResolvedImage,
}

impl std::fmt::Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprType::Number => write!(f, "number"),
            ExprType::String => write!(f, "string"),
            ExprType::Boolean => write!(f, "boolean"),
            ExprType::Color => write!(f, "color"),
            ExprType::Array => write!(f, "array"),
            ExprType::Object => write!(f, "object"),
            ExprType::Any => write!(f, "any"),
            ExprType::Formatted => write!(f, "formatted"),
            ExprType::ResolvedImage => write!(f, "resolvedImage"),
        }
    }
}

/// Validate an expression value, returning diagnostics.
/// `path` is the JSON pointer path to this expression (for diagnostic paths).
/// `depth` tracks recursion depth for W006.
pub fn validate_expression(value: &Value, path: &str, depth: usize) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    if depth > 10 {
        diags.push(
            Diagnostic::warning(
                "W006",
                path,
                "expression depth exceeds 10 (performance impact)",
            )
            .with_hint("simplify nested expressions or use intermediate variables with 'let'"),
        );
        // Don't recurse further — avoid noise
        return diags;
    }

    match value {
        // Literal values are always valid expressions
        Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Null => {}

        // Array starting with a string = expression operator
        Value::Array(arr) if !arr.is_empty() && arr[0].is_string() => {
            let op = arr[0].as_str().unwrap();
            diags.extend(validate_operator(op, arr, path, depth));
        }

        // Empty array or array starting with non-string: not a valid expression operator
        Value::Array(arr) if arr.is_empty() => {
            diags.push(Diagnostic::error(
                "E020",
                path,
                "expression must be a non-empty array starting with an operator string",
            ));
        }

        // Objects and arrays with non-string first element treated as literal values
        Value::Object(_) | Value::Array(_) => {}
    }

    diags
}

/// Validate a known expression operator and its arguments.
fn validate_operator(op: &str, args: &[Value], path: &str, depth: usize) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    match op {
        // --- Lookup ---
        "get" => {
            if args.len() < 2 || args.len() > 3 {
                diags.push(arity_error(path, op, "1 or 2 arguments", args.len() - 1));
            }
        }
        "has" => {
            if args.len() < 2 || args.len() > 3 {
                diags.push(arity_error(path, op, "1 or 2 arguments", args.len() - 1));
            }
        }
        "at" => {
            if args.len() != 3 {
                diags.push(arity_error(path, op, "2 arguments", args.len() - 1));
            }
        }
        "in" => {
            if args.len() != 3 {
                diags.push(arity_error(path, op, "2 arguments", args.len() - 1));
            }
        }
        "index-of" => {
            if args.len() < 3 || args.len() > 4 {
                diags.push(arity_error(path, op, "2 or 3 arguments", args.len() - 1));
            }
        }
        "slice" => {
            if args.len() < 3 || args.len() > 4 {
                diags.push(arity_error(path, op, "2 or 3 arguments", args.len() - 1));
            }
        }
        "length" => {
            if args.len() != 2 {
                diags.push(arity_error(path, op, "1 argument", args.len() - 1));
            }
        }

        // --- Decision ---
        "case" => {
            // case: [condition, output, condition, output, ..., fallback]
            if args.len() < 4 || args.len() % 2 != 0 {
                diags.push(Diagnostic::error(
                    "E021",
                    path,
                    "\"case\" requires pairs of [condition, output] plus a fallback",
                ));
            }
        }
        "match" => {
            // match: [input, label, output, ..., fallback] — at least 4 args
            if args.len() < 5 {
                diags.push(Diagnostic::error(
                    "E021",
                    path,
                    "\"match\" requires input, at least one label-output pair, and a fallback",
                ));
            }
        }
        "coalesce" => {
            if args.len() < 2 {
                diags.push(arity_error(path, op, "at least 1 argument", 0));
            }
        }
        "within" => {
            if args.len() != 2 {
                diags.push(arity_error(
                    path,
                    op,
                    "1 argument (GeoJSON polygon)",
                    args.len() - 1,
                ));
            }
        }

        // --- Math ---
        // "+" and "*" are variadic (0+ args); "-" allows 1 arg (negation) or 2+ (subtraction)
        "+" | "*" => {}
        "-" => {
            if args.len() < 2 {
                diags.push(arity_error(path, op, "1 or 2 arguments", args.len() - 1));
            }
        }
        "/" | "%" | "^" => {
            if args.len() != 3 {
                diags.push(arity_error(path, op, "exactly 2 arguments", args.len() - 1));
            }
        }
        // Math (unary)
        "abs" | "ceil" | "floor" | "round" | "sqrt" | "log2" | "log10" | "ln" | "sin" | "cos"
        | "tan" | "asin" | "acos" | "atan" => {
            if args.len() != 2 {
                diags.push(arity_error(path, op, "1 argument", args.len() - 1));
            }
        }
        "min" | "max" => {
            if args.len() < 3 {
                diags.push(arity_error(
                    path,
                    op,
                    "at least 2 arguments",
                    args.len() - 1,
                ));
            }
        }

        // --- Comparison ---
        "==" | "!=" | "<" | "<=" | ">" | ">=" => {
            if args.len() < 3 || args.len() > 4 {
                diags.push(arity_error(path, op, "2 or 3 arguments", args.len() - 1));
            }
        }

        // --- Logic ---
        "all" | "any" => {
            if args.len() < 2 {
                diags.push(arity_error(path, op, "at least 1 argument", args.len() - 1));
            }
        }
        "!" => {
            if args.len() != 2 {
                diags.push(arity_error(path, op, "1 argument", args.len() - 1));
            }
        }

        // --- String ---
        "concat" => {
            if args.len() < 3 {
                diags.push(arity_error(
                    path,
                    op,
                    "at least 2 arguments",
                    args.len() - 1,
                ));
            }
        }
        "downcase" | "upcase" | "to-string" => {
            if args.len() != 2 {
                diags.push(arity_error(path, op, "1 argument", args.len() - 1));
            }
        }
        "number-format" => {
            if args.len() < 3 {
                diags.push(arity_error(
                    path,
                    op,
                    "at least 2 arguments",
                    args.len() - 1,
                ));
            }
        }

        // --- Color ---
        "rgb" => {
            if args.len() != 4 {
                diags.push(arity_error(
                    path,
                    op,
                    "3 arguments (r, g, b)",
                    args.len() - 1,
                ));
            }
        }
        "rgba" => {
            if args.len() != 5 {
                diags.push(arity_error(
                    path,
                    op,
                    "4 arguments (r, g, b, a)",
                    args.len() - 1,
                ));
            }
        }
        "hsl" => {
            if args.len() != 4 {
                diags.push(arity_error(
                    path,
                    op,
                    "3 arguments (h, s, l)",
                    args.len() - 1,
                ));
            }
        }
        "hsla" => {
            if args.len() != 5 {
                diags.push(arity_error(
                    path,
                    op,
                    "4 arguments (h, s, l, a)",
                    args.len() - 1,
                ));
            }
        }
        "to-color" => {
            if args.len() < 2 {
                diags.push(arity_error(path, op, "at least 1 argument", args.len() - 1));
            }
        }

        // --- Camera ---
        "zoom" | "pitch" | "distance-from-center" => {
            if args.len() != 1 {
                diags.push(arity_error(path, op, "0 arguments", args.len() - 1));
            }
        }

        // --- Color output ---
        "to-rgba" => {
            if args.len() != 2 {
                diags.push(arity_error(path, op, "1 argument (color)", args.len() - 1));
            }
        }

        // --- Image ---
        "resolved-image" => {
            if args.len() < 2 {
                diags.push(arity_error(path, op, "at least 1 argument", args.len() - 1));
            }
        }

        // --- Script / i18n ---
        "is-supported-script" => {
            if args.len() != 2 {
                diags.push(arity_error(path, op, "1 argument (string)", args.len() - 1));
            }
        }
        "collator" => {
            if args.len() != 2 {
                diags.push(arity_error(
                    path,
                    op,
                    "1 argument (options object)",
                    args.len() - 1,
                ));
            }
        }

        // --- Config ---
        "config" => {
            if args.len() < 2 {
                diags.push(arity_error(path, op, "at least 1 argument", args.len() - 1));
            }
        }

        // --- Distance ---
        "distance" => {
            if args.len() != 2 {
                diags.push(arity_error(
                    path,
                    op,
                    "1 argument (GeoJSON geometry)",
                    args.len() - 1,
                ));
            }
        }

        // --- Random ---
        "random" => {
            if args.len() < 3 || args.len() > 4 {
                diags.push(arity_error(
                    path,
                    op,
                    "2 or 3 arguments (min, max[, seed])",
                    args.len() - 1,
                ));
            }
        }

        // --- Interpolation ---
        "interpolate" | "interpolate-hcl" | "interpolate-lab" => {
            if args.len() < 5 {
                diags.push(Diagnostic::error(
                    "E021",
                    path,
                    format!(
                        "\"{}\" requires interpolation type, input, and at least one stop pair",
                        op
                    ),
                ));
            }
        }
        "step" => {
            if args.len() < 5 {
                diags.push(Diagnostic::error(
                    "E021",
                    path,
                    "\"step\" requires input, default, and at least one stop pair",
                ));
            }
        }

        // --- Binding ---
        "let" => {
            // let: [name, value, name, value, ..., expression] — at least 3, odd length
            if args.len() < 4 || args.len() % 2 != 0 {
                diags.push(Diagnostic::error(
                    "E021",
                    path,
                    "\"let\" requires pairs of [name, value] followed by an output expression",
                ));
            }
        }
        "var" => {
            if args.len() != 2 || !args[1].is_string() {
                diags.push(Diagnostic::error(
                    "E021",
                    path,
                    "\"var\" requires a single string argument (variable name)",
                ));
            }
        }

        // --- Type assertions ---
        "string" | "number" | "boolean" | "object" | "array" | "to-boolean" | "to-number" => {
            if args.len() < 2 {
                diags.push(arity_error(path, op, "at least 1 argument", args.len() - 1));
            }
        }
        "literal" => {
            if args.len() != 2 {
                diags.push(arity_error(path, op, "1 argument", args.len() - 1));
            }
        }
        "typeof" => {
            if args.len() != 2 {
                diags.push(arity_error(path, op, "1 argument", args.len() - 1));
            }
        }

        // --- Feature data ---
        "feature-state" => {
            if args.len() != 2 || !args[1].is_string() {
                diags.push(Diagnostic::error(
                    "E021",
                    path,
                    "\"feature-state\" requires a single string key argument",
                ));
            }
        }
        "geometry-type" | "id" | "line-progress" | "properties" | "accumulated" => {
            if args.len() != 1 {
                diags.push(arity_error(path, op, "0 arguments", args.len() - 1));
            }
        }

        // --- Format ---
        "format" | "image" => {
            // format: [string-or-expr, options, ...] — at least 2
            if args.len() < 2 {
                diags.push(arity_error(path, op, "at least 1 argument", args.len() - 1));
            }
        }

        // Interpolation type sub-expressions (used inside interpolate)
        "linear" => {
            if args.len() != 1 {
                diags.push(arity_error(path, op, "0 arguments", args.len() - 1));
            }
        }
        "exponential" => {
            if args.len() != 2 {
                diags.push(arity_error(path, op, "1 argument (base)", args.len() - 1));
            }
        }
        "cubic-bezier" => {
            if args.len() != 5 {
                diags.push(arity_error(
                    path,
                    op,
                    "4 arguments (x1, y1, x2, y2)",
                    args.len() - 1,
                ));
            }
        }

        // Unknown operator
        _ => {
            diags.push(Diagnostic::error(
                "E022",
                path,
                format!("unknown expression operator \"{}\"", op),
            ));
        }
    }

    // Recursively validate sub-expressions (all args except the operator itself)
    for (i, arg) in args[1..].iter().enumerate() {
        // Only recurse into array sub-expressions
        if arg.is_array() || arg.is_object() {
            let child_path = format!("{}/{}", path, i + 1);
            diags.extend(validate_expression(arg, &child_path, depth + 1));
        }
    }

    diags
}

/// Returns true if the value looks like a legacy (pre-expression) filter.
/// Used by W011 to warn and by the validator to skip expression validation.
pub fn is_legacy_filter(value: &Value) -> bool {
    if let Some(arr) = value.as_array() {
        if let Some(first) = arr.first() {
            if let Some(op) = first.as_str() {
                return match op {
                    "!=" | "!in" | "!has" | "none" => true,
                    "==" | ">" | ">=" | "<" | "<=" | "in" | "has" => {
                        arr.get(1).map(|a| a.is_string()).unwrap_or(false)
                    }
                    "all" | "any" => arr[1..].iter().any(is_unambiguously_legacy_filter),
                    _ => false,
                };
            }
        }
    }
    false
}

fn is_unambiguously_legacy_filter(value: &Value) -> bool {
    if let Some(arr) = value.as_array() {
        if let Some(first) = arr.first() {
            if let Some(op) = first.as_str() {
                return match op {
                    "!=" | "!in" | "!has" | "none" => true,
                    "==" | ">" | ">=" | "<" | "<=" | "in" => {
                        arr.get(1).map(|a| a.is_string()).unwrap_or(false)
                    }
                    _ => false,
                };
            }
        }
    }
    false
}

fn arity_error(path: &str, op: &str, expected: &str, got: usize) -> Diagnostic {
    Diagnostic::error(
        "E021",
        path,
        format!("\"{}\" expects {} but got {}", op, expected, got),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_literal_number_is_valid() {
        let diags = validate_expression(&json!(42), "paint.circle-radius", 0);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_literal_string_is_valid() {
        let diags = validate_expression(&json!("round"), "layout.line-cap", 0);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_get_expression_valid() {
        let diags = validate_expression(&json!(["get", "name"]), "layout.text-field", 0);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_get_expression_wrong_arity() {
        let diags = validate_expression(&json!(["get"]), "layout.text-field", 0);
        assert!(diags.iter().any(|d| d.code == "E021"));
    }

    #[test]
    fn test_unknown_operator() {
        let diags = validate_expression(&json!(["not-an-op", "x"]), "paint.fill-color", 0);
        assert!(diags.iter().any(|d| d.code == "E022"));
    }

    #[test]
    fn test_zoom_no_args() {
        let diags = validate_expression(&json!(["zoom"]), "paint.circle-radius", 0);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_zoom_wrong_arity() {
        let diags = validate_expression(&json!(["zoom", "extra"]), "paint.circle-radius", 0);
        assert!(diags.iter().any(|d| d.code == "E021"));
    }

    #[test]
    fn test_interpolate_valid() {
        let expr = json!(["interpolate", ["linear"], ["zoom"], 5, 1, 10, 5]);
        let diags = validate_expression(&expr, "paint.line-width", 0);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_interpolate_too_few_args() {
        let expr = json!(["interpolate", ["linear"], ["zoom"]]);
        let diags = validate_expression(&expr, "paint.line-width", 0);
        assert!(diags.iter().any(|d| d.code == "E021"));
    }

    #[test]
    fn test_depth_warning_at_11() {
        // Build a deeply nested get expression: ["get", ["get", ["get", ...]]]
        let mut expr = json!(["get", "x"]);
        for _ in 0..11 {
            expr = json!(["get", expr]);
        }
        let diags = validate_expression(&expr, "paint.fill-color", 0);
        assert!(diags.iter().any(|d| d.code == "W006"));
    }

    #[test]
    fn test_empty_array_is_error() {
        let diags = validate_expression(&json!([]), "paint.fill-color", 0);
        assert!(diags.iter().any(|d| d.code == "E020"));
    }

    #[test]
    fn test_case_requires_pairs_plus_fallback() {
        // valid: ["case", cond, val, fallback]
        let valid = json!(["case", ["has", "name"], "yes", "no"]);
        assert!(validate_expression(&valid, "p", 0).is_empty());

        // invalid: odd number without fallback
        let invalid = json!(["case", ["has", "name"], "yes"]);
        let diags = validate_expression(&invalid, "p", 0);
        assert!(diags.iter().any(|d| d.code == "E021"));
    }

    #[test]
    fn test_rgb_arity() {
        let valid = json!(["rgb", 255, 0, 0]);
        assert!(validate_expression(&valid, "p", 0).is_empty());

        let invalid = json!(["rgb", 255, 0]);
        let diags = validate_expression(&invalid, "p", 0);
        assert!(diags.iter().any(|d| d.code == "E021"));
    }
}
