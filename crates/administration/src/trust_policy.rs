//! The default trust policy (PRV-02, I2): the trust level of an event,
//! derived per event from what the record and the node's key set say of
//! it, never stored with it (I10). It trusts only the trusted sources:
//! on the recording node, its `operator` events and the structural
//! events it records itself, the `harness_meta` its launcher signed,
//! and, while the deployment mode is `interactive` and no run leaves R3
//! or R8 open, the `user` events matched to a prompt whose hook handler
//! found the attended mark. Every other event, every `ingested` one
//! included, is untrusted. A settings layer cannot widen this (PRV-05).
//!
//! Until OQ-40 provides a key that signs a sandbox state, every run
//! leaves R3 and R8 open (OWN-22), so no `user` event and no hook
//! observation is trusted, whatever the deployment mode. Stamps, trust
//! grants and the acts of a device key a principal key certified stay
//! out of the policy until PRV-10 ships (M7). Nothing here reads a
//! clock or randomness.

use record::event::{EventKind, Provenance};
use record::id::Address;

use crate::origin::Origin;

/// An event's trust level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    Trusted,
    Untrusted,
}

impl TrustLevel {
    /// The level's name, as the envelope shows it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
        }
    }
}

/// Whether a run on the node leaves R3 or R8 open: until OQ-40 every
/// run does (OWN-22), so no caller may say otherwise.
const RISK_OPEN: bool = true;

/// Whether a `user` event is trusted: only in `interactive` mode, matched
/// to a prompt whose hook handler found the attended mark, while no run
/// leaves R3 or R8 open (PRV-02, PRV-08).
fn user_trusted(input: &Input<'_>, risk_open: bool) -> bool {
    input.interactive && input.prompt_matched && input.attended && !risk_open
}

/// The structural events the core records itself. A hook observation, a
/// prompt match and a repeat record are derived from a transcript line or
/// a hook input, so they are trusted for their sanitized structural
/// fields alone (PRV-08, PRV-06).
const STRUCTURAL: [EventKind; 4] = [
    EventKind::KeyEvent,
    EventKind::CreateRoom,
    EventKind::CertificateRecord,
    EventKind::NodeIdentityChange,
];

/// What an artifact built from events inherits from them (PRV-06): the
/// events it `derives_from`, and the trust level it takes from theirs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Taint {
    pub derives_from: Vec<Address>,
    pub trust: TrustLevel,
}

impl Taint {
    /// The taint of an artifact made from `sources`, each an event's
    /// address and trust level. It is untrusted if any source is, or if
    /// it holds the free text of a cut or neutral principal act
    /// (OWN-11), whatever that act's event is trusted at. Its
    /// sanitized structural fields stay usable and are not part of it.
    #[must_use]
    pub fn of(sources: &[(Address, TrustLevel)], holds_act_free_text: bool) -> Self {
        let untrusted =
            holds_act_free_text || sources.iter().any(|(_, t)| *t == TrustLevel::Untrusted);

        Self {
            derives_from: sources.iter().map(|(a, _)| *a).collect(),
            trust: if untrusted {
                TrustLevel::Untrusted
            } else {
                TrustLevel::Trusted
            },
        }
    }
}

/// What the trust policy reads of an event.
#[derive(Debug, Clone, Copy)]
pub struct Input<'a> {
    pub provenance: &'a Provenance,
    pub kind: EventKind,
    /// Derived by [`crate::origin`]; `witnessed` means this node's own
    /// capture or core recorded it.
    pub origin: Origin,
    /// The event lies in a device seat's writer, as the certificate of
    /// its seat says (PRV-11): only one records the operator and
    /// structural events this policy trusts.
    pub device_seat: bool,
    /// A `harness_meta` event the launcher recorded and signed.
    pub launcher_signed: bool,
    /// The recorded deployment mode is `interactive`.
    pub interactive: bool,
    /// A `user` event matched to a prompt its hook handler got.
    pub prompt_matched: bool,
    /// That hook handler found the attended mark.
    pub attended: bool,
}

/// The trust level the policy gives the event `input` describes.
#[must_use]
pub fn trust_level(input: &Input<'_>) -> TrustLevel {
    if input.origin != Origin::Witnessed {
        return TrustLevel::Untrusted;
    }
    let trusted = match input.provenance {
        Provenance::Operator => input.device_seat,
        Provenance::Structural => input.device_seat && STRUCTURAL.contains(&input.kind),
        Provenance::HarnessMeta => input.launcher_signed,
        Provenance::User => user_trusted(input, RISK_OPEN),
        _ => false,
    };

    if trusted {
        TrustLevel::Trusted
    } else {
        TrustLevel::Untrusted
    }
}

#[cfg(test)]
mod tests;
