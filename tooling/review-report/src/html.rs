//! The report page: one static, self-contained HTML file built from a
//! review's trace and its gaps. It loads nothing: its style is inline,
//! it holds no script, and its content security policy refuses any
//! that an escaping slip would let in. Every string a run could shape
//! (an agent's name, a model, a reason, the head) is untrusted: it is
//! cut to [`MAX_TEXT`] characters and escaped before it reaches the
//! page. At most [`MAX_ROWS`] rows stand in a table, so the page stays
//! small whatever the review ran; the trace holds every span.

use std::fmt::Write as _;

use crate::inputs::Gap;
use crate::trace::{Span, Status, Trace};

/// The most rows a table holds; the rest are counted below it.
pub const MAX_ROWS: usize = 300;
/// The most characters of one string the page shows.
pub const MAX_TEXT: usize = 300;

/// The page's style: no external load, nothing executable.
const STYLE: &str = "body{font:14px/1.4 system-ui,sans-serif;margin:2em;color:#222}\
table{border-collapse:collapse;margin:1em 0}td,th{border:1px solid #ccc;padding:2px 6px;\
text-align:left}td.n{text-align:right}.tl{width:100%}.row{display:flex;align-items:center;\
gap:8px;margin:2px 0}.label{width:22em;overflow:hidden;white-space:nowrap}\
.track{position:relative;flex:1;height:12px;background:#f2f2f2}.bar{position:absolute;\
top:0;height:12px}.accepted{background:#3a7}.gap{background:#c43}";

/// `text` with every character HTML gives meaning to escaped, so it
/// stands as text in an element or in a quoted attribute.
#[must_use]
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }

    out
}

/// `text` cut to [`MAX_TEXT`] characters, then escaped.
#[must_use]
pub fn text(text: &str) -> String {
    match text.char_indices().nth(MAX_TEXT) {
        Some((at, _)) => format!("{}…", escape(&text[..at])),
        None => escape(text),
    }
}

/// The page for `trace`, `gaps` being what the review did not review.
#[must_use]
pub fn render(trace: &Trace, gaps: &[Gap]) -> String {
    let h = &trace.header;
    let mut s = String::new();
    let _ = write!(
        s,
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; \
         style-src 'unsafe-inline'\"><title>Review {id}</title><style>{STYLE}</style>\
         </head><body>\n<h1>Review {id}</h1>\n",
        id = text(&h.id)
    );
    summary(&mut s, trace);
    timeline(&mut s, trace);
    costs(&mut s, &trace.spans);
    token_table(&mut s, &trace.spans);
    gap_reasons(&mut s, &trace.spans);
    not_reviewed(&mut s, gaps);
    if !h.notes.is_empty() {
        s.push_str("<h2>Files that did not read</h2>\n<ul>\n");
        for note in h.notes.iter().take(MAX_ROWS) {
            let _ = writeln!(s, "<li>{}</li>", text(note));
        }
        s.push_str("</ul>\n");
    }
    s.push_str("</body></html>\n");

    s
}

/// The verdict and the counts.
fn summary(s: &mut String, trace: &Trace) {
    let h = &trace.header;
    let cost: f64 = trace.spans.iter().map(|span| span.cost_usd).sum();
    let _ = writeln!(s, "<p>Verdict: <strong>{}</strong></p>", text(&h.verdict));
    if let Some(failure) = &h.failure {
        let _ = writeln!(s, "<p>The review could not run: {}</p>", text(failure));
    }
    let _ = writeln!(
        s,
        "<ul><li>blocking findings: {}</li><li>nits: {}</li><li>not reviewed: {}</li>\
         <li>agent runs: {}</li><li>cost: {cost:.4} USD</li></ul>",
        h.findings_blocking,
        h.findings_nit,
        h.gaps,
        trace.spans.len()
    );
}

/// How a row names `span`: its agent, its part and its attempt.
fn label(span: &Span) -> String {
    let part = span.part.map(|p| format!(" part {p}")).unwrap_or_default();
    text(&format!("{}{part} #{}", span.name, span.attempt))
}

/// The class a span's bar takes.
fn class(status: Status) -> &'static str {
    match status {
        Status::Accepted => "accepted",
        Status::Gap => "gap",
    }
}

