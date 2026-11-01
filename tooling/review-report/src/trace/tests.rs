use std::fs;
use std::time::Duration;

use testkit::TempDir;

use super::*;

/// A run that answered: its result reports a total cost unlike the sum
/// its models' costs give, so a span that recomputed cost would differ.
const ANSWERED: &str = r#"{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":10,"cache_creation_input_tokens":20,"cache_read_input_tokens":30,"output_tokens":40}}}
{"type":"result","subtype":"success","is_error":false,"result":"outcome: none — fine","total_cost_usd":0.1234,"num_turns":3,"duration_ms":4000,"modelUsage":{"claude-haiku-5-5":{"inputTokens":10,"outputTokens":40,"cacheReadInputTokens":30,"cacheCreationInputTokens":20,"costUSD":0.9}}}
"#;

/// A run its deadline cut off: calls but no result.
const CUT: &str = r#"{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":1,"output_tokens":2}}}
"#;

fn log(file: &str, text: &str, modified_ms: u64) -> LogFile {
    LogFile {
        file: file.to_owned(),
        text: text.to_owned(),
        modified_ms: Some(modified_ms),
        created_ms: Some(modified_ms - 500),
    }
}

fn definition(name: &str) -> (String, String) {
    (format!("model-of-{name}"), "low".to_owned())
}

fn gap(name: &str, part: Option<usize>, reason: &str) -> Gap {
    Gap {
        name: name.to_owned(),
        kind: "specialist".to_owned(),
        part,
        reason: reason.to_owned(),
    }
}

#[test]
fn a_log_file_name_says_its_agent_its_part_and_whether_it_was_a_first_attempt() {
    assert_eq!(
        stem("lang-rust.jsonl"),
        Some(("lang-rust".to_owned(), None, false))
    );
    assert_eq!(
        stem("lang-rust.first.jsonl"),
        Some(("lang-rust".to_owned(), None, true))
    );
    assert_eq!(
        stem("lang-rust.2.jsonl"),
        Some(("lang-rust".to_owned(), Some(2), false))
    );
    assert_eq!(
        stem("lang-rust.2.first.jsonl"),
        Some(("lang-rust".to_owned(), Some(2), true))
    );
    assert_eq!(stem("a.b.jsonl"), Some(("a.b".to_owned(), None, false)));
    assert_eq!(stem("lang-rust.stderr"), None);
    assert_eq!(stem(".jsonl"), None);
}

#[test]
fn a_run_ends_when_its_log_was_written_and_starts_its_duration_before() {
    assert_eq!(
        times(4000, Some(10_000), Some(9_000)),
        (Some(6_000), Some(10_000))
    );
    // No result, so no duration: it started when its log was made.
    assert_eq!(
        times(0, Some(10_000), Some(9_000)),
        (Some(9_000), Some(10_000))
    );
    assert_eq!(times(0, Some(10_000), None), (Some(10_000), Some(10_000)));
    assert_eq!(times(4000, None, Some(9_000)), (Some(9_000), None));
    assert_eq!(times(20_000, Some(10_000), None), (Some(0), Some(10_000)));
}

#[test]
fn the_span_cost_is_the_log_s_own_total_never_recomputed() {
    let spans = spans(
        &[log("lang-rust.jsonl", ANSWERED, 10_000)],
        &[],
        &[],
        &definition,
    );

    let span = &spans[0];
    // The result's total_cost_usd, not its models' 0.9, nor a price
    // table's sum of tokens.
    assert!((span.cost_usd - 0.1234).abs() < 1e-12, "{}", span.cost_usd);
    assert_eq!(span.name, "lang-rust");
    assert_eq!(span.kind, "specialist");
    assert_eq!((span.attempt, span.turns), (1, 3));
    assert_eq!(span.model, "claude-haiku-5-5");
    assert_eq!(span.effort, "low");
    assert_eq!(
        (span.started_ms, span.ended_ms),
        (Some(6_000), Some(10_000))
    );
    assert_eq!(
        span.tokens,
        Tokens {
            fresh: 10,
            cache_write: 20,
            cache_read: 30,
            output: 40
        }
    );
    assert_eq!(
        (span.status, span.reason.as_deref()),
        (Status::Accepted, None)
    );
    assert!(!span.reused);
}

