//! The `review-run` executable as the review workflow runs it: a work
//! directory holding a recorded review bundle, the repository's own
//! agent definitions, and a stand-in `claude` on the path that writes
//! every run's stream-json log, a specialist's findings stated in its
//! text. The run assembles each specialist's report from its log, and
//! the review gate then reads what the run wrote, as the post job does.
//! Each run has a temporary `HOME` and `PARINN_HOME`, and the variables
//! a Claude Code harness session sets for its children, which the run
//! must clear.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use testkit::TempDir;

/// A `claude` that answers as the agent it runs as: a specialist states
/// one linked finding in its text and answers its outcome line; a
/// generalist keeps it as a nit and closes with a sentence. Started as a
/// harness session's child it refuses; the agent `SILENT_AGENT` names
/// never answers, and the one `CUT_AGENT` names states its finding and
/// ends at its turn budget without an answer, or only its part
/// `CUT_PART` does. A specialist run on a part of its diff also states
/// a finding of that part's own. The agent `REFUSE_AGENT` names also
/// states a revision in no form, until its prompt tells it of its
/// previous attempt, or always when `REFUSE_ALWAYS` is set.
const CLAUDE: &str = r#"#!/bin/sh
if [ -n "$CLAUDE_CODE_CHILD_SESSION" ] || [ -n "$CLAUDE_AUTO_BACKGROUND_TASKS" ]; then
  echo "started as a harness session's child" >&2
  exit 3
fi
if [ -n "$CLAUDE_CODE_OAUTH_TOKEN" ]; then
  echo "started with the Claude token in its environment" >&2
  exit 4
fi
prompt="$2"
agent=""
prev=""
for a in "$@"; do
  if [ "$prev" = "--agent" ]; then agent="$a"; fi
  prev="$a"
done
if [ "$agent" = "$SILENT_AGENT" ]; then exec sleep 30; fi
if [ -n "$LEAK_AGENT" ] && [ "$agent" = "$LEAK_AGENT" ]; then
  token=$(sed -n 's/.*"accessToken":"\([^"]*\)".*/\1/p' "$CLAUDE_CONFIG_DIR/.credentials.json")
  printf '{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m-%s","model":"claude-haiku-5-5","usage":{"input_tokens":3,"output_tokens":2},"content":[{"type":"text","text":"my environment holds %s"}]}}\n' "$agent" "$token"
  printf '{"type":"result","subtype":"success","is_error":false,"result":"outcome: none — %s","total_cost_usd":0.01,"num_turns":2}\n' "$token"
  echo "stderr too: $token" >&2
  exit 0
fi
part=""
own=""
case "$prompt" in
  *"you are part "*)
    part=$(printf '%s' "$prompt" | sed -n 's/.*you are part \([0-9]*\) of.*/\1/p')
    own="\\nsrc/part$part.rs:1 — if not addressed: part $part breaks — I9"
    ;;
esac
if [ "$agent" = "$REFUSE_AGENT" ]; then
  case "$prompt" in
    *"previous attempt"*) if [ -n "$REFUSE_ALWAYS" ]; then own="$own$BAD"; fi ;;
    *) own="$own$BAD" ;;
  esac
fi
case "$prompt" in
  *"a specialist"*)
    printf '{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m-%s","model":"claude-haiku-5-5","usage":{"input_tokens":3,"cache_read_input_tokens":7,"output_tokens":2},"content":[{"type":"text","text":"I read the diff.\\nsrc/a.rs:3 — if not addressed: a long line hides a bug — I9%s"}]}}\n' "$agent" "$own"
    if [ "$agent" = "$CUT_AGENT" ] && { [ -z "$CUT_PART" ] || [ "$part" = "$CUT_PART" ]; }; then
      printf '{"type":"result","subtype":"error_max_turns","is_error":true,"result":"","total_cost_usd":0.03,"num_turns":20}\n'
    else
      printf '{"type":"result","subtype":"success","is_error":false,"result":"outcome: findings — %s saw one finding.","total_cost_usd":0.01,"num_turns":2}\n' "$agent"
    fi
    ;;
  *)
    printf '{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m-%s","model":"claude-haiku-5-5","usage":{"input_tokens":3,"cache_read_input_tokens":7,"output_tokens":2},"content":[]}}\n' "$agent"
    printf '{"type":"result","subtype":"success","is_error":false,"result":"src/a.rs:3 — nit — a long line — lang-rust\\n%s finds its scope sound.","total_cost_usd":0.05,"num_turns":4}\n' "$agent"
    ;;
