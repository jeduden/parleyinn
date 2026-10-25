//! Seat certificates (PRV-11): the device key's signature over a seat
//! key, scoped to the seat's room and naming its seat kind and whether
//! it is an ingest seat, which the seat key countersigns. ParleyInn shows
//! a seat's kind only from its certificate.

use record::canonical;
use record::digest::Digest;
use record::id::{RoomId, SeatId, WriterId};
use serde_json::{Value, json};

use crate::signing::verify;
pub use crate::signing::{DeviceKey, KeyError, PublicKey, SeatKey, Signature, SignatureKind};

/// A seat's kind (PRV-11). An ingest seat is a run seat whose
/// certificate marks it so.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatKind {
    Run,
    Device,
}

impl SeatKind {
    /// The kind's name in a certificate.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Device => "device",
        }
    }

    /// The kind `name` names.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        [Self::Run, Self::Device]
            .into_iter()
            .find(|k| k.as_str() == name)
    }
}

/// A seat certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatCertificate {
    pub room: RoomId,
    pub seat_key: PublicKey,
    pub seat_kind: SeatKind,
    pub ingest_seat: bool,
    /// The device key that certified the seat key.
    pub certifying_key: PublicKey,
    /// The certifying_key's signature over the body.
    pub signature: Signature,
    /// The seat key's signature over the body and the certifying_key's
    /// signature.
    pub countersignature: Signature,
}

impl SeatCertificate {
    /// `device`'s certificate for `seat` in `room`, countersigned by
    /// `seat`.
    ///
    /// # Errors
    ///
    /// Fails only if a body has no canonical encoding, which a
    /// certificate's never lacks.
    pub fn issue(
        device: &DeviceKey,
        seat: &SeatKey,
        room: RoomId,
        seat_kind: SeatKind,
        ingest_seat: bool,
    ) -> Result<Self, KeyError> {
        let (seat_key, certifying_key) = (seat.public(), device.public());
        let body = body(room, seat_key, seat_kind, ingest_seat, certifying_key);
        let kind = SignatureKind::SeatCertificate;
        let signature = device.sign(kind, &body)?;
        let countersignature = seat.sign(kind, &countersigned(&body, &signature))?;

        Ok(Self {
            room,
            seat_key,
            seat_kind,
            ingest_seat,
            certifying_key,
            signature,
            countersignature,
        })
    }

    /// Checks both signatures.
    ///
    /// # Errors
    ///
    /// Fails when either does not verify.
    pub fn check(&self) -> Result<(), KeyError> {
        let body = body(
            self.room,
            self.seat_key,
            self.seat_kind,
            self.ingest_seat,
            self.certifying_key,
        );
        let kind = SignatureKind::SeatCertificate;
        verify(&self.certifying_key, kind, &body, &self.signature)?;
        verify(
            &self.seat_key,
            kind,
            &countersigned(&body, &self.signature),
            &self.countersignature,
        )
    }

    /// The certified seat's id.
    #[must_use]
    pub fn seat_id(&self) -> SeatId {
        SeatId::derive(&self.room, self.seat_key.as_bytes())
    }

    /// The certified seat's writer id.
    #[must_use]
    pub fn writer_id(&self) -> WriterId {
        WriterId::derive(self.seat_key.as_bytes())
    }

    /// The certificate as a JSON value.
    #[must_use]
    pub fn to_value(&self) -> Value {
        json!({
            "room": self.room.to_string(),
            "seat_key": self.seat_key.to_string(),
            "seat_kind": self.seat_kind.as_str(),
            "ingest_seat": self.ingest_seat,
            "certifying_key": self.certifying_key.to_string(),
            "signature": self.signature.to_string(),
            "countersignature": self.countersignature.to_string(),
        })
    }

    /// The certificate a JSON value holds.
    ///
    /// # Errors
    ///
    /// Fails, naming the field, on a value that holds none.
    pub fn from_value(value: &Value) -> Result<Self, KeyError> {
        let text = |field: &str| {
            value[field]
                .as_str()
                .ok_or_else(|| KeyError::Malformed(format!("seat certificate: no {field}")))
        };
        let named = |field: &str, e: KeyError| {
            KeyError::Malformed(format!("seat certificate {field}: {e}"))
        };
        let room = RoomId::parse(text("room")?)
            .map_err(|e| KeyError::Malformed(format!("seat certificate room: {e}")))?;
        let seat_kind = SeatKind::parse(text("seat_kind")?).ok_or_else(|| {
            KeyError::Malformed("seat certificate: seat_kind is neither run nor device".to_owned())
        })?;
        let ingest_seat = value["ingest_seat"]
            .as_bool()
            .ok_or_else(|| KeyError::Malformed("seat certificate: no ingest_seat".to_owned()))?;

        Ok(Self {
            room,
            seat_key: PublicKey::from_hex(text("seat_key")?).map_err(|e| named("seat_key", e))?,
            seat_kind,
            ingest_seat,
            certifying_key: PublicKey::from_hex(text("certifying_key")?)
                .map_err(|e| named("certifying_key", e))?,
            signature: Signature::from_hex(text("signature")?)
                .map_err(|e| named("signature", e))?,
            countersignature: Signature::from_hex(text("countersignature")?)
                .map_err(|e| named("countersignature", e))?,
        })
    }

    /// Its digest: SHA-256 over its canonical encoding, which its
    /// certificate record holds. A certificate holds only strings and a
    /// bool, which always encode.
    #[must_use]
    pub fn digest(&self) -> Digest {
        Digest::of(&[canonical::encode(&self.to_value())
            .unwrap_or_default()
            .as_bytes()])
    }
}

/// What the certifying_key signs.
fn body(
    room: RoomId,
    seat_key: PublicKey,
    seat_kind: SeatKind,
    ingest_seat: bool,
    certifying_key: PublicKey,
) -> Value {
    json!({
        "room": room.to_string(),
        "seat_key": seat_key.to_string(),
        "seat_kind": seat_kind.as_str(),
        "ingest_seat": ingest_seat,
        "certifying_key": certifying_key.to_string(),
    })
}

/// What the seat key countersigns: the body and the certifying_key's
/// signature.
fn countersigned(body: &Value, signature: &Signature) -> Value {
    let mut out = body.clone();
    out["signature"] = json!(signature.to_string());

    out
}

#[cfg(test)]
mod tests;