#[test]
fn a_first_attempt_is_a_gap_retried_and_its_retry_attempt_two() {
    let logs = [
        log("lang-rust.first.jsonl", CUT, 5_000),
        log("lang-rust.jsonl", ANSWERED, 10_000),
        log("review-code.first.jsonl", ANSWERED, 5_000),
        log("review-code.jsonl", CUT, 10_000),
    ];
    let gaps = [gap("review-code", None, "silent: its run ended with none")];

    let spans = spans(&logs, &gaps, &[], &definition);

    let got: Vec<_> = spans
        .iter()
        .map(|s| (s.name.as_str(), s.attempt, s.status, s.reason.as_deref()))
        .collect();
    assert_eq!(
        got,
        vec![
            (
                "lang-rust",
                1,
                Status::Gap,
                Some("retried: its run ended with none")
            ),
            ("lang-rust", 2, Status::Accepted, None),
            (
                "review-code",
                1,
                Status::Gap,
                Some("retried: its answer or report was not accepted")
            ),
            (
                "review-code",
                2,
                Status::Gap,
                Some("silent: its run ended with none")
            ),
        ]
    );
    // A run with no model in its log takes its definition's.
    assert_eq!(spans[3].kind, "generalist");
}

#[test]
fn a_part_s_gap_marks_that_part_and_a_whole_agent_s_gap_every_part() {
    let logs = [
        log("lang-rust.1.jsonl", ANSWERED, 10_000),
        log("lang-rust.2.jsonl", CUT, 10_000),
        log("lang-json.1.jsonl", ANSWERED, 10_000),
        log("lang-json.2.jsonl", ANSWERED, 10_000),
    ];
    let gaps = [
        gap("lang-rust", Some(2), "silent: x"),
        gap("lang-json", None, "refused: y"),
    ];

    let spans = spans(&logs, &gaps, &[], &definition);

    let got: Vec<_> = spans
        .iter()
        .map(|s| (s.name.as_str(), s.part, s.status))
        .collect();
    assert_eq!(
        got,
        vec![
            ("lang-json", Some(1), Status::Gap),
            ("lang-json", Some(2), Status::Gap),
            ("lang-rust", Some(1), Status::Accepted),
            ("lang-rust", Some(2), Status::Gap),
        ]
    );
}

#[test]
fn an_agent_with_no_log_still_has_a_span_a_gap_or_reused() {
    let gaps = [gap(
        "lang-sql",
        None,
        "silent: its definition does not read",
    )];
    let usage = [
        Usage {
            agent: "lang-toml".to_owned(),
            cost_usd: 0.0,
            reused: true,
        },
        Usage {
            agent: "lang-rust".to_owned(),
            cost_usd: 0.1,
            reused: true,
        },
    ];

    let spans = spans(
        &[log("lang-rust.jsonl", ANSWERED, 10_000)],
        &gaps,
        &usage,
        &definition,
    );

    let got: Vec<_> = spans
        .iter()
        .map(|s| (s.name.as_str(), s.attempt, s.status, s.reused, s.cost_usd))
        .collect();
    assert_eq!(
        got,
        vec![
            ("lang-rust", 1, Status::Accepted, true, 0.0),
            ("lang-sql", 0, Status::Gap, false, 0.0),
            ("lang-toml", 0, Status::Accepted, true, 0.0),
        ]
    );
    assert_eq!(spans[1].model, "model-of-lang-sql");
    assert_eq!(spans[1].started_ms, None);
}

#[test]
fn a_log_that_names_no_model_takes_its_definition_s() {
    let spans = spans(&[log("lang-json.jsonl", "", 10_000)], &[], &[], &definition);

    assert_eq!(spans[0].model, "model-of-lang-json");
    assert_eq!(spans[0].cost_usd, 0.0);
}

#[test]
fn the_header_names_the_head_the_verdict_the_counts_and_the_span_bounds() {
    let outcome = Outcome {
        verdict: "request_changes".to_owned(),
        findings: ["blocking", "nit", "nit"]
            .iter()
            .map(|s| inputs::OutcomeFinding {
                severity: (*s).to_owned(),
            })
            .collect(),
    };
    let span = |start, end| Span {
        started_ms: start,
        ended_ms: end,
        ..Span::default()
    };
    let spans = [
        span(Some(5), Some(9)),
        span(Some(3), Some(7)),
        span(None, None),
    ];
    let run_id = RunId {
        head: Some("abc".to_owned()),
        inputs: String::new(),
    };

    let h = header(&run_id, &outcome, 2, &spans, None, Vec::new());

    assert_eq!(h.id, "abc");
    assert_eq!((h.started, h.finished), (Some(3), Some(9)));
    assert_eq!(h.verdict, "request_changes");
    assert_eq!((h.findings_blocking, h.findings_nit, h.gaps), (1, 2, 1 + 1));
    assert_eq!(h.failure, None);
}

