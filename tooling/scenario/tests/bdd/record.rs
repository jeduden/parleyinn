//! The steps of SRS §5.1, the record (REC). `@REC-10` is bound: each
//! writer's hash chain, its header and the causal order its cited
//! heads give. The steps every record scenario shares are here too: an
//! isolated home, and the person running a `parinn` command, which
//! starts the built core executable confined to its boundary (ENG-12)
//! with `HOME` and `PARINN_HOME` in the scenario's own home (ENG-14).
//!
//! A record a scenario needs before its command runs is built through
//! the product crates directly, as the core itself would build it.

use std::collections::BTreeMap;
use std::process::Output;

use ::record::canonical;
use ::record::chain::{self, Link};
use ::record::commitment::CommitmentKey;
use ::record::digest::Digest;
use ::record::event::{EventKind, Provenance};
use ::record::header::FIELD_SET;
use ::record::id::{Address, WriterId};
use administration::home::Home;
use administration::node::Node;
use cucumber::{given, then, when};
use key::random::OsEntropy;
use serde_json::json;
use store::{NewEvent, Store};
use testkit::boundary::{Component, Confinement};

use crate::World;

/// What the record steps track: the last command's outcome, and the
/// writers a scenario's record holds with their transcript times.
#[derive(Debug, Default)]
pub struct RecordSteps {
    ran: Option<Output>,
    /// Each writer the scenario built, in the order it was built, with
    /// the wall-clock time its transcript gave each of its events.
    writers: Vec<(WriterId, Vec<String>)>,
    /// Variables a step set in the environment of every command the
    /// scenario starts after it, beside `HOME` and `PARINN_HOME`.
    env: Vec<(String, String)>,
}

fn rec(w: &mut World) -> &mut RecordSteps {
    w.section::<RecordSteps>()
}

/// The scenario's home, as the core finds it.
pub fn home(w: &World) -> Result<Home, String> {
    Home::locate(Some(w.parinn_home.as_os_str()), None).map_err(|e| e.to_string())
}

/// The node's first start in the scenario's home, as the core makes
/// it: the home, the device key and the personal room in `store.db`.
pub fn first_start(w: &World) -> Result<Home, String> {
    let home = home(w)?;
    home.create().map_err(|e| e.to_string())?;
    let mut store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    Node::open(&home, &mut store, &mut OsEntropy).map_err(|e| e.to_string())?;
    Ok(home)
}

#[given(regex = r"^an isolated ParleyInn home$")]
fn isolated_home(w: &mut World) -> Result<(), String> {
    if !w.parinn_home.starts_with(&w.home) || w.parinn_home.exists() {
        return Err(format!(
            "the ParleyInn home {} is not a fresh one inside the scenario's home",
            w.parinn_home.display()
        ));
    }
    Ok(())
}

#[given(
    regex = r"^a record in which two writers have each appended (\d+) events, each having seen the other's log, the second writer's clock running an hour behind$"
)]
fn two_writers(w: &mut World, count: usize) -> Result<(), String> {
    let home = home(w)?;
    home.create().map_err(|e| e.to_string())?;
    let mut store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = Node::open(&home, &mut store, &mut OsEntropy).map_err(|e| e.to_string())?;
    let mut writers = Vec::new();
    for _ in 0..2 {
        let seat = node
            .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
            .map_err(|e| e.to_string())?;
        writers.push((seat.writer, Vec::new()));
    }
    // The writers take turns, so each event after the first cites the
    // other writer's newest. The second writer's clock runs an hour
    // behind the first's: by wall-clock time every one of its events
    // would come first.
    for i in 0..count {
        for (n, (writer, times)) in writers.iter_mut().enumerate() {
            let time = format!("2026-10-14T{:02}:00:{i:02}Z", 12 - n);
            let line = json!({"type": "user", "timestamp": time}).to_string();
            let new = NewEvent {
                kind: EventKind::UnparsedLine,
                provenance: Provenance::Unparsed,
                content: json!({"line": i + 1, "text": line}),
                commitment_key: CommitmentKey::from_bytes(
                    key::random::draw(&mut OsEntropy).map_err(|e| e.to_string())?,
                ),
                transcript_timestamp: None,
            };
            store.append(*writer, &[new]).map_err(|e| e.to_string())?;
            times.push(time);
        }
    }
    rec(w).writers = writers;
    Ok(())
}

