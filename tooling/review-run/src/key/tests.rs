use std::fs;
use std::path::Path;

use review_eval::host::{Command, Output};
use testkit::TempDir;

use super::*;
use crate::fake::{self, FakeHost};
use crate::plan::{Plan, parse_dispatch};

fn plan() -> Plan {
    Plan {
        class: "standard".to_owned(),
        touched: 1,
        dispatch: parse_dispatch(
            r#"{"review-code": ["lang-rust", "lang-shell"], "review-spec": ["lang-markdown"]}"#,
        )
        .unwrap(),
    }
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// A work directory holding every agent of [`plan`] and each
/// specialist's diff.
fn workdir() -> TempDir {
    let dir = TempDir::new().unwrap();
    for name in [
        "lang-rust",
        "lang-shell",
        "lang-markdown",
        "review-code",
        "review-spec",
    ] {
        fake::define(dir.path(), name, 20);
    }
    for name in ["lang-rust", "lang-shell", "lang-markdown"] {
        write(dir.path(), &format!("review/diffs/{name}.diff"), "+a\n");
    }
    dir
}

fn keys(dir: &Path) -> Keys {
    Keys::compute(dir, &plan(), CLAUDE, 9)
}

const CLAUDE: &str = "2.1.0 (Claude Code)";

#[test]
fn a_key_is_a_full_sha_256_in_hex() {
    let dir = workdir();

    let key = keys(dir.path()).own("lang-rust").unwrap().to_owned();

    assert!(is_key(&key), "{key}");
    assert!(!is_key("abc"));
    assert!(!is_key(&key.to_uppercase()));
    assert!(!is_key(&format!("{}/", &key[..63])));
}

#[test]
fn the_same_inputs_give_the_same_key() {
    let dir = workdir();

    assert_eq!(keys(dir.path()), keys(dir.path()));
}

/// Every key of [`plan`]'s agents, the generalists' over no report.
fn all(keys: &Keys) -> Vec<Option<String>> {
    let mut out: Vec<Option<String>> = ["lang-rust", "lang-shell", "lang-markdown"]
        .iter()
        .map(|n| keys.own(n).map(str::to_owned))
        .collect();
    out.extend(
        ["review-code", "review-spec"]
            .iter()
            .map(|n| keys.generalist(n, &|_| None)),
    );
    out
}

#[test]
fn a_changed_diff_changes_its_specialist_s_key_and_its_generalist_s_alone() {
    let dir = workdir();
    let before = all(&keys(dir.path()));

    write(dir.path(), "review/diffs/lang-rust.diff", "+b\n");
    let after = all(&keys(dir.path()));

    let changed: Vec<bool> = before.iter().zip(&after).map(|(a, b)| a != b).collect();
    assert_eq!(changed, [true, false, false, true, false]);
}

#[test]
fn a_changed_context_changes_its_specialist_s_key_and_its_generalist_s_alone() {
    let dir = workdir();
    write(dir.path(), "review/context/lang-rust.md", "one\n");
    let before = all(&keys(dir.path()));

    write(dir.path(), "review/context/lang-rust.md", "two\n");
    let after = all(&keys(dir.path()));

    let changed: Vec<bool> = before.iter().zip(&after).map(|(a, b)| a != b).collect();
    assert_eq!(changed, [true, false, false, true, false]);
}

#[test]
fn a_changed_definition_changes_its_agent_s_key_and_its_generalist_s_alone() {
    let dir = workdir();
    let before = all(&keys(dir.path()));

    write(
        dir.path(),
        ".claude/agents/lang-markdown.md",
        &format!(
            "{}\nmore\n",
            fs::read_to_string(dir.path().join(".claude/agents/lang-markdown.md")).unwrap()
        ),
    );
    let after = all(&keys(dir.path()));

    let changed: Vec<bool> = before.iter().zip(&after).map(|(a, b)| a != b).collect();
    assert_eq!(changed, [false, false, true, false, true]);
}

#[test]
fn a_changed_model_effort_or_turns_changes_the_key() {
    for (from, to) in [
        ("model: haiku", "model: sonnet"),
        ("effort: low", "effort: high"),
        ("maxTurns: 20", "maxTurns: 21"),
    ] {
        let dir = workdir();
        let before = keys(dir.path()).own("lang-rust").map(str::to_owned);
        let path = dir.path().join(".claude/agents/lang-rust.md");
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, text.replace(from, to)).unwrap();

        let after = keys(dir.path()).own("lang-rust").map(str::to_owned);

        assert_ne!(before, after, "{to}");
        assert!(after.is_some(), "{to}");
    }
}

#[test]
fn every_key_reads_the_claude_version_the_split_limit_and_the_instructions() {
    let dir = workdir();
    let base = all(&keys(dir.path()));

    let other = Keys::compute(dir.path(), &plan(), "2.1.1 (Claude Code)", 9);
    let split = Keys::compute(dir.path(), &plan(), CLAUDE, 10);
    write(dir.path(), "review/pr-instructions.txt", "do this\n");
    let told = keys(dir.path());

    for changed in [other, split, told] {
        let after = all(&changed);
        assert!(base.iter().zip(&after).all(|(a, b)| a != b), "{after:?}");
    }
}

