use super::*;
use record::digest::Digest;
use store::Position;
use testkit::TempDir;

/// When the tests' `parinn verify` runs, in milliseconds.
const NOW: u64 = 100 * MARKER_CEILING_MS;

/// A source of `path` whose ingest consumed `offset` bytes of
/// [`BODY`] and last saw `seen`, its marker left [`NOW`].
fn source(path: &Path, offset: u64, seen: u64) -> Source {
    Source {
        id: 1,
        path: path.to_string_lossy().into_owned(),
        generation: 1,
        directory: String::new(),
        position: Position {
            offset,
            prefix: Digest::of(&[&BODY[..offset as usize]]),
            lines: 1,
            seen,
        },
        held_since: None,
        lost: false,
        marked_at: NOW,
    }
}

/// The bytes the tests' sources hold: a line every five bytes.
const BODY: &[u8] = b"aaaa\nbbbb\ncccc\ndddd\n";

#[test]
fn a_source_is_lost_when_gone_with_bytes_unread_or_lines_held() {
    let dir = TempDir::new().unwrap();
    let here = dir.path().join("here.jsonl");
    std::fs::write(&here, "x").unwrap();
    let gone = dir.path().join("gone.jsonl");

    let whole_gone = source(&gone, 5, 5);
    let cut_short = source(&gone, 5, 9);
    let held = Source {
        held_since: Some(1),
        ..source(&gone, 5, 5)
    };
    let found = Source {
        lost: true,
        ..source(&here, 5, 5)
    };
    let unread_here = source(&here, 1, 9);
    let all = [
        whole_gone,
        cut_short.clone(),
        held.clone(),
        found.clone(),
        unread_here,
    ];

    assert_eq!(lost(&all), [&cut_short, &held, &found]);
}

#[test]
fn the_status_names_held_and_lost_sources() {
    let status = SourcesStatus {
        held: vec!["/t/f.jsonl".into()],
        lost: vec!["/t/c.jsonl".into()],
    };

    assert_eq!(
        status.json(),
        json!({"held": ["/t/f.jsonl"], "lost": ["/t/c.jsonl"]})
    );
    assert_eq!(
        status.lines(),
        [
            "repeated lines held at an ingest marker: /t/f.jsonl",
            "transcript sources gone before they were fully ingested: /t/c.jsonl",
        ]
    );
    let none = SourcesStatus {
        held: Vec::new(),
        lost: Vec::new(),
    };
    assert!(none.lines().is_empty());
}

#[test]
fn a_home_with_no_store_has_no_sources_and_a_store_reads_them() {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();
    assert_eq!(read(&home).unwrap(), None);

    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let s = store
        .start_generation(&dir.path().join("gone.jsonl").to_string_lossy(), "")
        .unwrap();
    store.hold(s.id, Some(3)).unwrap();
    drop(store);

    let status = read(&home).unwrap().unwrap();
    assert_eq!(status.held.len(), 1);
    assert_eq!(status.lost.len(), 1);
}

#[cfg(unix)]
#[test]
fn a_store_holding_no_schema_yet_has_no_sources() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();
    home.create().unwrap();
    std::fs::write(home.store(), "").unwrap();
    std::fs::set_permissions(home.store(), std::fs::Permissions::from_mode(0o600)).unwrap();

    assert_eq!(read(&home).unwrap(), None);
}

/// What `parinn verify` finds of each of `files` now, by its bytes; any
/// other path is gone.
fn files(files: &[(&Path, &[u8])]) -> impl Fn(&Path, u64) -> Found + use<> {
    let files: Vec<(PathBuf, Vec<u8>)> = files
        .iter()
        .map(|(p, b)| (p.to_path_buf(), b.to_vec()))
        .collect();
    move |path, offset| match files.iter().find(|(p, _)| p == path) {
        Some((_, bytes)) => scan(bytes.as_slice(), offset).map_or(Found::Unreadable, Found::File),
        None => Found::Gone,
    }
}

#[test]
fn a_source_with_every_line_ingested_is_complete() {
    let path = Path::new("/t/a.jsonl");
    let found = completeness(&[source(path, 20, 20)], &[], &files(&[(path, BODY)]), NOW);

    assert_eq!(found, Completeness::default());
    assert_eq!(found.failure(), None);
    assert_eq!(found.short_text(), "");
}

