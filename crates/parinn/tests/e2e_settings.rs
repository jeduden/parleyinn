//! Plan 2609292012, phase 3, end to end: the settings layers (ADM-04,
//! PRV-05) and the principal acts the CLI takes at a terminal (OWN-11,
//! OWN-12, OWN-22, SEC-22), driven through the built core in an
//! isolated home, every start confined to the core's boundary
//! (ENG-12). A person's start runs at a terminal; an agent's tool call
//! runs with none.

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

use serde_json::Value;
use testkit::TempDir;
use testkit::boundary::{Component, Confinement};

/// A home for one test, with managed policy read from inside it.
struct Node {
    dir: TempDir,
}

impl Node {
    fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
        }
    }

    fn home(&self) -> &Path {
        self.dir.path()
    }

    fn parinn_home(&self) -> PathBuf {
        self.home().join(".parinn")
    }

    fn managed(&self) -> PathBuf {
        self.home().join("managed_policy.toml")
    }

    fn env(&self) -> Vec<(&'static str, String)> {
        vec![(
            "PARINN_TEST_MANAGED_POLICY",
            self.managed().display().to_string(),
        )]
    }

    /// The agent's tool call: no terminal.
    #[track_caller]
    fn agent(&self, args: &[&str]) -> Output {
        let env = self.env();
        let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (*k, v.as_str())).collect();
        core()
            .run_in(
                Path::new(env!("CARGO_BIN_EXE_parinn")),
                args,
                self.home(),
                &env,
                None,
            )
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// The person at a terminal: stderr arrives in stdout.
    #[track_caller]
    fn person(&self, args: &[&str]) -> Output {
        let env = self.env();
        let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (*k, v.as_str())).collect();
        core()
            .run_at_terminal(
                Path::new(env!("CARGO_BIN_EXE_parinn")),
                args,
                self.home(),
                &env,
            )
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// Writes the principal's configuration, owner-only.
    fn configure(&self, text: &str) {
        fs::create_dir_all(self.parinn_home()).unwrap();
        fs::set_permissions(self.parinn_home(), fs::Permissions::from_mode(0o700)).unwrap();
        let path = self.parinn_home().join("config.toml");
        fs::write(&path, text).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    /// Ingests a three-line transcript: one run.
    fn run_an_agent(&self) -> Output {
        let dir = self.home().join(".claude").join("projects").join("p");
        fs::create_dir_all(&dir).unwrap();
        let transcript = dir.join("t.jsonl");
        fs::write(&transcript, "{\"a\":1}\n{\"b\":2}\n{\"c\":3}\n").unwrap();
        let path = transcript.display().to_string();
        self.agent(&["ingest", "--path", &path, "--json"])
    }

    /// The audit log's entries naming `counter`, as JSON.
    fn audited(&self, counter: &str) -> Vec<Value> {
        let dir = self.parinn_home().join("audit");
        let mut files: Vec<PathBuf> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
            .collect();
        files.sort();
        files
            .iter()
            .flat_map(|f| {
                fs::read_to_string(f)
                    .unwrap()
                    .lines()
                    .map(|l| serde_json::from_str::<Value>(l).unwrap())
                    .collect::<Vec<_>>()
            })
            .filter(|e| e["counter"] == counter)
            .collect()
    }
}

