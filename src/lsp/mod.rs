//! A Language Server Protocol server, served over stdio.
//!
//! Lives in the same binary as the CLI so an editor and CI run byte-identical
//! analysis — a squiggle can never disagree with a pipeline failure. One rule
//! follows from that: **stdout is the JSON-RPC transport**, so nothing reachable
//! from here may print to it. Diagnostics for the operator go to stderr.

mod analysis;
mod config_cache;
mod document;
mod handlers;
mod server;
mod transport;

pub use transport::{serve, serve_connection};

/// Any failure here ends the session, so the loop carries one boxed error rather
/// than a taxonomy nothing would match on.
pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;