/// One bar per timed span, placed between the trace's start and end.
fn timeline(s: &mut String, trace: &Trace) {
    s.push_str("<h2>Timeline</h2>\n");
    let (Some(start), Some(end)) = (trace.header.started, trace.header.finished) else {
        s.push_str("<p>No run left a timed log.</p>\n");
        return;
    };
    let total = end.saturating_sub(start).max(1);
    let at = |ms: u64| (ms.saturating_sub(start) as f64) * 100.0 / (total as f64);
    s.push_str("<div class=\"tl\">\n");
    let timed = trace
        .spans
        .iter()
        .filter_map(|span| Some((span, span.started_ms?, span.ended_ms?)));
    for (span, from, to) in timed.take(MAX_ROWS) {
        let _ = writeln!(
            s,
            "<div class=\"row\"><span class=\"label\">{}</span><span class=\"track\">\
             <span class=\"bar {}\" style=\"left:{:.2}%;width:{:.2}%\"></span></span>\
             <span>{:.1} s</span></div>",
            label(span),
            class(span.status),
            at(from),
            (at(to) - at(from)).max(0.5),
            (to.saturating_sub(from) as f64) / 1000.0
        );
    }
    s.push_str("</div>\n");
    more(s, trace.spans.len());
}

/// The note under a table cut at [`MAX_ROWS`].
fn more(s: &mut String, rows: usize) {
    if rows > MAX_ROWS {
        let _ = writeln!(
            s,
            "<p>{} more rows not shown; the trace holds them.</p>",
            rows - MAX_ROWS
        );
    }
}

/// The cost of each run, by agent, model, attempt and part.
fn costs(s: &mut String, spans: &[Span]) {
    s.push_str(
        "<h2>Cost</h2>\n<table><tr><th>agent</th><th>kind</th><th>part</th><th>attempt</th>\
         <th>model</th><th>effort</th><th>turns</th><th>status</th><th>cost (USD)</th></tr>\n",
    );
    for span in spans.iter().take(MAX_ROWS) {
        let reused = if span.reused { " (reused)" } else { "" };
        let _ = writeln!(
            s,
            "<tr><td>{}</td><td>{}</td><td class=\"n\">{}</td><td class=\"n\">{}</td>\
             <td>{}</td><td>{}</td><td class=\"n\">{}</td><td>{}{reused}</td>\
             <td class=\"n\">{:.4}</td></tr>",
            text(&span.name),
            text(&span.kind),
            span.part.map(|p| p.to_string()).unwrap_or_default(),
            span.attempt,
            text(&span.model),
            text(&span.effort),
            span.turns,
            class(span.status),
            span.cost_usd
        );
    }
    let total: f64 = spans.iter().map(|span| span.cost_usd).sum();
    let _ = writeln!(
        s,
        "<tr><th colspan=\"8\">total</th><td class=\"n\">{total:.4}</td></tr></table>"
    );
    more(s, spans.len());
}

/// The tokens of each run, bucket by bucket.
fn token_table(s: &mut String, spans: &[Span]) {
    s.push_str(
        "<h2>Tokens</h2>\n<table><tr><th>run</th><th>fresh</th><th>cache write</th>\
         <th>cache read</th><th>output</th></tr>\n",
    );
    for span in spans.iter().take(MAX_ROWS) {
        let t = span.tokens;
        let _ = writeln!(
            s,
            "<tr><td>{}</td><td class=\"n\">{}</td><td class=\"n\">{}</td>\
             <td class=\"n\">{}</td><td class=\"n\">{}</td></tr>",
            label(span),
            t.fresh,
            t.cache_write,
            t.cache_read,
            t.output
        );
    }
    s.push_str("</table>\n");
    more(s, spans.len());
}

/// Why each run whose answer was not taken was not.
fn gap_reasons(s: &mut String, spans: &[Span]) {
    s.push_str("<h2>Gap reasons</h2>\n");
    let gaps: Vec<&Span> = spans.iter().filter(|x| x.status == Status::Gap).collect();
    if gaps.is_empty() {
        s.push_str("<p>Every run's answer was taken.</p>\n");
        return;
    }
    s.push_str("<table><tr><th>run</th><th>reason</th></tr>\n");
    for span in gaps.iter().take(MAX_ROWS) {
        let reason = span.reason.as_deref().unwrap_or_default();
        let _ = writeln!(
            s,
            "<tr><td>{}</td><td>{}</td></tr>",
            label(span),
            text(reason)
        );
    }
    s.push_str("</table>\n");
    more(s, gaps.len());
}

/// Every agent, or part, the review did not review, never as clean.
fn not_reviewed(s: &mut String, gaps: &[Gap]) {
    s.push_str("<h2>Not reviewed</h2>\n");
    if gaps.is_empty() {
        s.push_str("<p>Every dispatched agent has an accepted report or answer.</p>\n");
        return;
    }
    s.push_str("<ul>\n");
    for gap in gaps.iter().take(MAX_ROWS) {
        let _ = writeln!(
            s,
            "<li><code>{}</code> ({}): {}</li>",
            text(&gap.label()),
            text(&gap.kind),
            text(&gap.reason)
        );
    }
    s.push_str("</ul>\n");
    more(s, gaps.len());
}

#[cfg(test)]
mod tests;
