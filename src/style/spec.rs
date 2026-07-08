/// Properties that exist in MapLibre but NOT in Mapbox (or vice-versa).
/// Used to emit warnings when --spec mapbox is set and a maplibre-only property is found.
/// Layer types only in MapLibre (not Mapbox)
pub const MAPLIBRE_ONLY_LAYER_TYPES: &[&str] = &["sky", "color-relief"];

/// Layer types only in Mapbox (not MapLibre)
pub const MAPBOX_ONLY_LAYER_TYPES: &[&str] = &[];

/// Expression operators only in MapLibre
pub const MAPLIBRE_ONLY_EXPRESSIONS: &[&str] = &["distance-from-center"];

/// Expression operators only in Mapbox
pub const MAPBOX_ONLY_EXPRESSIONS: &[&str] = &["hsl", "hsla"];

/// Root properties only in MapLibre
pub const MAPLIBRE_ONLY_ROOT_PROPS: &[&str] = &["terrain", "fog"];

/// Root properties only in Mapbox
pub const MAPBOX_ONLY_ROOT_PROPS: &[&str] = &[];

use crate::cli::Spec;

/// Declares which spec a feature belongs to exclusively.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SpecAffinity {
    /// Feature exists in MapLibre but not Mapbox
    MaplibreOnly,
    /// Feature exists in Mapbox but not MapLibre
    MapboxOnly,
}

impl SpecAffinity {
    /// Returns true when this feature's spec affinity is incompatible with the chosen spec.
    ///
    /// `Spec::Both` means "the style must work on both MapLibre and Mapbox," so any
    /// feature exclusive to one spec conflicts — it would break the other. Use this to
    /// decide whether to emit E023 for a compat validator.
    pub fn conflicts_with(&self, spec: &Spec) -> bool {
        matches!(
            (self, spec),
            (SpecAffinity::MaplibreOnly, Spec::Mapbox | Spec::Both)
                | (SpecAffinity::MapboxOnly, Spec::Maplibre | Spec::Both)
        )
    }
}
