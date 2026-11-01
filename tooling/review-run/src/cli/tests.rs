use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::*;
use crate::fake::{self, FakeHost};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|a| (*a).to_owned()).collect()
}

/// A work directory as the workflow prepares it, for one generalist
/// and two specialists.
fn workdir() -> TempDir {
    let dir = TempDir::new().unwrap();
    let review = dir.path().join("review");
    fs::create_dir_all(&review).unwrap();
    fs::write(
        review.join("dispatch.json"),
        r#"{"review-code": ["lang-rust", "test-engineer-unit"]}"#,
    )
    .unwrap();
    fs::write(dir.path().join("class.txt"), "class=standard\ntouched=1\n").unwrap();
    for name in ["lang-rust", "test-engineer-unit", "review-code"] {
        fake::define(dir.path(), name, 20);
    }
    dir
}

fn run_in(host: &FakeHost, dir: &Path) -> (u8, String, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        host,
        args(&["--workdir", dir.to_str().unwrap()]),
        &mut out,
        &mut err,
    );
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

fn read(dir: &Path, rel: &str) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(dir.join(rel)).unwrap()).unwrap()
}

#[test]
fn a_review_writes_every_file_the_gate_reads() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    let (code, out, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(
        out,
        "verdict=approve\ndispatched=3\nconsulted=3\nfindings=1\ncost=0.0300\n"
    );
    assert_eq!(err.lines().count(), 3, "{err}");
    let outcome = fs::read_to_string(dir.path().join("review/outcome.json")).unwrap();
    let outcome = review_gate::parse_outcome(&outcome).unwrap();
    assert_eq!(outcome.verdict, "approve");
    let consulted = fs::read_to_string(dir.path().join("review/consulted.json")).unwrap();
    assert_eq!(
        review_gate::parse_consulted(&consulted).unwrap(),
        ["lang-rust", "review-code", "test-engineer-unit"]
            .map(String::from)
            .into()
    );
    let answers = fs::read_to_string(dir.path().join("review/answers.json")).unwrap();
    assert_eq!(review_gate::parse_answers(&answers).unwrap().len(), 3);
    let usage = read(dir.path(), "review/usage.json");
    assert_eq!(usage.as_array().unwrap().len(), 3);
    assert_eq!(usage[0]["calls"], 1);
    assert!(dir.path().join("review/logs/review-code.jsonl").exists());
}

/// Standard output that takes no write.
struct Closed;

impl Write for Closed {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("closed"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_review_whose_step_outputs_cannot_be_printed_fails_naming_why() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);
    let mut err = Vec::new();

    let code = run(
        &host,
        args(&["--workdir", dir.path().to_str().unwrap()]),
        &mut Closed,
        &mut err,
    );

    assert_eq!(code, EXIT_FAILURE);
    let err = String::from_utf8(err).unwrap();
    assert!(
        err.ends_with("review-run: print the step outputs: closed\n"),
        "{err}"
    );
    assert!(Closed.flush().is_ok());
}

#[test]
fn a_review_whose_specialist_fails_is_still_decided() {
    let dir = workdir();
    let host = FakeHost::new(|c| {
        if fake::agent_of(c) == "lang-rust" {
            Err("claude: not found".to_owned())
        } else {
            fake::either(c)
        }
    });

    let (code, out, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_OK);
    assert!(out.starts_with("verdict=request_changes\n"), "{out}");
    assert!(
        err.contains("review-run: lang-rust (specialist): did not answer: claude: not found"),
        "{err}"
    );
    let consulted = read(dir.path(), "review/consulted.json");
    assert_eq!(
        consulted,
        serde_json::json!(["review-code", "test-engineer-unit"])
    );
}

#[test]
fn a_review_without_its_plan_cannot_run() {
    let dir = TempDir::new().unwrap();
    let host = FakeHost::new(fake::either);

    let (code, out, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_FAILURE);
    // The reason goes to the output too, which the workflow puts in
    // the job summary of a review that failed.
    assert!(out.starts_with("failed=read "), "{out}");
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(err.starts_with("review-run: read "), "{err}");
    assert!(host.agents().is_empty());
}

