use std::cell::Cell;

use super::{Deadline, check};
use crate::Error;

#[test]
fn a_closure_is_a_deadline() {
    assert!(!(|| false).expired());
    assert!((|| true).expired());
}

#[test]
fn a_deadline_that_has_not_passed_lets_the_walk_go_on() {
    assert!(check(&|| false).is_ok());
}

#[test]
fn a_deadline_that_has_passed_stops_the_walk() {
    assert!(matches!(check(&|| true), Err(Error::Deadline)));
}

#[test]
fn a_deadline_is_asked_each_time() {
    let asked = Cell::new(0);
    let deadline = || {
        asked.set(asked.get() + 1);
        asked.get() > 2
    };
    assert!(check(&deadline).is_ok());
    assert!(check(&deadline).is_ok());
    assert!(check(&deadline).is_err());
}
