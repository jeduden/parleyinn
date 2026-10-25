use super::*;
use std::path::PathBuf;

fn req(id: &str, text: &str) -> srs::Requirement {
    srs::Requirement {
        id: id.into(),
        priority: "P0".into(),
        verify: "T".into(),
        traces: vec!["I1".into()],
        text: text.into(),
        path: "docs/srs/05.md".into(),
        line: 3,
    }
}

fn sc(id: &str, pending: bool) -> scenario::Scenario {
    scenario::Scenario {
        path: PathBuf::from("features/record.feature"),
        line: 7,
        name: format!("{id} works"),
        id: id.into(),
        priority: "P0".into(),
        invariants: vec!["I1".into()],
        pending,
        steps: vec!["a step".into()],
    }
}

const ADR: &str = "docs/adr/ADR-2610111200-invariant.md";

fn record(status: &str, body: &str) -> String {
    format!(
        "---\nid: ADR-2610111200\ntitle: \"t\"\nstatus: {status}\nsummary: >-\n  s\n---\n# ADR-2610111200: t\n\n{body}\n"
    )
}

fn base() -> Snapshot {
    Snapshot {
        requirements: vec![
            req("REC-01", "Ingest."),
            req("REC-02", "Link."),
            req("PRV-10", "Certify."),
            req(
                "ENG-29",
                "A change to an invariant, or to a requirement marked for I2 review (PRV-10, PRV-11), MUST land with an ADR.",
            ),
        ],
        scenarios: vec![sc("REC-01", false), sc("REC-02", true), sc("PRV-10", true)],
        codeowners: "/docs/srs/ @owner\n/tooling/ @owner\n".into(),
        invariants: "I1".into(),
        decisions: BTreeMap::new(),
    }
}

fn header(path: &str) -> String {
    format!("diff --git a/{path} b/{path}\n")
}

#[test]
fn class_names_are_the_ones_the_workflow_reads() {
    assert_eq!(
        [Class::Light, Class::Standard, Class::Deep].map(Class::name),
        ["light", "standard", "deep"]
    );
}

#[test]
fn analyse_routes_by_the_paths_that_changed() {
    let cases = [
        (vec!["docs/guide.md", "README.md"], Class::Light),
        (vec!["plan/x/plan.md", "README.md"], Class::Standard),
        (vec!["docs/domain-model/run.md"], Class::Standard),
        (vec!["features/notes.md"], Class::Standard),
        (vec![], Class::Standard),
        (
            vec!["plan/x/plan.md", "crates/parinn/src/lib.rs"],
            Class::Standard,
        ),
        (vec!["features/record.feature"], Class::Standard),
        (vec!["README.md", "docs/srs/05.md"], Class::Deep),
        (vec!["tooling/drift/src/cases.rs"], Class::Deep),
    ];
    for (paths, want) in cases {
        let diff: String = paths.iter().map(|p| header(p)).collect();
        let got = analyse(&base(), &base(), &diff, false);
        assert_eq!(got.class, want, "{paths:?}");
        assert_eq!(got.class == Class::Deep, !got.owned.is_empty(), "{paths:?}");
        assert!(got.unparsed.is_empty(), "{paths:?}");
    }
}

#[test]
fn analyse_finds_nothing_in_an_unchanged_tree() {
    let got = analyse(&base(), &base(), &header("README.md"), false);

    assert!(got.requirements.is_empty());
    assert!(got.scenarios.is_empty());
    assert!(got.touched.is_empty());
    assert!(got.findings.is_empty());
    assert_eq!(got.paths, ["README.md"]);
}

#[test]
fn analyse_reads_added_removed_and_changed_rows() {
    let mut head = base();
    head.requirements[0].text = "Ingest every transcript.".into();
    head.requirements[1].line = 40;
    head.requirements.remove(2);
    head.requirements.push(req("REC-03", "Index."));

    let got = analyse(&base(), &head, "", false);

    assert_eq!(
        got.requirements,
        BTreeMap::from([
            ("PRV-10".to_owned(), Change::Removed),
            ("REC-01".to_owned(), Change::Changed),
            ("REC-03".to_owned(), Change::Added),
        ])
    );
}

#[test]
fn analyse_reads_scenario_changes_and_mentions_into_the_touched_ids() {
    let mut head = base();
    head.scenarios[1].steps.push("another".into());
    let diff = format!(
        "{}+see REC-01 and NFR-99\n",
        header("crates/parinn/src/lib.rs")
    );

    let got = analyse(&base(), &head, &diff, false);

    assert_eq!(
        got.scenarios,
        BTreeMap::from([("REC-02".to_owned(), Change::Changed)])
    );
    assert_eq!(got.touched, ["REC-01", "REC-02"]);
}

