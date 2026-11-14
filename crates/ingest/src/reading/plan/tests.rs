use super::*;

#[test]
fn the_directory_s_other_transcripts_give_their_uuids() {
    let dir = testkit::TempDir::new().unwrap();
    let own = dir.path().join("own.jsonl");
    std::fs::write(&own, "{\"uuid\":\"mine\"}\n").unwrap();
    std::fs::write(
        dir.path().join("other.jsonl"),
        "{\"uuid\":\"u1\"}\nnot json\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("notes.txt"), "{\"uuid\":\"u2\"}\n").unwrap();
    std::fs::create_dir(dir.path().join("sub.jsonl")).unwrap();

    let found = sibling_uuids(&dir.path().to_string_lossy(), &own.to_string_lossy());

    assert_eq!(found, BTreeSet::from(["u1".to_owned()]));
    assert!(sibling_uuids(&dir.path().join("none").to_string_lossy(), "").is_empty());
}
fn home(dir: &testkit::TempDir) -> Home {
    Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap()
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

#[test]
fn a_witnessed_line_repeating_an_event_whose_content_was_erased_is_refused() {
    let dir = testkit::TempDir::new().unwrap();
    let home = home(&dir);
    let parent = dir.path().join("parent.jsonl");
    std::fs::write(&parent, "{\"uuid\":\"u1\"}\n").unwrap();
    crate::ingest_path(&home, &parent, &redactor(), &mut key::random::OsEntropy).unwrap();
    // The content gone while its place stays: no refused copy names it.
    rusqlite::Connection::open(home.store())
        .unwrap()
        .execute(
            "UPDATE events SET content = NULL
             WHERE rowid IN (SELECT event FROM transcript_places)",
            [],
        )
        .unwrap();
    let path = dir
        .path()
        .join("5e55a0de-0000-4000-8000-00000000000a.jsonl");
    std::fs::write(&path, "{\"uuid\":\"u1\"}\n{\"uuid\":\"u2\"}\n").unwrap();

    let ingested = crate::live::ingest_live(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut crate::live::RunSeats::default(),
        None,
        &mut key::random::OsEntropy,
    )
    .unwrap()
    .unwrap();

    assert_eq!((ingested.refused, ingested.events), (1, 1));
    assert_eq!(ingested.witnessed_repeats, 0);
}

#[test]
fn a_uuid_redaction_would_change_is_not_kept_beside_its_line_s_place() {
    let dir = testkit::TempDir::new().unwrap();
    let home = home(&dir);
    let path = dir.path().join("run.jsonl");
    std::fs::write(
        &path,
        "{\"uuid\":\"u1\"}\n{\"uuid\":\"AKIAIOSFODNN7EXAMPLE\"}\n",
    )
    .unwrap();

    crate::ingest_path(&home, &path, &redactor(), &mut key::random::OsEntropy).unwrap();

    let store = Store::open(&home.store()).unwrap();
    let source = store.source(&path.to_string_lossy()).unwrap().unwrap();
    let kept: Vec<Option<String>> = store
        .placed(source.id)
        .unwrap()
        .into_iter()
        .map(|p| p.place.uuid)
        .collect();
    assert_eq!(kept, [Some("u1".to_owned()), None]);
    let bytes = std::fs::read(home.store()).unwrap();
    assert!(!bytes.windows(20).any(|w| w == b"AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn a_uuid_is_kept_only_when_redaction_leaves_it_whole() {
    let redactor = redactor();

    assert_eq!(
        kept_uuid(&json!({"uuid": "u1"}), &redactor),
        Some("u1".to_owned())
    );
    assert_eq!(
        kept_uuid(&json!({"uuid": "AKIAIOSFODNN7EXAMPLE"}), &redactor),
        None
    );
    assert_eq!(kept_uuid(&json!({"uuid": 7}), &redactor), None);
    assert_eq!(kept_uuid(&json!({}), &redactor), None);
}
