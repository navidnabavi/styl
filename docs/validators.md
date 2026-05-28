# Validators (E-codes)

Validators check conformance with the MapLibre / Mapbox GL Style Spec. All validator diagnostics use `E` codes and are emitted by `styl validate` and `styl check`.

---

## E001 — Invalid version

Style `version` must equal `8`.

```json
{ "version": 7 }
```

---

## E002 — Invalid root property value

Out-of-range values on root properties:

| Property | Valid range |
|----------|-------------|
| `center[0]` (longitude) | `-180` to `180` |
| `center[1]` (latitude) | `-90` to `90` |
| `zoom` | `0` to `24` |
| `pitch` | `0` to `85` |

`bearing` is not range-validated (the spec normalizes any value).

---

## E003 — Invalid glyphs URL

`glyphs` must be a URL template containing both `{fontstack}` and `{range}` placeholders.

```json
{ "glyphs": "https://example.com/fonts/{fontstack}/{range}.pbf" }
```

---

## E004 — Missing source

Non-background, non-sky layers must have a `source` field (unless they use `ref`).

Applies to: `fill`, `fill-extrusion`, `line`, `symbol`, `raster`, `circle`, `heatmap`, `hillshade`, `color-relief`.

---

## E005 — Missing source-layer

Layers that reference a vector source must include `source-layer`.

```json
{
  "sources": { "osm": { "type": "vector", "url": "..." } },
  "layers": [{
    "id": "roads",
    "type": "line",
    "source": "osm"
  }]
}
```

Fix: add `"source-layer": "<layer-name-in-tile>"`.

---

## E006 — Invalid paint property

A paint property is not valid for the layer's type. See [Layer Properties](layer-properties.md) for valid properties per type.

```json
{
  "type": "fill",
  "paint": { "line-width": 2 }
}
```

---

## E007 — Invalid layout property

A layout property is not valid for the layer's type. See [Layer Properties](layer-properties.md) for valid properties per type.

```json
{
  "type": "fill",
  "layout": { "text-field": "hello" }
}
```

---

## E008 — Invalid sprite

`sprite` must be a non-empty URL string, or a non-empty array of `{id, url}` entries where both `id` and `url` are non-empty strings.

```json
{ "sprite": "" }
{ "sprite": [] }
{ "sprite": [{"id": "", "url": "https://example.com/sprite"}] }
```

---

## E009 — Layer source not found

A layer references a `source` ID that doesn't exist in the top-level `sources` map.

```json
{
  "sources": {},
  "layers": [{ "id": "roads", "type": "line", "source": "osm" }]
}
```

---

## E010 — Missing source tiles/url

Vector, raster, and raster-dem sources must have either a `url` field or a non-empty `tiles` array.

```json
{ "type": "vector" }
```

---

## E011 — Invalid source scheme

Source `scheme` must be `"xyz"` or `"tms"`. Applies to vector and raster sources.

```json
{ "type": "vector", "url": "...", "scheme": "bad" }
```

---

## E012 — Invalid raster-dem encoding

`raster-dem` source `encoding` must be `"terrarium"`, `"mapbox"`, or `"none"`.

```json
{ "type": "raster-dem", "url": "...", "encoding": "bad" }
```

---

## E013 — Invalid source zoom range

Source `minzoom`/`maxzoom` must be in `[0, 22]`. `minzoom` must be ≤ `maxzoom`. Applies to vector, raster, and raster-dem sources. GeoJSON sources only support `maxzoom`.

```json
{ "type": "vector", "url": "...", "minzoom": 25 }
```

---

## E014 — Invalid layer zoom range

Layer `minzoom`/`maxzoom` must be in `[0, 24]`. `minzoom` must be ≤ `maxzoom`.

```json
{ "id": "roads", "type": "line", "source": "s", "minzoom": 10, "maxzoom": 5 }
```

---

## E015 — Invalid source bounds

Source `bounds` must be `[sw_lng, sw_lat, ne_lng, ne_lat]` where longitudes are in `[-180, 180]`, latitudes in `[-90, 90]`, and sw ≤ ne. Applies to vector, raster, and raster-dem sources.

```json
{ "type": "vector", "url": "...", "bounds": [-200, -90, 180, 90] }
```

---

## E016 — Missing GeoJSON data

GeoJSON sources must have a `data` field set to a GeoJSON object (inline) or a URL string. Absent or `null` values are not valid.

```json
{ "type": "geojson" }
{ "type": "geojson", "data": null }
```

