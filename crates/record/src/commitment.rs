//! The commitment `C` every event carries (REC-17): HMAC-SHA-256 under
//! the event's commitment key, a random 256-bit secret kept with the
//! content and never in the header, over the content's canonical
//! encoding. Without the key, `C` confirms no guess at the content
//! (hiding); with it, `C` opens to that content alone (binding). The
//! chain refers to content only through `C`.

use std::fmt;

use serde_json::Value;

use crate::canonical;
pub use crate::digest::hmac_sha256;
use crate::digest::{Digest, HexError, hex, unhex};

/// The label every commitment's message starts with.
const LABEL: &[u8] = b"parinn commitment v1\0";

/// An event's commitment key: 256 random bits the caller draws. It
/// never shows in `Debug`, so no log can leak it.
#[derive(Clone, PartialEq, Eq)]
pub struct CommitmentKey([u8; 32]);

impl CommitmentKey {
    /// The key these random bytes are.
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Its bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Its 64 lowercase hex digits, for the store.
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex(&self.0)
    }

    /// The key 64 lowercase hex digits spell.
    ///
    /// # Errors
    ///
    /// Fails on any other text.
    pub fn from_hex(text: &str) -> Result<Self, HexError> {
        unhex(text).map(Self)
    }
}

impl fmt::Debug for CommitmentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CommitmentKey(..)")
    }
}

/// A commitment to an event's content.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Commitment(Digest);

impl Commitment {
    /// The commitment this digest is.
    #[must_use]
    pub fn from_digest(digest: Digest) -> Self {
        Self(digest)
    }

    /// The digest it is.
    #[must_use]
    pub fn digest(&self) -> &Digest {
        &self.0
    }

    /// The commitment 64 lowercase hex digits spell.
    ///
    /// # Errors
    ///
    /// Fails on any other text.
    pub fn from_hex(text: &str) -> Result<Self, HexError> {
        Digest::from_hex(text).map(Self)
    }
}

impl fmt::Display for Commitment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for Commitment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Commitment({})", self.0)
    }
}

/// The commitment to `content` under `key`.
///
/// # Errors
///
/// Fails when `content` has no canonical encoding.
pub fn commit(key: &CommitmentKey, content: &Value) -> Result<Commitment, canonical::Error> {
    Ok(commit_encoded(key, &canonical::encode(content)?))
}

/// The commitment under `key` to the content whose canonical encoding
/// is `canonical`, for a caller that encoded it already.
#[must_use]
pub fn commit_encoded(key: &CommitmentKey, canonical: &str) -> Commitment {
    Commitment(hmac_sha256(&key.0, &[LABEL, canonical.as_bytes()]))
}

#[cfg(test)]
mod tests;
