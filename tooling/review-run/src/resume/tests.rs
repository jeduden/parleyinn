use std::fs;

use testkit::TempDir;

use super::*;
use crate::plan::parse_dispatch;
use crate::run_id::AgentId;

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

fn plan() -> Plan {
    Plan {
        class: "standard".to_owned(),
        touched: 1,
        dispatch: parse_dispatch(
            r#"{"review-code": ["lang-rust", "test-engineer-unit"], "review-security": []}"#,
        )
        .unwrap(),
    }
}

/// An agent's identity, its key one its definition and diff give.
fn agent(definition: &str, diff: Option<&str>) -> AgentId {
    AgentId {
        definition: Some(definition.to_owned()),
        diff: diff.map(str::to_owned),
        key: Some(format!("k-{definition}-{diff:?}")),
    }
}

fn id() -> RunId {
    RunId {
        head: Some(HEAD.to_owned()),
        inputs: "i".to_owned(),
        split: 9,
        agents: [
            ("lang-rust".to_owned(), agent("d1", Some("x1"))),
            ("test-engineer-unit".to_owned(), agent("d2", Some("x2"))),
            ("review-code".to_owned(), agent("d3", None)),
            ("review-security".to_owned(), agent("d4", None)),
        ]
        .into(),
        resumed_from: None,
        claude: None,
    }
}

fn answered(answer: &str) -> RunLog {
    RunLog {
        answered: true,
        answer: answer.to_owned(),
        ..RunLog::default()
    }
}

/// An earlier run on [`id`] that took every agent's answer, its gaps
/// `gaps`.
fn earlier(gaps: Vec<Gap>) -> Earlier {
    let names = [
        "lang-rust",
        "test-engineer-unit",
        "review-code",
        "review-security",
    ];
    Earlier {
        dir: PathBuf::from("old"),
        id: id(),
        gaps,
        answers: names
            .iter()
            .map(|n| ((*n).to_owned(), format!("{n} answered")))
            .collect(),
        consulted: names.iter().map(|n| (*n).to_owned()).collect(),
        parts: BTreeMap::new(),
        stalls: Vec::new(),
    }
}

fn whole(name: &str) -> Reuse {
    Reuse::Whole(format!("{name} answered"))
}

#[test]
fn with_no_gap_and_nothing_changed_every_agent_keeps_its_answer() {
    let resume = plan_of(&earlier(Vec::new()), &id());

    assert_eq!(resume.from, "old");
    assert!(resume.notes.is_empty(), "{:?}", resume.notes);
    assert_eq!(resume.reuse.len(), 4);
    assert_eq!(resume.reuse["lang-rust"], whole("lang-rust"));
    assert_eq!(resume.reuse["review-security"], whole("review-security"));
}

fn plan_of(earlier: &Earlier, now: &RunId) -> Resume {
    super::plan(&plan(), earlier, now, "old", crate::stall::MAX_RESUMES)
}

#[test]
fn a_gap_runs_again_with_every_generalist_that_weighs_it() {
    let gaps = vec![Gap::new("lang-rust", None, "silent: x".to_owned())];

    let resume = plan_of(&earlier(gaps), &id());

    assert_eq!(
        resume.notes,
        [
            "lang-rust: runs again: silent: x",
            "review-code: runs again: its specialist lang-rust runs again",
        ]
    );
    assert_eq!(
        resume.reuse.keys().collect::<Vec<_>>(),
        ["review-security", "test-engineer-unit"]
    );
}

#[test]
fn a_generalist_that_was_a_gap_runs_again_alone() {
    let gaps = vec![Gap::new("review-security", None, "unread: y".to_owned())];

    let resume = plan_of(&earlier(gaps), &id());

    assert_eq!(resume.notes, ["review-security: runs again: unread: y"]);
    assert_eq!(resume.reuse.len(), 3);
}

#[test]
fn a_moved_or_otherwise_fed_review_keeps_every_agent_whose_key_is_unchanged() {
    let mut moved = id();
    moved.head = Some("89abcdef0123456789abcdef0123456789abcdef".to_owned());
    let mut unknown = id();
    unknown.head = None;
    let mut inputs = id();
    inputs.inputs = "j".to_owned();
    for (now, notes) in [(moved, 1), (unknown, 0), (inputs, 0)] {
        let resume = plan_of(&earlier(Vec::new()), &now);

        assert_eq!(resume.reuse.len(), 4, "{:?}", resume.notes);
        assert_eq!(resume.notes.len(), notes, "{:?}", resume.notes);
    }
    let mut moved = id();
    moved.head = Some("89abcdef0123456789abcdef0123456789abcdef".to_owned());
    let resume = plan_of(&earlier(Vec::new()), &moved);
    assert!(
        resume.notes[0].starts_with(&format!("the head changed from {HEAD} to 89ab")),
        "{:?}",
        resume.notes
    );
}

