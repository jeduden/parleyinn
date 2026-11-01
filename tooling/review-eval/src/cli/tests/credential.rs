use super::*;

/// A stand-in for the Claude token; never a real one.
const TOKEN: &str = "dummy-access-token-0123456789";

fn run_cli_with(host: &dyn Host, token: Option<&str>, args: &[&str]) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run_with(
        host,
        args.iter().map(|a| (*a).to_owned()),
        token,
        &mut out,
        &mut err,
    );
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn stage_credential_writes_the_token_it_was_handed_and_names_none() {
    let root = TempDir::new().unwrap();
    let dir = root.path().join("cred");
    let host = FakeHost::new(root.path());
    let stage = ["stage-credential", "--dir", &text(&dir)];

    let (code, out, err) = run_cli_with(&host, Some(TOKEN), &stage);

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(crate::credential::read(&dir).unwrap(), TOKEN);
    assert!(!out.contains(TOKEN) && !err.contains(TOKEN));
    assert_eq!(
        out,
        format!("staged the Claude token under {}\n", dir.display())
    );

    let (code, _, err) = run_cli_with(&host, None, &stage);
    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(err, "review-eval: CLAUDE_CODE_OAUTH_TOKEN is not set\n");
    let (code, _, err) = run_cli_with(&host, Some("a b"), &stage);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("holds a character no token has"), "{err}");
    let (code, _, _) = run_cli_with(&host, Some(TOKEN), &["stage-credential"]);
    assert_eq!(code, EXIT_USAGE);
}

#[test]
fn scrub_replaces_the_token_in_every_path_and_fails_closed_when_it_found_one() {
    let root = TempDir::new().unwrap();
    let cred = root.path().join("cred");
    crate::credential::stage(&cred, TOKEN).unwrap();
    let reports = root.path().join("reports");
    fs::create_dir_all(&reports).unwrap();
    fs::write(reports.join("a.md"), "clean\n").unwrap();
    let answers = root.path().join("answers.json");
    fs::write(&answers, "{}\n").unwrap();
    let host = FakeHost::new(root.path());
    let (cred_arg, reports_arg, answers_arg) = (text(&cred), text(&reports), text(&answers));
    let args = [
        "scrub",
        "--credential",
        &cred_arg,
        "--path",
        &reports_arg,
        "--path",
        &answers_arg,
    ];

    let (code, out, err) = run_cli_with(&host, None, &args);
    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(out, "redacted=0\n");

    fs::write(&answers, format!("{{\"a\":\"{TOKEN}\"}}\n")).unwrap();
    let (code, out, err) = run_cli_with(&host, None, &args);
    assert_eq!(code, EXIT_LEAKED, "{err}");
    assert_eq!(out, "redacted=1\n");
    assert_eq!(err, format!("review-eval: {}\n", leaked(1)));
    assert_eq!(
        fs::read_to_string(&answers).unwrap(),
        format!("{{\"a\":\"{}\"}}\n", crate::secret::REDACTED)
    );

    // The token the process holds is a form too, staged or not.
    fs::write(&answers, format!("x {TOKEN}")).unwrap();
    let (code, _, _) = run_cli_with(&host, Some(TOKEN), &["scrub", "--path", &answers_arg]);
    assert_eq!(code, EXIT_LEAKED);

    let missing = text(&root.path().join("missing"));
    let (code, _, err) = run_cli_with(
        &host,
        None,
        &["scrub", "--credential", &missing, "--path", &answers_arg],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("review-eval: read "), "{err}");
    let (code, _, _) = run_cli_with(&host, None, &["scrub"]);
    assert_eq!(code, EXIT_USAGE);
    fs::write(reports.join("b"), "x").unwrap();
    let under = text(&reports.join("b/c"));
    let (code, _, err) = run_cli_with(&host, None, &["scrub", "--path", &under]);
    assert_eq!(code, EXIT_FAILURE, "{err}");
}

