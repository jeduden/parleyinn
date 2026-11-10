//! The review and the Claude token, and the job summary: an agent's run holds
//! the staged credential nowhere it can read, a leak is scrubbed and fails the
//! review closed, and the table of runs joins the job summary.

use super::*;

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
    let code = super::super::run_with(host, args(&line), token, &mut out, &mut err);
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
    let calls = host.calls.lock().unwrap();
    assert!(!calls.is_empty(), "no agent ran");
    for command in calls.iter() {
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
fn a_review_holding_a_token_too_short_for_the_scrub_runs_no_agent() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    let (code, out, _) = run_holding(&host, dir.path(), Some("dummy-short"), &[]);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        out,
        format!(
            "failed=the Claude token is shorter than the {} characters the scrub can match; \
             refusing to use it\n",
            review_eval::secret::SHORTEST
        )
    );
    assert!(host.agents().is_empty());
}

#[test]
fn a_review_holding_a_staged_token_and_its_own_scrubs_both() {
    // The process's own token differs from the staged one: each is a
    // form, and an agent leaking either fails the review closed.
    let own = "dummy-process-token-9876543210";
    let dir = workdir();
    let cred = dir.path().join("cred");
    review_eval::credential::stage(&cred, TOKEN).unwrap();
    let host = FakeHost::new(move |c: &review_eval::host::Command| {
        let text = format!("{} saw {TOKEN} and {own}", fake::agent_of(c));
        fake::answer(c, &[&text], "outcome: none — fine.")
    });

    let (code, out, err) = run_holding(
        &host,
        dir.path(),
        Some(own),
        &["--credential", cred.to_str().unwrap()],
    );

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        out.starts_with("failed=an agent's output contained the Claude token: "),
        "{out}"
    );
    let all = written(dir.path()) + &out + &err;
    assert!(!all.contains(TOKEN) && !all.contains(own), "{all}");
    assert!(all.contains(review_eval::secret::REDACTED));
    assert!(!host.agents().is_empty());
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

#[test]
fn a_cap_or_a_fraction_out_of_range_is_refused() {
    assert_eq!(
        refusal(&["--workdir", "w", "--max-cost-usd", "-1"]),
        "--max-cost-usd \"-1\" is not a number above zero"
    );
    for value in ["0", "1.5", "most", "2", "0/3", "4/3", "1/0", "1/x", "1/3/4"] {
        assert_eq!(
            refusal(&["--workdir", "w", "--breaker-fraction", value]),
            format!("--breaker-fraction {value:?} is no ratio above zero and at most one, as 1/3")
        );
    }
}

#[test]
fn a_mass_failure_opens_the_outcome_with_the_abort_and_counts_what_it_kept_from_starting() {
    let dir = workdir();
    let host = FakeHost::new(|c| {
        if fake::agent_of(c) == "lang-rust" {
            Err("claude: not found".to_owned())
        } else {
            fake::either(c)
        }
    });

    let (code, out, err) = run_with(&host, dir.path(), &[]);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.starts_with("verdict=request_changes\n"), "{out}");
    assert!(
        out.ends_with("over_budget=0\naborted=1\nignored=0\n"),
        "{out}"
    );
    let outcome = read(dir.path(), "review/outcome.json");
    assert!(
        outcome["findings"][0]["body"]
            .as_str()
            .unwrap()
            .starts_with("review aborted: 1 of 2 specialists failed"),
        "{outcome}"
    );
    assert_eq!(
        read(dir.path(), "review/gaps.json")[1]["name"],
        "review-code"
    );
}

#[test]
fn the_table_of_every_run_is_added_to_the_job_summary() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);
    let summary = dir.path().join("summary.md");
    fs::write(&summary, "# Before\n").unwrap();

    let (code, _, err) = run_with(&host, dir.path(), &["--summary", summary.to_str().unwrap()]);

    assert_eq!(code, EXIT_OK, "{err}");
    let text = fs::read_to_string(&summary).unwrap();
    assert!(
        text.starts_with("# Before\n\n### How each agent ran\n"),
        "{text}"
    );
    assert!(
        text.contains("| review-code | generalist | answered |"),
        "{text}"
    );
}

