//! The one error every read of a clone ends in.

use std::fmt;

use crate::CommitId;

/// Why a clone could not be read. Each failure class has its own
/// variant (ENG-04, I6) so a caller counts it apart. No variant names a
/// path or repeats a byte of the clone: a hostile clone chooses those.
#[derive(Debug)]
pub enum Error {
    /// A file could not be opened or read; `context` says which kind.
    Io {
        /// The kind of file being read.
        context: &'static str,
        /// The cause.
        source: std::io::Error,
    },
    /// A file is not in the format git writes; the text says where.
    Corrupt(&'static str),
    /// An object a reference or a parent names is not in the clone.
    Missing(CommitId),
    /// The repository stores its objects in a hash other than SHA-1.
    UnsupportedObjectFormat,
    /// A layout this crate does not read: alternates, reftable, a
    /// symbolic link where a directory is expected, a newer pack.
    Unsupported(&'static str),
    /// The deadline passed before the answer was known.
    Deadline,
    /// A size or count passed its bound; the text says which.
    TooLarge(&'static str),
}

impl Error {
    /// An adapter for `map_err` that wraps an I/O failure with the kind
    /// of file being read.
    pub(crate) fn io(context: &'static str) -> impl FnOnce(std::io::Error) -> Self {
        move |source| Self::Io { context, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { context, .. } => write!(f, "cannot read {context}"),
            Self::Corrupt(what) => write!(f, "corrupt repository: {what}"),
            Self::Missing(id) => write!(f, "object {} is not in the clone", id.as_str()),
            Self::UnsupportedObjectFormat => f.write_str("unsupported object format"),
            Self::Unsupported(what) => write!(f, "unsupported repository layout: {what}"),
            Self::Deadline => f.write_str("deadline passed"),
            Self::TooLarge(what) => write!(f, "too large: {what}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