#[test]
fn a_specialist_report_that_cannot_be_read_names_why() {
    let dir = workdir();
    let path = dir.path().join(report_file("lang-rust"));
    fs::create_dir_all(&path).unwrap();
    let host = FakeHost::new(fake::either);

    let (code, out, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.starts_with("verdict=request_changes\n"), "{out}");
    assert!(
        err.contains(&format!(
            "review-run: read {}: Is a directory (os error 21)",
            path.display()
        )),
        "{err}"
    );
}

/// Runs a review whose file `rel` the gate reads cannot be written, and
/// requires it to fail naming that file.
fn fails_writing(rel: &str) {
    let dir = workdir();
    let path = dir.path().join(rel);
    fs::create_dir_all(&path).unwrap();
    let host = FakeHost::new(fake::either);

    let (code, _, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_FAILURE);
    // Each agent's progress line comes first, in the order they end.
    assert_eq!(
        err.lines().last(),
        Some(
            format!(
                "review-run: write {}: Is a directory (os error 21)",
                path.display()
            )
            .as_str()
        ),
        "{err}"
    );
}

#[test]
fn a_review_whose_outcome_cannot_be_written_fails() {
    fails_writing(OUTCOME);
}

#[test]
fn a_review_whose_consulted_list_cannot_be_written_fails() {
    fails_writing(CONSULTED);
}

#[test]
fn a_review_whose_answers_cannot_be_written_fails() {
    fails_writing(ANSWERS);
}

#[test]
fn a_review_whose_usage_cannot_be_written_fails() {
    fails_writing(USAGE_FILE);
}

#[test]
fn a_review_whose_gaps_or_run_id_cannot_be_written_fails() {
    fails_writing(GAPS);
    fails_writing(NOT_REVIEWED);
    fails_writing(RUN_ID);
}

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const MOVED: &str = "89abcdef0123456789abcdef0123456789abcdef";

