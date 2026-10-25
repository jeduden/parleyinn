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
