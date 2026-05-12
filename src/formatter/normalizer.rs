use indexmap::IndexMap;
use serde_json::Value;

use super::key_order::{
    reorder_object, sort_paint_layout, LAYER_KEY_ORDER, ROOT_KEY_ORDER, SOURCE_KEY_ORDER,
};

#[derive(Copy, Clone)]
enum Context {
    Layer,
    Source,
    PaintLayout,
    Other,
}

fn normalize_value(value: &Value, ctx: Context) -> Value {
    match value {
        Value::Object(obj) => {
            let ordered: IndexMap<String, Value> = match ctx {
                Context::Layer => reorder_object(obj, LAYER_KEY_ORDER)
                    .into_iter()
                    .map(|(k, v)| {
                        let child_ctx = match k.as_str() {
                            "paint" | "layout" => Context::PaintLayout,
                            _ => Context::Other,
                        };
                        (k, normalize_value(&v, child_ctx))
                    })
                    .collect(),
                Context::Source => reorder_object(obj, SOURCE_KEY_ORDER)
                    .into_iter()
                    .map(|(k, v)| (k, normalize_value(&v, Context::Other)))
                    .collect(),
                Context::PaintLayout => sort_paint_layout(obj)
                    .into_iter()
                    .map(|(k, v)| (k, normalize_value(&v, Context::Other)))
                    .collect(),
                Context::Other => obj
                    .iter()
                    .map(|(k, v)| (k.clone(), normalize_value(v, Context::Other)))
                    .collect(),
            };
            // Convert IndexMap to serde_json::Value via intermediate Map
            let map: serde_json::Map<String, Value> = ordered.into_iter().collect();
            Value::Object(map)
        }
        Value::Array(arr) => {
            // Special handling: layers array → each element gets Layer context
            // Sources object handled above. For arrays, we need the parent context.
            // Since we don't know if this array is "layers" or "sources.x.tiles" etc,
            // we apply Layer context only when explicitly called from Root with key "layers".
            Value::Array(
                arr.iter()
                    .map(|v| normalize_value(v, Context::Other))
                    .collect(),
            )
        }
        other => other.clone(),
    }
}

/// Normalize root-level value, properly applying Layer context to layers array
/// and Source context to sources object values.
fn normalize_root(value: &Value) -> Value {
    if let Value::Object(obj) = value {
        let reordered = reorder_object(obj, ROOT_KEY_ORDER);
        let map: serde_json::Map<String, Value> = reordered
            .into_iter()
            .map(|(k, v)| {
                let normalized = match k.as_str() {
                    "layers" => {
                        if let Value::Array(arr) = &v {
                            Value::Array(
                                arr.iter()
                                    .map(|l| normalize_value(l, Context::Layer))
                                    .collect(),
                            )
                        } else {
                            v
                        }
                    }
                    "sources" => {
                        if let Value::Object(sources) = &v {
                            let normalized_sources: serde_json::Map<String, Value> = sources
                                .iter()
                                .map(|(id, src)| {
                                    (id.clone(), normalize_value(src, Context::Source))
                                })
                                .collect();
                            Value::Object(normalized_sources)
                        } else {
                            v
                        }
                    }
                    _ => normalize_value(&v, Context::Other),
                };
                (k, normalized)
            })
            .collect();
        Value::Object(map)
    } else {
        value.clone()
    }
}

/// Re-indent JSON string from 2 spaces to `indent` spaces per level.
fn re_indent(json: &str, indent: usize) -> String {
    let spaces = " ".repeat(indent);
    let mut result = String::new();
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut escaped = false;

    for ch in json.chars() {
        if escaped {
            result.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            result.push(ch);
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            continue;
        }
        if in_string {
            result.push(ch);
            continue;
        }
        match ch {
            '{' | '[' => {
                result.push(ch);
                depth += 1;
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                result.push(ch);
            }
            '\n' => {
                result.push('\n');
                result.push_str(&spaces.repeat(depth));
            }
            ' ' => {
                // Skip existing indent spaces (they come after \n)
                // We already added the indent after \n, so skip leading spaces
                if result.ends_with('\n') || result.ends_with(&spaces.repeat(depth)) {
                    // skip
                } else {
                    result.push(ch);
                }
            }
            _ => result.push(ch),
        }
    }
    result
}

/// Public entry point: format_style with proper root normalization
pub fn format_style(value: &Value, indent: usize) -> String {
    let ordered = normalize_root(value);
    let serialized = serde_json::to_string_pretty(&ordered).unwrap_or_default();
    let result = if indent == 2 {
        serialized
    } else {
        re_indent(&serialized, indent)
    };
    if result.ends_with('\n') {
        result
    } else {
        result + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_root_key_order() {
        let style = json!({
            "layers": [],
            "sources": {},
            "version": 8,
            "name": "test"
        });
        let formatted = format_style(&style, 2);
        let parsed: Value = serde_json::from_str(&formatted).unwrap();
        let keys: Vec<&str> = parsed
            .as_object()
            .unwrap()
            .keys()
            .map(|s| s.as_str())
            .collect();
        // version must come before name which must come before sources, layers
        let vi = keys.iter().position(|&k| k == "version").unwrap();
        let ni = keys.iter().position(|&k| k == "name").unwrap();
        let si = keys.iter().position(|&k| k == "sources").unwrap();
        let li = keys.iter().position(|&k| k == "layers").unwrap();
        assert!(vi < ni && ni < si && si < li);
    }

    #[test]
    fn test_layer_key_order() {
        let style = json!({
            "version": 8,
            "sources": {"s": {"type": "geojson", "data": null}},
            "layers": [{
                "paint": {"fill-color": "red"},
                "source": "s",
                "type": "fill",
                "id": "my-layer"
            }]
        });
        let formatted = format_style(&style, 2);
        let parsed: Value = serde_json::from_str(&formatted).unwrap();
        let layer = &parsed["layers"][0];
        let keys: Vec<&str> = layer
            .as_object()
            .unwrap()
            .keys()
            .map(|s| s.as_str())
            .collect();
        let ii = keys.iter().position(|&k| k == "id").unwrap();
        let ti = keys.iter().position(|&k| k == "type").unwrap();
        assert!(ii < ti);
    }

    #[test]
    fn test_paint_keys_sorted() {
        let style = json!({
            "version": 8,
            "sources": {"s": {"type": "geojson", "data": null}},
            "layers": [{
                "id": "l", "type": "fill", "source": "s",
                "paint": {"fill-opacity": 0.5, "fill-color": "blue", "fill-antialias": true}
            }]
        });
        let formatted = format_style(&style, 2);
        let parsed: Value = serde_json::from_str(&formatted).unwrap();
        let paint = &parsed["layers"][0]["paint"];
        let keys: Vec<&str> = paint
            .as_object()
            .unwrap()
            .keys()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(keys, vec!["fill-antialias", "fill-color", "fill-opacity"]);
    }

    #[test]
    fn test_trailing_newline() {
        let style = json!({"version": 8, "sources": {}, "layers": []});
        let formatted = format_style(&style, 2);
        assert!(formatted.ends_with('\n'));
    }

    #[test]
    fn test_roundtrip_preserves_values() {
        let style = json!({
            "version": 8,
            "sources": {"s": {"type": "vector", "url": "mapbox://foo"}},
            "layers": [{"id": "l", "type": "line", "source": "s", "source-layer": "road"}]
        });
        let formatted = format_style(&style, 2);
        let parsed: Value = serde_json::from_str(&formatted).unwrap();
        assert_eq!(parsed["version"], 8);
        assert_eq!(parsed["layers"][0]["id"], "l");
    }
}
