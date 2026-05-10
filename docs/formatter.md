# Formatter

`styl fmt` formats a style JSON file in-place using canonical key ordering.

## Usage

```bash
styl fmt style.json              # format in-place
styl fmt --check style.json      # CI check — exit 1 if formatting would change
```

`--check` never modifies the file. Use it in CI pipelines:

```yaml
- run: styl fmt --check style.json
```

## What Gets Formatted

### Root Key Order

```
version → name → metadata → center → zoom → bearing → pitch →
light → terrain → fog → sprite → glyphs → transition →
sources → layers
```

Unknown root keys are appended alphabetically after the known keys.

### Layer Key Order

```
id → type → metadata → ref → source → source-layer →
minzoom → maxzoom → filter → layout → paint
```

### Source Key Order

```
type → url → tiles → bounds → scheme → minzoom → maxzoom →
attribution → promoteId
```

### Paint and Layout Properties

Within `paint` and `layout` objects, properties are sorted **alphabetically**.

## Indentation

Default indent is 2 spaces. Configure with [`format.indent`](config.md) in `.mapboxlintrc`.

## Idempotency

Running `styl fmt` twice produces the same result. If `--check` passes once, it will always pass until the file is modified.

## Limitations

- Does not reformat expression values (preserved as-is)
- Does not sort layer array or sources map entries (insertion order preserved)
- `tileSize` is not yet in the source key order (appears in the unknown-key alphabetical section)
