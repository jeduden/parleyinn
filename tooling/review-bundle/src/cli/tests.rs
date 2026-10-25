use super::*;
use crate::fixture;
use testkit::TempDir;

/// Runs the command, returning its code, stdout and stderr.
fn invoke(args: &[&str]) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(args.iter().map(|a| (*a).to_owned()), &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// Two copies of the fixture tree, a diff and an output directory.
struct World {
    base: TempDir,
    head: TempDir,
    files: TempDir,
}

impl World {
    fn new(diff: &str) -> Self {
        let w = Self {
            base: TempDir::new().unwrap(),
            head: TempDir::new().unwrap(),
            files: TempDir::new().unwrap(),
        };
        fixture::tree(w.base.path());
        fixture::tree(w.head.path());
        fs::write(w.files.path().join("pr.diff"), diff).unwrap();
        w.consent("[]");
        w
    }

    fn consent(&self, json: &str) {
        fs::write(self.files.path().join("consent.json"), json).unwrap();
    }

    fn path(dir: &TempDir) -> String {
        dir.path().display().to_string()
    }

    fn args(&self) -> Vec<String> {
        vec![
            "--base".into(),
            Self::path(&self.base),
            format!("--head={}", Self::path(&self.head)),
            "--diff".into(),
            self.files.path().join("pr.diff").display().to_string(),
            "--out".into(),
            Self::path(&self.files),
            "--consent".into(),
            self.files.path().join("consent.json").display().to_string(),
        ]
    }

    fn run(&self) -> (u8, String, String) {
        let args = self.args();
        invoke(&args.iter().map(String::as_str).collect::<Vec<_>>())
    }

    fn output(&self, name: &str) -> String {
        fs::read_to_string(self.files.path().join(name)).unwrap()
    }
}

#[test]
fn run_writes_the_bundle_and_findings_and_prints_the_route() {
    let w = World::new("diff --git a/docs/srs/05.md b/docs/srs/05.md\n+REC-01\n");

    let (code, out, err) = w.run();

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(out, "class=deep\ntouched=1\nfindings=0\nconsults=4\n");
    assert!(w.output(BUNDLE).contains("### REC-01"));
    assert_eq!(w.output(CHECKED), "[]\n");
    let dispatch: serde_json::Value = serde_json::from_str(&w.output(DISPATCH)).unwrap();
    assert_eq!(
        dispatch,
        serde_json::json!({"review-spec": ["domain-model-hub", "lang-markdown", "srs-05"]})
    );
}

#[test]
fn run_writes_the_ratchet_finding() {
    let w = World::new("diff --git a/features/record.feature b/features/record.feature\n");
    let feature = w.head.path().join("features/record.feature");
    let body = fs::read_to_string(&feature).unwrap();
    fs::write(
        &feature,
        body.replace("@REC-01 @P0 @I1", "@REC-01 @P0 @I1 @pending"),
    )
    .unwrap();

    let (code, out, err) = w.run();

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.contains("findings=1"), "{out}");
    let checked: serde_json::Value = serde_json::from_str(&w.output(CHECKED)).unwrap();
    assert_eq!(checked[0]["severity"], "blocking");
    assert_eq!(checked[0]["path"], "features/record.feature");
}

#[test]
fn run_reads_a_diff_that_holds_bytes_that_are_not_text() {
    // A seed git shows as text, since it holds no NUL byte, though it is
    // not UTF-8; the file the diff names after it is still read.
    let w = World::new("");
    let mut diff =
        b"diff --git a/fuzz/corpus/seed b/fuzz/corpus/seed\n+\xa9\xff\xfe\x80\n".to_vec();
    diff.extend(b"diff --git a/docs/srs/05.md b/docs/srs/05.md\n+REC-01\n");
    fs::write(w.files.path().join("pr.diff"), diff).unwrap();

    let (code, out, err) = w.run();

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(out, "class=deep\ntouched=1\nfindings=0\nconsults=4\n");
    assert!(w.output(BUNDLE).contains("### REC-01"));
}

/// A change that breaks one input.
type Breaks<'a> = &'a dyn Fn(&World);

