use super::*;

fn bench(name: &'static str) -> Benchmark {
    Benchmark {
        name,
        requirement: "NFR-01",
        coverage: Coverage::Target,
        measures: "something",
        args: &[],
        fixture: Fixture::None,
    }
}

#[test]
fn the_suite_has_no_problems() {
    assert!(problems(&SUITE, &RETIRED).is_empty());
}

#[test]
fn problems_names_a_name_the_samples_file_cannot_carry() {
    assert_eq!(
        problems(&[bench("core start")], &[]),
        [r#"benchmark "core start" is not lowercase letters, digits and underscores"#]
    );
}

#[test]
fn problems_names_a_benchmark_registered_twice() {
    assert_eq!(
        problems(&[bench("a"), bench("b"), bench("a")], &[]),
        ["benchmark a is registered twice"]
    );
}

#[test]
fn problems_names_a_benchmark_registered_and_retired() {
    assert_eq!(
        problems(&[bench("a")], &["a"]),
        ["benchmark a is registered and retired"]
    );
}

#[test]
fn ingestion_is_timed_over_a_transcript_in_a_fresh_home() {
    let ingest = SUITE.iter().find(|b| b.requirement == "NFR-04").unwrap();

    assert_eq!(ingest.coverage, Coverage::Target);
    assert_eq!(ingest.args, ["ingest", "--all"]);
    assert_eq!(
        ingest.fixture,
        Fixture::Transcript {
            lines: INGEST_LINES
        }
    );
}

#[test]
fn the_core_start_up_is_a_floor_not_a_hook_handler() {
    let start_up = SUITE.iter().find(|b| b.name == "core_start_up").unwrap();

    assert_eq!(start_up.coverage, Coverage::Floor);
    assert_eq!(start_up.args, ["version"]);
}

#[test]
fn every_nfr_id_eng_15_names_but_nfr_02_nfr_03_and_nfr_04_is_awaiting_a_benchmark_of_real_code() {
    let ids: Vec<&str> = AWAITING.iter().map(|a| a.requirement).collect();

    assert_eq!(ids, ["NFR-01", "NFR-05"]);
    assert!(AWAITING.iter().all(|a| !a.on.is_empty()));
}

#[test]
fn each_hook_handler_is_timed_on_its_hook_input_in_a_fresh_home() {
    let start = SUITE
        .iter()
        .find(|b| b.name == "hook_session_start")
        .unwrap();
    let prompt = SUITE
        .iter()
        .find(|b| b.name == "hook_user_prompt_submit")
        .unwrap();

    assert_eq!(
        (start.requirement, start.coverage, start.args),
        ("NFR-02", Coverage::Target, &["hook", "SessionStart"][..])
    );
    assert_eq!(
        start.fixture,
        Fixture::HookInput {
            hook: "SessionStart"
        }
    );
    assert_eq!(
        (prompt.requirement, prompt.coverage),
        ("NFR-01", Coverage::Floor)
    );
}

#[test]
fn search_is_timed_over_an_ingested_store_fed_one_recall_request() {
    let search = SUITE.iter().find(|b| b.requirement == "NFR-03").unwrap();

    assert_eq!(search.coverage, Coverage::Target);
    assert_eq!(search.args, ["mcp"]);
    assert_eq!(
        search.fixture,
        Fixture::Search {
            lines: SEARCH_LINES,
            prepare: &["ingest", "--all"],
        }
    );
}
