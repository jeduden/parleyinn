use std::path::PathBuf;

use audit::log::Log;
use mcp::tool_call::Runs;
use serde_json::json;
use store::Observation;
use testkit::TempDir;

use super::*;

const SESSION: &str = "5e55a0de-0000-4000-8000-0000000000d1";
const HARNESS: u32 = 77;

struct Fixture {
    dir: TempDir,
    home: Home,
    shared: Arc<Shared>,
}

impl Fixture {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();
        home.create().unwrap();
        std::fs::create_dir_all(dir.path().join("projects").join("p")).unwrap();
        Self {
            dir,
            home,
            shared: Arc::new(Shared::default()),
        }
    }

    fn projects(&self) -> PathBuf {
        self.dir.path().join("projects").join("p")
    }

    fn main(&self) -> PathBuf {
        self.projects().join(format!("{SESSION}.jsonl"))
    }

    fn subagent(&self, id: &str) -> PathBuf {
        let dir = self.projects().join(SESSION).join("subagents");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(format!("agent-{id}.jsonl"))
    }

    fn running(&self, harness: Option<u32>, limit: Option<usize>) -> Running {
        let roots = AllowedRoots::default().with(&self.dir.path().join("projects"), Access::Read);
        Running {
            capture: Capture {
                ceiling: None,
                home: self.home.clone(),
                roots,
                redactor: Redactor::new(&[]).unwrap().0,
                inputs: ingest::Inputs::default(),
                harness,
                limit,
                clock: Box::new(|| 1_760_000_000),
                shared: self.shared.clone(),
            },
            seats: RunSeats::default(),
            sessions: Vec::new(),
            transcripts: BTreeMap::new(),
            refused: BTreeSet::new(),
            marker: None,
            closed: now_instant(),
            close_every: SEGMENT_EVERY,
        }
    }

    /// Holds a `SessionStart` hook observation of `session` naming
    /// `path`, of the harness process `harness`.
    fn bind(&self, session: &str, path: &Path, harness: u32) {
        self.bind_in(session, path, harness, Path::new("/w"));
    }

    /// [`bind`](Self::bind), its harness session's worktree `cwd`.
    fn bind_in(&self, session: &str, path: &Path, harness: u32, cwd: &Path) {
        let mut store = Store::open(&self.home.store()).unwrap();
        let observation = Observation {
            hook: "SessionStart".to_owned(),
            harness_session: session.to_owned(),
            transcript_path: path.to_string_lossy().into_owned(),
            cwd: cwd.to_string_lossy().into_owned(),
            offset: Some(0),
            source: Some("startup".to_owned()),
            prompt_id: None,
            attended: false,
            ancestry: Some((1, false)),
            harness: Some(harness),
            at: 1,
        };
        store.hold_observation(&observation, None).unwrap();
    }

    /// The counter of each audit entry.
    fn counted(&self) -> Vec<String> {
        Log::new(self.home.audit())
            .entries()
            .unwrap()
            .into_iter()
            .filter_map(|e| e.counter)
            .collect()
    }

    fn markers(&self) -> Vec<PathBuf> {
        std::fs::read_dir(self.home.root().join(MARKERS))
            .map(|l| l.flatten().map(|e| e.path()).collect())
            .unwrap_or_default()
    }
}

/// A transcript line of the main run, or of the subagent `agent`,
/// carrying the tool call `tool_use_id` when given.
fn line(n: u64, tool_use_id: Option<&str>) -> String {
    let value = match tool_use_id {
        Some(id) => json!({
            "type": "assistant",
            "n": n,
            "message": {"content": [{"type": "tool_use", "id": id, "name": "event_search"}]},
        }),
        // In Claude Code 2.1.296's recorded shape, so nothing drifts.
        None => json!({"type": "atis-latch", "atis": n.to_string(), "sessionId": "s"}),
    };
    format!("{value}\n")
}

#[test]
fn a_run_is_named_by_its_harness_session_and_a_subagent_by_its_agent_id_too() {
    let main = RunKey {
        agent_id: None,
        harness_session: "s".to_owned(),
    };
    let agent = RunKey {
        agent_id: Some("a1".to_owned()),
        ..main.clone()
    };

    assert_eq!(run_name(&main), "s");
    assert_eq!(run_name(&agent), "s/a1");
}

