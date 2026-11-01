//! The review gate half of ENG-28: the decision table the gate is held to.

use std::collections::BTreeMap;

/// Keys each body row of `rows` by the header row.
///
/// # Errors
///
/// Fails when the table has no rows under its header.
pub fn table_rows(rows: &[Vec<String>]) -> Result<Vec<BTreeMap<String, String>>, String> {
    let [header, body @ ..] = rows else {
        return Err("the table has no rows under its header".to_owned());
    };
    if body.is_empty() {
        return Err("the table has no rows under its header".to_owned());
    }

    Ok(body
        .iter()
        .map(|row| header.iter().cloned().zip(row.iter().cloned()).collect())
        .collect())
}

/// Runs every row through the review gate and compares the outcome
/// with the row's review column.
#[must_use]
pub fn gate_decides(rows: &[BTreeMap<String, String>]) -> Vec<String> {
    rows.iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let got = gate_outcome(row);
            (row.get("review").map(String::as_str) != Some(got))
                .then(|| format!("row {} {row:?}: the review gate gives {got}", i + 1))
        })
        .collect()
}

/// The review gate's command line, as the review workflow runs it, on
/// the files [`gate_outcome`] writes.
fn gate_args(reviewed: &str, head: &str) -> Vec<String> {
    [
        "--outcome",
        "outcome.json",
        "--check-runs",
        "check-runs.jsonl",
        "--reviewed",
        reviewed,
        "--head",
        head,
        "--checked",
        "checked.json",
        "--dispatch",
        "dispatch.json",
        "--consulted",
        "consulted.json",
        "--reports",
        "reports",
        "--answers",
        "answers.json",
    ]
    .map(str::to_owned)
    .to_vec()
}

/// The report review-run assembled from the row's one specialist's run
/// log, and the answers file holding its final answer: a `specialist`
/// cell of `reported`, or none, hands the gate a report with a linked
/// finding that ends with the outcome line the specialist answered,
/// `requested` one with a requirement request instead, `no report` no
/// report, `empty report` a blank one, `incomplete` the report of a run
/// cut off before its answer, headed so and ending with no outcome, `no
/// outcome answered` an answer with no outcome line, `other outcome
/// answered` an answer whose outcome is not the report's, `malformed
/// finding` a logged finding that gives a severity where its
/// consequence and requirements belong, `revised nothing` the report
/// assembled from a log whose one finding line revises a finding the
/// specialist never stated, so only the revision standing as the finding
/// it states keeps the report's outcome of findings, `withdrew nothing`
/// the one assembled from a log that states a linked finding and
/// withdraws one it never stated, `none beside a finding` an
/// outcome of none after a finding, `findings with none` an outcome of
/// findings after none, and `unknown id` a finding citing ENG-999, an id
/// of the right shape that no requirement has, which the gate, holding
/// no tree, takes; `None` for any other value.
fn specialist(row: &BTreeMap<String, String>) -> Option<(Option<String>, &'static str)> {
    const FINDINGS: &str = r#"{"lang-rust":"outcome: findings — fine","review-code":"r"}"#;
    const NONE: &str = r#"{"lang-rust":"outcome: none — fine","review-code":"r"}"#;
    const NO_OUTCOME: &str = r#"{"lang-rust":"I am done.","review-code":"r"}"#;
    let (report, answers) = match row.get("specialist").map_or("reported", String::as_str) {
        "reported" => (
            Some("a.rs:1 — if not addressed: x breaks — I2\noutcome: findings — fine\n"),
            FINDINGS,
        ),
        "requested" => (
            Some("a.rs:1 — request: add a rule on x — none says so\noutcome: findings — fine\n"),
            FINDINGS,
        ),
        "no report" => (None, FINDINGS),
        "empty report" => (Some("  \n"), FINDINGS),
        "incomplete" => (
            Some(
                "incomplete: its run ended with error_max_turns\na.rs:1 — if not addressed: x breaks — I2\n",
            ),
            NO_OUTCOME,
        ),
        "no outcome answered" => (
            Some("a.rs:1 — if not addressed: x breaks — I2\noutcome: findings — fine\n"),
            NO_OUTCOME,
        ),
        "other outcome answered" => (
            Some("a.rs:1 — if not addressed: x breaks — I2\noutcome: findings — fine\n"),
            NONE,
        ),
        "malformed finding" => (
            Some("a.rs:1 — nit — x\noutcome: findings — fine\n"),
            FINDINGS,
        ),
        "revised nothing" => {
            return Some((
                Some(assembled("revised: b.rs:2 — request: add y — why")),
                FINDINGS,
            ));
        }
        "withdrew nothing" => {
            return Some((
                Some(assembled(
                    "a.rs:1 — if not addressed: x breaks — I2\nwithdrawn: b.rs:2 — I checked it and it holds",
                )),
                FINDINGS,
            ));
        }
        "none beside a finding" => (
            Some("a.rs:1 — if not addressed: x breaks — I2\noutcome: none — fine\n"),
            NONE,
        ),
        "findings with none" => (Some("outcome: findings — fine\n"), FINDINGS),
        "unknown id" => (
            Some("a.rs:1 — if not addressed: x breaks — ENG-999\noutcome: findings — fine\n"),
            FINDINGS,
        ),
        _ => return None,
    };

    Some((report.map(str::to_owned), answers))
}

/// The report review-run assembles from a run log that states `lines`
/// and answers an outcome of findings.
fn assembled(lines: &str) -> String {
    let texts = [lines.to_owned(), "outcome: findings — fine".to_owned()];
    review_gate::report::assemble(&texts, "outcome: findings — fine", None)
}

