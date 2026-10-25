//! Key events (PRV-12): the structural events a node records in its
//! device seat for its device keys, forming key event logs. The slice
//! records the genesis log's inception, `minted`; rotation, recovery
//! and the other kinds land with their acts.
//!
//! A node identity change event (REC-24) is no key event, but stands
//! just before one: the new key's `minted`, in the new device seat.
//!
//! A key event carries only its kind, the key's fingerprint, the
//! digest of its recovery key's public key, the digest of the key
//! event it follows, the fingerprint of each key it replaces and its
//! signatures.

use record::canonical;
use record::chain::SeenHead;
use record::digest::Digest;
use record::id::{Address, Seq, WriterId};
use serde_json::{Value, json};

use crate::signing::verify;
pub use crate::signing::{DeviceKey, KeyError, PublicKey, Signature, SignatureKind};

/// The kinds of key event (PRV-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEventKind {
    Minted,
    TokenKeyAdopted,
    MintedAnew,
    Rotated,
    Recovered,
    Revoked,
}

impl KeyEventKind {
    /// The kind's name in a key event.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minted => "minted",
            Self::TokenKeyAdopted => "token_key_adopted",
            Self::MintedAnew => "minted_anew",
            Self::Rotated => "rotated",
            Self::Recovered => "recovered",
            Self::Revoked => "revoked",
        }
    }

    /// The kind `name` names.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        [
            Self::Minted,
            Self::TokenKeyAdopted,
            Self::MintedAnew,
            Self::Rotated,
            Self::Recovered,
            Self::Revoked,
        ]
        .into_iter()
        .find(|k| k.as_str() == name)
    }
}

/// A key event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub kind: KeyEventKind,
    /// The key's fingerprint.
    pub key: Digest,
    /// The digest of its recovery key's public key.
    pub recovery: Digest,
    /// The digest of the key event it follows; none for an inception.
    pub follows: Option<Digest>,
    /// The fingerprint of each key it replaces.
    pub replaces: Vec<Digest>,
    pub signatures: Vec<Signature>,
}

impl KeyEvent {
    /// The inception `minted` of `device`'s key event log, committing to
    /// `recovery`, signed by `device`.
    ///
    /// # Errors
    ///
    /// Fails only if its body has no canonical encoding, which a key
    /// event's never lacks.
    pub fn minted(device: &DeviceKey, recovery: &PublicKey) -> Result<Self, KeyError> {
        let mut event = Self {
            kind: KeyEventKind::Minted,
            key: device.public().fingerprint(),
            recovery: recovery.digest(),
            follows: None,
            replaces: Vec::new(),
            signatures: Vec::new(),
        };
        event.signatures = vec![device.sign(SignatureKind::KeyEvent, &event.body())?];

        Ok(event)
    }

    /// Checks that it is a `minted` of `key`: an inception that follows
    /// and replaces nothing, signed once, by `key`.
    ///
    /// # Errors
    ///
    /// Fails when it is not.
    pub fn check_minted(&self, key: &PublicKey) -> Result<(), KeyError> {
        let malformed = |why: &str| Err(KeyError::Malformed(format!("minted key event: {why}")));
        if self.kind != KeyEventKind::Minted || self.follows.is_some() || !self.replaces.is_empty()
        {
            return malformed("not an inception");
        }
        if self.key != key.fingerprint() {
            return malformed("it names another key");
        }
        let [signature] = self.signatures.as_slice() else {
            return malformed("want one signature");
        };

        verify(key, SignatureKind::KeyEvent, &self.body(), signature)
    }

    /// What its signatures sign: everything but them.
    fn body(&self) -> Value {
        json!({
            "kind": self.kind.as_str(),
            "key": self.key.to_hex(),
            "recovery": self.recovery.to_hex(),
            "follows": self.follows.map(|d| d.to_hex()),
            "replaces": self.replaces.iter().map(Digest::to_hex).collect::<Vec<_>>(),
        })
    }

