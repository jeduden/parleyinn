//! The ENG-09 steps: 50 writers append 10,000 events in all through the
//! store's soak test into an isolated home, no address repeats and no
//! seq is skipped, `parinn verify` then exits 0 over that store, the
//! nightly workflow is set to run the soak at 1,000,000 events, no
//! crate but the one unsafe shim can hold unsafe code and CI runs Miri
//! over that crate.

use std::collections::BTreeSet;
use std::process::Command;

use administration::node::Node;
use cucumber::{then, when};
use engineering::soak::{CI, EVENTS, NIGHTLY, SOAK, miri_problems, nightly_problems};
use engineering::static_analysis::crates_forbid_unsafe;
use key::random::OsEntropy;
use store::Store;

use crate::World;
use crate::record::{first_start, last_run, run_core};

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
    regex = r#"^the nightly workflow "([^"]+)" is set to run the same soak test with 50 writers and "([^"]+)" events$"#
)]
fn nightly_runs_the_soak(_: &mut World, file: String, events: String) -> Result<(), String> {
    if file != NIGHTLY || events != EVENTS {
        return Err(format!(
            "the step names {file} and {events}, not {NIGHTLY} and {EVENTS}"
        ));
    }
    let checkout = engineering::Checkout::new(&testkit::repo_root());
    let (nightly, soak) = (checkout.read(NIGHTLY)?, checkout.read(SOAK)?);

    engineering::outcome(nightly_problems(&nightly, &soak))
}

#[then(
    regex = r#"^every unsafe block that shares memory between threads runs under Miri or ThreadSanitizer in the pull request workflow "([^"]+)"$"#
)]
fn ci_runs_miri(_: &mut World, file: String) -> Result<(), String> {
    if file != CI {
        return Err(format!("the step names {file}, not {CI}"));
    }
    let checkout = engineering::Checkout::new(&testkit::repo_root());
    let ci = checkout.read(&file)?;
    // Every unsafe block is the one shim's while no other crate can
    // switch the ban off, and Miri runs that shim's tests: the row's
    // "Miri or ThreadSanitizer", met by Miri.
    let mut problems = crates_forbid_unsafe(&checkout);
    problems.extend(miri_problems(&ci));

    engineering::outcome(problems)
}
