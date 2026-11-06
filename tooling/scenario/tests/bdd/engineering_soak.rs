//! The ENG-09 steps: 50 writers append 10,000 events in all through the
//! store's soak test into an isolated home, no address repeats and no
//! seq is skipped, `parinn verify` then exits 0 over that store, the
//! nightly workflow runs the soak at 1,000,000 events and CI runs Miri
//! over the `platform` crate.

use std::collections::BTreeSet;
use std::process::Command;

use administration::node::Node;
use cucumber::{then, when};
use key::random::OsEntropy;
use store::Store;

use crate::World;
use crate::record::{first_start, last_run, run_core};

/// The soak test's source, the nightly workflow's evidence for the
/// 1,000,000-event claim.
const SOAK: &str = "crates/store/tests/soak.rs";

/// The nightly workflow that runs the soak at 1,000,000 events.
const NIGHTLY: &str = ".github/workflows/nightly.yml";

/// The writers the soak runs, as its source declares them.
const WRITERS: &str = "const WRITERS: u8 = 50;";

/// The lines of the job `name` in a workflow, comments dropped: the
/// job's own line to the next job's.
fn job(workflow: &str, name: &str) -> Option<Vec<String>> {
    let head = format!("  {name}:");
    let mut lines = workflow
        .lines()
        .skip_while(|l| l.trim_end() != head)
        .skip(1)
        .take_while(|l| {
            l.trim().is_empty() || l.starts_with("   ") || l.trim_start().starts_with('#')
        })
        .filter(|l| !l.trim_start().starts_with('#'))
        .map(str::to_owned)
        .peekable();
    lines.peek()?;

    Some(lines.collect())
}

/// Whether some line of `job` holds every one of `parts`.
fn has_line(job: &[String], parts: &[&str]) -> bool {
    job.iter().any(|l| parts.iter().all(|p| l.contains(p)))
}

/// What is wrong with the nightly workflow's soak job, given the
/// workflow's text and the soak test's source.
fn nightly_problems(nightly: &str, soak: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if !soak.contains(WRITERS) {
        problems.push(format!(
            "{SOAK} does not declare {WRITERS:?}: the soak must run 50 writers"
        ));
    }
    let Some(soak_job) = job(nightly, "soak") else {
        problems.push(format!(
            "{NIGHTLY} has no job \"soak\": add it, running \"cargo test --locked --release -p store --test soak\""
        ));
        return problems;
    };
    if !has_line(&soak_job, &["cargo test", "-p store", "--test soak"]) {
        problems.push(format!(
            "the soak job of {NIGHTLY} does not run \"cargo test -p store --test soak\": the soak test is the evidence"
        ));
    }
    if !has_line(&soak_job, &["PARINN_SOAK_EVENTS", "\"1000000\""]) {
        problems.push(format!(
            "the soak job of {NIGHTLY} does not set PARINN_SOAK_EVENTS to \"1000000\": set it, or the 1000000-event claim has no evidence"
        ));
    }
    problems
}

/// What is wrong with CI's Miri job, given the workflow's text.
fn miri_problems(ci: &str) -> Vec<String> {
    let Some(miri) = job(ci, "miri") else {
        return vec![
            ".github/workflows/ci.yml has no job \"miri\": add it, running \"cargo miri test -p platform\""
                .to_owned(),
        ];
    };
    if has_line(&miri, &["miri test", "-p platform"]) {
        return Vec::new();
    }

    vec![
        "the miri job of .github/workflows/ci.yml does not run \"cargo miri test -p platform\": the platform crate holds the unsafe code Miri must run"
            .to_owned(),
    ]
}

/// The store the scenario's home holds.
fn store_path(w: &World) -> std::path::PathBuf {
    w.parinn_home.join("store.db")
}