/// Runs the review in `dir` with `extra` after the work directory.
fn run_with(host: &FakeHost, dir: &Path, extra: &[&str]) -> (u8, String, String) {
    let mut line = vec!["--workdir", dir.to_str().unwrap()];
    line.extend(extra);
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(host, args(&line), &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// Copies the tree at `from` to `to`.
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A first review of [`workdir`] on [`HEAD`] in which lang-rust never
/// starts, its review directory kept in `earlier` as the local run
/// keeps it.
fn first_review(earlier: &Path) {
    let dir = workdir();
    let host = FakeHost::new(|c| {
        if fake::agent_of(c) == "lang-rust" {
            Err("claude: not found".to_owned())
        } else {
            fake::either(c)
        }
    });
    let (code, out, _) = run_with(&host, dir.path(), &["--head-sha", HEAD]);
    assert_eq!(code, EXIT_OK);
    assert!(out.starts_with("verdict=request_changes\n"), "{out}");
    copy_tree(&dir.path().join("review"), earlier);
}

/// Resumes the review in a fresh [`workdir`] from `earlier` on `head`,
/// every agent answering; `change` edits the work directory first.
fn resume_from(earlier: &Path, head: &str, change: &dyn Fn(&Path)) -> (FakeHost, TempDir, String) {
    let dir = workdir();
    change(dir.path());
    let host = FakeHost::new(fake::either);
    let gaps = earlier.join("gaps.json");
    let (code, _, err) = run_with(
        &host,
        dir.path(),
        &[
            "--head-sha",
            head,
            "--only",
            gaps.to_str().unwrap(),
            "--from",
            earlier.to_str().unwrap(),
        ],
    );
    assert_eq!(code, EXIT_OK, "{err}");
    (host, dir, err)
}

#[test]
fn a_review_writes_what_it_did_not_review_and_what_it_ran_on() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    let (code, _, err) = run_with(&host, dir.path(), &["--head-sha", HEAD]);

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(read(dir.path(), GAPS), serde_json::json!([]));
    assert!(
        fs::read_to_string(dir.path().join(NOT_REVIEWED))
            .unwrap()
            .contains("Every dispatched agent has an accepted report")
    );
    assert!(!err.contains("not reviewed"), "{err}");
    let id = read(dir.path(), RUN_ID);
    assert_eq!(id["head"], HEAD);
    let definition = fs::read(dir.path().join(".claude/agents/lang-rust.md")).unwrap();
    assert_eq!(
        id["agents"]["lang-rust"]["definition"],
        run_id::sha256(&definition)
    );
    assert!(id.get("resumed_from").is_none());
}

#[test]
fn a_review_whose_head_is_not_given_reads_it_from_the_head_s_tree() {
    let dir = workdir();
    fs::create_dir_all(dir.path().join("pr-head/.git")).unwrap();
    fs::write(dir.path().join("pr-head/.git/HEAD"), format!("{MOVED}\n")).unwrap();
    let host = FakeHost::new(fake::either);

    let (code, _, err) = run_with(&host, dir.path(), &[]);

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(read(dir.path(), RUN_ID)["head"], MOVED);
}

#[test]
fn an_agent_that_did_not_answer_is_a_gap_and_said_not_reviewed() {
    let earlier = TempDir::new().unwrap();

    first_review(earlier.path());

    assert_eq!(
        read(earlier.path(), "gaps.json"),
        serde_json::json!([{"name": "lang-rust", "kind": "specialist",
                            "reason": "silent: claude: not found"}])
    );
    let page = fs::read_to_string(earlier.path().join("not-reviewed.md")).unwrap();
    assert!(
        page.contains("- `lang-rust` (specialist): silent: claude: not found"),
        "{page}"
    );
    let outcome = read(earlier.path(), "outcome.json");
    assert!(
        outcome["summary"]
            .as_str()
            .unwrap()
            .contains(" Not reviewed: lang-rust."),
        "{outcome}"
    );
}

#[test]
fn a_failed_review_s_last_line_says_what_was_not_reviewed() {
    let dir = workdir();
    let host = FakeHost::new(|c| {
        if fake::agent_of(c) == "lang-rust" {
            Err("claude: not found".to_owned())
        } else {
            fake::either(c)
        }
    });

    let (_, _, err) = run_with(&host, dir.path(), &[]);

    assert_eq!(
        err.lines().last(),
        Some("review-run: not reviewed: lang-rust"),
        "{err}"
    );
}

#[test]
fn a_resumed_review_runs_only_the_gap_and_its_generalist_and_approves() {
    let earlier = TempDir::new().unwrap();
    first_review(earlier.path());

    let (host, dir, err) = resume_from(earlier.path(), HEAD, &|_| {});

    assert_eq!(host.agents(), ["lang-rust", "review-code"], "{err}");
    assert_eq!(read(dir.path(), OUTCOME)["verdict"], "approve");
    assert_eq!(
        read(dir.path(), CONSULTED),
        serde_json::json!(["lang-rust", "review-code", "test-engineer-unit"])
    );
    assert_eq!(read(dir.path(), GAPS), serde_json::json!([]));
    let usage = read(dir.path(), USAGE_FILE);
    let entry = |name: &str| {
        usage
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["agent"] == name)
            .unwrap()
            .clone()
    };
    assert_eq!(entry("test-engineer-unit")["reused"], true);
    assert_eq!(entry("test-engineer-unit")["cost_usd"], 0.0);
    assert_eq!(
        entry("test-engineer-unit")["resumed_from"],
        earlier.path().to_str().unwrap()
    );
    assert!(entry("lang-rust").get("reused").is_none());
    assert_eq!(
        read(dir.path(), RUN_ID)["resumed_from"],
        earlier.path().to_str().unwrap()
    );
    // The kept specialist's report and logs are the earlier run's.
    for rel in [
        "review/reports/test-engineer-unit.md",
        "review/logs/test-engineer-unit.jsonl",
    ] {
        let name = rel.trim_start_matches("review/");
        assert_eq!(
            fs::read(dir.path().join(rel)).unwrap(),
            fs::read(earlier.path().join(name)).unwrap(),
            "{rel}"
        );
    }
    assert!(
        err.contains("review-run: test-engineer-unit (specialist): reused from the earlier run"),
        "{err}"
    );
    assert!(
        err.contains("review-run: lang-rust: runs again: silent: claude: not found"),
        "{err}"
    );
    assert!(
        err.contains(&format!(
            "review-run: resumed from {}: reused 1 agents, re-ran 2 (lang-rust, review-code)",
            earlier.path().display()
        )),
        "{err}"
    );
}

