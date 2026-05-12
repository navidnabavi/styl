use crate::diagnostic::Diagnostic;
use crate::style::{
    types::{GeoJsonSource, RasterDemSource, RasterSource, Source, VectorSource},
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
            Source::GeoJson(s) => diags.extend(validate_geojson(s, &path)),
            Source::Image(_) | Source::Video(_) => {
                // These sources have no additional required fields beyond type
            }
        }
    }

    diags
}

fn validate_bounds(bounds: &[f64; 4], path: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let [sw_lng, sw_lat, ne_lng, ne_lat] = *bounds;
    if !(-180.0..=180.0).contains(&sw_lng) {
        diags.push(Diagnostic::error(
            "E015",
            format!("{}.bounds[0]", path),
            format!("sw longitude {} is out of range [-180, 180]", sw_lng),
        ));
    }
    if !(-90.0..=90.0).contains(&sw_lat) {
        diags.push(Diagnostic::error(
            "E015",
            format!("{}.bounds[1]", path),
            format!("sw latitude {} is out of range [-90, 90]", sw_lat),
        ));
    }
    if !(-180.0..=180.0).contains(&ne_lng) {
        diags.push(Diagnostic::error(
            "E015",
            format!("{}.bounds[2]", path),
            format!("ne longitude {} is out of range [-180, 180]", ne_lng),
        ));
    }
    if !(-90.0..=90.0).contains(&ne_lat) {
        diags.push(Diagnostic::error(
            "E015",
            format!("{}.bounds[3]", path),
            format!("ne latitude {} is out of range [-90, 90]", ne_lat),
        ));
    }
    if sw_lng > ne_lng {
        diags.push(Diagnostic::error(
            "E015",
            format!("{}.bounds", path),
            format!(
                "sw longitude ({}) must be <= ne longitude ({})",
                sw_lng, ne_lng
            ),
        ));
    }
    if sw_lat > ne_lat {
        diags.push(Diagnostic::error(
            "E015",
            format!("{}.bounds", path),
            format!(
                "sw latitude ({}) must be <= ne latitude ({})",
                sw_lat, ne_lat
            ),
        ));
    }
    diags
}

fn validate_zoom_range(
    minzoom: Option<f64>,
    maxzoom: Option<f64>,
    max_allowed: f64,
    path: &str,
    code: &'static str,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if let Some(z) = minzoom {
        if !(0.0..=max_allowed).contains(&z) {
            diags.push(Diagnostic::error(
                code,
                format!("{}.minzoom", path),
                format!("minzoom {} is out of range [0, {}]", z, max_allowed),
            ));
        }
    }
    if let Some(z) = maxzoom {
        if !(0.0..=max_allowed).contains(&z) {
            diags.push(Diagnostic::error(
                code,
                format!("{}.maxzoom", path),
                format!("maxzoom {} is out of range [0, {}]", z, max_allowed),
            ));
        }
    }
    if let (Some(min), Some(max)) = (minzoom, maxzoom) {
        if min > max {
            diags.push(Diagnostic::error(
                code,
                format!("{}.minzoom", path),
                format!("minzoom ({}) must be <= maxzoom ({})", min, max),
            ));
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
            diags.push(Diagnostic::error(
                "E010",
                format!("{}.tiles", path),
                "\"tiles\" array must not be empty",
            ));
        }
    }
    if let Some(scheme) = &s.scheme {
        if scheme != "xyz" && scheme != "tms" {
            diags.push(
                Diagnostic::error(
                    "E011",
                    format!("{}.scheme", path),
                    format!("scheme must be \"xyz\" or \"tms\", got \"{}\"", scheme),
                )
                .with_hint("use \"xyz\" (default) or \"tms\""),
            );
        }
    }
    if let Some(bounds) = &s.bounds {
        diags.extend(validate_bounds(bounds, path));
    }
    diags.extend(validate_zoom_range(
        s.minzoom, s.maxzoom, 22.0, path, "E013",
    ));
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
    if let Some(scheme) = &s.scheme {
        if scheme != "xyz" && scheme != "tms" {
            diags.push(
                Diagnostic::error(
                    "E011",
                    format!("{}.scheme", path),
                    format!("scheme must be \"xyz\" or \"tms\", got \"{}\"", scheme),
                )
                .with_hint("use \"xyz\" (default) or \"tms\""),
            );
        }
    }
    if let Some(bounds) = &s.bounds {
        diags.extend(validate_bounds(bounds, path));
    }
    diags.extend(validate_zoom_range(
        s.minzoom, s.maxzoom, 22.0, path, "E013",
    ));
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
    if let Some(encoding) = &s.encoding {
        if encoding != "terrarium" && encoding != "mapbox" && encoding != "none" {
            diags.push(
                Diagnostic::error(
                    "E012",
                    format!("{}.encoding", path),
                    format!(
                        "encoding must be \"terrarium\", \"mapbox\", or \"none\", got \"{}\"",
                        encoding
                    ),
                )
                .with_hint("use \"terrarium\" (Mapzen), \"mapbox\", or \"none\""),
            );
        }
    }
    if let Some(bounds) = &s.bounds {
        diags.extend(validate_bounds(bounds, path));
    }
    diags.extend(validate_zoom_range(
        s.minzoom, s.maxzoom, 22.0, path, "E013",
    ));
    diags
}

