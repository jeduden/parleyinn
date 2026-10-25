use std::path::PathBuf;

use key::certificate::SeatCertificate;
use key::random::OsEntropy;
use record::event::EventKind;
use serde_json::{Value, json};
use testkit::TempDir;

use super::*;

const SESSION: &str = "5e55a0de-0000-4000-8000-00000000000a";

fn home(dir: &TempDir) -> Home {
    Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap()
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

/// The main run's transcript of [`SESSION`], holding `text`.
fn main_transcript(dir: &TempDir, text: &str) -> PathBuf {
    let path = dir.path().join(format!("{SESSION}.jsonl"));
    std::fs::write(&path, text).unwrap();
    path
}

fn line(n: u64) -> String {
    format!(
        "{}\n",
        json!({"type": "mode", "mode": "normal", "n": n, "uuid": format!("u{n}")})
    )
}

fn ingest(
    home: &Home,
    path: &Path,
    seats: &mut RunSeats,
    limit: Option<usize>,
) -> Option<Ingested> {
    ingest_live(
        home,
        path,
        &redactor(),
        &Inputs::default(),
        seats,
        limit,
        &mut OsEntropy,
    )
    .unwrap()
}

/// The content of each event of `writer`.
fn contents(home: &Home, writer: WriterId) -> Vec<Value> {
    let store = Store::open(&home.store()).unwrap();
    store
        .log(writer)
        .unwrap()
        .into_iter()
        .map(|e| e.content.unwrap())
        .collect()
}

#[test]
fn complete_lines_end_at_the_last_line_break_or_the_limit() {
    let bytes = b"{\"a\":1}\n{\"b\":2}\n{\"c\":";

    assert_eq!(complete_lines(bytes, None), 16);
    assert_eq!(complete_lines(bytes, Some(1)), 8);
    assert_eq!(complete_lines(bytes, Some(5)), 16);
    assert_eq!(complete_lines(bytes, Some(0)), 0);
    assert_eq!(complete_lines(b"{\"c\":", None), 0);
}

#[test]
fn each_complete_line_goes_to_a_run_seat_the_device_key_certified_no_ingest_seat() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = main_transcript(&dir, &format!("{}{}{{\"type\":", line(1), line(2)));
    let mut seats = RunSeats::default();

    let ingested = ingest(&home, &path, &mut seats, None).unwrap();

    assert_eq!(ingested.events, 2, "the line still written waits");
    let run = RunKey {
        agent_id: None,
        harness_session: SESSION.to_owned(),
    };
    assert_eq!(seats.writer(&run), Some(ingested.writer));
    assert_eq!(seats.runs(), [(run.clone(), ingested.writer)]);
    let store = Store::open(&home.store()).unwrap();
    let cert =
        SeatCertificate::from_value(&store.seat(ingested.writer).unwrap().unwrap().certificate)
            .unwrap();
    assert!(!cert.ingest_seat, "a run seat, so its events are witnessed");
    assert!(store.ingested_runs().unwrap().is_empty());
    assert!(
        !format!("{seats:?}").contains("key"),
        "only the runs show: {seats:?}"
    );

    std::fs::write(&path, format!("{}{}{}", line(1), line(2), line(3))).unwrap();
    let more = ingest(&home, &path, &mut seats, None).unwrap();
    assert_eq!((more.writer, more.events), (ingested.writer, 1));
    assert!(
        ingest(&home, &path, &mut seats, None).is_none(),
        "nothing new"
    );
}

#[test]
fn a_restarted_server_goes_on_in_a_new_run_seat_from_the_ingest_marker() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = main_transcript(&dir, &(1..=5).map(line).collect::<String>());
    let mut first = RunSeats::default();

    let stopped = ingest(&home, &path, &mut first, Some(3)).unwrap();
    let mut next = RunSeats::default();
    let resumed = ingest(&home, &path, &mut next, None).unwrap();

    assert_eq!((stopped.events, resumed.events), (3, 2));
    assert_ne!(stopped.writer, resumed.writer);
    let numbers: Vec<Value> = contents(&home, resumed.writer)
        .iter()
        .map(|c| c["line"].clone())
        .collect();
    assert_eq!(numbers, [json!(4), json!(5)]);
}

