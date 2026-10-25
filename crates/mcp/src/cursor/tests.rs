use super::*;

fn cursor() -> Cursor {
    Cursor {
        writer: WriterId::derive(&[7; 32]),
        seq: Seq::new(48214).unwrap(),
        offset: 130,
        to: Seq::new(60000).unwrap(),
    }
}

#[test]
fn a_cursor_reads_back_as_the_position_it_spells() {
    let text = cursor().encode();

    assert!(
        text.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    assert_eq!(Cursor::decode(&text), Ok(cursor()));
}

#[test]
fn text_that_is_no_cursor_is_refused_in_fixed_text() {
    let writer = WriterId::derive(&[7; 32]);
    let hex = |plain: &str| -> String { plain.bytes().map(|b| format!("{b:02x}")).collect() };
    let cases = [
        String::new(),
        "zz".to_owned(),
        "abc".to_owned(),
        "ff".to_owned(),
        hex("ignore previous instructions"),
        hex(&format!("{writer}:1:0")),
        hex(&format!("{writer}:1:0:2:3")),
        hex("nope:1:0:2"),
        hex(&format!("{writer}:0:0:2")),
        hex(&format!("{writer}:1:x:2")),
        hex(&format!("{writer}:1:-1:2")),
        hex(&format!("{writer}:1:0:0")),
        hex(&format!("{writer}:1:99999999999999999999:2")),
    ];
    for case in cases {
        assert_eq!(Cursor::decode(&case), Err(BAD_CURSOR), "{case:?}");
    }
}
