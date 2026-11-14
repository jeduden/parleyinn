use super::*;
use std::collections::BTreeMap;

fn defs(text: &str) -> Vec<StepDef> {
    definitions_in("a.rs", text).unwrap()
}

fn sc(pending: bool, steps: &[(&str, &str)]) -> Scenario {
    Scenario {
        pending,
        steps: steps.iter().map(|s| s.1.to_owned()).collect(),
        step_kinds: steps.iter().map(|s| s.0.to_owned()).collect(),
        ..Scenario::default()
    }
}

const TWO: &str = r##"
#[given(regex = r"^a home (\d+)$")]
fn home(w: &mut World) {}

#[then(
    regex = r#"^the "x" ran$"#
)]
async fn ran() {}

#[when("it runs")]
pub fn runs() {}
"##;

#[test]
fn definitions_in_reads_each_attribute_form_and_its_function() {
    let got = defs(TWO);

    let names: Vec<_> = got
        .iter()
        .map(|d| (d.kind.as_str(), d.name.as_str()))
        .collect();
    assert_eq!(
        names,
        [("given", "home"), ("then", "ran"), ("when", "runs")]
    );
    assert!(got[0].matches("given", "a home 3"));
    assert!(!got[0].matches("when", "a home 3"));
    assert!(got[1].matches("then", "the \"x\" ran"));
    assert!(got[2].matches("when", "it runs"));
    assert!(!got[2].matches("when", "it runs on"));
    assert_eq!(got[0].key(), "a.rs home");
}

#[test]
fn definitions_in_reads_an_escaped_literal() {
    let got = defs("#[given(\"say \\\"hi\\\"\\n\\t\\\\\")]\nfn f() {}\n");

    assert!(got[0].matches("given", "say \"hi\"\n\t\\"));
}

#[test]
fn definitions_in_refuses_what_it_cannot_read() {
    for (text, want) in [
        ("#[given(regex = r\"^(\")]\nfn f() {}", "pattern"),
        ("#[given(\"{string} ran\")]\nfn f() {}", "parameters"),
        ("#[given(unquoted)]\nfn f() {}", "not a string literal"),
        ("#[given(r\"open)]\nfn f() {}", "unterminated raw"),
        ("#[given(r#)]\nfn f() {}", "not a string literal"),
        ("#[given(\"open)]\nfn f() {}", "unterminated string"),
        ("#[given(\"a\\", "unterminated string"),
        ("#[given(\"a\")]\nstruct S;", "no function"),
    ] {
        let err = definitions_in("a.rs", text).unwrap_err().to_string();

        assert!(err.contains(want), "{text:?}: {err}");
        assert!(err.contains("a.rs:1"), "{err}");
    }
}

#[test]
fn step_definitions_reads_the_rs_files_of_a_directory() {
    let dir = testkit::TempDir::new().unwrap();
    std::fs::write(dir.path().join("b.rs"), "#[given(\"b\")]\nfn b() {}\n").unwrap();
    std::fs::write(dir.path().join("a.rs"), "#[given(\"a\")]\nfn a() {}\n").unwrap();
    std::fs::write(dir.path().join("notes.txt"), "#[given(\"c\")]\n").unwrap();

    let got = step_definitions(dir.path()).unwrap();

    let keys: Vec<_> = got.iter().map(StepDef::key).collect();
    assert_eq!(keys, ["a.rs a", "b.rs b"]);
    assert!(step_definitions(&dir.path().join("none")).is_err());
}

#[test]
fn check_steps_names_a_definition_no_running_row_uses() {
    let d = defs(TWO);
    let rows = [sc(false, &[("given", "a home 1")])];

    let got = check_steps(&d, &rows, &[]);

    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got[0].starts_with("a.rs ran: no row"));
    assert!(got[1].starts_with("a.rs runs: no row"));
}

#[test]
fn check_steps_does_not_count_a_pending_scenario_or_another_kind() {
    let d = defs(TWO);
    let rows = [
        sc(true, &[("given", "a home 1"), ("when", "it runs")]),
        sc(false, &[("given", "it runs")]),
    ];

    let got = check_steps(&d, &rows, &[]);

    assert_eq!(got.len(), 3, "{got:?}");
}

#[test]
fn check_steps_fills_an_outlines_placeholders_from_each_row() {
    let d = defs(TWO);
    let mut outline = sc(false, &[("given", "a home <n>")]);
    outline.examples = vec![BTreeMap::from([("n".to_owned(), "7".to_owned())])];
    let rows = [
        outline,
        sc(false, &[("then", "the \"x\" ran"), ("when", "it runs")]),
    ];

    assert!(check_steps(&d, &rows, &[]).is_empty());
}

#[test]
fn check_steps_accepts_a_listed_definition() {
    let d = defs(TWO);
    let rows = [sc(false, &[("given", "a home 1")])];
    let allowed = ["a.rs ran".to_owned(), "a.rs runs".to_owned()];

    assert!(check_steps(&d, &rows, &allowed).is_empty());
}

#[test]
fn check_steps_refuses_a_listed_entry_that_is_used_or_gone() {
    let d = defs(TWO);
    let rows = [sc(false, &[("given", "a home 1")])];
    let allowed = [
        "a.rs ran".to_owned(),
        "a.rs runs".to_owned(),
        "a.rs home".to_owned(),
        "a.rs deleted".to_owned(),
    ];

    let got = check_steps(&d, &rows, &allowed);

    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got[0].starts_with("a.rs deleted: listed, but no step definition"));
    assert!(got[1].starts_with("a.rs home: listed, but a row that runs"));
}
