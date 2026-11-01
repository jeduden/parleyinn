//! The ENG-27 steps: the drift registry covers every check that keeps
//! the records in step. CI's drift job injects the cases; these steps
//! check the registry itself, so they run without mdsmith.

use cucumber::{then, when};
use engineering::{ci_gates, outcome, registry};

use crate::World;
use crate::engineering::eng;

#[when(regex = r"^the drift cases are read$")]
fn read_drift_cases(w: &mut World) {
    eng(w).drifts = drift::cases();
}

#[then(regex = r#"^the CI workflow runs the drift suite with "([^"]+)"$"#)]
fn ci_runs_the_drift_suite(w: &mut World, cmd: String) -> Result<(), String> {
    eng(w)
        .checkout()?
        .contains(".github/workflows/ci.yml", &cmd)
}

#[then(
    regex = r#"^the CI workflow runs each gate of "([^"]+)" through "([^"]+)", with no gate's command written inline$"#
)]
fn ci_runs_each_gate_from_the_list(
    w: &mut World,
    list: String,
    runner: String,
) -> Result<(), String> {
    if list != ci_gates::GATES || runner != ci_gates::CALL.trim_end() {
        return Err(format!("the step names {list} and {runner}, not CI's"));
    }
    let checkout = eng(w).checkout()?;
    outcome(ci_gates::check_repository(checkout))
}

#[then(regex = r"^every drift case's edit applies to the checkout$")]
fn drifts_apply(w: &mut World) -> Result<(), String> {
    let root = eng(w).checkout()?.root.clone();
    outcome(registry::drifts_apply(&root, &eng(w).drifts))
}

#[then(
    regex = r"^every non-pending scenario that inspects the repository checkout has a drift case guarding its id$"
)]
fn checkout_scenarios_guarded(w: &mut World) -> Result<(), String> {
    let features = eng(w).checkout()?.root.join("features");
    let scenarios = scenario::scenarios(&features).map_err(|e| e.to_string())?;
    outcome(registry::guarded(
        &eng(w).drifts,
        &registry::checkout_scenarios(&scenarios),
    ))
}

#[then(
    regex = r"^the requirement-scenario gate, the Appendix B check and the persona gates each have a drift case$"
)]
fn gate_tests_guarded(w: &mut World) -> Result<(), String> {
    outcome(registry::guarded(&eng(w).drifts, &registry::GATE_TESTS))
}
