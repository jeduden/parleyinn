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

    let served = BTreeSet::from([dir.path().join("other.jsonl")]);

    let found = sibling_uuids(
        &dir.path().to_string_lossy(),
        &own.to_string_lossy(),
        &served,
    );
    let unserved = sibling_uuids(
        &dir.path().to_string_lossy(),
        &own.to_string_lossy(),
        &BTreeSet::new(),
    );

    assert_eq!(found.all, BTreeSet::from(["u1".to_owned()]));
    assert_eq!(found.served, found.all, "a transcript a server serves");
    assert!(unserved.served.is_empty());
    let none = sibling_uuids(
        &dir.path().join("none").to_string_lossy(),
        "",
        &BTreeSet::new(),
    );
    assert!(none.all.is_empty());
}

#[test]
fn a_transcript_or_directory_that_cannot_be_read_is_kept_unread_with_why() {
    let dir = testkit::TempDir::new().unwrap();
    std::fs::write(dir.path().join("good.jsonl"), "{\"uuid\":\"u1\"}\n").unwrap();
    std::fs::create_dir(dir.path().join("sub.jsonl")).unwrap();
    let directory = dir.path().to_string_lossy().into_owned();

    let found = sibling_uuids(&directory, "", &BTreeSet::new());
    let missing = sibling_uuids(&format!("{directory}/none"), "", &BTreeSet::new());

    assert_eq!(found.all, BTreeSet::from(["u1".to_owned()]));
    assert_eq!(found.unread.len(), 1);
    assert_eq!(found.unread[0].0, dir.path().join("sub.jsonl"));
    assert!(
        found.unread[0].1.contains("not a regular file"),
        "{:?}",
        found.unread
    );
    assert_eq!(missing.unread.len(), 1);
    assert_eq!(missing.unread[0].0, dir.path().join("none"));
}

#[test]
fn an_entry_of_the_listing_that_failed_is_kept_unread_beside_the_ones_that_read() {
    let listing = vec![
        Err(std::io::Error::other("entry failed")),
        Ok(PathBuf::from("/nowhere/x.txt")),
    ];

    let found = sibling_uuids_of(Ok(listing.into_iter()), "/nowhere", "", &BTreeSet::new());

    assert_eq!(
        found.unread,
        [(PathBuf::from("/nowhere"), "entry failed".to_owned())]
    );
}

#[test]
fn what_could_not_be_read_is_named_as_the_redactor_shows_it() {
    let secret = "0f0e0d0c-0000-0000-0000-000000000001";
    let named = Named {
        path: String::new(),
        directory: String::new(),
        redacted_path: String::new(),
        name: String::new(),
    };
    let inputs = Inputs::default();
    let source = store::Source {
        id: 1,
        path: String::new(),
        generation: 1,
        directory: String::new(),
        position: store::Position::start(),
        purged: Vec::new(),
        held_since: None,
        lost: false,
        marked_at: 0,
    };
    let dir = testkit::TempDir::new().unwrap();
    let store = Store::open(&dir.path().join("s.db")).unwrap();
    let mut recorder = crate::reading::Recorder::Cli;
    let runs = Runs::load(&store, "", &mut recorder).unwrap();
    let mut planner = Planner::new(&store, &named, &inputs, &source, &runs).unwrap();
    assert!(
        planner.unread(&Redactor::new(&[]).unwrap().0).is_empty(),
        "nothing is read before it is needed"
    );
    planner.siblings = Some(Siblings {
        unread: vec![(
            PathBuf::from(format!("/d/{secret}.jsonl")),
            "why".to_owned(),
        )],
        ..Siblings::default()
    });
    let (redactor, _) = Redactor::new(&[]).unwrap();
    let redactor = redactor
        .with_identifiers(&{
            let mut ids = redaction::account::HarnessAccountIdentifiers::default();
            ids.learn(&format!(
                "{{\"attachment\":{{\"type\":\"credential_org\",\"organizationUuid\":\"{secret}\"}}}}"
            ));
            ids
        })
        .unwrap();

    let unread = planner.unread(&redactor);

    assert_eq!(unread.len(), 1);
    assert_eq!(unread[0].name, "[REDACTED:organization_id]");
    assert_eq!(unread[0].why, "why");
}

#[test]
fn a_path_is_compared_with_its_links_resolved_when_they_can_be() {
    let dir = testkit::TempDir::new().unwrap();
    let real = dir.path().join("real.jsonl");
    std::fs::write(&real, "").unwrap();

    assert_eq!(canonical(&real), real.canonicalize().unwrap());
    let missing = dir.path().join("missing.jsonl");
    assert_eq!(canonical(&missing), missing);
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