#[test]
fn a_prompt_template_the_runner_changes_changes_the_key() {
    let dir = workdir();
    let mut changed = plan();
    changed.class = "deep".to_owned();

    let deep = Keys::compute(dir.path(), &changed, CLAUDE, 9);

    // The class is in a generalist's prompt alone.
    assert_eq!(deep.own("lang-rust"), keys(dir.path()).own("lang-rust"));
    assert_ne!(
        deep.generalist("review-code", &|_| None),
        keys(dir.path()).generalist("review-code", &|_| None)
    );
}

#[test]
fn a_file_every_agent_may_read_salts_every_key() {
    for rel in [
        "pr-head/docs/srs/invariants.md",
        "pr-head/CLAUDE.md",
        "pr-head/AGENTS.md",
        "pr-head/docs/domain-model/index.md",
        "pr-head/.claude/agents/persona-agent.md",
        "pr-head/.claude/agents/deeper/x.md",
        "CLAUDE.md",
        ".claude/settings.json",
        "docs/srs/invariants.md",
    ] {
        let dir = workdir();
        let before = all(&keys(dir.path()));

        write(dir.path(), rel, "changed\n");
        let after = all(&keys(dir.path()));

        assert!(
            before.iter().zip(&after).all(|(a, b)| a != b),
            "{rel}: {after:?}"
        );
    }
}

#[test]
fn a_main_definition_is_in_its_own_key_alone_not_the_salt() {
    let dir = workdir();
    let before = keys(dir.path());

    write(dir.path(), ".claude/agents/persona-agent.md", "x\n");

    assert_eq!(before, keys(dir.path()));
}

#[test]
fn a_salt_tree_entry_that_does_not_read_is_marked_not_skipped() {
    let dir = workdir();
    let before = salt(dir.path());
    // A directory where a file is named reads as one that is there.
    fs::create_dir_all(dir.path().join("pr-head/CLAUDE.md")).unwrap();

    assert_ne!(before, salt(dir.path()));
}

#[test]
fn an_agent_whose_definition_does_not_load_has_no_key() {
    let dir = workdir();
    fs::remove_file(dir.path().join(".claude/agents/lang-rust.md")).unwrap();

    let keys = keys(dir.path());

    assert_eq!(keys.own("lang-rust"), None);
    assert_eq!(keys.part("lang-rust", 1, 2), None);
    // Its generalist chains a missing key, so it has one of its own.
    assert!(keys.generalist("review-code", &|_| None).is_some());
    assert_eq!(keys.own("lang-unknown"), None);
    assert_eq!(keys.generalist("review-unknown", &|_| None), None);
}

#[test]
fn a_part_s_key_follows_its_specialist_s_and_its_place() {
    let dir = workdir();
    let keys = keys(dir.path());

    let one = keys.part("lang-rust", 1, 2).unwrap();

    assert!(is_key(&one));
    assert_ne!(Some(one.as_str()), keys.own("lang-rust"));
    assert_ne!(Some(one.clone()), keys.part("lang-rust", 2, 2));
    assert_ne!(Some(one.clone()), keys.part("lang-rust", 1, 3));
    assert_eq!(Some(one), keys.part("lang-rust", 1, 2));
}

#[test]
fn a_generalist_s_key_reads_each_specialist_s_report() {
    let dir = workdir();
    let keys = keys(dir.path());
    let report = |text: &'static str| {
        move |name: &str| (name == "lang-rust").then(|| text.as_bytes().to_vec())
    };

    let a = keys.generalist("review-code", &report("a"));
    let b = keys.generalist("review-code", &report("b"));

    assert_ne!(a, b);
    assert_ne!(a, keys.generalist("review-code", &|_| None));
    assert_eq!(a, keys.generalist("review-code", &report("a")));
}

#[test]
fn the_claude_version_is_asked_once_with_a_deadline() {
    let host = FakeHost::new(|_| Err("not asked".to_owned()));
    let dir = TempDir::new().unwrap();

    let version = claude_version(&host, dir.path());

    assert_eq!(version.as_deref(), Ok(fake::VERSION));
    let calls = host.versions.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].args, ["--version"]);
    assert_eq!(calls[0].deadline, Some(VERSION_DEADLINE));
    assert_eq!(calls[0].stdout_to, None);
}

/// A host whose `claude --version` answers `out`.
struct Versioned(Result<Output, String>);

impl Host for Versioned {
    fn run(&self, _: &Command) -> Result<Output, String> {
        self.0.clone()
    }
}

#[test]
fn a_claude_version_that_does_not_read_is_named() {
    let dir = TempDir::new().unwrap();
    let ok = |stdout: &str| Output {
        code: Some(0),
        stdout: stdout.to_owned(),
        ..Output::default()
    };
    let cases = [
        (Err("claude: not found".to_owned()), "claude: not found"),
        (
            Ok(Output {
                code: Some(1),
                stderr: "boom\n".to_owned(),
                ..Output::default()
            }),
            "claude --version failed: boom",
        ),
        (
            Ok(Output {
                timed_out: true,
                ..ok("2.1.0")
            }),
            "claude --version failed: ",
        ),
        (Ok(ok("  \n")), "claude --version printed nothing"),
    ];
    for (out, want) in cases {
        let got = claude_version(&Versioned(out), dir.path());
        assert_eq!(got, Err(want.to_owned()));
    }
    assert_eq!(
        claude_version(&Versioned(Ok(ok(" 2.1.0 (Claude Code)\n"))), dir.path()),
        Ok("2.1.0 (Claude Code)".to_owned())
    );
}
