use std::fs;

use testkit::TempDir;

use super::*;

fn fid(path: &str) -> Fid {
    Fid::new(path, vec!["ENG-28".to_owned()], "the gate posts unread")
}

fn review(verdict: &str, findings: &[(&str, &[&str])], cost: f64) -> Review {
    Review {
        dir: format!("review-{verdict}"),
        verdict: verdict.to_owned(),
        snapshot: (Some("abc".to_owned()), "inputs".to_owned()),
        findings: findings
            .iter()
            .map(|(agent, paths)| ((*agent).to_owned(), paths.iter().map(|p| fid(p)).collect()))
            .collect(),
        costs: findings
            .iter()
            .map(|(agent, _)| ((*agent).to_owned(), cost))
            .collect(),
        gaps: BTreeSet::new(),
    }
}

#[test]
fn identical_runs_are_stable() {
    let one = review(
        "approve",
        &[("lang-rust", &["a.rs", "b.rs"]), ("review-code", &[])],
        0.5,
    );
    let reviews = [one.clone(), one.clone(), one];

    let s = compare(&reviews);

    assert_eq!(s.reviews, 3);
    assert!(s.same_snapshot);
    assert_eq!(s.mode, "approve");
    assert_eq!((s.flip_rate, s.jaccard), (0.0, 1.0));
    assert!(s.stable && s.within_limits);
    assert_eq!(s.agents[0].agent, "lang-rust");
    assert_eq!(s.agents[0].findings, vec![Some(2), Some(2), Some(2)]);
    assert_eq!(s.agents[0].cost_spread, 0.0);
    assert_eq!(s.agents[1].jaccard, 1.0);
}

#[test]
fn one_verdict_flip_in_three_is_detected_and_past_the_limit() {
    let approve = review("approve", &[("lang-rust", &["a.rs"])], 0.4);
    let reviews = [
        approve.clone(),
        review("request_changes", &[("lang-rust", &["a.rs"])], 0.7),
        approve,
    ];

    let s = compare(&reviews);

    assert_eq!(s.mode, "approve");
    assert_eq!(s.flip_rate, 0.333_333);
    assert!(!s.stable);
    assert!(
        !s.within_limits,
        "a third of the reviews flipping is past {FLIP_RATE_LIMIT}"
    );
    assert_eq!(s.jaccard, 1.0);
    let a = &s.agents[0];
    assert_eq!((a.cost_min, a.cost_max, a.cost_spread), (0.4, 0.7, 0.3));
}

#[test]
fn disjoint_finding_sets_overlap_nowhere() {
    let reviews = [
        review("approve", &[("lang-rust", &["a.rs"])], 0.1),
        review("approve", &[("lang-rust", &["b.rs"])], 0.1),
    ];

    let s = compare(&reviews);

    assert_eq!(s.flip_rate, 0.0);
    assert_eq!(s.jaccard, 0.0);
    assert!(!s.stable && !s.within_limits);
}

#[test]
fn partly_shared_sets_overlap_in_proportion_and_a_missing_agent_is_unstable() {
    let reviews = [
        review(
            "approve",
            &[("lang-rust", &["a.rs", "b.rs"]), ("lang-json", &[])],
            0.1,
        ),
        review("approve", &[("lang-rust", &["a.rs", "c.rs"])], 0.1),
    ];

    let s = compare(&reviews);

    let json = &s.agents[0];
    assert_eq!((json.agent.as_str(), json.reported), ("lang-json", 1));
    assert_eq!(json.findings, vec![Some(0), None]);
    assert_eq!(
        json.jaccard, 0.0,
        "an agent a review missed overlaps nothing"
    );
    assert_eq!(s.agents[1].jaccard, 0.333_333);
    assert_eq!(s.jaccard, 0.166_667);
    assert!(!s.stable);
}

#[test]
fn runs_with_no_findings_are_stable_and_no_runs_measure_nothing() {
    let empty = review("approve", &[], 0.0);
    let s = compare(&[empty.clone(), empty]);
    assert!(s.agents.is_empty());
    assert_eq!(s.jaccard, 1.0);
    assert!(s.stable && s.within_limits);

    let none = compare(&[]);
    assert_eq!(
        (none.reviews, none.flip_rate, none.mode.as_str()),
        (0, 0.0, "")
    );
    assert!(!none.stable && !none.within_limits);
    assert!(none.same_snapshot);
    assert_eq!(none.unmeasured, vec!["there are no reviews".to_owned()]);
}

#[test]
fn runs_of_different_states_are_named_so() {
    let mut other = review("approve", &[], 0.0);
    other.snapshot.0 = Some("def".to_owned());

    let s = compare(&[review("approve", &[], 0.0), other]);

    assert!(!s.same_snapshot);
    assert!(!s.stable && !s.within_limits);
    assert_eq!(s.unmeasured.len(), 1, "{:?}", s.unmeasured);
}

#[test]
fn runs_of_reviews_that_failed_are_unmeasured_never_stable() {
    let failed = review(crate::agent_runs::NO_VERDICT, &[], 0.0);

    let s = compare(&[failed.clone(), failed.clone(), failed]);

    assert_eq!(s.flip_rate, 0.0);
    assert!(!s.stable && !s.within_limits);
    assert_eq!(s.unmeasured.len(), 1);
    assert!(s.unmeasured[0].starts_with("3 of 3 reviews have no verdict"));
}