#[test]
fn only_an_assistant_line_s_tool_use_blocks_carry_tool_use_ids() {
    let call = json!({"type": "assistant", "message": {"content": [
        {"type": "text", "text": "x"},
        {"type": "tool_use", "id": "toolu_1"},
        {"type": "tool_use"},
    ]}});
    let result = json!({"type": "user", "message": {"content": [
        {"type": "tool_result", "tool_use_id": "toolu_1"},
    ]}});

    assert_eq!(tool_use_ids(&call), ["toolu_1"]);
    assert!(tool_use_ids(&result).is_empty());
    assert!(tool_use_ids(&json!({"type": "assistant"})).is_empty());
}

#[test]
fn a_finder_returns_the_runs_read_or_none_once_its_wait_passes() {
    let shared = Arc::new(Shared::default());
    shared.read("toolu_1".to_owned(), "s".to_owned());
    shared.read("toolu_1".to_owned(), "s".to_owned());
    let finder = Finder(shared.clone());

    assert!(!finder.serving());
    assert_eq!(finder.find("toolu_1", Duration::from_secs(5)).0, ["s"]);
    let (none, waited) = finder.find("toolu_2", Duration::from_millis(30));
    assert!(none.is_empty());
    assert!(waited >= Duration::from_millis(30));
    shared.stop();
    assert!(shared.stopping());
}

#[test]
fn live_ingest_takes_only_its_own_harness_process_s_binding_and_reads_its_tool_uses() {
    let f = Fixture::new();
    std::fs::write(
        f.main(),
        format!("{}{}", line(1, None), line(2, Some("toolu_m"))),
    )
    .unwrap();
    std::fs::write(f.subagent("a1"), line(1, Some("toolu_a"))).unwrap();
    f.bind("other", &f.projects().join("other.jsonl"), HARNESS + 1);
    let mut running = f.running(Some(HARNESS), None);

    running.bind();
    running.poll();
    assert!(!f.shared.serving.load(Ordering::SeqCst), "no binding yet");
    assert!(running.seats.runs().is_empty());

    f.bind(SESSION, &f.main(), HARNESS);
    running.bind();
    running.poll();

    assert!(f.shared.serving.load(Ordering::SeqCst));
    assert_eq!(
        running.seats.runs().len(),
        2,
        "the main run's and the subagent's"
    );
    assert_eq!(f.shared.runs_of("toolu_m"), [SESSION]);
    assert_eq!(f.shared.runs_of("toolu_a"), [format!("{SESSION}/a1")]);
    assert_eq!(f.markers().len(), 1, "its start marker");
    running.poll();
    running.stop();
    assert!(f.markers().is_empty(), "cleared as it stopped");
    assert!(!f.counted().contains(&"ingest_remainder_left".to_owned()));
}

#[test]
fn a_stop_short_of_a_transcript_s_end_is_audited_by_the_server() {
    let f = Fixture::new();
    std::fs::write(f.main(), (1..=4).map(|n| line(n, None)).collect::<String>()).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), Some(2));

    running.bind();
    running.poll();
    running.stop();

    assert_eq!(f.counted(), ["ingest_remainder_left"]);
    let detail = Log::new(f.home.audit()).entries().unwrap()[0]
        .detail
        .clone();
    assert!(detail.ends_with(SESSION), "{detail}");
}

#[test]
fn a_killed_server_s_remainder_is_audited_once_by_the_next_reader() {
    let f = Fixture::new();
    std::fs::write(f.main(), (1..=4).map(|n| line(n, None)).collect::<String>()).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), Some(2));
    running.bind();
    running.poll();
    let audit = |o: &Occurrence| {
        Log::new(f.home.audit()).append(o, 1).unwrap();
    };

    assert_eq!(count_stopped(&f.home, "ingest", &audit), 0, "it still runs");
    // Killed: its marker's lock is released, its file left.
    drop(running);
    assert_eq!(count_stopped(&f.home, "ingest", &audit), 1);
    assert_eq!(count_stopped(&f.home, "ingest", &audit), 0);
    assert_eq!(f.counted(), ["ingest_remainder_left"]);
    assert!(f.markers().is_empty());
}

#[test]
fn no_marker_directory_and_files_that_are_not_markers_are_passed_over() {
    let f = Fixture::new();
    let audit = |_: &Occurrence| panic!("nothing to audit");
    assert_eq!(count_stopped(&f.home, "ingest", &audit), 0);

    std::fs::create_dir_all(f.home.root().join(MARKERS)).unwrap();
    std::fs::write(f.home.root().join(MARKERS).join("x.staged"), "").unwrap();
    std::fs::create_dir(f.home.root().join(MARKERS).join("d.marker")).unwrap();
    assert_eq!(count_stopped(&f.home, "ingest", &audit), 0);
}

