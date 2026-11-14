//! The requirement–scenario gate on the real repository.

/// Every SRS requirement has its one scenario under features/, tagged
/// with the row's priority and invariants, and no scenario names an id
/// the SRS lacks. Adding a requirement means adding its scenario,
/// tagged `@pending` until its steps exist, in the same change.
#[test]
fn specification_and_features_agree() {
    let root = testkit::repo_root();
    let reqs = srs::load(&root.join("docs/srs")).unwrap();
    let scenarios = scenario::scenarios(&root.join("features")).unwrap();

    let problems = scenario::check(&reqs, &scenarios);

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The step definitions no row of a scenario that runs uses, written
/// as `<file> <function>`, one per line.
const UNRUN_STEPS: &str = "tests/unrun-steps.txt";

/// Every step definition under tests/bdd is used by a row of a scenario
/// that is not `@pending`, or stands on the list at [`UNRUN_STEPS`]. The
/// list only shrinks: a definition a running row comes to use, or one
/// that is deleted, leaves it in the same change.
#[test]
fn every_step_definition_has_a_running_row() {
    let root = testkit::repo_root();
    let here = root.join("tooling/scenario");
    let scenarios = scenario::scenarios(&root.join("features")).unwrap();
    let defs = scenario::step_definitions(&here.join("tests/bdd")).unwrap();
    let listed = std::fs::read_to_string(here.join(UNRUN_STEPS)).unwrap();
    let allowed: Vec<String> = listed
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect();

    let problems = scenario::check_steps(&defs, &scenarios, &allowed);

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
