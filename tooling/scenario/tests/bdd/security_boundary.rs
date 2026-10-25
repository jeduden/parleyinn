//! The steps of SEC-15, SEC-17, SEC-19 and ENG-22: the boundary
//! register and the processes it lists, no telemetry or update check,
//! the threat model and every privacy statement's test. Each reads the
//! repository checkout through the `engineering` crate's checks, which
//! the gate tests of `tooling/engineering/tests/boundary.rs` run too.
//! SEC-01 stays pending until the B1 components exist; its core row is
//! those gate tests. SEC-17, SEC-19 and ENG-22 stay pending too: these
//! bindings check part of each (phase 15's findings say which part is
//! missing), and they run as tripwires while the rest is built.

use cucumber::{given, then, when};
use engineering::boundary::{self, Row};
use engineering::review_workflow;
use engineering::{
    Checkout, confinement, outcome, privacy, static_analysis, telemetry, threat_model,
};

use crate::World;

/// What the steps track: the checkout, the register's rows and the
/// problems a check found.
#[derive(Debug, Default)]
pub struct Operations {
    checkout: Option<Checkout>,
    rows: Vec<Row>,
    problems: Vec<String>,
}

fn ops(w: &mut World) -> &mut Operations {
    w.section::<Operations>()
}

fn checkout(w: &mut World) -> Result<Checkout, String> {
    ops(w)
        .checkout
        .clone()
        .ok_or_else(|| "no step named the repository".to_owned())
}

/// Names the repository the steps inspect.
fn inspect(w: &mut World) {
    ops(w).checkout = Some(Checkout::new(&testkit::repo_root()));
}

#[given(regex = r"^the boundary register kept in the repository$")]
fn the_register(w: &mut World) -> Result<(), String> {
    inspect(w);
    let rows = boundary::register(&checkout(w)?)?;
    ops(w).rows = rows;
    Ok(())
}

#[when(regex = r"^the CI boundary check runs$")]
fn boundary_check_runs(w: &mut World) -> Result<(), String> {
    let c = checkout(w)?;
    let flow = review_workflow::parse(&c.read(static_analysis::CI_WORKFLOW)?);
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest = c.root.join("Cargo.toml");
    let metadata = engineering::dependencies::cargo_metadata(&cargo, &manifest, &w.parinn_home)?;
    ops(w).problems = boundary::check(&c, &flow, &metadata);
    Ok(())
}

#[then(
    regex = r"^every ParleyInn component, process and protocol is assigned to exactly one of B0, B1, B2 and B3$"
)]
fn assigned_to_one(w: &mut World) -> Result<(), String> {
    let rows = ops(w).rows.clone();
    outcome(boundary::assigned_once(&rows))?;
    outcome(ops(w).problems.clone())
}

#[then(
    regex = r"^the check fails when a component's build-time reach evidence is missing, or when that evidence or a test of a component confined to its boundary shows more reach than its row grants$"
)]
fn fails_without_evidence(_w: &mut World) -> Result<(), String> {
    // A checkout with no clippy.toml, reach test or CI jobs: the core's
    // evidence is missing, and the check says so.
    let empty = testkit::TempDir::new().map_err(|e| e.to_string())?;
    let bare = Checkout::new(empty.path());
    let flow = review_workflow::parse("");
    let problems = boundary::evidence(&bare, &flow, &["parinn".to_owned()]);
    if problems.iter().any(|p| p.contains("clippy.toml")) {
        return Ok(());
    }
    Err(format!(
        "the check passed a core with no evidence: {problems:?}"
    ))
}

#[then(regex = r"^the check fails when a process exists that the register does not list$")]
fn fails_on_an_unlisted_process(w: &mut World) -> Result<(), String> {
    let rows = ops(w).rows.clone();
    let problems = boundary::unlisted_processes(&["rogue".to_owned()], &rows);
    if problems.len() == 1 && problems[0].contains("rogue") {
        return Ok(());
    }
    Err(format!("an unlisted process went unnamed: {problems:?}"))
}

