use crate::diagnostic::Diagnostic;
use crate::style::{
    types::{RasterDemSource, RasterSource, Source, VectorSource},
    Style,
};

pub fn validate_sources(style: &Style) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (id, source) in &style.sources {
        let path = format!("sources.{}", id);
        match source {
            Source::Vector(s) => diags.extend(validate_vector(s, &path)),
            Source::Raster(s) => diags.extend(validate_raster(s, &path)),
            Source::RasterDem(s) => diags.extend(validate_raster_dem(s, &path)),
            Source::GeoJson(_) | Source::Image(_) | Source::Video(_) => {
                // These sources have no additional required fields beyond type
            }
        }
    }

    diags
}

fn validate_vector(s: &VectorSource, path: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if s.url.is_none() && s.tiles.is_none() {
        diags.push(
            Diagnostic::error(
                "E010",
                path,
                "vector source must have either \"url\" or \"tiles\"",
            )
            .with_hint("add a \"url\" (TileJSON endpoint) or \"tiles\" array"),
        );
    }
    if let Some(tiles) = &s.tiles {
        if tiles.is_empty() {
            diags.push(Diagnostic::error("E010", format!("{}.tiles", path), "\"tiles\" array must not be empty"));
        }
    }
    diags
}

fn validate_raster(s: &RasterSource, path: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if s.url.is_none() && s.tiles.is_none() {
        diags.push(
            Diagnostic::error(
                "E010",
                path,
                "raster source must have either \"url\" or \"tiles\"",
            )
            .with_hint("add a \"url\" (TileJSON endpoint) or \"tiles\" array"),
        );
    }
    diags
}

fn validate_raster_dem(s: &RasterDemSource, path: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if s.url.is_none() && s.tiles.is_none() {
        diags.push(
            Diagnostic::error(
                "E010",
                path,
                "raster-dem source must have either \"url\" or \"tiles\"",
            )
            .with_hint("add a \"url\" (TileJSON endpoint) or \"tiles\" array"),
        );
    }
    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> crate::style::Style {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_vector_source_with_url() {
        let style = parse(r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://foo"}},"layers":[]}"#);
        assert!(validate_sources(&style).is_empty());
    }

    #[test]
    fn test_vector_source_with_tiles() {
        let style = parse(r#"{"version":8,"sources":{"s":{"type":"vector","tiles":["https://a.tiles"]}},"layers":[]}"#);
        assert!(validate_sources(&style).is_empty());
    }

    #[test]
    fn test_vector_source_missing_url_and_tiles() {
        let style = parse(r#"{"version":8,"sources":{"s":{"type":"vector"}},"layers":[]}"#);
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E010"));
    }

    #[test]
    fn test_raster_source_missing_url_and_tiles() {
        let style = parse(r#"{"version":8,"sources":{"s":{"type":"raster"}},"layers":[]}"#);
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E010"));
    }

    #[test]
    fn test_geojson_source_valid() {
        let style = parse(r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[]}"#);
        assert!(validate_sources(&style).is_empty());
    }
}
