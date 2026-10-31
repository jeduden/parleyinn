use std::fs;

use testkit::TempDir;

use super::*;

const DISPATCH: &str = r#"{"review-code": ["lang-rust", "test-engineer-unit"], "review-spec-deep": ["srs-engineering-quality"]}"#;

#[test]
fn a_dispatch_reads_each_generalist_with_its_specialists() {
    let d = parse_dispatch(DISPATCH).unwrap();

    assert_eq!(
        d["review-code"],
        ["lang-rust".to_owned(), "test-engineer-unit".to_owned()].into()
    );
    assert_eq!(d["review-spec-deep"].len(), 1);
}

#[test]
fn a_dispatch_refuses_what_is_no_map_of_agents() {
    for (text, want) in [
        ("[]", "malformed dispatch: "),
        (r#"{"review-code": ["../x"]}"#, "agent name \"../x\""),
        (r#"{"Review-code": []}"#, "agent name \"Review-code\""),
        (r#"{"review-code": [""]}"#, "agent name \"\""),
        (r#"{"lang-rust": []}"#, "lang-rust is no generalist"),
        (
            r#"{"review-code": ["review-spec"]}"#,
            "review-spec is no specialist",
        ),
        (
            r#"{"review-code": ["persona-reviewer"]}"#,
            "is no specialist",
        ),
    ] {
        let err = parse_dispatch(text).unwrap_err();
        assert!(err.contains(want), "{text}: {err}");
    }
}

#[test]
fn the_class_reads_from_the_bundle_s_printed_output() {
    assert_eq!(
        parse_class("class=deep\ntouched=3\nfindings=0\nconsults=5\n").unwrap(),
        ("deep".to_owned(), 3)
    );
    for (text, want) in [
        ("touched=3\n", "names no class"),
        (
            "class=huge\ntouched=1\n",
            "class \"huge\" is none of light, standard and deep",
        ),
        ("class=light\n", "names no touched count"),
        (
            "class=light\ntouched=many\n",
            "touched \"many\" is not a number",
        ),
    ] {
        let err = parse_class(text).unwrap_err();
        assert!(err.contains(want), "{text}: {err}");
    }
}

#[test]
fn a_plan_names_its_specialists_generalists_and_every_agent() {
    let plan = Plan {
        class: "standard".to_owned(),
        touched: 1,
        dispatch: parse_dispatch(DISPATCH).unwrap(),
    };

    assert_eq!(
        plan.specialists(),
        vec!["lang-rust", "srs-engineering-quality", "test-engineer-unit"]
    );
    assert_eq!(plan.generalists(), vec!["review-code", "review-spec-deep"]);
    assert_eq!(plan.agents().len(), 5);
    assert_eq!(
        plan.reports_of("review-code"),
        vec![
            "review/reports/lang-rust.md",
            "review/reports/test-engineer-unit.md"
        ]
    );
    assert!(plan.reports_of("review-security").is_empty());
}

#[test]
fn a_plan_reads_from_the_work_directory() {
    let dir = TempDir::new().unwrap();
    let err = read(dir.path()).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
    assert!(err.contains("dispatch.json"), "{err}");

    fs::create_dir_all(dir.path().join("review")).unwrap();
    fs::write(dir.path().join("review/dispatch.json"), DISPATCH).unwrap();
    let err = read(dir.path()).unwrap_err();
    assert!(err.contains("class.txt"), "{err}");

    fs::write(dir.path().join("class.txt"), "class=light\n").unwrap();
    let err = read(dir.path()).unwrap_err();
    assert!(err.contains("class.txt: names no touched count"), "{err}");

    fs::write(dir.path().join("review/dispatch.json"), "{").unwrap();
    let err = read(dir.path()).unwrap_err();
    assert!(err.contains("dispatch.json: malformed dispatch"), "{err}");

    fs::write(dir.path().join("review/dispatch.json"), DISPATCH).unwrap();
    fs::write(dir.path().join("class.txt"), "class=light\ntouched=2\n").unwrap();
    let plan = read(dir.path()).unwrap();
    assert_eq!(plan.class, "light");
    assert_eq!(plan.touched, 2);
    assert_eq!(plan.dispatch.len(), 2);
}

#[test]
fn a_specialist_s_report_file_is_its_name_under_the_reports() {
    assert_eq!(report_file("lang-rust"), "review/reports/lang-rust.md");
}
