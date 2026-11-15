//! Request and notification handlers.

use std::collections::HashSet;
use std::time::Instant;

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument,
        DidOpenTextDocument, DidSaveTextDocument, Notification as _,
    },
    request::{Formatting, Request as _},
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentFormattingParams,
};

use super::document::Document;
use super::server::{Server, DEBOUNCE};
use super::{analysis, BoxError};

impl Server {
    pub(super) fn handle_request(
        &mut self,
        connection: &Connection,
        request: Request,
    ) -> Result<(), BoxError> {
        if request.method != Formatting::METHOD {
            return Ok(());
        }

        let (id, params) = match request.extract::<DocumentFormattingParams>(Formatting::METHOD) {
            Ok(extracted) => extracted,
            Err(error) => {
                eprintln!("styl: malformed formatting request: {}", error);
                return Ok(());
            }
        };

        let uri = params.text_document.uri;
        let edits = match self.documents.get(uri.as_str()) {
            Some(document) if document.is_style => analysis::format(
                &document.text,
                &uri,
                Some(params.options.tab_size),
                &mut self.configs,
            ),
            _ => None,
        };

        connection
            .sender
            .send(Message::Response(Response::new_ok(id, edits)))?;
        Ok(())
    }

    pub(super) fn handle_notification(
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
                self.settings.apply(Some(&params.settings));
                // Settings change what the rules report, so refresh everything.
                // If the server was just switched off, the refresh clears the
                // editor rather than leaving stale diagnostics on screen.
                let open: Vec<String> = self.documents.keys().cloned().collect();
                for key in open {
                    if self.settings.enabled {
                        self.publish(connection, &key)?;
                    } else {
                        self.clear(connection, &key)?;
                    }
                }
            }

            DidChangeWatchedFiles::METHOD => {
                let _: DidChangeWatchedFilesParams =
                    notification.extract(DidChangeWatchedFiles::METHOD)?;
                // Only `.stylrc` is watched, and one edit can change the config
                // seen by any directory below it, so drop the whole cache.
                self.configs.clear();
                let open: Vec<String> = self.documents.keys().cloned().collect();
                for key in open {
                    self.publish(connection, &key)?;
                }
            }

            _ => {}
        }

        Ok(())
    }
}
