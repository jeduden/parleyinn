use super::*;

fn writer() -> WriterId {
    WriterId::derive(&[1; 32])
}

fn fork(first: u64, last: u64, offered: u8) -> Fork {
    Fork {
        writer: writer(),
        first: Seq::new(first).unwrap(),
        last: Seq::new(last).unwrap(),
        held: Digest::of(&[b"held"]),
        offered: Digest::of(&[&[offered]]),
    }
}

fn conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn
}

#[test]
fn a_store_with_no_refusals_table_holds_none() {
    let conn = Connection::open_in_memory().unwrap();

    assert_eq!(all(&conn).unwrap(), vec![]);
}

#[test]
fn a_refusal_is_recorded_once_and_read_back_by_writer_then_seq() {
    let conn = conn();
    let (a, b) = (fork(3, 4, 1), fork(1, 2, 2));

    assert_eq!(record(&conn, &[a, b]).unwrap(), vec![a, b]);
    assert_eq!(record(&conn, &[a]).unwrap(), vec![]);
    assert_eq!(all(&conn).unwrap(), vec![b, a]);
}

#[test]
fn a_refusal_row_that_does_not_parse_names_what() {
    let conn = conn();
    let put = |writer: &str, first: i64, held: &str| {
        conn.execute("DELETE FROM fork_refusals", []).unwrap();
        conn.execute(
            "INSERT INTO fork_refusals VALUES (?1, ?2, 5, ?3, ?3)",
            params![writer, first, held],
        )
        .unwrap();
    };
    let digest = Digest::of(&[b"x"]).to_hex();
    let corrupt = |e: StoreError| match e {
        StoreError::Corrupt {
            address: None,
            what,
        } => what,
        other => panic!("not corrupt: {other:?}"),
    };

    put("x", 1, &digest);
    assert_eq!(corrupt(all(&conn).unwrap_err()), "fork writer \"x\"");
    put(&writer().to_string(), 0, &digest);
    assert_eq!(corrupt(all(&conn).unwrap_err()), "fork first seq 0");
    put(&writer().to_string(), 1, "nope");
    assert_eq!(corrupt(all(&conn).unwrap_err()), "fork digest \"nope\"");
    put(&writer().to_string(), 1, &digest);
    conn.execute("UPDATE fork_refusals SET last_seq = -1", [])
        .unwrap();
    assert_eq!(corrupt(all(&conn).unwrap_err()), "fork last seq -1");
}