/// Starts the built core with `args`, confined, in the scenario's home
/// and with the environment its steps set, `input` on its stdin when
/// given; the outcome is the one `the command exits` and the steps after
/// it read.
pub fn run_core(w: &mut World, args: &[&str], input: Option<&[u8]>) -> Result<(), String> {
    run_core_with(w, args, input, &[])
}

/// [`run_core`] with `extra` set in this one command's environment.
pub fn run_core_with(
    w: &mut World,
    args: &[&str],
    input: Option<&[u8]>,
    extra: &[(&str, &str)],
) -> Result<(), String> {
    let exe = testkit::core_executable()?;
    run_program_with(w, &exe, args, input, extra)
}

/// [`run_core_with`] starting the release build of the core, which
/// ignores every development seam (NFR-02): a step that holds the core
/// to a time budget starts it this way, so it times the build users run.
pub fn run_release_core(w: &mut World, args: &[&str], input: Option<&[u8]>) -> Result<(), String> {
    let exe = testkit::release_core_executable()?;
    run_program_with(w, &exe, args, input, &[])
}

/// [`run_core_with`] starting `program`, such as a script standing in
/// for a harness that starts the core, in the core's confinement.
pub fn run_program_with(
    w: &mut World,
    exe: &std::path::Path,
    args: &[&str],
    input: Option<&[u8]>,
    extra: &[(&str, &str)],
) -> Result<(), String> {
    let core = Confinement::of(Component::Core).map_err(|e| e.to_string())?;
    let mut set = environment(w);
    set.extend(
        extra
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
    );
    let env: Vec<(&str, &str)> = set.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let out = core
        .run_in(exe, args, &w.home, &env, input)
        .map_err(|e| e.to_string())?;
    rec(w).ran = Some(out);
    Ok(())
}

/// Starts the built core with `args` as [`run_core_with`] does, with no
/// input, and kills it with everything it started once `after` passed,
/// as a harness stops a hook handler at its harness timeout. Returns
/// whether it was still running.
pub fn run_core_killed(
    w: &mut World,
    args: &[&str],
    extra: &[(&str, &str)],
    after: std::time::Duration,
) -> Result<bool, String> {
    let core = Confinement::of(Component::Core).map_err(|e| e.to_string())?;
    let exe = testkit::core_executable()?;
    let mut set = environment(w);
    set.extend(
        extra
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
    );
    let env: Vec<(&str, &str)> = set.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    core.run_killed(&exe, args, &w.home, &env, after)
        .map_err(|e| e.to_string())
}

/// [`run_core`] with a pseudo-terminal as the core's standard input and
/// output, as a person at a terminal gives it: what a CLI verb that
/// writes a principal act needs (OWN-12). Its stderr arrives in its
/// stdout.
pub fn run_core_at_terminal(w: &mut World, args: &[&str]) -> Result<(), String> {
    let core = Confinement::of(Component::Core).map_err(|e| e.to_string())?;
    let exe = testkit::core_executable()?;
    let set = environment(w);
    let env: Vec<(&str, &str)> = set.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let out = core
        .run_at_terminal(&exe, args, &w.home, &env)
        .map_err(|e| e.to_string())?;
    rec(w).ran = Some(out);
    Ok(())
}

/// The environment a command starts with beside `HOME` and
/// `PARINN_HOME`: managed policy read from inside the scenario's home,
/// never the machine's (ENG-14), a hook budget no development build
/// reaches on a loaded machine unless a step sets the hook's own
/// (NFR-02), then what the steps set.
pub fn environment(w: &mut World) -> Vec<(String, String)> {
    let managed = w.home.join(MANAGED_POLICY).display().to_string();
    let mut env = vec![
        ("PARINN_TEST_MANAGED_POLICY".to_owned(), managed),
        (HOOK_BUDGET.to_owned(), LENIENT_BUDGET_MS.to_owned()),
    ];
    env.extend(rec(w).env.iter().cloned());
    env
}

/// Where a scenario's managed policy lies, in its home.
pub const MANAGED_POLICY: &str = "managed_policy.toml";

