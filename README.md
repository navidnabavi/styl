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
styl check --format json style.json   # machine-readable output
styl check --format github style.json # GitHub Actions annotations
cat style.json | styl check --stdin   # read from stdin
```

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | No diagnostics |
| `1` | Diagnostics found (any error or warning) |
| `2` | Tool error (bad JSON, I/O failure) |

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

## Supported Spec

- **MapLibre GL Style Spec v8** (default) — [maplibre.org/maplibre-style-spec](https://maplibre.org/maplibre-style-spec/)
- **Mapbox GL Style Spec v8** — pass `--spec mapbox`

---

## Development

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
cargo run -- check style.json
```
