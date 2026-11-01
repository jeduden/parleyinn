use super::*;
use testkit::TempDir;

/// Lays one .feature file into a fresh directory.
fn write_feature(body: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("x.feature"), body).unwrap();
    dir
}

#[test]
fn scenarios_reads_tags_and_pending() {
    let dir = write_feature(
        "Feature: Record

  @REC-01 @P0 @I10 @I1 @I1 @pending
  Scenario: ingest transcripts
    Given nothing

  Rule: sources

    @NFR-01
    Scenario: fast hooks
      Given nothing
      Then a hook returns
",
    );

    let got = scenarios(dir.path()).unwrap();

    assert_eq!(got.len(), 2);
    assert_eq!(
        got[0],
        Scenario {
            path: dir.path().join("x.feature"),
            line: 4,
            name: "ingest transcripts".into(),
            id: "REC-01".into(),
            priority: "P0".into(),
            invariants: vec!["I1".into(), "I10".into()],
            pending: true,
            steps: vec!["nothing".into()],
        }
    );
    assert_eq!(got[1].id, "NFR-01");
    assert!(got[1].priority.is_empty());
    assert!(got[1].invariants.is_empty());
    assert_eq!(got[1].line, 10);
    assert!(!got[1].pending);
    assert_eq!(got[1].steps, ["nothing", "a hook returns"]);
}

#[test]
fn scenarios_in_reads_a_feature_from_its_text_as_the_walk_reads_the_file() {
    let body = "Feature: Record\n\n  @REC-01 @P0 @I1 @pending\n  Scenario: ingest\n    Given nothing\n\n  Rule: sources\n\n    @NFR-01\n    Scenario: fast\n      Given nothing\n";
    let dir = write_feature(body);

    let from_text = scenarios_in(&dir.path().join("x.feature"), body).unwrap();

    assert_eq!(from_text, scenarios(dir.path()).unwrap());
    assert_eq!(from_text.len(), 2);
    assert_eq!(from_text[0].path, dir.path().join("x.feature"));
    let err = scenarios_in(
        Path::new("features/x.feature"),
        "Feature: x\n  Scenario: y\n    Given z\n",
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("features/x.feature:2: \"y\" carries 0 requirement tags, want exactly one"),
        "{err}"
    );
    let err = scenarios_in(Path::new("features/x.feature"), "Scenario: no feature\n").unwrap_err();
    assert!(
        err.to_string().starts_with(
            "scenario: read features: Could not parse feature file: features/x.feature"
        ),
        "{err}"
    );
}

#[test]
fn scenarios_folds_outline_rows_onto_one_scenario() {
    let dir = write_feature(
        "Feature: Outline

  @SEC-04 @P0 @I9
  Scenario Outline: literal terms
    Given <q>

    @pending
    Examples:
      | q |
      | a |
      | b |
",
    );

    let got = scenarios(dir.path()).unwrap();

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id, "SEC-04");
    assert_eq!(got[0].steps, ["<q>"]);
    assert!(got[0].pending, "an Examples block's tags count");
}

#[test]
fn scenarios_inherits_feature_and_rule_tags() {
    let dir = write_feature(
        "@pending
Feature: Inherited

  @ASM-01
  Scenario: hook payloads
    Given nothing

  @P1
  Rule: r

    @REC-02
    Scenario: in a rule
      Given nothing
",
    );

    let got = scenarios(dir.path()).unwrap();

    assert_eq!(got.len(), 2);
    assert!(got[0].pending);
    assert_eq!(got[1].priority, "P1");
}

#[test]
fn scenarios_rejects_missing_or_doubled_tags() {
    let cases = [
        ("@P0", "carries 0 requirement tags, want exactly one"),
        (
            "@REC-01 @REC-02",
            "carries 2 requirement tags, want exactly one",
        ),
        ("@REC-1", "carries 0 requirement tags, want exactly one"),
        (
            "@REC-01 @P0 @P1",
            "carries 2 priority tags, want at most one",
        ),
    ];
    for (tags, want) in cases {
        let dir = write_feature(&format!(
            "Feature: F\n\n  {tags}\n  Scenario: s\n    Given x\n"
        ));
        let err = scenarios(dir.path()).unwrap_err().to_string();
        assert!(
            err.ends_with(&format!(r#"x.feature:4: "s" {want}"#)),
            "{tags}: {err}"
        );
    }
}

#[test]
fn scenarios_reports_unreadable_and_malformed_features() {
    let dir = TempDir::new().unwrap();
    let err = scenarios(&dir.path().join("missing")).unwrap_err();
    assert!(
        err.to_string().starts_with("scenario: read features: "),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());

    let dir = write_feature("not gherkin at all\n");
    let err = scenarios(dir.path()).unwrap_err();
    assert!(
        err.to_string().starts_with("scenario: read features: "),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
    assert!(std::error::Error::source(&Error::Tags(String::new())).is_none());
}

#[test]
fn feature_files_walks_subdirectories_in_path_order() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("b")).unwrap();
    for name in ["b/one.feature", "a.FEATURE", "c.feature", "notes.md"] {
        fs::write(dir.path().join(name), "").unwrap();
    }

    let got = feature_files(dir.path()).unwrap();

    let want: Vec<PathBuf> = ["a.FEATURE", "b/one.feature", "c.feature"]
        .iter()
        .map(|n| dir.path().join(n))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn scenarios_accepts_the_lane_families() {
    let dir = write_feature(
        "Feature: F\n\n  @LANE-01 @P0 @I1\n  Scenario: a\n    Given x\n\n  @VIEW-19 @P2\n  Scenario: b\n    Given x\n\n\
             \x20 @OWN-22 @P1\n  Scenario: c\n    Given x\n\n  @PEER-11 @P2\n  Scenario: d\n    Given x\n",
    );

    let ids: Vec<String> = scenarios(dir.path())
        .unwrap()
        .into_iter()
        .map(|s| s.id)
        .collect();

    assert_eq!(ids, ["LANE-01", "VIEW-19", "OWN-22", "PEER-11"]);
}
