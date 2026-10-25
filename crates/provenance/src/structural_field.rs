//! LMK-03: a structural field untrusted input can influence (a file
//! path, a tool name, an MCP server name, a branch name) is sanitized
//! before any structural use. A value made only of `A–Z a–z 0–9 . _ / @
//! : + -` and no longer than the limit is kept as it is; any other
//! value is replaced whole with `<field:HASH8>`, HASH8 the first eight
//! lowercase hex digits of the SHA-256 of the original UTF-8 bytes.
//! Nothing here reads a clock or randomness (I10).

use record::digest::Digest;

/// The default maximum length of a structural field, in bytes (LMK-03).
pub const DEFAULT_LIMIT: usize = 80;

/// A structural field after sanitizing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sanitized {
    text: String,
    replaced: bool,
}

impl Sanitized {
    /// The text to use: the value itself, or its `<field:HASH8>`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Whether the value did not conform and was replaced whole.
    #[must_use]
    pub fn is_replaced(&self) -> bool {
        self.replaced
    }
}

/// `value` as a structural field of at most `limit` bytes.
#[must_use]
///
/// The replacement holds `<` and `>`, outside the allow-list, so
/// sanitizing a [`Sanitized::as_str`] again would replace it again:
/// sanitize once, where untrusted input enters, and keep the
/// [`Sanitized`].
pub fn sanitize(value: &str, limit: usize) -> Sanitized {
    if value.len() <= limit && value.bytes().all(allowed) {
        return Sanitized {
            text: value.to_owned(),
            replaced: false,
        };
    }
    Sanitized {
        text: format!("<field:{}>", hash8(value)),
        replaced: true,
    }
}

/// Whether `byte` is in LMK-03's allow-list.
fn allowed(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'@' | b':' | b'+' | b'-')
}

/// The first eight lowercase hex digits of the SHA-256 of `value`.
fn hash8(value: &str) -> String {
    Digest::of(&[value.as_bytes()])
        .to_hex()
        .chars()
        .take(8)
        .collect()
}

#[cfg(test)]
mod tests;