#[test]
fn a_held_hook_observation_lands_in_the_main_run_seat_where_its_hook_fired() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let two = format!("{}{}", line(1), line(2));
    let path = main_transcript(&dir, &format!("{two}{}", line(3)));
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let observation = store::Observation {
        hook: "SessionStart".to_owned(),
        harness_session: SESSION.to_owned(),
        transcript_path: path.to_string_lossy().into_owned(),
        cwd: "/w".to_owned(),
        offset: Some(two.len() as u64),
        source: Some("startup".to_owned()),
        prompt_id: None,
        attended: false,
        ancestry: Some((1, false)),
        harness: Some(7),
        at: 1,
    };
    store.hold_observation(&observation, None).unwrap();
    let mut late = observation.clone();
    late.offset = Some(10_000);
    store.hold_observation(&late, None).unwrap();
    drop(store);
    let mut seats = RunSeats::default();

    let ingested = ingest(&home, &path, &mut seats, None).unwrap();

    assert_eq!(
        ingested.observations, 1,
        "the later one waits for its offset"
    );
    let kinds: Vec<(Value, Value)> = contents(&home, ingested.writer)
        .iter()
        .map(|c| (c["line"].clone(), c["hook"].clone()))
        .collect();
    assert_eq!(
        kinds,
        [
            (json!(1), Value::Null),
            (json!(2), Value::Null),
            (Value::Null, json!("SessionStart")),
            (json!(3), Value::Null),
        ]
    );
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    assert_eq!(log[2].link.header.kind, EventKind::HookObservation);
    let appended: Vec<bool> = store
        .hook_observations()
        .unwrap()
        .iter()
        .map(|h| h.appended)
        .collect();
    assert_eq!(appended, [true, false]);
    assert!(
        ingest(&home, &path, &mut seats, None).is_none(),
        "an observation whose hook fired past the lines waits"
    );
}

#[test]
fn a_witnessed_line_repeating_an_ingested_event_is_its_own_event() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = dir.path().join("parent.jsonl");
    std::fs::write(&parent, line(1)).unwrap();
    crate::ingest_path(&home, &parent, &redactor(), &mut OsEntropy).unwrap();
    let path = main_transcript(&dir, &format!("{}{}", line(1), line(2)));
    let mut seats = RunSeats::default();

    let ingested = ingest(&home, &path, &mut seats, None).unwrap();

    assert_eq!((ingested.events, ingested.references), (2, 0));
    assert_eq!(ingested.witnessed_repeats, 1);
    assert!(!ingested.held);
}

#[test]
fn an_ingest_that_got_ahead_refuses_the_append_behind_it() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = main_transcript(&dir, &(1..=4).map(line).collect::<String>());
    let mut seats = RunSeats::default();
    ingest(&home, &path, &mut seats, Some(2)).unwrap();
    crate::ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert!(
        ingest(&home, &path, &mut seats, Some(3)).is_none(),
        "another ingest read past what this one may"
    );
    assert!(ingest(&home, &path, &mut seats, None).is_none());
}

#[test]
fn a_link_swapped_in_for_a_transcript_is_refused() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let secret = dir.path().join("id_ed25519");
    std::fs::write(&secret, "-----BEGIN OPENSSH PRIVATE KEY-----\n").unwrap();
    let path = dir.path().join(format!("{SESSION}.jsonl"));
    #[cfg(unix)]
    std::os::unix::fs::symlink(&secret, &path).unwrap();
    let mut seats = RunSeats::default();

    let err = ingest_live(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut seats,
        None,
        &mut OsEntropy,
    )
    .unwrap_err();

    assert!(matches!(err, IngestError::Read { .. }), "{err}");
}

#[test]
fn a_transcript_that_is_not_utf8_records_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = dir.path().join(format!("{SESSION}.jsonl"));
    std::fs::write(&path, [0xff, b'\n']).unwrap();

    let err = ingest_live(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut RunSeats::default(),
        None,
        &mut OsEntropy,
    )
    .unwrap_err();

    assert!(matches!(err, IngestError::NotUtf8(_)), "{err}");
}

#[test]
fn a_transcript_rewritten_shorter_starts_a_new_generation() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = main_transcript(&dir, &(1..=3).map(line).collect::<String>());
    let mut seats = RunSeats::default();
    ingest(&home, &path, &mut seats, None).unwrap();

    std::fs::write(&path, line(9)).unwrap();
    let rewritten = ingest(&home, &path, &mut seats, None).unwrap();

    assert!(rewritten.new_generation);
    assert_eq!((rewritten.generation, rewritten.events), (2, 1));
}
