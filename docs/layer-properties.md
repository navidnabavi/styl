# Layer Properties

Valid `paint` and `layout` properties per layer type. Using any other property triggers [E006](validators.md#e006--invalid-paint-property) (paint) or [E007](validators.md#e007--invalid-layout-property) (layout).

All layer types support `"visibility"` in `layout`.

---

## background

**Paint:**
- `background-color`
- `background-pattern`
- `background-opacity`

**Layout:**
- `visibility`

---

## fill

**Paint:**
- `fill-antialias`
- `fill-opacity`
- `fill-color`
- `fill-outline-color`
- `fill-translate`
- `fill-translate-anchor`
- `fill-pattern`
- `fill-layer-opacity`

**Layout:**
- `visibility`
- `fill-sort-key`

---

## fill-extrusion

**Paint:**
- `fill-extrusion-opacity`
- `fill-extrusion-color`
- `fill-extrusion-translate`
- `fill-extrusion-translate-anchor`
- `fill-extrusion-pattern`
- `fill-extrusion-height`
- `fill-extrusion-base`
- `fill-extrusion-vertical-gradient`

**Layout:**
- `visibility`

---

## line

**Paint:**
- `line-opacity`
- `line-color`
- `line-translate`
- `line-translate-anchor`
- `line-width`
- `line-gap-width`
- `line-offset`
- `line-blur`
- `line-dasharray`
- `line-pattern`
- `line-gradient`
- `line-layer-opacity`

**Layout:**
- `visibility`
- `line-cap`
- `line-join`
- `line-miter-limit`
- `line-round-limit`
- `line-sort-key`

---

## symbol

**Paint:**
- `icon-opacity`
- `icon-color`
- `icon-halo-color`
- `icon-halo-width`
- `icon-halo-blur`
- `icon-translate`
- `icon-translate-anchor`
- `text-opacity`
- `text-color`
- `text-halo-color`
- `text-halo-width`
- `text-halo-blur`
- `text-translate`
- `text-translate-anchor`

**Layout:**
- `visibility`
- `symbol-placement`
- `symbol-spacing`
- `symbol-avoid-edges`
- `symbol-sort-key`
- `symbol-z-order`
- `icon-allow-overlap`
- `icon-ignore-placement`
- `icon-optional`
- `icon-rotation-alignment`
- `icon-size`
- `icon-text-fit`
- `icon-text-fit-padding`
- `icon-image`
- `icon-rotate`
- `icon-padding`
- `icon-keep-upright`
- `icon-offset`
- `icon-anchor`
- `icon-pitch-alignment`
- `icon-overlap`
- `text-pitch-alignment`
- `text-rotation-alignment`
- `text-field`
- `text-font`
- `text-size`
- `text-max-width`
- `text-line-height`
- `text-letter-spacing`
- `text-justify`
- `text-radial-offset`
- `text-variable-anchor`
- `text-variable-anchor-offset`
- `text-anchor`
- `text-max-angle`
- `text-writing-mode`
- `text-rotate`
- `text-padding`
- `text-keep-upright`
- `text-transform`
- `text-offset`
- `text-allow-overlap`
- `text-ignore-placement`
- `text-optional`
- `text-overlap`

---

## raster

**Paint:**
- `raster-opacity`
- `raster-hue-rotate`
- `raster-brightness-min`
- `raster-brightness-max`
- `raster-saturation`
- `raster-contrast`
- `raster-resampling`
- `raster-fade-duration`

**Layout:**
- `visibility`

---

## circle

**Paint:**
- `circle-radius`
- `circle-color`
- `circle-blur`
- `circle-opacity`
- `circle-translate`
- `circle-translate-anchor`
- `circle-pitch-scale`
- `circle-pitch-alignment`
- `circle-stroke-width`
- `circle-stroke-color`
- `circle-stroke-opacity`

**Layout:**
- `visibility`
- `circle-sort-key`

---

## heatmap

**Paint:**
- `heatmap-radius`
- `heatmap-weight`
- `heatmap-intensity`
- `heatmap-color`
- `heatmap-opacity`

**Layout:**
- `visibility`

---

## hillshade

**Paint:**
- `hillshade-illumination-direction`
- `hillshade-illumination-anchor`
- `hillshade-exaggeration`
- `hillshade-shadow-color`
- `hillshade-highlight-color`
- `hillshade-accent-color`
- `hillshade-illumination-altitude`
- `hillshade-method`
- `resampling`

**Layout:**
- `visibility`

---

## sky

**Paint:**
- `sky-type`
- `sky-atmosphere-sun`
- `sky-atmosphere-sun-intensity`
- `sky-gradient-center`
- `sky-gradient-radius`
- `sky-gradient`
- `sky-atmosphere-halo-color`
- `sky-atmosphere-color`
- `sky-opacity`

**Layout:**
- `visibility`

---

## color-relief

**Paint:**
- `color-relief-color`
- `color-relief-opacity`
- `resampling`

**Layout:**
- `visibility`
