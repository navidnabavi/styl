# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                        # compile
cargo test                         # all tests (unit + integration, 4 suites)
cargo test --test pipeline_test    # integration tests only
cargo test validator::root         # specific module tests
cargo run -- check style.json      # validate + lint a file
cargo run -- validate style.json   # validators only (E-codes)
cargo run -- lint style.json       # linter only (W-codes)
cargo run -- lint --fix style.json    # autofix safe issues in-place (W004, W007, W010, W011, W015-W017)
cargo run -- fmt style.json        # format in-place
cargo run -- fmt --check style.json  # CI check (exit 1 if would change)
cargo run -- check --format json style.json  # machine-readable output
```

## Development Flow

**Before every commit, without exception, run all four checks and fix any failures:**

```bash
cargo build                  # must compile clean
cargo test                   # all 4 suites must pass
cargo fmt --check            # zero formatting diffs (run cargo fmt to fix)
cargo clippy -- -D warnings  # zero errors (same strictness as CI)
```

Never commit if any of these fail. No exceptions for "just a docs change" or "just a refactor".

Documents must be updated after any change in rules (format, validate and lint)

## Architecture

Dual crate: `src/lib.rs` exposes the public API as `styl`; `src/main.rs` is the CLI binary (`styl`).

**Data flow:** JSON → `serde_json::Value` → `Style` (typed structs) → validators/linters → `Vec<Diagnostic>` → renderer → stdout.

### Core types

- `src/diagnostic.rs` — `Diagnostic { severity, code, path, message, hint }` + three renderers (`render_human`, `render_json`, `render_github`). All validators/linters produce `Vec<Diagnostic>`.
- `src/style/types.rs` — `Style` root struct + all `Source` variants (vector, raster, raster-dem, geojson, image, video). Uses `indexmap::IndexMap` for sources to preserve insertion order.
- `src/style/layer.rs` — `Layer` struct + `LayerType` enum (10 variants). Paint/layout stored as `serde_json::Value` for flexible validation.
- `src/style/expression.rs` — `validate_expression(value, path, depth)` recursively validates expression operator arity and emits W006 at depth > 10.

### Validators (E-codes, spec violations)

`src/validator/mod.rs::run_all()` chains: root → sources → layers → refs.

- `root.rs` — version==8, center/zoom/bearing/pitch ranges, glyphs placeholders
- `sources.rs` — required fields per source type (url or tiles)
- `layers.rs` — `valid_paint_props(LayerType)` and `valid_layout_props(LayerType)` hardcoded allowlists; source-layer required for vector sources
- `refs.rs` — source IDs exist in sources map, sprite non-empty

### Linter (W-codes, best practices)

`src/linter/mod.rs::run_all()` instantiates all 19 rules via `LintRule` trait.

Rules in `src/linter/rules/`: `duplicate_ids` (W001), `visibility` (W002), `unused_layers` (W003), `stop_order` (W004), `z_order` (W005), `expression_depth` (W006), `perf_hints` (W007–W019).

Config in `src/linter/config.rs` — TOML `.stylrc` auto-discovered by walking up the directory tree. Supports per-rule severity overrides (error/warn/off) and `format.indent`.

### Formatter

`src/formatter/key_order.rs` — canonical key order tables (`ROOT_KEY_ORDER`, `LAYER_KEY_ORDER`, `SOURCE_KEY_ORDER`).  
`src/formatter/normalizer.rs::format_style(value, indent)` — applies key ordering recursively: root → sources (Source context) → layers (Layer context) → paint/layout (alphabetical sort).

Requires `serde_json` `preserve_order` feature (in Cargo.toml) so `IndexMap`-backed ordering survives serialization.

### Validator/linter patterns

- **Expression skip**: when validating a literal property value, skip if it's an array whose first element is a string — that's an expression. Check `arr.first().map(|v| v.is_string()).unwrap_or(false)`.
- **Fixable rules**: must be added to BOTH `run_all` and `run_fixes` in `src/linter/mod.rs`. `run_all` detects; `run_fixes` applies.
- **GeoJSON source** has no `minzoom` field (only `maxzoom`). Don't add minzoom validation there.
- **ref layers**: exempt from E004 (source required) and E019 (type required) — they inherit from parent.

### Exit codes

`0` = clean, `1` = diagnostics found (any error or warning), `2` = tool error (bad JSON, I/O failure).

## Documentation

When fixing a known gap or adding a validator/linter rule, update the relevant doc in `docs/`:
- New E-code → `docs/validators.md` (add section, remove from Known Gaps)
- New W-code → `docs/linter.md`
- New autofix on a W-code → `docs/linter.md` (Autofix section)
- Formatter key order change → `docs/formatter.md`
- After fixing a gap → remove it from Known Gaps in both `CLAUDE.md` and `docs/validators.md`

## Publishing

- Homebrew tap: `github.com/navidnabavi/homebrew-tap` — formula at `Formula/styl.rb`
- Release workflow generates per-binary `.sha256` files — use when updating formula SHA256s

## Release Workflow

1. Bump version in `Cargo.toml`
2. `git tag vX.Y.Z && git push https vX.Y.Z` — triggers release CI
3. CI uploads binaries + `.sha256` files to GitHub release
4. Update `Formula/styl.rb` in `homebrew-tap` repo: bump `version`, fetch new SHA256 via `curl -fsSL <release-url>.sha256`

## Assets

- `assets/overview.svg` — project infographic, embedded in README
- `install.sh` — cross-platform installer, auto-detects OS/arch, fetches latest release

## Spec references

- MapLibre v8 (primary): https://maplibre.org/maplibre-style-spec/
- v8 JSON schema (authoritative property lists): https://github.com/maplibre/maplibre-style-spec/blob/main/src/reference/v8.json
- Mapbox (secondary, `--spec mapbox`): https://docs.mapbox.com/mapbox-gl-js/style-spec/

Divergence between specs tracked in `src/style/spec.rs` (constants only — not yet wired into runtime checks).
