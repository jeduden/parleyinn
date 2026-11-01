//! The answer store in a review: an unchanged key is taken from it, a
//! changed one runs, and only an accepted answer is kept.

use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use review_eval::host::{Command, Output};
use testkit::TempDir;

use crate::controls::SystemClock;
use crate::fake::{self, FakeHost};
use crate::key::Keys;
use crate::plan::{Plan, parse_dispatch};
use crate::run::{AgentRun, Options, Review, review_on};
use crate::store::{self, Store};

const DISPATCH: &str =
    r#"{"review-code": ["lang-rust", "test-engineer-unit"], "review-spec": ["lang-markdown"]}"#;

fn plan() -> Plan {
    Plan {
        class: "standard".to_owned(),
        touched: 2,
        dispatch: parse_dispatch(DISPATCH).unwrap(),
    }
}

/// A work directory defining every agent of [`DISPATCH`], each
/// specialist with a diff, and a store beside it.
fn workdir() -> TempDir {
    let dir = TempDir::new().unwrap();
    for name in ["lang-rust", "test-engineer-unit", "lang-markdown"] {
        fake::define(dir.path(), name, 20);
        write(dir.path(), &format!("review/diffs/{name}.diff"), "+a\n");
    }
    for name in ["review-code", "review-spec"] {
        fake::define(dir.path(), name, 40);
    }
    dir
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn store_in(dir: &Path) -> Store {
    Store {
        dir: dir.join("answers"),
        read_only: false,
    }
}

/// What one review with the store gave.
struct Ran {
    runs: Vec<AgentRun>,
    lines: Vec<String>,
    /// The agents started, sorted: they run in parallel.
    started: Vec<String>,
}

impl Ran {
    fn run(&self, name: &str) -> &AgentRun {
        self.runs.iter().find(|r| r.name == name).unwrap()
    }
}

/// Reviews `dir` with `store`, split at `split`, `claude` answering as
/// `script`.
fn reviewed(
    dir: &Path,
    store: &Store,
    split: usize,
    script: impl Fn(&Command) -> Result<Output, String> + Sync + 'static,
) -> Ran {
    let host = FakeHost::new(script);
    let plan = plan();
    let keys = Keys::compute(dir, &plan, fake::VERSION, split);
    let lines = Mutex::new(Vec::new());
    let runs = review_on(&Review {
        host: &host,
        clock: &SystemClock::default(),
        workdir: dir,
        plan: &plan,
        known: &review_gate::report::invariants(),
        options: Options {
            deadline: Duration::from_secs(9),
            split,
            breaker: crate::controls::Fraction::WHOLE,
            ..Options::default()
        },
        resume: None,
        keys: Some(&keys),
        store: Some(store),
        scrubber: None,
        progress: &|l| lines.lock().unwrap().push(l.to_owned()),
    })
    .unwrap();
    let mut started = host.agents();
    started.sort();
    Ran {
        runs,
        lines: lines.into_inner().unwrap(),
        started,
    }
}

fn review(dir: &Path, store: &Store) -> Ran {
    reviewed(dir, store, crate::parts::LIMIT, fake::either)
}

#[test]
fn an_unchanged_key_is_taken_from_the_store_and_runs_nothing() {
    let dir = workdir();
    let store = store_in(dir.path());
    let first = review(dir.path(), &store);
    assert_eq!(first.started.len(), 5);
    assert!(
        first
            .runs
            .iter()
            .all(|r| r.answer_key.is_some() && !r.reused)
    );
    let report = fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap();
    fs::remove_dir_all(dir.path().join("review")).unwrap();
    for name in ["lang-rust", "test-engineer-unit", "lang-markdown"] {
        write(dir.path(), &format!("review/diffs/{name}.diff"), "+a\n");
    }

    let second = review(dir.path(), &store);

    assert!(second.started.is_empty(), "{:?}", second.started);
    for run in &second.runs {
        assert!(run.answered() && run.reused && run.from_store, "{run:?}");
        assert_eq!(
            run.answer_key,
            first.run(&run.name).answer_key,
            "{}",
            run.name
        );
    }
    assert_eq!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap(),
        report
    );
    assert!(dir.path().join("review/logs/review-code.jsonl").exists());
    assert!(
        second
            .lines
            .contains(&"lang-rust (specialist): reused from the answer store".to_owned()),
        "{:?}",
        second.lines
    );
}

