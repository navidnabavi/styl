//! A Language Server Protocol server, served over stdio.
//!
//! Lives in the same binary as the CLI so an editor and CI run byte-identical
//! analysis — a squiggle can never disagree with a pipeline failure. One rule
//! follows from that: **stdout is the JSON-RPC transport**, so nothing reachable
//! from here may print to it. Diagnostics for the operator go to stderr.

mod analysis;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::{Duration, Instant};

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        DidSaveTextDocument, Notification as _, PublishDiagnostics,
    },
    request::{Formatting, Request as _},
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, DocumentFormattingParams, OneOf,
    PositionEncodingKind, PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Uri,
};
use serde::Deserialize;

use crate::cli::Spec;
use crate::linter::config::Config;

type BoxError = Box<dyn Error + Send + Sync>;

/// How long typing must settle before a buffer is re-analysed.
const DEBOUNCE: Duration = Duration::from_millis(300);

/// Serve the language server over stdin/stdout until the client shuts it down.
pub fn serve() -> Result<(), BoxError> {
    let (connection, io_threads) = Connection::stdio();
    let initialize = connection.initialize(serde_json::to_value(capabilities())?)?;

    // Only the options are needed; deserializing all of `InitializeParams` would
    // reject clients sending shapes this version of `lsp-types` predates.
    let options = serde_json::from_value::<InitializeOptions>(initialize)
        .ok()
        .and_then(|params| params.initialization_options);

    serve_connection(&connection, options.as_ref())?;

    // The writer thread lives until the last `Sender` drops, so the connection
    // has to go before the join or it blocks forever.
    drop(connection);
    io_threads.join()?;
    Ok(())
}

/// Serve over an already-established connection, skipping the stdio handshake.
///
/// [`serve`] wraps this. Tests and alternative transports drive it directly.
pub fn serve_connection(
    connection: &Connection,
    initialization_options: Option<&serde_json::Value>,
) -> Result<(), BoxError> {
    let mut server = Server::new();
    server.apply_settings(initialization_options);
    server.main_loop(connection)
}

fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // Declared explicitly because every range this server produces is
        // counted in UTF-16 code units.
        position_encoding: Some(PositionEncodingKind::UTF16),
        // Full sync: style documents are small, and it removes a whole class of
        // incremental-patch bugs.
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..Default::default()
    }
}

#[derive(Deserialize)]
struct InitializeOptions {
    #[serde(default, rename = "initializationOptions")]
    initialization_options: Option<serde_json::Value>,
}

/// Editor-provided settings, which outrank `.stylrc`.
struct Settings {
    enabled: bool,
    spec: Option<Spec>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            spec: None,
        }
    }
}

struct Document {
    uri: Uri,
    text: String,
    /// Whether this buffer is a style. Sticky once true, so a recognized style
    /// keeps its diagnostics while it is mid-edit and momentarily unparseable.
    is_style: bool,
}

impl Document {
    fn new(uri: Uri, text: String) -> Self {
        Self {
            uri,
            is_style: analysis::looks_like_style(&text),
            text,
        }
    }

    fn update(&mut self, text: String) {
        self.is_style |= analysis::looks_like_style(&text);
        self.text = text;
    }
}

struct Server {
    /// Keyed by URI string. `Uri` itself hashes via `as_str()`, but it carries a
    /// `Cell` internally, which rules it out as a key under `clippy::all`.
    documents: HashMap<String, Document>,
    settings: Settings,
}

