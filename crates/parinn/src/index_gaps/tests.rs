use audit::entry::Entry;
use audit::log::Log;
use record::commitment::CommitmentKey;
use record::digest::Digest;
use record::event::{EventKind, Provenance};
use record::id::{RoomId, SeatId, WriterId};
use serde_json::json;
use store::{NewEvent, StoredSeat};
use testkit::TempDir;

use super::*;

const NOW: u64 = 1_760_000_000;

fn writer(n: u8) -> WriterId {
    WriterId::parse(&Digest::from_bytes([n; 32]).to_hex()).unwrap()
}

/// A larger writer id sharing `writer`'s slot in the search index.
fn sharing_the_slot_of(writer: WriterId) -> WriterId {
    let mut bytes = *writer.digest().as_bytes();
    bytes[5..].fill(0xff);

    WriterId::parse(&Digest::from_bytes(bytes).to_hex()).unwrap()
}

fn seated_with_an_event(store: &mut Store, writer: WriterId) {
    let room = RoomId::parse(&Digest::from_bytes([9; 32]).to_hex()).unwrap();
    store
        .add_seat(&StoredSeat {
            writer,
            seat: SeatId::derive(&room, writer.digest().as_bytes()),
            room,
            certificate: json!({}),
        })
        .unwrap();
    store
        .append(
            writer,
            &[NewEvent {
                kind: EventKind::UnparsedLine,
                provenance: Provenance::Unparsed,
                content: json!({"line": 1, "text": "words"}),
                commitment_key: CommitmentKey::from_bytes([1; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap();
}

/// Records a writer whose slot in the search index `small` holds, so
/// the index leaves its events out; returns it.
pub(crate) fn leave_out_a_writer_sharing_the_slot_of(home: &Home, small: WriterId) -> WriterId {
    let large = sharing_the_slot_of(small);
    let mut store = Store::open(&home.store()).unwrap();
    seated_with_an_event(&mut store, large);

    large
}

/// A home whose search index leaves out the second of two writers.
fn home_with_a_gap(dir: &TempDir) -> (Home, WriterId) {
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();
    home.create().unwrap();
    let small = writer(1);
    seated_with_an_event(&mut Store::open(&home.store()).unwrap(), small);

    (
        home.clone(),
        leave_out_a_writer_sharing_the_slot_of(&home, small),
    )
}

fn entries(home: &Home) -> Vec<(String, String, String)> {
    Log::new(home.audit())
        .entries()
        .unwrap()
        .into_iter()
        .map(|e| (e.kind, e.counter.unwrap_or_default(), e.detail))
        .collect()
}

#[test]
fn each_class_of_gap_has_its_own_counter_and_names_its_writer() {
    let id = writer(3);
    for (class, counter, words) in [
        (
            GapClass::WriterUnindexed,
            "search_index_writer_unindexed",
            "search index leaves out the events of writer",
        ),
        (
            GapClass::SeqBeyondLimit,
            "search_index_seq_beyond_limit",
            "search index leaves out the events from seq 2^28 on of writer",
        ),
    ] {
        let text = id.to_string();
        let entry = Entry::next(
            &occurrence(IndexGap { writer: id, class }, &text, "ingest"),
            1,
            NOW,
            Digest::ZERO,
        )
        .unwrap();

        assert_eq!(entry.kind, "dropped");
        assert_eq!(entry.counter.as_deref(), Some(counter));
        assert_eq!(entry.operation, "ingest");
        assert_eq!(entry.detail, format!("{words} {id}"));
    }
}

#[test]
fn a_gap_is_audited_once_and_then_noted_so_the_next_ingest_does_not_audit_it_again() {
    let dir = TempDir::new().unwrap();
    let (home, large) = home_with_a_gap(&dir);
    let gaps = Store::open(&home.store())
        .unwrap()
        .unreported_search_index_gaps()
        .unwrap();
    assert_eq!(gaps.len(), 1);

    let audit = |o: &Occurrence| Log::new(home.audit()).append(o, NOW).is_ok();
    assert_eq!(report(&home, &gaps, "ingest", &mut { audit }).unwrap(), 0);

    let seen = entries(&home);
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].1, "search_index_writer_unindexed");
    assert!(seen[0].2.ends_with(&large.to_string()));
    let store = Store::open(&home.store()).unwrap();
    assert!(store.unreported_search_index_gaps().unwrap().is_empty());
}

#[test]
fn a_gap_whose_entry_the_audit_log_refused_stays_unreported_for_the_next_ingest() {
    let dir = TempDir::new().unwrap();
    let (home, _) = home_with_a_gap(&dir);
    let gaps = Store::open(&home.store())
        .unwrap()
        .unreported_search_index_gaps()
        .unwrap();

    assert_eq!(report(&home, &gaps, "ingest", &mut |_| false).unwrap(), 1);

    let store = Store::open(&home.store()).unwrap();
    assert_eq!(store.unreported_search_index_gaps().unwrap(), gaps);
}

#[test]
fn a_store_that_cannot_be_opened_audits_nothing_so_the_next_ingest_audits_each_gap_once() {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();
    home.create().unwrap();
    std::fs::create_dir(home.store()).unwrap();
    let gap = IndexGap {
        writer: writer(2),
        class: GapClass::WriterUnindexed,
    };
    let mut audit = |_: &Occurrence| -> bool { panic!("nothing is audited before the note") };

    let err = report(&home, &[gap], "ingest", &mut audit).unwrap_err();

    assert!(matches!(err, ReportError::Unnoted(_)), "{err}");
    assert!(err.to_string().ends_with("; the next ingest tries again"));
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn refused_entries_whose_note_cannot_be_taken_back_are_said_with_how_many() {
    let dir = TempDir::new().unwrap();
    let (home, _) = home_with_a_gap(&dir);
    let gaps = Store::open(&home.store())
        .unwrap()
        .unreported_search_index_gaps()
        .unwrap();
    let index = home.root().join(store::search_index::FILE);
    let mut audit = |_: &Occurrence| {
        let conn = rusqlite::Connection::open(&index).unwrap();
        conn.execute_batch("DROP TABLE search_gaps_reported")
            .unwrap();
        false
    };

    let err = report(&home, &gaps, "ingest", &mut audit).unwrap_err();

    assert!(
        matches!(err, ReportError::Unaudited { entries: 1, .. }),
        "{err}"
    );
    assert!(
        err.to_string()
            .starts_with("the audit log refused 1 entries and their gaps stay noted as reported: "),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn a_report_says_a_problem_only_when_an_entry_or_a_note_failed() {
    assert_eq!(problem(&Ok(0)), None);
    assert_eq!(
        problem(&Ok(2)).as_deref(),
        Some("the audit log refused 2 entries; the next ingest tries again")
    );
    assert_eq!(
        problem(&Err(ReportError::Unnoted(StoreError::NoIndex))).as_deref(),
        Some(
            "the store has no search index: open it for writing once; the next ingest tries again"
        )
    );
}

#[test]
fn a_failed_lookup_has_its_own_counter_and_names_its_operation() {
    let entry = Entry::next(&lookup_failed("mcp"), 1, NOW, Digest::ZERO).unwrap();

    assert_eq!(entry.kind, "failed");
    assert_eq!(
        entry.counter.as_deref(),
        Some("search_index_gap_lookup_failed")
    );
    assert_eq!(entry.operation, "mcp");
}

#[test]
fn no_gap_opens_no_store() {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join("missing").as_os_str()), None).unwrap();

    assert_eq!(report(&home, &[], "ingest", &mut |_| false).unwrap(), 0);

    assert!(!home.store().exists());
}