    /// The key event as a JSON value: its body and its signatures.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut out = self.body();
        out["signatures"] = json!(
            self.signatures
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );

        out
    }

    /// The key event a JSON value holds.
    ///
    /// # Errors
    ///
    /// Fails, naming the field, on a value that holds none.
    pub fn from_value(value: &Value) -> Result<Self, KeyError> {
        let malformed = |field: &str| KeyError::Malformed(format!("key event: bad {field}"));
        let digest = |field: &str, v: &Value| {
            v.as_str()
                .and_then(|t| Digest::from_hex(t).ok())
                .ok_or_else(|| malformed(field))
        };
        let list = |field: &str| value[field].as_array().ok_or_else(|| malformed(field));
        let kind = value["kind"]
            .as_str()
            .and_then(KeyEventKind::parse)
            .ok_or_else(|| malformed("kind"))?;
        let follows = match &value["follows"] {
            Value::Null => None,
            v => Some(digest("follows", v)?),
        };

        Ok(Self {
            kind,
            key: digest("key", &value["key"])?,
            recovery: digest("recovery", &value["recovery"])?,
            follows,
            replaces: list("replaces")?
                .iter()
                .map(|v| digest("replaces", v))
                .collect::<Result<_, _>>()?,
            signatures: list("signatures")?
                .iter()
                .map(|v| {
                    v.as_str()
                        .and_then(|t| Signature::from_hex(t).ok())
                        .ok_or_else(|| malformed("signatures"))
                })
                .collect::<Result<_, _>>()?,
        })
    }

    /// Its digest: SHA-256 over its canonical encoding, which the key
    /// event after it names. A key event holds only strings, which
    /// always encode.
    #[must_use]
    pub fn digest(&self) -> Digest {
        Digest::of(&[canonical::encode(&self.to_value())
            .unwrap_or_default()
            .as_bytes()])
    }
}

/// A node identity change event (REC-24, PRV-12): it carries only the
/// fingerprints of the device key it replaces and of the new one, and
/// the chain head of the device seat writer that recorded the newest
/// key event of the key event log it leaves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeIdentityChange {
    /// The fingerprint of the device key it replaces.
    pub replaces: Digest,
    /// The fingerprint of the new device key.
    pub key: Digest,
    /// The head of the writer that recorded the newest key event of the
    /// key event log it leaves.
    pub chain_head: SeenHead,
}

impl NodeIdentityChange {
    /// The event as a JSON value.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let head = &self.chain_head;
        json!({
            "replaces": self.replaces.to_hex(),
            "key": self.key.to_hex(),
            "chain_head": {
                "writer": head.address.writer.to_string(),
                "seq": head.address.seq.get(),
                "head": head.head.to_hex(),
            },
        })
    }

    /// The node identity change event a JSON value holds.
    ///
    /// # Errors
    ///
    /// Fails, naming the field, on a value that holds none.
    pub fn from_value(value: &Value) -> Result<Self, KeyError> {
        let malformed =
            |field: &str| KeyError::Malformed(format!("node identity change event: bad {field}"));
        let digest = |field: &str, v: &Value| {
            v.as_str()
                .and_then(|t| Digest::from_hex(t).ok())
                .ok_or_else(|| malformed(field))
        };
        let head = &value["chain_head"];
        let writer = head["writer"]
            .as_str()
            .and_then(|w| WriterId::parse(w).ok())
            .ok_or_else(|| malformed("chain_head"))?;
        let seq = head["seq"]
            .as_u64()
            .and_then(Seq::new)
            .ok_or_else(|| malformed("chain_head"))?;

        Ok(Self {
            replaces: digest("replaces", &value["replaces"])?,
            key: digest("key", &value["key"])?,
            chain_head: SeenHead::new(
                Address::new(writer, seq),
                digest("chain_head", &head["head"])?,
            ),
        })
    }
}

#[cfg(test)]
mod tests;
