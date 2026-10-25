use super::*;
use crate::review_workflow;

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

fn bench(name: &'static str, requirement: &'static str, coverage: Coverage) -> Benchmark {
    Benchmark {
        fixture: benchmark::suite::Fixture::None,
        name,
        requirement,
        coverage,
        measures: "something",
        args: &[],
    }
}

fn awaits(requirement: &'static str) -> Awaiting {
    Awaiting {
        requirement,
        on: "the code it measures",
    }
}

#[test]
fn named_ids_expands_a_range_and_keeps_single_ids() {
    assert_eq!(
        named_ids(
            "A benchmark suite MUST cover NFR-01..05, and NFR-07 and NFR-03.",
            "NFR"
        ),
        ids(&["NFR-01", "NFR-02", "NFR-03", "NFR-04", "NFR-05", "NFR-07"])
    );
}

#[test]
fn named_ids_skips_what_is_not_two_digits() {
    assert_eq!(
        named_ids("NFR-x, NFR-1, NFR-09.., NFR-10..1x", "NFR"),
        ids(&["NFR-09", "NFR-10"])
    );
    assert!(named_ids("ENG-15", "NFR").is_empty());
}

#[test]
fn registered_holds_ids_each_covered_or_awaiting() {
    let suite = [
        bench("a", "NFR-01", Coverage::Target),
        bench("b", "NFR-02", Coverage::Floor),
    ];

    assert!(registered(&ids(&["NFR-01", "NFR-02"]), &suite, &[awaits("NFR-02")]).is_empty());
}

#[test]
fn registered_names_an_id_neither_covered_nor_awaiting() {
    let suite = [bench("b", "NFR-02", Coverage::Floor)];

    assert_eq!(
        registered(&ids(&["NFR-02"]), &suite, &[]),
        ["NFR-02 has no benchmark of real code and is not declared awaiting one"]
    );
}

#[test]
fn registered_names_an_id_both_covered_and_awaiting() {
    let suite = [bench("a", "NFR-01", Coverage::Target)];

    assert_eq!(
        registered(&ids(&["NFR-01"]), &suite, &[awaits("NFR-01")]),
        ["NFR-01 has a benchmark of real code yet is still declared awaiting one"]
    );
}

#[test]
fn registered_names_what_eng_15_does_not_name() {
    let suite = [bench("a", "NFR-09", Coverage::Target)];

    assert_eq!(
        registered(&[], &suite, &[awaits("NFR-08")]),
        [
            "benchmark a registers against NFR-09, which ENG-15 does not name",
            "NFR-08 is declared awaiting a benchmark, but ENG-15 does not name it",
        ]
    );
}

#[test]
fn uncovered_names_each_id_without_a_benchmark_of_real_code() {
    let suite = [
        bench("a", "NFR-01", Coverage::Target),
        bench("b", "NFR-02", Coverage::Floor),
    ];

    assert_eq!(
        uncovered(
            &ids(&["NFR-01", "NFR-02", "NFR-03"]),
            &suite,
            &[awaits("NFR-02")]
        ),
        [
            "NFR-02 has no benchmark of real code yet: it awaits the code it measures",
            "NFR-03 has no benchmark of real code",
        ]
    );
}

const WORKFLOW: &str = "\
jobs:
  benchmark:
    steps:
      - run: |
          \"$bin/benchmark\" run --core \"$bin/parinn\"
          \"$head/benchmark\" gate --base base --head head
  ci:
    needs: [test, benchmark, build]
";

#[test]
fn gated_holds_a_job_that_runs_the_gate_and_that_ci_needs() {
    assert!(gated(&review_workflow::parse(WORKFLOW)).is_empty());
}

#[test]
fn gated_names_a_missing_job() {
    let w = review_workflow::parse("jobs:\n  ci:\n    needs: [benchmark]\n");

    assert_eq!(gated(&w), ["the CI workflow has no job benchmark"]);
}

#[test]
fn gated_names_what_the_job_does_not_run_and_a_ci_job_that_does_not_need_it() {
    let w = review_workflow::parse(
        &WORKFLOW
            .replace("gate --base", "gate --head")
            .replace("benchmark, build", "benchmarks, build"),
    );

    assert_eq!(
        gated(&w),
        [
            r#"job benchmark does not run "benchmark\" gate --base""#,
            "job ci does not need benchmark",
        ]
    );
}

#[test]
fn gated_names_a_missing_ci_job() {
    let w = review_workflow::parse(&WORKFLOW.replace("  ci:\n", "  other:\n"));

    assert_eq!(gated(&w), ["job ci does not need benchmark"]);
}

#[test]
fn the_gate_fails_naming_a_benchmark_slowed_by_more_than_ten_percent() {
    let (base, head) = slowed("core_start_up", 20);

    assert_eq!(gate_fails_naming(&base, &head, "core_start_up"), Ok(()));
}

#[test]
fn gate_fails_naming_refuses_a_gate_that_stayed_within_the_threshold() {
    let (base, head) = slowed("core_start_up", 5);

    let err = gate_fails_naming(&base, &head, "core_start_up").unwrap_err();

    assert!(
        err.starts_with("the gate exited 0 without naming core_start_up as regressed"),
        "{err}"
    );
}

#[test]
fn gate_fails_naming_refuses_a_gate_that_named_another_benchmark() {
    let (base, head) = slowed("other", 20);

    let err = gate_fails_naming(&base, &head, "core_start_up").unwrap_err();

    assert!(
        err.starts_with("the gate exited 1 without naming core_start_up as regressed"),
        "{err}"
    );
}
