//! The gates on the real specification under docs/srs: each reads the
//! repository checkout and fails, naming every disagreement, when a
//! table drifts from what it summarises. ENG-27's drift cases prove
//! each one fails when it should.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn srs_dir() -> PathBuf {
    testkit::repo_root().join("docs/srs")
}

fn read(name: &str) -> String {
    fs::read_to_string(srs_dir().join(name)).unwrap()
}

/// Every requirement table under docs/srs reads cleanly, and every id
/// is unique across the whole document.
#[test]
fn specification_parses() {
    let reqs = srs::load(&srs_dir()).unwrap();

    assert_eq!(
        reqs.len(),
        239 + 15 + 30 + 22,
        "§5–§6, NFR, ENG and ASM rows"
    );
}

/// Appendix B says it is generated from the Traces column of §5–§6, so
/// an edit to a trace that forgets the appendix, or the other way
/// round, fails here.
#[test]
fn appendix_b_matches_the_traces() {
    let reqs = srs::load(&srs_dir()).unwrap();
    let body = read("appendix-b-invariant-coverage.md");
    let stated = srs::invariant_coverage(&body).unwrap();
    let traced = srs::traced_coverage(&reqs);

    let mut problems = Vec::new();
    if stated.len() != 10 {
        problems.push(format!(
            "Appendix B has rows for {:?}, want every invariant I1–I10",
            stated.keys()
        ));
    }
    let invariants: BTreeSet<&String> = stated.keys().chain(traced.keys()).collect();
    for invariant in invariants {
        let sorted = |m: &BTreeMap<String, Vec<String>>| {
            let mut ids = m.get(invariant).cloned().unwrap_or_default();
            ids.sort();
            ids
        };
        let (want, got) = (sorted(&traced), sorted(&stated));
        if want != got {
            problems.push(format!(
                "Appendix B row {invariant} lists {got:?}, the traces give {want:?}"
            ));
        }
    }
    let (stated_counts, counted) = (
        srs::stated_counts(&body).unwrap(),
        srs::priority_counts(&reqs),
    );
    if stated_counts != counted {
        problems.push(format!(
            "Appendix B's requirement counts {stated_counts:?} disagree with the requirements' {counted:?}"
        ));
    }

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Every requirement of §5–§7 and §10 serves at least one persona,
/// every id Appendix C lists is a real requirement, every persona it
/// names is one §2.5 defines, and every persona of §2.5 serves some
/// requirement.
#[test]
fn appendix_c_covers_every_requirement() {
    let reqs = srs::load(&srs_dir()).unwrap();
    let coverage = srs::persona_coverage(&read("appendix-c-persona-coverage.md")).unwrap();
    let defined = srs::persona_agents(&read("02-context.md")).unwrap();

    let mut problems = Vec::new();
    let known: BTreeSet<&str> = reqs
        .iter()
        .map(|r| r.id.as_str())
        .filter(|id| !id.starts_with("ASM-"))
        .collect();
    for id in &known {
        if !coverage.contains_key(*id) {
            problems.push(format!("{id} serves no persona in Appendix C"));
        }
    }
    let mut served = BTreeSet::new();
    for (id, personas) in &coverage {
        if !known.contains(id.as_str()) {
            problems.push(format!("Appendix C lists {id}, which is no requirement"));
        }
        for p in personas {
            if !defined.contains_key(p) {
                problems.push(format!("Appendix C names {p}, which §2.5 does not define"));
            }
            served.insert(p);
        }
    }
    for p in defined.keys().filter(|p| !served.contains(p)) {
        problems.push(format!("persona {p} serves no requirement"));
    }

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// §2.5 and .claude/agents stay in step: every persona names an agent
/// file that exists, and every persona agent file is named by a
/// persona.
#[test]
fn personas_match_the_agents() {
    let named: BTreeSet<String> = srs::persona_agents(&read("02-context.md"))
        .unwrap()
        .into_values()
        .collect();
    let on_disk: BTreeSet<String> = fs::read_dir(testkit::repo_root().join(".claude/agents"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter_map(|name| name.strip_suffix(".md").map(str::to_owned))
        .filter(|name| name.starts_with("persona-"))
        .collect();

    assert_eq!(
        named, on_disk,
        "§2.5 names {named:?}, but .claude/agents holds {on_disk:?}"
    );
}
