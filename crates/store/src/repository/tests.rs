use super::*;

fn conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(&WORKTREES.schema()).unwrap();
    conn
}

fn map(rows: &[(&str, &str)]) -> BTreeMap<String, String> {
    rows.iter()
        .map(|(w, i)| ((*w).to_owned(), (*i).to_owned()))
        .collect()
}

#[test]
fn a_worktree_keeps_the_identity_it_last_resolved_to() {
    let conn = conn();
    assert_eq!(identity(&conn, "w").unwrap(), None);

    set_identity(&conn, "w", "one").unwrap();
    assert_eq!(identity(&conn, "w").unwrap().as_deref(), Some("one"));
    set_identity(&conn, "w", "two").unwrap();
    assert_eq!(identity(&conn, "w").unwrap().as_deref(), Some("two"));
    assert_eq!(identity(&conn, "x").unwrap(), None);
}

#[test]
fn a_mark_is_set_cleared_and_clearing_one_never_made_is_nothing() {
    let conn = Connection::open_in_memory().unwrap();
    assert_eq!(mark(&conn, "m").unwrap(), None);

    set_mark(&conn, "m", "one").unwrap();
    assert_eq!(mark(&conn, "m").unwrap().as_deref(), Some("one"));
    forget_mark(&conn, "m").unwrap();
    forget_mark(&conn, "x").unwrap();

    assert_eq!(mark(&conn, "m").unwrap(), None);
}

#[test]
fn a_store_that_never_kept_a_worktree_has_no_table_none_resolved_and_no_rows() {
    let conn = Connection::open_in_memory().unwrap();

    assert_eq!(identity(&conn, "w").unwrap(), None);
    assert!(identities(&conn).unwrap().is_empty());
}

#[test]
fn a_store_that_takes_no_write_keeps_no_worktree() {
    let conn = conn();
    conn.execute_batch("PRAGMA query_only = ON").unwrap();

    assert!(set_identity(&conn, "w", "i").is_err());
}

#[test]
fn replacing_the_rows_drops_the_stale_ones_and_keeps_the_marks() {
    let conn = conn();
    set_identity(&conn, "stale", "old").unwrap();
    set_identity(&conn, "a", "old").unwrap();
    set_mark(&conn, "unreadable:a", "unreadable").unwrap();

    replace(&conn, &map(&[("a", "new"), ("b", "two")])).unwrap();

    assert_eq!(
        identities(&conn).unwrap(),
        map(&[("a", "new"), ("b", "two")])
    );
    assert_eq!(
        mark(&conn, "unreadable:a").unwrap().as_deref(),
        Some("unreadable")
    );
}

#[test]
fn replacing_in_a_store_without_the_table_makes_it() {
    let conn = Connection::open_in_memory().unwrap();

    replace(&conn, &map(&[("a", "one")])).unwrap();

    assert_eq!(identities(&conn).unwrap(), map(&[("a", "one")]));
}