#[test]
fn a_resumed_review_with_no_gap_runs_nothing_and_decides_as_the_full_run_did() {
    let earlier = TempDir::new().unwrap();
    let dir = workdir();
    let host = FakeHost::new(fake::either);
    let (code, _, _) = run_with(&host, dir.path(), &["--head-sha", HEAD]);
    assert_eq!(code, EXIT_OK);
    copy_tree(&dir.path().join("review"), earlier.path());

    let (host, again, err) = resume_from(earlier.path(), HEAD, &|_| {});

    assert!(host.agents().is_empty(), "{err}");
    for rel in [OUTCOME, CONSULTED, ANSWERS, GAPS] {
        assert_eq!(read(again.path(), rel), read(dir.path(), rel), "{rel}");
    }
}

#[test]
fn a_resumed_review_on_a_moved_head_reuses_nothing_and_says_why() {
    let earlier = TempDir::new().unwrap();
    first_review(earlier.path());

    let (host, _, err) = resume_from(earlier.path(), MOVED, &|_| {});

    assert_eq!(host.agents().len(), 3, "{err}");
    assert!(
        err.contains(&format!(
            "review-run: nothing reused: the head changed from {HEAD} to {MOVED}"
        )),
        "{err}"
    );
    assert!(err.contains("reused 0 agents, re-ran 3"), "{err}");
}

#[test]
fn a_resumed_review_runs_again_an_agent_whose_definition_changed_and_its_generalist() {
    let earlier = TempDir::new().unwrap();
    first_review(earlier.path());

    let (host, _, err) = resume_from(earlier.path(), HEAD, &|dir| {
        fake::define(dir, "test-engineer-unit", 19);
    });

    assert_eq!(
        host.agents().len(),
        3,
        "test-engineer-unit runs again: {err}"
    );
    assert!(
        err.contains("review-run: test-engineer-unit: runs again: its definition changed"),
        "{err}"
    );
}

#[test]
fn a_resume_from_this_run_s_own_review_directory_or_a_missing_gap_list_fails() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);
    let own = dir.path().join("review");
    let (code, _, err) = run_with(
        &host,
        dir.path(),
        &["--only", "g.json", "--from", own.to_str().unwrap()],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("is this run's own review directory"), "{err}");

    let earlier = TempDir::new().unwrap();
    let (code, _, err) = run_with(
        &host,
        dir.path(),
        &[
            "--only",
            "missing.json",
            "--from",
            earlier.path().to_str().unwrap(),
        ],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("review-run: read missing.json"), "{err}");
    assert!(host.agents().is_empty());
}

fn refusal(bad: &[&str]) -> String {
    parse(args(bad)).unwrap_err()
}

#[test]
fn the_work_directory_alone_takes_the_default_options() {
    assert_eq!(
        parse(args(&["--workdir", "w"])).unwrap(),
        Args {
            workdir: "w".to_owned(),
            options: Options::default(),
            head: None,
            resume: None,
            credential: None,
        }
    );
}

#[test]
fn the_flags_read_inline_or_as_the_next_argument() {
    assert_eq!(
        parse(args(&[
            "--workdir=w",
            "--parallel=2",
            "--deadline",
            "30",
            "--head-sha",
            HEAD,
            "--only=g.json",
            "--from",
            "old",
            "--credential",
            "cred",
        ]))
        .unwrap(),
        Args {
            workdir: "w".to_owned(),
            options: Options {
                parallel: 2,
                deadline: Duration::from_secs(30),
                split: crate::parts::LIMIT,
            },
            head: Some(HEAD.to_owned()),
            resume: Some(("g.json".to_owned(), "old".to_owned())),
            credential: Some("cred".to_owned()),
        }
    );
}