esac
"#;

/// The revision in no form the refused agent states, as the stand-in
/// writes it into the log's JSON.
const BAD: &str = r"\nrevised: `src/a.rs:3 — old` → `src/a.rs:3 — new`";

const DISPATCH: &str = r#"{
  "review-code": ["lang-rust", "test-engineer-unit"],
  "review-security": ["persona-security-officer"]
}
"#;

/// A work directory as the review job prepares it: the bundle's
/// output, the pull request's tree and diff, and main's definitions of
/// the dispatched agents; and a directory holding the stand-in.
fn workdir(root: &Path) {
    let write = |rel: &str, body: &str| {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    write("review/dispatch.json", DISPATCH);
    write("review/checked.json", "[]\n");
    write(
        "review/bundle.md",
        "# Review bundle\n\n## Review class\n\nClass: **standard**.\n",
    );
    for name in [
        "lang-rust",
        "test-engineer-unit",
        "persona-security-officer",
    ] {
        write(&format!("review/diffs/{name}.diff"), "+fn a() {}\n");
    }
    write(
        "class.txt",
        "class=standard\ntouched=1\nfindings=0\nconsults=5\n",
    );
    write("pr.diff", "diff --git a/src/a.rs b/src/a.rs\n+fn a() {}\n");
    write("pr-head/src/a.rs", "fn a() {}\n");
    let agents = testkit::repo_root().join(".claude/agents");
    for name in [
        "lang-rust",
        "test-engineer-unit",
        "persona-security-officer",
        "review-code",
        "review-security",
    ] {
        let file = format!("{name}.md");
        write(
            &format!(".claude/agents/{file}"),
            &fs::read_to_string(agents.join(&file)).unwrap(),
        );
    }
}

/// Runs the executable on `dir` with `args`, the stand-in first on the
/// path, as a Claude Code harness session's child would; `silent` names
/// the agent that never answers and `cut` the one its turn budget cuts
/// off, `name:i` only its part `i`.
fn review_run(home: &Path, dir: &Path, silent: &str, cut: &str, args: &[&str]) -> Output {
    review_run_refusing(home, dir, silent, cut, "", args)
}

/// [`review_run`], with `refuse` naming the agent whose report is
/// refused on its first attempt, `name:always` on both.
fn review_run_refusing(
    home: &Path,
    dir: &Path,
    silent: &str,
    cut: &str,
    refuse: &str,
    args: &[&str],
) -> Output {
    review_run_holding(home, dir, (silent, cut, refuse), &[], args)
}

/// [`review_run_refusing`], with `env` set for the executable too.
fn review_run_holding(
    home: &Path,
    dir: &Path,
    (silent, cut, refuse): (&str, &str, &str),
    env: &[(&str, &str)],
    args: &[&str],
) -> Output {
    let (cut, cut_part) = cut.split_once(':').unwrap_or((cut, ""));
    let (refuse, always) = refuse.split_once(':').unwrap_or((refuse, ""));
    let bin = home.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let claude = bin.join("claude");
    fs::write(&claude, CLAUDE).unwrap();
    fs::set_permissions(&claude, fs::Permissions::from_mode(0o755)).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_review-run"));
    command
        .env_clear()
        .env("HOME", home)
        .env("PARINN_HOME", home)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .env("SILENT_AGENT", silent)
        .env("CUT_AGENT", cut)
        .env("CUT_PART", cut_part)
        .env("REFUSE_AGENT", refuse)
        .env("REFUSE_ALWAYS", always)
        .env("BAD", BAD)
        .env("CLAUDE_CODE_CHILD_SESSION", "1")
        .env("CLAUDE_AUTO_BACKGROUND_TASKS", "1")
        .envs(env.iter().copied())
        .arg("--workdir")
        .arg(dir)
        .args(args);
    // Coverage of the executable needs its profile path, the one
    // variable the run keeps.
    if let Some(profile) = std::env::var_os("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", profile);
    }

    command.output().unwrap()
}