#[test]
fn a_changed_diff_runs_exactly_its_specialist_and_its_generalist() {
    let dir = workdir();
    let store = store_in(dir.path());
    review(dir.path(), &store);
    write(dir.path(), "review/diffs/lang-markdown.diff", "+b\n");

    let second = review(dir.path(), &store);

    assert_eq!(second.started, ["lang-markdown", "review-spec"]);
    assert!(second.run("lang-rust").from_store);
    assert!(second.run("review-code").from_store);
}

#[test]
fn a_specialist_s_fresh_report_that_differs_runs_its_generalist_again() {
    let dir = workdir();
    let store = store_in(dir.path());
    review(dir.path(), &store);
    // The specialist's answer is taken out of the store, and its second
    // sample states nothing.
    let key = Keys::compute(dir.path(), &plan(), fake::VERSION, crate::parts::LIMIT)
        .own("lang-markdown")
        .unwrap()
        .to_owned();
    store.drop(&key).unwrap();

    let second = reviewed(dir.path(), &store, crate::parts::LIMIT, |c| {
        if fake::agent_of(c) == "lang-markdown" {
            fake::answer(c, &[], "outcome: none — nothing.")
        } else {
            fake::either(c)
        }
    });

    assert_eq!(second.started, ["lang-markdown", "review-spec"]);
}

#[test]
fn a_refused_or_silent_run_is_never_stored() {
    let dir = workdir();
    let store = store_in(dir.path());
    let bad = "src/a.rs:3 — if not addressed: x — ENG-999";
    let first = reviewed(
        dir.path(),
        &store,
        crate::parts::LIMIT,
        move |c| match fake::agent_of(c).as_str() {
            "lang-rust" => fake::answer(c, &[bad], "outcome: findings — one."),
            "test-engineer-unit" => Err("cannot start".to_owned()),
            _ => fake::either(c),
        },
    );
    let key = |name: &str| first.run(name).answer_key.clone().unwrap();

    for name in ["lang-rust", "test-engineer-unit"] {
        assert_eq!(store.get(&key(name)), Ok(None), "{name}");
    }
    assert!(store.get(&key("lang-markdown")).unwrap().is_some());
    assert!(
        first.lines.iter().any(|l| l.starts_with(
            "lang-rust (specialist): its answer was not stored: its report is refused"
        )),
        "{:?}",
        first.lines
    );

    let second = review(dir.path(), &store);

    assert_eq!(
        second.started,
        ["lang-rust", "review-code", "test-engineer-unit"]
    );
}

#[test]
fn a_tampered_stored_report_is_run_again_not_trusted() {
    let dir = workdir();
    let store = store_in(dir.path());
    let first = review(dir.path(), &store);
    let key = first.run("lang-rust").answer_key.clone().unwrap();
    let report = store.dir.join(&key).join(store::REPORT);
    let text = fs::read_to_string(&report).unwrap();
    fs::write(&report, text.replace(" — I9", " — ENG-999")).unwrap();

    let second = review(dir.path(), &store);

    // Its fresh report is the one first stored, so its generalist's key
    // is unchanged.
    assert_eq!(second.started, ["lang-rust"]);
    assert!(
        second.lines.iter().any(|l| l.starts_with(&format!(
            "lang-rust (specialist): its stored answer {key} is not taken, so it runs: its report is not the one its log assembles"
        ))),
        "{:?}",
        second.lines
    );
    // The poisoned answer is taken out and the fresh one stored in its
    // place, so the next review takes it.
    assert!(
        second.lines.iter().any(|l| l.starts_with(&format!(
            "lang-rust (specialist): its stored answer {key} was dropped, so its new answer is stored in its place"
        ))),
        "{:?}",
        second.lines
    );
    assert!(fs::read_to_string(&report).unwrap().contains(" — I9"));
    assert!(review(dir.path(), &store).started.is_empty());
}

