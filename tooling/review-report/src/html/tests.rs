use crate::agent_runs::{Header, Tokens};

use super::*;

/// What a hostile run could make its name and its reason.
const HOSTILE: &str = "<script>alert(\"x\")</script>' onload=\"x";

fn agent_run(name: &str, status: Status, times: (Option<u64>, Option<u64>)) -> AgentRun {
    AgentRun {
        name: name.to_owned(),
        kind: "specialist".to_owned(),
        attempt: 1,
        model: "claude-haiku-5-5".to_owned(),
        effort: "low".to_owned(),
        started_ms: times.0,
        ended_ms: times.1,
        turns: 4,
        tokens: Tokens {
            fresh: 1,
            cache_write: 2,
            cache_read: 3,
            output: 4,
        },
        cost_usd: 0.5,
        status,
        reason: (status == Status::Gap).then(|| "silent: its run ended with none".to_owned()),
        ..AgentRun::default()
    }
}

fn runs_of(agent_runs: Vec<AgentRun>) -> AgentRuns {
    AgentRuns {
        header: Header {
            id: "abc".to_owned(),
            started: agent_runs.iter().filter_map(|s| s.started_ms).min(),
            finished: agent_runs.iter().filter_map(|s| s.ended_ms).max(),
            verdict: "approve".to_owned(),
            findings_blocking: 1,
            findings_nit: 2,
            gaps: 0,
            ..Header::default()
        },
        agent_runs,
    }
}

#[test]
fn every_character_html_gives_meaning_is_escaped() {
    assert_eq!(
        escape("<a href=\"x\">'&'</a>"),
        "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
    );
    assert_eq!(escape("plain — text"), "plain — text");
}

#[test]
fn a_long_string_is_cut_on_a_character_then_escaped() {
    let long = format!("{}<", "é".repeat(MAX_TEXT));
    let cut = text(&long);

    assert_eq!(cut, format!("{}…", "é".repeat(MAX_TEXT)));
    assert_eq!(text("<"), "&lt;");
}

#[test]
fn a_run_s_name_reason_and_the_failure_never_reach_the_report_as_markup() {
    let mut hostile = agent_run(HOSTILE, Status::Gap, (Some(0), Some(10)));
    hostile.reason = Some(HOSTILE.to_owned());
    hostile.model = HOSTILE.to_owned();
    hostile.effort = HOSTILE.to_owned();
    hostile.kind = HOSTILE.to_owned();
    let mut t = runs_of(vec![hostile]);
    t.header.id = HOSTILE.to_owned();
    t.header.verdict = HOSTILE.to_owned();
    t.header.failure = Some(HOSTILE.to_owned());
    t.header.notes = vec![HOSTILE.to_owned()];
    let gaps = [Gap {
        name: HOSTILE.to_owned(),
        kind: HOSTILE.to_owned(),
        part: Some(1),
        reason: HOSTILE.to_owned(),
    }];

    let report = render(&t, &gaps);

    assert!(!report.contains("<script"), "{report}");
    assert!(!report.contains("onload=\""), "{report}");
    assert!(!report.contains("alert(\""), "{report}");
    assert!(
        report.contains("&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;&#39; onload=&quot;x")
    );
    // The report itself refuses any script, should an escape ever slip.
    assert!(report.contains("default-src 'none'"));
}

/// The report of a review of three runs, one a reused part that left a
/// gap, and one gap without a timed log.
fn three_runs_report() -> String {
    let mut part = agent_run("lang-rust", Status::Gap, (Some(1_000), Some(3_000)));
    part.part = Some(2);
    part.reused = true;
    let t = runs_of(vec![
        agent_run("review-code", Status::Accepted, (Some(0), Some(4_000))),
        part,
        agent_run("lang-sql", Status::Gap, (None, None)),
    ]);
    let gaps = [Gap {
        name: "lang-rust".to_owned(),
        kind: "specialist".to_owned(),
        part: Some(2),
        reason: "silent: x".to_owned(),
    }];

    render(&t, &gaps)
}

#[test]
fn the_report_holds_the_verdict_counts_and_timeline() {
    let report = three_runs_report();

    assert!(report.starts_with("<!DOCTYPE html>"));
    assert!(report.contains("Verdict: <strong>approve</strong>"));
    assert!(report.contains("<li>blocking findings: 1</li><li>nits: 2</li>"));
    assert!(report.contains("<li>cost: 1.5000 USD</li>"));
    assert!(report.contains("style=\"left:0.00%;width:100.00%\""));
    assert!(report.contains("style=\"left:25.00%;width:50.00%\""));
    assert!(report.contains("<span>2.0 s</span>"));
    for load in ["src=", "href=", "url(", "<script", "@import"] {
        assert!(!report.contains(load), "the report loads nothing: {load}");
    }
}

#[test]
fn the_report_holds_the_tables_and_what_was_not_reviewed() {
    let report = three_runs_report();

    assert!(report.contains("lang-rust part 2 #1"));
    assert!(report.contains("gap (reused)"));
    assert!(report.contains("<td class=\"n\">1.5000</td>"));
    assert!(report.contains("<td class=\"n\">3</td><td class=\"n\">4</td></tr>"));
    assert!(report.contains("silent: its run ended with none"));
    assert!(report.contains("<li><code>lang-rust part 2</code> (specialist): silent: x</li>"));
    assert!(!report.contains("Files that did not read"));
}

#[test]
fn a_review_with_no_timed_run_and_no_gap_says_so() {
    let mut t = runs_of(vec![agent_run(
        "review-code",
        Status::Accepted,
        (None, None),
    )]);
    t.header.notes = vec!["review/usage.json does not read: x".to_owned()];

    let report = render(&t, &[]);

    assert!(report.contains("No run left a timed log."));
    assert!(report.contains("Every run's answer was taken."));
    assert!(report.contains("Every dispatched agent has an accepted report or answer."));
    assert!(report.contains("<li>review/usage.json does not read: x</li>"));
}

#[test]
fn a_gap_without_a_reason_shows_an_empty_one() {
    let mut s = agent_run("lang-rust", Status::Gap, (None, None));
    s.reason = None;

    let report = render(&runs_of(vec![s]), &[]);

    assert!(report.contains("<tr><td>lang-rust #1</td><td></td></tr>"));
}

#[test]
fn the_report_stays_bounded_however_many_runs_and_gaps_there_are() {
    let agent_runs: Vec<AgentRun> = (0..MAX_ROWS + 50)
        .map(|n| {
            let mut s = agent_run(
                &"x".repeat(5_000),
                Status::Gap,
                (Some(n as u64), Some(n as u64 + 1)),
            );
            s.reason = Some("y".repeat(5_000));
            s
        })
        .collect();
    let gaps: Vec<Gap> = (0..MAX_ROWS + 50)
        .map(|_| Gap {
            name: "z".repeat(5_000),
            reason: "w".repeat(5_000),
            ..Gap::default()
        })
        .collect();

    let report = render(&runs_of(agent_runs), &gaps);

    assert!(report.contains("50 more rows not shown; the runs holds them."));
    assert!(report.len() < 3 * 1024 * 1024, "{} bytes", report.len());
}
