# Language Server

`styl lsp` runs a [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) server over stdin/stdout. It is part of the same binary as the CLI, so an editor and a CI pipeline run byte-identical analysis — a squiggle can never disagree with a pipeline failure.

```bash
styl lsp           # serve over stdio
styl lsp stdio     # explicit
styl lsp serve     # alias
styl lsp --stdio   # accepted for editor configs that pass it
```

All four are equivalent. The flag and the subcommand both exist because editor configurations commonly pass `--stdio` verbatim, and rejecting it would surface as a server that dies with no explanation.

## Capabilities

| Capability | Behavior |
|------------|----------|
| `textDocument/publishDiagnostics` | Validators (E-codes) and linter (W-codes) on open, save, and 300 ms after typing stops |
| `textDocument/formatting` | `styl fmt` as a single whole-document edit |
| `positionEncoding` | `utf-16` — every range is counted in UTF-16 code units |
| `textDocumentSync` | `Full` — style documents are small, and full sync removes a class of patch bugs |

Not yet implemented: completion, hover, code actions, go-to-definition. See [Roadmap](#roadmap).

## Which files it analyzes

The server attaches to JSON documents but only reports on those that look like GL styles:

```
version == 8  AND  (has "layers"  OR  has "sources")
```

The test is on content, not filename. A filename test would misfire on the `styles.json` design-token files common in web projects, and attaching to every `.json` file would put diagnostics on `package.json` and `tsconfig.json`.

Detection is **sticky per document**: once a buffer is recognized as a style, it keeps its diagnostics even while an in-progress edit leaves it unparseable. A new file therefore gets no diagnostics until it first becomes a valid style, after which it is analyzed continuously.

## Diagnostics

Each diagnostic carries:

- a **range**, resolved from the rule's JSON path back to the source text
- a **code** (`E009`, `W004`, …) with a `codeDescription` link to the relevant section of [validators.md](validators.md) or [linter.md](linter.md)
- a **source** of `styl`
- the rule's hint, appended to the message after a blank line

Two synthetic codes exist for failures that precede rule evaluation:

| Code | Meaning |
|------|---------|
| `syntax` | The buffer is not valid JSON. Positioned at the parse error. |
| `shape` | Valid JSON that does not deserialize into a style. Positioned at the document start. |

### Range precision

Ranges span the key through the end of the value, so a diagnostic covers the whole of `"fill-colour": "#fff"` whether the fault is in the property name or its value.

When a rule reports a key that is *absent* — `E004` reports `layers[3].source` for a layer with no `source` — the range falls back to the nearest enclosing node that does exist, which puts the diagnostic on the layer object.

Expression diagnostics are coarser. `validate_expression` carries one path for a whole expression tree, so an arity error nested deep inside an expression highlights the entire property value rather than the offending sub-expression.

## Configuration

Settings are read from `initializationOptions` and from `workspace/didChangeConfiguration`, accepted either bare or nested under a `styl` key:

```json
{ "styl": { "enable": true, "spec": "maplibre" } }
```

| Setting | Default | Description |
|---------|---------|-------------|
| `enable` | `true` | When false, the server clears all diagnostics and stops analysing. Setting it back to true republishes. |
| `spec` | — | `maplibre`, `mapbox`, or `both` |

Precedence is **editor settings → `.stylrc` → `both`**, matching how the CLI resolves `--spec`. The `.stylrc` governing a document is discovered by walking up from that document's directory, exactly as the CLI does. See [Configuration](config.md).

For formatting indentation, a `.stylrc` that exists outranks the editor's `tabSize`: a project carrying one has opted into styl's own formatting. With no `.stylrc`, the editor's `tabSize` is used, falling back to 2.

Unsaved and remote buffers (any URI that is not `file:`) get no `.stylrc`, and so use defaults.

## Editor setup

Any LSP-capable editor can drive it. The server needs no initialization options.

### Neovim

```lua
vim.lsp.config.styl = {
  cmd = { 'styl', 'lsp' },
  filetypes = { 'json' },
  root_markers = { '.stylrc', '.git' },
}
vim.lsp.enable('styl')
```

### Helix

```toml
# languages.toml
[language-server.styl]
command = "styl"
args = ["lsp"]

[[language]]
name = "json"
language-servers = ["json", "styl"]
```

VS Code and Zed extensions are planned; see [Roadmap](#roadmap).

## Troubleshooting

**No diagnostics appear.** The document probably does not match the detection rule above. Confirm it has `"version": 8` and a `layers` or `sources` key, and that `enable` is not set to false.

**The server dies immediately.** Check that the binary supports the subcommand — `styl lsp --help` should succeed. A build produced with `--no-default-features` omits the `lsp` feature and exits with code 2 and an explanatory message.

**Server logs.** Everything the server reports to the operator goes to stderr; stdout carries only JSON-RPC. In VS Code this surfaces in the extension's output channel, in Neovim via `:LspLog`.

## Roadmap

| Feature | Notes |
|---------|-------|
| Completion | Paint/layout properties narrowed by the enclosing layer's `type`, layer types, source IDs, expression operators. A plain JSON schema cannot narrow `paint` by a sibling key, which is the main reason to build this. |
| Hover | Rule documentation per code, property documentation from the spec allowlists. |
| Code actions | `source.fixAll.styl`, running the autofixable rules. Per-site quick fixes need a new per-path fix API: `linter::run_fixes` currently rewrites the whole document across all fixable rules and reformats it, which is wrong behavior for a single lightbulb. |
| `.stylrc` watching | `didChangeWatchedFiles` to re-lint open documents when config changes. |
| Go to definition | `layer.source` to its entry in `sources`, and find-references the other way. |
| VS Code extension | Thin `vscode-languageclient` wrapper, platform-specific VSIXs matching the release matrix. |
| Zed extension | WASM extension using `zed_extension_api`, fetching the binary from GitHub releases. |

## Embedding

`styl::lsp::serve()` runs the stdio server. `styl::lsp::serve_connection()` drives the loop over an already-established `lsp_server::Connection`, for tests or alternative transports.

The `lsp` Cargo feature is on by default. Library consumers who do not need it can drop both LSP dependencies:

```toml
styl = { version = "0.0.6", default-features = false }
```