#[test]
fn a_job_summary_that_cannot_be_added_to_is_told_and_the_review_still_decided() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);
    let summary = dir.path().join("summary.md");
    fs::create_dir_all(&summary).unwrap();

    let (code, out, err) = run_with(&host, dir.path(), &["--summary", summary.to_str().unwrap()]);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.starts_with("verdict=approve\n"), "{out}");
    assert!(
        err.contains(&format!(
            "review-run: add to the job summary {}: ",
            summary.display()
        )),
        "{err}"
    );
}

#[test]
fn a_run_whose_text_holds_the_token_is_scrubbed_from_the_job_summary_and_counted() {
    let dir = TempDir::new().unwrap();
    let summary = dir.path().join("summary.md");
    let mut run = crate::run::reused(
        "lang-rust".to_owned(),
        review_eval::component::Kind::Specialist,
        "a",
        Vec::new(),
    );
    run.reused = false;
    run.failure = Some(format!("its run ended with {TOKEN} twice, {TOKEN}"));
    let leaks = AtomicUsize::new(0);

    let scrubber = Scrubber::new(&[TOKEN]);
    let written = Written {
        workdir: dir.path(),
        scrubber: &scrubber,
        leaks: &leaks,
    };

    add_scrubbed_to_summary(&summary, &[run], &written, &|_| {});

    let text = fs::read_to_string(&summary).unwrap();
    assert!(!text.contains(TOKEN), "{text}");
    assert!(text.contains(review_eval::secret::REDACTED), "{text}");
    assert_eq!(leaks.load(Ordering::SeqCst), 2);
}

#[test]
fn a_review_whose_failed_run_stated_the_token_leaves_it_out_of_the_job_summary() {
    let dir = workdir();
    let summary = dir.path().join("summary.md");
    let host = FakeHost::new(|c: &review_eval::host::Command| {
        if fake::agent_of(c) == "lang-rust" {
            fake::write_log(c, &fake::log_of(&[], &format!("error_{TOKEN}"), "", 0.0));
            return Ok(review_eval::host::Output::default());
        }
        fake::either(c)
    });

    let (code, out, err) = run_holding(
        &host,
        dir.path(),
        Some(TOKEN),
        &["--summary", summary.to_str().unwrap()],
    );

    assert_eq!(code, EXIT_FAILURE, "{err}");
    assert!(
        out.starts_with("failed=an agent's output contained the Claude token: "),
        "{out}"
    );
    let text = fs::read_to_string(&summary).unwrap();
    assert!(!text.contains(TOKEN), "{text}");
    assert!(text.contains("| lang-rust | specialist |"), "{text}");
}

#[test]
fn every_review_writes_its_run_list_and_report_a_failed_one_too() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    let (code, _, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_OK, "{err}");
    let runs = fs::read_to_string(dir.path().join(review_report::AGENT_RUNS)).unwrap();
    let lines: Vec<serde_json::Value> = runs
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines[0]["review"]["verdict"], "approve");
    assert_eq!(lines.len(), 1 + 3, "one agent run per agent run");
    assert!(dir.path().join(review_report::REPORT).is_file());

    let failed = TempDir::new().unwrap();
    let (code, _, _) = run_in(&host, failed.path());
    assert_eq!(code, EXIT_FAILURE);
    let runs = fs::read_to_string(failed.path().join(review_report::AGENT_RUNS)).unwrap();
    assert!(runs.starts_with("{\"review\":{"), "{runs}");
    assert!(runs.contains("\"verdict\":\"failed\""), "{runs}");
}

#[test]
fn a_report_that_cannot_be_written_is_named_and_changes_no_verdict() {
    let dir = workdir();
    fs::create_dir_all(dir.path().join(review_report::REPORT)).unwrap();
    let host = FakeHost::new(fake::either);

    let (code, out, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.starts_with("verdict=approve\n"), "{out}");
    assert!(
        err.contains("review-run: run report: review/report.html not written: "),
        "{err}"
    );
    assert!(
        err.contains("review-run: run report: 1 of 2 files not written"),
        "{err}"
    );
}

#[test]
fn a_leak_found_after_a_failed_review_is_named_beside_its_failure() {
    assert_eq!(with_leaks(Ok("v".to_owned()), 0), Ok("v".to_owned()));
    assert_eq!(with_leaks(Ok("v".to_owned()), 2), Err(leak::reason(2)));
    assert_eq!(
        with_leaks(Err("no agent".to_owned()), 0),
        Err("no agent".to_owned())
    );
    assert_eq!(
        with_leaks(Err("no agent".to_owned()), 1),
        Err(format!("no agent; {}", leak::reason(1)))
    );
}
