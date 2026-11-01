use super::*;

const KEY: &str = "APP_KEY";

/// The step that runs the review's agents.
const RUN: &str =
    "      - run: cargo run --locked --quiet --offline -p review-run -- --workdir .\n";

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
    assert_eq!(w.runs_after("CI"), Ok(()));
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
    assert_eq!(
        none.agents_unprivileged(KEY),
        ["no job runs the review's agents"]
    );
}

#[test]
fn a_harness_session_in_any_job_is_held_to_the_agents_rules() {
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
fn agents_run_by_wants_the_command_and_no_harness_session() {
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
        [
            "job review runs a harness session, anthropics/claude-code-action@, which would route the review"
        ]
    );
}

/// A step that installs the Claude Code command line, which starts an
/// agent from any later step of its job.
const CLI: &str = "      - uses: ./.github/actions/claude-code\n";

#[test]
fn runs_agents_sees_the_command_line_however_it_is_installed() {
    assert!(runs_agents(RUN));
    assert!(runs_agents(
        "      - uses: anthropics/claude-code-action@abc\n"
    ));
    assert!(runs_agents(CLI));
    assert!(runs_agents(
        "      - run: npm install --global @anthropic-ai/claude-code@2\n"
    ));
    assert!(!runs_agents("      - run: cargo run -p review-gate\n"));
}

#[test]
fn agents_run_by_refuses_the_command_line_outside_the_review_s_runners() {
    let cmd = "-p review-run -- --workdir .";
    let mut w = parse(EVALUATING_FLOW);
    w.jobs[0].text.push_str(CLI);
    w.jobs[1].text.push_str(CLI);
    assert!(
        w.agents_run_by(cmd).is_empty(),
        "review-run and review-eval"
    );

    w.jobs.push(Job {
        name: "route".into(),
        // Building review-run is not running it, and planning the
        // evaluation runs no case.
        text: format!(
            "    steps:\n{CLI}      - run: cargo build --locked --quiet -p review-run\n      - run: cargo run --locked --quiet -p review-eval -- plan\n      - run: claude -p review\n"
        ),
    });
    assert_eq!(
        w.agents_run_by(cmd),
        [
            "job route starts the Claude Code command line outside review-run and review-eval, which would route the review"
        ]
    );
}

#[test]
fn agents_run_by_refuses_the_command_line_started_beside_review_run() {
    let cmd = "-p review-run -- --workdir .";
    let mut w = parse(REVIEW_FLOW);
    w.jobs[0].text.push_str(CLI);
    w.jobs[0]
        .text
        .push_str("      - run: claude --permission-mode bypassPermissions -p review\n");

    assert_eq!(
        w.agents_run_by(cmd),
        [
            "job review starts the Claude Code command line itself, outside review-run and review-eval, which would route the review"
        ]
    );
}

#[test]
fn agents_run_by_refuses_the_command_line_in_a_job_that_installs_none() {
    let cmd = "-p review-run -- --workdir .";
    let mut w = parse(REVIEW_FLOW);
    w.jobs[1]
        .text
        .push_str("      - run: |\n          claude -p review\n");

    assert_eq!(
        w.agents_run_by(cmd),
        [
            "job post starts the Claude Code command line itself, outside review-run and review-eval, which would route the review"
        ]
    );
}

#[test]
fn starts_cli_reads_a_command_at_a_line_s_start_alone() {
    assert!(starts_cli("      - run: claude -p review"));
    assert!(starts_cli("          claude"));
    assert!(starts_cli("        run: claude --agent x"));
    assert!(!starts_cli("      # claude -p review"));
    assert!(!starts_cli("          echo claude -p review"));
    assert!(!starts_cli("          claude-code --version"));
    assert!(!starts_cli("      - uses: ./.github/actions/claude-code"));
}

#[test]
fn starts_cli_reads_a_command_started_through_another() {
    assert!(starts_cli(
        "          npx @anthropic-ai/claude-code -p review"
    ));
    assert!(starts_cli(
        "          npx -y @anthropic-ai/claude-code@2.1.0 -p review"
    ));
    assert!(starts_cli(
        "      - run: cd pr-head && claude --permission-mode x"
    ));
    assert!(starts_cli("          env X=1 claude -p review"));
    assert!(starts_cli("          HOME=\"$tmp\" exec claude -p review"));
    assert!(starts_cli(
        "          true; ./node_modules/.bin/claude -p review"
    ));
    assert!(!starts_cli("          cd pr-head && echo claude"));
    assert!(!starts_cli("          env X=1 cargo run -p review-run"));
}

