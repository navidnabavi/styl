# Configuration

`styl` reads a TOML config file named `.stylrc`. When `--config` is not specified, `styl` searches for `.stylrc` by walking up the directory tree from the style file's location.

## Example

```toml
[rules]
W002 = "off"
W011 = "error"
W003 = "warn"

[format]
indent = 4
```

## Auto-Discovery

Starting from the directory containing the style file, `styl` walks up to the filesystem root looking for `.stylrc`. The first file found is used.

To use an explicit config path:

```bash
styl check --config /path/to/.stylrc style.json
```

## `[rules]`

Override severity for individual linter rules.

```toml
[rules]
W001 = "error"    # treat duplicate IDs as errors
W002 = "off"      # disable hidden-layer warnings
W011 = "warn"     # keep legacy filter warnings (default behavior)
```

Valid values:

| Value | Behavior |
|-------|---------|
| `"error"` | Emit as error (contributes to exit code 1) |
| `"warn"` | Emit as warning (contributes to exit code 1) |
| `"off"` | Suppress rule entirely |

Rule codes: `W001` through `W012`. See [Linter Rules](linter.md) for descriptions.

When no `.stylrc` is present (or a rule has no override), all rules default to `"warn"`.

## `[format]`

Formatter settings used by `styl fmt`.

```toml
[format]
indent = 2    # spaces (default: 2)
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `indent` | integer | `2` | Number of spaces for JSON indentation |
