use std::collections::BTreeSet;

use super::*;
use crate::agent::holds;
use crate::lines::FIXTURE;

fn templates() -> Templates {
    Templates::from_transcript(
        &std::fs::read_to_string(testkit::repo_root().join(FIXTURE)).unwrap(),
    )
    .unwrap()
}

#[test]
fn a_seed_replays_its_dataset_and_another_seed_makes_another() {
    let t = templates();
    assert_eq!(generate(&t, 7), generate(&t, 7));
    assert_ne!(generate(&t, 7), generate(&t, 8));
}

#[test]
fn every_run_plants_one_fact_of_each_kind_per_depth_with_values_never_repeated() {
    let runs = generate(&templates(), 1);
    assert_eq!(runs.len(), RUNS);
    let mut values = BTreeSet::new();
    for run in &runs {
        assert_eq!(run.facts.len(), COMPACTIONS * KINDS.len());
        for depth in 1..=COMPACTIONS {
            let kinds: Vec<Kind> = run
                .facts
                .iter()
                .filter(|f| f.depth == depth)
                .map(|f| f.kind)
                .collect();
            assert_eq!(kinds.len(), KINDS.len(), "depth {depth}");
            assert!(KINDS.iter().all(|k| kinds.contains(k)), "depth {depth}");
        }
        for fact in &run.facts {
            assert!(values.insert(fact.value.clone()), "{} repeats", fact.value);
        }
    }
}

#[test]
fn a_value_lies_before_as_many_compactions_as_its_depth_and_the_last_context_lacks_it() {
    for run in generate(&templates(), 2) {
        let boundaries: Vec<usize> = run
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.contains("\"compact_boundary\""))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(boundaries.len(), COMPACTIONS);
        assert_eq!(run.boundary, boundaries[COMPACTIONS - 1]);
        for fact in &run.facts {
            let first = run
                .lines
                .iter()
                .position(|l| holds(l, &fact.value))
                .unwrap();
            let after = boundaries.iter().filter(|&&b| b > first).count();
            assert_eq!(after, fact.depth, "{fact:?}");
            assert!(
                !run.after_last_compaction()
                    .iter()
                    .any(|l| holds(l, &fact.value))
            );
        }
    }
}

#[test]
fn a_run_names_its_facts_subjects_again_without_their_values() {
    let runs = generate(&templates(), 3);
    let echoed = runs
        .iter()
        .flat_map(|r| r.lines.iter())
        .filter(|l| l.contains("still the one we found earlier"))
        .count();
    assert!(echoed > 0);
    let summaries: Vec<&String> = runs[0]
        .lines
        .iter()
        .filter(|l| l.contains("\"isCompactSummary\":true"))
        .collect();
    assert_eq!(summaries.len(), COMPACTIONS);
    for fact in runs[0].facts.iter().filter(|f| f.depth == COMPACTIONS) {
        assert!(summaries[0].contains(&fact.question()), "{fact:?}");
        assert!(!holds(summaries[0], &fact.value));
    }
}

#[test]
fn every_kind_of_turn_shows_in_a_dataset() {
    let lines: Vec<String> = generate(&templates(), 1)
        .into_iter()
        .flat_map(|r| r.lines)
        .collect();
    for marker in [
        "tests pass",
        "shows no errors",
        "reports healthy",
        "retry loop is refactored",
        "On ",
        "incident ticket",
        "p95 latency",
        "crash log is at",
        "migration error was",
        "debug port",
        "last good commit is",
    ] {
        assert!(lines.iter().any(|l| l.contains(marker)), "{marker}");
    }
}

#[test]
fn each_kind_s_value_has_its_shape() {
    let mut rng = Rng::new(5);
    let ticket = value_of(Kind::Ticket, "ledger", &mut rng);
    assert!(ticket.starts_with("LED-") && ticket.len() == 9, "{ticket}");
    let latency = value_of(Kind::Latency, "ledger", &mut rng);
    assert!(latency.ends_with(" ms") && latency.len() == 7, "{latency}");
    let log = value_of(Kind::CrashLog, "ledger", &mut rng);
    assert!(
        log.starts_with("/srv/ledger/releases/r") && log.ends_with(".log"),
        "{log}"
    );
    let error = value_of(Kind::MigrationError, "ledger", &mut rng);
    assert!(error.contains(" on ledger shard "), "{error}");
    let port: u64 = value_of(Kind::DebugPort, "ledger", &mut rng)
        .parse()
        .unwrap();
    assert!((20_000..60_000).contains(&port));
    let commit = value_of(Kind::GoodCommit, "ledger", &mut rng);
    assert!(commit.len() == 10 && commit.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn a_question_names_the_component_and_the_kind_s_subject() {
    let fact = Fact {
        kind: Kind::GoodCommit,
        component: "ledger".into(),
        value: "abc".into(),
        depth: 1,
    };
    assert_eq!(fact.question(), "ledger last good commit");
    let subjects: BTreeSet<&str> = KINDS.iter().map(|k| k.subject()).collect();
    assert_eq!(subjects.len(), KINDS.len());
}

#[test]
fn the_generator_s_draws_stay_in_range_and_ids_take_a_session_s_form() {
    let mut rng = Rng::new(9);
    for _ in 0..100 {
        let n = rng.within(3, 7);
        assert!((3..7).contains(&n));
    }
    assert!(["a", "b"].contains(rng.pick(&["a", "b"])));
    let id = rng.uuid();
    let groups: Vec<usize> = id.split('-').map(str::len).collect();
    assert_eq!(groups, [8, 4, 4, 4, 12], "{id}");
}

#[test]
fn a_value_drawn_twice_is_drawn_again() {
    let mut planted = Planted::default();
    let mut rng = Rng::new(1);
    assert_eq!(planted.fresh(&mut rng, |_| "x".to_owned()), "x");
    let mut draws = vec!["y".to_owned(), "x".to_owned()];
    let draw = std::cell::RefCell::new(&mut draws);
    assert_eq!(
        planted.fresh(&mut rng, |_| draw.borrow_mut().pop().unwrap()),
        "y"
    );
}