#[test]
fn starts_cli_reads_every_wrapper_a_subshell_and_a_shell_s_command() {
    for line in [
        "          npx --yes claude -p review",
        "          command claude -p review",
        "          nohup claude -p review",
        "          time claude -p review",
        "          sudo -E claude -p review",
        "          timeout 30m claude -p review",
        "          timeout -k 5 30m claude -p review",
        "          bash -c \"claude -p review\"",
        "          sh -c 'cd pr-head; claude -p review'",
        "          (claude -p review)",
        "          out=$(claude -p review)",
        "          echo `claude -p review`",
    ] {
        assert!(starts_cli(line), "{line}");
    }
    assert!(!starts_cli("          timeout 30m cargo run -p review-run"));
    assert!(!starts_cli("          bash -c \"echo claude\""));
    assert!(!starts_cli("          out=$(git rev-parse HEAD)"));
}

#[test]
fn key_held_apart_refuses_the_command_line_in_the_key_s_job() {
    let mut w = parse(REVIEW_FLOW);
    w.jobs[1].text.push_str(CLI);

    assert_eq!(
        w.key_held_apart(KEY).unwrap_err(),
        "job post holds APP_KEY and runs the agents"
    );
}

#[test]
fn evaluate_job_runs_with_the_command_line_needs_not_itself() {
    let w = parse(&EVALUATING_FLOW.replace(
        "      - run: cargo run --locked --quiet -p review-eval -- run\n",
        &format!("{CLI}      - run: cargo run --locked --quiet -p review-eval -- run\n"),
    ));

    assert_eq!(
        w.evaluate_job_runs("cargo run --locked --quiet -p review-eval", KEY),
        Ok(Vec::new())
    );
}

