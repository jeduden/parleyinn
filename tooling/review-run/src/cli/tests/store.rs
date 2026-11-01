//! The command with an answer store: a second review of unchanged
//! inputs takes every answer from it and says so, in its step outputs,
//! its usage, its run id and its progress; and a `claude` whose version
//! does not read keys nothing.

use review_eval::host::{Command, Host, Output};

use super::*;

/// Reviews `dir` with the store at `store`, `extra` after.
fn with_store(host: &FakeHost, dir: &Path, store: &Path, extra: &[&str]) -> (u8, String, String) {
    let at = format!("--answers={}", store.display());
    let mut list = vec!["--head-sha", HEAD, at.as_str()];
    list.extend_from_slice(extra);
    run_with(host, dir, &list)
}

/// Holds every usage entry in `dir` to an answer reused from the
/// store: at no cost, with its key.
fn every_usage_entry_reused_from_the_store(dir: &Path) {
    let usage = read(dir, USAGE_FILE);
    for entry in usage.as_array().unwrap() {
        assert_eq!(entry["reused"], true, "{entry}");
        assert_eq!(entry["answer_store"], true, "{entry}");
        assert_eq!(entry["cost_usd"], 0.0, "{entry}");
        assert_eq!(entry["answer_key"].as_str().unwrap().len(), 64, "{entry}");
        assert!(entry.get("resumed_from").is_none(), "{entry}");
    }
}

#[test]
fn a_second_review_of_the_same_inputs_takes_every_answer_from_the_store() {
    let store = TempDir::new().unwrap();
    let first = workdir();
    let host = FakeHost::new(fake::either);
    let (code, out, err) = with_store(&host, first.path(), store.path(), &[]);
    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.contains("\nreused=0\nstore_failed=0\n"), "{out}");
    assert_eq!(fs::read_dir(store.path()).unwrap().count(), 3);

    let again = workdir();
    let host = FakeHost::new(fake::either);
    let (code, out, err) = with_store(&host, again.path(), store.path(), &["--answers-read-only"]);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(host.agents().is_empty(), "{err}");
    assert!(out.contains("\ncost=0.0000\nreused=3\n"), "{out}");
    for rel in [CONSULTED, ANSWERS, GAPS] {
        assert_eq!(read(again.path(), rel), read(first.path(), rel), "{rel}");
    }
    for field in ["verdict", "findings"] {
        assert_eq!(
            read(again.path(), OUTCOME)[field],
            read(first.path(), OUTCOME)[field],
            "{field}"
        );
    }
    every_usage_entry_reused_from_the_store(again.path());
    let id = read(again.path(), RUN_ID);
    assert_eq!(id["claude"], fake::VERSION);
    let key = id["agents"]["lang-rust"]["key"].as_str().unwrap();
    assert!(store.path().join(key).exists());
    assert!(
        err.contains("review-run: review-code (generalist): reused from the answer store"),
        "{err}"
    );
    let outcome = read(again.path(), OUTCOME);
    assert!(
        outcome["summary"]
            .as_str()
            .unwrap()
            .contains("Reused from the answer store: lang-rust, review-code, test-engineer-unit."),
        "{outcome}"
    );
}

/// A host whose `claude --version` fails, every agent answering.
struct Unversioned(FakeHost);

impl Host for Unversioned {
    fn run(&self, command: &Command) -> Result<Output, String> {
        if command.args == ["--version"] {
            return Err("claude: not found".to_owned());
        }
        self.0.run(command)
    }
}

#[test]
fn a_claude_whose_version_does_not_read_keys_nothing_and_says_so() {
    let store = TempDir::new().unwrap();
    let dir = workdir();
    let host = Unversioned(FakeHost::new(fake::either));
    let at = format!("--answers={}", store.path().display());
    let mut out = Vec::new();
    let mut err = Vec::new();

    let code = run(
        &host,
        args(&["--workdir", dir.path().to_str().unwrap(), &at]),
        &mut out,
        &mut err,
    );

    let err = String::from_utf8(err).unwrap();
    assert_eq!(code, EXIT_OK, "{err}");
    assert!(
        err.contains(
            "review-run: the answer keys are unknown, so no answer is reused or stored: \
             claude: not found"
        ),
        "{err}"
    );
    assert_eq!(fs::read_dir(store.path()).unwrap().count(), 0);
    let id = read(dir.path(), RUN_ID);
    assert!(id.get("claude").is_none(), "{id}");
    assert!(id["agents"]["lang-rust"].get("key").is_none(), "{id}");
}

#[test]
fn the_answers_command_runs_in_place_of_a_review() {
    let host = FakeHost::new(fake::either);
    let mut out = Vec::new();
    let mut err = Vec::new();

    let code = run(&host, args(&["answers", "list"]), &mut out, &mut err);

    assert_eq!(code, EXIT_USAGE);
    assert!(
        String::from_utf8(err)
            .unwrap()
            .contains(crate::answers::USAGE)
    );
    assert!(host.agents().is_empty());
}