#[test]
fn a_changed_key_runs_its_agent_again_with_why() {
    let mut now = id();
    now.agents.get_mut("lang-rust").unwrap().key = Some("other".to_owned());
    now.agents.get_mut("review-security").unwrap().key = None;
    let mut was = earlier(Vec::new());
    was.id.agents.get_mut("test-engineer-unit").unwrap().key = None;

    let resume = plan_of(&was, &now);

    assert_eq!(
        resume.notes,
        [
            "lang-rust: runs again: its answer key changed: its model, effort, turns or \
             prompt, the claude version, the split limit, the pull request's agent \
             instructions or a file every key reads",
            "test-engineer-unit: runs again: the earlier run recorded no answer key for it",
            "review-code: runs again: its specialist lang-rust runs again",
            "review-security: runs again: its answer key is unknown",
        ]
    );
    assert!(resume.reuse.is_empty());
}

#[test]
fn a_changed_definition_or_diff_or_an_agent_new_or_unanswered_runs_again() {
    let mut now = id();
    now.agents
        .insert("lang-rust".to_owned(), agent("other", Some("x1")));
    now.agents
        .insert("test-engineer-unit".to_owned(), agent("d2", Some("other")));
    let mut was = earlier(Vec::new());
    was.id.agents.remove("review-security");
    was.consulted.remove("review-code");

    let resume = plan_of(&was, &now);

    assert_eq!(
        resume.notes,
        [
            "lang-rust: runs again: its definition changed",
            "test-engineer-unit: runs again: its diff changed",
            "review-code: runs again: its specialist lang-rust runs again",
            "review-security: runs again: it was not in the earlier run",
        ]
    );
    assert!(resume.reuse.is_empty());

    // A definition that did not read in the earlier run has no key, so
    // it never matches.
    let mut was = earlier(Vec::new());
    was.id.agents.get_mut("review-security").unwrap().key = None;
    was.answers.remove("lang-rust");
    let resume = plan_of(&was, &id());
    assert_eq!(
        resume.notes,
        [
            "lang-rust: runs again: the earlier run took no answer from it",
            "review-code: runs again: its specialist lang-rust runs again",
            "review-security: runs again: the earlier run recorded no answer key for it",
        ]
    );
}

#[test]
fn a_specialist_named_by_parts_keeps_the_parts_that_answered() {
    let gaps = vec![Gap::new("lang-rust", Some(2), "silent: x".to_owned())];
    let mut was = earlier(gaps);
    was.parts.insert(
        "lang-rust".to_owned(),
        [
            (1, answered("outcome: none — one.")),
            (2, RunLog::default()),
            (3, answered("outcome: none — three.")),
            (4, answered(" ")),
        ]
        .into(),
    );

    let resume = plan_of(&was, &id());

    let Reuse::Parts(kept) = &resume.reuse["lang-rust"] else {
        panic!("{:?}", resume.reuse["lang-rust"]);
    };
    assert_eq!(kept.keys().collect::<Vec<_>>(), [&1, &3]);
    assert_eq!(
        resume.notes,
        [
            "lang-rust: runs again in part, keeping part 1, part 3",
            "review-code: runs again: its specialist lang-rust runs again",
        ]
    );
}

#[test]
fn a_part_that_read_part_of_its_diff_runs_again_beside_one_that_did_not_run() {
    // The outcome names part 1's partial read beside part 2's silence,
    // so part 1 is not kept, and with no part left to keep the whole
    // specialist runs again.
    let gaps = vec![
        Gap::new("lang-rust", Some(1), "partial: lines 5-9".to_owned()),
        Gap::new("lang-rust", Some(2), "silent: x".to_owned()),
    ];
    let mut was = earlier(gaps);
    was.parts.insert(
        "lang-rust".to_owned(),
        [
            (1, answered("outcome: none — one.")),
            (2, RunLog::default()),
            (3, answered("outcome: none — three.")),
        ]
        .into(),
    );

    let resume = plan_of(&was, &id());

    let Reuse::Parts(kept) = &resume.reuse["lang-rust"] else {
        panic!("{:?}", resume.reuse["lang-rust"]);
    };
    assert_eq!(kept.keys().collect::<Vec<_>>(), [&3]);
    was.parts.get_mut("lang-rust").unwrap().remove(&3);
    let resume = plan_of(&was, &id());
    assert!(!resume.reuse.contains_key("lang-rust"));
    assert_eq!(resume.notes[0], "lang-rust: runs again: partial: lines 5-9");
}