/// The review gate on what the run wrote, as the post job calls it,
/// with CI green on the reviewed head: its exit code, output and
/// errors. It runs in this process, not as the `review-gate`
/// executable, which this package cannot build; that executable's own
/// e2e, `tooling/review-gate/tests/e2e_review_gate.rs`, covers its
/// command line and exit codes.
fn gate(home: &Path, dir: &Path) -> (u8, String, String) {
    let runs = home.join("check-runs.jsonl");
    fs::write(
        &runs,
        "{\"id\":1,\"name\":\"CI\",\"status\":\"completed\",\"conclusion\":\"success\",\"started_at\":\"2026-10-30T00:00:00Z\",\"app\":1}\n",
    )
    .unwrap();
    let at = |rel: &str| dir.join(rel).display().to_string();
    let args = vec![
        "--outcome".to_owned(),
        at("review/outcome.json"),
        "--check-runs".to_owned(),
        runs.display().to_string(),
        "--reviewed".to_owned(),
        "abc123".to_owned(),
        "--head".to_owned(),
        "abc123".to_owned(),
        "--checked".to_owned(),
        at("review/checked.json"),
        "--dispatch".to_owned(),
        at("review/dispatch.json"),
        "--consulted".to_owned(),
        at("review/consulted.json"),
        "--reports".to_owned(),
        at("review/reports"),
        "--answers".to_owned(),
        at("review/answers.json"),
    ];
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = review_gate::cli::run(args, &mut out, &mut err, &|p| fs::read_to_string(p));

    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// A stand-in for the Claude token; never a real one.
const TOKEN: &str = "dummy-access-token-0123456789-e2e";

/// Every file under `dir`, read.
fn every_file(dir: &Path) -> String {
    let mut all = String::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            all.push_str(&every_file(&path));
        } else {
            all.push_str(&String::from_utf8_lossy(&fs::read(&path).unwrap()));
        }
    }
    all
}

#[test]
fn an_agent_that_leaks_the_staged_token_fails_the_review_closed_with_every_trace_scrubbed() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("work");
    workdir(&dir);
    let cred = home.path().join("cred");
    review_eval::credential::stage(&cred, TOKEN).unwrap();
    let cred_arg = cred.display().to_string();

    // The process holds the token as the workflow's old step gave it;
    // no agent it starts may.
    let clean = review_run_holding(
        home.path(),
        &dir,
        ("", "", ""),
        &[("CLAUDE_CODE_OAUTH_TOKEN", TOKEN)],
        &["--credential", &cred_arg],
    );
    let stderr = String::from_utf8_lossy(&clean.stderr);
    assert_eq!(clean.status.code(), Some(0), "{stderr}");
    // A stand-in that found the token in its environment answers
    // nothing, so a review that approves gave it to none.
    assert!(
        String::from_utf8_lossy(&clean.stdout).starts_with("verdict=approve\n"),
        "{stderr}"
    );

    let out = review_run_holding(
        home.path(),
        &dir,
        ("", "", ""),
        &[
            ("CLAUDE_CODE_OAUTH_TOKEN", TOKEN),
            ("LEAK_AGENT", "lang-rust"),
        ],
        &["--credential", &cred_arg],
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(
        stdout.starts_with("failed=an agent's output contained the Claude token: "),
        "{stdout}"
    );
    assert!(
        !stdout.contains(TOKEN) && !stderr.contains(TOKEN),
        "{stderr}"
    );
    let all = every_file(&dir.join("review"));
    assert!(!all.contains(TOKEN), "a file under review/ holds the token");
    assert!(all.contains("[REDACTED token]"));
    let outcome = fs::read_to_string(dir.join("review/outcome.json")).unwrap();
    assert!(
        outcome.contains("\"verdict\": \"request_changes\""),
        "{outcome}"
    );
    assert!(
        outcome.contains("an agent's output contained the Claude token: "),
        "{outcome}"
    );
}