fn core() -> Confinement {
    Confinement::of(Component::Core).unwrap_or_else(|e| panic!("{e}"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

#[test]
fn an_invalid_configuration_fails_status_naming_the_key_and_ingest_refuses_until_it_is_fixed() {
    for (setting, key, problem) in [
        ("recal.max_k = 10", "recal.max_k", "unknown key"),
        ("recall.max_k = 'ten'", "recall.max_k", "type error"),
        ("recall.max_k = 500", "recall.max_k", "out of range"),
        (
            "payload.threshold_bytes = -1",
            "payload.threshold_bytes",
            "out of range",
        ),
    ] {
        let node = Node::new();
        node.configure(&format!("{setting}\n"));

        let ingested = node.run_an_agent();
        let status = node.agent(&["status"]);

        assert_eq!(ingested.status.code(), Some(1), "{setting}");
        assert_eq!(
            node.audited("ingest_refused_settings_invalid").len(),
            1,
            "{setting}"
        );
        assert!(!node.parinn_home().join("store.db").exists(), "{setting}");
        assert_eq!(status.status.code(), Some(2), "{setting}");
        let err = text(&status.stderr);
        assert!(
            err.contains(&format!("key \"{key}\": {problem}")),
            "{setting}: {err}"
        );
        assert_eq!(node.audited("settings_layer_invalid").len(), 1, "{setting}");

        node.configure("recall.max_k = 10\n");
        let fixed = node.run_an_agent();
        let got: Value = serde_json::from_slice(&fixed.stdout).unwrap();
        assert_eq!(got["events"], 3, "{setting}");
    }
}

#[test]
fn an_agents_tool_call_cannot_accept_the_configuration_and_the_person_can() {
    let node = Node::new();
    node.configure("node.deployment_mode = 'interactive'\n");

    let refused = node.agent(&["configuration", "accept"]);

    assert_eq!(refused.status.code(), Some(1));
    assert!(text(&refused.stderr).contains("needs a terminal"));
    assert_eq!(node.audited("principal_act_refused").len(), 1);
    let status: Value = serde_json::from_slice(&node.agent(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["deployment_mode"], "automation");
    assert_eq!(status["awaiting_acceptance"][0], "node.deployment_mode");

    let accepted = node.person(&["configuration", "accept"]);

    assert!(accepted.status.success(), "{}", text(&accepted.stdout));
    assert!(text(&accepted.stdout).contains("now applying: node.deployment_mode"));
    let status: Value = serde_json::from_slice(&node.agent(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["deployment_mode"], "interactive");
}

#[test]
fn a_run_leaves_every_residual_risk_open_until_the_person_accepts_them() {
    let node = Node::new();
    assert!(node.run_an_agent().status.success());
    node.configure("recall.max_k = 10\n");

    let refused = node.person(&["configuration", "accept"]);

    assert_eq!(refused.status.code(), Some(1));
    let said = text(&refused.stdout);
    for id in ["R1", "R2", "R3", "R4", "R5", "R6", "R7", "R8"] {
        assert!(said.contains(&format!("  {id}: ")), "{id}: {said}");
    }
    assert!(said.contains("runs leaving them open"), "{said}");

    let agent = node.agent(&["risk-acceptance", "record"]);
    assert_eq!(agent.status.code(), Some(1));
    let accepted = node.person(&["risk-acceptance", "record"]);
    assert!(accepted.status.success(), "{}", text(&accepted.stdout));
    assert!(text(&accepted.stdout).contains("can forge this risk acceptance"));
    assert!(node.person(&["configuration", "accept"]).status.success());

    assert!(
        node.person(&["risk-acceptance", "withdraw"])
            .status
            .success()
    );
    assert_eq!(
        node.person(&["configuration", "accept"]).status.code(),
        Some(1)
    );
    let shown: Value =
        serde_json::from_slice(&node.agent(&["risk-acceptance", "show", "--json"]).stdout).unwrap();
    assert_eq!(shown["open"].as_array().unwrap().len(), 8);
    assert_eq!(shown["accepted"], Value::Null);
}

#[test]
fn managed_policy_the_principal_can_write_is_refused_and_no_widening_act_is_taken() {
    let node = Node::new();
    // The test's own OS user owns the file, so it could write it.
    fs::write(node.managed(), "authenticator.required = true\n").unwrap();
    node.configure("recall.max_k = 10\n");

    let status = node.agent(&["status"]);
    let accepted = node.person(&["configuration", "accept"]);
    let ingested = node.run_an_agent();

    assert!(status.status.success());
    assert!(
        text(&status.stdout)
            .contains("is writable by the running OS user, so it is no managed policy"),
        "{}",
        text(&status.stdout)
    );
    assert_eq!(accepted.status.code(), Some(1));
    assert!(text(&accepted.stdout).contains("managed policy is invalid"));
    assert!(ingested.status.success());
    assert_eq!(node.audited("managed_policy_invalid").len(), 1);
}

#[test]
fn no_settings_layer_changes_the_trust_policy() {
    let node = Node::new();
    node.configure("trust_policy.\"tool_result:Bash\" = 'trusted'\n");

    assert!(node.run_an_agent().status.success());

    let ignored = node.audited("trust_policy_key_ignored");
    assert_eq!(ignored.len(), 1);
    assert_eq!(
        ignored[0]["detail"],
        "configuration: rejected trust override for tool_result:Bash"
    );
}

#[test]
fn acknowledging_the_counters_is_a_neutral_principal_act_the_person_takes() {
    let node = Node::new();
    node.configure("recal.max_k = 1\n");
    assert_eq!(node.run_an_agent().status.code(), Some(1));
    assert_eq!(node.agent(&["doctor"]).status.code(), Some(1));

    assert_eq!(node.agent(&["counter", "ack"]).status.code(), Some(1));
    assert_eq!(node.agent(&["doctor"]).status.code(), Some(1));

    // Entry 1 audits the failing configuration, entry 2 the refused
    // ingest, entry 3 the agent's refused acknowledgement.
    let acked = node.person(&["counter", "ack"]);
    assert!(acked.status.success(), "{}", text(&acked.stdout));
    assert!(
        text(&acked.stdout).contains("counters acknowledged through audit entry 3"),
        "{}",
        text(&acked.stdout)
    );
    assert_eq!(node.agent(&["doctor"]).status.code(), Some(0));
}