#[test]
fn key_held_apart_wants_one_holder_that_runs_no_agent() {
    let mut w = parse(REVIEW_FLOW);
    assert_eq!(w.key_held_apart(KEY), Ok(()));

    w.jobs[1].text.push_str(RUN);
    assert_eq!(
        w.key_held_apart(KEY).unwrap_err(),
        "job post holds APP_KEY and runs the agents"
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
    assert_eq!(
        w.key_job_after_agents("OTHER").unwrap_err(),
        r#"0 jobs read "OTHER", want exactly one"#
    );
}

#[test]
fn key_job_runs_wants_the_command() {
    let w = parse(REVIEW_FLOW);

    assert_eq!(w.key_job_runs(KEY, "cargo run -p review-gate"), Ok(()));
    assert_eq!(
        w.key_job_runs(KEY, "cargo run -p other").unwrap_err(),
        r#"job post does not run "cargo run -p other""#
    );
    assert_eq!(
        w.key_job_runs("OTHER", "cargo run -p review-gate")
            .unwrap_err(),
        r#"0 jobs read "OTHER", want exactly one"#
    );
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
    assert_eq!(
        parse(REVIEW_FLOW)
            .key_job_posts_check("OTHER", "review-gate")
            .unwrap_err(),
        r#"0 jobs read "OTHER", want exactly one"#
    );
}

#[test]
fn grants_write_reads_quoted_values_and_write_all() {
    assert!(grants_write("      contents: \"write\"\n"));
    assert!(grants_write("      contents: 'write'\n"));
    assert!(grants_write("    permissions: write-all\n"));
    assert!(!grants_write("    permissions: read-all\n"));
}

/// The tool lists of review-run's agents, as the runner gives them:
/// reading tools alone for both kinds.
const READING_ONLY: [(&str, &str); 2] = [
    ("specialist", "Read,Glob,Grep"),
    ("generalist", "Read,Glob,Grep"),
];

#[test]
fn agents_write_nothing_refuses_every_tool_beyond_reading() {
    let with = |tools: &str| {
        REVIEW_FLOW.replace(
            RUN,
            &format!("{RUN}      - run: claude -p review --allowedTools {tools}\n"),
        )
    };

    assert!(
        parse(REVIEW_FLOW)
            .agents_write_nothing(&READING_ONLY)
            .is_empty()
    );
    assert!(
        parse(&with("\"Read(./pr-head/**),Grep,Glob\""))
            .agents_write_nothing(&READING_ONLY)
            .is_empty()
    );
    assert_eq!(
        parse(&with("Read,Write,Bash(ls:*)")).agents_write_nothing(&READING_ONLY),
        [
            "agent job review may use Write, want Read, Glob, Grep alone",
            "agent job review may use Bash(ls:*), want Read, Glob, Grep alone",
        ]
    );
    // A scoped writing rule is a writing tool all the same: the reports
    // are assembled from the run logs, not written by an agent.
    assert_eq!(
        parse(&with("\"Read,Edit(./review/reports/**)\"")).agents_write_nothing(&READING_ONLY),
        ["agent job review may use Edit(./review/reports/**), want Read, Glob, Grep alone"]
    );
}

#[test]
fn agents_write_nothing_holds_review_run_s_tools_to_reading() {
    let loose = [
        ("specialist", "Read,Glob,Grep,Edit(./review/reports/**)"),
        ("generalist", "Read,Bash"),
    ];

    assert_eq!(
        parse(REVIEW_FLOW).agents_write_nothing(&loose),
        [
            "review-run's specialist may use Edit(./review/reports/**), want Read, Glob, Grep alone",
            "review-run's generalist may use Bash, want Read, Glob, Grep alone",
        ]
    );
}

#[test]
fn review_run_s_tools_are_the_runner_s_reading_tools() {
    assert_eq!(RUN_TOOLS, READING_ONLY);
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

#[test]
fn agents_write_nothing_names_the_web_an_agent_and_a_server_tool() {
    let loose = [("specialist", "Read,Glob,Grep,WebFetch,Agent,mcp__x__y")];

    assert_eq!(
        parse(REVIEW_FLOW).agents_write_nothing(&loose),
        [
            "review-run's specialist may use WebFetch, want Read, Glob, Grep alone",
            "review-run's specialist may use Agent, want Read, Glob, Grep alone",
            "review-run's specialist may use mcp__x__y, want Read, Glob, Grep alone",
        ]
    );
}

/// An agent job's step that uploads `path` as an artifact.
fn upload(path: &str) -> String {
    format!(
        "      - uses: actions/upload-artifact@abc # v7\n        with:\n          name: x\n          path: {path}\n"
    )
}

#[test]
fn run_logs_stay_in_the_agent_job() {
    let mut w = parse(REVIEW_FLOW);
    assert!(w.run_logs_stay(RUN_LOGS).is_empty());

    w.jobs[0].text.push_str(&upload(
        "|\n            review/reports/\n            review/answers.json",
    ));
    assert!(
        w.run_logs_stay(RUN_LOGS).is_empty(),
        "the reports leave the job"
    );

    for path in [
        "review/logs/",
        "review/logs",
        "review/",
        "review",
        ".",
        "review/**",
        "review/**/*.jsonl",
        "**/*.jsonl",
        "./**",
        "review/log?/",
        "review/[l]ogs/",
        "|\n            review/\n            !review/**/*.jsonl",
        "|\n            review/reports/\n            review/logs/agent.jsonl",
    ] {
        let mut leaks = parse(REVIEW_FLOW);
        leaks.jobs[0].text.push_str(&upload(path));
        assert_eq!(
            leaks.run_logs_stay(RUN_LOGS),
            ["agent job review uploads review/logs, every tool result its agents read"],
            "{path}"
        );
    }

    let mut excluded = parse(REVIEW_FLOW);
    excluded.jobs[0].text.push_str(&upload(
        "|\n            review/\n            !review/logs/**",
    ));
    assert!(
        excluded.run_logs_stay(RUN_LOGS).is_empty(),
        "an excluded log stays"
    );

    let mut whole = parse(REVIEW_FLOW);
    whole.jobs[0]
        .text
        .push_str(&upload("|\n            review/\n            !review/"));
    assert!(
        whole.run_logs_stay(RUN_LOGS).is_empty(),
        "excluding what holds the logs keeps them"
    );

    let mut other = parse(REVIEW_FLOW);
    other.jobs[1].text.push_str(&upload("review/logs/"));
    assert!(
        other.run_logs_stay(RUN_LOGS).is_empty(),
        "a job that runs no agent has no log"
    );
}

#[test]
fn excluding_one_log_still_uploads_the_rest() {
    let mut one = parse(REVIEW_FLOW);
    one.jobs[0].text.push_str(&upload(
        "|\n            review/\n            !review/logs/a.jsonl",
    ));

    assert_eq!(
        one.run_logs_stay(RUN_LOGS),
        ["agent job review uploads review/logs, every tool result its agents read"]
    );
}

#[test]
fn upload_paths_reads_inline_and_block_paths() {
    assert_eq!(upload_paths(&upload("a/")), ["a/"]);
    assert_eq!(
        upload_paths(&upload("|\n            a/\n            !b\n")),
        ["a/", "!b"]
    );
    assert!(upload_paths("      - run: echo path: x\n").is_empty());
    assert!(upload_paths("      - uses: actions/upload-artifact@abc\n").is_empty());
}

#[test]
fn reads_of_the_process_files_and_outside_the_trees_must_be_refused() {
    use review_eval::runner::{DISALLOWED, SETTINGS};
    assert!(Workflow::reads_refused(DISALLOWED, SETTINGS).is_empty());

    assert_eq!(
        Workflow::reads_refused("Bash,Read(//proc/**), Read(//dev/**)", "{}"),
        [
            "review-run does not refuse its agents Read(//sys/**)",
            "review-run does not refuse its agents reads outside their trees",
        ]
    );
}

const TOKEN: &str = "MODEL_TOKEN";

/// An agent job that keeps the token from its agents: one step stages
/// it, the agents run on the staged credential, a scrub follows and
/// gates the upload.
const KEPT: &str = "jobs:
  review:
    steps:
      - name: Stage
        env:
          T: ${{ secrets.MODEL_TOKEN }}
        run: cargo run --locked --quiet --offline -p review-eval -- stage-credential --dir d
      - name: Review
        run: cargo run --locked --quiet --offline -p review-run -- --workdir . --credential d
      - name: Scrub
        id: scrub
        if: always()
        run: cargo run --locked --quiet --offline -p review-eval -- scrub --path review
      - name: Keep
        if: always() && steps.scrub.outputs.scrubbed == 'true'
        uses: actions/upload-artifact@abc
";

#[test]
fn a_job_that_stages_the_token_and_scrubs_before_it_uploads_keeps_it_from_its_agents() {
    assert!(parse(KEPT).token_kept_from_agents(TOKEN).is_empty());
    assert_eq!(
        parse(REVIEW_FLOW).token_kept_from_agents(KEY),
        [
            "agent job review stages no APP_KEY with \"-p review-eval -- stage-credential\"",
            "agent job review runs agents in step \"- run: cargo run --locked --quiet --offline -p review-run -- --workdir .\" without --credential",
        ]
    );
}

#[test]
fn the_evaluation_s_run_is_a_step_that_runs_agents_and_its_scrub_is_not() {
    assert!(runs_agent_step(
        "      - run: cargo run --locked --quiet -p review-eval -- run --out o\n"
    ));
    assert!(!runs_agent_step(
        "      - run: cargo run --locked --quiet -p review-eval -- scrub --path o\n"
    ));
    // An evaluation run after the scrub leaves the upload ungated.
    let late = KEPT.replace(
        "      - name: Keep\n",
        "      - name: Evaluate\n        run: cargo run --locked --quiet -p review-eval -- run --out o\n      - name: Keep\n",
    );
    assert_eq!(unscrubbed_uploads(&steps(&late)), ["Keep"]);
}

#[test]
fn a_scrub_gates_an_upload_in_steps_that_run_no_agent() {
    let steps = steps(&KEPT.replace(
        "      - name: Review\n        run: cargo run --locked --quiet --offline -p review-run -- --workdir . --credential d\n",
        "",
    ));

    assert_eq!(steps.len(), 3);
    assert!(unscrubbed_uploads(&steps).is_empty(), "{steps:?}");
}

#[test]
fn the_token_in_a_step_that_runs_agents_or_the_whole_job_is_named() {
    let in_review = KEPT.replace(
        "      - name: Review\n",
        "      - name: Review\n        env:\n          T: ${{ secrets.MODEL_TOKEN }}\n",
    );
    assert_eq!(
        parse(&in_review).token_kept_from_agents(TOKEN),
        ["agent job review hands MODEL_TOKEN to step \"Review\", which does more than stage it"]
    );
    let job_wide = KEPT.replace(
        "    steps:\n",
        "    env:\n      T: ${{ secrets.MODEL_TOKEN }}\n    steps:\n",
    );
    assert_eq!(
        parse(&job_wide).token_kept_from_agents(TOKEN),
        ["agent job review sets MODEL_TOKEN for every step"]
    );
    let staged_and_run = KEPT.replace(
        "stage-credential --dir d\n",
        "stage-credential --dir d && claude -p x\n",
    );
    assert_eq!(
        parse(&staged_and_run).token_kept_from_agents(TOKEN),
        [
            "agent job review hands MODEL_TOKEN to step \"Stage\", which does more than stage it",
            "agent job review runs agents in step \"Stage\" without --credential",
        ]
    );
}

#[test]
fn agents_without_the_staged_credential_or_a_job_that_stages_none_are_named() {
    let unstaged = KEPT.replace("${{ secrets.MODEL_TOKEN }}", "x");
    assert_eq!(
        parse(&unstaged).token_kept_from_agents(TOKEN),
        ["agent job review stages no MODEL_TOKEN with \"-p review-eval -- stage-credential\""]
    );
    let bare = KEPT.replace(" --credential d\n", "\n");
    assert_eq!(
        parse(&bare).token_kept_from_agents(TOKEN),
        ["agent job review runs agents in step \"Review\" without --credential"]
    );
}

#[test]
fn an_upload_a_failed_step_before_it_skips_is_named() {
    let skipped = KEPT.replace(
        "        if: always() && steps.scrub.outputs.scrubbed == 'true'\n",
        "        if: steps.scrub.outputs.scrubbed == 'true'\n",
    );

    assert_eq!(
        parse(&skipped).token_kept_from_agents(TOKEN),
        [
            "agent job review uploads in step \"Keep\" only when every step before it succeeded, so a failed run keeps nothing for its author: its if: needs always()"
        ]
    );
}

#[test]
fn an_upload_no_scrub_after_the_agents_gates_is_named() {
    let want = [
        "agent job review uploads in step \"Keep\" before a scrub of MODEL_TOKEN after its agents gates it",
    ];
    let ungated = KEPT.replace(
        "        if: always() && steps.scrub.outputs.scrubbed == 'true'\n",
        "        if: always()\n",
    );
    assert_eq!(parse(&ungated).token_kept_from_agents(TOKEN), want);
    let no_scrub = KEPT.replace("-- scrub --path", "-- plan --path");
    assert_eq!(parse(&no_scrub).token_kept_from_agents(TOKEN), want);
    let early = KEPT.replace(
        "      - name: Review\n        run: cargo run --locked --quiet --offline -p review-run -- --workdir . --credential d\n",
        "",
    ) + "      - name: Review\n        run: cargo run --locked --quiet --offline -p review-run -- --workdir . --credential d\n";
    assert_eq!(parse(&early).token_kept_from_agents(TOKEN), want);
    // An upload whose `if:` is its first line is gated by it all the
    // same, and named by that line when nothing gates it.
    let unnamed = KEPT.replace("      - name: Keep\n        if:", "      - if:");
    assert!(
        parse(&unnamed).token_kept_from_agents(TOKEN).is_empty(),
        "{:?}",
        parse(&unnamed).token_kept_from_agents(TOKEN)
    );
    assert_eq!(
        parse(&unnamed.replace("&& steps.scrub.outputs.scrubbed == 'true'", ""))
            .token_kept_from_agents(TOKEN),
        [
            "agent job review uploads in step \"- if: always()\" before a scrub of MODEL_TOKEN after its agents gates it"
        ]
    );
}

fn workflow_checkout(files: &[(&str, &str)]) -> (testkit::TempDir, Checkout) {
    let dir = testkit::TempDir::new().unwrap();
    let flows = dir.path().join(".github/workflows");
    std::fs::create_dir_all(&flows).unwrap();
    for (name, text) in files {
        std::fs::write(flows.join(name), text).unwrap();
    }
    let checkout = Checkout::new(dir.path());

    (dir, checkout)
}

const CLAUDE: &str = "CLAUDE_CODE_OAUTH_TOKEN";

const AGENT_FLOW: &str = "on:\n  workflow_dispatch:\njobs:\n  go:\n    steps:\n      - name: Go\n        run: cargo run -p review-run -- --workdir w --credential c\n";

#[test]
fn every_workflow_in_the_checkout_keeps_the_token_from_its_agents() {
    let checkout = Checkout::new(&testkit::repo_root());

    assert_eq!(
        every_workflow_keeps_the_token(&checkout, CLAUDE),
        Ok(Vec::new())
    );
}

#[test]
fn an_agent_job_in_any_workflow_is_held_to_the_token_rules_naming_its_file() {
    let (_dir, checkout) = workflow_checkout(&[
        (
            "a.yml",
            "on:\n  push:\njobs:\n  t:\n    steps:\n      - run: cargo test -p review-run\n",
        ),
        ("b.yml", AGENT_FLOW),
    ]);
    let problems = every_workflow_keeps_the_token(&checkout, CLAUDE).unwrap();

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].starts_with(".github/workflows/b.yml: agent job go "));
}

