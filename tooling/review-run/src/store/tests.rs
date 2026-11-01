use std::collections::BTreeSet;
use std::fs;

use testkit::TempDir;

use super::*;
use crate::fake;
use crate::usage::AgentUsage;

const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const FINDING: &str = fake::FINDING;
const OUTCOME: &str = "outcome: findings — lang-rust saw one finding.";

fn known() -> BTreeSet<String> {
    ["I9".to_owned()].into()
}

fn usage() -> AgentUsage {
    AgentUsage {
        agent: "lang-rust".to_owned(),
        calls: 1,
        fresh: 10,
        cache_write: 20,
        cache_read: 30,
        output: 5,
        attempts: 1,
        parts: 1,
        cost_usd: 0.01,
        reused: false,
        resumed_from: None,
        answer_key: Some(KEY.to_owned()),
        answer_store: false,
    }
}

/// A specialist's accepted answer: its log and its report.
fn answer() -> (String, String) {
    let log = fake::log_of(&[FINDING, OUTCOME], "success", OUTCOME, 0.01);
    let parsed = review_eval::log::parse(&log);
    (log, assemble(&parsed.texts, &parsed.answer, None))
}

fn store(dir: &TempDir) -> Store {
    Store {
        dir: dir.path().join("answers"),
        read_only: false,
    }
}

#[test]
fn an_answer_stored_reads_back_whole() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    let (log, report) = answer();

    let put = store.put(KEY, &log, Some(&report), &usage());

    assert_eq!(put, Ok(Put::Stored));
    let got = store.get(KEY).unwrap().unwrap();
    assert_eq!(got.log_text, log);
    assert_eq!(got.report.as_deref(), Some(report.as_str()));
    assert_eq!(got.log.answer, OUTCOME);
    let usage_text = fs::read_to_string(store.dir.join(KEY).join(USAGE_FILE)).unwrap();
    assert!(usage_text.contains("\"cost_usd\": 0.01"), "{usage_text}");
    // Nothing is left beside it but its own directory.
    let names: Vec<String> = fs::read_dir(&store.dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, [KEY]);
}

#[test]
fn a_generalist_s_answer_is_stored_with_no_report() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);

    store
        .put(KEY, &fake::log("x", 0.1), None, &usage())
        .unwrap();

    assert_eq!(store.get(KEY).unwrap().unwrap().report, None);
}

#[test]
fn a_key_stored_once_is_kept_as_the_first_writer_left_it() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    let (log, report) = answer();
    store.put(KEY, &log, Some(&report), &usage()).unwrap();

    let again = store.put(KEY, "other", Some("other"), &usage());

    assert_eq!(again, Ok(Put::Present));
    assert_eq!(store.get(KEY).unwrap().unwrap().log_text, log);
}

#[test]
fn a_rename_that_finds_the_key_written_meanwhile_keeps_that_one() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    let (log, report) = answer();
    // The other writer renames first, between this one's check and its
    // rename.
    let raced = store.put_racing(KEY, &log, Some(&report), &usage(), &|| {
        let other = store.dir.join(KEY);
        fs::create_dir_all(&other).unwrap();
        fs::write(other.join(LOG), "first").unwrap();
    });

    assert_eq!(raced, Ok(Put::Present));
    assert_eq!(store.get(KEY).unwrap().unwrap().log_text, "first");
    assert_eq!(fs::read_dir(&store.dir).unwrap().count(), 1);
}

#[test]
fn a_store_read_alone_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let store = Store {
        read_only: true,
        ..store(&dir)
    };
    let (log, report) = answer();

    assert_eq!(
        store.put(KEY, &log, Some(&report), &usage()),
        Ok(Put::ReadOnly)
    );
    assert!(!store.dir.exists());
}

#[test]
fn a_key_not_stored_reads_as_none_and_one_that_does_not_read_is_named() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);

    assert_eq!(store.get(KEY), Ok(None));
    fs::create_dir_all(store.dir.join(KEY)).unwrap();
    let err = store.get(KEY).unwrap_err();
    assert!(err.contains(LOG), "{err}");
    fs::write(store.dir.join(KEY).join(LOG), "{}").unwrap();
    fs::create_dir_all(store.dir.join(KEY).join(REPORT)).unwrap();
    let err = store.get(KEY).unwrap_err();
    assert!(err.contains(REPORT), "{err}");
}

