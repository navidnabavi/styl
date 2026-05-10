# styl

A fast, opinionated linter, validator, and formatter for **Mapbox GL** and **MapLibre GL** style JSON files.

```
$ styl check style.json
warning[W002] layers[9].layout.visibility: layer "Building" is permanently invisible
  --> style.json
  hint: remove the layer or set visibility to "visible" if it should be shown

warning[W011] layers[3].filter: layer "Landuse" uses deprecated legacy filter syntax
  --> style.json
  hint: migrate to expression-based filters: https://maplibre.org/maplibre-style-spec/expressions/
```

---

## Features

- **Validator** — catches spec violations (missing required fields, invalid property values, bad source references)
- **Linter** — flags best-practice issues (duplicate IDs, invisible layers, legacy filter syntax, deep expressions, performance anti-patterns)
- **Formatter** — rewrites style JSON with canonical key ordering; `--check` mode for CI enforcement
- **Three output formats** — human-readable, JSON (for tooling), GitHub Actions annotations
- **Config file** — per-project `.mapboxlintrc` with per-rule severity overrides and format settings
- **Spec switching** — `--spec maplibre` (default) or `--spec mapbox`

---

## Installation

```bash
cargo install --path .
```

Or build locally:

```bash
cargo build --release
# binary at: target/release/styl
```

---

## Usage

```bash
styl check style.json            # validators + linter
styl validate style.json         # spec violations only (E-codes)
styl lint style.json             # best-practice warnings (W-codes)
styl fmt style.json              # format in-place
styl fmt --check style.json      # CI: exit 1 if formatting would change
```

Read from stdin (`check`, `validate`, `lint` only — not compatible with `fmt`):

```bash
cat style.json | styl check --stdin
curl https://example.com/style.json | styl lint --stdin
```

Machine-readable output:

```bash
styl check --format json style.json
styl check --format github style.json   # GitHub Actions annotations
```

Suppress output, use exit code only:

```bash
styl check -q style.json && echo "clean"
```

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | No diagnostics |
| `1` | Diagnostics found (any error or warning) |
| `2` | Tool error (bad JSON, I/O failure) |

---

## Validators (E-codes)

Validators check conformance with the style spec. Run with `styl validate`.

| Code | Description |
|------|-------------|
| E001 | `version` must be `8` |
| E002 | `center`, `zoom`, `pitch` out of range |
| E003 | `glyphs` missing `{fontstack}` or `{range}` placeholder |
| E003 | Layer `source` references an ID not in `sources` |
| E004 | Non-background layer missing `source` |
| E005 | Vector source layer missing `source-layer` |
| E006 | Invalid `paint` property for layer type |
| E007 | Invalid `layout` property for layer type |
| E008 | `sprite` is empty |
| E010 | Vector/raster source missing both `url` and `tiles` |
| E020 | Empty expression array `[]` |
| E022 | Unknown expression operator |

→ [Full validator reference](docs/validators.md)

---

## Linter Rules (W-codes)

Linter rules enforce best practices. Run with `styl lint`.

| Code | Description |
|------|-------------|
| W001 | Duplicate layer IDs |
| W002 | Layer permanently invisible (`"visibility": "none"`) |
| W003 | Unused source (defined but not referenced by any layer) |
| W004 | Zoom/property stops not in ascending order |
| W005 | Background layer appears after fill-extrusion |
| W006 | Expression nesting depth > 10 |
| W007 | Large inline GeoJSON data |
| W008 | Unusually large raster `tileSize` |
| W009 | Duplicate source (same URL/tiles) |
| W010 | Non-zero `bearing` on root |
| W011 | Legacy filter syntax (deprecated) |
| W012 | Excessive zoom stops — use expressions instead |

→ [Full linter reference](docs/linter.md)

---

## Formatter

`styl fmt` rewrites style JSON with canonical key ordering:

**Root:** `version` → `name` → `metadata` → `center` → `zoom` → `bearing` → `pitch` → `light` → `terrain` → `fog` → `sprite` → `glyphs` → `transition` → `sources` → `layers`

**Layers:** `id` → `type` → `metadata` → `ref` → `source` → `source-layer` → `minzoom` → `maxzoom` → `filter` → `layout` → `paint`

**Paint/layout:** alphabetical

Use `--check` in CI to enforce consistent formatting without modifying files:

```yaml
- name: Check style formatting
  run: styl fmt --check style.json
```

→ [Formatter reference](docs/formatter.md)

---

## Configuration

Create `.mapboxlintrc` in your project root (auto-discovered by walking up the directory tree):

```toml
[rules]
W002 = "off"      # traffic layers are intentionally hidden at load time
W011 = "error"    # treat legacy filters as errors in this project

[format]
indent = 4
```

Rule severities: `"error"` | `"warn"` | `"off"`. Both `"error"` and `"warn"` produce exit code 1 — the distinction is only in the diagnostic label emitted. Use `"off"` to suppress a rule entirely. All rules default to `"warn"` when no config is present.

→ [Full config reference](docs/config.md)

---

## Output Formats

### Human (default)

```
error[E005] layers[2].source-layer: layer "roads" uses a vector source but has no "source-layer"
  --> style.json
  hint: add "source-layer" matching the layer name in the vector tile
```

### JSON (`--format json`)

```json
[
  {
    "severity": "error",
    "code": "E005",
    "path": "layers[2].source-layer",
    "message": "layer \"roads\" uses a vector source but has no \"source-layer\"",
    "hint": "add \"source-layer\" matching the layer name in the vector tile"
  }
]
```

### GitHub Actions (`--format github`)

```
::error file=style.json,title=E005::layers[2].source-layer: layer "roads" uses a vector source but has no "source-layer"
```

---

## Supported Spec

- **MapLibre GL Style Spec v8** (default) — [maplibre.org/maplibre-style-spec](https://maplibre.org/maplibre-style-spec/)
- **Mapbox GL Style Spec v8** — pass `--spec mapbox`

Spec-level divergences are tracked in `src/style/spec.rs`.

---

## Documentation

| Document | Description |
|----------|-------------|
| [CLI Reference](docs/cli.md) | All subcommands and flags |
| [Validators](docs/validators.md) | E-code spec violations |
| [Linter Rules](docs/linter.md) | W-code best-practice warnings |
| [Expressions](docs/expressions.md) | All supported expression operators |
| [Formatter](docs/formatter.md) | Key ordering and `--check` mode |
| [Configuration](docs/config.md) | `.mapboxlintrc` reference |
| [Layer Properties](docs/layer-properties.md) | Valid paint/layout props per layer type |

---

## Architecture

Dual-crate layout: `src/lib.rs` exposes the public API as crate `styl`; `src/main.rs` is the CLI binary.

**Data flow:**
```
JSON → serde_json::Value → Style (typed structs) → validators/linters → Vec<Diagnostic> → renderer → stdout
```

- `src/diagnostic.rs` — `Diagnostic` type and three renderers
- `src/style/` — `Style`, `Layer`, `Source` types; expression validator
- `src/validator/` — spec validators (E-codes), chained via `run_all()`
- `src/linter/` — 12 lint rules implementing `LintRule` trait, config discovery
- `src/formatter/` — canonical key ordering, in-place formatting

---

## Development

```bash
cargo build                          # compile
cargo test                           # all tests
cargo test --test pipeline_test      # integration tests only
cargo test validator::root           # specific module
cargo run -- check style.json        # run against a file
cargo run -- fmt --check style.json  # CI format check
```
