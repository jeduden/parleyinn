//! The `review-run` executable as the review workflow runs it: a work
//! directory holding a recorded review bundle, the repository's own
//! agent definitions, and a stand-in `claude` on the path that writes
//! each specialist's report and every run's stream-json log. The review
//! gate then reads what the run wrote, as the post job does. Each run
//! has a temporary `HOME` and `PARINN_HOME`, and the variables a Claude
//! Code session sets for its children, which the run must clear.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use testkit::TempDir;

/// A `claude` that answers as the agent it runs as: a specialist
/// writes its report with one nit and answers its outcome line and
/// path; a generalist keeps the nit and closes with a sentence. Started
/// as a session's child it refuses, and the agent `SILENT_AGENT` names
/// never answers.
const CLAUDE: &str = r#"#!/bin/sh
if [ -n "$CLAUDE_CODE_CHILD_SESSION" ] || [ -n "$CLAUDE_AUTO_BACKGROUND_TASKS" ]; then
  echo "started as a session's child" >&2
  exit 3
fi
prompt="$2"
agent=""
prev=""
for a in "$@"; do
  if [ "$prev" = "--agent" ]; then agent="$a"; fi
  prev="$a"
done
if [ "$agent" = "$SILENT_AGENT" ]; then exec sleep 30; fi
printf '{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m-%s","model":"claude-haiku-5-5","usage":{"input_tokens":3,"cache_read_input_tokens":7,"output_tokens":2},"content":[]}}\n' "$agent"
case "$prompt" in
  *"a specialist"*)
    printf 'src/a.rs:3 — nit — a long line — ENG-02\n\noutcome: nits — %s saw one nit.\n' "$agent" > "review/reports/$agent.md"
    printf '{"type":"result","subtype":"success","is_error":false,"result":"outcome: nits — %s saw one nit.\\nreview/reports/%s.md","total_cost_usd":0.01,"num_turns":2}\n' "$agent" "$agent"
    ;;
  *)
    printf '{"type":"result","subtype":"success","is_error":false,"result":"src/a.rs:3 — nit — a long line — lang-rust\\n%s finds its scope sound.","total_cost_usd":0.05,"num_turns":4}\n' "$agent"
    ;;
esac
"#;

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
/// path, as a Claude Code session's child would.
fn review_run(home: &Path, dir: &Path, silent: &str, args: &[&str]) -> Output {
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
        .env("CLAUDE_CODE_CHILD_SESSION", "1")
        .env("CLAUDE_AUTO_BACKGROUND_TASKS", "1")
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

#[test]
fn the_gate_approves_what_the_run_wrote() {
    let home = TempDir::new().unwrap();
    let dir = home.path().join("work");
    workdir(&dir);

    let out = review_run(home.path(), &dir, "", &[]);

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
    let out = review_run(home.path(), &dir, "lang-rust", &["--deadline", "2"]);

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
fn a_work_directory_without_a_dispatch_cannot_run_and_a_wrong_line_is_usage() {
    let home = TempDir::new().unwrap();

    let out = review_run(home.path(), home.path(), "", &[]);

    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("review-run: read "));

    let out = review_run(home.path(), home.path(), "", &["--parallel", "none"]);

    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage: review-run --workdir DIR"));
}
