use super::*;

fn writer() -> WriterId {
    WriterId::derive(&[1; 32])
}

fn row(first: i64, last: i64, version: i64) -> Row {
    (writer().to_string(), first, last, version, false)
}

#[test]
fn a_segment_row_parses_into_its_range_and_version() {
    assert_eq!(
        parse(row(1, 3, 1)),
        Ok(Segment {
            writer: writer(),
            first: Seq::FIRST,
            last: Seq::new(3).unwrap(),
            format_version: 1,
            closed: false,
        })
    );
}

#[test]
fn a_segment_row_out_of_range_names_what_does_not_parse() {
    assert_eq!(
        parse(row(0, 3, 1)),
        Err(BadRow("segment first seq 0".into()))
    );
    assert_eq!(
        parse(row(1, -2, 1)),
        Err(BadRow("segment last seq -2".into()))
    );
    assert_eq!(
        parse(row(1, 3, -1)),
        Err(BadRow("segment format version -1".into()))
    );
    let mut bad = row(1, 3, 1);
    bad.0 = "x".into();
    assert_eq!(parse(bad), Err(BadRow("segment writer \"x\"".into())));
}

#[test]
fn a_segment_names_its_writer_and_range() {
    let segment = parse(row(4, 5, 2)).unwrap();

    assert_eq!(
        segment.to_string(),
        format!("the segment of writer {} from seq 4 to 5", writer())
    );
}

#[test]
fn a_segment_grows_while_open_and_contiguous_and_opens_anew_otherwise() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    let seq = |n| Seq::new(n).unwrap();
    let all = || -> Vec<Segment> {
        select(&conn, "WHERE ?1 = ?1 AND ?2 = ?2", "")
            .unwrap()
            .into_iter()
            .map(Result::unwrap)
            .collect()
    };

    extend(&conn, writer(), seq(1), seq(2), false).unwrap();
    extend(&conn, writer(), seq(3), seq(4), true).unwrap();
    extend(&conn, writer(), seq(5), seq(5), false).unwrap();
    extend(&conn, writer(), seq(9), seq(9), false).unwrap();
    close(&conn, writer()).unwrap();

    let ranges: Vec<(u64, u64, bool)> = all()
        .iter()
        .map(|s| (s.first.get(), s.last.get(), s.closed))
        .collect();
    assert_eq!(ranges, [(1, 4, true), (5, 5, true), (9, 9, true)]);
    assert_eq!(newer(&conn, writer()).unwrap(), None);
}
