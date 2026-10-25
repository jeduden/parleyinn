use super::*;
use envelope::{IntegrityStatus, Origin, Run, TrustLevel};
use record::id::WriterId;
use serde_json::json;

fn item(content: &str) -> Item {
    Item {
        address: "parinn:room/r/w/w/1".into(),
        writer: "w".into(),
        author: "s".into(),
        origin: Origin::Ingested,
        run: Run::Other,
        kind: "unparsed_line".into(),
        provenance: "unparsed".into(),
        trust: TrustLevel::Untrusted,
        flags: vec![],
        continuation: vec![],
        integrity: IntegrityStatus::Unsigned,
        score: None,
        content: content.into(),
    }
}

fn writer() -> WriterId {
    WriterId::derive(&[1; 32])
}

#[test]
fn the_cap_is_8000_estimated_tokens_less_the_envelope_around_the_items() {
    let budget = Budget::new();

    assert_eq!(CAP, 8_000);
    assert!(
        budget.left < CAP && budget.left > CAP - 800,
        "{}",
        budget.left
    );
    assert!(!budget.holds);
}

#[test]
fn an_item_that_fits_is_taken_whole_and_costs_its_estimate_and_a_comma() {
    let mut budget = Budget::new();
    let before = budget.left;
    let small = item("small");

    let Taken::Whole(taken) = budget.take(small.clone()) else {
        panic!("a small item fits");
    };

    assert_eq!(taken, small);
    assert_eq!(before - budget.left, small.tokens() + 1);
    assert!(budget.holds);
}

#[test]
fn an_item_that_does_not_fit_beside_others_waits_for_the_next_call() {
    let mut budget = Budget::new();
    let big = item(&"ab ".repeat(2_000));
    let Taken::Whole(_) = budget.take(big.clone()) else {
        panic!("fits")
    };
    let Taken::Whole(_) = budget.take(big.clone()) else {
        panic!("fits")
    };

    let waits = budget.take(item(&"ab ".repeat(6_000)));

    assert!(matches!(waits, Taken::Stop));
}

#[test]
fn the_first_item_past_the_cap_is_cut_to_fit_and_flagged() {
    let mut budget = Budget::new();
    let left = budget.left;
    let content = "ab ".repeat(30_000);

    let Taken::Cut(cut, chars) = budget.take(item(&content)) else {
        panic!("too big to fit");
    };

    assert_eq!(cut.content.chars().count(), chars);
    assert!(content.starts_with(&cut.content));
    assert_eq!(cut.continuation, [Continuation::Cut]);
    assert!(cut.flags.is_empty(), "a cut is no injection-pattern mark");
    assert!(item_cost(&cut) <= left, "it fits");
    let longer = Item {
        content: content.chars().take(chars + 1).collect(),
        ..cut
    };
    assert!(item_cost(&longer) > left, "one more character would not");
}

#[test]
fn a_cut_keeps_at_least_one_character_so_a_call_makes_progress() {
    let mut budget = Budget::new();
    budget.left = 0;

    let Taken::Cut(cut, chars) = budget.take(item("abc")) else {
        panic!("cut");
    };

    assert_eq!((cut.content.as_str(), chars), ("a", 1));
}

#[test]
fn a_range_is_one_string_or_two_addresses_of_one_writer() {
    let w = writer();
    let one = optional_range(&json!({"range": format!("{w}:3-9")}))
        .unwrap()
        .unwrap();
    let two = optional_range(&json!({"range": {"from": format!("{w}:3"), "to": format!("{w}:9")}}))
        .unwrap()
        .unwrap();

    assert_eq!(one, two);
    assert_eq!(optional_range(&json!({})), Ok(None));
    for bad in [
        json!({"range": 3}),
        json!({"range": "nope"}),
        json!({"range": {"from": format!("{w}:3")}}),
        json!({"range": {"from": format!("{w}:3"), "to": "nope"}}),
        json!({"range": {"from": format!("{w}:3-4"), "to": format!("{w}:9")}}),
        json!({"range": {"from": format!("{w}:9"), "to": format!("{w}:3")}}),
        json!({"range": {"from": format!("{w}:3"), "to": format!("{}:9", WriterId::derive(&[2; 32]))}}),
        json!({"range": {"from": format!("parinn:room/{}/w/{w}/3", record::id::RoomId::derive(&[1; 32], &[1; 16])), "to": format!("{w}:9")}}),
    ] {
        assert_eq!(optional_range(&bad), Err(BAD_RANGE.to_owned()), "{bad}");
    }
}

#[test]
fn what_a_range_asks_for_starts_at_its_first_event() {
    let w = writer();
    let range = Range::parse(&format!("{w}:2-5")).unwrap();

    let asked = asked_of(range);

    assert_eq!((asked.from.get(), asked.to.get(), asked.offset), (2, 5, 0));
    assert!(asked.named.is_none());
    assert_eq!(asked.writer, Writer::Id(w));
}

