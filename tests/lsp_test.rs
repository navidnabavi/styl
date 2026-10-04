//! End-to-end tests driving the language server over an in-memory connection.

#![cfg(feature = "lsp")]

use std::time::Duration;

use lsp_server::{Connection, Message, Notification, Request, RequestId};
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument,
        DidOpenTextDocument, Notification as _,
    },
    request::{Formatting, Request as _},
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
    FormattingOptions, NumberOrString, PublishDiagnosticsParams, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentItem, TextEdit, Uri, VersionedTextDocumentIdentifier,
};

/// Long enough to clear the server's 300ms change debounce.
const WAIT: Duration = Duration::from_secs(3);

const BROKEN: &str = "{\n  \"version\": 8,\n  \"sources\": {},\n  \"layers\": [\n    {\n      \"id\": \"roads\",\n      \"type\": \"line\",\n      \"source\": \"missing\"\n    }\n  ]\n}";

fn uri(raw: &str) -> Uri {
    raw.parse().expect("valid uri")
}

struct Harness {
    client: Connection,
    server: Option<std::thread::JoinHandle<()>>,
}

impl Harness {
    fn start() -> Self {
        let (server_connection, client) = Connection::memory();
        let server = std::thread::spawn(move || {
            styl::lsp::serve_connection(&server_connection, None).expect("server loop");
        });
        Self {
            client,
            server: Some(server),
        }
    }

    fn notify<P: serde::Serialize>(&self, method: &str, params: P) {
        let _ = self.client.sender.send(Message::Notification(Notification {
            method: method.to_string(),
            params: serde_json::to_value(params).expect("serialize params"),
        }));
    }

    fn request<P: serde::Serialize>(&self, id: i32, method: &str, params: P) {
        let _ = self.client.sender.send(Message::Request(Request {
            id: RequestId::from(id),
            method: method.to_string(),
            params: serde_json::to_value(params).expect("serialize params"),
        }));
    }

    fn open(&self, path: &str, text: &str) {
        self.notify(
            DidOpenTextDocument::METHOD,
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri(path),
                    language_id: "json".to_string(),
                    version: 1,
                    text: text.to_string(),
                },
            },
        );
    }

    fn change(&self, path: &str, version: i32, text: &str) {
        self.notify(
            DidChangeTextDocument::METHOD,
            DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri(path),
                    version,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: text.to_string(),
                }],
            },
        );
    }

    /// Next `publishDiagnostics`, or panic on timeout.
    fn diagnostics(&self) -> PublishDiagnosticsParams {
        loop {
            match self.client.receiver.recv_timeout(WAIT) {
                Ok(Message::Notification(notification))
                    if notification.method == "textDocument/publishDiagnostics" =>
                {
                    return serde_json::from_value(notification.params)
                        .expect("diagnostics params");
                }
                Ok(_) => continue,
                Err(e) => panic!("expected diagnostics, got {}", e),
            }
        }
    }

    /// Assert nothing at all arrives within the debounce window.
    fn expect_silence(&self) {
        match self
            .client
            .receiver
            .recv_timeout(Duration::from_millis(800))
        {
            Err(e) if e.is_timeout() => {}
            Ok(message) => panic!("expected silence, got {:?}", message),
            Err(e) => panic!("channel closed: {}", e),
        }
    }

    fn response(&self) -> serde_json::Value {
        loop {
            match self.client.receiver.recv_timeout(WAIT) {
                Ok(Message::Response(response)) => {
                    return response
                        .response_result
                        .unwrap_or_else(|e| panic!("error response: {:?}", e));
                }
                Ok(_) => continue,
                Err(e) => panic!("expected a response, got {}", e),
            }
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        // The real shutdown handshake, so the loop exits the way a client ends it.
        self.request(i32::MAX, "shutdown", serde_json::Value::Null);
        self.notify("exit", serde_json::Value::Null);
        if let Some(server) = self.server.take() {
            let _ = server.join();
        }
    }
}