#[test]
fn a_poisoned_answer_in_a_store_read_alone_stays_and_the_agent_runs_every_time() {
    let dir = workdir();
    let store = store_in(dir.path());
    let first = review(dir.path(), &store);
    let key = first.run("lang-rust").answer_key.clone().unwrap();
    let report = store.dir.join(&key).join(store::REPORT);
    let text = fs::read_to_string(&report).unwrap();
    fs::write(&report, text.replace(" — I9", " — ENG-999")).unwrap();
    let reader = Store {
        read_only: true,
        ..store
    };

    let second = review(dir.path(), &reader);

    assert_eq!(second.started, ["lang-rust"]);
    assert!(fs::read_to_string(&report).unwrap().contains("ENG-999"));
    assert!(second.lines.iter().all(|l| !l.contains("was dropped")));
    assert!(
        second.lines.iter().any(|l| l
            == "lang-rust (specialist): its answer was not stored: the answer store is read alone"),
        "{:?}",
        second.lines
    );
}

#[test]
fn a_stored_answer_that_cannot_be_dropped_is_told() {
    let dir = workdir();
    let store = store_in(dir.path());
    let first = review(dir.path(), &store);
    let key = first.run("lang-rust").answer_key.clone().unwrap();
    // A file where its directory stands neither reads nor goes.
    fs::remove_dir_all(store.dir.join(&key)).unwrap();
    fs::write(store.dir.join(&key), "a file").unwrap();

    let second = review(dir.path(), &store);

    assert!(
        second
            .lines
            .iter()
            .any(|l| l.contains(&format!("its stored answer {key} was not dropped: drop "))),
        "{:?}",
        second.lines
    );
}

#[test]
fn an_answer_stored_by_another_run_meanwhile_is_told_and_kept() {
    let dir = workdir();
    let store = store_in(dir.path());
    let key = review(dir.path(), &store)
        .run("lang-rust")
        .answer_key
        .clone()
        .unwrap();
    let other = TempDir::new().unwrap();
    let fresh = store_in(other.path());
    let entry = fresh.dir.join(&key);

    let ran = reviewed(dir.path(), &fresh, crate::parts::LIMIT, move |c| {
        if c.args.iter().any(|a| a == "lang-rust") {
            fs::create_dir_all(&entry).unwrap();
        }
        fake::either(c)
    });

    assert!(
        ran.lines.iter().any(
            |l| l == &format!("lang-rust (specialist): its answer was already stored as {key}")
        ),
        "{:?}",
        ran.lines
    );
}

#[test]
fn a_stored_answer_that_does_not_read_is_told_and_run_again() {
    let dir = workdir();
    let store = store_in(dir.path());
    let first = review(dir.path(), &store);
    let key = first.run("lang-rust").answer_key.clone().unwrap();
    fs::remove_file(store.dir.join(&key).join(store::LOG)).unwrap();

    let second = review(dir.path(), &store);

    assert_eq!(second.started, ["lang-rust"]);
    assert!(
        second
            .lines
            .iter()
            .any(|l| l.contains(&format!("its stored answer {key} does not read"))),
        "{:?}",
        second.lines
    );
}

#[test]
fn a_store_read_alone_is_read_and_never_written() {
    let dir = workdir();
    let writer = store_in(dir.path());
    review(dir.path(), &writer);
    let reader = Store {
        read_only: true,
        ..writer.clone()
    };
    write(dir.path(), "review/diffs/lang-markdown.diff", "+b\n");

    let second = review(dir.path(), &reader);
    let third = review(dir.path(), &reader);

    assert_eq!(second.started, ["lang-markdown", "review-spec"]);
    assert_eq!(third.started, ["lang-markdown", "review-spec"]);
}

#[test]
fn a_split_specialist_s_parts_are_each_stored_and_taken() {
    let dir = workdir();
    let store = store_in(dir.path());
    let diff = "diff --git a/a.rs b/a.rs\n+a\ndiff --git a/b.rs b/b.rs\n+b\n";
    write(dir.path(), "review/diffs/lang-rust.diff", diff);
    let first = reviewed(dir.path(), &store, 10, fake::either);
    assert_eq!(first.run("lang-rust").parts, 2);
    fs::remove_dir_all(dir.path().join("review/reports")).unwrap();

    let second = reviewed(dir.path(), &store, 10, fake::either);

    assert!(second.started.is_empty(), "{:?}", second.started);
    let rust = second.run("lang-rust");
    assert!(rust.answered(), "{:?}", rust.failure);
    assert_eq!(rust.answer_key, first.run("lang-rust").answer_key);
    assert!(dir.path().join("review/reports/lang-rust.md").exists());
}

