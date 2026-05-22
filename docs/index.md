# styl

`styl` is a CLI linter, validator, and formatter for Mapbox GL / MapLibre GL style JSON files.

## Commands

| Command | Purpose |
|---------|---------|
| `styl check <file>` | Run validators + linter (all diagnostics) |
| `styl validate <file>` | Validators only — spec violations (E-codes) |
| `styl lint <file>` | Linter only — best-practice warnings (W-codes) |
| `styl fmt <file>` | Format file in-place (canonical key order) |

## Quick Start

```bash
styl check style.json
styl check --format json style.json
styl check --format html style.json > report.html
styl fmt --check style.json      # CI mode: exit 1 if formatting would change
styl lint --spec mapbox style.json
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Clean — no diagnostics |
| `1` | Diagnostics found (any error or warning) |
| `2` | Tool error (bad JSON, I/O failure, invalid arguments) |

## Global Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--spec maplibre\|mapbox` | `maplibre` | Style spec to validate against |
| `--format human\|json\|github\|html` | `human` | Output format |
| `--config <path>` | auto-discover | Path to `.stylrc` config file |
| `--stdin` | — | Read style from stdin instead of a file |
| `-q, --quiet` | — | Suppress output; use exit code only |

## Documentation

- [CLI Reference](cli.md) — full flag and subcommand reference
- [Validators](validators.md) — E-code spec violations
- [Linter Rules](linter.md) — W-code best-practice warnings
- [Expressions](expressions.md) — supported expression operators
- [Formatter](formatter.md) — canonical key ordering
- [Configuration](config.md) — `.stylrc` config file
- [Layer Properties](layer-properties.md) — valid paint/layout props per layer type
