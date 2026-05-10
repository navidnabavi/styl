# Linter Rules (W-codes)

Linter rules enforce best practices that aren't spec violations. All rules use `W` codes and are emitted by `styl lint` and `styl check`.

Rules can be configured via [`.mapboxlintrc`](config.md).

---

## W001 — Duplicate layer IDs

Two or more layers share the same `id`. IDs must be unique.

```json
"layers": [
  { "id": "water", "type": "fill", ... },
  { "id": "water", "type": "line", ... }
]
```

---

## W002 — Hidden layer

A layer has `"visibility": "none"` in its `layout`. This may be intentional but is worth reviewing.

```json
"layout": { "visibility": "none" }
```

---

## W003 — Unused source

A source is defined in the top-level `sources` map but not referenced by any layer's `source` field.

---

## W004 — Stop order

Zoom stops or property stops are not in ascending order.

```json
"line-width": {
  "stops": [[10, 2], [5, 4]]
}
```

Stops must be in ascending order by the first value.

---

## W005 — Z-order issue

A `background` layer appears after a `fill-extrusion` layer in the layer stack. Background layers should come first.

---

## W006 — Deep expression nesting

An expression exceeds 10 levels of nesting. Deeply nested expressions are hard to maintain and may impact parse performance.

---

## W007 — Large GeoJSON

A GeoJSON source's `data` field contains inline data exceeding the recommended size. Consider hosting the data externally and referencing it by URL.

---

## W008 — High raster tile size

A raster source's `tileSize` is unusually large. Standard tile sizes are `256` or `512`.

---

## W009 — Duplicate source

Two or more sources have the same `url` or `tiles` values, meaning they fetch the same data. Consider deduplicating.

---

## W010 — Non-standard bearing

`bearing` is set to a non-zero value. This is valid but may surprise users if unintentional.

---

## W011 — Legacy filter syntax

A layer uses the legacy array filter syntax instead of the modern expression syntax.

**Legacy:**
```json
"filter": ["==", "class", "motorway"]
```

**Modern equivalent:**
```json
"filter": ["==", ["get", "class"], "motorway"]
```

Legacy filters are still supported by MapLibre/Mapbox but are deprecated.

---

## W012 — Excessive stops

A paint or layout property uses more than the recommended number of zoom stops. Consider using expressions instead.

---

## Configuring Rules

Severity can be overridden per-rule in `.mapboxlintrc`:

```toml
[rules]
W002 = "off"      # disable hidden-layer warnings
W011 = "error"    # treat legacy filters as errors
W007 = "warn"     # keep as warning (default)
```

Valid values: `"error"`, `"warn"`, `"off"`.

See [Configuration](config.md) for full details.