/// A transcript of `bytes` written as `name` in `dir`.
fn on_disk(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn bytes_past_the_last_line_break_are_a_line_still_being_written() {
    let dir = TempDir::new().unwrap();
    let path = on_disk(&dir, "a.jsonl", &[BODY, b"{\"partial\":"].concat());

    let found = completeness(
        &[source(&path, 20, 31)],
        &[],
        &look,
        NOW + 2 * MARKER_CEILING_MS,
    );

    assert_eq!(found, Completeness::default());
}

#[test]
fn a_source_past_its_position_is_short_while_a_server_holds_it_and_fails_else() {
    let (held, free) = (Path::new("/t/held.jsonl"), Path::new("/t/free.jsonl"));
    let sources = [source(held, 5, 5), source(free, 5, 5)];
    let served = [held.to_path_buf()];

    let found = completeness(
        &sources,
        &served,
        &files(&[(held, BODY), (free, BODY)]),
        NOW,
    );

    assert_eq!(found.short, ["/t/held.jsonl"]);
    assert_eq!(
        found.failed,
        [(
            "/t/free.jsonl".to_owned(),
            "holds lines the record lacks and no running MCP server ingests them"
        )]
    );
    assert_eq!(
        found.short_text(),
        "; transcript sources short of their end, which ingest resumes: /t/held.jsonl"
    );
    assert_eq!(
        found.failure().as_deref(),
        Some(
            "transcript sources fall short: /t/free.jsonl holds lines the record lacks and no running MCP server ingests them"
        )
    );
}

/// A source an ingest stopped short of and one it began and consumed
/// nothing of, each with whole lines past its position.
fn stopped_and_begun(dir: &TempDir) -> ([Source; 2], [String; 2]) {
    let stopped = on_disk(dir, "stopped.jsonl", BODY);
    let begun = on_disk(dir, "begun.jsonl", BODY);
    let names = [stopped.display().to_string(), begun.display().to_string()];

    ([source(&stopped, 5, 20), source(&begun, 0, 0)], names)
}

#[test]
fn a_source_an_ingest_stopped_short_of_is_short_while_its_marker_is_recent() {
    let dir = TempDir::new().unwrap();
    let (sources, names) = stopped_and_begun(&dir);

    let found = completeness(&sources, &[], &look, NOW + MARKER_CEILING_MS);

    assert_eq!(found.short, names);
    assert_eq!(found.failed, []);
    let early = completeness(&sources, &[], &look, NOW - MARKER_CEILING_MS);
    assert_eq!(early.short, names, "a clock that stepped back");
}

#[test]
fn a_source_an_ingest_stopped_short_of_long_ago_fails() {
    let dir = TempDir::new().unwrap();
    let (sources, [stopped, begun]) = stopped_and_begun(&dir);

    for now in [NOW + MARKER_CEILING_MS + 1, NOW - MARKER_CEILING_MS - 1] {
        let found = completeness(&sources, &[], &look, now);

        assert_eq!(found.short, Vec::<String>::new());
        assert_eq!(
            found.failed,
            [(stopped.clone(), STALE), (begun.clone(), STALE)]
        );
    }
    assert_eq!(
        STALE,
        "was left short of its end by an ingest over 10 minutes ago and nothing resumed it: \
         parinn ingest resumes it"
    );
}

#[test]
fn a_source_that_disappeared_before_it_was_fully_ingested_fails_and_one_gone_whole_does_not() {
    let (gone, whole) = (Path::new("/t/gone.jsonl"), Path::new("/t/whole.jsonl"));
    let sources = [source(gone, 5, 9), source(whole, 5, 5)];

    let found = completeness(&sources, &[], &files(&[]), NOW);

    assert_eq!(
        found.failed,
        [(
            "/t/gone.jsonl".to_owned(),
            "disappeared before it was fully ingested"
        )]
    );
}

#[test]
fn a_source_cut_below_its_position_or_no_longer_beginning_with_what_was_ingested_fails() {
    let (cut, rewritten, unread) = (
        Path::new("/t/cut.jsonl"),
        Path::new("/t/rewritten.jsonl"),
        Path::new("/t/unread.jsonl"),
    );
    let sources = [
        source(cut, 10, 10),
        source(rewritten, 10, 10),
        source(unread, 10, 10),
    ];
    let mut other = BODY.to_vec();
    other[0] = b'z';
    let look = |path: &Path, offset| match path.to_str() {
        Some("/t/unread.jsonl") => Found::Unreadable,
        Some("/t/cut.jsonl") => files(&[(cut, &BODY[..7])])(path, offset),
        _ => files(&[(rewritten, &other)])(path, offset),
    };

    let found = completeness(&sources, &[], &look, NOW);

    assert_eq!(
        found.failed,
        [
            ("/t/cut.jsonl".to_owned(), REPLACED),
            ("/t/rewritten.jsonl".to_owned(), REPLACED),
            ("/t/unread.jsonl".to_owned(), "cannot be read to check it"),
        ]
    );
    assert_eq!(
        REPLACED,
        "no longer begins with the bytes ingested from it: cut, replaced or rewritten since"
    );
}

#[test]
fn a_scan_hashes_only_the_consumed_prefix_and_finds_a_line_past_it() {
    let at = scan(BODY, 10).unwrap();
    assert_eq!(at.prefix, Some(Digest::of(&[&BODY[..10]])));
    assert!(at.unread_line);

    let end = scan(BODY, 20).unwrap();
    assert_eq!(end.prefix, Some(Digest::of(&[BODY])));
    assert!(!end.unread_line);

    let short = scan(&BODY[..4], 10).unwrap();
    assert_eq!(short.prefix, None);
    assert!(!short.unread_line);

    let long = vec![b'x'; 3 * CHUNK + 7];
    let past = scan(
        [long.as_slice(), b"\n"].concat().as_slice(),
        2 * CHUNK as u64 + 1,
    )
    .unwrap();
    assert_eq!(past.prefix, Some(Digest::of(&[&long[..2 * CHUNK + 1]])));
    assert!(past.unread_line);
}

#[test]
fn a_failing_read_fails_the_scan() {
    struct Failing;
    impl std::io::Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("no"))
        }
    }

    assert!(scan(Failing, 1).is_err());
}

