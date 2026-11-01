use crate::trace::{Header, Tokens};

use super::*;

/// What a hostile run could make its name and its reason.
const HOSTILE: &str = "<script>alert(\"x\")</script>' onload=\"x";

fn span(name: &str, status: Status, times: (Option<u64>, Option<u64>)) -> Span {
    Span {
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
        ..Span::default()
    }
}

fn trace(spans: Vec<Span>) -> Trace {
    Trace {
        header: Header {
            id: "abc".to_owned(),
            started: spans.iter().filter_map(|s| s.started_ms).min(),
            finished: spans.iter().filter_map(|s| s.ended_ms).max(),
            verdict: "approve".to_owned(),
            findings_blocking: 1,
            findings_nit: 2,
            gaps: 0,
            ..Header::default()
        },
        spans,
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
fn a_run_s_name_reason_and_the_failure_never_reach_the_page_as_markup() {
    let mut hostile = span(HOSTILE, Status::Gap, (Some(0), Some(10)));
    hostile.reason = Some(HOSTILE.to_owned());
    hostile.model = HOSTILE.to_owned();
    hostile.effort = HOSTILE.to_owned();
    hostile.kind = HOSTILE.to_owned();
    let mut t = trace(vec![hostile]);
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

    let page = render(&t, &gaps);

    assert!(!page.contains("<script"), "{page}");
    assert!(!page.contains("onload=\""), "{page}");
    assert!(!page.contains("alert(\""), "{page}");
    assert!(page.contains("&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;&#39; onload=&quot;x"));
    // The page itself refuses any script, should an escape ever slip.
    assert!(page.contains("default-src 'none'"));
}

/// The page of a review of three runs, one a reused part that left a
/// gap, and one gap without a timed log.
fn three_runs_page() -> String {
    let mut part = span("lang-rust", Status::Gap, (Some(1_000), Some(3_000)));
    part.part = Some(2);
    part.reused = true;
    let t = trace(vec![
        span("review-code", Status::Accepted, (Some(0), Some(4_000))),
        part,
        span("lang-sql", Status::Gap, (None, None)),
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
fn the_page_holds_the_verdict_counts_and_timeline() {
    let page = three_runs_page();

    assert!(page.starts_with("<!DOCTYPE html>"));
    assert!(page.contains("Verdict: <strong>approve</strong>"));
    assert!(page.contains("<li>blocking findings: 1</li><li>nits: 2</li>"));
    assert!(page.contains("<li>cost: 1.5000 USD</li>"));
    assert!(page.contains("style=\"left:0.00%;width:100.00%\""));
    assert!(page.contains("style=\"left:25.00%;width:50.00%\""));
    assert!(page.contains("<span>2.0 s</span>"));
    for load in ["src=", "href=", "url(", "<script", "@import"] {
        assert!(!page.contains(load), "the page loads nothing: {load}");
    }
}

#[test]
fn the_page_holds_the_tables_and_what_was_not_reviewed() {
    let page = three_runs_page();

    assert!(page.contains("lang-rust part 2 #1"));
    assert!(page.contains("gap (reused)"));
    assert!(page.contains("<td class=\"n\">1.5000</td>"));
    assert!(page.contains("<td class=\"n\">3</td><td class=\"n\">4</td></tr>"));
    assert!(page.contains("silent: its run ended with none"));
    assert!(page.contains("<li><code>lang-rust part 2</code> (specialist): silent: x</li>"));
    assert!(!page.contains("Files that did not read"));
}

#[test]
fn a_review_with_no_timed_run_and_no_gap_says_so() {
    let mut t = trace(vec![span("review-code", Status::Accepted, (None, None))]);
    t.header.notes = vec!["review/usage.json does not read: x".to_owned()];

    let page = render(&t, &[]);

    assert!(page.contains("No run left a timed log."));
    assert!(page.contains("Every run's answer was taken."));
    assert!(page.contains("Every dispatched agent has an accepted report or answer."));
    assert!(page.contains("<li>review/usage.json does not read: x</li>"));
}

#[test]
fn a_gap_without_a_reason_shows_an_empty_one() {
    let mut s = span("lang-rust", Status::Gap, (None, None));
    s.reason = None;

    let page = render(&trace(vec![s]), &[]);

    assert!(page.contains("<tr><td>lang-rust #1</td><td></td></tr>"));
}

#[test]
fn the_page_stays_bounded_however_many_runs_and_gaps_there_are() {
    let spans: Vec<Span> = (0..MAX_ROWS + 50)
        .map(|n| {
            let mut s = span(
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

    let page = render(&trace(spans), &gaps);

    assert!(page.contains("50 more rows not shown; the trace holds them."));
    assert!(page.len() < 3 * 1024 * 1024, "{} bytes", page.len());
}
