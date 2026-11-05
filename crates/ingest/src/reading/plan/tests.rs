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
fn a_path_is_compared_with_its_links_resolved_when_they_can_be() {
    let dir = testkit::TempDir::new().unwrap();
    let real = dir.path().join("real.jsonl");
    std::fs::write(&real, "").unwrap();

    assert_eq!(canonical(&real), real.canonicalize().unwrap());
    let missing = dir.path().join("missing.jsonl");
    assert_eq!(canonical(&missing), missing);
}

#[test]
fn dense_json_is_refused_before_value_allocation_but_a_maximum_string_fits() {
    let dense = format!("{{\"items\":[{}]}}", "0,".repeat(40_000) + "0");
    let plain = format!(
        "{{\"text\":\"{}\"}}",
        "x".repeat(read::bounded::LINE_LIMIT - 12)
    );

    assert!(json_heap_bound(&dense) > PARSED_LINE_LIMIT);
    assert!(json_heap_bound(&plain) <= PARSED_LINE_LIMIT);
}

#[test]
fn a_sibling_line_of_dense_json_gives_no_uuid() {
    let dir = testkit::TempDir::new().unwrap();
    let dense = format!(
        "{{\"uuid\":\"dense\",\"items\":[{}]}}\n{{\"uuid\":\"u1\"}}\n",
        "0,".repeat(40_000) + "0"
    );
    std::fs::write(dir.path().join("other.jsonl"), dense).unwrap();

    let found = sibling_uuids(&dir.path().to_string_lossy(), "", &BTreeSet::new());

    assert_eq!(found.all, BTreeSet::from(["u1".to_owned()]));
}

fn home(dir: &testkit::TempDir) -> Home {
    Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap()
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

/// What the main planning loop makes of the lines of `text`, a
/// transcript at `path` read from its start.
fn planned(home: &Home, path: &Path, text: &str) -> Plan {
    let redactor = redactor();
    let lines = read::lines_at(text, 0);
    let mut entropy = key::random::OsEntropy;
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(home, &mut store, &mut entropy).unwrap();
    let observed = crate::observed::Observed::load(&store).unwrap();
    let inputs = Inputs::default();
    let named = Named::of(path, &redactor);
    let source = store
        .start_generation(&named.path, &named.directory)
        .unwrap();
    let defaults = Defaults::of(path);
    let mut recorder = crate::reading::Recorder::Cli;
    let mut runs = Runs::load(&store, &named.path, &mut recorder).unwrap();
    let mut planner = Planner::new(&store, &named, &inputs, &source, &runs).unwrap();

    planner
        .plan(
            &mut store,
            &mut runs,
            &observed.session(&defaults.own().harness_session),
            Read {
                node: &node,
                home,
                lines: &lines,
                from: 0,
                first_line: 0,
                path,
                defaults: &defaults,
                redactor: &redactor,
                memory_limit: usize::MAX,
            },
            &mut entropy,
        )
        .unwrap()
}

#[test]
fn a_dense_line_reaching_the_planning_loop_is_kept_unparsed_and_never_parsed() {
    let dir = testkit::TempDir::new().unwrap();
    let home = home(&dir);
    let path = dir.path().join("t.jsonl");
    let dense = format!(
        "{{\"uuid\":\"dense\",\"items\":[{}]}}\n",
        "0,".repeat(40_000) + "0"
    );
    let text = format!("{dense}{{\"type\":\"mode\",\"mode\":\"normal\",\"uuid\":\"u1\"}}\n");

    let plan = planned(&home, &path, &text);

    assert_eq!((plan.events.len(), plan.unparsed), (2, 1));
    assert_eq!(plan.events[0].kind, EventKind::UnparsedLine);
    assert_eq!(plan.refused, 0, "its uuid is not read from unparsed text");
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
