use super::*;

const KEY: &str = "APP_KEY";

/// The step that runs the review's agents.
const RUN: &str = "      - run: cargo run --locked --quiet --offline -p review-run -- --workdir .\n";

/// The smallest workflow the ENG-28 steps accept.
const REVIEW_FLOW: &str = "name: Review
on: # a comment
  workflow_run:
    workflows: [CI]
    types: [completed]
  issue_comment:
    types: [created]

jobs:
  review:
    permissions:
      contents: read # never write
    steps:
      - run: cargo run --locked --quiet --offline -p review-run -- --workdir .
  # a comment between jobs
  post:
    needs: [review]
    steps:
      - env:
          K: ${{ secrets.APP_KEY }}
        run: cargo run -p review-gate
";

fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|r| r.iter().map(|c| (*c).to_owned()).collect())
        .collect()
}

const HEADER: &[&str] = &["verdict", "finding", "head", "CI", "other check", "review"];

#[test]
fn parse_splits_triggers_and_jobs() {
    let w = parse(&format!("stray: before\n  sub: key\n{REVIEW_FLOW}"));

    assert_eq!(w.triggers, ["workflow_run", "issue_comment"]);
    assert_eq!(w.jobs.len(), 2);
    assert_eq!(w.jobs[0].name, "review");
    assert!(
        !w.jobs[0].text.contains("never write"),
        "comments are dropped"
    );
    assert!(!w.jobs[0].text.contains("between jobs"));
    assert!(w.jobs[1].text.contains("cargo run -p review-gate"));
}

#[test]
fn parse_skips_lines_before_the_first_job() {
    let w = parse("jobs:\n    orphan: line\n  a:\n    run: x\n");

    assert_eq!(
        w.jobs,
        [Job {
            name: "a".into(),
            text: "    run: x\n".into()
        }]
    );
}

#[test]
fn keys_have_their_shapes() {
    assert_eq!(top_key("on:\n"), Some("on"));
    assert_eq!(top_key("jobs:\n"), Some("jobs"));
    assert_eq!(top_key("name: Review\n"), Some("other"));
    assert_eq!(top_key("  on:\n"), None);
    assert_eq!(top_key(":\n"), None);
    assert_eq!(top_key("no colon\n"), None);
    assert_eq!(sub_key("  review:\n"), Some("review"));
    assert_eq!(sub_key("    deeper:\n"), None);
    assert_eq!(sub_key("  :\n"), None);
    assert_eq!(sub_key("  - item\n"), None);
}

#[test]
fn uncomment_drops_comments() {
    assert_eq!(uncomment("  # whole line\n"), "");
    assert_eq!(uncomment("on: # trailing\n"), "on:\n");
    assert_eq!(uncomment("a#b\n"), "a#b\n");
}

#[test]
fn grants_write_and_has_word_match_whole_words() {
    assert!(grants_write("      pull-requests: write\n"));
    assert!(grants_write("contents:write"));
    assert!(!grants_write("contents: read\n"));
    assert!(!grants_write("contents: writer\n"));
    assert!(!grants_write("write: x\n"));
    assert!(has_word("needs: [review]", "review"));
    assert!(!has_word("needs: [reviewer]", "review"));
    assert!(!has_word("needs: [pre_review]", "review"));
}

#[test]
fn runs_after_wants_the_named_workflow_and_comments_only() {
    let mut w = parse(REVIEW_FLOW);
    assert!(w.runs_after("CI").is_ok());
    assert_eq!(
        w.runs_after("Build").unwrap_err(),
        r#"the workflow_run trigger does not say "workflows: [Build]""#
    );

    let mut done = w.clone();
    done.on = done.on.replace("types: [completed]", "types: [requested]");
    assert_eq!(
        done.runs_after("CI").unwrap_err(),
        r#"the workflow_run trigger does not say "types: [completed]""#
    );

    let mut edited = w.clone();
    edited.on = edited.on.replace("types: [created]", "types: [edited]");
    assert_eq!(
        edited.runs_after("CI").unwrap_err(),
        r#"the issue_comment trigger does not say "types: [created]""#
    );

    w.triggers.push("workflow_dispatch".into());
    assert_eq!(
        w.runs_after("CI").unwrap_err(),
        "the review workflow triggers on [workflow_run issue_comment workflow_dispatch], \
         want workflow_run and issue_comment"
    );

    w.triggers = vec!["workflow_run".into()];
    assert_eq!(
        w.runs_after("CI").unwrap_err(),
        "the review workflow triggers on [workflow_run], want workflow_run and issue_comment"
    );
}