#[test]
fn the_gate_approves_what_the_run_wrote() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("work");
    workdir(&dir);

    let out = review_run(home.path(), &dir, "", "", &[]);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "verdict=approve\ndispatched=5\nconsulted=5\nfindings=1\ncost=0.1300\n"
    );
    assert_eq!(stderr.lines().count(), 5, "{stderr}");
    for name in ["lang-rust", "review-security"] {
        let log = fs::read_to_string(dir.join(format!("review/logs/{name}.jsonl"))).unwrap();
        assert!(log.contains("\"type\":\"result\""), "{log}");
    }
    // The report the run assembled from the specialist's log: the
    // finding it stated, then the outcome line it answered, no prose.
    assert_eq!(
        fs::read_to_string(dir.join("review/reports/lang-rust.md")).unwrap(),
        "src/a.rs:3 — if not addressed: a long line hides a bug — I9\n\
         outcome: findings — lang-rust saw one finding.\n"
    );
    let usage: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("review/usage.json")).unwrap()).unwrap();
    assert_eq!(usage.as_array().unwrap().len(), 5);

    let (code, review, err) = gate(home.path(), &dir);

    assert_eq!(code, review_gate::cli::EXIT_OK, "{err}");
    let review: serde_json::Value = serde_json::from_str(&review).unwrap();
    assert_eq!(review["event"], "APPROVE");
    assert_eq!(review["commit_id"], "abc123");
    let body = review["body"].as_str().unwrap();
    assert!(body.contains("5 of 5 dispatched agents answered"), "{body}");
    assert!(
        body.contains("- nit `src/a.rs:3`: a long line — lang-rust"),
        "{body}"
    );
}

#[test]
fn a_specialist_past_its_deadline_is_decided_and_the_gate_posts_nothing() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("work");
    workdir(&dir);

    // The deadline binds every agent, not only the silent one: two
    // seconds leave the stand-in's shell room to start on a loaded host.
    let out = review_run(home.path(), &dir, "lang-rust", "", &["--deadline", "2"]);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("verdict=request_changes\n"),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "review-run: lang-rust (specialist): did not answer: its run ended with deadline"
        ),
        "{stderr}"
    );
    let outcome = fs::read_to_string(dir.join("review/outcome.json")).unwrap();
    let outcome = review_gate::parse_outcome(&outcome).unwrap();
    assert!(outcome.findings.iter().any(|f| f.path == "review/"
        && f.severity == "blocking"
        && f.body.starts_with("lang-rust did not answer")));

    let (code, review, err) = gate(home.path(), &dir);

    assert_eq!(code, review_gate::cli::EXIT_FAILURE);
    assert_eq!(review, "");
    // Another stand-in that misses the deadline on a loaded host joins
    // the list, so only lang-rust's place in it is asserted.
    assert!(
        err.starts_with("review-gate: review: dispatched but not consulted: ")
            && err.contains("lang-rust"),
        "{err}"
    );
}

#[test]
fn a_specialist_cut_off_at_its_turn_budget_blocks_the_review_but_keeps_what_it_stated() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("work");
    workdir(&dir);

    let out = review_run(home.path(), &dir, "", "lang-rust", &[]);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("verdict=request_changes\n"),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "review-run: lang-rust (specialist): did not answer: its run ended with error_max_turns; running it once more"
        ),
        "{stderr}"
    );
    let outcome = fs::read_to_string(dir.join("review/outcome.json")).unwrap();
    let outcome = review_gate::parse_outcome(&outcome).unwrap();
    assert!(outcome.findings.iter().any(|f| f.path == "review/"
        && f.severity == "blocking"
        && f.body == "lang-rust did not answer (its run ended with error_max_turns), so the review is incomplete"));
    // The finding it stated before the cut is kept for the author and
    // the generalists, under a line that says the report is partial.
    assert_eq!(
        fs::read_to_string(dir.join("review/reports/lang-rust.md")).unwrap(),
        "incomplete: its run ended with error_max_turns\n\
         src/a.rs:3 — if not addressed: a long line hides a bug — I9\n"
    );
    let consulted = fs::read_to_string(dir.join("review/consulted.json")).unwrap();
    assert!(!consulted.contains("lang-rust"), "{consulted}");

    let (code, review, err) = gate(home.path(), &dir);

    assert_eq!(code, review_gate::cli::EXIT_FAILURE);
    assert_eq!(review, "");
    assert_eq!(
        err,
        "review-gate: review: dispatched but not consulted: lang-rust\n"
    );
}

