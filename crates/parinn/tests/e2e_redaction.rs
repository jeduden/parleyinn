//! Phase 4 of plan 2609292012 end to end, against the built core in an
//! isolated home, every start confined: a hook input past 1 MiB or
//! nested past depth 64 is refused and audited, and the hook exits 0
//! with no injection (SEC-05); `parinn ingest` redacts a secret, the
//! person's pattern and the harness account identifiers before anything
//! is written (SEC-08); and it reads nothing outside the transcript
//! roots (SEC-18).

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]
#![allow(
    clippy::panic,
    reason = "a start the confinement refuses fails the test with its message"
)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::{Value, json};
use testkit::TempDir;
use testkit::boundary::{Component, Confinement};

const CORE: &str = env!("CARGO_BIN_EXE_parinn");

fn core() -> Confinement {
    Confinement::of(Component::Core).unwrap_or_else(|e| panic!("{e}"))
}

#[track_caller]
fn parinn(home: &Path, args: &[&str]) -> Output {
    core()
        .run(Path::new(CORE), args, home)
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Runs the `UserPromptSubmit` hook handler on `input`, with a hook
/// budget the development build's seam sets long enough for reading a
/// 1 MiB input under the confinement's trace.
#[track_caller]
fn hook(home: &Path, input: &[u8]) -> Output {
    let env = [("PARINN_TEST_HOOK_BUDGET_MS", "20000")];
    core()
        .run_in(
            Path::new(CORE),
            &["hook", "UserPromptSubmit"],
            home,
            &env,
            Some(input),
        )
        .unwrap_or_else(|e| panic!("{e}"))
}

/// The audit log's entries in `home`, as JSON.
fn audited(home: &Path) -> Vec<Value> {
    let dir = home.join(".parinn").join("audit");
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let text = fs::read_to_string(entry.unwrap().path()).unwrap();
        out.extend(
            text.lines()
                .map(|l| serde_json::from_str::<Value>(l).unwrap()),
        );
    }
    out
}

/// Every byte the ParleyInn home holds, file by file.
fn home_bytes(dir: &Path, out: &mut Vec<u8>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            home_bytes(&path, out);
        } else {
            out.extend(fs::read(&path).unwrap());
        }
    }
}

fn holds(home: &Path, needle: &str) -> bool {
    let mut bytes = Vec::new();
    home_bytes(&home.join(".parinn"), &mut bytes);
    bytes.windows(needle.len()).any(|w| w == needle.as_bytes())
}

#[test]
fn a_hook_input_past_its_limits_is_refused_audited_and_injects_nothing() {
    let home = TempDir::new().unwrap();
    let mut large = b"{\"prompt\":\"".to_vec();
    large.resize((1 << 20) - 1, b'x');
    large.extend_from_slice(b"\"}");
    let deep = format!("{}1{}", "{\"a\":".repeat(65), "}".repeat(65));

    for (input, detail) in [
        (large, "hook input exceeds 1 MiB"),
        (deep.into_bytes(), "hook input nesting exceeds depth 64"),
    ] {
        let out = hook(home.path(), &input);

        assert_eq!(out.status.code(), Some(0));
        assert!(out.stdout.is_empty() && out.stderr.is_empty(), "{out:?}");
        assert!(
            audited(home.path()).iter().any(|e| e["detail"] == detail),
            "no audit entry records {detail:?}"
        );
    }
}

/// A transcript under the home's default transcript root carrying a
/// secret, the person's pattern and the harness account.
fn secrets(home: &Path) -> PathBuf {
    let dir = home.join(".claude").join("projects").join("-w-app");
    fs::create_dir_all(&dir).unwrap();
    let lines = [
        json!({"type": "attachment", "attachment": {"type": "session_context", "context": {"userEmail": "The user's email address is person@example.com."}}}),
        json!({"type": "attachment", "attachment": {"type": "credential_org", "organizationUuid": "4f7d2c1e-9a3b-4c8d-8e2f-1a2b3c4d5e6f"}}),
        json!({"type": "user", "toolUseResult": {"stdout": "AKIAIOSFODNN7EXAMPLE ACME-12345678 for person@example.com"}}),
    ];
    let path = dir.join("secrets.jsonl");
    let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn ingest_redacts_before_anything_is_written() {
    let home = TempDir::new().unwrap();
    let parinn_home = home.path().join(".parinn");
    fs::create_dir_all(&parinn_home).unwrap();
    fs::set_permissions(&parinn_home, fs::Permissions::from_mode(0o700)).unwrap();
    let config = parinn_home.join("config.toml");
    fs::write(
        &config,
        "[redaction]\nextra_patterns = [\"ACME-[0-9]{8}\"]\n",
    )
    .unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let path = secrets(home.path());

    let out = parinn(home.path(), &["ingest", "--path", path.to_str().unwrap()]);

    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    for secret in [
        "AKIAIOSFODNN7EXAMPLE",
        "ACME-12345678",
        "person@example.com",
        "4f7d2c1e-9a3b-4c8d-8e2f-1a2b3c4d5e6f",
    ] {
        assert!(!holds(home.path(), secret), "the home holds {secret}");
    }
    for marker in [
        "aws_access_key",
        "extra_pattern_1",
        "account_email",
        "organization_id",
    ] {
        assert!(
            holds(home.path(), &format!("[REDACTED:{marker}]")),
            "{marker}"
        );
    }
    let redacted = audited(home.path())
        .iter()
        .filter(|e| e["counter"] == "redacted")
        .count();
    assert_eq!(redacted, 5);
}

#[test]
fn ingest_reads_nothing_outside_the_transcript_roots() {
    let home = TempDir::new().unwrap();
    let outside = home.path().join("t.jsonl");
    fs::write(&outside, "AKIAIOSFODNN7EXAMPLE\n").unwrap();
    let root = home.path().join(".claude").join("projects").join("p");
    fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("link.jsonl")).unwrap();

    for path in [outside, root.join("link.jsonl")] {
        let out = parinn(home.path(), &["ingest", "--path", path.to_str().unwrap()]);

        assert_eq!(out.status.code(), Some(1));
        assert_eq!(
            String::from_utf8_lossy(&out.stderr),
            "parinn ingest: path outside allowed roots\n"
        );
    }
    let refused = audited(home.path())
        .iter()
        .filter(|e| e["detail"] == "path outside allowed roots")
        .count();
    assert_eq!(refused, 2);
    assert!(!home.path().join(".parinn").join("store.db").exists());
}
