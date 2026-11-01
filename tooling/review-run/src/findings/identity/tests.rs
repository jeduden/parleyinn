use super::*;

fn finding(path: &str, line: i64, body: &str) -> Finding {
    Finding {
        path: path.to_owned(),
        line,
        severity: "blocking".to_owned(),
        body: body.to_owned(),
    }
}

#[test]
fn a_consequence_normalises_its_case_markup_spaces_and_closing_punctuation() {
    assert_eq!(
        normalise("  The **Gate**  posts\tan `unread` report.  "),
        "the gate posts an unread report"
    );
    assert_eq!(normalise("** x"), "x");
}

#[test]
fn ids_are_requirement_ids_and_invariants_alone() {
    assert_eq!(
        ids("breaks ENG-28, I2 and I10 — not lang-rust, I, ENG-, -2, Ix or eng-1"),
        ["ENG-28", "I2", "I10"]
    );
}

#[test]
fn a_finding_s_identity_ignores_its_line_and_its_source() {
    let a = finding("src/a.rs", 3, "the gate posts — ENG-28, I2 — lang-rust");
    let b = finding(
        "src/a.rs",
        90,
        "The **gate** posts. — I2, ENG-28 — test-engineer-unit",
    );

    assert_eq!(of_finding(&a), of_finding(&b));
    assert_eq!(of_finding(&a).ids, ["ENG-28", "I2"]);
    assert_eq!(of_finding(&a).consequence.len(), 64);
}

#[test]
fn another_path_ids_or_consequence_is_another_finding() {
    let a = of_finding(&finding("src/a.rs", 3, "x — I2"));
    for other in [
        finding("src/b.rs", 3, "x — I2"),
        finding("src/a.rs", 3, "x — I3"),
        finding("src/a.rs", 3, "y — I2"),
    ] {
        assert_ne!(a, of_finding(&other), "{other:?}");
    }
    assert_eq!(
        Fid::new("p", vec!["I2".to_owned(), "I2".to_owned()], "x").ids,
        ["I2"]
    );
}
