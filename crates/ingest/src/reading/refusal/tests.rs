use record::digest::Digest;
use record::id::WriterId;
use store::source::Span;

use super::*;

#[test]
fn zeros_are_fed_in_pieces_and_hash_as_the_zeros_they_are() {
    let mut running = Zeroed::new(&[]);

    feed_zeroes(&mut running, 10_000);

    assert_eq!(running.digest(), Digest::of(&[&[0; 10_000]]));
}

struct Failing;

impl key::random::Entropy for Failing {
    fn fill(&mut self, _: &mut [u8]) -> Result<(), key::KeyError> {
        Err(key::KeyError::Entropy("none".into()))
    }
}

fn over() -> crate::read::bounded::Refused {
    crate::read::bounded::Refused {
        offset: 2,
        length: 5,
    }
}

#[test]
fn a_line_over_the_ceiling_is_an_unparsed_event_redacted_whole_with_its_place_recorded() {
    let writer = WriterId::derive(&[1; 32]);
    let mut plan = Plan::default();

    plan.pass_over(over(), 7, writer, &mut key::random::OsEntropy)
        .unwrap();

    let event = &plan.events[0];
    assert_eq!(event.kind, record::event::EventKind::UnparsedLine);
    assert_eq!(event.provenance, record::event::Provenance::Unparsed);
    assert_eq!(
        event.content,
        serde_json::json!({"line": 7, "text": "[REDACTED:transcript_line_too_large]", "redactions": 1})
    );
    assert_eq!(plan.places[0].length, 6);
    assert_eq!(plan.refused_places, plan.places);
    assert_eq!(plan.ends, vec![(writer, true, 8)]);
    assert_eq!((plan.unparsed, plan.redactions.len()), (1, 1));
    assert_eq!(plan.redactions[0], "transcript_line_too_large");
}

#[test]
fn a_line_over_the_ceiling_whose_key_cannot_be_drawn_is_an_error() {
    let mut plan = Plan::default();

    let err = plan
        .pass_over(over(), 7, WriterId::derive(&[1; 32]), &mut Failing)
        .unwrap_err();

    assert!(matches!(err, crate::IngestError::Key(_)), "{err}");
    assert!(plan.events.is_empty() && plan.over.is_none());
}

#[test]
fn a_line_over_the_ceiling_is_passed_as_the_zeros_its_span_zeroes() {
    let span = [Span {
        offset: 2,
        length: 6,
    }];
    let mut plan = Plan::default();
    plan.pass_over(
        over(),
        7,
        WriterId::derive(&[1; 32]),
        &mut key::random::OsEntropy,
    )
    .unwrap();
    let mut running = Zeroed::new(&span);
    running.update(b"ab");
    let start = Position {
        offset: 2,
        prefix: running.digest(),
        lines: 6,
        seen: 9,
    };
    let mut whole = Zeroed::new(&span);

    hash_through(
        &mut whole,
        b"ab",
        0,
        &Position {
            offset: 0,
            ..start.clone()
        },
        8,
        &plan,
    );
    let at_line = &mut running;
    hash_through(at_line, b"ab", 0, &start, 8, &plan);

    assert_eq!(whole.digest(), Digest::of(&[b"ab", &[0; 6]]));
    assert_eq!(running.digest(), Digest::of(&[b"ab", &[0; 6]]));
}

#[test]
fn a_transaction_ending_before_the_line_over_the_ceiling_hashes_only_its_bytes() {
    let mut plan = Plan::default();
    plan.pass_over(
        over(),
        7,
        WriterId::derive(&[1; 32]),
        &mut key::random::OsEntropy,
    )
    .unwrap();
    let mut running = Zeroed::new(&[]);
    let start = Position {
        offset: 0,
        prefix: Digest::ZERO,
        lines: 0,
        seen: 9,
    };

    hash_through(&mut running, b"ab", 0, &start, 2, &plan);

    assert_eq!(running.digest(), Digest::of(&[b"ab"]));
}

#[test]
fn a_refusal_no_recorded_line_follows_moves_the_position_past_it() {
    let span = [Span {
        offset: 2,
        length: 3,
    }];
    let plan = Plan {
        refused_to: Some((3, 5)),
        ..Plan::default()
    };
    let mut running = Zeroed::new(&span);
    running.update(b"ab");
    let start = Position {
        offset: 2,
        prefix: running.digest(),
        lines: 2,
        seen: 9,
    };

    let moved = move_past(&mut running, b"abxyz", 0, start, &plan, 9);

    assert_eq!((moved.offset, moved.lines), (5, 3));
    assert_eq!(moved.prefix, Digest::of(&[b"ab", &[0; 3]]));
}