#[test]
fn a_workflow_whose_script_cannot_be_read_is_an_error_naming_it() {
    let (_dir, checkout) = workflow_checkout(&[(
        "c.yml",
        "on:\n  push:\njobs:\n  t:\n    steps:\n      - run: scripts/nosuch.sh arg\n",
    )]);
    let err = every_workflow_keeps_the_token(&checkout, CLAUDE).unwrap_err();

    assert!(err.contains("nosuch.sh"), "{err}");
}

#[test]
fn a_build_or_test_of_review_run_is_no_agent_job_and_its_binary_is() {
    assert!(!runs_agents("run: cargo test -p review-run -p srs\n"));
    assert!(!runs_agents("run: cargo build --locked -p review-run\n"));
    assert!(runs_agents(
        "run: \"$W/target/debug/review-run\" --workdir w\n"
    ));
    assert!(runs_agent_step(
        "run: \"$W/target/debug/review-run\" --workdir w\n"
    ));
}

#[test]
fn the_workflows_that_cannot_be_listed_or_read_are_an_error() {
    let dir = testkit::TempDir::new().unwrap();
    let err = every_workflow_keeps_the_token(&Checkout::new(dir.path()), CLAUDE).unwrap_err();
    assert!(err.starts_with("read .github/workflows: "), "{err}");

    let (_dir, checkout) = workflow_checkout(&[]);
    std::fs::create_dir(_dir.path().join(".github/workflows/bad.yml")).unwrap();
    let err = every_workflow_keeps_the_token(&checkout, CLAUDE).unwrap_err();
    assert!(err.contains("bad.yml"), "{err}");
}

