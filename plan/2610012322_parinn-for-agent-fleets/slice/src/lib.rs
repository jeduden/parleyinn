//! Spike S12: two writers' segments, received in any order, rebuild
//! one byte-identical search index and one room state, foreign events
//! stay untrusted, and a purged event leaves no stored or replicated
//! value that confirms a guess at its content (REC-06, REC-10, REC-17,
//! ADM-07, ADM-08, LANE-31, PEER-03, PRV-02).
//!
//! The record side follows SRS 2.x: each writer appends hash-chained
//! events whose header holds only the address, kind, provenance,
//! `prev_hash`, the other writers' chain heads it had seen, and the
//! commitment. Content and its commitment key travel beside the header,
//! never inside it, so a purge erases both and leaves the chain whole.

// S2's corpus, read where it lives; formatting it is S2's business.
#[path = "../../../2609292004_spike-s2-sqlite-driver/bench/src/corpus.rs"]
#[allow(dead_code, clippy::all)]
#[rustfmt::skip]
pub mod corpus;

pub mod canon;
pub mod commit;
pub mod fixture;
pub mod index;
pub mod node;
pub mod record;
pub mod room;
pub mod scan;

use std::fmt;

/// Every failure in the slice, with its cause as text.
#[derive(Debug)]
pub struct Error(pub String);

impl Error {
    pub fn new(msg: impl Into<String>) -> Self {
        Error(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error(format!("io: {e}"))
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error(format!("sqlite: {e}"))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error(format!("json: {e}"))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Lowercase hex of bytes.
pub fn hex(b: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(DIGITS[(x >> 4) as usize] as char);
        s.push(DIGITS[(x & 15) as usize] as char);
    }
    s
}

/// Parses 64 lowercase hex digits into 32 bytes.
pub fn unhex32(s: &str) -> Result<[u8; 32]> {
    let b = s.as_bytes();
    if b.len() != 64 {
        return Err(Error::new(format!(
            "hex of length {} is not 32 bytes",
            b.len()
        )));
    }
    let nib = |c: u8| match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        _ => Err(Error::new("not lowercase hex")),
    };
    let mut out = [0u8; 32];
    for (i, o) in out.iter_mut().enumerate() {
        *o = (nib(b[2 * i])? << 4) | nib(b[2 * i + 1])?;
    }
    Ok(out)
}
