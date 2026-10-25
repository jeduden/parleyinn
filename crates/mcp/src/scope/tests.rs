use super::*;
use record::event::{EventKind, Provenance};
use serde_json::json;
use store::{IngestedRun, NewEvent, SeatId, StoredSeat};
use testkit::TempDir;

fn room(n: u8) -> RoomId {
    RoomId::derive(&[n; 32], &[2; 16])
}

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

const NO_RUN: &str = "no run";

/// A store whose writer 1 is a seat in room 1 and writer 2 a seat in
/// room 2, both of the run `s1`, and writer 3 a seat in room 3 of
/// another run.
fn store(dir: &TempDir) -> Store {
    let mut store = Store::open(&dir.path().join("store.db")).unwrap();
    for (n, run) in [(1, "s1"), (2, "s1"), (3, "s2")] {
        store
            .add_seat(&StoredSeat {
                writer: writer(n),
                seat: SeatId::derive(&room(n), writer(n).digest().as_bytes()),
                room: room(n),
                certificate: json!({}),
            })
            .unwrap();
        store
            .add_run(&IngestedRun {
                writer: writer(n),
                path: format!("/t/{run}.jsonl"),
                harness_session: run.to_owned(),
                agent_id: None,
            })
            .unwrap();
    }
    store
        .append(
            writer(1),
            &[NewEvent {
                kind: EventKind::TranscriptLine,
                provenance: Provenance::User,
                content: json!({"line": 1, "text": "x"}),
                commitment_key: store::CommitmentKey::from_bytes([1; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap();
    store
}

/// `ids` in the order a scope keeps them.
fn sorted<T: Ord>(mut ids: Vec<T>) -> Vec<T> {
    ids.sort();
    ids
}

fn resolved(store: &Store, run: Option<&str>, arguments: &Value) -> Result<Scope, Refused> {
    let call = Call {
        store,
        personal_room: room(1),
        run,
    };

    resolve(&call, arguments, None, NO_RUN)
}

#[test]
fn a_scope_is_run_unless_the_call_names_room_or_rooms() {
    assert_eq!(Name::of(&json!({})), Ok(Name::Run));
    assert_eq!(Name::of(&json!({"scope": "run"})), Ok(Name::Run));
    assert_eq!(Name::of(&json!({"scope": "room"})), Ok(Name::Room));
    assert_eq!(Name::of(&json!({"scope": "rooms"})), Ok(Name::Rooms));
    assert_eq!(Name::of(&json!({"scope": "world"})), Err(BAD_SCOPE));
    assert_eq!(Name::of(&json!({"scope": 1})), Err(BAD_SCOPE));
    assert_eq!(
        [Name::Run, Name::Room, Name::Rooms].map(Name::as_str),
        ["run", "room", "rooms"]
    );
}

#[test]
fn a_server_serving_no_run_serves_the_room_scope_alone() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);

    let default = resolved(&store, None, &json!({"scope": "room"})).unwrap();
    let named = resolved(
        &store,
        None,
        &json!({"scope": "room", "room": room(3).to_string()}),
    )
    .unwrap();

    assert_eq!(default.rooms, BTreeSet::from([room(1)]));
    assert_eq!(named.rooms, BTreeSet::from([room(3)]));
    assert!(default.current.is_empty());
    assert_eq!(
        resolved(&store, None, &json!({})).unwrap_err(),
        Refused::Text(NO_RUN)
    );
    assert_eq!(
        resolved(&store, None, &json!({"scope": "rooms"})).unwrap_err(),
        Refused::Text(ROOMS_NEED_A_RUN)
    );
}

#[test]
fn a_run_serves_its_writers_its_rooms_and_nothing_else() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);

    let run = resolved(&store, Some("s1"), &json!({})).unwrap();
    let rooms = resolved(&store, Some("s1"), &json!({"scope": "rooms"})).unwrap();
    let room_two = resolved(
        &store,
        Some("s1"),
        &json!({"scope": "room", "room": room(2).to_string()}),
    )
    .unwrap();
    let lost = resolved(
        &store,
        Some("s1"),
        &json!({"scope": "room", "room": room(3).to_string()}),
    );

    assert_eq!(run.current, BTreeSet::from([writer(1), writer(2)]));
    assert_eq!(rooms.rooms, BTreeSet::from([room(1), room(2)]));
    assert_eq!(room_two.rooms, BTreeSet::from([room(2)]));
    assert_eq!(lost.unwrap_err(), Refused::Text(NO_SEAT_IN_ROOM));
    assert!(!run.extends() && rooms.extends() && room_two.extends());
}

#[test]
fn a_run_ingest_never_recorded_still_has_its_personal_room() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);

    let room_scope = resolved(&store, Some("new"), &json!({"scope": "room"})).unwrap();
    let rooms = resolved(&store, Some("new"), &json!({"scope": "rooms"})).unwrap();

    assert_eq!(room_scope.rooms, BTreeSet::from([room(1)]));
    assert_eq!(rooms.rooms, BTreeSet::from([room(1)]));
    assert!(room_scope.current.is_empty());
}

