# Contributing

## Prerequisites

- Rust (stable, via [rustup](https://rustup.rs/))
- `cargo clippy` and `cargo fmt` (included with Rust toolchain)

## Development Setup

```bash
git clone <repo>
cd styl
cargo build
cargo test
```

## Before Submitting

Run all checks and fix any issues:

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings
```

All four must pass cleanly.

## Adding a Validator (E-code)

1. Add rule in `src/validator/` and call it from `run_all()` in `src/validator/mod.rs`
2. Add entry to `docs/validators.md`

## Adding a Linter Rule (W-code)

1. Implement `LintRule` trait in `src/linter/rules/`
2. Register in `src/linter/mod.rs::run_all()`
3. Add entry to `docs/linter.md`

## Spec References

- MapLibre v8 (primary): https://maplibre.org/maplibre-style-spec/
- v8 JSON schema: https://github.com/maplibre/maplibre-style-spec/blob/main/src/reference/v8.json
- Mapbox (secondary): https://docs.mapbox.com/mapbox-gl-js/style-spec/