fn codes(params: &PublishDiagnosticsParams) -> Vec<String> {
    params
        .diagnostics
        .iter()
        .filter_map(|d| match &d.code {
            Some(NumberOrString::String(code)) => Some(code.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn publishes_diagnostics_on_open_with_real_positions() {
    let harness = Harness::start();
    harness.open("file:///project/style.json", BROKEN);

    let params = harness.diagnostics();
    assert_eq!(params.uri, uri("file:///project/style.json"));

    let e009 = params
        .diagnostics
        .iter()
        .find(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "E009"))
        .expect("E009 for the undefined source");

    // `"source": "missing"` sits on line 8 (zero-based 7).
    assert_eq!(e009.range.start.line, 7);
    assert_eq!(e009.range.start.character, 6);
    assert_eq!(e009.source.as_deref(), Some("styl"));
    assert!(e009.message.contains("missing"));
}

#[test]
fn republishes_after_an_edit_clears_the_error() {
    let harness = Harness::start();
    harness.open("file:///project/style.json", BROKEN);
    assert!(codes(&harness.diagnostics()).contains(&"E009".to_string()));

    harness.change(
        "file:///project/style.json",
        2,
        &BROKEN.replace("\"sources\": {}", "\"sources\": {\"missing\": {\"type\": \"vector\", \"url\": \"https://example.com/t.json\"}}"),
    );

    assert!(
        !codes(&harness.diagnostics()).contains(&"E009".to_string()),
        "E009 should clear once the source exists"
    );
}

#[test]
fn reports_invalid_json_while_typing() {
    let harness = Harness::start();
    harness.open("file:///project/style.json", BROKEN);
    let _ = harness.diagnostics();

    harness.change(
        "file:///project/style.json",
        2,
        "{\n  \"version\": 8,\n  \"layers\": [",
    );

    let params = harness.diagnostics();
    assert_eq!(params.diagnostics.len(), 1);
    assert!(params.diagnostics[0].message.starts_with("invalid JSON"));
}

/// The detection gate: unrelated JSON must never be squiggled.
#[test]
fn stays_silent_on_json_that_is_not_a_style() {
    let harness = Harness::start();
    harness.open(
        "file:///project/package.json",
        r#"{"name": "app", "version": "1.0.0", "dependencies": {}}"#,
    );
    harness.expect_silence();
}

/// Once recognized, a style keeps its diagnostics through an unparseable edit.
#[test]
fn style_detection_is_sticky() {
    let harness = Harness::start();
    harness.open("file:///project/style.json", BROKEN);
    let _ = harness.diagnostics();

    // Momentarily not a style by content, but still analysed.
    harness.change("file:///project/style.json", 2, "{}");
    let params = harness.diagnostics();
    assert!(
        !params.diagnostics.is_empty(),
        "a recognized style should keep reporting"
    );
}

#[test]
fn formats_the_whole_document() {
    let harness = Harness::start();
    harness.open(
        "file:///project/style.json",
        r#"{"layers":[],"version":8,"sources":{}}"#,
    );
    let _ = harness.diagnostics();

    harness.request(
        1,
        Formatting::METHOD,
        DocumentFormattingParams {
            text_document: TextDocumentIdentifier {
                uri: uri("file:///project/style.json"),
            },
            options: FormattingOptions {
                tab_size: 2,
                insert_spaces: true,
                ..Default::default()
            },
            work_done_progress_params: Default::default(),
        },
    );

    let edits: Vec<TextEdit> = serde_json::from_value(harness.response()).expect("text edits");
    assert_eq!(edits.len(), 1);
    assert!(edits[0].new_text.starts_with("{\n  \"version\": 8"));
}

#[test]
fn clears_diagnostics_when_a_document_closes() {
    let harness = Harness::start();
    harness.open("file:///project/style.json", BROKEN);
    assert!(!harness.diagnostics().diagnostics.is_empty());

    harness.notify(
        DidCloseTextDocument::METHOD,
        DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier {
                uri: uri("file:///project/style.json"),
            },
        },
    );

    assert!(
        harness.diagnostics().diagnostics.is_empty(),
        "closing must clear the client's copy"
    );
}

/// Switching the server off has to clear what the editor already shows. Stopping
/// at "publish nothing" freezes stale diagnostics on screen until the file is
/// edited or closed.
#[test]
fn disabling_clears_diagnostics_instead_of_freezing_them() {
    let harness = Harness::start();
    harness.open("file:///project/style.json", BROKEN);
    assert!(!harness.diagnostics().diagnostics.is_empty());

    harness.notify(
        DidChangeConfiguration::METHOD,
        DidChangeConfigurationParams {
            settings: serde_json::json!({ "styl": { "enable": false } }),
        },
    );

    assert!(
        harness.diagnostics().diagnostics.is_empty(),
        "disabling must clear, not merely stop publishing"
    );

    harness.notify(
        DidChangeConfiguration::METHOD,
        DidChangeConfigurationParams {
            settings: serde_json::json!({ "styl": { "enable": true } }),
        },
    );
    assert!(
        !harness.diagnostics().diagnostics.is_empty(),
        "re-enabling must republish"
    );
}

/// `spec` from editor settings must change what the rules report.
#[test]
fn spec_from_settings_changes_what_is_reported() {
    const SKY: &str = "{\n  \"version\": 8,\n  \"sources\": {},\n  \"layers\": [\n    { \"id\": \"sky\", \"type\": \"sky\" }\n  ]\n}";

    let harness = Harness::start();
    harness.open("file:///project/style.json", SKY);
    // Default spec is `both`, which flags the MapLibre-only sky layer.
    assert!(codes(&harness.diagnostics()).contains(&"E023".to_string()));

    harness.notify(
        DidChangeConfiguration::METHOD,
        DidChangeConfigurationParams {
            settings: serde_json::json!({ "styl": { "spec": "maplibre" } }),
        },
    );
    assert!(
        !codes(&harness.diagnostics()).contains(&"E023".to_string()),
        "sky is valid under the MapLibre spec"
    );
}

/// A `.stylrc` edit must take effect without reopening the file. Config lookups
/// are cached to keep them off the keystroke path, so the watcher notification
/// is the only thing that can invalidate them.
#[test]
fn editing_stylrc_takes_effect_via_the_watcher() {
    const SKY: &str = "{\n  \"version\": 8,\n  \"sources\": {},\n  \"layers\": [\n    { \"id\": \"sky\", \"type\": \"sky\" }\n  ]\n}";

    let project = tempfile::tempdir().expect("tempdir");
    let config = project.path().join(".stylrc");
    std::fs::write(&config, "spec = \"maplibre\"\n").expect("write config");

    let style_path = project.path().join("style.json");
    let style_uri: Uri = format!("file://{}", style_path.display())
        .parse()
        .expect("uri");

    let harness = Harness::start();
    harness.notify(
        DidOpenTextDocument::METHOD,
        DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: style_uri.clone(),
                language_id: "json".to_string(),
                version: 1,
                text: SKY.to_string(),
            },
        },
    );
    // spec = maplibre, so the MapLibre-only sky layer is fine.
    assert!(
        !codes(&harness.diagnostics()).contains(&"E023".to_string()),
        "sky is valid under the configured maplibre spec"
    );

    std::fs::write(&config, "spec = \"mapbox\"\n").expect("rewrite config");
    harness.notify(
        DidChangeWatchedFiles::METHOD,
        DidChangeWatchedFilesParams { changes: vec![] },
    );

    assert!(
        codes(&harness.diagnostics()).contains(&"E023".to_string()),
        "the rewritten .stylrc should switch the spec to mapbox"
    );
}
