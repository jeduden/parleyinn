//! Ed25519 keys of each kind, and the signatures they make. Every
//! signature names its kind (PRV-12): it signs a label, the kind's name
//! and the body's canonical encoding, so a signature of one kind never
//! verifies as another.

use std::fmt;

use ed25519_dalek::{Signer as _, SigningKey, VerifyingKey};
use record::canonical;
use record::digest::{Digest, hex, hmac_sha256, unhex};
use serde_json::Value;

use crate::key_store::{Entry, KeyStore};
use crate::random::{Entropy, draw};

/// The label every signed message starts with.
const LABEL: &[u8] = b"parinn signature v1\0";
/// The label a key's fingerprint is derived under.
const FINGERPRINT: &[u8] = b"parinn key fingerprint v1\0";

/// What can go wrong with a key or a signature.
#[derive(Debug)]
pub enum KeyError {
    /// The random source failed.
    Entropy(String),
    /// A signed body has no canonical encoding.
    Encoding(canonical::Error),
    /// A signature of this kind does not verify.
    Signature(SignatureKind),
    /// A key, signature, certificate or key event is not well formed.
    Malformed(String),
    /// A key file cannot be written or read.
    File {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    /// A key file is readable by others.
    Permissions { path: std::path::PathBuf, mode: u32 },
    /// The key store entry holds no key (SEC-10).
    Missing { entry: String },
    /// The key store entry already holds a key, which is never replaced.
    Occupied { entry: String },
    /// The key store is locked or cannot be read: not a missing key
    /// (PRV-12).
    Unavailable { entry: String, reason: String },
}

impl KeyError {
    /// Whether the key is missing from its entry, which, unlike a key
    /// store that is locked or cannot be read, may be minted anew
    /// (PRV-12, SEC-27).
    #[must_use]
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing { .. })
    }

    /// Whether the key store is locked or cannot be read: a key file that
    /// cannot be read, or that others can read, included (PRV-12).
    #[must_use]
    pub fn is_unavailable(&self) -> bool {
        matches!(
            self,
            Self::Unavailable { .. } | Self::File { .. } | Self::Permissions { .. }
        )
    }
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entropy(e) => write!(f, "draw random bytes: {e}"),
            Self::Encoding(e) => write!(f, "{e}"),
            Self::Signature(kind) => write!(f, "a {} signature does not verify", kind.as_str()),
            Self::Malformed(what) => write!(f, "malformed {what}"),
            Self::File { path, source } => write!(f, "key file {}: {source}", path.display()),
            Self::Permissions { path, mode } => write!(
                f,
                "key file {} is readable by others (mode {mode:o}); want 600",
                path.display()
            ),
            Self::Missing { entry } => write!(f, "no key in the key store entry {entry}"),
            Self::Occupied { entry } => {
                write!(f, "the key store entry {entry} already holds a key")
            }
            Self::Unavailable { entry, reason } => {
                write!(f, "the key store entry {entry} is unavailable: {reason}")
            }
        }
    }
}

impl std::error::Error for KeyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Encoding(e) => Some(e),
            Self::File { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// The kinds of signature ParleyInn makes so far; PRV-12 names confirm,
/// choose and reject besides, which land with their acts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureKind {
    KeyEvent,
    SeatCertificate,
}

impl SignatureKind {
    /// The kind's name, which every signature of it signs.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeyEvent => "key_event",
            Self::SeatCertificate => "seat_certificate",
        }
    }
}

/// An Ed25519 public key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// The public key these bytes are; whether a point lies behind them
    /// is checked when a signature is verified.
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Its bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Its fingerprint: SHA-256 over a label of its own and the key.
    #[must_use]
    pub fn fingerprint(&self) -> Digest {
        Digest::of(&[FINGERPRINT, &self.0])
    }

    /// Its digest: SHA-256 over the key's bytes, which an inception and
    /// a rotation commit to for their recovery key (PRV-12).
    #[must_use]
    pub fn digest(&self) -> Digest {
        Digest::of(&[&self.0])
    }

    /// The public key 64 lowercase hex digits spell.
    ///
    /// # Errors
    ///
    /// Fails on any other text.
    pub fn from_hex(text: &str) -> Result<Self, KeyError> {
        unhex(text)
            .map(Self)
            .map_err(|e| KeyError::Malformed(format!("public key: {e}")))
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex(&self.0))
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({self})")
    }
}

/// An Ed25519 signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; 64]);