#[test]
fn a_work_directory_without_a_dispatch_cannot_run_and_a_wrong_line_is_usage() {
    let home = TempDir::new().unwrap();

    let out = review_run(home.path(), home.path(), "", "", &[]);

    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("review-run: read "));

    let out = review_run(home.path(), home.path(), "", "", &["--parallel", "none"]);

    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage: review-run --workdir DIR"));
}

/// Writes test-engineer-unit's diff past the split limit: three files
/// of 100,000 bytes each, of which no two fit one part, so it runs in
/// three parts.
fn big_diff(dir: &Path) {
    let section = |path: &str| {
        let head = format!("diff --git a/{path} b/{path}\n+++ b/{path}\n");
        let line = format!("+{}\n", "x".repeat(99));
        format!("{head}{}", line.repeat((100_000 - head.len()) / line.len()))
    };
    let diff = ["src/x3.rs", "src/x1.rs", "src/x2.rs"]
        .map(section)
        .concat();
    fs::write(dir.join("review/diffs/test-engineer-unit.diff"), diff).unwrap();
}

/// The usage entry of `agent` that the run wrote.
fn usage_of(dir: &Path, agent: &str) -> serde_json::Value {
    let usage: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("review/usage.json")).unwrap()).unwrap();
    usage
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["agent"] == agent)
        .unwrap()
        .clone()
}

#[test]
fn a_specialist_whose_diff_is_past_the_limit_runs_in_parts_and_the_gate_reads_the_merge() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("work");
    workdir(&dir);
    big_diff(&dir);

    let out = review_run(home.path(), &dir, "", "", &[]);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("verdict=approve\ndispatched=5\n"),
        "{stderr}"
    );
    for i in 1..=3 {
        let part =
            fs::read_to_string(dir.join(format!("review/diffs/test-engineer-unit.{i}.diff")))
                .unwrap();
        assert!(
            part.starts_with(&format!("diff --git a/src/x{i}.rs b/src/x{i}.rs\n")),
            "part {i}"
        );
        assert!(
            dir.join(format!("review/logs/test-engineer-unit.{i}.jsonl"))
                .is_file()
        );
    }
    assert!(
        stderr.contains("review-run: test-engineer-unit (specialist): answered (0.0300 USD, 6 turns) in 3 parts, 3 attempts"),
        "{stderr}"
    );
    // The finding every part stated once, each part's own, then one
    // outcome line saying each part's.
    assert_eq!(
        fs::read_to_string(dir.join("review/reports/test-engineer-unit.md")).unwrap(),
        "src/a.rs:3 — if not addressed: a long line hides a bug — I9\n\
         src/part1.rs:1 — if not addressed: part 1 breaks — I9\n\
         src/part2.rs:1 — if not addressed: part 2 breaks — I9\n\
         src/part3.rs:1 — if not addressed: part 3 breaks — I9\n\
         outcome: findings — in 3 parts: part 1, findings — test-engineer-unit saw one \
         finding.; part 2, findings — test-engineer-unit saw one finding.; part 3, findings \
         — test-engineer-unit saw one finding.\n"
    );
    let unit = usage_of(&dir, "test-engineer-unit");
    assert_eq!(
        (unit["parts"].as_u64(), unit["attempts"].as_u64()),
        (Some(3), Some(3))
    );
    assert_eq!(usage_of(&dir, "lang-rust")["parts"], 1);

    let (code, review, err) = gate(home.path(), &dir);

    assert_eq!(code, review_gate::cli::EXIT_OK, "{err}");
    let review: serde_json::Value = serde_json::from_str(&review).unwrap();
    assert_eq!(review["event"], "APPROVE");
}