#[test]
fn agents_unprivileged_names_each_privilege() {
    let mut w = parse(REVIEW_FLOW);
    assert!(w.agents_unprivileged(KEY).is_empty());

    w.jobs[0]
        .text
        .push_str("      pull-requests: write\n        K: ${{ secrets.APP_KEY }}\n");
    assert_eq!(
        w.agents_unprivileged(KEY),
        [
            "agent job review is granted a write permission",
            "agent job review reads APP_KEY"
        ]
    );

    let none = parse("on:\n  workflow_run:\njobs:\n  a:\n    steps: []\n");
    assert_eq!(none.agents_unprivileged(KEY), ["no job runs the review's agents"]);
}

#[test]
fn an_agent_session_in_any_job_is_held_to_the_agents_rules() {
    let mut w = parse(REVIEW_FLOW);
    w.jobs.push(Job {
        name: "route".into(),
        text: "    permissions:\n      contents: write\n    steps:\n      - uses: anthropics/claude-code-action@abc\n"
            .into(),
    });

    assert_eq!(
        w.agents_unprivileged(KEY),
        ["agent job route is granted a write permission"]
    );
}

#[test]
fn agents_run_by_wants_the_command_and_no_agent_session() {
    let cmd = "-p review-run -- --workdir .";
    let mut w = parse(REVIEW_FLOW);
    assert!(w.agents_run_by(cmd).is_empty());

    assert_eq!(
        w.agents_run_by("-p review-run -- --workdir other"),
        [r#"no job runs the review's agents with "-p review-run -- --workdir other""#]
    );

    w.jobs[0]
        .text
        .push_str("      - uses: anthropics/claude-code-action@abc\n");
    assert_eq!(
        w.agents_run_by(cmd),
        ["job review runs an agent session, anthropics/claude-code-action@, which would route the review"]
    );
}

#[test]
fn key_held_apart_wants_one_holder_that_runs_no_agent() {
    let mut w = parse(REVIEW_FLOW);
    assert!(w.key_held_apart(KEY).is_ok());

    w.jobs[1].text.push_str(RUN);
    assert_eq!(
        w.key_held_apart(KEY).unwrap_err(),
        "job post holds APP_KEY and runs the agent"
    );
    assert_eq!(
        w.key_held_apart("OTHER").unwrap_err(),
        r#"0 jobs read "OTHER", want exactly one"#
    );
}

#[test]
fn key_job_after_agents_wants_the_agents_needed_and_no_override() {
    let mut w = parse(REVIEW_FLOW);
    assert!(w.key_job_after_agents(KEY).unwrap().is_empty());

    w.jobs[1].text =
        "    needs: [reviewer]\n    if: always()\n    K: ${{ secrets.APP_KEY }}\n".into();
    assert_eq!(
        w.key_job_after_agents(KEY).unwrap(),
        [
            "job post does not need agent job review",
            "job post runs on always(), even after a failed review"
        ]
    );
    assert!(w.key_job_after_agents("OTHER").is_err());
}

#[test]
fn key_job_runs_wants_the_command() {
    let w = parse(REVIEW_FLOW);

    assert!(w.key_job_runs(KEY, "cargo run -p review-gate").is_ok());
    assert_eq!(
        w.key_job_runs(KEY, "cargo run -p other").unwrap_err(),
        r#"job post does not run "cargo run -p other""#
    );
    assert!(w.key_job_runs("OTHER", "cargo run -p review-gate").is_err());
}

#[test]
fn key_job_posts_check_wants_the_permission_the_post_and_a_failing_default() {
    let posts = format!(
        "{REVIEW_FLOW}      - with:
          permission-checks: write
        run: |
          conclusion=failure
          gh api --method POST \"repos/$REPO/check-runs\" -f name=review-gate
"
    );
    assert_eq!(
        parse(&posts).key_job_posts_check(KEY, "review-gate"),
        Ok(vec![])
    );

    assert_eq!(
        parse(REVIEW_FLOW).key_job_posts_check(KEY, "review-gate"),
        Ok(vec![
            "job post does not mint permission-checks: write".to_owned(),
            "job post does not post to repos/$REPO/check-runs".to_owned(),
            "job post does not name the check name=review-gate".to_owned(),
            "job post does not fail the check by default: conclusion=failure".to_owned(),
        ])
    );
    assert!(
        parse(REVIEW_FLOW)
            .key_job_posts_check("OTHER", "review-gate")
            .is_err()
    );
}

#[test]
fn gate_decides_compares_every_row() {
    let right = table_rows(&rows(&[
        HEADER,
        &["approve", "none", "current", "passed", "passed", "APPROVE"],
    ]))
    .unwrap();
    assert!(gate_decides(&right).is_empty());

    let wrong = table_rows(&rows(&[
        HEADER,
        &[
            "approve",
            "none",
            "current",
            "passed",
            "passed",
            "REQUEST_CHANGES",
        ],
    ]))
    .unwrap();
    let problems = gate_decides(&wrong);
    assert!(problems[0].starts_with("row 1 "), "{problems:?}");
}

#[test]
fn table_rows_keys_cells_by_header() {
    let got = table_rows(&rows(&[&["a", "b"], &["1", "2"]])).unwrap();

    assert_eq!(
        got,
        [BTreeMap::from([
            ("a".to_owned(), "1".to_owned()),
            ("b".to_owned(), "2".to_owned())
        ])]
    );
    assert_eq!(
        table_rows(&rows(&[HEADER])).unwrap_err(),
        "the table has no rows under its header"
    );
    assert_eq!(
        table_rows(&[]).unwrap_err(),
        "the table has no rows under its header"
    );
}

#[test]
fn gate_outcome_names_what_the_gate_does() {
    let row = |verdict: &str, finding: &str, head: &str, ci: &str, other: &str| {
        [
            ("verdict", verdict),
            ("finding", finding),
            ("head", head),
            ("CI", ci),
            ("other check", other),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    assert_eq!(
        gate_outcome(&row("approve", "none", "current", "passed", "passed")),
        "APPROVE"
    );
    assert_eq!(
        gate_outcome(&row("approve", "nit", "current", "passed", "passed")),
        "APPROVE"
    );
    assert_eq!(
        gate_outcome(&row("approve", "blocking", "current", "passed", "passed")),
        "REQUEST_CHANGES"
    );
    assert_eq!(
        gate_outcome(&row("approve", "none", "current", "passed", "failed")),
        "REQUEST_CHANGES"
    );
    assert_eq!(
        gate_outcome(&row("approve", "none", "moved", "passed", "passed")),
        "nothing"
    );
    assert_eq!(
        gate_outcome(&row("approve", "none", "current", "failed", "passed")),
        "an error"
    );
    assert_eq!(
        gate_outcome(&row("malformed", "none", "current", "passed", "passed")),
        "an error"
    );
    assert_eq!(
        gate_outcome(&row("missing", "none", "current", "passed", "passed")),
        "an error"
    );
}

#[test]
fn gate_outcome_hands_the_gate_a_deterministic_finding() {
    let row = |check: &str| {
        [
            ("verdict", "approve"),
            ("finding", "none"),
            ("check finding", check),
            ("head", "current"),
            ("CI", "passed"),
            ("other check", "passed"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    assert_eq!(gate_outcome(&row("none")), "APPROVE");
    assert_eq!(gate_outcome(&row("nit")), "APPROVE");
    assert_eq!(gate_outcome(&row("missing")), "an error");
    assert_eq!(gate_outcome(&row("blocking")), "REQUEST_CHANGES");
    assert_eq!(gate_outcome(&row("malformed")), "an error");
}

#[test]
fn gate_outcome_hands_the_gate_the_agents_consulted() {
    let row = |consulted: &str| {
        [
            ("verdict", "approve"),
            ("finding", "none"),
            ("consulted", consulted),
            ("head", "current"),
            ("CI", "passed"),
            ("other check", "passed"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    assert_eq!(gate_outcome(&row("all")), "APPROVE");
    assert_eq!(gate_outcome(&row("not all")), "an error");
    assert_eq!(gate_outcome(&row("malformed")), "an error");
    assert_eq!(gate_outcome(&row("al")), UNKNOWN_CELL);
}

#[test]
fn gate_outcome_hands_the_gate_the_dispatch() {
    let row = |dispatch: &str| {
        [
            ("verdict", "approve"),
            ("finding", "none"),
            ("dispatch", dispatch),
            ("head", "current"),
            ("CI", "passed"),
            ("other check", "passed"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    assert_eq!(gate_outcome(&row("given")), "APPROVE");
    assert_eq!(gate_outcome(&row("missing")), "an error");
    assert_eq!(gate_outcome(&row("malformed")), "an error");
    assert_eq!(gate_outcome(&row("givne")), UNKNOWN_CELL);
}

#[test]
fn gate_outcome_hands_the_gate_the_specialist_s_report() {
    let row = |cell: &str, value: &str| {
        [
            ("verdict", "approve"),
            ("finding", "none"),
            ("head", "current"),
            ("CI", "passed"),
            ("other check", "passed"),
            (cell, value),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    for (value, want) in [
        ("given", "APPROVE"),
        ("missing", "an error"),
        ("empty", "an error"),
        ("no outcome", "an error"),
        ("other outcome", "an error"),
        ("givne", UNKNOWN_CELL),
    ] {
        assert_eq!(gate_outcome(&row("report", value)), want, "{value}");
    }
    for (value, want) in [
        ("none", "APPROVE"),
        ("failed", "APPROVE"),
        ("faild", UNKNOWN_CELL),
    ] {
        assert_eq!(gate_outcome(&row("own check", value)), want, "{value}");
    }
}

#[test]
fn own_check_is_a_failed_review_gate_run_from_the_reviewer_app() {
    assert_eq!(own_check("none"), Some(String::new()));
    assert_eq!(own_check("x"), None);
    let line = own_check("failed").unwrap();
    let run: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(run["name"], review_gate::OWN_CHECK);
    assert_eq!(run["conclusion"], "failure");
    assert_eq!(run["app"], review_gate::REVIEWER_APP_ID);
}

#[test]
fn grants_write_reads_quoted_values_and_write_all() {
    assert!(grants_write("      contents: \"write\"\n"));
    assert!(grants_write("      contents: 'write'\n"));
    assert!(grants_write("    permissions: write-all\n"));
    assert!(!grants_write("    permissions: read-all\n"));
}

/// The tool lists of review-run's agents, as the runner gives them.
const SCOPED: [(&str, &str); 2] = [
    ("specialist", "Read,Glob,Grep,Edit(./review/reports/**)"),
    ("generalist", "Read,Glob,Grep"),
];

#[test]
fn agents_write_only_wants_writes_scoped_to_the_reports() {
    let with = |tools: &str| {
        REVIEW_FLOW.replace(
            RUN,
            &format!("{RUN}      - run: claude -p review --allowedTools {tools}\n"),
        )
    };

    assert!(
        parse(REVIEW_FLOW)
            .agents_write_only("review/reports", &SCOPED)
            .is_empty()
    );
    assert!(
        parse(&with("\"Read,Grep,Edit(./review/reports/**)\""))
            .agents_write_only("review/reports", &SCOPED)
            .is_empty()
    );
    assert_eq!(
        parse(&with("Read,Write,Bash(ls:*)")).agents_write_only("review/reports", &SCOPED),
        [
            "agent job review may use Write, want Edit(./review/reports/**) alone",
            "agent job review may use Bash(ls:*), want Edit(./review/reports/**) alone",
        ]
    );
    assert_eq!(
        parse(&with("\"Read,Write(./review/reports/**)\""))
            .agents_write_only("review/reports", &SCOPED),
        [
            "agent job review may use Write(./review/reports/**), want Edit(./review/reports/**) alone"
        ]
    );
}

#[test]
fn agents_write_only_holds_review_run_s_tools_to_the_reports() {
    let loose = [("specialist", "Read,Write"), ("generalist", "Read,Bash")];

    assert_eq!(
        parse(REVIEW_FLOW).agents_write_only("review/reports", &loose),
        [
            "review-run's specialist may use Write, want Edit(./review/reports/**) alone",
            "review-run's generalist may use Bash, want Edit(./review/reports/**) alone",
        ]
    );
}

#[test]
fn review_run_s_tools_are_the_runner_s() {
    assert_eq!(RUN_TOOLS, SCOPED);
}

/// The review workflow with the evaluate job ENG-32 asks for.
const EVALUATING_FLOW: &str = "name: Review
on:
  workflow_run:
    workflows: [CI]
    types: [completed]
  issue_comment:
    types: [created]

jobs:
  review:
    permissions:
      contents: read
    steps:
      - run: cargo run --locked --quiet --offline -p review-run -- --workdir .
  evaluate:
    needs: review
    permissions:
      contents: read
    steps:
      - run: cargo run --locked --quiet -p review-eval -- run
  post:
    needs: [review, evaluate]
    steps:
      - env:
          K: ${{ secrets.APP_KEY }}
        run: cargo run -p review-gate
";

#[test]
fn evaluate_job_runs_accepts_the_job_that_evaluates_between_the_agent_and_the_key() {
    let w = parse(EVALUATING_FLOW);

    assert_eq!(
        w.evaluate_job_runs("cargo run --locked --quiet -p review-eval", KEY),
        Ok(Vec::new())
    );
}

#[test]
fn evaluate_job_runs_names_each_problem() {
    let w = parse(
        &EVALUATING_FLOW
            .replace(
                "    needs: review\n    permissions:\n      contents: read\n",
                "    permissions:\n      contents: write\n",
            )
            .replace(
                "      - run: cargo run --locked --quiet -p review-eval -- run\n",
                "      - run: echo skipped ${{ secrets.APP_KEY }}\n",
            )
            .replace("    needs: [review, evaluate]\n", "    needs: [review]\n"),
    );

    let got = w
        .evaluate_job_runs("cargo run --locked --quiet -p review-eval", KEY)
        .unwrap();

    assert_eq!(
        got,
        vec![
            "job evaluate does not run \"cargo run --locked --quiet -p review-eval\"",
            "job evaluate is granted a write permission",
            "job evaluate reads APP_KEY",
            "job evaluate does not need agent job review",
            "job post does not need job evaluate",
        ]
    );
}

#[test]
fn evaluate_job_runs_fails_without_the_job_or_the_key_s_job() {
    assert_eq!(
        parse(REVIEW_FLOW).evaluate_job_runs("x", KEY).unwrap_err(),
        "no evaluate job in the review workflow"
    );
    let no_key = parse(&EVALUATING_FLOW.replace("secrets.APP_KEY", "secrets.OTHER"));
    assert_eq!(
        no_key.evaluate_job_runs("x", KEY).unwrap_err(),
        "0 jobs read \"APP_KEY\", want exactly one"
    );
}