#[test]
fn analyse_refuses_a_live_scenario_removed_or_pended_again() {
    let mut head = base();
    head.scenarios[0].pending = true;
    let pended = analyse(&base(), &head, "", false);
    head.scenarios.remove(0);
    let removed = analyse(&base(), &head, "", false);

    assert_eq!(
        pended.findings,
        [Finding {
            path: "features/record.feature".into(),
            line: 7,
            severity: BLOCKING,
            body: "the REC-01 scenario is live on main and this change puts it back on @pending"
                .into(),
        }]
    );
    assert_eq!(removed.findings, [removal("REC-01")]);
}

/// The finding for a live scenario of `base()` that the change removes.
fn removal(id: &str) -> Finding {
    Finding {
        path: "features/record.feature".into(),
        line: 7,
        severity: BLOCKING,
        body: format!("the {id} scenario is live on main and this change removes it"),
    }
}

#[test]
fn analyse_refuses_a_live_scenario_retagged_to_another_id() {
    let mut head = base();
    head.scenarios[0].id = "REC-09".into();
    head.scenarios[0].path = "features/other.feature".into();
    head.scenarios[0].line = 30;

    let got = analyse(&base(), &head, "", false);

    assert_eq!(got.findings, [removal("REC-01")]);
}

#[test]
fn analyse_reads_an_unparsed_header_as_deep_and_names_it() {
    let bad = "diff --git \"a/x\" y";
    let diff = format!("{}{bad}\n", header("README.md"));

    let got = analyse(&base(), &base(), &diff, false);

    assert_eq!(got.class, Class::Deep);
    assert!(got.owned.is_empty());
    assert_eq!(got.unparsed, [bad]);
    assert_eq!(got.paths, ["README.md"]);
    assert_eq!(
        got.findings,
        [Finding {
            path: bad.into(),
            line: 0,
            severity: NIT,
            body: "the paths this diff header names do not read, so the review class is deep"
                .into(),
        }]
    );
}

#[test]
fn analyse_flags_each_file_that_lifts_a_ban() {
    let diff = format!(
        "+#[allow(clippy::disallowed_methods)] before any header\n{}+#[allow(clippy::disallowed_types)]\n+#![expect(clippy::disallowed_methods)]\n-#[allow(clippy::disallowed_types)]\n{}+ a line\n",
        header("crates/parinn/src/a.rs"),
        header("crates/parinn/src/b.rs")
    );

    let got = analyse(&base(), &base(), &diff, false);

    assert_eq!(
        got.findings,
        [Finding {
            path: "crates/parinn/src/a.rs".into(),
            line: 0,
            severity: NIT,
            body: "lifts a reach or determinism ban: weigh its reason".into(),
        }]
    );
}

#[test]
fn lifts_a_ban_reads_the_lint_and_the_groups_that_hold_it() {
    for line in [
        "#[allow(clippy::disallowed_methods)]",
        "#![allow(clippy::style)]",
        "#[expect(clippy::all, reason = \"r\")]",
    ] {
        assert!(lifts_a_ban(line), "{line}");
    }
    for line in [
        "#[allow(clippy::allow_attributes)]",
        "#[allow(clippy::styled)]",
        "x",
    ] {
        assert!(!lifts_a_ban(line), "{line}");
    }
}

#[test]
fn analyse_lets_a_pending_scenario_go_live_or_go() {
    let mut head = base();
    head.scenarios[1].pending = false;
    head.scenarios.remove(2);

    assert!(analyse(&base(), &head, "", false).findings.is_empty());
}

/// The findings and the ENG-29 record when `head` holds `adr` at
/// [`ADR`], the diff changes `paths` and the stakeholder `consent`s.
fn guarded(
    head: &mut Snapshot,
    adr: Option<String>,
    paths: &[&str],
    consent: bool,
) -> (Vec<Finding>, Option<Record>) {
    if let Some(body) = adr {
        head.decisions.insert(ADR.into(), body);
    }
    let diff: String = paths.iter().map(|p| header(p)).collect();
    let got = analyse(&base(), head, &diff, consent);
    (got.findings, got.eng_29_record)
}

/// The ENG-29 finding at `path` with `body`.
fn eng_29_finding(path: &str, body: &str) -> Finding {
    Finding {
        path: path.into(),
        line: 0,
        severity: BLOCKING,
        body: body.into(),
    }
}

/// `base()` with PRV-10's row changed.
fn prv_10_changed() -> Snapshot {
    let mut head = base();
    head.requirements[2].text = "Certify, now differently.".into();
    head
}

#[test]
fn analyse_asks_eng_29_s_record_and_consent_for_an_invariant_change() {
    let (findings, record) = guarded(&mut base(), None, &[INVARIANTS], false);

    assert_eq!(
        findings,
        [eng_29_finding(
            INVARIANTS,
            "this change touches an invariant (ENG-29), but no accepted ADR under docs/adr that this change changes names ENG-29 and invariant; and no stakeholder comment consents to it"
        )]
    );
    assert_eq!(record, None);
}

