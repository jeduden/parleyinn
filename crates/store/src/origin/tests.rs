use super::*;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn
}

#[test]
fn a_store_with_no_origins_table_holds_none() {
    let conn = Connection::open_in_memory().unwrap();

    assert_eq!(all(&conn).unwrap(), BTreeMap::new());
}

#[test]
fn origins_are_read_back_by_writer() {
    let conn = conn();
    let kept: BTreeMap<_, _> = [(writer(1), "witnessed"), (writer(2), "peer")]
        .into_iter()
        .map(|(w, o)| (w, o.to_owned()))
        .collect();

    replace(&conn, &kept).unwrap();

    assert_eq!(all(&conn).unwrap(), kept);
}

#[test]
fn replacing_drops_the_origins_not_offered() {
    let conn = conn();
    let first: BTreeMap<_, _> = [(writer(1), "witnessed".to_owned())].into();
    let second: BTreeMap<_, _> = [(writer(2), "ingested".to_owned())].into();

    replace(&conn, &first).unwrap();
    replace(&conn, &second).unwrap();

    assert_eq!(all(&conn).unwrap(), second);
}

#[test]
fn a_kept_writer_that_does_not_parse_is_corrupt() {
    let conn = conn();
    conn.execute(
        "INSERT INTO origins (writer, origin) VALUES ('nope', 'peer')",
        [],
    )
    .unwrap();

    assert!(matches!(all(&conn), Err(StoreError::Corrupt { .. })));
}

#[test]
fn a_store_that_takes_no_write_refuses_to_keep_origins() {
    let conn = conn();
    conn.execute_batch("PRAGMA query_only = ON").unwrap();
    let kept: BTreeMap<_, _> = [(writer(1), "peer".to_owned())].into();

    assert!(replace(&conn, &kept).is_err());
}

#[test]
fn a_store_that_refuses_a_row_after_clearing_the_table_reports_it() {
    let conn = conn();
    conn.execute_batch(
        "CREATE TRIGGER refuse BEFORE INSERT ON origins BEGIN SELECT RAISE(ABORT, 'no'); END",
    )
    .unwrap();
    let kept: BTreeMap<_, _> = [(writer(1), "peer".to_owned())].into();

    assert!(replace(&conn, &kept).is_err());
}
