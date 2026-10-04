//! Maps the JSON paths carried by [`crate::diagnostic::Diagnostic`] back to byte
//! ranges in the original source text.
//!
//! Diagnostic paths are built by string concatenation in the validators and lint
//! rules (`format!("{}.paint.{}", path, key)` and friends), which yields a closed
//! grammar:
//!
//! ```text
//! path    := segment ('.' segment)*
//! segment := ident | ident '[' N ']' ('[' N ']')*
//! ```
//!
//! [`SourceMap`] scans the raw text and builds its keys with the *same*
//! concatenation rules, so resolution is an exact map lookup and no path parsing
//! is needed. That matters for keys containing the delimiters themselves: a source
//! named `openmaptiles.v3` produces `sources.openmaptiles.v3` on both sides and
//! matches, where splitting on `.` would not.

use std::collections::HashMap;
use std::ops::Range;

use crate::diagnostic::{Diagnostic, Position, TextRange};

/// Guards against stack overflow on pathologically nested input.
const MAX_DEPTH: usize = 256;

/// Byte ranges of a single JSON node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The object key including its quotes. Empty (`start == end`) for array
    /// elements and for the document root, which have no key.
    pub key: Range<usize>,
    /// The value.
    pub value: Range<usize>,
}

impl Span {
    /// Key through end of value, so a diagnostic covers `"fill-colour": "#fff"`
    /// whether the fault is in the name or the value.
    fn outer(&self) -> Range<usize> {
        let start = if self.key.is_empty() {
            self.value.start
        } else {
            self.key.start.min(self.value.start)
        };
        start..self.value.end
    }
}

/// Path-to-byte-range index over a JSON document, plus line/column resolution.
pub struct SourceMap {
    spans: HashMap<String, Span>,
    /// Byte offset of the start of each line.
    line_starts: Vec<usize>,
    text: String,
}

impl SourceMap {
    /// Index `text`. Tolerant: malformed or truncated input yields whatever was
    /// scanned before the fault rather than an error, which is the normal case
    /// for a buffer being edited in a language server.
    pub fn parse(text: &str) -> Self {
        let mut scanner = Scanner {
            bytes: text.as_bytes(),
            pos: 0,
            spans: HashMap::new(),
        };
        if let Some(root) = scanner.value("", 0) {
            scanner.record(String::new(), root.start..root.start, root);
        }

        let mut line_starts = vec![0usize];
        line_starts.extend(
            text.bytes()
                .enumerate()
                .filter(|(_, b)| *b == b'\n')
                .map(|(i, _)| i + 1),
        );

        Self {
            spans: scanner.spans,
            line_starts,
            text: text.to_string(),
        }
    }

    /// Exact lookup. `None` when the path is absent from the text — which happens
    /// legitimately, e.g. E004 reports `layers[3].source` for a missing key.
    pub fn resolve(&self, path: &str) -> Option<&Span> {
        self.spans.get(path)
    }

    /// Exact lookup, falling back to the nearest ancestor that does exist, then to
    /// the document root. This is what makes missing-key diagnostics land on the
    /// enclosing object instead of nowhere, with no per-code special casing.
    pub fn resolve_or_parent(&self, path: &str) -> Option<&Span> {
        if let Some(span) = self.spans.get(path) {
            return Some(span);
        }
        let mut current = path;
        while let Some(parent) = trim_last_segment(current) {
            current = parent;
            if let Some(span) = self.spans.get(current) {
                return Some(span);
            }
        }
        self.spans.get("")
    }

    /// Line/column range for a path, via [`Self::resolve_or_parent`].
    pub fn range_for_path(&self, path: &str) -> Option<TextRange> {
        let outer = self.resolve_or_parent(path)?.outer();
        Some(TextRange {
            start: self.position(outer.start),
            end: self.position(outer.end),
        })
    }

    /// Byte offset to a zero-based line and UTF-16 column, matching how LSP
    /// positions are counted by default.
    pub fn position(&self, offset: usize) -> Position {
        let offset = offset.min(self.text.len());
        let line = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            // `line_starts[0]` is 0 and `offset >= 0`, so `i >= 1` here.
            Err(i) => i - 1,
        };
        let line_start = self.line_starts[line];
        let character = self.text[line_start..offset].encode_utf16().count();
        Position {
            line: line as u32,
            character: character as u32,
        }
    }

    /// Byte offset where a zero-based line begins. `None` past the last line.
    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    /// Number of indexed nodes. Useful in tests.
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }
}

/// Fill in [`Diagnostic::range`] for each diagnostic from its path.
pub fn resolve_ranges(diagnostics: &mut [Diagnostic], map: &SourceMap) {
    for d in diagnostics.iter_mut() {
        d.range = map.range_for_path(&d.path);
    }
}