impl Signature {
    /// The signature 128 lowercase hex digits spell.
    ///
    /// # Errors
    ///
    /// Fails on any other text.
    pub fn from_hex(text: &str) -> Result<Self, KeyError> {
        let malformed = |e: String| KeyError::Malformed(format!("signature: {e}"));
        if text.len() != 128 || !text.is_char_boundary(64) {
            return Err(malformed(format!(
                "want 128 hex digits, got {}",
                text.len()
            )));
        }
        let (a, b) = text.split_at(64);
        let (a, b): ([u8; 32], [u8; 32]) = (
            unhex(a).map_err(|e| malformed(e.to_string()))?,
            unhex(b).map_err(|e| malformed(e.to_string()))?,
        );
        let mut out = [0; 64];
        out[..32].copy_from_slice(&a);
        out[32..].copy_from_slice(&b);

        Ok(Self(out))
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex(&self.0))
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({self})")
    }
}

/// The message a signature of `kind` over `body` signs.
fn message(kind: SignatureKind, body: &Value) -> Result<Vec<u8>, KeyError> {
    let canonical = canonical::encode(body).map_err(KeyError::Encoding)?;
    let mut out = LABEL.to_vec();
    out.extend_from_slice(kind.as_str().as_bytes());
    out.push(0);
    out.extend_from_slice(canonical.as_bytes());

    Ok(out)
}

/// Checks that `signature` is `public`'s signature of `kind` over
/// `body`.
///
/// # Errors
///
/// Fails when it is not, or when `body` has no canonical encoding.
pub fn verify(
    public: &PublicKey,
    kind: SignatureKind,
    body: &Value,
    signature: &Signature,
) -> Result<(), KeyError> {
    let message = message(kind, body)?;
    let key = VerifyingKey::from_bytes(&public.0).map_err(|_| KeyError::Signature(kind))?;
    key.verify_strict(
        &message,
        &ed25519_dalek::Signature::from_bytes(&signature.0),
    )
    .map_err(|_| KeyError::Signature(kind))
}

macro_rules! signing_key {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        pub struct $name(SigningKey);

        impl $name {
            /// A fresh key from `entropy`.
            ///
            /// # Errors
            ///
            /// Fails when `entropy` cannot supply 32 bytes.
            pub fn mint(entropy: &mut dyn Entropy) -> Result<Self, KeyError> {
                Ok(Self(SigningKey::from_bytes(&draw(entropy)?)))
            }

            /// The key whose seed 64 lowercase hex digits spell.
            ///
            /// # Errors
            ///
            /// Fails on any other text.
            pub fn from_seed(text: &str) -> Result<Self, KeyError> {
                unhex(text)
                    .map(|seed| Self(SigningKey::from_bytes(&seed)))
                    .map_err(|e| KeyError::Malformed(format!("key seed: {e}")))
            }

            /// Its seed as hex, for its key file alone.
            pub(crate) fn seed_hex(&self) -> String {
                hex(&self.0.to_bytes())
            }

            /// A secret only this key's holder knows, named by `label`:
            /// the same for the same key and label and unrelated to any
            /// other label's, so a node's local names derive from its key
            /// set and rebuild with it (I10).
            #[must_use]
            pub fn derive_secret(&self, label: &[u8]) -> [u8; 32] {
                let seed = self.0.to_bytes();
                *hmac_sha256(&seed, &[b"parinn derived secret v1\0", label]).as_bytes()
            }

            /// Its public half.
            #[must_use]
            pub fn public(&self) -> PublicKey {
                PublicKey(self.0.verifying_key().to_bytes())
            }

            /// Its signature of `kind` over `body`.
            ///
            /// # Errors
            ///
            /// Fails when `body` has no canonical encoding.
            pub fn sign(&self, kind: SignatureKind, body: &Value) -> Result<Signature, KeyError> {
                Ok(Signature(self.0.sign(&message(kind, body)?).to_bytes()))
            }

            /// Keeps it in `entry` of `store`, which must hold no key
            /// yet (SEC-10).
            ///
            /// # Errors
            ///
            /// Fails when the entry holds a key or the key store cannot
            /// be written.
            pub fn keep(&self, store: &dyn KeyStore, entry: &Entry) -> Result<(), KeyError> {
                store.put(entry, &self.seed_hex())
            }

            /// Reads it from `entry` of `store`.
            ///
            /// # Errors
            ///
            /// Fails when the entry holds no key, the key store cannot
            /// be read, or the entry holds no seed.
            pub fn fetch(store: &dyn KeyStore, entry: &Entry) -> Result<Self, KeyError> {
                Self::from_seed(&store.get(entry)?)
            }
        }

        impl fmt::Debug for $name {
            /// Its public half only: no log shows a private key.
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.public())
            }
        }
    };
}

signing_key!(
    /// A node's device key: it certifies the node's seat keys and signs
    /// its key events.
    DeviceKey
);
signing_key!(
    /// A seat's key: it countersigns its seat certificate, and later
    /// signs the seat's room acts and seals its writer.
    SeatKey
);
signing_key!(
    /// The key a key event commits to by its public key's digest,
    /// which signs only the rotation that follows (PRV-12).
    RecoveryKey
);

#[cfg(test)]
mod tests;