#[test]
fn a_cut_falls_on_a_character_boundary_of_multibyte_text() {
    for unit in ["東", "🚀", "é", "a東🚀é"] {
        let mut budget = Budget::new();
        budget.left = 40;
        let content = unit.repeat(2_000);

        let Taken::Cut(cut, chars) = budget.take(item(&content)) else {
            panic!("too big to fit");
        };

        assert!(chars > 0 && chars < 2_000 * unit.chars().count(), "{unit}");
        assert_eq!(cut.content.chars().count(), chars, "{unit}");
        assert!(content.starts_with(&cut.content), "{unit}");
        assert!(content.is_char_boundary(cut.content.len()), "{unit}");
        let rest: String = content.chars().skip(chars).collect();
        assert_eq!(format!("{}{rest}", cut.content), content, "{unit}");
    }
}

fn seat(label: bool) -> (StoredSeat, Option<Label>) {
    let room = RoomId::derive(&[1; 32], &[2; 16]);
    let seat = StoredSeat {
        writer: writer(),
        seat: record::id::SeatId::derive(&room, &[1; 32]),
        room,
        certificate: json!({}),
    };

    (seat, label.then(|| Label::new(1).unwrap()))
}

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

#[test]
fn consecutive_seqs_of_one_reason_share_a_tombstone() {
    let (seat, label) = seat(true);
    let mut marks = Marks::new(&seat, label);
    assert!(marks.is_empty());

    for n in [3, 4, 5] {
        marks.add(seq(n), Held::Purged(TombstoneReason::Purged));
    }
    marks.add(seq(6), Held::Quarantined);
    marks.add(seq(9), Held::Quarantined);

    assert!(!marks.is_empty());
    marks.add(seq(10), Held::Purged(TombstoneReason::Retention));
    let (tombstones, markers) = marks.finish();
    let tombstones: Vec<(String, &str)> = tombstones
        .into_iter()
        .map(|t| (t.range, t.reason.as_str()))
        .collect();
    let markers: Vec<String> = markers.into_iter().map(|m| m.range).collect();
    assert_eq!(
        tombstones,
        [
            ("A1·3–5".to_owned(), "purged"),
            ("A1·10".to_owned(), "retention"),
        ]
    );
    assert_eq!(markers, ["A1·6", "A1·9"]);
}

#[test]
fn a_seq_begins_a_tombstone_unless_it_follows_the_last_of_its_reason() {
    let (seat, label) = seat(true);
    let mut marks = Marks::new(&seat, label);

    assert!(marks.begins(seq(1), Held::Purged(TombstoneReason::Purged)));
    marks.add(seq(1), Held::Purged(TombstoneReason::Purged));
    assert!(!marks.begins(seq(2), Held::Purged(TombstoneReason::Purged)));
    assert!(marks.begins(seq(2), Held::Quarantined));
    assert!(marks.begins(seq(3), Held::Purged(TombstoneReason::Purged)));
}

#[test]
fn a_writer_without_a_label_shows_its_ranges_in_the_full_form() {
    let (seat, label) = seat(false);
    let mut marks = Marks::new(&seat, label);
    marks.add(seq(2), Held::Purged(TombstoneReason::Purged));
    marks.add(seq(3), Held::Purged(TombstoneReason::Purged));

    let (shown, _) = marks.finish();

    assert_eq!(
        shown[0].range,
        format!("parinn:room/{}/w/{}/2-3", seat.room, seat.writer)
    );
    assert!(record::range::Range::parse(&shown[0].range).is_ok());
}

#[test]
fn a_new_tombstone_costs_tokens_and_one_that_extends_another_costs_none() {
    let mut budget = Budget::new();
    let before = budget.left;

    assert!(budget.mark(false, 40));
    assert_eq!(budget.left, before);
    assert!(budget.mark(true, 40));
    assert_eq!(before - budget.left, 40);
    assert!(budget.holds);
}

#[test]
fn a_tombstone_that_does_not_fit_after_others_stops_the_call() {
    let mut budget = Budget::new();
    budget.holds = true;
    budget.left = 39;

    assert!(!budget.mark(true, 40));
    assert_eq!(budget.left, 39);
    let mut first = Budget::new();
    first.left = 39;
    assert!(first.mark(true, 40), "the first of a call always goes in");
    assert_eq!(first.left, 0);
}

#[test]
fn the_budget_charges_a_tombstone_its_longest_range() {
    let (seat, label) = seat(true);
    let short = Marks::new(&seat, label);
    let (seat, none) = self::seat(false);
    let full = Marks::new(&seat, none);

    // A label's range is short; a full-form one costs several times more.
    assert!(short.cost() > 20 && short.cost() < 120, "{}", short.cost());
    assert!(full.cost() > short.cost());
    let widest = format!(
        "parinn:room/{}/w/{}/{WIDEST_SEQ}-{WIDEST_SEQ}",
        seat.room, seat.writer
    );
    assert!(record::range::Range::parse(&widest).is_ok());
    assert_eq!(WIDEST_SEQ.len(), 16);
}
