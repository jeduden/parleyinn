use super::*;
use crate::accepted::Acceptance;
use crate::samples::Samples;

const NAME: &str = "core_start_up";

/// `n` samples near `ns`, each off by a small repeating jitter, as a
/// quiet runner times them.
fn clean(ns: u64, n: u64) -> Vec<u64> {
    (0..n).map(|i| ns + i % 7).collect()
}

/// `samples` each scaled to `percent` of itself.
fn scaled(samples: &[u64], percent: u64) -> Vec<u64> {
    samples.iter().map(|s| s * percent / 100).collect()
}

/// Samples spread wide around 1000 ns, as a noisy runner times them.
fn noisy() -> Vec<u64> {
    vec![
        500, 600, 700, 800, 900, 1000, 1000, 1100, 1200, 1300, 1400, 1500,
    ]
}

/// `samples` as the one stratum of one file.
fn strata(samples: Samples) -> Strata {
    Strata::from([(String::new(), samples)])
}

fn one(name: &str, samples: Vec<u64>) -> Strata {
    strata(Samples::from([(name.to_owned(), samples)]))
}

fn compare(base: Vec<u64>, head: Vec<u64>) -> Delta {
    let mut all = decide(&one(NAME, base), &one(NAME, head), &[]);
    assert_eq!(all.len(), 1);
    all.remove(0)
}

#[test]
fn nine_percent_slower_with_significance_is_within_the_threshold() {
    let base = clean(10_000, 30);
    let head = scaled(&base, 109);
    assert!(
        stats::mann_whitney_z(&base, &head) > Z_CRITICAL,
        "the slowdown must be significant for this case to mean anything"
    );

    let c = compare(base, head);

    assert_eq!(c.status, Status::Within);
    assert!(!c.fails());
}

#[test]
fn eleven_percent_slower_but_noisy_is_within_the_threshold() {
    let base = noisy();
    let head = scaled(&base, 111);
    assert_eq!(
        (stats::twice_median(&base), stats::twice_median(&head)),
        (Some(2000), Some(2220)),
        "the head's median must be 11% slower for this case to mean anything"
    );
    assert!(stats::mann_whitney_z(&base, &head) < Z_CRITICAL);

    let c = compare(base, head);

    assert_eq!(c.status, Status::Within);
    assert!(failures(&[c]).is_empty());
}

#[test]
fn eleven_percent_slower_and_clean_fails_naming_the_benchmark() {
    let base = clean(10_000, 30);
    let head = scaled(&base, 111);

    let c = compare(base, head);

    assert_eq!(c.status, Status::Regressed);
    let failed = failures(&[c]);
    assert_eq!(failed.len(), 1);
    assert!(
        failed[0].starts_with("core_start_up regressed by 11.0%"),
        "{failed:?}"
    );
}

#[test]
fn a_faster_head_is_within_the_threshold() {
    let base = clean(10_000, 30);
    let head = scaled(&base, 80);

    assert_eq!(compare(base, head).status, Status::Within);
}

#[test]
fn exactly_ten_percent_slower_holds_since_the_gate_wants_more() {
    let base = vec![1000; 20];
    let head = vec![1100; 20];

    assert_eq!(compare(base, head).status, Status::Within);
}

#[test]
fn a_benchmark_the_head_did_not_sample_fails() {
    let all = decide(&one(NAME, clean(10_000, 30)), &Strata::new(), &[]);

    assert_eq!(all[0].status, Status::Missing);
    assert_eq!(
        failures(&all),
        ["core_start_up was sampled on the base but not on the head"]
    );
}

#[test]
fn a_benchmark_the_head_retired_is_reported_not_failed() {
    let all = decide(&one(NAME, clean(10_000, 30)), &Strata::new(), &[NAME]);

    assert_eq!(all[0].status, Status::Retired);
    assert!(failures(&all).is_empty());
}

#[test]
fn a_retired_benchmark_the_head_still_samples_is_compared() {
    let base = clean(10_000, 30);
    let head = scaled(&base, 150);

    let all = decide(&one(NAME, base), &one(NAME, head), &[NAME]);

    assert_eq!(all[0].status, Status::Regressed);
}

#[test]
fn a_benchmark_new_on_the_head_is_reported_not_failed() {
    let all = decide(&Strata::new(), &one(NAME, clean(10_000, 30)), &[]);

    assert_eq!(all[0].status, Status::New);
    assert!(failures(&all).is_empty());
}

#[test]
fn too_few_samples_on_either_side_fail() {
    let few = clean(10_000, MIN_SAMPLES as u64 - 1);
    let enough = clean(10_000, MIN_SAMPLES as u64);

    for (base, head) in [(few.clone(), enough.clone()), (enough, few)] {
        let c = compare(base, head);
        assert_eq!(c.status, Status::TooFew);
        assert_eq!(
            failures(&[c]),
            ["core_start_up has too few samples to decide: the gate needs 10 on each side"]
        );
    }
}

#[test]
fn nothing_sampled_on_either_side_fails() {
    assert_eq!(
        failures(&decide(&Strata::new(), &Strata::new(), &[])),
        ["no benchmark was sampled on the base or the head"]
    );
}

