use std::fs;

use testkit::TempDir;

use super::*;

fn workdir() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("review")).unwrap();
    dir
}

#[test]
fn a_file_not_there_reads_as_empty_without_a_note() {
    let dir = workdir();
    let mut notes = Vec::new();

    assert_eq!(outcome(dir.path(), &mut notes), Outcome::default());
    assert!(usage(dir.path(), &mut notes).is_empty());
    assert!(gaps(dir.path(), &mut notes).is_empty());
    assert_eq!(run_id(dir.path(), &mut notes), RunId::default());
    assert!(answers(dir.path(), &mut notes).is_empty());
    assert!(notes.is_empty());
}

#[test]
fn each_file_reads_by_field_name_ignoring_fields_it_does_not_know() {
    let dir = workdir();
    let write = |rel: &str, text: &str| fs::write(dir.path().join(rel), text).unwrap();
    write(
        OUTCOME,
        r#"{"verdict":"approve","summary":"s","findings":[{"path":"a","line":1,"severity":"nit","body":"b"}]}"#,
    );
    write(
        USAGE,
        r#"[{"agent":"lang-rust","calls":2,"cost_usd":0.25,"reused":true,"resumed_from":"x"}]"#,
    );
    write(
        GAPS,
        r#"[{"name":"lang-rust","kind":"specialist","part":2,"reason":"silent: x"}]"#,
    );
    write(
        RUN_ID,
        r#"{"head":"abc","inputs":"0123","split":1,"agents":{}}"#,
    );
    write(ANSWERS, r#"{"review-code":"none — nothing to add — x"}"#);
    let mut notes = Vec::new();

    let o = outcome(dir.path(), &mut notes);
    assert_eq!(o.verdict, "approve");
    assert_eq!(o.findings[0].severity, "nit");
    let u = usage(dir.path(), &mut notes);
    assert_eq!(u[0].agent, "lang-rust");
    assert!(u[0].reused && (u[0].cost_usd - 0.25).abs() < 1e-9);
    let g = gaps(dir.path(), &mut notes);
    assert_eq!(g[0].label(), "lang-rust part 2");
    assert_eq!(g[0].reason, "silent: x");
    let id = run_id(dir.path(), &mut notes);
    assert_eq!(
        (id.head.as_deref(), id.inputs.as_str()),
        (Some("abc"), "0123")
    );
    assert_eq!(
        answers(dir.path(), &mut notes)["review-code"],
        "none — nothing to add — x"
    );
    assert!(notes.is_empty());
}

#[test]
fn a_whole_agent_s_gap_is_labelled_by_its_name() {
    let gap = Gap {
        name: "review-code".to_owned(),
        ..Gap::default()
    };

    assert_eq!(gap.label(), "review-code");
}

#[test]
fn a_file_that_does_not_read_is_named_and_reads_as_empty() {
    let dir = workdir();
    fs::write(dir.path().join(OUTCOME), "not json").unwrap();
    // A directory where the file belongs cannot be read as one.
    fs::create_dir_all(dir.path().join(USAGE)).unwrap();
    let mut notes = Vec::new();

    assert_eq!(outcome(dir.path(), &mut notes), Outcome::default());
    assert!(usage(dir.path(), &mut notes).is_empty());
    assert_eq!(notes.len(), 2, "{notes:?}");
    assert!(notes[0].starts_with("review/outcome.json does not read: "));
    assert!(notes[1].starts_with("review/usage.json does not read: "));
}
