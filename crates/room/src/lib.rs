//! Where a room's branches and a run's events belong (LANE-01,
//! LANE-31). Pure functions of what the caller hands in: no clock, no
//! randomness, no file, so every result is a function of the set of
//! acts given (I10).
//!
//! - [`branch`]: a branch's identity, the branch links that name it,
//!   and which link stands when several name one branch.
//! - [`route`]: the seat a run's event is written to.

pub mod branch;
pub mod route;
