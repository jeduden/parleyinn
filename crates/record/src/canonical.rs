//! RFC 8785, the JSON Canonicalization Scheme, which the hash chain,
//! the commitment and the audit chain hash (§8.3): no whitespace,
//! object members sorted by their names' UTF-16 code units, strings
//! escaped as ECMAScript's `JSON.stringify` escapes them, and numbers
//! as ECMAScript writes them.
//!
//! ParleyInn's field sets hold integers only, so a number must be an
//! integer ECMAScript represents exactly, within ±(2^53 - 1), and is
//! written in decimal. Any other number is refused rather than
//! rounded: a value that would encode differently elsewhere never
//! reaches a hash.

use std::fmt;

use serde_json::{Number, Value};

/// The largest integer an IEEE 754 double represents exactly with
/// every smaller one: 2^53 - 1.
pub const MAX_SAFE: u64 = (1 << 53) - 1;

/// Why a value has no canonical encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A number with a fraction, or an integer past ±(2^53 - 1).
    Number(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(
                f,
                "canonical encoding: {n} is not an integer within ±(2^53 - 1)"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// `value`'s canonical encoding.
///
/// # Errors
///
/// Fails on a number that is not an integer within ±(2^53 - 1).
pub fn encode(value: &Value) -> Result<String, Error> {
    let mut out = String::new();
    write_value(value, &mut out)?;

    Ok(out)
}

fn write_value(value: &Value, out: &mut String) -> Result<(), Error> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&number(n)?),
        Value::String(s) => write_string(s, out),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(item, out)?;
            }
            out.push(']');
        }
        Value::Object(members) => {
            let mut sorted: Vec<(&String, &Value)> = members.iter().collect();
            sorted.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
            out.push('{');
            for (i, (name, member)) in sorted.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(name, out);
                out.push(':');
                write_value(member, out)?;
            }
            out.push('}');
        }
    }

    Ok(())
}

/// An integer within ±(2^53 - 1), in decimal.
fn number(n: &Number) -> Result<String, Error> {
    let safe = match (n.as_u64(), n.as_i64()) {
        (Some(u), _) => u <= MAX_SAFE,
        (None, Some(i)) => i.unsigned_abs() <= MAX_SAFE,
        (None, None) => false,
    };
    if safe {
        Ok(n.to_string())
    } else {
        Err(Error::Number(n.to_string()))
    }
}

/// Appends `s` as a JSON string, escaped as RFC 8785 §3.2.2.2 says:
/// the quote, the backslash and the control characters, the five with
/// a short form in it, the others as `\u00xx`.
pub fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests;
