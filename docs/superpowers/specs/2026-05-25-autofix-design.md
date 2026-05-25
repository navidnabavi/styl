# Autofix Design (`lint --fix`)

**Date:** 2026-05-25  
**Status:** Approved

## Summary

Add a `--fix` flag to the `lint` subcommand that automatically applies safe, mechanical fixes to a style JSON file in-place. Fixes only run when `--fix` is explicitly passed — never automatically.

## Scope

Five rules are fixable in this iteration (all safe/lossless):

| Code | Rule | Fix Applied |
|------|------|-------------|
| W004 | `StopOrder` | Sort stops arrays numerically ascending by first element |
| W007 | `EmptyTextField` | Remove the `text-field` key from layer layout |
| W015 | `BackgroundPatternOverridesColor` | Remove `background-color` from paint (no effect when pattern set) |
| W016 | `FillPatternOverridesColor` | Remove `fill-color` from paint |
| W017 | `LinePatternOverridesColor` | Remove `line-color` from paint |

## Trait Changes (`src/linter/mod.rs`)

Add two methods to `LintRule` with no-op defaults:

```rust
pub trait LintRule {
    fn code(&self) -> &'static str;
    fn spec_affinity(&self) -> Option<SpecAffinity> { None }
    fn check(&self, style: &Style) -> Vec<Diagnostic>;
    fn fix(&self, _value: &mut serde_json::Value) {}
    fn is_fixable(&self) -> bool { false }
}
```

- `is_fixable()` — rules that implement `fix` return `true`
- `fix()` — operates on the raw `serde_json::Value` (not the typed `Style` struct), to avoid deserialization roundtrip loss

## CLI Changes (`src/cli.rs`)

```rust
Command::Lint {
    file: Option<PathBuf>,
    #[arg(long)]
    fix: bool,
}
```

`--fix` and `--stdin` together: apply fixes, print result to stdout.  
`--fix` is incompatible with validator-only (`validate`) — only available on `lint`.

## Execution Flow (`src/main.rs`)

When `lint --fix file.json`:

1. Read file → parse `serde_json::Value` (keep raw value)
2. Parse `Style` from `value.clone()` → run all `check()` → collect diagnostics
3. For each rule where `is_fixable()`: call `rule.fix(&mut value)`
4. Write mutated `value` to file, formatted using current indent config (same serialization as `fmt`)
5. Print summary line: `fixed N issue(s) (W004, W015, ...)` — only if any fixable diagnostics were found
6. Print remaining diagnostics (from non-fixable rules)
7. Exit code: same logic as normal lint (1 if any error/warning remains, 0 if clean)

## Testing

Each fixable rule gets a unit test in its rule file:

```rust
#[test]
fn test_fix_stop_order() {
    let mut value = serde_json::json!({ /* out-of-order stops */ });
    StopOrder.fix(&mut value);
    // assert stops are now ascending
}
```

Integration test in `tests/pipeline_test.rs`:
- Fixture with W004 + W015 violations
- Run `lint --fix` → assert specific JSON paths are corrected
- Assert unfixable diagnostics still appear in output

## Out of Scope

The following rules were considered and excluded (require judgment or are destructive):

- W001 (duplicate IDs) — ambiguous which to keep
- W002 (visibility: none) — unclear intent (remove vs flip)
- W003 (unused source) — destructive delete
- W005 (z-order) — layer reorder changes rendering
- W006 (expression depth) — requires restructuring complex expressions
- W008 (placeholder icon) — can't know real sprite name
- W009 (layer count) — merging layers is non-mechanical
- W011 (legacy filter migration) — complex expression transform
- W012/W014/W018 (missing properties) — no safe default value to inject
- W013 (symbol no content) — no mechanical fix