/// The development build's seam for a hook budget in place of each
/// hook's own, in milliseconds.
pub const HOOK_BUDGET: &str = "PARINN_TEST_HOOK_BUDGET_MS";

/// The hook budget every scenario's hook handler gets unless a step sets
/// another.
const LENIENT_BUDGET_MS: &str = "20000";

/// Sets `key` to `value` in the environment of every command the
/// scenario starts from now on.
pub fn set_env(w: &mut World, key: &str, value: &str) {
    rec(w).env.push((key.to_owned(), value.to_owned()));
}

/// The outcome of the command a step last ran.
pub fn last_run(w: &mut World) -> Result<&Output, String> {
    rec(w)
        .ran
        .as_ref()
        .ok_or_else(|| "no step ran a command".to_owned())
}

#[when(regex = r#"^the person runs "parinn ([^"]*)"$"#)]
fn person_runs(w: &mut World, args: String) -> Result<(), String> {
    let args: Vec<&str> = args.split_whitespace().collect();
    run_core(w, &args, None)
}

#[when(regex = r#"^the person runs "parinn ([^"]*)" at a terminal$"#)]
fn person_runs_at_terminal(w: &mut World, args: String) -> Result<(), String> {
    let args: Vec<&str> = args.split_whitespace().collect();
    run_core_at_terminal(w, &args)
}

#[when(
    regex = r#"^the agent's tool call runs "parinn ([^"]*)" with its standard input and output not a terminal$"#
)]
fn tool_call_runs(w: &mut World, args: String) -> Result<(), String> {
    let args: Vec<&str> = args.split_whitespace().collect();
    run_core(w, &args, None)
}

#[then(regex = r"^the command exits (\d+)$")]
fn exits(w: &mut World, code: i32) -> Result<(), String> {
    let out = last_run(w)?;
    if out.status.code() == Some(code) {
        return Ok(());
    }
    Err(format!(
        "the command exited {}, not {code}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

/// The two writers' logs as the store holds them.
fn logs(w: &mut World) -> Result<BTreeMap<WriterId, Vec<Link>>, String> {
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let mut out = BTreeMap::new();
    for (writer, _) in &rec(w).writers {
        let log = store.log(*writer).map_err(|e| e.to_string())?;
        out.insert(*writer, log.into_iter().map(|s| s.link).collect());
    }
    Ok(out)
}

#[then(
    regex = r"^each event's hash is a collision-resistant hash of prev_hash, its writer's previous event, followed by the canonical encoding of its header$"
)]
fn hash_over_prev_and_header(w: &mut World) -> Result<(), String> {
    for (writer, links) in logs(w)? {
        chain::check_writer(writer, &links).map_err(|e| e.to_string())?;
        let mut prev = Digest::ZERO;
        for link in links {
            let h = &link.header;
            if h.prev_hash != prev {
                return Err(format!(
                    "{}: prev_hash is not its writer's previous event's hash",
                    h.address
                ));
            }
            let encoded = canonical::encode(&h.to_value()).map_err(|e| e.to_string())?;
            let want = Digest::of(&[prev.as_bytes(), encoded.as_bytes()]);
            if link.hash != want {
                return Err(format!(
                    "{}: its hash is not SHA-256 over prev_hash and its canonical header",
                    h.address
                ));
            }
            prev = link.hash;
        }
        if prev == Digest::ZERO {
            return Err(format!("writer {writer} holds no event"));
        }
    }
    Ok(())
}

#[then(
    regex = r"^the header contains only the address, kind, provenance class, prev_hash, the heads of the other writers' logs the writer had seen, and the commitment$"
)]
fn header_fields(w: &mut World) -> Result<(), String> {
    let want = [
        "address",
        "commitment",
        "heads",
        "kind",
        "prev_hash",
        "provenance",
    ];
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    if store
        .meta("header_field_set")
        .map_err(|e| e.to_string())?
        .as_deref()
        != Some(FIELD_SET)
    {
        return Err("the store's meta does not record the header's field set".to_owned());
    }
    let times: Vec<String> = rec(w).writers.iter().flat_map(|(_, t)| t.clone()).collect();
    let record = logs(w)?;
    let writers: Vec<WriterId> = rec(w).writers.iter().map(|(writer, _)| *writer).collect();
    for (n, &writer) in writers.iter().enumerate() {
        let other = &record[&writers[1 - n]];
        for (i, link) in record[&writer].iter().enumerate() {
            let value = link.header.to_value();
            let fields: Vec<&str> = value
                .as_object()
                .into_iter()
                .flatten()
                .map(|(k, _)| k.as_str())
                .collect();
            if fields != want {
                return Err(format!(
                    "{}: the header holds {fields:?}",
                    link.header.address
                ));
            }
            let encoded = canonical::encode(&value).map_err(|e| e.to_string())?;
            if times.iter().any(|t| encoded.contains(t.as_str())) {
                return Err(format!(
                    "{}: the header carries content",
                    link.header.address
                ));
            }
            // The writers took turns, the first writer first: its event
            // i had seen the other's event i - 1, and the second's event
            // i the first's event i. Each cites that head, by address
            // and hash, and nothing of its own writer.
            let seen = if n == 0 { i.checked_sub(1) } else { Some(i) };
            if let Some(head) = seen.and_then(|j| other.get(j)) {
                let cited = link
                    .header
                    .heads
                    .iter()
                    .any(|s| s.address == head.header.address && s.head == head.hash);
                if !cited {
                    return Err(format!(
                        "{}: the header does not cite {}, the other writer's head it had seen",
                        link.header.address, head.header.address
                    ));
                }
            }
            if link.header.heads.iter().any(|s| s.address.writer == writer) {
                return Err(format!(
                    "{}: the header cites its own writer",
                    link.header.address
                ));
            }
        }
    }
    Ok(())
}

