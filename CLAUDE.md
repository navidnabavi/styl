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
cargo run -- lsp                   # language server over stdio
cargo build --no-default-features  # verify the crate builds without the lsp feature
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

**Data flow:** JSON → `serde_json::Value` → `Style` (typed structs) → validators/linters → `Vec<Diagnostic>` → span resolution → renderer → stdout.

### Core types

- `src/diagnostic/` — `Diagnostic { severity, code, path, message, hint, range }` + four renderers (`render_human`, `render_json`, `render_github`, `render_html`). All validators/linters produce `Vec<Diagnostic>`.
- `src/style/types.rs` — `Style` root struct + all `Source` variants (vector, raster, raster-dem, geojson, image, video). Uses `indexmap::IndexMap` for sources to preserve insertion order.
- `src/style/layer.rs` — `Layer` struct + `LayerType` enum (11 variants, `color-relief` is MapLibre-only). Paint/layout stored as `serde_json::Value` for flexible validation.
- `src/style/expression.rs` — `validate_expression(value, path, depth)` recursively validates expression operator arity and emits W006 at depth > 20.

### Span resolution

`src/span.rs::SourceMap` maps a diagnostic's JSON `path` back to a byte range in the original text, then to a zero-based line and UTF-16 column.

The scanner builds its keys with the **same string concatenation the validators use** (`.` for object keys, `[n]` for array indices), so lookup is an exact `HashMap` hit and no path parsing happens. This is what makes a source id containing a delimiter work: `openmaptiles.v3` yields `sources.openmaptiles.v3` on both sides and matches, where splitting on `.` would not.

- `resolve_or_parent` strips trailing path segments until something resolves, so rules reporting an *absent* key (E004's `layers[3].source`) land on the enclosing object with no per-code special casing.
- Parsing is tolerant and never panics — the language server indexes buffers mid-edit.
- `span::resolve_ranges(&mut diags, &map)` fills in `Diagnostic::range`. Rules never set it.
- `tests/span_test.rs::every_fixture_diagnostic_path_resolves_exactly` asserts every path any rule emits over `tests/fixtures/` resolves **exactly**. A new rule inventing a path shape the scanner cannot produce fails this test rather than silently degrading to a parent range. Codes that deliberately report an absence go in that test's `ABSENCE_CODES`.

### Language server

`src/lsp/` — `styl lsp` serves LSP over stdio from the same binary, so an editor and CI run identical analysis.

- `mod.rs` — `serve()`, `serve_connection()`, the message loop, handlers. **stdout is the JSON-RPC transport: nothing reachable from here may print to it.** All operator output goes to stderr. The library is clean of `print!` — keep it that way.
- `analysis.rs` — style detection, diagnostics, formatting, `.stylrc` resolution, `file:` URI decoding.
- Behind the default-on `lsp` Cargo feature, so library consumers can use `default-features = false`.
- Documents are keyed by **URI string**, not `Uri`: `lsp_types::Uri` hashes via `as_str()` but carries a `Cell` internally, which `clippy::mutable_key_type` rejects as a map key.
- `serve()` must `drop(connection)` before `io_threads.join()` — the writer thread lives until the last `Sender` drops, so joining first hangs forever. This only reproduces over real stdio, not `Connection::memory()`.
- Style detection is content-based (`version == 8` plus `layers` or `sources`) and **sticky** per document. Never key off the filename: `styles.json` design-token files are common, and attaching to all JSON would squiggle `package.json`.
- `tests/lsp_test.rs` drives the server over `Connection::memory()` — no process spawning.

### Validators (E-codes, spec violations)

`src/validator/mod.rs::run_all()` chains: root → sources → layers → refs → compat (sky, color-relief, terrain, fog, expression, mapbox-only-expression). Compat validators are filtered by `spec_affinity()` against the active `--spec`.

- `root.rs` — version==8, center/zoom/bearing/pitch ranges, glyphs placeholders
- `sources.rs` — required fields per source type (url or tiles)
- `layers.rs` — `valid_paint_props(LayerType)` and `valid_layout_props(LayerType)` hardcoded allowlists; source-layer required for vector sources
- `refs.rs` — source IDs exist in sources map, sprite non-empty

### Linter (W-codes, best practices)

`src/linter/mod.rs::run_all()` instantiates all 22 rules via `LintRule` trait.

Rules in `src/linter/rules/`: `duplicate_ids` (W001), `visibility` (W002), `unused_layers` (W003), `stop_order` (W004), `z_order` (W005), `expression_depth` (W006), `perf_hints` (W007–W022).

Config in `src/linter/config.rs` — TOML `.stylrc` auto-discovered by walking up the directory tree. Supports per-rule severity overrides (error/warn/off) and `format.indent`.

### Formatter

`src/formatter/key_order.rs` — canonical key order tables (`ROOT_KEY_ORDER`, `LAYER_KEY_ORDER`, `SOURCE_KEY_ORDER`).  
`src/formatter/normalizer.rs::format_style(value, indent)` — applies key ordering recursively: root → sources (Source context) → layers (Layer context) → paint/layout (alphabetical sort).

Requires `serde_json` `preserve_order` feature (in Cargo.toml) so `IndexMap`-backed ordering survives serialization.

### Validator/linter patterns

- **Untyped root fields**: `fog`, `light`, `terrain`, `transition` in `Style` are `Option<Value>` (not typed structs). Access via `.as_object()?.get("field")`. Skip validation when value is an expression array (first element is a string).
- **Human renderer is plain text**: `render_human` emits no ANSI color codes. Adding `--no-color` is pointless without first adding colors to the renderer.
- **Expression skip**: when validating a literal property value, skip if it's an array whose first element is a string — that's an expression. Check `arr.first().map(|v| v.is_string()).unwrap_or(false)`.
- **Fixable rules**: must be added to BOTH `run_all` and `run_fixes` in `src/linter/mod.rs`. `run_all` detects; `run_fixes` applies.
- **GeoJSON source** has no `minzoom` field (only `maxzoom`). Don't add minzoom validation there.
- **ref layers**: exempt from E004 (source required) and E019 (type required) — they inherit from parent.
- **Diagnostic paths must be real**: a path has to name a node that exists in the document, because `SourceMap` resolves it to a range. Do not append a segment the JSON does not have.
- **Spec precedence** is `--spec` flag → `.stylrc` `spec` → `Spec::Both`, resolved once in `main.rs` and identically in `lsp/analysis.rs`. `Cli::spec` is `Option<Spec>` precisely so an explicit flag is distinguishable from a default.

### Exit codes

`0` = clean, `1` = diagnostics found (any error or warning), `2` = tool error (bad JSON, I/O failure).

## Documentation

When fixing a known gap or adding a validator/linter rule, update the relevant doc in `docs/`:
- New E-code → `docs/validators.md` (add section, remove from Known Gaps)
- New W-code → `docs/linter.md`
- New autofix on a W-code → `docs/linter.md` (Autofix section)
- Formatter key order change → `docs/formatter.md`
- New LSP capability or setting → `docs/lsp.md`
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

Divergence between specs tracked in `src/style/spec.rs` (`MAPLIBRE_ONLY_*` / `MAPBOX_ONLY_*` constants). Wired into runtime via `src/validator/compat.rs` (E023 + compat validators), filtered by `SpecAffinity` against the active `--spec`.
