//! The scrub: every text the review writes out of an agent's run, a
//! file it keeps or a line it prints, is searched for the model access
//! token before it leaves, and each form of it found is replaced with
//! [`REDACTED`] and counted (ENG-28, I6). A form is the token itself,
//! its base64 at any offset in either alphabet, its percent encoding
//! and its hex with hex digits in either case, and any token of the
//! Claude shape, `sk-ant-` and a run of token characters, the review
//! holds or not. A count above zero means an agent's output held the
//! token: the review fails closed.

use std::fs;
use std::io;
use std::path::Path;

/// What stands in for each form of the token found.
pub const REDACTED: &str = "[REDACTED token]";

/// How a Claude token starts.
pub const SHAPE: &str = "sk-ant-";

/// How many token characters past [`SHAPE`] make a token rather than
/// the shape named in prose.
pub const SHAPE_TAIL: usize = 20;

/// The shortest secret, or encoded part of one, searched for: a
/// shorter one would match ordinary text.
pub const SHORTEST: usize = 12;

/// Whether `secret`, trimmed, holds something yet is shorter than
/// [`SHORTEST`]: the scrub cannot match it, so it must not be used.
#[must_use]
pub fn too_short(secret: &str) -> bool {
    let secret = secret.trim();
    !secret.is_empty() && secret.len() < SHORTEST
}

/// Why a token [`too_short`] is refused, `doing` what with it.
#[must_use]
pub fn refusal(doing: &str) -> String {
    format!(
        "the Claude token is shorter than the {SHORTEST} characters the scrub can match; \
         refusing to {doing} it"
    )
}

/// The two base64 alphabets.
pub const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
pub const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// The forms of the secrets it searches for, longest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scrubber {
    forms: Vec<String>,
}

impl Scrubber {
    /// A scrubber for `secrets`; one shorter than [`SHORTEST`] after
    /// trimming is no form of its own, though the shape still is.
    #[must_use]
    pub fn new(secrets: &[&str]) -> Self {
        let mut forms: Vec<String> = secrets
            .iter()
            .map(|s| s.trim())
            .filter(|s| s.len() >= SHORTEST)
            .flat_map(forms_of)
            .filter(|f| f.len() >= SHORTEST)
            .collect();
        forms.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        forms.dedup();

        Self { forms }
    }

    /// A scrubber for `secrets` that fails closed, never naming the
    /// secret, when one is too short to be a form: the scrub could not
    /// match it, so nothing it guards may run. An empty one is none.
    ///
    /// # Errors
    ///
    /// Fails when a secret is shorter than [`SHORTEST`] after trimming.
    pub fn guarded(secrets: &[&str]) -> Result<Self, String> {
        if secrets.iter().any(|s| too_short(s)) {
            return Err(refusal("use"));
        }

        Ok(Self::new(secrets))
    }

    /// `text` with every form replaced, and how many were.
    #[must_use]
    pub fn scrub(&self, text: &str) -> (String, usize) {
        let mut out = text.to_owned();
        let mut count = 0;
        for form in &self.forms {
            let n = out.matches(form.as_str()).count();
            if n > 0 {
                out = out.replace(form.as_str(), REDACTED);
                count += n;
            }
        }
        let (out, shaped) = scrub_shape(&out);

        (out, count + shaped)
    }

    /// Scrubs the file at `path` in place, rewriting it only when it
    /// held a form; a missing file holds none.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be read or written back.
    pub fn scrub_file(&self, path: &Path) -> Result<usize, String> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(format!("read {}: {e}", path.display())),
        };
        let (text, n) = self.scrub(&String::from_utf8_lossy(&bytes));
        if n > 0 {
            fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))?;
        }

        Ok(n)
    }

    /// Scrubs every file under `dir`, at any depth; a missing directory
    /// holds none.
    ///
    /// # Errors
    ///
    /// Fails when a directory or file cannot be read, or a file written
    /// back.
    pub fn scrub_tree(&self, dir: &Path) -> Result<usize, String> {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(format!("read {}: {e}", dir.display())),
        };
        let mut count = 0;
        for entry in entries {
            let path = entry
                .map_err(|e| format!("read {}: {e}", dir.display()))?
                .path();
            count += if path.is_dir() {
                self.scrub_tree(&path)?
            } else {
                self.scrub_file(&path)?
            };
        }

        Ok(count)
    }
}

/// Every form `secret` can take in text: itself, its percent encoding
/// and its hex, each with hex digits in either case, and its base64 at
/// each of the three offsets in either alphabet, the characters the
/// neighbouring bytes share left off.
fn forms_of(secret: &str) -> Vec<String> {
    let mut forms = vec![
        secret.to_owned(),
        percent(secret, true),
        percent(secret, false),
    ];
    let hex: String = secret.bytes().map(|b| format!("{b:02x}")).collect();
    forms.push(hex.to_uppercase());
    forms.push(hex);
    for alphabet in [STANDARD, URL_SAFE] {
        for offset in 0..3 {
            let mut bytes = vec![0u8; offset];
            bytes.extend_from_slice(secret.as_bytes());
            let encoded = encode(&bytes, alphabet);
            // A character holding a bit of a neighbouring byte differs
            // with that byte, so it is no part of the form.
            let skip = (offset * 8).div_ceil(6);
            let keep = (bytes.len() * 8) / 6;
            forms.push(encoded.chars().take(keep).skip(skip).collect());
        }
    }

    forms
}

/// `text` percent-encoded: every byte outside the unreserved set, its
/// hex digits in `upper` case or lower.
fn percent(text: &str, upper: bool) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                char::from(b).to_string()
            } else if upper {
                format!("%{b:02X}")
            } else {
                format!("%{b:02x}")
            }
        })
        .collect()
}

/// `bytes` in base64 with `alphabet`, unpadded.
#[must_use]
pub fn encode(bytes: &[u8], alphabet: &[u8; 64]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk.first().copied().unwrap_or(0),
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..=chunk.len() {
            let at = (n >> (18 - 6 * i)) & 0x3f;
            out.extend(alphabet.get(at as usize).map(|c| char::from(*c)));
        }
    }

    out
}

/// Whether `c` can be part of a token.
fn token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

/// `text` with every token of the Claude shape replaced, and how many
/// were: [`SHAPE`] followed by at least [`SHAPE_TAIL`] token characters.
fn scrub_shape(text: &str) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut count = 0;
    while let Some(at) = rest.find(SHAPE) {
        let (before, from) = rest.split_at(at);
        out.push_str(before);
        let tail = from[SHAPE.len()..]
            .find(|c: char| !token_char(c))
            .unwrap_or(from.len() - SHAPE.len());
        let end = SHAPE.len() + tail;
        if tail >= SHAPE_TAIL {
            out.push_str(REDACTED);
            count += 1;
        } else {
            out.push_str(&from[..end]);
        }
        rest = &from[end..];
    }
    out.push_str(rest);

    (out, count)
}

#[cfg(test)]
mod tests;
