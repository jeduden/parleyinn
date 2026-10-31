use super::*;

const ROW_TEXT: &str = "MUST flag: 1000 for Rust (`.rs`), 500 for Gherkin (`.feature`), \
                        500 for YAML (`.yml`, `.yaml`), 200 for TOML (`.toml`) and 200 for \
                        shell (`.sh`); a file for review is no limit.";

fn srs_with(text: &str) -> String {
    format!(
        "# 10\n\n| ID | Pri | Requirement | Ver |\n|---|---|---|---|\n| ENG-34 | P0 | {text} | T |\n"
    )
}

fn nit(path: &str, body: &str) -> Nit {
    Nit {
        path: path.to_owned(),
        severity: "nit".to_owned(),
        body: body.to_owned(),
    }
}

fn built(checked: Vec<Nit>, bundle: &str) -> Built {
    Built {
        checked,
        bundle: bundle.to_owned(),
    }
}

#[test]
fn the_row_s_limits_read_by_kind() {
    assert_eq!(
        stated_limits(ROW_TEXT),
        vec![
            ("Rust".to_owned(), 1000),
            ("Gherkin".to_owned(), 500),
            ("YAML".to_owned(), 500),
            ("TOML".to_owned(), 200),
            ("shell".to_owned(), 200),
        ]
    );
    assert!(stated_limits("for x, 12 for y z, for").is_empty());
}

#[test]
fn limits_that_match_the_row_pass() {
    assert!(limits_match(ROW_TEXT).is_empty());
}

#[test]
fn a_limit_the_row_and_the_bundle_disagree_on_is_named_both_ways() {
    let text = ROW_TEXT.replace("200 for TOML", "250 for TOML");

    assert_eq!(
        limits_match(&text),
        vec![
            "the ENG-34 row states no limit of 200 for TOML, which the review bundle holds",
            "the review bundle holds no limit of 250 for TOML, which the ENG-34 row states",
        ]
    );
}

#[test]
fn the_checkout_s_row_is_read_and_a_missing_row_is_named() {
    let (_d, c) = crate::tests::checkout(&[("docs/srs/10.md", &srs_with(ROW_TEXT))]);
    assert!(limits_match_the_row(&c).is_empty());

    let (_d, c) = crate::tests::checkout(&[("docs/srs/10.md", "# 10\n")]);
    assert_eq!(limits_match_the_row(&c), vec!["no ENG-34 row in docs/srs"]);

    let (_d, c) = crate::tests::checkout(&[]);
    assert_eq!(limits_match_the_row(&c).len(), 1);
}

#[test]
fn the_fixtures_hold_a_file_past_its_limit_and_files_within_them() {
    let past = past_the_limit();
    assert_eq!(past[0].0, OVERSIZE);
    assert_eq!(past[0].1.lines().count(), RUST_LINES + 1);
    assert_eq!(past[1].0, UNREAD);
    for (path, body) in within_the_limits() {
        assert!(body.lines().count() <= RUST_LINES, "{path}");
    }
}

#[test]
fn build_runs_the_review_bundle_on_the_change() {
    let work = testkit::TempDir::new().unwrap();

    let past = build(work.path(), &past_the_limit()).unwrap();

    assert!(nit_at(&past, OVERSIZE, "ENG-34").is_empty(), "{past:?}");
    assert!(nit_at(&past, UNREAD, "ENG-28").is_empty(), "{past:?}");
    assert!(listed(&past).is_empty(), "{past:?}");
    assert_eq!(no_nit(&past).len(), 2);

    let work = testkit::TempDir::new().unwrap();
    let within = build(work.path(), &within_the_limits()).unwrap();

    assert!(no_nit(&within).is_empty(), "{within:?}");
}

#[test]
fn build_names_what_it_could_not_do() {
    let work = testkit::TempDir::new().unwrap();
    fs::write(work.path().join("base"), "a file in the tree's place").unwrap();
    assert!(build(work.path(), &[]).unwrap_err().contains("base"));

    let work = testkit::TempDir::new().unwrap();
    fs::create_dir(work.path().join("pr.diff")).unwrap();
    assert!(build(work.path(), &[]).unwrap_err().contains("pr.diff"));

    let work = testkit::TempDir::new().unwrap();
    // A second row REC-01 the pull request's tree does not read.
    let twice =
        "| ID | Pri | Requirement | Ver |\n|---|---|---|---|\n| REC-01 | P0 | Again. | T |\n";
    let err = build(work.path(), &[("docs/srs/x.md", twice.to_owned())]);
    assert!(err.unwrap_err().contains("review-bundle"));
}

#[test]
fn findings_that_do_not_read_are_named() {
    assert!(read_checked("nope").unwrap_err().contains("checked.json"));
    assert_eq!(
        read_checked(r#"[{"path": "a", "line": 0, "severity": "nit", "body": "b"}]"#),
        Ok(vec![nit("a", "b")])
    );
}

#[test]
fn a_missing_or_malformed_nit_is_named() {
    let good = nit("a.rs", "a.rs — if not addressed: big — ENG-34");
    assert!(nit_at(&built(vec![good.clone()], ""), "a.rs", "ENG-34").is_empty());

    for wrong in [
        nit("b.rs", "b.rs — if not addressed: big — ENG-34"),
        nit("a.rs", "a.rs: big — ENG-34"),
        nit("a.rs", "a.rs — if not addressed: big — ENG-28"),
        Nit {
            severity: "blocking".to_owned(),
            ..good.clone()
        },
    ] {
        assert_eq!(
            nit_at(&built(vec![wrong.clone()], ""), "a.rs", "ENG-34").len(),
            1,
            "{wrong:?}"
        );
    }
}

#[test]
fn a_nit_the_bundle_does_not_list_is_named() {
    let found = nit("a.rs", "a.rs — if not addressed: big — ENG-34");
    let other = nit("b.rs", "b.rs: something else");

    assert!(
        listed(&built(
            vec![found.clone(), other.clone()],
            "- nit `a.rs:0`: a.rs — if not addressed: big — ENG-34\n"
        ))
        .is_empty()
    );
    assert_eq!(
        listed(&built(vec![found], "")),
        vec!["the review bundle does not list the nit at a.rs"]
    );
    assert_eq!(
        listed(&built(vec![other], "")),
        vec!["the review bundle lists no ENG-34 or ENG-28 nit"]
    );
}