#[test]
fn a_dispatched_agent_left_in_a_gap_makes_the_runs_unstable() {
    let mut gapped = review("approve", &[("lang-rust", &[])], 0.1);
    gapped.gaps.insert("lang-json".to_owned());
    let plain = review("approve", &[("lang-rust", &[])], 0.1);

    let s = compare(&[gapped.clone(), gapped.clone(), gapped]);
    let json = s.agents.iter().find(|a| a.agent == "lang-json").unwrap();
    assert_eq!((json.reported, json.findings.clone()), (0, vec![None; 3]));
    assert!(!s.stable && !s.within_limits);
    assert_eq!(json.jaccard, 0.0);
    assert_eq!(
        s.unmeasured,
        vec!["no review holds a report or answer of lang-json".to_owned()]
    );

    let mixed = compare(&[plain.clone(), plain]);
    assert!(mixed.stable, "an empty finding set is a report");
}

#[test]
fn the_mode_breaks_a_tie_by_order_and_overlap_of_no_sets_is_one() {
    let v = |s: &[&str]| s.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
    assert_eq!(mode(&v(&["request_changes", "approve"])), "approve");
    assert_eq!(flip_rate(&v(&["request_changes", "approve"])), 0.5);
    assert_eq!(jaccard(&[]), 1.0);
}

#[test]
fn a_review_is_read_from_its_work_directory() {
    let dir = TempDir::new().unwrap();
    let reports = dir.path().join(inputs::REPORTS);
    fs::create_dir_all(&reports).unwrap();
    fs::write(
        reports.join("lang-rust.md"),
        "a.rs:1 — if not addressed: x — I2\noutcome: findings — x\n",
    )
    .unwrap();
    fs::write(reports.join("notes.txt"), "not a report").unwrap();
    fs::create_dir_all(reports.join("broken.md")).unwrap();
    fs::write(
        dir.path().join(inputs::ANSWERS),
        r#"{"lang-rust":"outcome: findings — x","review-code":"a.rs:1 — nit — x — lang-rust"}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join(inputs::USAGE),
        r#"[{"agent":"lang-rust","cost_usd":0.2},{"agent":"review-code","cost_usd":0.3}]"#,
    )
    .unwrap();
    fs::write(
        dir.path().join(inputs::RUN_ID),
        r#"{"head":"abc","inputs":"i"}"#,
    )
    .unwrap();
    let mut notes = Vec::new();

    let r = read(dir.path(), &mut notes);

    assert_eq!(r.verdict, crate::agent_runs::NO_VERDICT);
    assert_eq!(r.snapshot, (Some("abc".to_owned()), "i".to_owned()));
    assert_eq!(r.findings["lang-rust"].len(), 1);
    assert_eq!(r.findings["review-code"].len(), 1);
    assert_eq!(r.costs["review-code"], 0.3);
    assert_eq!(notes.len(), 1, "{notes:?}");

    fs::write(
        dir.path().join(inputs::OUTCOME),
        r#"{"verdict":"approve","findings":[]}"#,
    )
    .unwrap();
    assert_eq!(read(dir.path(), &mut notes).verdict, "approve");
}

#[test]
fn a_gap_list_and_an_unreadable_reports_directory_are_read_and_named() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("review")).unwrap();
    fs::write(dir.path().join(inputs::REPORTS), "a file, not a directory").unwrap();
    fs::write(
        dir.path().join(inputs::GAPS),
        r#"[{"name":"lang-json","kind":"specialist","part":null,"reason":"no report"}]"#,
    )
    .unwrap();
    let mut notes = Vec::new();

    let r = read(dir.path(), &mut notes);

    assert!(r.gaps.contains("lang-json"), "{notes:?}");
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(notes[0].contains("does not read"), "{notes:?}");
}

#[test]
fn a_report_entry_that_does_not_read_is_named_and_the_rest_are_read() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("lang-rust.md"), "text").unwrap();
    let paths = vec![
        Err(io::Error::other("gone")),
        Ok(dir.path().join("lang-rust.md")),
    ];
    let mut notes = Vec::new();
    let mut reports = BTreeMap::new();

    read_reports(paths.into_iter(), dir.path(), &mut notes, &mut reports);

    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(notes[0].ends_with("does not read: gone"), "{notes:?}");
    assert_eq!(reports.get("lang-rust").map(String::as_str), Some("text"));
}

#[test]
fn an_agent_missing_from_one_review_lowers_the_mean_even_with_no_findings() {
    let with = review(
        "approve",
        &[("lang-rust", &["a.rs"]), ("lang-json", &[])],
        0.1,
    );
    let without = review("approve", &[("lang-rust", &["a.rs"])], 0.1);

    let s = compare(&[with.clone(), with, without]);

    let json = s.agents.iter().find(|a| a.agent == "lang-json").unwrap();
    assert_eq!((json.reported, json.jaccard), (2, 0.0));
    assert_eq!(s.jaccard, 0.5);
    assert!(s.unmeasured.is_empty());
    assert!(
        !s.within_limits,
        "absence cannot keep the mean above the floor"
    );
}

#[test]
fn an_agent_in_no_review_makes_the_comparison_unmeasured_and_never_raises_the_mean() {
    let mut gapped = review("approve", &[("lang-rust", &["a.rs"])], 0.1);
    gapped.gaps.insert("lang-json".to_owned());

    let s = compare(&[gapped.clone(), gapped]);

    assert_eq!(s.jaccard, 0.5);
    assert!(!s.within_limits && !s.stable);
    assert_eq!(s.unmeasured.len(), 1);
}

#[test]
fn fewer_reviews_than_expected_measure_nothing_though_they_agree() {
    let one = review("approve", &[("a", &["x.rs"])], 1.0);

    let short = compare_expecting(&[one.clone(), one.clone()], 3);
    assert!(!short.stable && !short.within_limits, "{short:?}");
    assert_eq!(
        short.unmeasured,
        vec!["2 of 3 expected reviews were measured".to_owned()]
    );

    let full = compare_expecting(&[one.clone(), one.clone(), one], 3);
    assert!(full.stable && full.within_limits && full.unmeasured.is_empty());
}
