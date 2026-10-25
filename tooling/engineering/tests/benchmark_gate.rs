//! ENG-15's gate tests over the repository checkout: the benchmark
//! suite registers every NFR id ENG-15 names, and the CI workflow runs
//! the suite and the gate in a job the `CI` check needs. Each has a
//! drift case (ENG-27). They run while the ENG-15 scenario is pending,
//! so the gate cannot quietly fall out of CI before the suite covers
//! NFR-01..05.

use benchmark::suite::{AWAITING, SUITE};
use engineering::{Checkout, benchmarks, calibration, outcome, release_budgets, review_workflow};

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn the_benchmark_suite_registers_every_nfr_id_eng_15_names() {
    let eng_15 = checkout().requirement("ENG-15").unwrap();
    let ids = benchmarks::named_ids(&eng_15.text, "NFR");

    assert!(!ids.is_empty(), "ENG-15 names no NFR id");
    outcome(benchmarks::registered(&ids, &SUITE, &AWAITING)).unwrap();
}

#[test]
fn benchmarks_and_timed_steps_run_release_builds() {
    outcome(release_budgets::problems(&checkout())).unwrap();
}

#[test]
fn the_calibration_workload_is_frozen() {
    outcome(calibration::frozen(&checkout().root)).unwrap();
}

#[test]
fn ci_gates_on_the_benchmark_job() {
    let ci = checkout().read(".github/workflows/ci.yml").unwrap();

    outcome(benchmarks::gated(&review_workflow::parse(&ci))).unwrap();
}
