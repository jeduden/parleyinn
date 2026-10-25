use super::*;
use record::digest::Digest;
use store::Position;
use testkit::TempDir;

fn source(path: &Path, offset: u64, seen: u64) -> Source {
    Source {
        id: 1,
        path: path.to_string_lossy().into_owned(),
        generation: 1,
        directory: String::new(),
        position: Position {
            offset,
            prefix: Digest::ZERO,
            lines: 1,
            seen,
        },
        held_since: None,
        lost: false,
    }
}

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

fn length_of(sizes: &[(&Path, u64)]) -> impl Fn(&Path) -> Option<u64> {
    let sizes: Vec<(PathBuf, u64)> = sizes.iter().map(|(p, n)| (p.to_path_buf(), *n)).collect();
    move |path| sizes.iter().find(|(p, _)| p == path).map(|(_, n)| *n)
}

#[test]
fn a_source_with_every_byte_ingested_is_complete() {
    let path = Path::new("/t/a.jsonl");
    let found = completeness(&[source(path, 5, 5)], &[], &length_of(&[(path, 5)]));

    assert_eq!(found, Completeness::default());
    assert_eq!(found.failure(), None);
    assert_eq!(found.pending_text(), "");
}

#[test]
fn a_source_past_its_position_is_pending_while_a_server_holds_it_and_fails_else() {
    let (held, free) = (Path::new("/t/held.jsonl"), Path::new("/t/free.jsonl"));
    let sources = [source(held, 5, 5), source(free, 5, 5)];
    let served = [held.to_path_buf()];

    let found = completeness(&sources, &served, &length_of(&[(held, 9), (free, 9)]));

    assert_eq!(found.pending, ["/t/held.jsonl"]);
    assert_eq!(
        found.failed,
        [(
            "/t/free.jsonl".to_owned(),
            "holds lines the record lacks and no running MCP server ingests them"
        )]
    );
    assert_eq!(
        found.pending_text(),
        "; transcript sources pending, a running MCP server ingests them: /t/held.jsonl"
    );
    assert_eq!(
        found.failure().as_deref(),
        Some(
            "transcript sources fall short: /t/free.jsonl holds lines the record lacks and no running MCP server ingests them"
        )
    );
}

#[test]
fn a_source_an_ingest_stopped_short_of_is_pending_whatever_holds_it() {
    let dir = TempDir::new().unwrap();
    let (short, begun) = (
        dir.path().join("short.jsonl"),
        dir.path().join("begun.jsonl"),
    );
    std::fs::write(&short, "x").unwrap();
    std::fs::write(&begun, "x").unwrap();
    let sources = [source(&short, 5, 9), source(&begun, 0, 0)];

    let found = completeness(&sources, &[], &length_of(&[(&short, 9), (&begun, 9)]));

    assert_eq!(
        found.pending,
        [short.display().to_string(), begun.display().to_string()]
    );
    assert_eq!(found.failed, []);
}

#[test]
fn a_source_that_disappeared_before_it_was_fully_ingested_fails_and_one_that_shrank_does_not() {
    let (gone, shrunk) = (Path::new("/t/gone.jsonl"), Path::new("/t/shrunk.jsonl"));
    let sources = [source(gone, 5, 9), source(shrunk, 5, 5)];

    let found = completeness(&sources, &[], &length_of(&[(shrunk, 2)]));

    assert_eq!(
        found.failed,
        [(
            "/t/gone.jsonl".to_owned(),
            "disappeared before it was fully ingested"
        )]
    );
}
