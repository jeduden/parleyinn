//! A room's roles and capabilities (LANE-16), as far as M1 reaches: the
//! personal room, where the owner's device seat has every capability but
//! writing a room summary, and each run's seat, the seat ingest starts
//! included, has the contributor role. Every other seat is a viewer.
//!
//! A seat's capabilities are a deterministic function of its seat
//! certificate and this node's own device keys: no model decides one.
//! Role assignments, appointments, mutes and other principals' seats
//! land with rooms (M7).

use std::collections::BTreeSet;

use key::certificate::{SeatCertificate, SeatKind};
use key::signing::PublicKey;

/// A room capability, from LANE-16's closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Capability {
    Read,
    RoleRequest,
    Post,
    Link,
    Pin,
    Edit,
    Unpin,
    ListRemoval,
    SummaryRequest,
    RoomSummary,
    Work,
    Present,
    Pick,
    Kick,
    Bar,
    Unbar,
    Mute,
    Unmute,
    SetTitle,
    SetLabels,
    SetAssignments,
}

/// Every capability, in the order LANE-16 lists them.
pub const CAPABILITIES: [Capability; 21] = [
    Capability::Read,
    Capability::RoleRequest,
    Capability::Post,
    Capability::Link,
    Capability::Pin,
    Capability::Edit,
    Capability::Unpin,
    Capability::ListRemoval,
    Capability::SummaryRequest,
    Capability::RoomSummary,
    Capability::Work,
    Capability::Present,
    Capability::Pick,
    Capability::Kick,
    Capability::Bar,
    Capability::Unbar,
    Capability::Mute,
    Capability::Unmute,
    Capability::SetTitle,
    Capability::SetLabels,
    Capability::SetAssignments,
];

/// A room role (LANE-16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Viewer,
    Contributor,
    Moderator,
}

impl Role {
    /// Its capabilities. A contributor edits and unpins only its own
    /// pins, which the pin acts check.
    #[must_use]
    pub fn capabilities(self) -> BTreeSet<Capability> {
        use Capability::{
            Bar, Edit, Kick, Link, ListRemoval, Mute, Pick, Pin, Post, Present, Read, RoleRequest,
            SetAssignments, SetLabels, SetTitle, SummaryRequest, Unbar, Unmute, Unpin, Work,
        };
        let list: &[Capability] = match self {
            Self::Viewer => &[Read, RoleRequest, SummaryRequest],
            Self::Contributor => &[
                Read,
                SummaryRequest,
                Post,
                Link,
                Present,
                Pin,
                Edit,
                Unpin,
                Work,
            ],
            Self::Moderator => &[
                Read,
                SummaryRequest,
                Post,
                Link,
                Present,
                Pin,
                Edit,
                Unpin,
                Work,
                ListRemoval,
                Kick,
                Bar,
                Unbar,
                Mute,
                Unmute,
                Pick,
                SetTitle,
                SetLabels,
                SetAssignments,
            ],
        };
        list.iter().copied().collect()
    }
}

/// Whether the seat `cert` certifies is one of the owner's device seats:
/// a device seat a device key of this node's own certified.
fn owners_device_seat(cert: &SeatCertificate, own: &[PublicKey]) -> bool {
    cert.seat_kind == SeatKind::Device && own.contains(&cert.certifying_key)
}

/// The role of the seat `cert` certifies, in the personal room, for the
/// node whose device keys are `own`: none for the owner's device seat,
/// which holds every capability by being the owner's, the contributor
/// role for a run's seat a key of `own` certified, the seat ingest
/// starts included, and the viewer role for any other seat.
#[must_use]
pub fn seat_role(cert: &SeatCertificate, own: &[PublicKey]) -> Option<Role> {
    if owners_device_seat(cert, own) {
        return None;
    }
    if cert.seat_kind == SeatKind::Run && own.contains(&cert.certifying_key) {
        return Some(Role::Contributor);
    }

    Some(Role::Viewer)
}

/// The capabilities of the seat `cert` certifies, in the personal room,
/// for the node whose device keys are `own`. The owner's device seat has
/// every capability but writing a room summary, which only the
/// facilitator writes.
#[must_use]
pub fn seat_capabilities(cert: &SeatCertificate, own: &[PublicKey]) -> BTreeSet<Capability> {
    match seat_role(cert, own) {
        Some(role) => role.capabilities(),
        None => CAPABILITIES
            .into_iter()
            .filter(|c| *c != Capability::RoomSummary)
            .collect(),
    }
}

#[cfg(test)]
mod tests;
