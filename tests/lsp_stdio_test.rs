//! Drives the real binary over stdio with hand-written JSON-RPC framing.
//!
//! `tests/lsp_test.rs` exercises the same loop over `Connection::memory()`, which
//! is faster but structurally cannot catch anything about the process: stdio
//! framing, stdout purity, thread joining, or the exit code. The `drop(connection)`
//! hang in `serve()` was invisible to the in-memory test and would have shipped.

#![cfg(feature = "lsp")]

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const STYLE: &str = "{\n  \"version\": 8,\n  \"sources\": {},\n  \"layers\": [\n    {\n      \"id\": \"roads\",\n      \"type\": \"line\",\n      \"source\": \"missing\"\n    }\n  ]\n}";

struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Server {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_styl"))
            .arg("lsp")
            .arg("stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn styl lsp");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, message: serde_json::Value) {
        let body = serde_json::to_vec(&message).expect("serialize");
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len()).expect("write header");
        self.stdin.write_all(&body).expect("write body");
        self.stdin.flush().expect("flush");
    }

    /// Read one framed message. Panics if the stream ends first.
    fn recv(&mut self) -> serde_json::Value {
        let mut length = None;
        loop {
            let mut line = String::new();
            let read = self.stdout.read_line(&mut line).expect("read header");
            assert!(read > 0, "server closed the stream mid-message");
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(value) = line.strip_prefix("Content-Length: ") {
                length = Some(value.parse::<usize>().expect("content length"));
            }
        }
        let mut body = vec![0u8; length.expect("Content-Length header")];
        self.stdout.read_exact(&mut body).expect("read body");
        serde_json::from_slice(&body).expect("parse body")
    }
}

#[test]
fn real_stdio_session_round_trips_and_exits_cleanly() {
    let mut server = Server::start();

    server.send(serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "capabilities": {}, "processId": null, "rootUri": null }
    }));
    server.send(serde_json::json!({
        "jsonrpc": "2.0", "method": "initialized", "params": {}
    }));

    let initialize = server.recv();
    let capabilities = &initialize["result"]["capabilities"];
    // Ranges are counted in UTF-16 units, so the server must say so.
    assert_eq!(capabilities["positionEncoding"], "utf-16");
    assert_eq!(capabilities["documentFormattingProvider"], true);

    server.send(serde_json::json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": {
            "uri": "file:///project/style.json", "languageId": "json",
            "version": 1, "text": STYLE
        }}
    }));

    let published = server.recv();
    assert_eq!(published["method"], "textDocument/publishDiagnostics");
    let diagnostics = published["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics array");
    let e009 = diagnostics
        .iter()
        .find(|d| d["code"] == "E009")
        .expect("E009 over the wire");
    // `"source": "missing"` is on line 8, one-based.
    assert_eq!(e009["range"]["start"]["line"], 7);
    assert_eq!(e009["source"], "styl");

    server.send(serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": null
    }));
    let ack = server.recv();
    assert_eq!(ack["id"], 2);

    server.send(serde_json::json!({ "jsonrpc": "2.0", "method": "exit", "params": null }));

    // The regression this file exists for: `serve()` must drop the connection
    // before joining its IO threads, or this never returns.
    let status = server.child.wait().expect("server exits");
    assert_eq!(status.code(), Some(0), "clean shutdown");

    // stdout is the transport, so anything chatty would have corrupted the
    // frames above. stderr must also stay quiet on a normal session.
    let mut stderr = String::new();
    server
        .child
        .stderr
        .take()
        .expect("stderr")
        .read_to_string(&mut stderr)
        .expect("read stderr");
    assert!(stderr.is_empty(), "unexpected stderr: {:?}", stderr);
}

/// A build without the feature must fail loudly rather than hang an editor.
#[test]
fn lsp_subcommand_exists_in_this_build() {
    let output = Command::new(env!("CARGO_BIN_EXE_styl"))
        .args(["lsp", "--help"])
        .output()
        .expect("run styl lsp --help");
    assert!(output.status.success(), "`styl lsp --help` should succeed");
}
