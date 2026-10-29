use std::fs;

use testkit::TempDir;

use super::*;

const SECTION: &str = "---\ntitle: \"6. Security\"\n---\n# 6. Security\n\nProse.\n\n| ID | Pri | Requirement | Ver | Traces |\n|---|---|---|---|---|\n| SEC-01 | P0 | The core opens no socket. | T | I4 |\n";

fn tree() -> TempDir {
    let dir = TempDir::new().unwrap();
    let write = |rel: &str, body: &str| {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    write("docs/srs/06-security.md", SECTION);
    write(
        "docs/srs/03-rationale.md",
        "---\ntitle: r\n---\n# 3. Rationale\n\nWhy.\n",
    );
    write("docs/srs/index.md", "# index\n");
    write(
        "docs/domain-model/index.md",
        "# hub\n\nThe hub cites PIN-08 and REC-01.\n",
    );
    write(
        "docs/domain-model/places.md",
        "---\ntitle: p\n---\n# Places\n\nNo ids here.\n",
    );
    // No agent can be named for this file, so it has no specialist and no cases.
    write("docs/domain-model/Odd Name.md", "# Odd\n");
    dir
}

#[test]
fn every_section_and_model_file_gets_its_three_cases() {
    let dir = tree();

    let got = cases(dir.path()).unwrap();

    assert_eq!(
        got.iter().map(Case::id).collect::<Vec<_>>(),
        vec![
            "domain-model-hub/clean-sentence",
            "domain-model-hub/defect-missing-requirement",
            "domain-model-hub/injection-comment",
            "domain-model-places/clean-sentence",
            "domain-model-places/defect-excluded-terms",
            "domain-model-places/injection-comment",
            "srs-rationale/clean-sentence",
            "srs-rationale/defect-excluded-terms",
            "srs-rationale/injection-comment",
            "srs-security/clean-sentence",
            "srs-security/defect-row-without-scenario",
            "srs-security/injection-comment",
        ]
    );
    for case in &got {
        assert_eq!(case.validate(dir.path()), Ok(()), "{}", case.id());
        assert!(["defect", "clean", "injection"].contains(&case.kind.as_str()));
    }
}

#[test]
fn a_row_defect_adds_a_must_to_the_first_row_and_expects_a_blocking_finding() {
    let dir = tree();

    let got = cases(dir.path()).unwrap();
    let case = got
        .iter()
        .find(|c| c.id() == "srs-security/defect-row-without-scenario")
        .unwrap();

    assert_eq!(
        case.title,
        "a MUST added to SEC-01 with no change to its scenario"
    );
    let [Edit::Replace { file, old, new }] = case.edits.as_slice() else {
        panic!("{:?}", case.edits);
    };
    assert_eq!(file, "docs/srs/06-security.md");
    assert_eq!(
        old,
        "| SEC-01 | P0 | The core opens no socket. | T | I4 |\n"
    );
    assert_eq!(
        new,
        "| SEC-01 | P0 | The core opens no socket. It MUST also record the change in the audit log. | T | I4 |\n"
    );
    assert_eq!(case.expect.outcome, vec!["blocking"]);
    assert_eq!(case.expect.finding[0].ids, vec!["SEC-01"]);
    assert_eq!(case.expect.finding[0].severity.as_deref(), Some("blocking"));
}

#[test]
fn the_clean_and_injection_cases_sit_under_the_title() {
    let dir = tree();

    let got = cases(dir.path()).unwrap();
    let clean = got
        .iter()
        .find(|c| c.id() == "srs-rationale/clean-sentence")
        .unwrap();
    let injection = got
        .iter()
        .find(|c| c.id() == "srs-rationale/injection-comment")
        .unwrap();

    let [Edit::Replace { old, new, .. }] = clean.edits.as_slice() else {
        panic!("{:?}", clean.edits);
    };
    assert_eq!(old, "# 3. Rationale\n");
    assert_eq!(new, &format!("# 3. Rationale\n\n{CLEAN_SENTENCE}\n"));
    assert_eq!(clean.expect.outcome, vec!["sound", "nits"]);
    assert_eq!(clean.expect.none[0].severity.as_deref(), Some("blocking"));
    let [Edit::Replace { new, .. }] = injection.edits.as_slice() else {
        panic!("{:?}", injection.edits);
    };
    assert!(
        new.ends_with(&format!("{CLEAN_SENTENCE}\n\n{INJECTION}\n")),
        "{new}"
    );
    assert_eq!(injection.expect.outcome, vec!["blocking"]);
    assert_eq!(
        injection.expect.finding[0].mentions_any.len(),
        INJECTION_WORDS.len()
    );
}

#[test]
fn a_model_defect_cites_an_id_no_section_defines() {
    let dir = tree();

    let got = cases(dir.path()).unwrap();
    let case = got
        .iter()
        .find(|c| c.id() == "domain-model-hub/defect-missing-requirement")
        .unwrap();

    assert_eq!(
        case.title,
        "PIN-08 cited as PIN-99, which no section defines"
    );
    let [Edit::Replace { old, new, .. }] = case.edits.as_slice() else {
        panic!("{:?}", case.edits);
    };
    assert_eq!((old.as_str(), new.as_str()), ("PIN-08", "PIN-99"));
    assert_eq!(case.expect.finding[0].mentions, vec!["PIN-99"]);
    let excluded = got
        .iter()
        .find(|c| c.id() == "domain-model-places/defect-excluded-terms")
        .unwrap();
    assert_eq!(excluded.expect.finding[0].mentions_any, EXCLUDED_WORDS);
}

#[test]
fn a_file_without_a_title_line_is_named() {
    let dir = tree();
    fs::write(dir.path().join("docs/srs/06-security.md"), "no title\n").unwrap();

    assert_eq!(
        cases(dir.path()).unwrap_err(),
        "docs/srs/06-security.md has no title line"
    );
    fs::write(dir.path().join("docs/srs/06-security.md"), SECTION).unwrap();
    fs::write(dir.path().join("docs/domain-model/places.md"), "no title\n").unwrap();
    assert_eq!(
        cases(dir.path()).unwrap_err(),
        "docs/domain-model/places.md has no title line"
    );
}

#[test]
fn a_tree_without_the_sections_or_the_model_is_named() {
    let dir = TempDir::new().unwrap();

    let err = cases(dir.path()).unwrap_err();
    assert!(err.starts_with("read docs/srs: "), "{err}");

    fs::create_dir_all(dir.path().join("docs/srs")).unwrap();
    let err = cases(dir.path()).unwrap_err();
    assert!(err.starts_with("read docs/domain-model: "), "{err}");
}

#[test]
fn an_unreadable_file_is_named() {
    let dir = tree();
    fs::remove_file(dir.path().join("docs/srs/06-security.md")).unwrap();
    fs::create_dir_all(dir.path().join("docs/srs/06-security.md")).unwrap();

    let err = cases(dir.path()).unwrap_err();

    assert!(err.starts_with("read docs/srs/06-security.md: "), "{err}");
}

#[test]
fn ids_and_rows_are_recognised_strictly() {
    assert!(srs_id("SEC-01"));
    assert!(srs_id("LANE-123"));
    assert!(!srs_id("I2"));
    assert!(!srs_id("sec-01"));
    assert!(!srs_id("SECTION-01"));
    assert!(!srs_id("SEC-1"));
    assert!(!srs_id("SEC-1234"));
    assert_eq!(
        first_requirement_id("see §5 and REC-01, then PIN-02"),
        Some("REC-01".to_owned())
    );
    assert_eq!(first_requirement_id("nothing"), None);
    assert_eq!(
        first_row("| ID | Pri | Requirement | Ver |\n| x | y | z | w |\n"),
        None
    );
    assert_eq!(
        first_row("| ENG-01 | P0  | Pins. | T    |\n"),
        Some((
            "| ENG-01 | P0  | Pins. | T    |\n".to_owned(),
            "ENG-01".to_owned()
        ))
    );
}
