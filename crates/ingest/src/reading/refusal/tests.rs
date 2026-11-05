use record::digest::Digest;
use store::source::Span;

use super::*;

#[test]
fn zeros_are_fed_in_pieces_and_hash_as_the_zeros_they_are() {
    let mut running = Zeroed::new(&[]);

    feed_zeroes(&mut running, 10_000);

    assert_eq!(running.digest(), Digest::of(&[&[0; 10_000]]));
}

#[test]
fn a_line_over_the_ceiling_is_passed_as_the_zeros_its_span_zeroes() {
    let span = [Span {
        offset: 2,
        length: 6,
    }];
    let mut plan = Plan::default();
    plan.pass_over(
        crate::read::bounded::Refused {
            offset: 2,
            length: 5,
        },
        7,
    );
    let mut running = Zeroed::new(&span);
    running.update(b"ab");
    let start = Position {
        offset: 2,
        prefix: running.digest(),
        lines: 6,
        seen: 9,
    };

    let moved = move_past(&mut running, b"ab", 0, start, &plan, 9);

    assert_eq!((moved.offset, moved.lines, moved.seen), (8, 7, 9));
    assert_eq!(moved.prefix, Digest::of(&[b"ab", &[0; 6]]));
}
