use super::*;

#[test]
fn a_consequence_normalises_its_case_markup_spaces_and_closing_punctuation() {
    assert_eq!(
        normalise("  The **Gate**  posts\tan `unread` report.  "),
        "the gate posts an unread report"
    );
    assert_eq!(normalise("** x"), "x");
}

#[test]
fn ids_are_the_words_shaped_as_requirement_ids_or_invariants() {
    assert_eq!(
        ids("breaks ENG-28, I2 and (SEC-01); not eng-1, I, X-, -1 or Ix"),
        vec!["ENG-28", "I2", "SEC-01"]
    );
}

#[test]
fn a_finding_s_identity_ignores_its_line_its_wording_s_case_and_its_ids_order() {
    let a = of_report("a.rs:10 — if not addressed: The gate fails. — I2, ENG-28\n");
    let b = of_report("a.rs:99 — if not addressed: the gate  fails — ENG-28, I2\n");
    let other = of_report("a.rs:10 — if not addressed: the gate passes — ENG-28, I2\n");

    assert_eq!(a.len(), 1);
    assert_eq!(a, b);
    assert_ne!(a, other);
    let fid = a.iter().next().unwrap();
    assert_eq!(fid.path, "a.rs");
    assert_eq!(fid.ids, vec!["ENG-28", "I2"]);
    assert_eq!(fid.consequence.len(), 64);
}

#[test]
fn a_report_s_requests_and_revisions_are_findings_and_its_prose_is_not() {
    let report = "Some prose.\n\
        b.rs:1 — request: add a requirement for x — because\n\
        revised: c.rs:2 — if not addressed: y — I3\n\
        withdrawn: d.rs:3 — wrong\n\
        outcome: findings — two\n";

    let fids = of_report(report);

    let paths: Vec<&str> = fids.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["b.rs", "c.rs"]);
    assert_eq!(
        fids.iter().next().unwrap(),
        &Fid::new("b.rs", Vec::new(), "request: add a requirement for x")
    );
}

#[test]
fn a_generalist_s_kept_findings_are_identified_by_what_each_breaks() {
    let answer = "a.rs:3 — blocking — the gate posts unread — lang-rust (ENG-28)\n\
        dropped — b.rs:1 — duplicate — lang-rust\n\
        none — nothing to add — lang-json\n";

    let fids = of_answer(answer);

    assert_eq!(
        fids.into_iter().collect::<Vec<_>>(),
        vec![Fid::new(
            "a.rs",
            vec!["ENG-28".to_owned()],
            "the gate posts unread"
        )]
    );
}

#[test]
fn a_review_s_findings_are_each_specialist_s_report_and_each_generalist_s_answer() {
    let reports = BTreeMap::from([(
        "lang-rust".to_owned(),
        "a.rs:1 — if not addressed: x — I2\noutcome: findings — x\n".to_owned(),
    )]);
    let answers = BTreeMap::from([
        ("lang-rust".to_owned(), "outcome: findings — x".to_owned()),
        (
            "review-code".to_owned(),
            "a.rs:1 — nit — x — lang-rust".to_owned(),
        ),
    ]);

    let review = of_review(&reports, &answers);

    assert_eq!(review.len(), 2);
    assert_eq!(review["lang-rust"].len(), 1);
    assert_eq!(review["review-code"].len(), 1);
}

#[test]
fn a_revision_replaces_the_finding_it_revises_and_a_withdrawal_removes_it() {
    let one = |line: &str| of_report(&format!("{line}\n"));
    let revised = of_report(
        "a.rs:1 — if not addressed: x — I1\n\
         b.rs:2 — if not addressed: kept — I2\n\
         revised: a.rs:1 — if not addressed: y — I3\n",
    );
    assert_eq!(
        revised,
        BTreeSet::from([
            Fid::new("a.rs", vec!["I3".to_owned()], "y"),
            Fid::new("b.rs", vec!["I2".to_owned()], "kept"),
        ])
    );

    let withdrawn = of_report(
        "a.rs:1 — if not addressed: x — I1\n\
         withdrawn: a.rs:1 — wrong\n\
         b.rs:2 — request: add a requirement for z — why\n",
    );
    assert_eq!(
        withdrawn,
        one("b.rs:2 — request: add a requirement for z — why")
    );

    // A revision of a place nothing stood at is a finding; a withdrawal
    // of one removes nothing; a finding stated after its withdrawal stands.
    let fresh = of_report(
        "withdrawn: z.rs:9 — nothing\n\
         revised: a.rs:1 — if not addressed: y — I3\n\
         a.rs:1 — if not addressed: y — I3\n\
         revised: a.rs:1 — request: change r — because\n",
    );
    assert_eq!(fresh, one("a.rs:1 — request: change r — because"));
    let again = of_report(
        "a.rs:1 — if not addressed: x — I1\nwithdrawn: a.rs:1 — w\na.rs:1 — if not addressed: x — I1\n",
    );
    assert_eq!(again.len(), 1);
}

#[test]
fn a_revision_that_is_no_finding_changes_nothing() {
    let report = "a.rs:1 — if not addressed: x — I1\nrevised: a.rs:1 — no finding here\n";

    assert_eq!(of_report(report).len(), 1);
}
