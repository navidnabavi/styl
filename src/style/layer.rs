use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A map layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: String,
    #[serde(rename = "type")]
    pub layer_type: LayerType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub layer_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(rename = "source-layer", skip_serializing_if = "Option::is_none")]
    pub source_layer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paint: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayerType {
    Background,
    Fill,
    FillExtrusion,
    Line,
    Symbol,
    Raster,
    Circle,
    Heatmap,
    Hillshade,
    Sky,
}

impl std::fmt::Display for LayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LayerType::Background => "background",
            LayerType::Fill => "fill",
            LayerType::FillExtrusion => "fill-extrusion",
            LayerType::Line => "line",
            LayerType::Symbol => "symbol",
            LayerType::Raster => "raster",
            LayerType::Circle => "circle",
            LayerType::Heatmap => "heatmap",
            LayerType::Hillshade => "hillshade",
            LayerType::Sky => "sky",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_deserialize_with_paint() {
        let json = r##"{
            "id": "water",
            "type": "fill",
            "source": "mapbox",
            "source-layer": "water",
            "paint": {"fill-color": "#00f"}
        }"##;
        let layer: Layer = serde_json::from_str(json).unwrap();
        assert_eq!(layer.id, "water");
        assert!(layer.paint.is_some());
    }

    #[test]
    fn test_layer_roundtrip() {
        let json = r##"{"id":"bg","type":"background","paint":{"background-color":"#fff"}}"##;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let reserialized = serde_json::to_string(&layer).unwrap();
        let layer2: Layer = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(layer2.id, layer.id);
    }
}