#[test]
fn analyse_reads_an_invariant_change_from_the_trees_whatever_the_diff_names() {
    let mut head = base();
    head.invariants = "I1, rewritten".into();

    let (findings, _) = guarded(&mut head, None, &["docs/notes.md"], true);

    assert_eq!(
        findings,
        [eng_29_finding(
            INVARIANTS,
            "this change touches an invariant (ENG-29), but no accepted ADR under docs/adr that this change changes names ENG-29 and invariant"
        )]
    );
}

#[test]
fn analyse_names_the_invariant_and_the_marked_row_together() {
    let (findings, _) = guarded(
        &mut prv_10_changed(),
        None,
        &[INVARIANTS, "docs/srs/05.md"],
        true,
    );

    assert_eq!(
        findings,
        [eng_29_finding(
            INVARIANTS,
            "this change touches an invariant, PRV-10 (ENG-29), but no accepted ADR under docs/adr that this change changes names ENG-29, invariant and PRV-10"
        )]
    );
}

#[test]
fn analyse_asks_eng_29_s_record_for_a_row_marked_for_i2_review() {
    let mut other = base();
    other.requirements[0].text = "Changed, but not marked.".into();

    let (findings, _) = guarded(&mut prv_10_changed(), None, &["docs/srs/05.md"], true);

    assert_eq!(
        findings,
        [eng_29_finding(
            "docs/srs/05.md",
            "this change touches PRV-10 (ENG-29), but no accepted ADR under docs/adr that this change changes names ENG-29 and PRV-10"
        )]
    );
    assert!(
        guarded(&mut other, None, &["docs/srs/05.md"], false)
            .0
            .is_empty()
    );
}

#[test]
fn analyse_places_a_removed_marked_row_where_main_had_it() {
    let mut head = base();
    head.requirements.remove(2);

    let (findings, _) = guarded(&mut head, None, &[], true);

    assert_eq!(findings[0].path, "docs/srs/05.md");
}

#[test]
fn analyse_is_not_satisfied_by_a_record_without_eng_29() {
    let adr = record(
        "accepted",
        "Approved until a security reviewer is named; invariant PRV-10.",
    );

    let (findings, record) = guarded(
        &mut prv_10_changed(),
        Some(adr),
        &["docs/srs/05.md", ADR],
        true,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(record, None);
}

#[test]
fn analyse_asks_consent_beside_an_accepted_record() {
    let adr = record("accepted", "ENG-29: PRV-10 changes, approved.");

    let (findings, record) = guarded(
        &mut prv_10_changed(),
        Some(adr),
        &["docs/srs/05.md", ADR],
        false,
    );

    assert_eq!(
        findings,
        [eng_29_finding(
            "docs/srs/05.md",
            "this change touches PRV-10 (ENG-29), but no stakeholder comment consents to it"
        )]
    );
    assert_eq!(record, None);
}

#[test]
fn analyse_takes_an_accepted_record_and_consent() {
    let adr = record(
        "accepted",
        "ENG-29: PRV-10 and an invariant change, approved.",
    );

    let (findings, record) = guarded(
        &mut prv_10_changed(),
        Some(adr),
        &[INVARIANTS, "docs/srs/05.md", ADR],
        true,
    );

    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(
        record,
        Some(Record {
            path: ADR.into(),
            summary: "s".into(),
        })
    );
}

#[test]
fn analyse_takes_only_an_accepted_record_naming_every_guarded_id() {
    let paths = [INVARIANTS, "docs/srs/05.md", ADR];
    let count = |body: String| {
        guarded(&mut prv_10_changed(), Some(body), &paths, true)
            .0
            .len()
    };

    assert_eq!(count(record("proposed", "ENG-29 PRV-10 invariant")), 1);
    assert_eq!(count(record("accepted", "ENG-29 invariant")), 1);
    assert_eq!(count(record("accepted", "ENG-29 PRV-10")), 1);
    assert_eq!(count(record("accepted", "ENG-290 PRV-10 invariant")), 1);
    assert_eq!(count("no front matter".into()), 1);
}

#[test]
fn analyse_takes_only_a_record_the_change_touches() {
    let adr = record("accepted", "ENG-29 invariant");

    assert_eq!(
        guarded(&mut base(), Some(adr), &[INVARIANTS], true).0.len(),
        1
    );
}

#[test]
fn analyse_ignores_a_deleted_record() {
    let (findings, record) = guarded(&mut base(), None, &[INVARIANTS, ADR], true);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(record, None);
}

#[test]
fn marked_for_i2_review_reads_eng_29_s_row() {
    assert_eq!(marked_for_i2_review(&base()), ["PRV-10", "PRV-11"]);
    let mut without = base();
    without.requirements.pop();
    assert!(marked_for_i2_review(&without).is_empty());
}

#[test]
fn and_list_joins_with_a_final_and() {
    assert_eq!(and_list(&[]), "");
    assert_eq!(and_list(&["a"]), "a");
    assert_eq!(and_list(&["a", "b", "c"]), "a, b and c");
}