#[test]
fn a_head_that_is_no_commit_id_is_refused() {
    assert_eq!(
        refusal(&["--workdir", "w", "--head-sha", "HEAD"]),
        "--head-sha \"HEAD\" is no commit id"
    );
}

#[test]
fn a_gap_list_without_its_earlier_run_is_refused() {
    for line in [
        &["--workdir", "w", "--only", "g.json"][..],
        &["--workdir", "w", "--from", "old"][..],
    ] {
        assert_eq!(refusal(line), "--only and --from are given together");
    }
}

#[test]
fn a_command_line_without_the_work_directory_is_refused() {
    assert_eq!(refusal(&[]), "--workdir is required");
}

#[test]
fn an_argument_that_is_no_flag_is_refused() {
    assert_eq!(refusal(&["w"]), "unexpected argument \"w\"");
}

#[test]
fn an_unknown_flag_is_refused() {
    assert_eq!(refusal(&["--out", "x"]), "unknown flag \"--out\"");
}

#[test]
fn a_flag_without_a_value_is_refused() {
    assert_eq!(refusal(&["--workdir"]), "--workdir needs a value");
    assert_eq!(refusal(&["--workdir="]), "--workdir needs a value");
}

#[test]
fn a_zero_bound_is_refused() {
    assert_eq!(
        refusal(&["--workdir", "w", "--parallel", "0"]),
        "--parallel \"0\" is not a number above zero"
    );
}

#[test]
fn a_deadline_that_is_no_number_is_refused() {
    assert_eq!(
        refusal(&["--workdir", "w", "--deadline", "soon"]),
        "--deadline \"soon\" is not a number above zero"
    );
}

#[test]
fn a_wrong_command_line_prints_the_usage() {
    let host = FakeHost::new(fake::either);
    let mut out = Vec::new();
    let mut err = Vec::new();

    let code = run(&host, args(&["--nope"]), &mut out, &mut err);

    assert_eq!(code, EXIT_USAGE);
    let err = String::from_utf8(err).unwrap();
    assert!(err.starts_with("review-run: unknown flag"), "{err}");
    assert!(err.ends_with(&format!("{USAGE}\n")), "{err}");
}

#[test]
fn a_value_that_does_not_encode_is_an_error_naming_its_file() {
    let dir = TempDir::new().unwrap();
    let value: BTreeMap<(u8, u8), u8> = BTreeMap::from([((1, 2), 3)]);

    let leaks = AtomicUsize::new(0);
    let written = Written {
        workdir: dir.path(),
        scrubber: &Scrubber::default(),
        leaks: &leaks,
    };

    let err = written.json("out.json", &value).unwrap_err();

    assert!(err.starts_with("encode "), "{err}");
    assert!(err.contains("out.json: "), "{err}");
    assert!(!dir.path().join("out.json").exists());
}

/// A stand-in for the Claude token; never a real one.
const TOKEN: &str = "dummy-access-token-0123456789";

/// Runs the review in `dir` handed `token`, with `extra` after the work
/// directory.
fn run_holding(
    host: &FakeHost,
    dir: &Path,
    token: Option<&str>,
    extra: &[&str],
) -> (u8, String, String) {
    let mut line = vec!["--workdir", dir.to_str().unwrap()];
    line.extend(extra);
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = super::run_with(host, args(&line), token, &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// Every file under `dir`'s review directory, read.
fn written(dir: &Path) -> String {
    fn walk(at: &Path, all: &mut String) {
        for entry in fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, all);
            } else {
                all.push_str(&String::from_utf8_lossy(&fs::read(&path).unwrap()));
            }
        }
    }
    let mut all = String::new();
    walk(&dir.join("review"), &mut all);
    all
}

/// A specialist that reads the staged credential, as an agent whose
/// Read was not refused would, and states the token it found; every
/// other agent answers as it does.
fn leaking(command: &review_eval::host::Command) -> Result<review_eval::host::Output, String> {
    if fake::agent_of(command) != "lang-rust" {
        return fake::either(command);
    }
    let dir = command
        .env
        .iter()
        .find(|(k, _)| k == review_eval::credential::CONFIG_VAR)
        .map(|(_, v)| v.clone())
        .unwrap();
    let token = review_eval::credential::read(Path::new(&dir)).unwrap();
    let text = format!("my environment holds {token}");
    let outcome = format!("outcome: none — {token}");
    fake::answer(command, &[&text], &outcome)
}

