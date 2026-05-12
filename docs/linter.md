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

A layer has `"visibility": "none"` in its `layout`. May be intentional but worth reviewing.

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

## W007 — Empty text-field

A symbol layer has `text-field` set to an empty string `""`. The layer renders no text.

```json
"layout": { "text-field": "" }
```

Fix: remove `text-field` or provide a value such as `["get", "name"]`.

---

## W008 — Placeholder icon-image

A symbol layer's `icon-image` looks like a placeholder name (e.g. `TODO`, `my-icon`, `placeholder`).

```json
"layout": { "icon-image": "my-icon-24" }
```

Fix: replace with the actual sprite name from your sprite sheet.

---

## W009 — Layer count

The style has more than 200 layers. Consider merging layers for better performance.

---

## W010 — Zero dasharray segment

`line-dasharray` contains a zero-length segment. All segments must be `> 0`.

```json
"paint": { "line-dasharray": [2, 0, 2] }
```

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

## W012 — Raster resampling not set

A raster layer does not set `raster-resampling`. The default may cause blur when overzooming.

```json
"paint": { "raster-resampling": "nearest" }
```

---

## W013 — Symbol layer renders nothing

A symbol layer has neither `text-field` nor `icon-image`. The layer renders nothing.

```json
{ "id": "labels", "type": "symbol", "source": "s", "source-layer": "x" }
```

Fix: add `text-field` or `icon-image` to the layer's `layout`.

---

## W014 — Symbol missing text-font

A symbol layer uses `text-field` without setting `text-font`. The renderer falls back to a default font which may differ across platforms.

```json
"layout": { "text-field": ["get", "name"] }
```

Fix: add `text-font` with an explicit font stack.

---

## W015 — Background pattern overrides color

A background layer sets both `background-pattern` and `background-color`. The pattern takes precedence; the color has no effect.

```json
"paint": {
  "background-color": "#ffffff",
  "background-pattern": "dots"
}
```

Fix: remove `background-color` when `background-pattern` is set.

---

## Configuring Rules

Severity can be overridden per-rule in `.mapboxlintrc`:

```toml
[rules]
W002 = "off"      # disable hidden-layer warnings
W011 = "error"    # treat legacy filters as errors
W013 = "error"    # treat empty symbol layers as errors
```

Valid values: `"error"`, `"warn"`, `"off"`.

See [Configuration](config.md) for full details.