#[test]
fn no_key_but_a_full_sha_256_is_read_written_or_dropped() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    for bad in ["../x", "", "ABC", &KEY[..63]] {
        assert!(store.get(bad).is_err(), "{bad}");
        assert!(store.put(bad, "", None, &usage()).is_err(), "{bad}");
        assert!(store.drop(bad).is_err(), "{bad}");
    }
}

#[test]
fn a_dropped_key_is_gone_and_one_never_stored_is_named() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    let (log, report) = answer();
    store.put(KEY, &log, Some(&report), &usage()).unwrap();

    assert_eq!(store.drop(KEY), Ok(()));
    assert_eq!(store.get(KEY), Ok(None));
    let err = store.drop(KEY).unwrap_err();
    assert!(err.contains("no answer is stored under"), "{err}");
}

#[test]
fn a_store_that_cannot_be_made_is_named() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("answers"), "a file").unwrap();
    let store = store(&dir);
    let (log, report) = answer();

    let err = store.put(KEY, &log, Some(&report), &usage()).unwrap_err();

    assert!(err.contains("answers"), "{err}");
    let err = store.drop(KEY).unwrap_err();
    assert!(err.contains("drop"), "{err}");
}

#[test]
fn an_accepted_specialist_answer_is_one_its_log_and_report_agree_on() {
    let (log, report) = answer();
    let log = review_eval::log::parse(&log);

    assert_eq!(
        accepted(Kind::Specialist, &log, Some(&report), &known()),
        Ok(())
    );
}

#[test]
fn no_answer_but_a_successful_one_with_a_report_the_gate_takes_is_accepted() {
    let (text, report) = answer();
    let log = review_eval::log::parse(&text);
    let failed = review_eval::log::parse(&fake::log_of(&[FINDING], "error_max_turns", "", 0.01));
    let empty = review_eval::log::parse(&fake::log_of(&[], "success", " ", 0.01));
    let tampered = report.replace("a long line", "a short line");
    let unknown = BTreeSet::new();
    let cases: [(&RunLog, Option<&str>, &BTreeSet<String>, &str); 5] = [
        (
            &failed,
            Some(&report),
            &known(),
            "shows no successful result",
        ),
        (&empty, Some(&report), &known(), "answered nothing"),
        (&log, None, &known(), "holds no report"),
        (
            &log,
            Some(&tampered),
            &known(),
            "not the one its log assembles",
        ),
        (&log, Some(&report), &unknown, "refused"),
    ];
    for (log, report, known, want) in cases {
        let err = accepted(Kind::Specialist, log, report, known).unwrap_err();
        assert!(err.contains(want), "{want}: {err}");
    }
}

#[test]
fn a_generalist_s_answer_is_accepted_only_when_the_review_can_read_it() {
    let read = review_eval::log::parse(&fake::log(
        "src/a.rs:3 — nit — a long line — lang-rust\nsound.",
        0.1,
    ));
    let says_nothing = review_eval::log::parse(&fake::log("all good.", 0.1));

    assert_eq!(accepted(Kind::Generalist, &read, None, &known()), Ok(()));
    let err = accepted(Kind::Generalist, &says_nothing, None, &known()).unwrap_err();
    assert!(err.contains("no finding"), "{err}");
}

#[test]
fn a_rename_that_fails_with_no_answer_in_place_is_named() {
    let dir = TempDir::new().unwrap();
    let store = store(&dir);
    let (log, report) = answer();

    let err = store
        .put_racing(KEY, &log, Some(&report), &usage(), &|| {
            let other = store.dir.join(KEY);
            fs::create_dir_all(other.join("half")).unwrap();
        })
        .unwrap_err();

    assert!(err.starts_with("store "), "{err}");
    assert_eq!(fs::read_dir(&store.dir).unwrap().count(), 1);
}
