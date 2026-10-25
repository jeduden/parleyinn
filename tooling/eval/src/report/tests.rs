use super::*;

fn outcome(seed: u64, depth: usize, rank: Option<usize>) -> Outcome {
    Outcome {
        seed,
        depth,
        question: format!("q{seed}{depth}"),
        value: "v".into(),
        rank,
        in_context: false,
    }
}

#[test]
fn a_value_counts_within_a_page_that_reaches_its_rank() {
    let o = outcome(1, 1, Some(7));
    assert!(o.within(PAGE));
    assert!(!o.within(SMALL_PAGE));
    assert!(!outcome(1, 1, None).within(PAGE));
}

#[test]
fn a_tally_counts_what_it_keeps_and_has_no_recall_when_empty() {
    let all = [
        outcome(1, 1, Some(1)),
        outcome(1, 2, None),
        outcome(2, 1, Some(9)),
    ];
    assert_eq!(
        tally(&all, PAGE, |o| o.depth == 1),
        Tally { found: 2, asked: 2 }
    );
    assert_eq!(
        tally(&all, SMALL_PAGE, |o| o.depth == 1),
        Tally { found: 1, asked: 2 }
    );
    assert_eq!(Tally::default().recall(), None);
    assert_eq!(Tally::default().interval(), None);
    assert_eq!(Tally { found: 1, asked: 4 }.recall(), Some(0.25));
}

#[test]
fn the_interval_is_wilson_s_and_brackets_the_recall() {
    let (low, high) = Tally {
        found: 72,
        asked: 72,
    }
    .interval()
    .unwrap();
    assert!((low - 0.9493).abs() < 1e-4, "{low}");
    assert!((high - 1.0).abs() < 1e-9, "{high}");
    let (low, high) = Tally {
        found: 63,
        asked: 72,
    }
    .interval()
    .unwrap();
    assert!(
        (low - 0.7791).abs() < 1e-4 && (high - 0.9328).abs() < 1e-4,
        "{low} {high}"
    );
}

#[test]
fn a_depth_below_the_target_is_named() {
    let all = [
        outcome(1, 1, Some(1)),
        outcome(1, 2, None),
        outcome(1, 2, Some(2)),
    ];
    assert_eq!(below(&all, 0.95), [2]);
    assert!(below(&all, 0.5).is_empty());
}

#[test]
fn the_report_gives_each_depth_both_pages_the_context_each_seed_and_every_miss() {
    let mut kept = outcome(2, 1, Some(7));
    kept.in_context = true;
    let all = [outcome(1, 1, Some(1)), outcome(1, 2, None), kept];

    assert_eq!(
        render(1, &[1, 2], &all),
        "recall after compaction, dataset v1, seeds [1, 2], 3 questions, 10 hits read per question\n\
         within 10 hits, depth 1: 2/2 recalled, 100.0% (95% interval 34.2% to 100.0%)\n\
         within 10 hits, depth 2: 0/1 recalled, 0.0% (95% interval 0.0% to 79.3%)\n\
         within 5 hits, depth 1: 1/2 recalled, 50.0% (95% interval 9.5% to 90.5%)\n\
         within 5 hits, depth 2: 0/1 recalled, 0.0% (95% interval 0.0% to 79.3%)\n\
         without recall, in the context after the last compaction, depth 1: 1/2\n\
         without recall, in the context after the last compaction, depth 2: 0/1\n\
         seed 1, depth 1: 1/1\n\
         seed 1, depth 2: 0/1\n\
         seed 2, depth 1: 1/1\n\
         seed 2, depth 2: 0/0\n\
         not recalled: seed 1, depth 2: \"q12\" = \"v\"\n"
    );
}
