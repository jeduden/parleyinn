//! The recall evaluation on real data: the first run of the dataset's
//! first seed, ingested and asked through the recall tools in a fresh
//! home, recalls the target share of its values at every depth, and the
//! transcript parser reads every line of it. The whole dataset, every
//! seed, runs through `parinn-eval recall --published`, in release
//! builds in CI.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use parinn_eval::dataset::{self, COMPACTIONS, SEEDS};
use parinn_eval::report::{self, PAGE};
use parinn_eval::{TARGET, evaluate, templates};
use testkit::TempDir;

#[test]
fn the_first_run_of_the_dataset_recalls_its_values_at_every_depth() {
    let templates = templates(&testkit::repo_root()).unwrap();
    let mut runs = dataset::generate(&templates, SEEDS[0]);
    runs.truncate(1);
    let dir = TempDir::new().unwrap();

    let evaluated = evaluate(&runs, SEEDS[0], dir.path()).unwrap();

    assert_eq!(evaluated.unparsed, 0, "every generated line parses");
    for depth in 1..=COMPACTIONS {
        let t = report::tally(&evaluated.outcomes, PAGE, |o| o.depth == depth);
        assert!(t.asked > 0, "depth {depth}");
        assert!(t.recall().unwrap() >= TARGET, "depth {depth}: {t:?}");
    }
    assert!(
        evaluated.outcomes.iter().all(|o| !o.in_context),
        "the summaries hold no value"
    );
}