#[test]
fn a_review_without_a_head_or_a_verdict_or_that_failed_says_so() {
    let inputs = RunId {
        head: None,
        inputs: "0123456789abcdef0123".to_owned(),
    };
    let none = header(&inputs, &Outcome::default(), 0, &[], None, Vec::new());
    assert_eq!(none.id, "run-0123456789abcdef");
    assert_eq!(none.verdict, NO_VERDICT);
    assert_eq!((none.started, none.finished), (None, None));

    let failed = header(
        &RunId::default(),
        &Outcome::default(),
        0,
        &[],
        Some("plan: no dispatch"),
        vec!["n".to_owned()],
    );
    assert_eq!(failed.id, "unknown");
    assert_eq!(failed.verdict, FAILED);
    assert_eq!(failed.failure.as_deref(), Some("plan: no dispatch"));
    assert_eq!(failed.notes, vec!["n"]);
}

#[test]
fn the_trace_is_its_header_line_then_one_line_per_span() {
    let trace = Trace {
        header: Header {
            id: "abc".to_owned(),
            verdict: "approve".to_owned(),
            ..Header::default()
        },
        spans: spans(
            &[log("lang-rust.2.jsonl", ANSWERED, 10_000)],
            &[gap("lang-sql", None, "silent: x")],
            &[],
            &definition,
        ),
    };

    let text = trace.jsonl().unwrap();
    let lines: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["trace"]["id"], "abc");
    assert_eq!(lines[0]["trace"]["verdict"], "approve");
    assert!(lines[0]["trace"].get("failure").is_none());
    assert_eq!(lines[1]["name"], "lang-rust");
    assert_eq!(lines[1]["part"], 2);
    assert_eq!(lines[1]["cost_usd"], 0.1234);
    assert_eq!(lines[1]["tokens"]["cache_read"], 30);
    assert_eq!(lines[1]["status"], "accepted");
    assert!(lines[1].get("reason").is_none() && lines[1].get("reused").is_none());
    assert_eq!(lines[2]["status"], "gap");
    assert_eq!(lines[2]["reason"], "silent: x");
    assert_eq!(lines[2]["attempt"], 0);
    assert!(lines[2]["started_ms"].is_null());
}

#[test]
fn a_review_s_trace_is_built_from_its_files_and_its_logs() {
    let dir = TempDir::new().unwrap();
    let logs = dir.path().join(inputs::LOGS);
    fs::create_dir_all(&logs).unwrap();
    fs::write(logs.join("lang-rust.jsonl"), ANSWERED).unwrap();
    fs::write(logs.join("lang-rust.stderr"), "noise").unwrap();
    fs::write(
        dir.path().join(inputs::OUTCOME),
        r#"{"verdict":"approve","summary":"","findings":[]}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join(inputs::RUN_ID),
        r#"{"head":"abc","inputs":"x"}"#,
    )
    .unwrap();
    fs::write(dir.path().join(inputs::GAPS), "not json").unwrap();

    let trace = build(dir.path(), None);

    assert_eq!(trace.header.id, "abc");
    assert_eq!(trace.header.verdict, "approve");
    assert_eq!(trace.header.notes.len(), 1, "{:?}", trace.header.notes);
    assert_eq!(trace.spans.len(), 1);
    assert_eq!(trace.spans[0].effort, "");
    assert!(trace.spans[0].ended_ms.is_some());
}

#[test]
fn a_log_that_does_not_read_is_named_and_a_missing_log_directory_is_none() {
    let dir = TempDir::new().unwrap();
    let mut notes = Vec::new();
    assert!(read_logs(&dir.path().join("none"), &mut notes).is_empty());
    assert!(notes.is_empty());

    fs::create_dir_all(dir.path().join("logs/lang-rust.jsonl")).unwrap();
    fs::write(dir.path().join("file"), "").unwrap();
    assert!(read_logs(&dir.path().join("logs"), &mut notes).is_empty());
    assert!(read_logs(&dir.path().join("file"), &mut notes).is_empty());
    assert_eq!(notes.len(), 2, "{notes:?}");
}

#[cfg(unix)]
#[test]
fn a_log_whose_name_is_not_utf8_is_passed_by() {
    use std::os::unix::ffi::OsStrExt;
    let dir = TempDir::new().unwrap();
    let name = std::ffi::OsStr::from_bytes(b"\xff.jsonl");
    fs::write(dir.path().join(name), ANSWERED).unwrap();
    let mut notes = Vec::new();

    assert!(read_logs(dir.path(), &mut notes).is_empty());
    assert!(notes.is_empty());
}

#[test]
fn a_time_that_does_not_read_or_is_before_the_epoch_is_none() {
    assert_eq!(ms(Err(io::Error::other("no time"))), None);
    assert_eq!(ms(Ok(UNIX_EPOCH - Duration::from_secs(1))), None);
    assert_eq!(ms(Ok(UNIX_EPOCH + Duration::from_millis(7))), Some(7));
}
