//! The home's audit log of ParleyInn's own operations (OPS-01, OPS-02),
//! and the counters derived from it (OPS-03).
//!
//! - [`entry`]: one audit entry, its canonical line and its hash.
//! - [`log`]: the log's files under `audit/`, appended under a lock,
//!   rotated with chain continuation, and checked whole.
//! - [`counters`]: each named counter, and how far it rose since the
//!   counters were last acknowledged.
//!
//! Every field an entry is built from is text ParleyInn ships, so no record
//! content and no secret reaches the log.

use std::fmt;
use std::path::PathBuf;

pub mod counters;
pub mod entry;
pub mod log;

/// What can go wrong keeping the audit log.
#[derive(Debug)]
pub enum AuditError {
    /// A file or directory of the log cannot be read or written.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Another process held the log's lock past the deadline (ENG-03).
    Busy { path: PathBuf },
    /// The log's chain is broken at `line` of `file`; line 0 names the
    /// file as a whole.
    Broken {
        file: String,
        line: usize,
        why: &'static str,
    },
    /// An entry has no canonical encoding.
    Encode(record::canonical::Error),
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Busy { path } => write!(f, "{}: the audit log stays locked", path.display()),
            Self::Broken { file, line: 0, why } => write!(f, "audit log {file}: {why}"),
            Self::Broken { file, line, why } => write!(f, "audit log {file} line {line}: {why}"),
            Self::Encode(e) => write!(f, "audit entry: {e}"),
        }
    }
}

impl std::error::Error for AuditError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Encode(e) => Some(e),
            Self::Busy { .. } | Self::Broken { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