#[test]
fn an_upload_of_a_path_the_gating_scrub_does_not_name_is_named() {
    let kept = |scrub: &str, up: &str| {
        let text = KEPT
            .replace("--path review\n", &format!("{scrub}\n"))
            .replace(
                "uses: actions/upload-artifact@abc\n",
                &format!(
                    "uses: actions/upload-artifact@abc\n        with:\n          path: |\n{up}"
                ),
            );
        unscrubbed_uploads(&steps(&text))
    };
    let up = "            review/reports/\n            review/report.html\n";
    assert!(kept("--path review", up).is_empty());
    assert!(kept("--path review/reports --path review/report.html", up).is_empty());
    assert_eq!(kept("--path review/reports", up), ["Keep"]);
    // A staged work directory is the same tree under another prefix.
    let up = "            ${{ runner.temp }}/work/review/report.html\n";
    assert!(kept("--path \"$work/review/report.html\"", up).is_empty());
    assert_eq!(kept("--path \"$work/review/logs\"", up), ["Keep"]);
}

#[test]
fn an_exclusion_in_an_upload_needs_no_scrub_of_its_own() {
    let text = KEPT.replace(
        "uses: actions/upload-artifact@abc\n",
        "uses: actions/upload-artifact@abc\n        with:\n          path: |\n            review/\n            !review/runs/**/log.jsonl\n",
    );
    assert!(unscrubbed_uploads(&steps(&text)).is_empty());
}

#[test]
fn every_spelling_of_the_review_run_start_runs_agents() {
    for run in [
        "cargo run -p review-run -- --workdir .",
        "cargo run --package review-run -- --workdir .",
        "cargo run --bin review-run -- --workdir .",
        "target/release/review-run --workdir .",
    ] {
        let step = format!("      - run: {run}\n");
        assert!(runs_agent_step(&step), "{run}");
        assert!(runs_agents(&step), "{run}");
    }
    assert!(!runs_agent_step(
        "      - run: cargo build --package review-run\n"
    ));
}

#[test]
fn only_a_leading_work_directory_is_cut_from_a_scrubbed_or_uploaded_path() {
    for (given, want) in [
        ("\"$work/review/logs/\"", "review/logs"),
        ("${{ runner.temp }}/work/class.txt", "class.txt"),
        ("review/work/report.html", "review/work/report.html"),
        ("review/$work/report.html", "review/$work/report.html"),
        ("a/${{ x }}/work/report.html", "a/${{ x }}/work/report.html"),
    ] {
        assert_eq!(tree_path(given), want);
    }
}