impl Server {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
            settings: Settings::default(),
        }
    }

    /// Read `enable` and `spec`, accepting them either bare or nested under a
    /// `styl` key, which is how VS Code delivers a configuration section.
    fn apply_settings(&mut self, value: Option<&serde_json::Value>) {
        let Some(value) = value else { return };
        let scope = value.get("styl").unwrap_or(value);

        if let Some(enabled) = scope.get("enable").and_then(serde_json::Value::as_bool) {
            self.settings.enabled = enabled;
        }
        if let Some(name) = scope.get("spec").and_then(serde_json::Value::as_str) {
            self.settings.spec = Config::parse_spec(name);
        }
    }

    fn main_loop(&mut self, connection: &Connection) -> Result<(), BoxError> {
        // Trailing debounce: `didChange` pushes the deadline out, and analysis
        // runs once typing pauses.
        let mut dirty: HashSet<String> = HashSet::new();
        let mut deadline: Option<Instant> = None;

        loop {
            let received = match deadline {
                Some(at) => {
                    match connection
                        .receiver
                        .recv_timeout(at.saturating_duration_since(Instant::now()))
                    {
                        Ok(message) => Some(message),
                        Err(error) if error.is_timeout() => None,
                        Err(_) => break,
                    }
                }
                None => match connection.receiver.recv() {
                    Ok(message) => Some(message),
                    Err(_) => break,
                },
            };

            let Some(message) = received else {
                deadline = None;
                for uri in std::mem::take(&mut dirty) {
                    self.publish(connection, &uri)?;
                }
                continue;
            };

            match message {
                Message::Request(request) => {
                    if connection.handle_shutdown(&request)? {
                        return Ok(());
                    }
                    self.handle_request(connection, request)?;
                }
                Message::Notification(notification) => {
                    self.handle_notification(connection, notification, &mut dirty, &mut deadline)?;
                }
                Message::Response(_) => {}
            }
        }

        Ok(())
    }

    fn handle_request(
        &mut self,
        connection: &Connection,
        request: Request,
    ) -> Result<(), BoxError> {
        if request.method == Formatting::METHOD {
            let (id, params) = match request.extract::<DocumentFormattingParams>(Formatting::METHOD)
            {
                Ok(extracted) => extracted,
                Err(error) => {
                    eprintln!("styl: malformed formatting request: {}", error);
                    return Ok(());
                }
            };
            let uri = params.text_document.uri;
            let edits = self
                .documents
                .get(uri.as_str())
                .filter(|document| document.is_style)
                .and_then(|document| {
                    analysis::format(&document.text, &uri, Some(params.options.tab_size))
                });
            let response = Response::new_ok(id, edits);
            connection.sender.send(Message::Response(response))?;
        }
        Ok(())
    }

    fn handle_notification(
        &mut self,
        connection: &Connection,
        notification: Notification,
        dirty: &mut HashSet<String>,
        deadline: &mut Option<Instant>,
    ) -> Result<(), BoxError> {
        match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params: DidOpenTextDocumentParams =
                    notification.extract(DidOpenTextDocument::METHOD)?;
                let uri = params.text_document.uri;
                let key = uri.as_str().to_string();
                self.documents
                    .insert(key.clone(), Document::new(uri, params.text_document.text));
                // Opening is a deliberate action, so report straight away.
                self.publish(connection, &key)?;
            }

            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams =
                    notification.extract(DidChangeTextDocument::METHOD)?;
                let uri = params.text_document.uri;
                let key = uri.as_str().to_string();
                // Full sync, so the final change carries the whole document.
                if let Some(change) = params.content_changes.into_iter().next_back() {
                    match self.documents.get_mut(&key) {
                        Some(document) => document.update(change.text),
                        None => {
                            self.documents
                                .insert(key.clone(), Document::new(uri, change.text));
                        }
                    }
                    dirty.insert(key);
                    *deadline = Some(Instant::now() + DEBOUNCE);
                }
            }

            DidSaveTextDocument::METHOD => {
                let params: DidSaveTextDocumentParams =
                    notification.extract(DidSaveTextDocument::METHOD)?;
                let key = params.text_document.uri.as_str().to_string();
                dirty.remove(&key);
                self.publish(connection, &key)?;
            }

            DidCloseTextDocument::METHOD => {
                let params: DidCloseTextDocumentParams =
                    notification.extract(DidCloseTextDocument::METHOD)?;
                let uri = params.text_document.uri;
                let key = uri.as_str().to_string();
                self.documents.remove(&key);
                dirty.remove(&key);
                // Clear the client's copy; it is no longer ours to own.
                self.send_diagnostics(connection, &uri, Vec::new())?;
            }

            DidChangeConfiguration::METHOD => {
                let params: DidChangeConfigurationParams =
                    notification.extract(DidChangeConfiguration::METHOD)?;
                self.apply_settings(Some(&params.settings));
                // Settings change what the rules report, so refresh everything.
                let open: Vec<String> = self.documents.keys().cloned().collect();
                for key in open {
                    self.publish(connection, &key)?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    fn publish(&self, connection: &Connection, key: &str) -> Result<(), BoxError> {
        let Some(document) = self.documents.get(key) else {
            return Ok(());
        };
        if !self.settings.enabled || !document.is_style {
            return Ok(());
        }
        let diagnostics =
            analysis::diagnose(&document.text, &document.uri, self.settings.spec.clone());
        self.send_diagnostics(connection, &document.uri, diagnostics)
    }

    fn send_diagnostics(
        &self,
        connection: &Connection,
        uri: &Uri,
        diagnostics: Vec<lsp_types::Diagnostic>,
    ) -> Result<(), BoxError> {
        let params = PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version: None,
        };
        connection
            .sender
            .send(Message::Notification(Notification {
                method: PublishDiagnostics::METHOD.to_string(),
                params: serde_json::to_value(params)?,
            }))
            .map_err(Into::into)
    }
}
