//! LANE-01's routing of a run's event: to the run seat of the room whose
//! standing branch link names the branch of the run's latest preceding
//! event that names one, while that seat is a member with work (its
//! role, no mute); else to the run's seat in the personal room, never
//! refused and never an error.
//!
//! This routes a run's event only. An act, a tombstone and an event with
//! no run go to a device seat, which is not decided here.

use record::id::RoomId;

use crate::branch::{BranchIdentity, BranchLink, resolve};

/// The seat a run's event is written to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatTarget {
    /// The run's seat in this room.
    RunSeat(RoomId),
    /// The run's seat in the personal room.
    PersonalRoomSeat,
}

/// One of this node's principal's rooms, and whether the run's seat in
/// it is a member with work in the room state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSeatState {
    /// The room.
    pub room: RoomId,
    /// Whether the run's seat there has a role and no mute.
    pub member_with_work: bool,
}

/// The seat a run's event goes to. `branch` is the branch of the run's
/// latest preceding event that names one; `rooms` are this node's
/// principal's rooms with the run's seat in each; `links` are the branch
/// links known, and `causal(a, b)` says `a` is before `b`.
#[must_use]
pub fn seat_for_run_event(
    branch: Option<&BranchIdentity>,
    rooms: &[RunSeatState],
    links: &[BranchLink],
    causal: &dyn Fn(&BranchLink, &BranchLink) -> bool,
) -> SeatTarget {
    let Some(branch) = branch else {
        return SeatTarget::PersonalRoomSeat;
    };
    let ours = links
        .iter()
        .filter(|l| &l.branch == branch && rooms.iter().any(|r| r.room == l.room));
    let Some(standing) = resolve(ours, causal).standing else {
        return SeatTarget::PersonalRoomSeat;
    };
    let works = rooms
        .iter()
        .any(|r| r.room == standing.room && r.member_with_work);
    if works {
        SeatTarget::RunSeat(standing.room)
    } else {
        SeatTarget::PersonalRoomSeat
    }
}

#[cfg(test)]
mod tests;