/// The node's first start in the scenario's home, then 50 ingest seats
/// it certifies, as 50 runs' MCP servers hold: the file listing their
/// writers, one per line, which the soak appends to.
fn certified_writers(w: &World) -> Result<std::path::PathBuf, String> {
    let home = first_start(w)?;
    let mut store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = Node::open(&home, &mut store, &mut OsEntropy).map_err(|e| e.to_string())?;
    let mut ids = String::new();
    for _ in 0..50 {
        let seat = node
            .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
            .map_err(|e| e.to_string())?;
        ids.push_str(&format!("{}\n", seat.writer));
    }
    let file = w.home.join("soak-writers.txt");
    std::fs::write(&file, ids).map_err(|e| e.to_string())?;

    Ok(file)
}

#[when(regex = r"^(\d+) concurrent writers append (\d+) events in all$")]
fn writers_append(w: &mut World, writers: usize, events: usize) -> Result<(), String> {
    if writers != 50 {
        return Err(format!("the soak test runs 50 writers, not {writers}"));
    }
    let writers = certified_writers(w)?;
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let out = Command::new(cargo)
        .args(["test", "--locked", "-p", "store", "--test", "soak"])
        .current_dir(testkit::repo_root())
        .env("PARINN_SOAK_EVENTS", events.to_string())
        .env("PARINN_SOAK_STORE", store_path(w))
        .env("PARINN_SOAK_WRITERS", &writers)
        .output()
        .map_err(|e| format!("run the soak test: {e}"))?;
    if out.status.success() {
        return Ok(());
    }

    Err(format!(
        "the soak test failed, run \"PARINN_SOAK_EVENTS={events} cargo test -p store --test soak -- --nocapture\" to see why: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    ))
}

#[then(
    regex = r"^no two events share an address \(writer, seq\), and no seq is duplicated or skipped within any writer's log$"
)]
fn addresses_and_gaps(w: &mut World) -> Result<(), String> {
    let store = Store::open(&store_path(w)).map_err(|e| e.to_string())?;
    let (mut addresses, mut writers) = (BTreeSet::new(), 0_usize);
    for writer in store.writers().map_err(|e| e.to_string())? {
        let log = store.log(writer).map_err(|e| e.to_string())?;
        for (n, stored) in log.iter().enumerate() {
            let address = stored.link.header.address;
            if address.seq.get() != n as u64 + 1 {
                return Err(format!("{writer} skips or repeats a seq at entry {n}"));
            }
            if !addresses.insert(address) {
                return Err(format!("{address:?} is shared by two events"));
            }
        }
        writers += usize::from(!log.is_empty());
    }
    if writers < 50 {
        return Err(format!("the store holds {writers} writers' logs, not 50"));
    }

    Ok(())
}

#[then(regex = r#"^"parinn verify" exits 0$"#)]
fn verify_exits_zero(w: &mut World) -> Result<(), String> {
    run_core(w, &["verify"], None)?;
    let out = last_run(w)?;
    if out.status.code() == Some(0) {
        return Ok(());
    }

    Err(format!(
        "parinn verify exited {} over the soak's store: {}{}",
        out.status,
        String::from_utf8_lossy(&out.stdout).trim(),
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

#[then(
    regex = r#"^the nightly workflow "([^"]+)" runs the same soak test with 50 writers and the setting "([^"]+)" events, and its result is the evidence for the 1000000-event claim$"#
)]
fn nightly_runs_the_soak(_: &mut World, file: String, events: String) -> Result<(), String> {
    if file != NIGHTLY || events != "1000000" {
        return Err(format!(
            "the step names {file} and {events}, not {NIGHTLY} and 1000000"
        ));
    }
    let checkout = engineering::Checkout::new(&testkit::repo_root());
    let (nightly, soak) = (checkout.read(NIGHTLY)?, checkout.read(SOAK)?);

    engineering::outcome(nightly_problems(&nightly, &soak))
}

#[then(
    regex = r#"^the pull request workflow "([^"]+)" runs Miri over the tests of the "([^"]+)" crate, the one crate holding unsafe code$"#
)]
fn ci_runs_miri(_: &mut World, file: String, krate: String) -> Result<(), String> {
    if file != ".github/workflows/ci.yml" || krate != "platform" {
        return Err(format!(
            "the step names {file} and {krate}, not CI's and platform"
        ));
    }
    let ci = engineering::Checkout::new(&testkit::repo_root()).read(&file)?;

    engineering::outcome(miri_problems(&ci))
}
