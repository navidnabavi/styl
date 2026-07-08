# Expression Operators

`styl` validates MapLibre GL expression syntax. Expressions are arrays of the form `["operator", arg1, arg2, ...]`.

Deeply nested expressions (> 10 levels) trigger [W006](linter.md#w006--deep-expression-nesting).

---

## Comparison

| Operator | Args | Description |
|----------|------|-------------|
| `==` | 2 | Equal |
| `!=` | 2 | Not equal |
| `>` | 2 | Greater than |
| `>=` | 2 | Greater than or equal |
| `<` | 2 | Less than |
| `<=` | 2 | Less than or equal |

---

## Logic

| Operator | Args | Description |
|----------|------|-------------|
| `all` | 1+ | Logical AND |
| `any` | 1+ | Logical OR |
| `none` | 1+ | Logical NOR _(legacy filter syntax — deprecated; use `!` + `any` in expressions)_ |
| `!` | 1 | Logical NOT |
| `case` | 3+ | If/elif/else — condition, result pairs + fallback |
| `match` | 4+ | Switch — input, label, output pairs + fallback |
| `coalesce` | 1+ | First non-null value |

---

## Arithmetic

| Operator | Args | Description |
|----------|------|-------------|
| `+` | variadic | Addition |
| `*` | variadic | Multiplication |
| `-` | 1–2 | Subtraction (or unary negation with 1 arg) |
| `/` | 2 | Division |
| `%` | 2 | Modulo |
| `^` | 2 | Exponentiation |
| `abs` | 1 | Absolute value |
| `ceil` | 1 | Ceiling |
| `floor` | 1 | Floor |
| `round` | 1 | Round |
| `sqrt` | 1 | Square root |
| `log2` | 1 | Base-2 logarithm |
| `log10` | 1 | Base-10 logarithm |
| `ln` | 1 | Natural logarithm |
| `ln2` | 0 | Constant: ln(2) |
| `pi` | 0 | Constant: π |
| `e` | 0 | Constant: Euler's number |
| `min` | 1+ | Minimum value |
| `max` | 1+ | Maximum value |
| `random` | 2 | Random number in range |
| `distance` | 1 | Distance to geometry |

---

## String

| Operator | Args | Description |
|----------|------|-------------|
| `concat` | 1+ | String concatenation |
| `downcase` | 1 | Lowercase string |
| `upcase` | 1 | Uppercase string |
| `string` | 1–2 | Assert/convert to string |
| `split` | 2 | Split string by separator |
| `join` | 2 | Join array elements with separator |
| `number-format` | 2 | Format number as string |
| `is-supported-script` | 1 | Check script support |
| `resolved-locale` | 1 | Resolved locale from collator |

---

## Data Access

| Operator | Args | Description |
|----------|------|-------------|
| `get` | 1–2 | Feature property value |
| `has` | 1–2 | Feature property exists |
| `at` | 2 | Array element by index |
| `in` | 2 | Value in array/string |
| `index-of` | 2–3 | Index of value in array/string |
| `slice` | 2–3 | Array/string slice |
| `length` | 1 | Array or string length |
| `global-state` | 1 | Global state property value |
| `properties` | 0 | All feature properties |
| `feature-state` | 1 | Feature state value |
| `geometry-type` | 0 | Feature geometry type |
| `id` | 0 | Feature ID |
| `accumulated` | 0 | Cluster accumulated value |

---

## Type

| Operator | Args | Description |
|----------|------|-------------|
| `typeof` | 1 | Type name as string |
| `boolean` | 1–2 | Assert/convert to boolean |
| `number` | 1–2 | Assert/convert to number |
| `object` | 1–2 | Assert/convert to object |
| `array` | 1–3 | Assert/convert to array |
| `to-boolean` | 1 | Convert to boolean |
| `to-number` | 1+ | Convert to number |
| `to-string` | 1 | Convert to string |
| `to-color` | 1+ | Convert to color |
| `to-rgba` | 1 | Convert color to RGBA array |

---

## Color

| Operator | Args | Description |
|----------|------|-------------|
| `rgb` | 3 | Create color from RGB values (0–255) |
| `rgba` | 4 | Create color from RGBA values |
| `hsl` | 3 | Create color from HSL values _(Mapbox-only — triggers E023 under `--spec maplibre`/`--spec both`)_ |
| `hsla` | 4 | Create color from HSLA values _(Mapbox-only — triggers E023 under `--spec maplibre`/`--spec both`)_ |

---

## Interpolation and Steps

| Operator | Args | Description |
|----------|------|-------------|
| `interpolate` | 3+ | Smooth interpolation between stops |
| `interpolate-hcl` | 3+ | Interpolate in HCL color space |
| `interpolate-lab` | 3+ | Interpolate in Lab color space |
| `step` | 3+ | Stepped output between stops |

**Interpolation types** (used as second arg to `interpolate`):

| Type | Args | Description |
|------|------|-------------|
| `linear` | 0 | Linear interpolation |
| `exponential` | 1 | Exponential interpolation (base) |
| `cubic-bezier` | 4 | Cubic Bézier curve |

---

## Camera and Zoom

| Operator | Args | Description |
|----------|------|-------------|
| `zoom` | 0 | Current zoom level |
| `pitch` | 0 | Current map pitch |
| `distance-from-center` | 0 | Distance from map center |
| `heatmap-density` | 0 | Kernel density estimation (heatmap layers only) |
| `elevation` | 0 | Elevation from raster-dem (color-relief layers only) |

---

## Lookup

| Operator | Args | Description |
|----------|------|-------------|
| `let` | 3+ | Bind variable(s) |
| `var` | 1 | Reference bound variable |
| `literal` | 1 | Literal value |
| `config` | 1 | Config option |

---

## Image and Collator

| Operator | Args | Description |
|----------|------|-------------|
| `image` | 1 | Image reference |
| `collator` | 1 | Locale-aware string collator |
| `format` | 1+ | Rich text formatting |

---

## Error Codes

| Code | Meaning |
|------|---------|
| E020 | Empty expression array `[]` |
| E022 | Unknown operator name |
| W006 | Expression nesting depth > 10 |
