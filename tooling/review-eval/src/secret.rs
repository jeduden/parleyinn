//! The scrub: every text the review writes out of an agent's run, a
//! file it keeps or a line it prints, is searched for the model access
//! token before it leaves, and each form of it found is replaced with
//! [`REDACTED`] and counted (ENG-28, I6). A form is the token itself,
//! its base64 at any offset in either alphabet, its percent encoding
//! and its hex with hex digits in either case, and any token of the
//! Claude shape, `sk-ant-` and a run of token characters, the review
//! holds or not. A count above zero means an agent's output held the
//! token: the review fails closed.
//!
//! One text of that shape is no token of the review's: one a file of
//! the trees the agents read holds, a fixture a test or a corpus keeps.
//! An agent that reads such a file has it in its run log, and that is
//! the tree's text, not a leak. The scrub replaces it with
//! [`TREE_TEXT`], so nothing the review keeps holds the shape, and does
//! not count it. The review's own token counts wherever it stands.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

/// What stands in for each form of the token found.
pub const REDACTED: &str = "[REDACTED token]";

/// What stands in for a text of the Claude shape a tree holds.
pub const TREE_TEXT: &str = "[key-shaped text of the tree]";

/// The directories under a tree whose files are no text of it: what git
/// and a build wrote.
pub const NOT_THE_TREE: [&str; 2] = [".git", "target"];

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

/// The forms of the secrets it searches for, longest first, and the
/// texts of the Claude shape the trees hold.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scrubber {
    forms: Vec<String>,
    spared: BTreeSet<String>,
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

        Self {
            forms,
            spared: BTreeSet::new(),
        }
    }

    /// This scrubber, taking each of `texts` for the tree's own and no
    /// leak: [`shaped_under`] the trees the agents read.
    #[must_use]
    pub fn sparing(mut self, texts: BTreeSet<String>) -> Self {
        self.spared = texts;

        self
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

    /// `text` with every form replaced, and how many were: a text of
    /// the shape a tree holds is replaced too and is not counted.
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
        let (out, shaped) = scrub_shape(&out, &self.spared);

        (out, count + shaped)
    }

    /// Scrubs the file at `path` in place, rewriting it only when the
    /// scrub replaced something in it, and answers how many forms it
    /// held; a missing file holds none.
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
        let before = String::from_utf8_lossy(&bytes);
        let (text, n) = self.scrub(&before);
        if text != before {
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

/// The first text of the Claude shape in `text`: where it starts, where
/// it ends, and whether it is a token, [`SHAPE`] followed by at least
/// [`SHAPE_TAIL`] token characters, and not the shape named in prose.
fn shape_at(text: &str) -> Option<(usize, usize, bool)> {
    let at = text.find(SHAPE)?;
    let from = &text[at + SHAPE.len()..];
    let tail = from.find(|c: char| !token_char(c)).unwrap_or(from.len());

    Some((at, at + SHAPE.len() + tail, tail >= SHAPE_TAIL))
}

/// `text` with every token of the Claude shape replaced, and how many
/// were tokens no tree holds: one of `spared` gives way to
/// [`TREE_TEXT`] and is not counted.
fn scrub_shape(text: &str, spared: &BTreeSet<String>) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut count = 0;
    while let Some((at, end, token)) = shape_at(rest) {
        out.push_str(&rest[..at]);
        let found = &rest[at..end];
        if !token {
            out.push_str(found);
        } else if spared.contains(found) {
            out.push_str(TREE_TEXT);
        } else {
            out.push_str(REDACTED);
            count += 1;
        }
        rest = &rest[end..];
    }
    out.push_str(rest);

    (out, count)
}

/// Every token of the Claude shape a file under `root` holds, at any
/// depth, [`NOT_THE_TREE`] left out: the texts a scrub of what agents
/// read there spares. The directory `apart`, as resolved, is left out
/// too: the staged credential's, whose tokens are no fixture.
///
/// # Errors
///
/// Fails when `root` does not resolve, or a directory or a file cannot
/// be read.
pub fn shaped_under(root: &Path, apart: Option<&Path>) -> Result<BTreeSet<String>, String> {
    let mut found = BTreeSet::new();
    let mut dirs = vec![fs::canonicalize(root).map_err(|e| unread(root, &e))?];
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).map_err(|e| unread(&dir, &e))? {
            let entry = entry.map_err(|e| unread(&dir, &e))?;
            let kind = entry.file_type().map_err(|e| unread(&entry.path(), &e))?;
            if kind.is_dir() {
                let named = NOT_THE_TREE.iter().any(|name| entry.file_name() == *name);
                if !named && apart != Some(&entry.path()) {
                    dirs.push(entry.path());
                }
            } else if kind.is_file() {
                let bytes = fs::read(entry.path()).map_err(|e| unread(&entry.path(), &e))?;
                collect_shaped(&String::from_utf8_lossy(&bytes), &mut found);
            }
        }
    }

    Ok(found)
}

/// Adds every token of the Claude shape in `text` to `found`.
fn collect_shaped(text: &str, found: &mut BTreeSet<String>) {
    let mut rest = text;
    while let Some((at, end, token)) = shape_at(rest) {
        if token {
            found.insert(rest[at..end].to_owned());
        }
        rest = &rest[end..];
    }
}

/// Why `path` was not read.
fn unread(path: &Path, e: &io::Error) -> String {
    format!("read {}: {e}", path.display())
}

#[cfg(test)]
mod tests;
