//! Does anything stored confirm a guess at purged content?
//!
//! Someone holding the files guesses an event's content exactly and
//! looks for anything that would confirm it: the text itself, a plain
//! hash of the text or of the canonical content, or a commitment that
//! some stored 32-byte key opens to the guess (REC-17).

use std::collections::HashSet;
use std::path::PathBuf;

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{Result, canon, commit, hex, unhex32};

/// A guess at one event's content: its text and its whole content.
pub struct Guess {
    pub text: String,
    pub content: Value,
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && hay.windows(needle.len()).any(|w| w == needle)
}

/// Every run of 64 lowercase hex digits in the blob, as 32 bytes; a
/// longer run gives each of its 64-digit windows.
fn hex_keys(blob: &[u8], out: &mut HashSet<[u8; 32]>) {
    let is_hex = |b: &u8| b.is_ascii_digit() || (b'a'..=b'f').contains(b);
    for run in blob.split(|b| !is_hex(b)).filter(|r| r.len() >= 64) {
        for w in run.windows(64) {
            if let Ok(k) = std::str::from_utf8(w)
                .map_err(|_| ())
                .and_then(|s| unhex32(s).map_err(|_| ()))
            {
                out.insert(k);
            }
        }
    }
}

/// Each way the named blobs confirm the guess, empty if none does.
pub fn confirmations(blobs: &[(String, Vec<u8>)], guess: &Guess) -> Vec<String> {
    let canonical = canon::encode(&guess.content);
    let text = guess.text.as_bytes();
    let opening = &text[..text.len().min(48)];
    let text_hash: [u8; 32] = Sha256::digest(text).into();
    let content_hash: [u8; 32] = Sha256::digest(&canonical).into();
    let needles: [(&str, Vec<u8>); 7] = [
        ("the text", text.to_vec()),
        ("the text's opening", opening.to_vec()),
        ("the canonical content", canonical.clone()),
        ("SHA-256 of the text", text_hash.to_vec()),
        ("SHA-256 of the text, hex", hex(&text_hash).into_bytes()),
        ("SHA-256 of the content", content_hash.to_vec()),
        (
            "SHA-256 of the content, hex",
            hex(&content_hash).into_bytes(),
        ),
    ];
    let mut found = Vec::new();
    let mut keys = HashSet::new();
    for (name, blob) in blobs {
        for (what, needle) in &needles {
            if contains(blob, needle) {
                found.push(format!("{name}: {what}"));
            }
        }
        hex_keys(blob, &mut keys);
    }
    // A stored key that opens a stored commitment to the guess.
    for k in &keys {
        if keys.contains(&commit::commit(k, &canonical)) {
            found.push(format!(
                "a stored key {} opens a stored commitment to the guess",
                hex(k)
            ));
        }
    }
    found
}

/// Reads every file into a named blob.
pub fn read_all(files: &[PathBuf]) -> Result<Vec<(String, Vec<u8>)>> {
    files
        .iter()
        .map(|p| Ok((p.display().to_string(), std::fs::read(p)?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn guess() -> Guess {
        let content =
            json!({"text": "ka lo mi ne ru ta shi po an el or un is ex de re co pro tra ver"});
        Guess {
            text: content["text"].as_str().unwrap().to_string(),
            content,
        }
    }

    #[test]
    fn nothing_related_confirms_nothing() {
        assert!(confirmations(&[("f".into(), b"unrelated bytes".to_vec())], &guess()).is_empty());
    }

    #[test]
    fn the_text_confirms() {
        let g = guess();
        let blob = format!("xx{}yy", g.text).into_bytes();
        assert!(!confirmations(&[("f".into(), blob)], &g).is_empty());
    }

    #[test]
    fn a_stored_key_beside_its_commitment_confirms() {
        let g = guess();
        let k = [7u8; 32];
        let c = commit::commit(&k, &canon::encode(&g.content));
        let blob = format!("{{\"key\":\"{}\"}} {{\"c\":\"{}\"}}", hex(&k), hex(&c)).into_bytes();
        let found = confirmations(&[("f".into(), blob)], &g);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("opens a stored commitment"));
    }

    #[test]
    fn a_commitment_without_its_key_confirms_nothing() {
        let g = guess();
        let c = commit::commit(&[7u8; 32], &canon::encode(&g.content));
        let blob = format!("{{\"c\":\"{}\"}}", hex(&c)).into_bytes();
        assert!(confirmations(&[("f".into(), blob)], &g).is_empty());
    }
}