/// The dispatch a table row hands the gate by default: one generalist
/// with one specialist.
const DISPATCH: &str = r#"{"review-code":["lang-rust"]}"#;

/// What [`gate_outcome`] names for a row whose `dispatch`, `consulted`,
/// `specialist` or `own check` cell holds a value it does not define, so a
/// mistyped row fails rather than reads as the default.
pub const UNKNOWN_CELL: &str = "a cell value the table does not define";

/// The dispatch file a row hands the gate: [`DISPATCH`] for `given` or
/// no `dispatch` column, a file that is no map for `malformed`, and no
/// file (`Some(None)`) for `missing`; `None` for any other value.
fn dispatch(row: &BTreeMap<String, String>) -> Option<Option<&'static str>> {
    match row.get("dispatch").map_or("given", String::as_str) {
        "given" => Some(Some(DISPATCH)),
        "malformed" => Some(Some("[]")),
        "missing" => Some(None),
        _ => None,
    }
}

/// The agents a row says the review consulted: both of
/// [`DISPATCH`] for `all` or no `consulted` column, the generalist alone
/// for `not all`, and a file that is no list for `malformed`; `None` for
/// any other value.
fn consulted(row: &BTreeMap<String, String>) -> Option<&'static str> {
    match row.get("consulted").map_or("all", String::as_str) {
        "all" => Some(r#"["review-code","lang-rust"]"#),
        "not all" => Some(r#"["review-code"]"#),
        "malformed" => Some("nope"),
        _ => None,
    }
}

/// The check run line for the gate's own `review-gate` check a row
/// names: a failed one from the reviewer app for `failed`, none for
/// `none` or no `own check` column; `None` for any other value.
fn own_check(cell: &str) -> Option<String> {
    match cell {
        "failed" => Some(format!(
            "{{\"name\":{:?},\"status\":\"completed\",\"conclusion\":\"failure\",\"app\":{}}}\n",
            review_gate::OWN_CHECK,
            review_gate::REVIEWER_APP_ID
        )),
        "none" | "" => Some(String::new()),
        _ => None,
    }
}

/// Runs the review gate's command line, as the review workflow does,
/// on the files one table row describes, and names what it does: the
/// review event it prints, "nothing" for a head that moved, or "an
/// error" when it exits unsuccessfully. A `missing` verdict leaves the
/// outcome file out. The gate always gets `--checked`: a `check
/// finding` of `none`, or a row without the column, hands it an empty
/// list, `nit` or `blocking` one deterministic finding of that
/// severity, `malformed` a file that is no list, and `missing` no file.
/// It always gets `--dispatch`, `--consulted`, `--reports` and
/// `--answers` (see [`dispatch`], [`consulted`] and [`specialist`]), and a
/// row whose cell for any of them, or for `own check`, it does not
/// define gives [`UNKNOWN_CELL`].
#[must_use]
pub fn gate_outcome(row: &BTreeMap<String, String>) -> &'static str {
    const REVIEWED: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";
    const MOVED: &str = "490efa734617788d1dc1f831b26d0e23888e90fa";
    let cell = |name: &str| row.get(name).map(String::as_str).unwrap_or_default();
    let (Some(dispatched), Some(consulted), Some((report, answers)), Some(own)) = (
        dispatch(row),
        consulted(row),
        specialist(row),
        own_check(cell("own check")),
    ) else {
        return UNKNOWN_CELL;
    };
    let findings = match cell("finding") {
        "none" => "[]".to_owned(),
        severity => format!(r#"[{{"path":"a.rs","line":1,"severity":{severity:?},"body":"b"}}]"#),
    };
    let conclusion = |cell: &str| {
        if cell == "passed" {
            "success"
        } else {
            "failure"
        }
    };
    let check = match row.get("check finding").map_or("none", String::as_str) {
        "none" => Some("[]".to_owned()),
        "missing" => None,
        "malformed" => Some("nope".to_owned()),
        severity => Some(format!(
            r#"[{{"path":"features/x.feature","line":0,"severity":{severity:?},"body":"c"}}]"#
        )),
    };
    let mut files = BTreeMap::from([(
        "check-runs.jsonl",
        format!(
            "{{\"name\":\"CI\",\"status\":\"completed\",\"conclusion\":{:?}}}\n\
             {{\"name\":\"lint\",\"status\":\"completed\",\"conclusion\":{:?}}}\n{}",
            conclusion(cell("CI")),
            conclusion(cell("other check")),
            own
        ),
    )]);
    if cell("verdict") != "missing" {
        files.insert(
            "outcome.json",
            format!(
                r#"{{"verdict":{:?},"summary":"s","findings":{findings}}}"#,
                cell("verdict")
            ),
        );
    }
    let head = if cell("head") == "moved" {
        MOVED
    } else {
        REVIEWED
    };
    if let Some(body) = dispatched {
        files.insert("dispatch.json", body.to_owned());
    }
    files.insert("consulted.json", consulted.to_owned());
    if let Some(body) = report {
        files.insert("reports/lang-rust.md", body);
    }
    files.insert("answers.json", answers.to_owned());
    if let Some(body) = check {
        files.insert("checked.json", body);
    }
    let read = |path: &str| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = review_gate::cli::run(gate_args(REVIEWED, head), &mut out, &mut err, &read);
    match serde_json::from_slice::<review_gate::Review>(&out) {
        _ if code != review_gate::cli::EXIT_OK => "an error",
        _ if out.is_empty() => "nothing",
        Ok(review) if review.event == review_gate::APPROVE => review_gate::APPROVE,
        _ => review_gate::REQUEST_CHANGES,
    }
}

#[cfg(test)]
mod tests;