#[test]
fn a_review_confines_every_agent_to_the_staged_credential() {
    let dir = workdir();
    let cred = dir.path().join("cred");
    review_eval::credential::stage(&cred, TOKEN).unwrap();
    let host = FakeHost::new(fake::either);

    let (code, out, err) = run_holding(
        &host,
        dir.path(),
        None,
        &["--credential", cred.to_str().unwrap()],
    );

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.starts_with("verdict=approve\n"), "{out}");
    let shown = fs::canonicalize(&cred).unwrap().display().to_string();
    for command in host.calls.lock().unwrap().iter() {
        assert!(
            command.env.contains(&(
                review_eval::credential::CONFIG_VAR.to_owned(),
                shown.clone()
            )),
            "{command:?}"
        );
        let rule = format!(",Read(/{shown}/**),Read(/{shown})");
        assert!(
            command.args.iter().any(|a| a.ends_with(&rule)),
            "{command:?}"
        );
    }
}

#[test]
fn a_review_whose_agent_leaked_the_token_scrubs_it_everywhere_and_fails_closed() {
    let dir = workdir();
    let cred = dir.path().join("cred");
    review_eval::credential::stage(&cred, TOKEN).unwrap();
    let host = FakeHost::new(leaking);

    let (code, out, err) = run_holding(
        &host,
        dir.path(),
        None,
        &["--credential", cred.to_str().unwrap()],
    );

    assert_eq!(code, EXIT_FAILURE, "{err}");
    assert!(
        out.starts_with("failed=an agent's output contained the Claude token: "),
        "{out}"
    );
    assert!(out.ends_with("redacted; revoke and rotate it\n"), "{out}");
    assert!(!out.contains(TOKEN) && !err.contains(TOKEN), "{err}");
    let all = written(dir.path());
    assert!(!all.contains(TOKEN), "{all}");
    assert!(all.contains(review_eval::secret::REDACTED));
    let outcome = review_gate::parse_outcome(
        &fs::read_to_string(dir.path().join("review/outcome.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.verdict, "request_changes");
    assert!(
        outcome.findings.iter().any(|f| f.severity == "blocking"
            && f.body
                .starts_with("an agent's output contained the Claude token: ")),
        "{outcome:?}"
    );
}

#[test]
fn a_token_the_process_holds_is_scrubbed_without_a_staged_credential() {
    let dir = workdir();
    let host = FakeHost::new(|c: &review_eval::host::Command| {
        let text = format!("{} saw {TOKEN}", fake::agent_of(c));
        fake::answer(c, &[&text], "outcome: none — fine.")
    });

    let (code, out, _) = run_holding(&host, dir.path(), Some(TOKEN), &[]);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        out.starts_with("failed=an agent's output contained"),
        "{out}"
    );
    assert!(!written(dir.path()).contains(TOKEN));
}

#[test]
fn a_review_whose_credential_does_not_read_runs_no_agent() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);
    let missing = dir.path().join("missing");

    let (code, out, _) = run_holding(
        &host,
        dir.path(),
        None,
        &["--credential", missing.to_str().unwrap()],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(out.starts_with("failed=locate "), "{out}");
    fs::create_dir(&missing).unwrap();
    let (code, out, _) = run_holding(
        &host,
        dir.path(),
        None,
        &["--credential", missing.to_str().unwrap()],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(out.starts_with("failed=read "), "{out}");
    assert!(host.agents().is_empty());
}

#[test]
fn a_review_whose_logs_directory_cannot_be_made_fails_naming_it() {
    let dir = workdir();
    fs::write(dir.path().join("review/logs"), "a file").unwrap();
    let host = FakeHost::new(fake::either);

    let (code, _, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("review-run: make "), "{err}");
    assert!(host.agents().is_empty());
}