#[test]
fn run_fails_on_a_tree_a_diff_or_an_output_it_cannot_use() {
    let cases: [(&str, Breaks, &str); 6] = [
        (
            "main's tree",
            &|w| fs::remove_dir_all(w.base.path().join("docs/srs")).unwrap(),
            "(main's tree)",
        ),
        (
            "the pull request's tree",
            &|w| fs::remove_dir_all(w.head.path().join("docs/srs")).unwrap(),
            "(the pull request's tree)",
        ),
        (
            "the diff",
            &|w| fs::remove_file(w.files.path().join("pr.diff")).unwrap(),
            "pr.diff",
        ),
        (
            "the output",
            &|w| fs::create_dir(w.files.path().join(BUNDLE)).unwrap(),
            "write",
        ),
        (
            "the findings",
            &|w| fs::create_dir(w.files.path().join(CHECKED)).unwrap(),
            "checked.json",
        ),
        (
            "the consent",
            &|w| w.consent("{\"author\": \"a\"}"),
            "consent.json",
        ),
    ];
    for (name, change, want) in cases {
        let w = World::new("");
        change(&w);
        let (code, out, err) = w.run();
        assert_eq!(code, EXIT_FAILURE, "{name}");
        assert!(out.is_empty(), "{name}");
        assert!(err.contains(want), "{name}: {err}");
    }
}

#[test]
fn run_refuses_a_wrong_command_line() {
    for (args, want) in [
        (vec!["--base"], "--base needs a value"),
        (vec!["--nope", "x"], r#"unknown flag "--nope""#),
        (vec!["stray"], "unexpected argument"),
        (
            vec!["--base", "a", "--head", "b", "--diff", "c"],
            "--out is required",
        ),
        (
            vec!["--base", "a", "--head", "b", "--diff", "c", "--out", "d"],
            "--consent is required",
        ),
        (vec!["--base="], "--base needs a value"),
    ] {
        let (code, out, err) = invoke(&args);
        assert_eq!(code, EXIT_USAGE, "{args:?}");
        assert!(out.is_empty());
        assert!(
            err.contains(want) && err.contains("usage:"),
            "{args:?}: {err}"
        );
    }
}

#[test]
fn run_takes_an_eng_29_record_with_the_stakeholder_s_consent() {
    let adr = "docs/adr/ADR-2610111300-guard.md";
    let w = World::new(&format!(
        "diff --git a/docs/srs/invariants.md b/docs/srs/invariants.md\ndiff --git a/{adr} b/{adr}\n"
    ));
    fs::write(
        w.head.path().join(adr),
        "---\nid: ADR-2610111300\ntitle: \"g\"\nstatus: accepted\nsummary: >-\n  An invariant changes under ENG-29.\n---\n# ADR-2610111300: g\n",
    )
    .unwrap();

    let (_, refused, _) = w.run();
    w.consent(r#"[{"author": "owner", "at": "2026-10-11T12:00:00Z", "body": "ok"}]"#);
    let (code, out, err) = w.run();

    assert!(refused.contains("findings=1"), "{refused}");
    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.contains("findings=0"), "{out}");
    assert!(
        w.output(BUNDLE)
            .contains("ENG-29 record: `docs/adr/ADR-2610111300-guard.md`, whose summary reads:\n\n> An invariant changes under ENG-29.\n"),
        "{}",
        w.output(BUNDLE)
    );
}

#[test]
fn consents_holds_on_one_comment_with_a_body() {
    let comment = |body: &str| format!(r#"[{{"author": "o", "at": "t", "body": "{body}"}}]"#);

    assert_eq!(consents("[]"), Ok(false));
    assert_eq!(consents(&comment(" \\n\\t")), Ok(false));
    assert_eq!(consents(&comment("I consent")), Ok(true));
    let review =
        |state: &str| format!(r#"[{{"author": "o", "at": "t", "body": "b", "state": "{state}"}}]"#);
    assert_eq!(consents(&review("CHANGES_REQUESTED")), Ok(false));
    assert_eq!(consents(&review("APPROVED")), Ok(true));
    for bad in ["", "{}", "[1]", r#"[{"author": "o", "body": "x"}]"#] {
        assert!(consents(bad).is_err(), "{bad}");
    }
}