#[test]
fn a_token_too_short_for_the_scrub_is_neither_staged_nor_run_with() {
    let dir = checkout();
    sound_cases(dir.path());
    let root = TempDir::new().unwrap();
    let cred = root.path().join("cred");
    let host = FakeHost::new(dir.path());
    let short = "dummy-short";

    let (code, _, err) = run_cli_with(
        &host,
        Some(short),
        &["stage-credential", "--dir", &text(&cred)],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.ends_with("refusing to stage it\n"), "{err}");
    assert!(!cred.exists());

    let out_dir = root.path().join("out");
    let (code, _, err) = run_cli_with(
        &host,
        Some(short),
        &[
            "run",
            "--checkout",
            &text(dir.path()),
            "--component",
            "lang-rust",
            "--out",
            &text(&out_dir),
        ],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        err,
        format!(
            "review-eval: the Claude token is shorter than the {} characters the scrub can \
             match; refusing to use it\n",
            crate::secret::SHORTEST
        )
    );
    assert!(!out_dir.exists());
    assert!(!err.contains(short));
}

#[test]
fn leaked_names_the_count_and_what_to_do() {
    assert_eq!(
        leaked(2),
        "an agent's output contained the Claude token: 2 redacted; revoke and rotate it"
    );
}

#[test]
fn run_confines_each_agent_to_the_staged_credential_and_fails_closed_on_a_leak() {
    let dir = checkout();
    sound_cases(dir.path());
    let root = TempDir::new().unwrap();
    let cred = root.path().join("cred");
    crate::credential::stage(&cred, TOKEN).unwrap();
    let cred = fs::canonicalize(&cred).unwrap();
    let mut host = FakeHost::new(dir.path());
    let seen: std::sync::Arc<Mutex<Vec<Command>>> = std::sync::Arc::default();
    let record = std::sync::Arc::clone(&seen);
    let obedient = obedient_claude();
    host.claude = Box::new(move |c: &Command| {
        record.lock().unwrap().push(c.clone());
        let out = obedient(c)?;
        let path = c.stdout_to.as_ref().unwrap();
        let log = fs::read_to_string(path).unwrap().replace(
            "an unwrap outside tests panics",
            &format!("env holds {TOKEN}"),
        );
        fs::write(path, log).unwrap();
        Ok(out)
    });
    let out_dir = root.path().join("out");

    let (code, out, err) = run_cli_with(
        &host,
        None,
        &[
            "run",
            "--checkout",
            &text(dir.path()),
            "--component",
            "lang-rust",
            "--out",
            &text(&out_dir),
            "--credential",
            &text(&cred),
            "--repetitions",
            "1",
            "--passes",
            "1",
        ],
    );

    assert_eq!(code, EXIT_LEAKED, "{out}\n{err}");
    assert!(
        err.contains("review-eval: an agent's output contained the Claude token: "),
        "{err}"
    );
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 3);
    let shown = cred.display().to_string();
    assert!(
        seen[0]
            .env
            .contains(&(crate::credential::CONFIG_VAR.to_owned(), shown.clone()))
    );
    let rule = format!(",Read(/{shown}/**),Read(/{shown})");
    assert!(seen[0].args.iter().any(|a| a.ends_with(&rule)));
    let log = fs::read_to_string(out_dir.join("runs/lang-rust/defect/1/log.jsonl")).unwrap();
    assert!(
        !log.contains(TOKEN) && log.contains(crate::secret::REDACTED),
        "{log}"
    );
    assert!(!out.contains(TOKEN) && !err.contains(TOKEN));
}

#[test]
fn run_names_a_credential_it_cannot_read() {
    let dir = checkout();
    let host = FakeHost::new(dir.path());

    let (code, _, err) = run_cli_with(
        &host,
        None,
        &[
            "run",
            "--checkout",
            &text(dir.path()),
            "--all",
            "--out",
            &text(&dir.path().join("out")),
            "--credential",
            &text(&dir.path().join("missing")),
        ],
    );

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("review-eval: locate "), "{err}");
}
