# Linter Rules (W-codes)

Linter rules enforce best practices that aren't spec violations. All rules use `W` codes and are emitted by `styl lint` and `styl check`.

Rules can be configured via [`.stylrc`](config.md).

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

A source is defined in the top-level `sources` map but not referenced by any layer's `source` field or by the root `terrain.source` field.

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

## W016 — Fill pattern overrides color

A fill layer sets both `fill-pattern` and `fill-color`. The pattern takes precedence; the color has no effect.

```json
"paint": { "fill-color": "#ffffff", "fill-pattern": "dots" }
```

Fix: remove `fill-color` when `fill-pattern` is set.

---

## W017 — Line pattern overrides color

A line layer sets both `line-pattern` and `line-color`. The pattern takes precedence; the color has no effect.

```json
"paint": { "line-color": "#ff0000", "line-pattern": "dash" }
```

Fix: remove `line-color` when `line-pattern` is set.

---

## W018 — Heatmap missing color ramp

A heatmap layer does not set `heatmap-color`. The layer renders monochrome using the renderer default.

Fix: set `heatmap-color` to an `interpolate` expression mapping density to a meaningful color ramp.

---

## W019 — Missing glyphs URL

Style contains symbol layers with `text-field` but no root `glyphs` property. Text rendering will fail at runtime because the renderer has no font source.

```json
{
  "layers": [{ "id": "labels", "type": "symbol", "layout": { "text-field": ["get", "name"] } }]
}
```

Fix: add a `glyphs` URL template such as `"https://demotiles.maplibre.org/font/{fontstack}/{range}.pbf"`.

---

## W020 — Fog missing color

`fog` is defined but `color` is not explicitly set. The renderer uses a default color which may not match the intended appearance.

```json
{ "fog": { "range": [0, 10] } }
```

Fix: add `"color"` to the `fog` object.

---

## W021 — Fog missing range

`fog` is defined but `range` is not explicitly set. The renderer uses a default range which may not match the scene scale.

```json
{ "fog": { "color": "white" } }
```

Fix: add `"range": [start_distance, end_distance]` to the `fog` object.

---

## W022 — icon-image used without sprite

A layer sets `icon-image` with a literal string value but no root `sprite` is defined. Icons will silently fail to render at runtime.

```json
{
  "layers": [{ "id": "poi", "type": "symbol", "layout": { "icon-image": "marker" } }]
}
```

Fix: add a `"sprite"` URL to the root of the style.

> Expression-based `icon-image` values (e.g. `["get", "icon"]`) do not trigger this warning.

---

## Configuring Rules

Severity can be overridden per-rule in `.stylrc`:

```toml
[rules]
W002 = "off"      # disable hidden-layer warnings
W011 = "error"    # treat legacy filters as errors
W013 = "error"    # treat empty symbol layers as errors
W019 = "error"    # treat missing glyphs as errors
```

Valid values: `"error"`, `"warn"`, `"off"`.

See [Configuration](config.md) for full details.

---

## Autofix

Run `styl lint --fix <file>` to automatically apply safe, mechanical fixes:

| Code | Fix Applied |
|------|-------------|
| W004 | Sort stop arrays into ascending order |
| W007 | Remove empty `text-field` from layer layout |
| W010 | Remove zero-length segments from `line-dasharray` |
| W011 | Migrate legacy filter to equivalent expression syntax |
| W015 | Remove `background-color` when `background-pattern` is set |
| W016 | Remove `fill-color` when `fill-pattern` is set |
| W017 | Remove `line-color` when `line-pattern` is set |

The file is rewritten in-place. Unfixed diagnostics (from rules with no autofix) are still reported.
When reading from `--stdin`, the fixed JSON is written to stdout instead.