fn validate_geojson(s: &GeoJsonSource, path: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    match &s.data {
        None => {
            diags.push(
                Diagnostic::error(
                    "E016",
                    format!("{}.data", path),
                    "geojson source must have a \"data\" field",
                )
                .with_hint("set \"data\" to a GeoJSON object or a URL string"),
            );
        }
        Some(value) if value.is_object() => {
            // Inline GeoJSON object — validate structure via the geojson crate
            match value.to_string().parse::<geojson::GeoJson>() {
                Err(e) => {
                    diags.push(
                        Diagnostic::error(
                            "E017",
                            format!("{}.data", path),
                            format!("invalid GeoJSON: {}", e),
                        )
                        .with_hint("data must be a valid GeoJSON object (RFC 7946)"),
                    );
                }
                Ok(_) => {}
            }
        }
        Some(_) => {
            // String URL — cannot validate contents at lint time
        }
    }
    diags.extend(validate_zoom_range(None, s.maxzoom, 22.0, path, "E013"));
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
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://foo"}},"layers":[]}"#,
        );
        assert!(validate_sources(&style).is_empty());
    }

    #[test]
    fn test_vector_source_with_tiles() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","tiles":["https://a.tiles"]}},"layers":[]}"#,
        );
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
    fn test_geojson_source_null_data_invalid() {
        // "data": null and absent data both deserialize to None; neither is a valid GeoJSON data value
        let style =
            parse(r#"{"version":8,"sources":{"s":{"type":"geojson","data":null}},"layers":[]}"#);
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E016"));
    }

    #[test]
    fn test_geojson_source_with_url_data() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":"https://example.com/data.geojson"}},"layers":[]}"#,
        );
        assert!(validate_sources(&style).is_empty());
    }

    #[test]
    fn test_geojson_source_with_inline_object() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":{"type":"FeatureCollection","features":[]}}},"layers":[]}"#,
        );
        assert!(validate_sources(&style).is_empty());
    }

    #[test]
    fn test_geojson_source_invalid_inline_object() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":{"type":"NotAGeoJsonType"}}},"layers":[]}"#,
        );
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E017"));
    }

    #[test]
    fn test_geojson_source_valid_feature() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"geojson","data":{"type":"Feature","geometry":{"type":"Point","coordinates":[0,0]},"properties":{}}}},"layers":[]}"#,
        );
        assert!(validate_sources(&style).is_empty());
    }

    #[test]
    fn test_geojson_source_missing_data() {
        let style = parse(r#"{"version":8,"sources":{"s":{"type":"geojson"}},"layers":[]}"#);
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E016"));
    }

    #[test]
    fn test_vector_source_valid_scheme() {
        for scheme in &["xyz", "tms"] {
            let style = parse(&format!(
                r#"{{"version":8,"sources":{{"s":{{"type":"vector","url":"mapbox://foo","scheme":"{}"}}}},"layers":[]}}"#,
                scheme
            ));
            assert!(
                validate_sources(&style).is_empty(),
                "scheme {} should be valid",
                scheme
            );
        }
    }

    #[test]
    fn test_vector_source_invalid_scheme() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://foo","scheme":"bad"}},"layers":[]}"#,
        );
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E011"));
    }

    #[test]
    fn test_raster_source_invalid_scheme() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"raster","url":"https://example.com","scheme":"bad"}},"layers":[]}"#,
        );
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E011"));
    }

    #[test]
    fn test_raster_dem_valid_encoding() {
        for enc in &["terrarium", "mapbox", "none"] {
            let style = parse(&format!(
                r#"{{"version":8,"sources":{{"s":{{"type":"raster-dem","url":"https://example.com","encoding":"{}"}}}},"layers":[]}}"#,
                enc
            ));
            assert!(
                validate_sources(&style).is_empty(),
                "encoding {} should be valid",
                enc
            );
        }
    }

    #[test]
    fn test_raster_dem_invalid_encoding() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"raster-dem","url":"https://example.com","encoding":"bad"}},"layers":[]}"#,
        );
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E012"));
    }

    #[test]
    fn test_vector_source_invalid_minzoom() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://foo","minzoom":25}},"layers":[]}"#,
        );
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E013"));
    }

    #[test]
    fn test_source_minzoom_gt_maxzoom() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://foo","minzoom":10,"maxzoom":5}},"layers":[]}"#,
        );
        let diags = validate_sources(&style);
        assert!(diags.iter().any(|d| d.code == "E013"));
    }

    #[test]
    fn test_source_valid_zoom_range() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://foo","minzoom":0,"maxzoom":22}},"layers":[]}"#,
        );
        assert!(validate_sources(&style).is_empty());
    }

    #[test]
    fn test_bounds_valid() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://foo","bounds":[-180,-90,180,90]}},"layers":[]}"#,
        );
        assert!(validate_sources(&style).is_empty());
    }

    #[test]
    fn test_bounds_invalid_lng() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://foo","bounds":[-200,-90,180,90]}},"layers":[]}"#,
        );
        let diags = validate_sources(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E015" && d.path.contains("bounds[0]")));
    }

    #[test]
    fn test_bounds_sw_gt_ne() {
        let style = parse(
            r#"{"version":8,"sources":{"s":{"type":"vector","url":"mapbox://foo","bounds":[10,-90,-10,90]}},"layers":[]}"#,
        );
        let diags = validate_sources(&style);
        assert!(diags
            .iter()
            .any(|d| d.code == "E015" && d.path.contains("bounds")));
    }
}