#[test]
fn a_part_cut_off_at_its_turn_budget_blocks_the_review_naming_it_and_keeps_every_part_s_findings() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("work");
    workdir(&dir);
    big_diff(&dir);

    let out = review_run(home.path(), &dir, "", "test-engineer-unit:2", &[]);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("verdict=request_changes\n"),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "review-run: test-engineer-unit part 2 of 3 (specialist): did not answer: its run \
             ended with error_max_turns; running it once more"
        ),
        "{stderr}"
    );
    let outcome = fs::read_to_string(dir.join("review/outcome.json")).unwrap();
    let outcome = review_gate::parse_outcome(&outcome).unwrap();
    assert!(
        outcome.findings.iter().any(|f| f.path == "review/"
            && f.severity == "blocking"
            && f.body
                == "test-engineer-unit did not answer (part 2 of 3: its run ended with \
                    error_max_turns), so the review is incomplete"),
        "{:?}",
        outcome.findings
    );
    assert_eq!(
        fs::read_to_string(dir.join("review/reports/test-engineer-unit.md")).unwrap(),
        "incomplete: part 2 of 3: its run ended with error_max_turns\n\
         src/a.rs:3 — if not addressed: a long line hides a bug — I9\n\
         src/part1.rs:1 — if not addressed: part 1 breaks — I9\n\
         src/part2.rs:1 — if not addressed: part 2 breaks — I9\n\
         src/part3.rs:1 — if not addressed: part 3 breaks — I9\n"
    );
    let unit = usage_of(&dir, "test-engineer-unit");
    assert_eq!(
        (unit["parts"].as_u64(), unit["attempts"].as_u64()),
        (Some(3), Some(4))
    );
    let consulted = fs::read_to_string(dir.join("review/consulted.json")).unwrap();
    assert!(!consulted.contains("test-engineer-unit"), "{consulted}");

    let (code, review, err) = gate(home.path(), &dir);

    assert_eq!(code, review_gate::cli::EXIT_FAILURE);
    assert_eq!(review, "");
    assert_eq!(
        err,
        "review-gate: review: dispatched but not consulted: test-engineer-unit\n"
    );
}

#[test]
fn a_refused_report_runs_once_more_told_why_and_the_gate_approves_the_second() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("work");
    workdir(&dir);

    let out = review_run_refusing(home.path(), &dir, "", "", "lang-rust", &[]);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("verdict=approve\n"),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "review-run: lang-rust (specialist) has a report whose line 2 is neither a linked \
             finding nor a requirement request: `revised: `src/a.rs:3 — old` → `src/a.rs:3 — \
             new``; running it once more"
        ),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "lang-rust (specialist): answered (0.0200 USD, 4 turns) on its second attempt"
        ),
        "{stderr}"
    );
    // The second run is judged alone: its report is its own.
    assert_eq!(
        fs::read_to_string(dir.join("review/reports/lang-rust.md")).unwrap(),
        "src/a.rs:3 — if not addressed: a long line hides a bug — I9\n\
         outcome: findings — lang-rust saw one finding.\n"
    );
    assert!(dir.join("review/logs/lang-rust.first.jsonl").is_file());
    assert_eq!(usage_of(&dir, "lang-rust")["attempts"].as_u64(), Some(2));

    let (code, review, err) = gate(home.path(), &dir);

    assert_eq!(code, review_gate::cli::EXIT_OK, "{err}");
    let review: serde_json::Value = serde_json::from_str(&review).unwrap();
    assert_eq!(review["event"], "APPROVE");
}