#[test]
fn a_transcript_outside_the_roots_or_a_link_swapped_in_is_refused_once() {
    let f = Fixture::new();
    let outside = f.dir.path().join("elsewhere.jsonl");
    std::fs::write(&outside, line(1, None)).unwrap();
    f.bind(SESSION, &outside, HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();
    running.poll();
    assert_eq!(f.counted(), ["path_outside_allowed_roots"]);

    let f = Fixture::new();
    std::fs::write(f.main(), line(1, None)).unwrap();
    let subagent = f.subagent("a2");
    std::fs::write(&subagent, line(1, None)).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();
    running.poll();
    let secret = f.dir.path().join("id_ed25519");
    std::fs::write(&secret, "key\n").unwrap();
    std::fs::remove_file(&subagent).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&secret, &subagent).unwrap();
    running.poll();
    running.poll();

    assert_eq!(f.counted(), ["path_outside_allowed_roots"]);
    assert_eq!(running.transcripts.len(), 1, "only the main run's is read");
}

#[test]
fn a_transcript_not_written_yet_waits_and_one_that_fails_is_audited() {
    let f = Fixture::new();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();
    running.poll();
    assert!(f.counted().is_empty(), "not written yet");

    std::fs::write(f.main(), [0xff, b'\n']).unwrap();
    running.poll();
    assert_eq!(f.counted(), ["live_ingest_failed"]);
}

#[test]
fn without_a_harness_process_nothing_is_taken() {
    let f = Fixture::new();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(None, None);

    running.bind();

    assert!(running.sessions.is_empty());
    assert!(!f.shared.serving.load(Ordering::SeqCst));
}

#[test]
fn what_live_ingest_found_is_audited_entry_by_entry() {
    let f = Fixture::new();
    std::fs::write(
        f.main(),
        format!(
            "not json\n{}\n",
            json!({"type": "atis-latch", "atis": "AKIAIOSFODNN7EXAMPLE", "sessionId": "s"})
        ),
    )
    .unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();
    running.poll();
    let first = f.counted();
    assert!(first.contains(&"events_unparsed".to_owned()), "{first:?}");
    assert!(first.contains(&"redacted".to_owned()), "{first:?}");

    // The transcript is rewritten: a new generation.
    std::fs::write(f.main(), line(9, None)).unwrap();
    running.poll();
    assert!(
        f.counted()
            .contains(&"new_transcript_generation".to_owned())
    );
}

#[test]
fn a_witnessed_line_repeating_another_transcript_is_audited() {
    let f = Fixture::new();
    let uuid_line = format!(
        "{}\n",
        json!({"type": "mode", "mode": "normal", "uuid": "u-shared"})
    );
    let other = f.projects().join("other.jsonl");
    std::fs::write(&other, &uuid_line).unwrap();
    ingest::ingest_path(
        &f.home,
        &other,
        &Redactor::new(&[]).unwrap().0,
        &mut OsEntropy,
    )
    .unwrap();
    std::fs::write(f.main(), &uuid_line).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);

    running.bind();
    running.poll();

    let counted: Vec<String> = f
        .counted()
        .into_iter()
        .filter(|c| c != "format_drift")
        .collect();
    assert_eq!(counted, ["witnessed_repeat_recorded"]);
}

#[test]
fn a_start_marker_rewritten_keeps_its_path_and_one_it_cannot_write_keeps_the_old() {
    let f = Fixture::new();
    let first = StartMarker::leave(&f.home, &[f.main()], None).unwrap();
    let path = first.path.clone();
    let second = StartMarker::leave(&f.home, &[f.main(), f.subagent("a")], Some(first)).unwrap();
    assert_eq!(second.path, path);
    assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 2);

    std::fs::remove_dir_all(f.home.root().join(MARKERS)).unwrap();
    let kept = StartMarker::leave(&f.home, &[f.main()], Some(second)).unwrap();
    assert_eq!(kept.path, path);
    assert!(StartMarker::leave(&f.home, &[], None).is_some());
}