#[test]
fn decide_orders_its_comparisons_by_benchmark() {
    let base = strata(Samples::from([
        ("b".to_owned(), clean(10, 10)),
        ("a".to_owned(), clean(10, 10)),
    ]));
    let head = strata(Samples::from([("c".to_owned(), clean(10, 10))]));

    let names: Vec<String> = decide(&base, &head, &["a", "b"])
        .into_iter()
        .map(|c| c.benchmark)
        .collect();

    assert_eq!(names, ["a", "b", "c"]);
}

/// Ten 1000s against ten 1500s: U = 100 against a mean of 50, and the
/// two runs of ten ties give a variance of
/// 100 / 12 * (21 - 1980 / 380), so z = 49.5 / sqrt(131.58) = 4.32.
#[test]
fn the_report_has_a_row_per_benchmark() {
    let base = strata(Samples::from([
        (NAME.to_owned(), vec![1000; 10]),
        ("gone".to_owned(), vec![1000; 10]),
    ]));
    let head = strata(Samples::from([(NAME.to_owned(), vec![1500; 10])]));

    let report = report(&decide(&base, &head, &[]));

    assert_eq!(
        report,
        "| Benchmark | Base median | Head median | Change | z | Result |\n\
         | --- | --- | --- | --- | --- | --- |\n\
         | core_start_up | 1.0 µs | 1.5 µs | +50.0% | 4.32 | regressed |\n\
         | gone | 1.0 µs | – | – | – | missing on the head |\n"
    );
}

#[test]
fn each_status_has_its_words_in_the_report() {
    let words: Vec<&str> = [
        Status::Within,
        Status::Regressed,
        Status::Missing,
        Status::Retired,
        Status::New,
        Status::TooFew,
        Status::Unpaired,
    ]
    .into_iter()
    .map(Status::describe)
    .collect();

    assert_eq!(
        words,
        [
            "within the threshold",
            "regressed",
            "missing on the head",
            "retired",
            "new on the head",
            "too few samples",
            "no common stratum",
        ]
    );
}

/// Three runners of very different speed, each shard of the CI job one
/// of them, the head 12% slower on each: pooled, the spread between
/// runners would hide the slowdown; paired by stratum, the gate sees it.
#[test]
fn the_gate_pairs_each_stratum_s_base_and_head() {
    let side = |slower: u64| {
        [1_000, 3_000, 9_000]
            .into_iter()
            .enumerate()
            .map(|(i, ns)| {
                (
                    i.to_string(),
                    Samples::from([(NAME.to_owned(), clean(ns * slower / 100, 10))]),
                )
            })
            .collect::<Strata>()
    };

    let mut all = decide(&side(100), &side(112), &[]);
    let c = all.remove(0);

    assert_eq!(c.status, Status::Regressed, "{c:?}");
    assert!(c.z.is_some_and(|z| z > Z_CRITICAL));
    assert!(
        c.failure()
            .unwrap()
            .starts_with(&format!("{NAME} regressed by 12.0%"))
    );
}

#[test]
fn a_benchmark_on_no_common_stratum_fails() {
    let on = |stratum: &str| {
        Strata::from([(
            stratum.to_owned(),
            Samples::from([(NAME.to_owned(), clean(1000, 10))]),
        )])
    };

    let c = decide(&on("0"), &on("1"), &[]).remove(0);

    assert_eq!(c.status, Status::Unpaired);
    assert!(c.fails());
    assert_eq!(
        c.failure().unwrap(),
        format!(
            "{NAME} was sampled on no stratum both sides share: the gate pairs each shard's base and head"
        )
    );
}

#[test]
fn the_threshold_compares_medians_exactly_and_saturates() {
    assert!(!slower_than_threshold(2000, 2200));
    assert!(slower_than_threshold(2000, 2201));
    assert!(!slower_than_threshold(u128::MAX, u128::MAX));
}

fn acceptance(name: &str, percent: u128) -> Acceptance {
    Acceptance {
        benchmark: name.to_owned(),
        up_to_percent: percent,
        reason: "a reviewed step".to_owned(),
    }
}

#[test]
fn a_regression_within_a_fresh_acceptance_is_accepted_not_failed() {
    let base = clean(1000, 30);
    let regressed = compare(base.clone(), scaled(&base, 120));

    let got = accept(vec![regressed], &[acceptance("core_start_up", 30)]);

    assert_eq!(got[0].status, Status::Accepted);
    assert!(!got[0].fails());
    assert_eq!(got[0].failure(), None);
    assert_eq!(Status::Accepted.describe(), "accepted");
}

#[test]
fn a_regression_past_its_acceptance_or_of_another_benchmark_still_fails() {
    let base = clean(1000, 30);
    let regressed = compare(base.clone(), scaled(&base, 140));

    let past = accept(vec![regressed.clone()], &[acceptance("core_start_up", 30)]);
    let other = accept(vec![regressed], &[acceptance("other", 50)]);

    assert_eq!(past[0].status, Status::Regressed);
    assert_eq!(other[0].status, Status::Regressed);
}

#[test]
fn an_acceptance_leaves_a_benchmark_within_the_threshold_as_it_is() {
    let base = clean(1000, 30);
    let within = compare(base.clone(), scaled(&base, 105));

    let got = accept(vec![within], &[acceptance("core_start_up", 30)]);

    assert_eq!(got[0].status, Status::Within);
}
