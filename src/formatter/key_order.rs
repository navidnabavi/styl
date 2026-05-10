use indexmap::IndexMap;
use serde_json::Value;

/// Canonical root key order
pub const ROOT_KEY_ORDER: &[&str] = &[
    "version", "name", "metadata", "center", "zoom", "bearing", "pitch",
    "light", "terrain", "fog", "sprite", "glyphs", "transition", "sources", "layers",
];

/// Canonical layer key order
pub const LAYER_KEY_ORDER: &[&str] = &[
    "id", "type", "metadata", "ref", "source", "source-layer",
    "minzoom", "maxzoom", "filter", "layout", "paint",
];

/// Canonical source key order
pub const SOURCE_KEY_ORDER: &[&str] = &[
    "type", "url", "tiles", "bounds", "scheme", "minzoom", "maxzoom",
    "attribution", "promoteId",
];

/// Reorder keys in a JSON object according to a canonical order.
/// Unknown keys are appended at the end in their original order.
pub fn reorder_object(obj: &serde_json::Map<String, Value>, order: &[&str]) -> IndexMap<String, Value> {
    let mut result = IndexMap::new();

    // First: keys in canonical order
    for &key in order {
        if let Some(val) = obj.get(key) {
            result.insert(key.to_string(), val.clone());
        }
    }

    // Then: any remaining keys not in the canonical order (alphabetically)
    let mut extra: Vec<&String> = obj.keys()
        .filter(|k| !order.contains(&k.as_str()))
        .collect();
    extra.sort();
    for key in extra {
        result.insert(key.clone(), obj[key].clone());
    }

    result
}

/// Sort keys alphabetically within paint/layout objects
pub fn sort_paint_layout(obj: &serde_json::Map<String, Value>) -> IndexMap<String, Value> {
    let mut result = IndexMap::new();
    let mut keys: Vec<&String> = obj.keys().collect();
    keys.sort();
    for key in keys {
        result.insert(key.clone(), obj[key].clone());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_reorder_puts_canonical_first() {
        let obj = json!({"layers": [], "version": 8, "sources": {}});
        let map = obj.as_object().unwrap();
        let ordered = reorder_object(map, ROOT_KEY_ORDER);
        let keys: Vec<&str> = ordered.keys().map(|s| s.as_str()).collect();
        assert_eq!(keys[0], "version");
        assert_eq!(keys[1], "sources");
        assert_eq!(keys[2], "layers");
    }

    #[test]
    fn test_unknown_keys_appended() {
        let obj = json!({"version": 8, "sources": {}, "layers": [], "custom-field": "x"});
        let map = obj.as_object().unwrap();
        let ordered = reorder_object(map, ROOT_KEY_ORDER);
        let keys: Vec<&str> = ordered.keys().map(|s| s.as_str()).collect();
        assert_eq!(keys.last(), Some(&"custom-field"));
    }

    #[test]
    fn test_sort_paint_layout() {
        let obj = json!({"line-width": 2, "line-color": "red", "line-opacity": 0.5});
        let map = obj.as_object().unwrap();
        let sorted = sort_paint_layout(map);
        let keys: Vec<&str> = sorted.keys().map(|s| s.as_str()).collect();
        assert_eq!(keys, vec!["line-color", "line-opacity", "line-width"]);
    }
}