#[test]
fn live_ingest_starts_bound_and_stops_when_told() {
    let f = Fixture::new();
    std::fs::write(f.main(), line(1, None)).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let running = f.running(Some(HARNESS), None);
    let capture = Capture {
        ceiling: None,
        home: running.capture.home.clone(),
        roots: running.capture.roots.clone(),
        redactor: running.capture.redactor.clone(),
        inputs: ingest::Inputs::default(),
        harness: Some(HARNESS),
        limit: None,
        clock: Box::new(|| 1),
        shared: f.shared.clone(),
    };

    let started = capture.start();
    assert_eq!(started.seats.runs().len(), 1, "bound before the first call");
    f.shared.stop();
    started.run();

    assert!(f.markers().is_empty());
}

#[test]
fn a_run_seat_s_open_segment_closes_every_30_s_while_live_ingest_appends() {
    let f = Fixture::new();
    std::fs::write(f.main(), line(1, None)).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();
    running.poll();
    let (_, writer) = running.seats.runs()[0].clone();
    let open = || {
        Store::open(&f.home.store())
            .unwrap()
            .open_segments()
            .unwrap()
            .iter()
            .any(|s| s.writer == writer)
    };
    // Its first segment closed after its last line; the next stays open.
    let mut store = Store::open(&f.home.store()).unwrap();
    store
        .append_open(
            writer,
            &[store::NewEvent {
                kind: record::event::EventKind::HookObservation,
                provenance: record::event::Provenance::Structural,
                content: json!({"hook": "SessionStart"}),
                commitment_key: record::commitment::CommitmentKey::from_bytes([4; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap();
    running.poll();
    assert!(open(), "within 30 s it stays open");

    running.close_every = Duration::ZERO;
    running.poll();

    assert!(!open());
}

#[test]
fn a_transcript_in_the_run_s_worktree_is_read() {
    let f = Fixture::new();
    let worktree = f.dir.path().join("repo");
    std::fs::create_dir_all(worktree.join(".git")).unwrap();
    let path = worktree.join(format!("{SESSION}.jsonl"));
    std::fs::write(&path, line(1, None)).unwrap();
    f.bind_in(SESSION, &path, HARNESS, &worktree);
    let mut running = f.running(Some(HARNESS), None);

    running.bind();
    running.poll();

    assert_eq!(running.sessions.len(), 1);
    assert!(f.counted().is_empty());
}

#[test]
fn a_store_that_cannot_open_takes_no_binding_and_stops_without_an_audit() {
    let f = Fixture::new();
    std::fs::write(f.main(), line(1, None)).unwrap();
    let store = f.home.store();
    std::fs::write(&store, "not a database, and longer than its header").unwrap();
    #[cfg(unix)]
    std::fs::set_permissions(&store, std::os::unix::fs::PermissionsExt::from_mode(0o600)).unwrap();
    let mut running = f.running(Some(HARNESS), None);

    running.bind();
    assert!(running.sessions.is_empty());
    assert!(!f.shared.serving.load(Ordering::SeqCst));

    running.transcripts.insert(f.main(), 0);
    running.stop();
    assert!(f.counted().is_empty(), "no remainder it could not read");
}

#[test]
fn a_transcript_gone_after_its_ingest_gives_no_tool_uses_and_is_read_again_later() {
    let f = Fixture::new();
    std::fs::write(f.main(), line(1, Some("toolu_m"))).unwrap();
    let redactor = Redactor::new(&[]).unwrap().0;
    ingest::ingest_path(&f.home, &f.main(), &redactor, &mut OsEntropy).unwrap();
    std::fs::remove_file(f.main()).unwrap();
    let mut running = f.running(Some(HARNESS), None);
    running.transcripts.insert(f.main(), 0);

    running.read_tool_uses(&f.main());

    assert!(f.shared.runs_of("toolu_m").is_empty());
    assert_eq!(running.transcripts.get(&f.main()), Some(&0));
}

#[cfg(unix)]
#[test]
fn a_marker_that_cannot_be_opened_is_passed_over() {
    let f = Fixture::new();
    let markers = f.home.root().join(MARKERS);
    std::fs::create_dir_all(&markers).unwrap();
    let link = markers.join("gone.marker");
    std::os::unix::fs::symlink(f.dir.path().join("gone"), &link).unwrap();
    let audit = |_: &Occurrence| panic!("nothing to audit");

    assert_eq!(count_stopped(&f.home, "ingest", &audit), 0);
    assert!(link.symlink_metadata().is_ok(), "left as it is");
}

#[test]
fn a_lookup_of_the_search_index_gaps_that_failed_is_counted_by_the_live_ingest() {
    let f = Fixture::new();
    let other = f.projects().join("other.jsonl");
    std::fs::write(&other, line(1, None)).unwrap();
    let mut ingested = ingest::ingest_path(
        &f.home,
        &other,
        &Redactor::new(&[]).unwrap().0,
        &mut OsEntropy,
    )
    .unwrap();
    ingested.index_gaps_unread = Some("the store has no search index".to_owned());

    f.running(None, None).audit_ingested(&ingested);

    assert_eq!(f.counted(), ["search_index_gap_lookup_failed"]);
}

#[test]
fn a_gap_the_audit_log_refused_in_the_live_ingest_is_logged_and_reported_at_the_next() {
    let f = Fixture::new();
    let other = f.projects().join("other.jsonl");
    std::fs::write(&other, line(1, None)).unwrap();
    let first = ingest::ingest_path(
        &f.home,
        &other,
        &Redactor::new(&[]).unwrap().0,
        &mut OsEntropy,
    )
    .unwrap();
    crate::index_gaps::tests::leave_out_a_writer_sharing_the_slot_of(&f.home, first.writer);
    let ingested = ingest::ingest_path(
        &f.home,
        &other,
        &Redactor::new(&[]).unwrap().0,
        &mut OsEntropy,
    )
    .unwrap();
    assert_eq!(ingested.index_gaps.len(), 1);
    let audit_dir = f.home.root().join("audit");
    std::fs::write(&audit_dir, "").unwrap();

    f.running(None, None).audit_ingested(&ingested);

    std::fs::remove_file(&audit_dir).unwrap();
    let store = Store::open(&f.home.store()).unwrap();
    assert_eq!(store.unreported_search_index_gaps().unwrap().len(), 1);
}

#[test]
fn a_gap_of_the_search_index_is_audited_once_by_the_live_ingest() {
    let f = Fixture::new();
    let other = f.projects().join("other.jsonl");
    std::fs::write(&other, line(1, None)).unwrap();
    let first = ingest::ingest_path(
        &f.home,
        &other,
        &Redactor::new(&[]).unwrap().0,
        &mut OsEntropy,
    )
    .unwrap();
    crate::index_gaps::tests::leave_out_a_writer_sharing_the_slot_of(&f.home, first.writer);
    std::fs::write(f.main(), line(2, None)).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();

    running.poll();
    std::fs::write(f.main(), format!("{}{}", line(2, None), line(3, None))).unwrap();
    running.poll();

    let gaps = f
        .counted()
        .into_iter()
        .filter(|c| c == "search_index_writer_unindexed")
        .count();
    assert_eq!(gaps, 1);
}

#[test]
fn a_session_binding_taken_after_the_server_started_binds_the_repository_it_showed() {
    let f = Fixture::new();
    let mut store = Store::open(&f.home.store()).unwrap();
    administration::node::Node::open(&f.home, &mut store, &mut key::random::OsEntropy).unwrap();
    drop(store);
    let app = f.dir.path().join("app");
    std::fs::create_dir_all(app.join(".git/refs/heads")).unwrap();
    std::fs::write(app.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    f.bind_in(SESSION, &f.main(), HARNESS, &app);
    let mut running = f.running(Some(HARNESS), None);

    running.bind();

    let conn = rusqlite::Connection::open(f.home.store()).unwrap();
    let binds: i64 = conn
        .query_row(
            "SELECT count(*) FROM events WHERE kind = 'repository_bind'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(binds, 1);
}

#[test]
fn a_clone_a_taken_binding_shows_unreadable_is_counted_by_the_server() {
    let f = Fixture::new();
    let mut store = Store::open(&f.home.store()).unwrap();
    administration::node::Node::open(&f.home, &mut store, &mut key::random::OsEntropy).unwrap();
    drop(store);
    let app = f.dir.path().join("app");
    std::fs::create_dir_all(app.join(".git/refs/heads")).unwrap();
    std::fs::write(app.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    std::fs::write(
        app.join(".git/config"),
        "[extensions]\n\tobjectformat = sha256\n",
    )
    .unwrap();
    f.bind_in(SESSION, &f.main(), HARNESS, &app);
    let mut running = f.running(Some(HARNESS), None);

    running.bind();

    assert!(f.counted().contains(&"repository_read_failed".to_owned()));
}
