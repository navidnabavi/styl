# CLI Reference

## Synopsis

```
styl <COMMAND> [OPTIONS] [FILE]
```

## Subcommands

### `check`

Run validators and linter together. Emits all E-codes and W-codes.

```bash
styl check style.json
styl check --format json style.json
styl check --format github style.json    # GitHub Actions annotations
styl check --format html style.json      # self-contained HTML report
styl check --spec mapbox style.json
styl check --stdin < style.json
styl check -q style.json                 # exit code only, no output
```

### `validate`

Run validators only. Emits E-codes (spec violations).

```bash
styl validate style.json
styl validate --format json style.json
```

### `lint`

Run linter only. Emits W-codes (best-practice warnings).

```bash
styl lint style.json
styl lint --spec mapbox style.json
```

### `fmt`

Format `FILE` in-place using canonical key ordering.

```bash
styl fmt style.json                 # format in-place
styl fmt --check style.json         # exit 1 if formatting would change (CI)
```

`--check` does not modify the file. Use it in CI to enforce consistent formatting.

### `lsp`

Run the language server over stdin/stdout.

```bash
styl lsp           # serve over stdio
styl lsp stdio     # explicit
styl lsp serve     # alias
styl lsp --stdio   # accepted for editor configs that pass it
```

Takes no input file, and ignores the global options — the server resolves its own settings from the editor and from `.stylrc`. See [Language Server](lsp.md).

## Global Options

### `--spec <SPEC>`

Which style spec to validate against.

| Value | Description |
|-------|-------------|
| `both` | Flag anything unsupported by either spec (default) |
| `maplibre` | MapLibre GL Style Spec v8 |
| `mapbox` | Mapbox GL Style Spec v8 |

When the flag is omitted, the `spec` key in [`.stylrc`](config.md) is used; with neither set, the default is `both`. An explicit flag always outranks the config file.

### `--format <FORMAT>`

Output format for diagnostics.

| Value | Description |
|-------|-------------|
| `human` | Colored, human-readable (default) |
| `json` | Machine-readable JSON array |
| `github` | GitHub Actions `::error` / `::warning` annotations, anchored to the offending line |
| `html` | Self-contained HTML report with dark theme and collapsible sections |

**HTML output** groups diagnostics by severity (errors → warnings → info), then by code. Each group is collapsible via native `<details>/<summary>`. No external dependencies — single file, inline CSS.

```bash
styl check style.json --format html > report.html
```

**JSON output shape:**
```json
[
  {
    "severity": "error",
    "code": "E006",
    "path": "layers[2].paint.line-width",
    "message": "\"line-width\" is not a valid paint property for \"fill\" layers",
    "hint": null,
    "range": {
      "start": { "line": 11, "character": 6 },
      "end": { "line": 11, "character": 27 }
    }
  }
]
```

`range` is present whenever the diagnostic's path could be resolved back to the source text. Lines and characters are **zero-based**, and characters are counted in UTF-16 code units, matching the Language Server Protocol. Human output renders the same position one-based as `file:line:column`.

### `--config <PATH>`

Explicit path to a `.stylrc` config file. When omitted, `styl` searches for `.stylrc` by walking up the directory tree from the style file's location.

### `--stdin`

Read the style JSON from stdin instead of a file. Cannot be combined with `fmt`.

```bash
cat style.json | styl check --stdin
curl https://example.com/style.json | styl check --stdin
```

### `-q, --quiet`

Suppress all output. Rely on exit codes only.

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Clean |
| `1` | Diagnostics emitted |
| `2` | Tool error (parse failure, I/O error, bad arguments) |
