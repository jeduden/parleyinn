use super::*;
use record::id::{Seq, WriterId};

fn address() -> Address {
    Address::new(WriterId::derive(&[1; 32]), Seq::FIRST)
}

const LINE: &str =
    r#"{"type":"user","uuid":"u1","sessionId":"parent","promptId":"p1","text":"hi"}"#;
const FORKED: &str =
    r#"{"type":"user","uuid":"u1","sessionId":"fork","promptId":"p2","text":"hi"}"#;
const OTHER: &str = r#"{"type":"user","uuid":"u1","sessionId":"fork","text":"bye"}"#;

fn owner(fork: bool) -> Owner {
    Owner {
        address: address(),
        provenance: Provenance::HarnessText,
        text: LINE.into(),
        fork,
    }
}

fn facts(owner: Option<Owner>, fork: bool) -> Facts {
    Facts {
        owner,
        fork,
        in_sibling: false,
        held_since: None,
        now: 100_000,
    }
}

#[test]
fn a_fork_s_repeated_line_refers_to_its_owner_s_event() {
    assert_eq!(
        decide(&facts(Some(owner(false)), true), FORKED),
        Decision::Reference {
            owner: address(),
            provenance: Provenance::HarnessText,
            rule: Rule::ForkObservation,
        }
    );
}

#[test]
fn with_no_fork_observation_the_first_recorder_owns_the_line() {
    let Decision::Reference { rule, .. } = decide(&facts(Some(owner(false)), false), FORKED) else {
        panic!("no reference");
    };
    assert_eq!(rule, Rule::FirstRecorded);
}

#[test]
fn other_content_under_the_same_uuid_is_a_conflict() {
    assert_eq!(
        decide(&facts(Some(owner(false)), true), OTHER),
        Decision::Conflict
    );
}

#[test]
fn a_line_a_released_fork_holds_stays_the_owning_run_s_own() {
    assert_eq!(
        decide(&facts(Some(owner(true)), false), LINE),
        Decision::Own
    );
}

#[test]
fn a_fork_s_line_read_before_its_owner_s_is_held_up_to_30_s() {
    let mut f = facts(None, true);
    f.in_sibling = true;
    assert_eq!(decide(&f, FORKED), Decision::Hold);
    f.held_since = Some(f.now - HOLD_MS + 1);
    assert_eq!(decide(&f, FORKED), Decision::Hold);
    f.held_since = Some(f.now - HOLD_MS);
    assert_eq!(decide(&f, FORKED), Decision::Released);
}

#[test]
fn a_held_line_whose_owner_s_transcript_is_gone_is_released() {
    let mut f = facts(None, true);
    f.held_since = Some(f.now);
    assert_eq!(decide(&f, FORKED), Decision::Released);
    f.held_since = None;
    assert_eq!(decide(&f, FORKED), Decision::Own);
    assert_eq!(decide(&facts(None, false), LINE), Decision::Own);
}

#[test]
fn content_compares_without_the_harness_session_s_fields() {
    assert!(same_content(LINE, FORKED));
    assert!(!same_content(LINE, OTHER));
    assert!(same_content("not json", "not json"));
    assert!(!same_content("not json", LINE));
    assert!(same_content("[1]", "[1]"));
}
