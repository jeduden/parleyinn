use super::*;

fn colliding() -> (WriterId, WriterId) {
    let (a, b) = (writer(1), sharing_the_slot(0));

    if a < b { (a, b) } else { (b, a) }
}

fn gap(writer: WriterId, class: GapClass) -> IndexGap {
    IndexGap { writer, class }
}

#[test]
fn a_writer_without_a_slot_is_a_gap_and_a_writer_past_the_seq_limit_is_another() {
    let (small, large) = colliding();
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, small, room());
    store.append(small, &texts(&["shared"])).unwrap();
    let heads = [
        (small, 1),
        (large, 3),
        (writer(2), 0),
        (writer(3), SEQ_LIMIT),
    ];
    let mut found = gaps(&store.conn, &heads).unwrap();
    found.sort();
    let mut want = vec![
        gap(large, GapClass::WriterUnindexed),
        gap(writer(3), GapClass::WriterUnindexed),
        gap(writer(3), GapClass::SeqBeyondLimit),
    ];
    want.sort();

    assert_eq!(found, want);
}

#[test]
fn a_writer_whose_newest_seq_is_just_under_the_limit_is_no_gap() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(1), room());
    store.append(writer(1), &texts(&["a"])).unwrap();

    assert!(gaps(&store.conn, &[(writer(1), 1)]).unwrap().is_empty());
    assert!(
        gaps(&store.conn, &[(writer(1), SEQ_LIMIT - 1)])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_store_names_the_writer_its_index_left_out_whichever_arrived_first() {
    let (small, large) = colliding();
    for order in [[small, large], [large, small]] {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        for w in order {
            seat(&mut store, w, room());
            store.append(w, &texts(&["shared word"])).unwrap();
        }

        assert_eq!(
            store.search_index_gaps().unwrap(),
            [gap(large, GapClass::WriterUnindexed)]
        );
    }
}

#[test]
fn a_store_leaving_out_nothing_has_no_gap() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);

    assert!(store.search_index_gaps().unwrap().is_empty());
    assert!(store.unreported_search_index_gaps().unwrap().is_empty());
}

#[test]
fn a_gap_is_unreported_until_marked_and_stays_reported_through_a_rebuild() {
    let (small, large) = colliding();
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    for w in [small, large] {
        seat(&mut store, w, room());
        store.append(w, &texts(&["shared word"])).unwrap();
    }
    let found = store.unreported_search_index_gaps().unwrap();
    assert_eq!(found, [gap(large, GapClass::WriterUnindexed)]);

    store.mark_search_index_gaps_reported(&found).unwrap();
    store.mark_search_index_gaps_reported(&found).unwrap();
    assert!(store.unreported_search_index_gaps().unwrap().is_empty());
    store.rebuild_search_index().unwrap();
    store.append(large, &texts(&["more"])).unwrap();

    assert!(store.unreported_search_index_gaps().unwrap().is_empty());
    assert_eq!(store.search_index_gaps().unwrap().len(), 1);
}

#[test]
fn a_store_whose_index_lacks_the_note_of_reported_gaps_cannot_say_which_are() {
    let dir = TempDir::new().unwrap();
    let (small, large) = colliding();
    let mut store = open(&dir);
    for w in [small, large] {
        seat(&mut store, w, room());
        store.append(w, &texts(&["shared word"])).unwrap();
    }
    store
        .conn
        .execute_batch("DROP TABLE idx.search_gaps_reported")
        .unwrap();

    assert!(store.unreported_search_index_gaps().is_err());
    assert!(
        store
            .mark_search_index_gaps_reported(&[gap(large, GapClass::WriterUnindexed)])
            .is_err()
    );
    assert!(
        store
            .unmark_search_index_gaps_reported(&[gap(large, GapClass::WriterUnindexed)])
            .is_err()
    );
}

#[test]
fn a_gap_whose_report_did_not_reach_the_audit_log_can_be_unmarked_to_be_reported_again() {
    let (small, large) = colliding();
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    for w in [small, large] {
        seat(&mut store, w, room());
        store.append(w, &texts(&["shared word"])).unwrap();
    }
    let found = store.unreported_search_index_gaps().unwrap();
    store.mark_search_index_gaps_reported(&found).unwrap();

    store.unmark_search_index_gaps_reported(&found).unwrap();
    store.unmark_search_index_gaps_reported(&found).unwrap();

    assert_eq!(store.unreported_search_index_gaps().unwrap(), found);
}

#[test]
fn each_class_of_gap_has_its_counter_name() {
    assert_eq!(GapClass::WriterUnindexed.name(), "writer_unindexed");
    assert_eq!(GapClass::SeqBeyondLimit.name(), "seq_beyond_limit");
}
