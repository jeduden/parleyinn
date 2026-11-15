use std::fs;

use testkit::TempDir;

use super::*;

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "89abcdef0123456789abcdef0123456789abcdef";
const DISPATCH_TEXT: &str = r#"{"review-code": ["lang-rust"]}"#;
const OUTCOME_TEXT: &str = r#"{"verdict":"approve","summary":"fine","findings":[]}"#;

fn run_id_of(head: Option<&str>) -> String {
    let head = head.map_or("null".to_owned(), |h| format!("\"{h}\""));
    format!(r#"{{"head":{head},"inputs":"i","split":1,"agents":{{}}}}"#)
}

fn kept() -> Kept {
    Kept {
        run_id: run_id_of(Some(HEAD)),
        gaps: "[]".to_owned(),
        dispatch: DISPATCH_TEXT.to_owned(),
        outcome: OUTCOME_TEXT.to_owned(),
        consulted: r#"["lang-rust","review-code"]"#.to_owned(),
    }
}

/// Writes `kept` as an earlier review's directory.
fn dir_of(kept: &Kept) -> TempDir {
    let dir = TempDir::new().unwrap();
    for (name, text) in [
        ("run-id.json", &kept.run_id),
        ("gaps.json", &kept.gaps),
        ("dispatch.json", &kept.dispatch),
        ("outcome.json", &kept.outcome),
        ("consulted.json", &kept.consulted),
    ] {
        fs::write(dir.path().join(name), text).unwrap();
    }
    dir
}

#[test]
fn the_fixture_s_outcome_reads_as_the_gate_reads_it() {
    review_gate::parse_outcome(OUTCOME_TEXT).unwrap();
}

#[test]
fn the_lines_say_whether_the_verdict_is_reused_and_why_not() {
    assert_eq!(Verdict::Reused.lines(), "reused=true\n");
    assert_eq!(
        Verdict::Fresh("the head moved".to_owned()).lines(),
        "reused=false\nwhy=the head moved\n"
    );
}

#[test]
fn what_an_earlier_review_kept_is_read_from_its_directory() {
    let dir = dir_of(&kept());

    assert_eq!(read(dir.path()).unwrap(), kept());
}

#[test]
fn a_file_the_earlier_review_did_not_keep_is_named() {
    let dir = dir_of(&kept());
    fs::remove_file(dir.path().join("gaps.json")).unwrap();

    let err = read(dir.path()).unwrap_err();

    assert!(
        err.starts_with("the earlier review kept no gaps.json: "),
        "{err}"
    );
}

#[test]
fn the_same_head_under_the_same_dispatch_with_no_gap_is_reused() {
    let dir = dir_of(&kept());
    let spaced = r#"{ "review-code" : [ "lang-rust" ] }"#;

    assert_eq!(weigh(dir.path(), HEAD, DISPATCH_TEXT), Verdict::Reused);
    assert_eq!(weigh(dir.path(), HEAD, spaced), Verdict::Reused);
}

fn why(kept: &Kept, head: &str, dispatch: &str) -> String {
    match weigh(dir_of(kept).path(), head, dispatch) {
        Verdict::Fresh(why) => why,
        Verdict::Reused => "reused".to_owned(),
    }
}

#[test]
fn another_head_or_none_is_not_reused() {
    assert_eq!(
        why(&kept(), OTHER, DISPATCH_TEXT),
        format!("the earlier review was of {HEAD}, not {OTHER}")
    );
    let headless = Kept {
        run_id: run_id_of(None),
        ..kept()
    };
    assert_eq!(
        why(&headless, HEAD, DISPATCH_TEXT),
        "the earlier review recorded no head"
    );
}

#[test]
fn a_changed_dispatch_is_not_reused() {
    assert_eq!(
        why(
            &kept(),
            HEAD,
            r#"{"review-code": ["lang-rust", "lang-toml"]}"#
        ),
        "the dispatch changed since the earlier review"
    );
}

#[test]
fn a_review_that_left_a_gap_is_not_reused() {
    let gapped = Kept {
        gaps: r#"[{"name":"lang-rust","kind":"specialist","reason":"silent"}]"#.to_owned(),
        ..kept()
    };

    assert_eq!(
        why(&gapped, HEAD, DISPATCH_TEXT),
        "the earlier review left 1 agent run(s) without an accepted report"
    );
}

#[test]
fn a_kept_file_that_does_not_read_is_named_and_not_reused() {
    let cases = [
        (
            Kept {
                run_id: "{".to_owned(),
                ..kept()
            },
            "malformed run id: ",
        ),
        (
            Kept {
                gaps: "{".to_owned(),
                ..kept()
            },
            "malformed gaps: ",
        ),
        (
            Kept {
                dispatch: "{".to_owned(),
                ..kept()
            },
            "the earlier review's dispatch does not read: ",
        ),
        (
            Kept {
                outcome: "{".to_owned(),
                ..kept()
            },
            "the earlier review's outcome does not read: ",
        ),
        (
            Kept {
                consulted: "{".to_owned(),
                ..kept()
            },
            "the earlier review's consulted agents do not read: ",
        ),
    ];

    for (kept, start) in cases {
        let got = why(&kept, HEAD, DISPATCH_TEXT);
        assert!(got.starts_with(start), "{got}");
    }
    let got = why(&kept(), HEAD, "{");
    assert!(
        got.starts_with("this review's dispatch does not read: "),
        "{got}"
    );
}

#[test]
fn what_was_kept_is_weighed_without_a_directory() {
    assert_eq!(weigh_kept(&kept(), HEAD, DISPATCH_TEXT), Verdict::Reused);
    assert_eq!(
        weigh_kept(&kept(), OTHER, DISPATCH_TEXT),
        Verdict::Fresh(format!("the earlier review was of {HEAD}, not {OTHER}"))
    );
}
