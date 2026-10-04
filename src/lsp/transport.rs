//! Transport setup and the initialize handshake.

use lsp_server::Connection;
use lsp_types::{
    OneOf, PositionEncodingKind, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};
use serde::Deserialize;

use super::server::Server;
use super::BoxError;

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
    server.settings.apply(initialization_options);
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
