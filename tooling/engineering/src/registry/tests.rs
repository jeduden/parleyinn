use super::*;
use crate::tests::checkout;
use drift::Injection;

fn guarding(guards: &'static str, injection: Injection) -> Case {
    Case {
        name: "c",
        guards,
        injection,
        check: drift::mdsmith(),
        want: "w",
        known_gap: false,
    }
}

fn sc(id: &str, pending: bool, steps: &[&str]) -> Scenario {
    Scenario {
        id: id.into(),
        pending,
        steps: steps.iter().map(|s| (*s).to_owned()).collect(),
        ..Scenario::default()
    }
}

#[test]
fn drifts_apply_names_a_case_that_rotted() {
    let (dir, _) = checkout(&[("a.md", "one\n")]);
    let ok = guarding("g", Injection::replace("a.md", "one", "1"));
    assert!(drifts_apply(dir.path(), std::slice::from_ref(&ok)).is_empty());

    let rotted = Case {
        name: "rotted",
        ..guarding("g", Injection::replace("a.md", "two", "2"))
    };
    assert_eq!(
        drifts_apply(dir.path(), &[ok, rotted]),
        [r#"drift case "rotted": drift: a.md: "two" not found"#]
    );
}

#[test]
fn checkout_scenarios_are_the_bound_ones_opening_on_the_checkout() {
    let scenarios = [
        sc("ENG-01", false, &[CHECKOUT_STEP, "x"]),
        sc("ENG-02", true, &[CHECKOUT_STEP]),
        sc("ENG-03", false, &["an isolated ParleyInn home"]),
        sc("ENG-04", false, &[]),
    ];

    assert_eq!(checkout_scenarios(&scenarios), ["ENG-01"]);
}

#[test]
fn guarded_names_what_has_no_case() {
    let cases = [guarding("a", Injection::remove("x"))];

    assert!(guarded(&cases, &["a"]).is_empty());
    assert_eq!(guarded(&cases, &["a", "b"]), ["b has no drift case"]);
}
