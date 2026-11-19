use super::*;
use testkit::TempDir;

/// Writes `body` to `rel` under `dir`.
fn write(dir: &Path, rel: &str, body: &str) {
    let file = dir.join(rel);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, body).unwrap();
}

/// A deterministic findings file with one blocking finding per body of
/// `bodies` and one nit.
fn checked(bodies: &[&str]) -> String {
    let mut findings: Vec<serde_json::Value> = bodies
        .iter()
        .map(|body| serde_json::json!({"path": "a", "line": 0, "severity": "blocking", "body": body}))
        .collect();
    findings.push(serde_json::json!({"path": "a", "line": 0, "severity": "nit", "body": "n"}));
    serde_json::to_string(&findings).unwrap()
}

/// What a bundle that holds to the rule wrote, with no comment and
/// with one that consents.
fn sound() -> (TempDir, TempDir) {
    let (without, with) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    write(
        without.path(),
        CHECKED,
        &checked(&["21", "29", "34", "kept"]),
    );
    write(with.path(), CHECKED, &checked(&["kept"]));
    for dir in [without.path(), with.path()] {
        write(dir, BUNDLE, &format!("# Review bundle\n\n{CONSENT}\n"));
        write(
            dir,
            &format!("{CONTEXTS}/lang-markdown.md"),
            &format!("# Review context\n\n{CONSENT}\n"),
        );
    }

    (without, with)
}

#[test]
fn the_checkout_s_bundle_writes_an_agent_the_same_whatever_the_comments_say() {
    let work = TempDir::new().unwrap();

    let problems = consent_problems(&Checkout::new(&testkit::repo_root()), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_bundle_that_did_not_run_is_named_for_each_run() {
    let work = TempDir::new().unwrap();

    let problems = consent_problems(
        &Checkout::new(&work.path().join("none")),
        &work.path().join("out"),
    );

    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(
        problems
            .iter()
            .all(|p| p.starts_with("review-bundle exited ")),
        "{problems:?}"
    );
}

#[test]
fn the_change_waits_on_every_consent_and_the_comment_gives_each() {
    let diff = change();
    for path in [OWNED, INVARIANTS, RECORD, SIZE_LIST] {
        assert!(
            diff.contains(&format!("diff --git a/{path} b/{path}\n")),
            "{path}"
        );
    }
    assert!(diff.ends_with("+tooling/x.rs 1001\n"), "{diff}");

    let consents = review_bundle::cli::consents(&comment()).unwrap();

    assert_eq!(
        consents,
        [format!("I consent to docs/agents/ and {SIZE_LIST}.")]
    );
}

#[test]
fn files_that_hold_to_the_rule_have_no_problem() {
    let (without, with) = sound();

    assert_eq!(
        differences(without.path(), with.path()),
        Vec::<String>::new()
    );
}

#[test]
fn a_context_or_a_bundle_that_turns_on_the_comment_is_named() {
    let (without, with) = sound();
    let context = format!("{CONTEXTS}/lang-markdown.md");
    write(with.path(), &context, &format!("{CONSENT}\nconsented\n"));
    write(with.path(), BUNDLE, &format!("{CONSENT}\nconsented\n"));

    assert_eq!(
        differences(without.path(), with.path()),
        [
            format!(
                "{context} turns on the stakeholder's comments, though a comment's review \
                 takes the agents' outcome as its own"
            ),
            format!(
                "{BUNDLE} turns on the stakeholder's comments, though a comment's review takes \
                 the agents' outcome as its own"
            ),
        ]
    );
}

#[test]
fn a_file_that_does_not_say_whose_check_consent_is_is_named() {
    assert_eq!(
        file_problems("bundle.md", "x", "x"),
        ["bundle.md does not tell its reader that consent is the review gate's check"]
    );
    assert!(file_problems("bundle.md", CONSENT, CONSENT).is_empty());
}

#[test]
fn a_change_whose_comment_lifts_too_few_findings_proves_nothing() {
    let (without, with) = sound();
    write(without.path(), CHECKED, &checked(&["21", "29", "kept"]));

    assert_eq!(
        differences(without.path(), with.path()),
        [
            "the comment lifts 2 blocking finding(s) of the change, not the 3 that wait on consent \
          (ENG-21, ENG-29, ENG-34)"
        ]
    );
    assert!(blocking("not json").is_empty());
    assert_eq!(blocking(&checked(&["a", "b"])), ["a", "b"]);
}

#[test]
fn a_file_the_bundle_did_not_write_is_named() {
    let (without, with) = sound();
    fs::remove_file(with.path().join(CHECKED)).unwrap();
    fs::remove_file(with.path().join(BUNDLE)).unwrap();

    let problems = differences(without.path(), with.path());

    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(
        problems.iter().all(|p| p.starts_with("read ")),
        "{problems:?}"
    );
}

#[test]
fn a_bundle_that_wrote_no_context_is_named() {
    let (without, with) = sound();
    assert_eq!(
        contexts(with.path()),
        [format!("{CONTEXTS}/lang-markdown.md")]
    );
    fs::remove_dir_all(without.path().join(CONTEXTS)).unwrap();
    fs::remove_dir_all(with.path().join(CONTEXTS)).unwrap();

    let problems = differences(without.path(), with.path());

    assert_eq!(
        problems,
        [format!(
            "the review bundle wrote no context under {}",
            without.path().join(CONTEXTS).display()
        )]
    );
    assert!(contexts(without.path()).is_empty());
}

#[test]
fn a_context_only_one_run_wrote_is_named() {
    let (without, with) = sound();
    let only = format!("{CONTEXTS}/lang-rust.md");
    write(with.path(), &only, CONSENT);

    let problems = differences(without.path(), with.path());

    let missing = without.path().join(&only);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with(&format!("read {}: ", missing.display())),
        "{problems:?}"
    );
}