/// Drop the last `.key` or `[n]` from a path. `None` when only one segment remains.
fn trim_last_segment(path: &str) -> Option<&str> {
    let cut = match (path.rfind('.'), path.rfind('[')) {
        (Some(dot), Some(bracket)) => dot.max(bracket),
        (Some(i), None) | (None, Some(i)) => i,
        (None, None) => return None,
    };
    if cut == 0 {
        return None;
    }
    Some(&path[..cut])
}

struct Scanner<'a> {
    bytes: &'a [u8],
    pos: usize,
    spans: HashMap<String, Span>,
}

impl<'a> Scanner<'a> {
    fn record(&mut self, path: String, key: Range<usize>, value: Range<usize>) {
        // First write wins. Only reachable via pathological duplicate-key input
        // or a key that collides with a nested path (a source literally named
        // `a.b` alongside a source `a` with a field `b`).
        self.spans.entry(path).or_insert(Span { key, value });
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    /// Consume one value, returning its byte range. `None` on malformed input,
    /// which unwinds the whole scan and keeps what was already recorded.
    fn value(&mut self, path: &str, depth: usize) -> Option<Range<usize>> {
        if depth > MAX_DEPTH {
            return None;
        }
        self.skip_ws();
        let start = self.pos;
        match self.peek()? {
            b'{' => self.object(path, depth)?,
            b'[' => self.array(path, depth)?,
            b'"' => {
                self.string_raw()?;
            }
            _ => self.primitive()?,
        }
        Some(start..self.pos)
    }

    fn object(&mut self, path: &str, depth: usize) -> Option<()> {
        self.pos += 1; // '{'
        loop {
            self.skip_ws();
            match self.peek()? {
                b'}' => {
                    self.pos += 1;
                    return Some(());
                }
                b',' => self.pos += 1,
                b'"' => {
                    let key_start = self.pos;
                    let inner = self.string_raw()?;
                    let key_range = key_start..self.pos;
                    let raw = std::str::from_utf8(&self.bytes[inner]).ok()?;
                    let key = if raw.contains('\\') {
                        unescape(raw)
                    } else {
                        raw.to_string()
                    };

                    self.skip_ws();
                    if self.peek()? != b':' {
                        return None;
                    }
                    self.pos += 1;

                    let child = if path.is_empty() {
                        key
                    } else {
                        format!("{}.{}", path, key)
                    };
                    let value_range = self.value(&child, depth + 1)?;
                    self.record(child, key_range, value_range);
                }
                _ => return None,
            }
        }
    }

    fn array(&mut self, path: &str, depth: usize) -> Option<()> {
        self.pos += 1; // '['
        let mut index = 0usize;
        loop {
            self.skip_ws();
            match self.peek()? {
                b']' => {
                    self.pos += 1;
                    return Some(());
                }
                b',' => self.pos += 1,
                _ => {
                    let child = format!("{}[{}]", path, index);
                    let value_range = self.value(&child, depth + 1)?;
                    self.record(child, value_range.start..value_range.start, value_range);
                    index += 1;
                }
            }
        }
    }

    /// Consume a quoted string, returning the range *inside* the quotes.
    /// Scanning by byte is UTF-8 safe: no continuation byte can equal `"` or `\`.
    fn string_raw(&mut self) -> Option<Range<usize>> {
        self.pos += 1; // opening quote
        let inner_start = self.pos;
        loop {
            match self.peek()? {
                b'"' => {
                    let inner = inner_start..self.pos;
                    self.pos += 1;
                    return Some(inner);
                }
                b'\\' => self.pos += 2,
                _ => self.pos += 1,
            }
        }
    }

    /// Consume a number, `true`, `false` or `null`. Fails rather than consuming
    /// nothing, so a malformed document can never spin the enclosing loop.
    fn primitive(&mut self) -> Option<()> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if matches!(b, b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r') {
                break;
            }
            self.pos += 1;
        }
        if self.pos == start {
            None
        } else {
            Some(())
        }
    }
}

fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => match take_hex4(&mut chars) {
                Some(hi) if (0xD800..0xDC00).contains(&hi) => {
                    let rest = chars.clone();
                    let low = (chars.next() == Some('\\') && chars.next() == Some('u'))
                        .then(|| take_hex4(&mut chars))
                        .flatten()
                        .filter(|lo| (0xDC00..0xE000).contains(lo));
                    match low {
                        Some(lo) => {
                            let c = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                            out.push(char::from_u32(c).unwrap_or('\u{FFFD}'));
                        }
                        None => {
                            chars = rest;
                            out.push('\u{FFFD}');
                        }
                    }
                }
                Some(hi) => out.push(char::from_u32(hi).unwrap_or('\u{FFFD}')),
                None => out.push('\u{FFFD}'),
            },
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

fn take_hex4(chars: &mut std::str::Chars<'_>) -> Option<u32> {
    let mut value = 0u32;
    for _ in 0..4 {
        value = value * 16 + chars.next()?.to_digit(16)?;
    }
    Some(value)
}