#[test]
fn a_specialist_named_whole_or_with_no_part_to_keep_runs_again_whole() {
    let both = vec![
        Gap::new("lang-rust", Some(2), "silent: x".to_owned()),
        Gap::new("lang-rust", None, "refused: y".to_owned()),
    ];
    let resume = plan_of(&earlier(both), &id());
    assert!(resume.notes[0].starts_with("lang-rust: runs again: silent: x"));
    assert!(!resume.reuse.contains_key("lang-rust"));

    let parts = vec![Gap::new("lang-rust", Some(1), "silent: x".to_owned())];
    let resume = plan_of(&earlier(parts.clone()), &id());
    assert_eq!(resume.notes[0], "lang-rust: runs again: silent: x");

    let mut now = id();
    now.agents
        .insert("lang-rust".to_owned(), agent("d1", Some("x9")));
    let resume = plan_of(&earlier(parts), &now);
    assert_eq!(resume.notes[0], "lang-rust: runs again: its diff changed");
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// An earlier run's review directory under `dir`, `old/`, as
/// `review-run` leaves it, its gaps `gaps`.
fn earlier_dir(dir: &Path, gaps: &str) -> PathBuf {
    write(dir, "old/gaps.json", gaps);
    write(
        dir,
        "old/run-id.json",
        &serde_json::to_string(&id()).unwrap(),
    );
    write(
        dir,
        "old/answers.json",
        r#"{"lang-rust": "outcome: none — a."}"#,
    );
    write(dir, "old/consulted.json", r#"["lang-rust"]"#);
    write(
        dir,
        "old/logs/lang-rust.1.jsonl",
        "{\"type\":\"result\",\"subtype\":\"success\",\"result\":\"outcome: none — a.\"}\n",
    );
    write(dir, "old/logs/lang-rust.2.jsonl", "");
    write(dir, "old/logs/lang-rust.jsonl", "");
    write(dir, "old/logs/lang-rust-x.1.jsonl", "");
    write(dir, "old/logs/lang-rust.first.jsonl", "");
    dir.join("old")
}

#[test]
fn the_earlier_run_reads_with_its_parts_logs() {
    let dir = TempDir::new().unwrap();
    let old = earlier_dir(
        dir.path(),
        r#"[{"name": "lang-rust", "kind": "specialist", "part": 2, "reason": "silent: x"},
            {"name": "lang-rust", "kind": "specialist", "part": 3, "reason": "silent: x"}]"#,
    );

    let was = read(&old.join("gaps.json"), &old).unwrap();

    assert_eq!(was.dir, old);
    assert_eq!(was.id, id());
    assert_eq!(was.gaps.len(), 2);
    assert_eq!(was.answers["lang-rust"], "outcome: none — a.");
    assert_eq!(was.consulted, ["lang-rust".to_owned()].into());
    write(&old, "logs/other.1.jsonl", "");
    let again = read(&old.join("gaps.json"), &old).unwrap();
    assert_eq!(again.parts["lang-rust"].len(), 2);
    let parts = &was.parts["lang-rust"];
    assert_eq!(parts.keys().collect::<Vec<_>>(), [&1, &2]);
    assert!(parts[&1].answered);
}

#[test]
fn an_earlier_run_that_does_not_read_is_named() {
    for (rel, text, want) in [
        ("gaps.json", "{", "gaps.json: malformed gaps"),
        ("run-id.json", "{", "run-id.json: malformed run id"),
        ("answers.json", "[", "answers.json: "),
        ("consulted.json", "{", "consulted.json: "),
    ] {
        let dir = TempDir::new().unwrap();
        let old = earlier_dir(dir.path(), "[]");
        write(&old, rel, text);

        let err = read(&old.join("gaps.json"), &old).unwrap_err();

        assert!(err.contains(want), "{rel}: {err}");
    }
    let dir = TempDir::new().unwrap();
    let old = earlier_dir(dir.path(), "[]");
    fs::remove_file(old.join("run-id.json")).unwrap();
    let err = read(&old.join("gaps.json"), &old).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
    for gone in ["answers.json", "consulted.json"] {
        let dir = TempDir::new().unwrap();
        let old = earlier_dir(dir.path(), "[]");
        fs::remove_file(old.join(gone)).unwrap();
        let err = read(&old.join("gaps.json"), &old).unwrap_err();
        assert!(err.starts_with("read ") && err.contains(gone), "{err}");
    }
}

#[test]
fn a_directory_entry_that_does_not_list_is_named_with_its_directory() {
    let entries = vec![Ok(1), Err(io::Error::other("gone")), Ok(3)];
    let err = listed(entries.into_iter(), Path::new("logs")).unwrap_err();
    assert_eq!(err, "read logs: gone");
    assert_eq!(listed([Ok(1)].into_iter(), Path::new("x")).unwrap(), [1]);
}

#[test]
fn part_logs_that_do_not_read_are_named() {
    let gaps = r#"[{"name": "lang-rust", "kind": "specialist", "part": 2, "reason": "x"}]"#;
    let dir = TempDir::new().unwrap();
    let old = earlier_dir(dir.path(), gaps);
    fs::remove_file(old.join("logs/lang-rust.2.jsonl")).unwrap();
    fs::create_dir(old.join("logs/lang-rust.2.jsonl")).unwrap();
    let err = read(&old.join("gaps.json"), &old).unwrap_err();
    assert!(err.contains("lang-rust.2.jsonl"), "{err}");

    fs::remove_dir_all(old.join("logs")).unwrap();
    let err = read(&old.join("gaps.json"), &old).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
}

#[test]
fn a_kept_agent_s_report_and_logs_are_copied_and_a_kept_part_s_alone() {
    let dir = TempDir::new().unwrap();
    let old = earlier_dir(dir.path(), "[]");
    write(&old, "reports/lang-rust.md", "outcome: none — a.\n");
    write(&old, "logs/review-code.jsonl", "g");
    write(&old, "logs/test-engineer-unit.1.jsonl", "p1");
    write(&old, "logs/test-engineer-unit.2.first.stderr", "p2");
    let work = dir.path().join("work");
    let resume = Resume {
        from: "old".to_owned(),
        reuse: [
            ("lang-rust".to_owned(), whole("lang-rust")),
            ("review-code".to_owned(), whole("review-code")),
            (
                "test-engineer-unit".to_owned(),
                Reuse::Parts([(1, RunLog::default())].into()),
            ),
        ]
        .into(),
        notes: Vec::new(),
        ..Resume::default()
    };

    copy(&work, &old, &plan(), &resume).unwrap();

    let mut logs: Vec<String> = fs::read_dir(work.join(LOGS))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    logs.sort();
    assert_eq!(
        logs,
        [
            "lang-rust.1.jsonl",
            "lang-rust.2.jsonl",
            "lang-rust.first.jsonl",
            "lang-rust.jsonl",
            "review-code.jsonl",
            "test-engineer-unit.1.jsonl",
        ]
    );
    assert_eq!(
        fs::read_to_string(work.join(report_file("lang-rust"))).unwrap(),
        "outcome: none — a.\n"
    );
    // A generalist has no report, and a missing one is left out.
    assert!(!work.join(report_file("review-code")).exists());
}

#[test]
fn a_kept_file_that_cannot_be_copied_is_named_and_missing_logs_are_none() {
    let dir = TempDir::new().unwrap();
    let old = dir.path().join("old");
    let work = dir.path().join("work");
    let resume = Resume {
        from: "old".to_owned(),
        reuse: [("lang-rust".to_owned(), whole("lang-rust"))].into(),
        notes: Vec::new(),
        ..Resume::default()
    };
    copy(&work, &old, &plan(), &resume).unwrap();

    write(&old, "reports/lang-rust.md", "r");
    fs::create_dir_all(work.join(report_file("lang-rust"))).unwrap();
    let err = copy(&work, &old, &plan(), &resume).unwrap_err();
    assert!(err.starts_with("copy "), "{err}");

    fs::remove_dir_all(work.join(report_file("lang-rust"))).unwrap();
    write(&old, "logs", "not a directory");
    let err = copy(&work, &old, &plan(), &resume).unwrap_err();
    assert!(err.starts_with("read "), "{err}");

    write(dir.path(), "blocked", "a file");
    let err = copy(&dir.path().join("blocked"), &old, &plan(), &resume).unwrap_err();
    assert!(err.starts_with("make "), "{err}");
}

#[test]
fn a_kept_log_that_cannot_be_copied_is_named() {
    let dir = TempDir::new().unwrap();
    let old = earlier_dir(dir.path(), "[]");
    let work = dir.path().join("work");
    let resume = Resume {
        from: "old".to_owned(),
        reuse: [("lang-rust".to_owned(), whole("lang-rust"))].into(),
        ..Resume::default()
    };
    fs::create_dir_all(work.join(LOGS).join("lang-rust.jsonl")).unwrap();

    let err = copy(&work, &old, &plan(), &resume).unwrap_err();

    assert!(
        err.starts_with("copy ") && err.contains("lang-rust.jsonl"),
        "{err}"
    );
}

#[test]
fn a_log_is_an_agent_s_own_its_first_attempt_s_or_a_kept_part_s() {
    let all = whole("lang-rust");
    let part = Reuse::Parts([(2, RunLog::default())].into());
    for (file, reuse, want) in [
        ("lang-rust.jsonl", &all, true),
        ("lang-rust.first.stderr", &all, true),
        ("lang-rust.3.first.jsonl", &all, true),
        ("lang-rust.x.jsonl", &all, false),
        ("lang-rust.3.other", &all, false),
        ("lang-rust-x.jsonl", &all, false),
        ("lang-rust.2.jsonl", &part, true),
        ("lang-rust.1.jsonl", &part, false),
        ("lang-rust.jsonl", &part, false),
        ("lang-rust.x.jsonl", &part, false),
    ] {
        assert_eq!(is_log_of(file, "lang-rust", reuse), want, "{file}");
    }
}

#[test]
fn a_log_whose_name_is_not_utf8_is_not_copied() {
    use std::os::unix::ffi::OsStrExt;
    let dir = TempDir::new().unwrap();
    let old = earlier_dir(dir.path(), "[]");
    let odd = std::ffi::OsStr::from_bytes(b"lang-rust.\xff.jsonl");
    fs::write(old.join("logs").join(odd), "").unwrap();
    let work = dir.path().join("work");
    let resume = Resume {
        from: "old".to_owned(),
        reuse: [("lang-rust".to_owned(), whole("lang-rust"))].into(),
        notes: Vec::new(),
        ..Resume::default()
    };

    let notes = copy(&work, &old, &plan(), &resume).unwrap();

    assert_eq!(
        notes,
        ["lang-rust: log lang-rust.\u{fffd}.jsonl not copied: its name is not UTF-8"]
    );
    assert!(!work.join(LOGS).join(odd).exists());
    assert!(work.join(LOGS).join("lang-rust.jsonl").exists());
}

#[test]
fn an_agent_stalled_past_the_resumes_allowed_is_not_run_again() {
    let gaps = vec![Gap::new("lang-rust", None, "silent: x".to_owned())];
    let mut was = earlier(gaps.clone());
    was.stalls = (0..3).fold(Vec::new(), |s, _| crate::stall::tally(&gaps, &s));

    let resume = plan_of(&was, &id());

    let why = "stalled: lang-rust failed 3 times the same way; needs a person: silent: x; \
               silent: x; silent: x";
    assert_eq!(resume.stalled["lang-rust"], why);
    assert_eq!(resume.stalls, was.stalls);
    assert!(!resume.reuse.contains_key("lang-rust"));
    assert_eq!(
        resume.notes,
        [
            format!("lang-rust: not run again: {why}"),
            "review-code: runs again: its specialist lang-rust runs again".to_owned(),
        ]
    );
    // Allowed one more resume, it runs again.
    let more = super::plan(&plan(), &was, &id(), "old", 3);
    assert!(more.stalled.is_empty());
}

#[test]
fn an_earlier_run_s_stalls_read_when_there_and_as_none_when_not() {
    let dir = TempDir::new().unwrap();
    let old = earlier_dir(dir.path(), "[]");
    assert!(
        read(&old.join("gaps.json"), &old)
            .unwrap()
            .stalls
            .is_empty()
    );

    write(
        &old,
        "stalls.json",
        r#"[{"name": "lang-rust", "class": "silent:", "times": 2, "reasons": ["a", "b"]}]"#,
    );
    assert_eq!(
        read(&old.join("gaps.json"), &old).unwrap().stalls[0].times,
        2
    );

    fs::remove_file(old.join("stalls.json")).unwrap();
    fs::create_dir(old.join("stalls.json")).unwrap();
    let err = read(&old.join("gaps.json"), &old).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
}

#[test]
fn a_stalled_generalist_is_not_run_again_and_its_specialists_keep_theirs() {
    let gaps = vec![Gap::new("review-security", None, "unread: y".to_owned())];
    let mut was = earlier(gaps.clone());
    was.stalls = (0..3).fold(Vec::new(), |s, _| crate::stall::tally(&gaps, &s));

    let resume = plan_of(&was, &id());

    assert!(resume.stalled.contains_key("review-security"));
    assert!(!resume.reuse.contains_key("review-security"));
    assert_eq!(resume.reuse.len(), 3);
}
