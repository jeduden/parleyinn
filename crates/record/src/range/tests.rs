use super::*;

const KEY: [u8; 32] = [7; 32];

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

#[test]
fn a_label_is_a_and_a_number_from_one() {
    assert_eq!(Label::new(0), None);
    let label = Label::new(12).unwrap();
    assert_eq!(label.to_string(), "A12");
    assert_eq!(label.rank(), 12);
    assert_eq!(Label::parse("A12"), Some(label));
    for refused in [
        "",
        "A",
        "A0",
        "A01",
        "a1",
        "B1",
        "A1x",
        "A+1",
        "A1234567890",
        "1",
    ] {
        assert_eq!(Label::parse(refused), None, "{refused:?}");
    }
    assert_eq!(
        Label::parse("A999999999").map(Label::rank),
        Some(999_999_999)
    );
}

#[test]
fn every_form_writes_the_same_range() {
    let label = Label::new(2).unwrap();
    let want = |from, to| Range {
        writer: Writer::Label(label),
        from: seq(from),
        to: seq(to),
        room: None,
    };

    assert_eq!(Range::parse("A2·4812").unwrap(), want(4812, 4812));
    assert_eq!(Range::parse("A2:4812").unwrap(), want(4812, 4812));
    assert_eq!(Range::parse("A2·4812–5025").unwrap(), want(4812, 5025));
    assert_eq!(Range::parse("A2:4812-5025").unwrap(), want(4812, 5025));
    assert_eq!(Range::parse("A2:4812–5025").unwrap(), want(4812, 5025));
    assert!(Range::parse("A2:7").unwrap().is_single());
    assert!(!Range::parse("A2:7-8").unwrap().is_single());
}

#[test]
fn a_writer_id_stands_in_for_the_label_and_the_full_form_names_its_room() {
    let writer = WriterId::derive(&KEY);
    let room = RoomId::derive(&KEY, &[1; 16]);

    let ascii = Range::parse(&format!("{writer}:3-9")).unwrap();
    assert_eq!(ascii.writer, Writer::Id(writer));
    assert_eq!((ascii.from, ascii.to, ascii.room), (seq(3), seq(9), None));
    let full = Range::parse(&format!("parinn:room/{room}/w/{writer}/7")).unwrap();
    assert_eq!(full.writer, Writer::Id(writer));
    assert_eq!(
        (full.from, full.to, full.room),
        (seq(7), seq(7), Some(room))
    );
    let spans = Range::parse(&format!("parinn:room/{room}/w/{writer}/7-9")).unwrap();
    assert_eq!(spans.to, seq(9));
}

#[test]
fn text_that_is_no_range_is_refused_naming_it() {
    let writer = WriterId::derive(&KEY);
    let room = RoomId::derive(&KEY, &[1; 16]);
    for refused in [
        "",
        "A2",
        "A2:",
        "A2:0",
        "A2:x",
        "A2:9-3",
        "A2:3-",
        "A2:-3",
        "A2:3-4-5",
        "A2:+3",
        "zz:3",
        "parinn:room/",
        "parinn:room/x/w/y/1",
        &format!("parinn:room/{room}/x/{writer}/1"),
        &format!("parinn:room/{room}/w/{writer}"),
        &format!("parinn:room/{room}/w/zz/1"),
    ] {
        let error = Range::parse(refused).unwrap_err();
        assert!(
            error.to_string().starts_with("not an address or range: "),
            "{refused:?}"
        );
    }
    let long = "x".repeat(200);
    let shown = Range::parse(&long).unwrap_err().to_string();
    assert!(shown.contains('…') && shown.len() < 200, "{shown}");
}

#[test]
fn the_short_and_range_forms_read_back() {
    let label = Label::new(3).unwrap();

    assert_eq!(short(label, seq(7)), "A3·7");
    assert_eq!(range(label, seq(3), seq(9)), "A3·3–9");
    assert_eq!(Range::parse(&short(label, seq(7))).unwrap().from, seq(7));
    assert_eq!(
        Range::parse(&range(label, seq(3), seq(9))).unwrap().to,
        seq(9)
    );
}
