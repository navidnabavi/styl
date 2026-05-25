use crate::diagnostic::Diagnostic;
use crate::linter::LintRule;
use crate::style::Style;
use serde_json::Value;

/// W004: stop values not in ascending order in a stops array
pub struct StopOrder;

impl LintRule for StopOrder {
    fn code(&self) -> &'static str {
        "W004"
    }

    fn is_fixable(&self) -> bool {
        true
    }

    fn fix(&self, value: &mut serde_json::Value) {
        if let Some(layers) = value.get_mut("layers").and_then(|l| l.as_array_mut()) {
            for layer in layers.iter_mut() {
                if let Some(paint) = layer.get_mut("paint") {
                    fix_value_stops(paint);
                }
                if let Some(layout) = layer.get_mut("layout") {
                    fix_value_stops(layout);
                }
            }
        }
    }

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
                    && item
                        .as_array()
                        .map(|a| a.len() >= 2 && a[0].is_number())
                        .unwrap_or(false)
            }) && arr.len() >= 2
            {
                let stops: Vec<f64> = arr.iter().filter_map(|item| item[0].as_f64()).collect();
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

fn fix_value_stops(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(obj) => {
            for val in obj.values_mut() {
                fix_value_stops(val);
            }
        }
        serde_json::Value::Array(arr) => {
            // Detect stops array: [[number, value], ...]
            let is_stops = arr.len() >= 2
                && arr.iter().all(|item| {
                    item.is_array()
                        && item
                            .as_array()
                            .map(|a| a.len() >= 2 && a[0].is_number())
                            .unwrap_or(false)
                });
            if is_stops {
                arr.sort_by(|a, b| {
                    let fa = a[0].as_f64().unwrap_or(0.0);
                    let fb = b[0].as_f64().unwrap_or(0.0);
                    fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
                });
            } else {
                for item in arr.iter_mut() {
                    fix_value_stops(item);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Style {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_ascending_stops_ok() {
        let style = parse(
            r#"{
            "version":8,"sources":{},"layers":[{
                "id":"bg","type":"background",
                "paint":{"background-opacity":{"stops":[[0,0],[10,0.5],[20,1]]}}
            }]
        }"#,
        );
        assert!(StopOrder.check(&style).is_empty());
    }

    #[test]
    fn test_descending_stops_warn() {
        let style = parse(
            r#"{
            "version":8,"sources":{},"layers":[{
                "id":"bg","type":"background",
                "paint":{"background-opacity":{"stops":[[20,1],[10,0.5],[0,0]]}}
            }]
        }"#,
        );
        let diags = StopOrder.check(&style);
        assert!(diags.iter().any(|d| d.code == "W004"));
    }

    #[test]
    fn test_fix_stop_order() {
        let mut value = serde_json::json!({
            "version": 8,
            "sources": {},
            "layers": [{
                "id": "bg",
                "type": "background",
                "paint": {
                    "background-opacity": {
                        "stops": [[20, 1.0], [10, 0.5], [0, 0.0]]
                    }
                }
            }]
        });
        StopOrder.fix(&mut value);
        let stops = &value["layers"][0]["paint"]["background-opacity"]["stops"];
        assert_eq!(stops[0][0], serde_json::json!(0));
        assert_eq!(stops[1][0], serde_json::json!(10));
        assert_eq!(stops[2][0], serde_json::json!(20));
    }

    #[test]
    fn test_fix_stop_order_is_fixable() {
        assert!(StopOrder.is_fixable());
    }
}
