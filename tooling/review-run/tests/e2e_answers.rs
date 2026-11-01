//! The `review-run` executable with an answer store, as the review
//! workflow and a laptop run it: a stand-in `claude` on the path that
//! answers as the agent it runs as and notes each run it starts. A
//! second review of the same inputs starts no agent; a refused report is
//! never stored; `review-run answers drop` takes one answer out, so only
//! that agent runs again; a store read alone gains nothing. Each run has
//! a temporary `HOME` and `PARINN_HOME`.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use testkit::TempDir;

/// A `claude` that prints its version, notes in `$HOME/started` the
/// agent each run starts as, and answers as that agent: a specialist
/// states one linked finding and its outcome line, or, as the agent
/// `REFUSE_AGENT` names, a line in no form beside it; a generalist keeps
/// a nit and closes with a sentence.
const CLAUDE: &str = r#"#!/bin/sh
if [ "$1" = "--version" ]; then echo "2.1.0 (Claude Code)"; exit 0; fi
prompt="$2"
agent=""
prev=""
for a in "$@"; do
  if [ "$prev" = "--agent" ]; then agent="$a"; fi
  prev="$a"
done
echo "$agent" >> "$HOME/started"
bad=""
if [ "$agent" = "$REFUSE_AGENT" ]; then bad="\\nsrc/a.rs:9 — a line in no form"; fi
case "$prompt" in
  *"a specialist"*)
    printf '{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m-%s","model":"claude-haiku-5-5","usage":{"input_tokens":3,"output_tokens":2},"content":[{"type":"text","text":"src/a.rs:3 — if not addressed: a long line hides a bug — I9%s"}]}}\n' "$agent" "$bad"
    printf '{"type":"result","subtype":"success","is_error":false,"result":"outcome: findings — %s saw one finding.","total_cost_usd":0.01,"num_turns":2}\n' "$agent"
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

const AGENTS: [&str; 5] = [
    "lang-rust",
    "test-engineer-unit",
    "persona-security-officer",
    "review-code",
    "review-security",
];

/// A work directory as the review job prepares it, with the checkout's
/// definitions of the dispatched agents.
fn workdir(root: &Path) {
    let write = |rel: &str, body: &str| {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    write("review/dispatch.json", DISPATCH);
    write("review/checked.json", "[]\n");
    write("review/bundle.md", "# Review bundle\n");
    for name in &AGENTS[..3] {
        write(&format!("review/diffs/{name}.diff"), "+fn a() {}\n");
    }
    write("class.txt", "class=standard\ntouched=1\n");
    write("pr.diff", "diff --git a/src/a.rs b/src/a.rs\n+fn a() {}\n");
    write("pr-head/src/a.rs", "fn a() {}\n");
    let agents = testkit::repo_root().join(".claude/agents");
    for name in AGENTS {
        let file = format!("{name}.md");
        write(
            &format!(".claude/agents/{file}"),
            &fs::read_to_string(agents.join(&file)).unwrap(),
        );
    }
}

/// Runs the executable with `args`, the stand-in first on the path,
/// `refuse` naming the agent whose report is refused; returns its output
/// and the agents it started, sorted.
fn review_run(home: &Path, refuse: &str, args: &[&str]) -> (Output, Vec<String>) {
    let bin = home.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let claude = bin.join("claude");
    fs::write(&claude, CLAUDE).unwrap();
    fs::set_permissions(&claude, fs::Permissions::from_mode(0o755)).unwrap();
    let started = home.join("started");
    let _ = fs::remove_file(&started);
    let mut command = Command::new(env!("CARGO_BIN_EXE_review-run"));
    command
        .env_clear()
        .env("HOME", home)
        .env("PARINN_HOME", home)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .env("REFUSE_AGENT", refuse)
        .args(args);
    if let Some(profile) = std::env::var_os("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", profile);
    }
    let out = command.output().unwrap();
    let mut agents: Vec<String> = fs::read_to_string(&started)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect();
    agents.sort();
    (out, agents)
}

/// Reviews a fresh work directory under `home`, named `name`, with the
/// store `store`, `extra` after.
fn reviewed(
    home: &Path,
    name: &str,
    store: &Path,
    refuse: &str,
    extra: &[&str],
) -> (Output, Vec<String>) {
    let dir = home.join(name);
    workdir(&dir);
    let store = format!("--answers={}", store.display());
    let mut args = vec!["--workdir", dir.to_str().unwrap(), store.as_str()];
    args.extend_from_slice(extra);
    review_run(home, refuse, &args)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_second_review_of_the_same_inputs_starts_no_agent() {
    let home = TempDir::new().unwrap();
    let store = home.path().join("answers");

    let (out, started) = reviewed(home.path(), "first", &store, "", &[]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(started.len(), 5);
    assert_eq!(fs::read_dir(&store).unwrap().count(), 5);

    let (out, started) = reviewed(home.path(), "second", &store, "", &["--answers-read-only"]);

    assert!(started.is_empty(), "{started:?}");
    let printed = stdout(&out);
    assert!(printed.starts_with("verdict=approve\n"), "{printed}");
    assert!(printed.contains("\ncost=0.0000\nreused=5\n"), "{printed}");
    let first = fs::read_to_string(home.path().join("first/review/reports/lang-rust.md")).unwrap();
    let second =
        fs::read_to_string(home.path().join("second/review/reports/lang-rust.md")).unwrap();
    assert_eq!(first, second);
}

#[test]
fn a_refused_report_is_never_stored_and_its_agent_runs_again() {
    let home = TempDir::new().unwrap();
    let store = home.path().join("answers");

    let (out, _) = reviewed(home.path(), "first", &store, "lang-rust", &[]);
    assert!(stdout(&out).starts_with("verdict=request_changes\n"));
    assert_eq!(fs::read_dir(&store).unwrap().count(), 4);

    let (out, started) = reviewed(home.path(), "second", &store, "", &[]);

    assert_eq!(started, ["lang-rust", "review-code"]);
    assert!(
        stdout(&out).starts_with("verdict=approve\n"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn a_dropped_answer_runs_its_agent_again_alone_and_a_store_read_alone_gains_nothing() {
    let home = TempDir::new().unwrap();
    let store = home.path().join("answers");
    reviewed(home.path(), "first", &store, "", &[]);
    let id: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(home.path().join("first/review/run-id.json")).unwrap(),
    )
    .unwrap();
    let key = id["agents"]["test-engineer-unit"]["key"].as_str().unwrap();
    let at = format!("--answers={}", store.display());

    let (out, _) = review_run(home.path(), "", &["answers", "drop", key, &at]);

    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        stdout(&out),
        format!("dropped {key} from {}\n", store.display())
    );
    let (_, started) = reviewed(home.path(), "second", &store, "", &["--answers-read-only"]);
    // Its fresh report is the one dropped, so its generalist's key holds.
    assert_eq!(started, ["test-engineer-unit"]);
    assert_eq!(fs::read_dir(&store).unwrap().count(), 4);
    let (out, _) = review_run(home.path(), "", &["answers", "drop", key, &at]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no answer is stored under"));
}