#[test]
fn a_file_is_looked_at_by_its_bytes_and_a_missing_one_is_gone() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("a.jsonl");
    std::fs::write(&path, BODY).unwrap();

    match look(&path, 5) {
        Found::File(at) => assert_eq!(at, scan(BODY, 5).unwrap()),
        other => panic!("{other:?}"),
    }
    assert!(matches!(look(&dir.path().join("none"), 5), Found::Gone));
    assert!(matches!(look(dir.path(), 5), Found::Unreadable));
    assert!(matches!(look(&path.join("below"), 5), Found::Unreadable));
}

#[test]
fn a_path_with_control_characters_is_shown_escaped() {
    let found = Completeness {
        short: vec!["/t/a\nb.jsonl".to_owned()],
        failed: vec![("/t/c\u{1b}[2J.jsonl".to_owned(), "x")],
    };

    assert_eq!(
        found.short_text(),
        "; transcript sources short of their end, which ingest resumes: /t/a<U+000A>b.jsonl"
    );
    assert_eq!(
        found.failure().as_deref(),
        Some("transcript sources fall short: /t/c<U+001B>[2J.jsonl x")
    );
    assert_eq!(
        lost_text(&["/t/\u{7}.jsonl".to_owned()]),
        "transcript sources gone before they were fully ingested: /t/<U+0007>.jsonl"
    );
}

/// Makes a FIFO at `path` with `mkfifo`, which std cannot, waiting for
/// it up to a deadline (ENG-03).
#[cfg(unix)]
#[expect(
    clippy::disallowed_types,
    reason = "a test makes a FIFO with mkfifo; the core starts no program"
)]
pub(crate) fn mkfifo(path: &Path) {
    let mut child = std::process::Command::new("mkfifo")
        .arg(path)
        .spawn()
        .unwrap();
    let step = std::time::Duration::from_millis(10);
    let mut waited = std::time::Duration::ZERO;
    while child.try_wait().unwrap().is_none() {
        assert!(
            waited < std::time::Duration::from_secs(10),
            "mkfifo did not exit"
        );
        std::thread::sleep(step);
        waited += step;
    }
    assert!(path.exists());
}

/// What `parinn verify` finds of the source at `path`, or none when
/// it does not answer within five seconds (ENG-03).
fn verified_within_deadline(path: &Path) -> Option<Completeness> {
    let path = path.to_path_buf();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let found = completeness(&[source(&path, 5, 5)], &[], &look, NOW);
        let _ = tx.send(found);
    });

    rx.recv_timeout(std::time::Duration::from_secs(5)).ok()
}

#[cfg(unix)]
#[test]
fn a_fifo_at_a_recorded_path_fails_as_unreadable_without_waiting() {
    let dir = TempDir::new().unwrap();
    let fifo = dir.path().join("a.jsonl");
    mkfifo(&fifo);

    let found = verified_within_deadline(&fifo).expect("verify waited on a FIFO");

    assert!(found.short.is_empty());
    assert_eq!(
        found.failed,
        [(
            fifo.to_string_lossy().into_owned(),
            "cannot be read to check it"
        )]
    );
}

#[cfg(unix)]
#[test]
fn a_link_swapped_in_at_a_recorded_path_is_not_followed() {
    let dir = TempDir::new().unwrap();
    let outside = dir.path().join("outside.jsonl");
    std::fs::write(&outside, BODY).unwrap();
    let path = dir.path().join("a.jsonl");
    std::os::unix::fs::symlink(&outside, &path).unwrap();

    assert_eq!(look(&path, 5), Found::Unreadable);
}