#[then(
    regex = r"^every ParleyInn feature whose purpose is to reach off the machine through a tool the person runs outside ParleyInn, such as its own tunnel, has its own row, stays off by default and turns on only by a widening principal act$"
)]
fn tunnel_row(w: &mut World) -> Result<(), String> {
    // Off by default is the register's; that only a widening principal
    // act turns a row on is SEC-01 and SEC-22's, which no M1 component
    // yet needs to show.
    let rows = ops(w).rows.clone();
    outcome(boundary::tunnel_has_its_own_row(&rows))?;
    outcome(boundary::off_until_on(&rows))
}

#[given(
    regex = r"^the import graph and code of every component, whether the components ship in one executable or several$"
)]
fn import_graph(w: &mut World) {
    inspect(w);
}

#[when(regex = r"^the CI telemetry check runs$")]
fn telemetry_check_runs(w: &mut World) -> Result<(), String> {
    let c = checkout(w)?;
    ops(w).problems = telemetry::check(&c);
    Ok(())
}

#[then(regex = r"^no telemetry, crash-reporting, or update-check code or dependency is found$")]
fn no_telemetry(w: &mut World) -> Result<(), String> {
    outcome(ops(w).problems.clone())
}

#[then(
    regex = r"^the full test suite, with each component confined to its boundary, records no connection opened or tried outside that boundary$"
)]
fn suite_confined(w: &mut World) -> Result<(), String> {
    // Every start of the core in a test goes through the confinement,
    // which fails the test on a socket (ENG-12).
    outcome(confinement::unconfined_starts(&checkout(w)?))
}

#[given(regex = r"^the repository at a minor release tag$")]
fn at_a_minor_release(w: &mut World) {
    inspect(w);
}

#[when(regex = r"^the release checklist is inspected$")]
fn checklist_inspected(w: &mut World) -> Result<(), String> {
    let c = checkout(w)?;
    ops(w).problems = threat_model::check(&c);
    Ok(())
}

/// The problems of the threat model that `keep` selects.
fn threat_problems(w: &mut World, keep: impl Fn(&str) -> bool) -> Result<(), String> {
    let found: Vec<String> = ops(w)
        .problems
        .iter()
        .filter(|p| keep(p))
        .cloned()
        .collect();
    outcome(found)
}

#[then(
    regex = r"^a threat-model document exists in the repository with a recorded review naming the current minor release$"
)]
fn threat_model_reviewed(w: &mut World) -> Result<(), String> {
    let c = checkout(w)?;
    c.read(threat_model::MODEL)?;
    threat_problems(w, |p| p.contains("review") || p.contains("version"))
}

#[then(
    regex = r"^it covers every row of the boundary register and the threat sources acting across each boundary$"
)]
fn threat_model_covers(w: &mut World) -> Result<(), String> {
    threat_problems(w, |p| {
        p.contains("register row") || p.contains("threat sources") || p.contains("read ")
    })
}

#[then(
    regex = r"^it compares ParleyInn with Zed Delta control by control on record signing, trust in other principals, central-service dependence and key custody$"
)]
fn threat_model_compares(w: &mut World) -> Result<(), String> {
    threat_problems(w, |p| p.contains("comparison"))
}

#[given(regex = r"^the documentation$")]
fn the_documentation(w: &mut World) {
    inspect(w);
}

#[when(regex = r"^every privacy or data-flow statement is listed$")]
fn statements_listed(w: &mut World) -> Result<(), String> {
    let c = checkout(w)?;
    let found = privacy::all(&c)?;
    if found.is_empty() {
        return Err("the documentation holds no privacy statement to list".to_owned());
    }
    let scenarios = privacy::scenario_states(&c.root.join("features"))?;
    ops(w).problems = privacy::proven(&c, &scenarios);
    Ok(())
}

#[then(regex = r"^each one references a test that exists and passes$")]
fn each_cites_a_test(w: &mut World) -> Result<(), String> {
    outcome(ops(w).problems.clone())
}