#[then(
    regex = r"^events of the two writers are ordered by the heads each cites, never by wall-clock time$"
)]
fn causal_not_wall_clock(w: &mut World) -> Result<(), String> {
    let record = logs(w)?;
    let writers = rec(w).writers.clone();
    // The order the writers took turns in: the first's event i, then
    // the second's.
    let turns: Vec<Address> = (0..writers[0].1.len())
        .flat_map(|i| writers.iter().map(move |(writer, _)| (*writer, i)))
        .filter_map(|(writer, i)| record[&writer].get(i).map(|l| l.header.address))
        .collect();
    // The order `parinn verify` reports, of the two writers' events.
    let ours: Vec<WriterId> = writers.iter().map(|(writer, _)| *writer).collect();
    let order: Vec<Address> = verified_order(w)?
        .into_iter()
        .filter(|a| ours.contains(&a.writer))
        .collect();
    if order != turns {
        return Err(format!(
            "parinn verify orders the {} events otherwise than their cited heads do",
            turns.len()
        ));
    }
    if chain::causal_order(&record).map_err(|e| e.to_string())? != order {
        return Err("the record's causal order is not the one parinn verify reports".to_owned());
    }
    let mut by_clock: Vec<(String, Address)> = writers
        .iter()
        .flat_map(|(writer, times)| {
            times
                .iter()
                .cloned()
                .zip(record[writer].iter().map(|l| l.header.address))
        })
        .collect();
    by_clock.sort();
    let clock: Vec<Address> = by_clock.into_iter().map(|(_, a)| a).collect();
    if clock == order {
        return Err(
            "wall-clock time gives the same order, so the scenario proves nothing".to_owned(),
        );
    }

    Ok(())
}

/// The causal order `parinn verify --json` reports, run confined.
fn verified_order(w: &World) -> Result<Vec<Address>, String> {
    let core = Confinement::of(Component::Core).map_err(|e| e.to_string())?;
    let exe = testkit::core_executable()?;
    let out = core
        .run(&exe, &["verify", "--json"], &w.home)
        .map_err(|e| e.to_string())?;
    let report: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("parinn verify --json: {e}"))?;
    let order = report["order"]
        .as_array()
        .ok_or_else(|| format!("parinn verify --json reports no order: {report}"))?;
    order
        .iter()
        .map(|a| {
            let text = a.as_str().ok_or("an address in the order is no string")?;
            Address::parse(text)
                .map(|(address, _)| address)
                .map_err(|e| e.to_string())
        })
        .collect()
}