#[test]
fn an_answer_that_cannot_be_stored_is_told_and_counted() {
    let dir = workdir();
    fs::write(dir.path().join("answers"), "a file").unwrap();
    let store = store_in(dir.path());

    let ran = review(dir.path(), &store);

    assert!(ran.runs.iter().all(|r| r.answered() && r.store_failed));
    assert!(
        ran.lines
            .iter()
            .any(|l| l.starts_with("lang-rust (specialist): its answer was not stored: store ")),
        "{:?}",
        ran.lines
    );
}

#[test]
fn an_answer_whose_log_is_gone_is_told_and_counted() {
    let dir = workdir();
    let store = store_in(dir.path());
    let host = FakeHost::new(fake::either);
    let plan = plan();
    let lines = Mutex::new(Vec::new());
    let progress = |l: &str| lines.lock().unwrap().push(l.to_owned());
    let known = review_gate::report::invariants();
    let review = Review {
        host: &host,
        clock: &SystemClock::default(),
        workdir: dir.path(),
        plan: &plan,
        known: &known,
        options: Options::default(),
        resume: None,
        keys: None,
        store: None,
        scrubber: None,
        progress: &progress,
    };
    let unit = crate::run::Unit::whole("review-code".to_owned());
    let run = crate::run::reused("review-code".to_owned(), unit.kind, "x", Vec::new());

    let failed = super::keep(&review, &store, &unit, &"a".repeat(64), &(run, None));

    assert!(failed);
    // Without its keys a review looks nothing up.
    assert_eq!(super::key_of(&review, &unit), None);
    let lines = lines.lock().unwrap().clone();
    assert!(
        lines[0].starts_with("review-code (generalist): its answer was not stored: read "),
        "{lines:?}"
    );
}

#[test]
fn a_reused_specialist_whose_report_cannot_be_written_did_not_answer() {
    let dir = workdir();
    let store = store_in(dir.path());
    review(dir.path(), &store);
    for rel in ["review/reports/lang-rust.md", "review/logs/lang-rust.jsonl"] {
        let path = dir.path().join(rel);
        fs::remove_file(&path).unwrap();
        fs::create_dir_all(&path).unwrap();
    }

    let second = review(dir.path(), &store);

    let rust = second.run("lang-rust");
    assert!(
        !rust.answered() && !rust.reused && !rust.from_store,
        "{rust:?}"
    );
    assert!(
        second
            .lines
            .iter()
            .any(|l| l.starts_with("lang-rust (specialist): its stored log was not copied")),
        "{:?}",
        second.lines
    );
}

#[test]
fn an_answer_that_held_the_token_is_never_stored_and_is_counted() {
    const TOKEN: &str = "dummy-access-token-0123456789";
    let dir = workdir();
    let store = store_in(dir.path());
    let host = FakeHost::new(fake::either);
    let plan = plan();
    let lines = Mutex::new(Vec::new());
    let progress = |l: &str| lines.lock().unwrap().push(l.to_owned());
    let known = review_gate::report::invariants();
    let scrubber = review_eval::secret::Scrubber::new(&[TOKEN]);
    let review = Review {
        host: &host,
        clock: &SystemClock::default(),
        workdir: dir.path(),
        plan: &plan,
        known: &known,
        options: Options::default(),
        resume: None,
        keys: None,
        store: None,
        scrubber: Some(&scrubber),
        progress: &progress,
    };
    let unit = crate::run::Unit::whole("review-code".to_owned());
    write(
        dir.path(),
        &format!("review/logs/{}.jsonl", unit.stem()),
        &format!("{{\"note\":\"{TOKEN}\"}}\n"),
    );
    let run = crate::run::reused("review-code".to_owned(), unit.kind, "x", Vec::new());
    let key = "a".repeat(64);

    let failed = super::keep(&review, &store, &unit, &key, &(run, None));

    assert!(failed);
    assert!(!dir.path().join("answers").join(&key).exists());
    let lines = lines.lock().unwrap().clone();
    assert!(lines[0].ends_with("it held the Claude token"), "{lines:?}");
}