#[test]
fn a_report_refused_on_both_attempts_blocks_the_review_and_the_gate_posts_nothing() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("work");
    workdir(&dir);

    let out = review_run_refusing(home.path(), &dir, "", "", "lang-rust:always", &[]);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("verdict=request_changes\n"),
        "{stderr}"
    );
    let outcome = fs::read_to_string(dir.join("review/outcome.json")).unwrap();
    let outcome = review_gate::parse_outcome(&outcome).unwrap();
    assert!(
        outcome
            .findings
            .iter()
            .any(|f| f.path == "review/reports/lang-rust.md"
                && f.severity == "blocking"
                && f.body
                    == "lang-rust has a report whose line 2 is neither a linked finding nor a \
                    requirement request"),
        "{:?}",
        outcome.findings
    );
    assert_eq!(usage_of(&dir, "lang-rust")["attempts"].as_u64(), Some(2));

    let (code, review, err) = gate(home.path(), &dir);

    assert_eq!(code, review_gate::cli::EXIT_FAILURE);
    assert_eq!(review, "");
    assert!(err.contains("lang-rust"), "{err}");
}

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn a_resumed_run_reruns_only_the_specialist_cut_off_and_its_generalist_and_the_gate_approves() {
    let home = TempDir::new().unwrap();
    let first = home.path().join("first");
    workdir(&first);
    let out = review_run(home.path(), &first, "", "lang-rust", &["--head-sha", HEAD]);
    assert_eq!(out.status.code(), Some(0));
    let gaps: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(first.join("review/gaps.json")).unwrap()).unwrap();
    assert_eq!(
        gaps,
        serde_json::json!([{"name": "lang-rust", "kind": "specialist",
            "reason": "incomplete: its run ended with error_max_turns"}])
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.ends_with("review-run: not reviewed: lang-rust\n"),
        "{stderr}"
    );
    let earlier = first.join("review");
    let dir = home.path().join("work");
    workdir(&dir);

    let gaps = earlier.join("gaps.json");
    let out = review_run(
        home.path(),
        &dir,
        "",
        "",
        &[
            "--head-sha",
            HEAD,
            "--only",
            gaps.to_str().unwrap(),
            "--from",
            earlier.to_str().unwrap(),
        ],
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("verdict=approve\n"),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "review-run: resumed from {}: reused 3 agents, re-ran 2 (lang-rust, review-code)",
            earlier.display()
        )),
        "{stderr}"
    );
    assert_eq!(usage_of(&dir, "persona-security-officer")["reused"], true);
    assert_eq!(usage_of(&dir, "review-security")["cost_usd"], 0.0);
    assert!(usage_of(&dir, "lang-rust").get("reused").is_none());
    let id: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("review/run-id.json")).unwrap()).unwrap();
    assert_eq!(id["resumed_from"], earlier.to_str().unwrap());

    let (code, review, err) = gate(home.path(), &dir);

    assert_eq!(code, review_gate::cli::EXIT_OK, "{err}");
    let review: serde_json::Value = serde_json::from_str(&review).unwrap();
    assert_eq!(review["event"], "APPROVE");
    let body = review["body"].as_str().unwrap();
    assert!(body.contains("5 of 5 dispatched agents answered"), "{body}");
}

#[test]
fn a_resumed_run_on_another_head_reuses_nothing() {
    let home = TempDir::new().unwrap();
    let first = home.path().join("first");
    workdir(&first);
    let out = review_run(home.path(), &first, "", "lang-rust", &["--head-sha", HEAD]);
    assert_eq!(out.status.code(), Some(0));
    let earlier = first.join("review");
    let dir = home.path().join("work");
    workdir(&dir);

    let gaps = earlier.join("gaps.json");
    let out = review_run(
        home.path(),
        &dir,
        "",
        "",
        &[
            "--head-sha",
            &"f".repeat(40),
            "--only",
            gaps.to_str().unwrap(),
            "--from",
            earlier.to_str().unwrap(),
        ],
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(
        stderr.contains("review-run: nothing reused: the head changed from"),
        "{stderr}"
    );
    assert!(stderr.contains("reused 0 agents, re-ran 5"), "{stderr}");
}
