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

## E003 — Layer source not found

> Note: Two distinct checks share the E003 code — one on the root `glyphs` field (above), one on layer `source` references (below).

A layer references a `source` ID that doesn't exist in the top-level `sources` map.

```json
{
  "sources": {},
  "layers": [{ "id": "roads", "type": "line", "source": "osm" }]
}
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

## E008 — Empty sprite

`sprite` must be a non-empty string (or array form — see Known Gaps below).

---

## E010 — Missing source tiles/url

Vector, raster, and raster-dem sources must have either a `url` field or a non-empty `tiles` array.

```json
{ "type": "vector" }
```

---

## E020 — Empty expression array

An expression must be a non-empty array. `[]` is not valid.

---

## E022 — Unknown expression operator

An expression's first element is not a recognized operator. See [Expressions](expressions.md) for all supported operators.

---

## Known Gaps

- `sprite` array form `[{id, url}, ...]` not yet validated (only string form)
- Source `scheme` (`xyz`|`tms`) and raster-dem `encoding` not enum-validated
- Source/layer `minzoom`/`maxzoom` ranges not validated
- Source `bounds` array not validated
- GeoJSON `data` field not required by validator
- Filter expressions not validated (only heuristically detected by W011)
