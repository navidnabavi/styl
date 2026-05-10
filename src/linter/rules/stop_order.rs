use serde_json::Value;
use crate::diagnostic::Diagnostic;
use crate::linter::LintRule;
use crate::style::Style;

/// W004: stop values not in ascending order in a stops array
pub struct StopOrder;

impl LintRule for StopOrder {
    fn code(&self) -> &'static str { "W004" }

    fn check(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, layer) in style.layers.iter().enumerate() {
            let layer_path = format!("layers[{}]", i);
            if let Some(paint) = &layer.paint {
                check_value_stops(paint, &format!("{}.paint", layer_path), &mut diags);
            }
            if let Some(layout) = &layer.layout {
                check_value_stops(layout, &format!("{}.layout", layer_path), &mut diags);
            }
        }
        diags
    }
}

fn check_value_stops(value: &Value, path: &str, diags: &mut Vec<Diagnostic>) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                check_value_stops(val, &format!("{}.{}", path, key), diags);
            }
        }
        Value::Array(arr) => {
            // Check if this looks like a stops array: [[stop, value], ...]
            if arr.iter().all(|item| {
                item.is_array()
                    && item.as_array().map(|a| a.len() >= 2 && a[0].is_number()).unwrap_or(false)
            }) && arr.len() >= 2 {
                let stops: Vec<f64> = arr.iter()
                    .filter_map(|item| item[0].as_f64())
                    .collect();
                for window in stops.windows(2) {
                    if window[0] >= window[1] {
                        diags.push(
                            Diagnostic::warning(
                                "W004",
                                format!("{}.stops", path),
                                "stop values are not in ascending order",
                            )
                            .with_hint("sort stops from lowest to highest zoom/value"),
                        );
                        break;
                    }
                }
            } else {
                // Recurse into array elements
                for (i, item) in arr.iter().enumerate() {
                    check_value_stops(item, &format!("{}[{}]", path, i), diags);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(json: &str) -> Style { serde_json::from_str(json).unwrap() }

    #[test]
    fn test_ascending_stops_ok() {
        let style = parse(r#"{
            "version":8,"sources":{},"layers":[{
                "id":"bg","type":"background",
                "paint":{"background-opacity":{"stops":[[0,0],[10,0.5],[20,1]]}}
            }]
        }"#);
        assert!(StopOrder.check(&style).is_empty());
    }

    #[test]
    fn test_descending_stops_warn() {
        let style = parse(r#"{
            "version":8,"sources":{},"layers":[{
                "id":"bg","type":"background",
                "paint":{"background-opacity":{"stops":[[20,1],[10,0.5],[0,0]]}}
            }]
        }"#);
        let diags = StopOrder.check(&style);
        assert!(diags.iter().any(|d| d.code == "W004"));
    }
}
