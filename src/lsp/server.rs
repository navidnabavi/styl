//! Server state and the message loop.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use lsp_server::{Connection, Message, Notification};
use lsp_types::{
    notification::{Notification as _, PublishDiagnostics},
    PublishDiagnosticsParams, Uri,
};

use super::document::{Document, Settings};
use super::{analysis, BoxError};

/// How long typing must settle before a buffer is re-analysed.
pub(super) const DEBOUNCE: Duration = Duration::from_millis(300);

/// Why the loop woke up.
enum Wake {
    Message(Message),
    /// The debounce elapsed, so pending buffers are ready to analyse.
    Settled,
    /// The client closed the connection.
    Disconnected,
}

pub(super) struct Server {
    /// Keyed by URI string. `Uri` itself hashes via `as_str()`, but it carries a
    /// `Cell` internally, which rules it out as a key under `clippy::all`.
    pub(super) documents: HashMap<String, Document>,
    pub(super) settings: Settings,
}

impl Server {
    pub(super) fn new() -> Self {
        Self {
            documents: HashMap::new(),
            settings: Settings::default(),
        }
    }

    pub(super) fn main_loop(&mut self, connection: &Connection) -> Result<(), BoxError> {
        // Trailing debounce: `didChange` pushes the deadline out, and analysis
        // runs once typing pauses.
        let mut dirty: HashSet<String> = HashSet::new();
        let mut deadline: Option<Instant> = None;

        loop {
            match self.receive(connection, deadline) {
                Wake::Message(Message::Request(request)) => {
                    if connection.handle_shutdown(&request)? {
                        return Ok(());
                    }
                    self.handle_request(connection, request)?;
                }
                Wake::Message(Message::Notification(notification)) => {
                    self.handle_notification(connection, notification, &mut dirty, &mut deadline)?;
                }
                Wake::Message(Message::Response(_)) => {}
                Wake::Settled => {
                    deadline = None;
                    for key in std::mem::take(&mut dirty) {
                        self.publish(connection, &key)?;
                    }
                }
                // The client is gone; there is nobody left to report to.
                Wake::Disconnected => return Ok(()),
            }
        }
    }

    /// Block for the next message, bounded by `deadline` when analysis is pending.
    fn receive(&self, connection: &Connection, deadline: Option<Instant>) -> Wake {
        let Some(at) = deadline else {
            return match connection.receiver.recv() {
                Ok(message) => Wake::Message(message),
                Err(_) => Wake::Disconnected,
            };
        };
        match connection
            .receiver
            .recv_timeout(at.saturating_duration_since(Instant::now()))
        {
            Ok(message) => Wake::Message(message),
            Err(error) if error.is_timeout() => Wake::Settled,
            Err(_) => Wake::Disconnected,
        }
    }

    pub(super) fn publish(&self, connection: &Connection, key: &str) -> Result<(), BoxError> {
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

    pub(super) fn send_diagnostics(
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
