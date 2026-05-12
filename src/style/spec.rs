/// Properties that exist in MapLibre but NOT in Mapbox (or vice-versa).
/// Used to emit warnings when --spec mapbox is set and a maplibre-only property is found.
/// Layer types only in MapLibre (not Mapbox)
pub const MAPLIBRE_ONLY_LAYER_TYPES: &[&str] = &["sky"];

/// Layer types only in Mapbox (not MapLibre)
pub const MAPBOX_ONLY_LAYER_TYPES: &[&str] = &[];

/// Expression operators only in MapLibre
pub const MAPLIBRE_ONLY_EXPRESSIONS: &[&str] = &["distance-from-center"];

/// Expression operators only in Mapbox
pub const MAPBOX_ONLY_EXPRESSIONS: &[&str] = &[];

/// Root properties only in MapLibre
pub const MAPLIBRE_ONLY_ROOT_PROPS: &[&str] = &["terrain", "fog"];

/// Root properties only in Mapbox
pub const MAPBOX_ONLY_ROOT_PROPS: &[&str] = &[];