#[test]
fn the_room_argument_is_a_room_id_and_agrees_with_the_room_an_address_names() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    let call = Call {
        store: &store,
        personal_room: room(1),
        run: None,
    };
    let scope = json!({"scope": "room"});

    for bad in [json!({"room": "x"}), json!({"room": 3})] {
        let mut arguments = scope.clone();
        arguments["room"] = bad["room"].clone();
        assert_eq!(
            resolve(&call, &arguments, None, NO_RUN).unwrap_err(),
            Refused::Text(BAD_ROOM)
        );
    }
    let mut arguments = scope.clone();
    arguments["room"] = json!(room(2).to_string());
    assert_eq!(
        resolve(&call, &arguments, Some(room(3)), NO_RUN).unwrap_err(),
        Refused::Text(ROOM_CONFLICT)
    );
    let named = resolve(&call, &scope, Some(room(3)), NO_RUN).unwrap();
    assert_eq!(named.rooms, BTreeSet::from([room(3)]));
    let agreeing = resolve(&call, &arguments, Some(room(2)), NO_RUN).unwrap();
    assert_eq!(agreeing.rooms, BTreeSet::from([room(2)]));
}

#[test]
fn an_unreadable_record_is_a_refusal_of_its_own() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    rusqlite::Connection::open(dir.path().join("store.db"))
        .unwrap()
        .execute("ALTER TABLE ingested_runs RENAME TO moved", [])
        .unwrap();

    let refused = resolved(&store, Some("s1"), &json!({})).unwrap_err();

    assert_eq!(refused, Refused::Unreadable);
    assert_eq!(Refused::from("fixed"), Refused::Text("fixed"));
}

#[test]
fn a_scope_covers_the_writers_of_the_run_or_the_rooms_it_reaches() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    let run = resolved(&store, Some("s1"), &json!({})).unwrap();
    let rooms = resolved(&store, Some("s1"), &json!({"scope": "rooms"})).unwrap();

    assert!(run.covers(writer(1), room(1)));
    assert!(run.covers(writer(2), room(2)));
    assert!(!run.covers(writer(3), room(3)));
    assert!(
        rooms.covers(writer(3), room(1)),
        "a room covers any writer in it"
    );
    assert!(!rooms.covers(writer(1), room(3)));
    assert!(run.is_current(writer(1)) && !run.is_current(writer(3)));
}

#[test]
fn what_a_scope_reaches_is_its_writers_or_its_rooms() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    let run = resolved(&store, Some("s1"), &json!({})).unwrap();
    let rooms = resolved(&store, Some("s1"), &json!({"scope": "rooms"})).unwrap();

    assert_eq!(
        run.reach(),
        Reach::Writers(sorted(vec![writer(1), writer(2)]))
    );
    assert_eq!(rooms.reach(), Reach::Rooms(sorted(vec![room(1), room(2)])));
}

#[test]
fn a_search_is_limited_to_the_run_and_never_to_everything() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    let run = resolved(&store, Some("s1"), &json!({})).unwrap();
    let nobody = resolved(&store, Some("new"), &json!({})).unwrap();
    let rooms = resolved(&store, Some("s1"), &json!({"scope": "rooms"})).unwrap();
    let room_scope = resolved(&store, None, &json!({"scope": "room"})).unwrap();

    assert_eq!(
        run.limits(None),
        Some((vec![], sorted(vec![writer(1), writer(2)])))
    );
    assert_eq!(run.limits(Some(writer(2))), Some((vec![], vec![writer(2)])));
    assert_eq!(run.limits(Some(writer(3))), None);
    assert_eq!(nobody.limits(None), None);
    assert_eq!(nobody.limits(Some(writer(1))), None);
    assert_eq!(
        rooms.limits(None),
        Some((sorted(vec![room(1), room(2)]), vec![]))
    );
    assert_eq!(
        rooms.limits(Some(writer(1))),
        Some((sorted(vec![room(1), room(2)]), vec![writer(1)]))
    );
    assert_eq!(room_scope.limits(None), Some((vec![room(1)], vec![])));
}

#[test]
fn a_room_scope_with_no_room_reaches_nothing() {
    let none = Scope {
        name: Name::Rooms,
        current: BTreeSet::new(),
        rooms: BTreeSet::new(),
    };

    assert_eq!(none.limits(None), None);
}