Valid forms:
```json
{ "type": "geojson", "data": "https://example.com/data.geojson" }
{ "type": "geojson", "data": { "type": "FeatureCollection", "features": [] } }
```

---

## E017 — Invalid inline GeoJSON

When `data` is an inline JSON object, it must be a valid GeoJSON object per RFC 7946 (FeatureCollection, Feature, or Geometry).

```json
{ "type": "geojson", "data": { "type": "NotAGeoJsonType" } }
```

URL strings are not validated at lint time (remote resource).

---

## E018 — Invalid paint/layout property value

A paint or layout property has a literal value that does not match the MapLibre v8 spec type, range, or enum.

| Situation | Example message |
|-----------|----------------|
| Number out of range | `"fill-opacity" expects number in [0, 1], got 1.5` |
| Number below minimum | `"line-width" expects number >= 0, got -1` |
| Invalid enum value | `"line-cap" expects one of "butt", "round", or "square", got "flat"` |
| Invalid color string | `"fill-color" invalid color: "notacolor"` |
| Wrong type for color | `"fill-color" expects a CSS color string, got number` |
| Wrong type for boolean | `"fill-antialias" expects boolean, got string` |

Expression values (arrays starting with an operator string) are validated structurally via E020–E022 instead.

---

## E019 — Missing layer type

A layer does not have a `type` field and does not use `ref` to inherit one from a parent layer.

```json
{ "id": "roads" }
```

Fix: add `"type"` with one of: `background`, `fill`, `fill-extrusion`, `line`, `symbol`, `raster`, `circle`, `heatmap`, `hillshade`, `sky`.

---

## E020 — Empty expression array

An expression must be a non-empty array. `[]` is not valid.

---

## E021 — Invalid expression argument count

An expression operator received the wrong number of arguments.

- `"case"` requires pairs of `[condition, output]` plus a final fallback (minimum 4 args, even count)
- `"match"` requires an input, at least one label-output pair, and a fallback (minimum 5 args)
- Other operators (e.g. `"rgb"`, `"hsl"`, `"at"`, `"slice"`) enforce their own fixed or minimum arity

```json
["case", true, "red"]
```

→ `E021`: `"case"` requires pairs of [condition, output] plus a fallback

---

## E022 — Unknown expression operator

An expression's first element is not a recognized operator. See [Expressions](expressions.md) for all supported operators.

---

## E023 — Spec incompatibility

A feature in the style is not supported by the target spec (`--spec mapbox` or `--spec both`).

| Feature | MapLibre | Mapbox |
|---------|----------|--------|
| `sky` layer type | Yes | No |
| `terrain` root property | Yes | No |
| `fog` root property | Yes | No |
| `distance-from-center` expression | Yes | No |

**Example** — sky layer with `--spec mapbox`:

```json
{
  "layers": [{ "id": "s", "type": "sky" }]
}
```

**Fix:** remove the incompatible feature, or switch to `--spec maplibre` if targeting MapLibre only.

---

## E024 — Empty layer id

A layer has an empty string `""` for its `id`. Every layer must have a unique, non-empty id.

```json
{ "id": "", "type": "background" }
```

---

## E025 — Invalid terrain

The root `terrain` object has a structural error.

| Situation | Example message |
|-----------|----------------|
| Missing `source` | `terrain must have a "source" referencing a raster-dem source` |
| `source` not a string | `terrain.source must be a string` |
| `exaggeration` < 0 | `terrain.exaggeration must be >= 0, got -1` |
| `exaggeration` wrong type | `terrain.exaggeration must be a number` |

```json
{ "terrain": { "exaggeration": 1.5 } }
```

Fix: add `"source"` referencing a `raster-dem` source id.

---

## E026 — Invalid image/video source coordinates

An `image` or `video` source has a coordinate outside valid bounds (longitude in `[-180, 180]`, latitude in `[-90, 90]`).

```json
{
  "type": "image",
  "url": "...",
  "coordinates": [[-200, 37], [-80, 36], [-79, 36], [-79, 37]]
}
```

---

## E027 — Invalid light property

The root `light` object has a property with an invalid value.

| Property | Valid values |
|----------|-------------|
| `anchor` | `"map"` or `"viewport"` |
| `intensity` | number in `[0, 1]` |
| `color` | CSS color string |
| `position` | array of exactly 3 numbers `[radial, azimuthal, polar]` |

---

## E028 — Invalid transition property

`transition.duration` or `transition.delay` is negative or not a number. Both must be non-negative integers (milliseconds).

```json
{ "transition": { "duration": -100 } }
```
