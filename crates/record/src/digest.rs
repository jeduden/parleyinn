//! SHA-256, the chain's hash (§8.3), and HMAC-SHA-256 over it, the
//! commitment's (REC-17).

use std::fmt;

use sha2::{Digest as _, Sha256};

/// HMAC's block size for SHA-256, in bytes.
const BLOCK: usize = 64;

/// A SHA-256 digest.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest([u8; 32]);

impl Digest {
    /// The digest of nothing chained: the `prev_hash` of a writer's
    /// first event.
    pub const ZERO: Self = Self([0; 32]);

    /// SHA-256 over `parts`, one after the other.
    #[must_use]
    pub fn of(parts: &[&[u8]]) -> Self {
        let mut hasher = Sha256::new();
        for part in parts {
            hasher.update(part);
        }

        Self(hasher.finalize().into())
    }

    /// The digest these bytes are.
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Its bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Its 64 lowercase hex digits.
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex(&self.0)
    }

    /// The digest 64 lowercase hex digits spell.
    ///
    /// # Errors
    ///
    /// Fails on any other text.
    pub fn from_hex(text: &str) -> Result<Self, HexError> {
        unhex(text).map(Self)
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Digest({self})")
    }
}

/// Why text spells none of the bytes asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexError {
    /// It is not `want` characters long but `got`.
    Length { want: usize, got: usize },
    /// It holds a character that is no lowercase hex digit.
    Digit(char),
}

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { want, got } => {
                write!(f, "want {want} lowercase hex digits, got {got} characters")
            }
            Self::Digit(c) => write!(f, "want lowercase hex digits, got {c:?}"),
        }
    }
}

impl std::error::Error for HexError {}

/// `bytes` as lowercase hex digits.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(char::from(DIGITS[usize::from(b >> 4)]));
        out.push(char::from(DIGITS[usize::from(b & 0xf)]));
    }

    out
}

/// The `N` bytes `2 * N` lowercase hex digits spell.
///
/// # Errors
///
/// Fails on any other text.
pub fn unhex<const N: usize>(text: &str) -> Result<[u8; N], HexError> {
    let count = text.chars().count();
    if count != 2 * N {
        return Err(HexError::Length {
            want: 2 * N,
            got: count,
        });
    }
    let digits: Vec<u8> = text
        .chars()
        .map(|c| match c {
            '0'..='9' | 'a'..='f' => c.to_digit(16).map(|d| d as u8).ok_or(HexError::Digit(c)),
            _ => Err(HexError::Digit(c)),
        })
        .collect::<Result<_, _>>()?;

    // Each byte from its two digits; the length check above makes
    // every index exist.
    Ok(std::array::from_fn(|i| {
        digits.get(2 * i).map_or(0, |hi| hi << 4) | digits.get(2 * i + 1).copied().unwrap_or(0)
    }))
}

/// HMAC-SHA-256 (RFC 2104) of `message`, given in parts, under `key`.
#[must_use]
pub fn hmac_sha256(key: &[u8], message: &[&[u8]]) -> Digest {
    let mut block = [0u8; BLOCK];
    if key.len() > BLOCK {
        block[..32].copy_from_slice(Digest::of(&[key]).as_bytes());
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let pad = |byte: u8| block.map(|k| k ^ byte);
    let (inner_pad, outer_pad) = (pad(0x36), pad(0x5c));
    let mut inner: Vec<&[u8]> = vec![&inner_pad];
    inner.extend_from_slice(message);
    let inner = Digest::of(&inner);

    Digest::of(&[&outer_pad, inner.as_bytes()])
}

/// SHA-256 over bytes fed a piece at a time, readable after each: the
/// consumed prefix of a transcript source as ingest reads it (REC-03).
#[derive(Clone, Default)]
pub struct Running(Sha256);

impl Running {
    /// Feeds `bytes` after what came before.
    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    /// The digest of every byte fed so far.
    #[must_use]
    pub fn digest(&self) -> Digest {
        Digest(self.0.clone().finalize().into())
    }
}

impl fmt::Debug for Running {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Running({})", self.digest())
    }
}

#[cfg(test)]
mod tests;
