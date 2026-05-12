use crate::style::layer::Layer;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Sprite value: either a single URL string or an array of {id, url} entries (MapLibre v8+)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpriteValue {
    String(String),
    Array(Vec<SpriteEntry>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteEntry {
    pub id: String,
    pub url: String,
}

/// Root MapLibre/Mapbox GL Style object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terrain: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fog: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprite: Option<SpriteValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glyphs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<Value>,
    #[serde(default)]
    pub sources: IndexMap<String, Source>,
    #[serde(default)]
    pub layers: Vec<Layer>,
}

/// A map source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Source {
    Vector(VectorSource),
    Raster(RasterSource),
    #[serde(rename = "raster-dem")]
    RasterDem(RasterDemSource),
    #[serde(rename = "geojson")]
    GeoJson(GeoJsonSource),
    Image(ImageSource),
    Video(VideoSource),
}

impl Source {
    pub fn source_type(&self) -> &'static str {
        match self {
            Source::Vector(_) => "vector",
            Source::Raster(_) => "raster",
            Source::RasterDem(_) => "raster-dem",
            Source::GeoJson(_) => "geojson",
            Source::Image(_) => "image",
            Source::Video(_) => "video",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[f64; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    #[serde(rename = "promoteId", skip_serializing_if = "Option::is_none")]
    pub promote_id: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RasterSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[f64; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<f64>,
    #[serde(rename = "tileSize", skip_serializing_if = "Option::is_none")]
    pub tile_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RasterDemSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[f64; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<f64>,
    #[serde(rename = "tileSize", skip_serializing_if = "Option::is_none")]
    pub tile_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonSource {
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<bool>,
    #[serde(rename = "clusterRadius", skip_serializing_if = "Option::is_none")]
    pub cluster_radius: Option<f64>,
    #[serde(rename = "clusterMaxZoom", skip_serializing_if = "Option::is_none")]
    pub cluster_max_zoom: Option<f64>,
    #[serde(rename = "clusterProperties", skip_serializing_if = "Option::is_none")]
    pub cluster_properties: Option<Value>,
    #[serde(rename = "lineMetrics", skip_serializing_if = "Option::is_none")]
    pub line_metrics: Option<bool>,
    #[serde(rename = "generateId", skip_serializing_if = "Option::is_none")]
    pub generate_id: Option<bool>,
    #[serde(rename = "promoteId", skip_serializing_if = "Option::is_none")]
    pub promote_id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    pub url: String,
    pub coordinates: [[f64; 2]; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSource {
    pub urls: Vec<String>,
    pub coordinates: [[f64; 2]; 4],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_style() {
        let json = r#"{
            "version": 8,
            "sources": {},
            "layers": []
        }"#;
        let style: Style = serde_json::from_str(json).unwrap();
        assert_eq!(style.version, 8);
        assert!(style.sources.is_empty());
        assert!(style.layers.is_empty());
    }

    #[test]
    fn test_parse_style_with_vector_source() {
        let json = r#"{
            "version": 8,
            "sources": {
                "my-tiles": {
                    "type": "vector",
                    "url": "mapbox://mapbox.mapbox-terrain-v2"
                }
            },
            "layers": []
        }"#;
        let style: Style = serde_json::from_str(json).unwrap();
        assert!(style.sources.contains_key("my-tiles"));
        matches!(style.sources["my-tiles"], Source::Vector(_));
    }

    #[test]
    fn test_parse_style_with_geojson_source() {
        let json = r#"{
            "version": 8,
            "sources": {
                "geo": {"type": "geojson", "data": null}
            },
            "layers": []
        }"#;
        let style: Style = serde_json::from_str(json).unwrap();
        assert!(matches!(style.sources["geo"], Source::GeoJson(_)));
    }

    #[test]
    fn test_parse_layer() {
        let json = r#"{
            "version": 8,
            "sources": {"roads": {"type": "vector", "url": "mapbox://foo"}},
            "layers": [{
                "id": "roads-line",
                "type": "line",
                "source": "roads",
                "source-layer": "road"
            }]
        }"#;
        let style: Style = serde_json::from_str(json).unwrap();
        assert_eq!(style.layers.len(), 1);
        assert_eq!(style.layers[0].id, "roads-line");
        assert_eq!(
            style.layers[0].layer_type,
            Some(crate::style::layer::LayerType::Line)
        );
    }

    #[test]
    fn test_parse_all_layer_types() {
        use crate::style::layer::LayerType;
        let types = [
            ("background", LayerType::Background),
            ("fill", LayerType::Fill),
            ("fill-extrusion", LayerType::FillExtrusion),
            ("line", LayerType::Line),
            ("symbol", LayerType::Symbol),
            ("raster", LayerType::Raster),
            ("circle", LayerType::Circle),
            ("heatmap", LayerType::Heatmap),
            ("hillshade", LayerType::Hillshade),
            ("sky", LayerType::Sky),
            ("color-relief", LayerType::ColorRelief),
        ];
        for (type_str, expected) in &types {
            let lt: LayerType = serde_json::from_value(serde_json::json!(type_str)).unwrap();
            assert_eq!(lt, *expected);
        }
    }
}
