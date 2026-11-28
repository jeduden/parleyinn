//! The steps of SEC-12, RCL-06 and REC-17 that phase 14 of plan
//! 2609292012 carries, and of RCL-08's clause on an address that was
//! purged or quarantined: `parinn quarantine` holds events out of
//! recall at once, a purged range answers a tombstone, and a purged
//! event leaves nothing that confirms a guess at it.
//!
//! The person's commands are run at a terminal, as an act needs, and a
//! widening act on a node with a run needs the standing risk acceptance
//! (OWN-22): the first erasing command of a scenario records it, as a
//! person would before a purge. M1 has no landmark, restore block or
//! export, so the steps that name them show that nothing is there to
//! carry a quarantined event, and say so.
//!
//! `@SEC-12` stays pending on its `--flag` row: no event carries a flag
//! before PRV-07 (M5), so its store cannot be built.

use std::collections::BTreeSet;

use ::record::id::{Address, WriterId};
use ::record::range::Range;
use administration::node::Node;
use administration::purge::{self, Planned, Reason};
use cucumber::{given, then};
use key::random::OsEntropy;
use serde_json::{Value, json};
use store::Store;

use crate::World;
use crate::record::{home, last_run, person_command, run_core_at_terminal};
use crate::record_search::{call_tool, last_reply};

/// What the erasure steps track.
#[derive(Debug, Default)]
pub struct ErasureSteps {
    /// Whether the standing risk acceptance was recorded.
    pub(crate) accepted: bool,
    /// SEC-12: the selector as the scenario writes it, and what the
    /// words it uses for ids stand for.
    pub(crate) selector: Option<String>,
    pub(crate) placeholders: Vec<(String, String)>,
    /// SEC-12: the arguments the quarantine was added with.
    pub(crate) added: Vec<String>,
    /// REC-17: what the purged events held before the purge.
    pub(crate) purged: Vec<Original>,
    /// REC-17: the writer whose events were purged.
    pub(crate) writer: Option<WriterId>,
    /// RCL-06: the tombstones (range, reason) and quarantine markers the
    /// recall showed over all its pages.
    pub(crate) followed: Option<Shown>,
    /// RCL-06: the seqs a retention policy purged.
    pub(crate) retained: Option<(u64, u64)>,
}

/// The tombstones (range, reason) and quarantine markers of a recall.
pub(crate) type Shown = (Vec<(String, String)>, Vec<Value>);

/// An event as it stood before a purge.
#[derive(Debug, Clone)]
pub(crate) struct Original {
    pub(crate) seq: u64,
    pub(crate) content: Value,
    pub(crate) key: String,
}

pub(crate) fn er(w: &mut World) -> &mut ErasureSteps {
    w.section::<ErasureSteps>()
}

/// Splits `text` at white space, a part in single quotes kept whole.
pub(crate) fn words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut current, mut quoted, mut any) = (String::new(), false, false);
    for c in text.chars() {
        match c {
            '\'' => {
                quoted = !quoted;
                any = true;
            }
            c if c.is_whitespace() && !quoted => {
                if any {
                    out.push(std::mem::take(&mut current));
                    any = false;
                }
            }
            c => {
                current.push(c);
                any = true;
            }
        }
    }
    if any {
        out.push(current);
    }

    out
}

/// Runs the person's command `text`. A command that erases or
/// quarantines is run at a terminal, once the runs it names are
/// recorded and the open residual risks accepted.
pub fn person_runs(w: &mut World, text: &str) -> Result<(), String> {
    let args = words(text);
    let erasing = args.first().is_some_and(|v| v == "purge")
        || (args.first().is_some_and(|v| v == "quarantine")
            && args.get(1).is_some_and(|v| v != "list"));
    if !erasing {
        return person_command(w, text);
    }
    crate::recall::record_runs(w)?;
    accept_risks(w)?;
    let args: Vec<String> = args
        .into_iter()
        .map(|a| {
            er(w)
                .placeholders
                .iter()
                .find(|(shown, _)| *shown == a)
                .map_or(a, |(_, real)| real.clone())
        })
        .collect();
    let shown: Vec<&str> = args.iter().map(String::as_str).collect();
    run_core_at_terminal(w, &shown)
}

#[given(regex = r#"^the person runs "parinn ([^"]*)"$"#)]
fn given_person_runs(w: &mut World, text: String) -> Result<(), String> {
    person_runs(w, &text)?;
    // A command a scenario takes as given has to have worked.
    expect_success(w, &format!("parinn {text}"))
}

/// Records the standing risk acceptance once, as a person does before a
/// widening act on a node that has runs (OWN-22).
pub(crate) fn accept_risks(w: &mut World) -> Result<(), String> {
    if er(w).accepted {
        return Ok(());
    }
    run_core_at_terminal(w, &["risk-acceptance", "record"])?;
    let out = last_run(w)?;
    if !out.status.success() {
        return Err(format!(
            "parinn risk-acceptance record exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    er(w).accepted = true;

    Ok(())
}

pub(crate) fn open(w: &World) -> Result<Store, String> {
    Store::open(&home(w)?.store()).map_err(|e| e.to_string())
}

/// The writer the label `A1`, or another, names.
pub(crate) fn writer_of(w: &World, label: &str) -> Result<WriterId, String> {
    let store = open(w)?;
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let labels = store
        .writer_labels(node.device_writer())
        .map_err(|e| e.to_string())?;
    let want = ::record::range::Label::parse(label).ok_or("not a label")?;

    labels
        .into_iter()
        .find(|(_, l)| *l == want)
        .map(|(w, _)| w)
        .ok_or_else(|| format!("no writer holds the label {label}"))
}

/// What a recall of every event of `writer` showed: the seqs returned as
/// items, the tombstones, as seq ranges with their reasons, and the
/// quarantine markers, as seq ranges alone (SEC-12, RCL-06).
pub(crate) struct Recalled {
    pub(crate) items: BTreeSet<u64>,
    pub(crate) marks: Vec<(u64, u64, String)>,
    pub(crate) markers: Vec<(u64, u64)>,
}

impl Recalled {
    /// Whether a quarantine marker covers `seq`: a tombstone that names
    /// the word "quarantined" does not.
    pub(crate) fn quarantined(&self, seq: u64) -> bool {
        self.markers
            .iter()
            .any(|(from, to)| *from <= seq && seq <= *to)
    }
}

/// Proves the quarantine check reads markers only: a recall showing a
/// tombstone of the reason "quarantined" holds nothing back, and one
/// showing a marker does.
pub(crate) fn marker_is_not_a_tombstone() -> Result<(), String> {
    let mut seen = Recalled {
        items: BTreeSet::new(),
        marks: vec![(1, 1, "quarantined".to_owned())],
        markers: Vec::new(),
    };
    if seen.quarantined(1) {
        return Err("a tombstone naming \"quarantined\" passed for a quarantine marker".to_owned());
    }
    seen.markers.push((1, 1));
    if !seen.quarantined(1) {
        return Err("a quarantine marker was not read as one".to_owned());
    }
    Ok(())
}

/// Recalls every event of `writer` with `event_expand`, following the
/// cursor to the end.
pub(crate) fn recall_all(w: &mut World, writer: WriterId) -> Result<Recalled, String> {
    let head = open(w)?
        .head(writer)
        .map_err(|e| e.to_string())?
        .ok_or("the writer holds no event")?;
    let mut arguments = json!({"range": format!("{writer}:1-{head}")});
    let mut out = Recalled {
        items: BTreeSet::new(),
        marks: Vec::new(),
        markers: Vec::new(),
    };
    for _ in 0..200 {
        let answer = call_tool(w, "event_expand", arguments)?;
        if answer.is_error {
            return Err(format!("event_expand was refused: {}", answer.text));
        }
        let envelope: Value = serde_json::from_str(&answer.text).map_err(|e| e.to_string())?;
        for item in envelope["items"].as_array().into_iter().flatten() {
            let (address, _) = Address::parse(item["address"].as_str().unwrap_or_default())
                .map_err(|e| e.to_string())?;
            out.items.insert(address.seq.get());
        }
        for mark in envelope["tombstones"].as_array().into_iter().flatten() {
            let range = Range::parse(mark["range"].as_str().unwrap_or_default())
                .map_err(|e| e.to_string())?;
            let reason = mark["reason"].as_str().unwrap_or_default().to_owned();
            out.marks.push((range.from.get(), range.to.get(), reason));
        }
        // A quarantine marker is the range alone, kept apart from the
        // tombstones.
        for mark in envelope["quarantine_markers"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let range = Range::parse(mark["range"].as_str().unwrap_or_default())
                .map_err(|e| e.to_string())?;
            out.markers.push((range.from.get(), range.to.get()));
        }
        match envelope["next_cursor"].as_str() {
            Some(cursor) => arguments = json!({"cursor": cursor}),
            None => return Ok(out),
        }
    }

    Err("event_expand never reached the end of the writer".to_owned())
}

// ---------------------------------------------------------------- RCL-06

#[then(
    regex = r#"^no item has address (\S+) and no item lies in (\S+), which appears as a tombstone with reason "([^"]+)"$"#
)]
fn quarantined_and_purged(
    w: &mut World,
    quarantined: String,
    purged: String,
    reason: String,
) -> Result<(), String> {
    let held = Range::parse(&quarantined).map_err(|e| e.to_string())?;
    let gone = Range::parse(&purged).map_err(|e| e.to_string())?;
    let ::record::range::Writer::Label(label) = held.writer else {
        return Err("a label was wanted".to_owned());
    };
    let writer = writer_of(w, &label.to_string())?;
    // The agent follows the cursor until the range it asked for is
    // covered: the cap holds a page to a few events.
    let (seqs, marks, markers) = follow(w)?;
    er(w).followed = Some((marks.clone(), markers));
    if let Some(seq) = seqs.iter().find(|(wr, s)| {
        *wr == writer && (*s == held.from.get() || (gone.from.get()..=gone.to.get()).contains(s))
    }) {
        return Err(format!("an item has the address of a held event: {seq:?}"));
    }
    if !marks.contains(&(purged.clone(), reason.clone())) {
        return Err(format!(
            "no tombstone {purged:?} of reason {reason:?} among {marks:?}"
        ));
    }

    Ok(())
}

#[then(
    regex = r"^(\S+) appears as a quarantine marker carrying no reason, and not as a tombstone$"
)]
fn quarantine_marker_alone(w: &mut World, address: String) -> Result<(), String> {
    // The pages were followed by the step before: the cursor is spent.
    let (marks, markers) = er(w)
        .followed
        .clone()
        .ok_or("no step followed the recall to its end")?;
    if !markers.iter().any(|m| m == &json!({"range": address})) {
        return Err(format!(
            "no quarantine marker of the range alone for {address:?} among {markers:?}"
        ));
    }
    let held = Range::parse(&address).map_err(|e| e.to_string())?;
    for (range, _) in &marks {
        let shown = Range::parse(range).map_err(|e| e.to_string())?;
        if shown.from <= held.to && held.from <= shown.to {
            return Err(format!(
                "{address:?} also appears as the tombstone {range:?}"
            ));
        }
    }

    Ok(())
}

/// A retention policy has purged `range`, as the node does on its
/// schedule (REC-15): the purge is made in the store, the schedule
/// itself is SEC-22's.
#[given(regex = r"^a retention policy has purged (\S+)$")]
fn retention_purged(w: &mut World, range: String) -> Result<(), String> {
    let parsed = Range::parse(&range).map_err(|e| e.to_string())?;
    let ::record::range::Writer::Label(label) = parsed.writer else {
        return Err("a label was wanted".to_owned());
    };
    let writer = writer_of(w, &label.to_string())?;
    let mut store = open(w)?;
    let node = Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let why = Reason::Retention {
        policy: "retention_policy.*.web".to_owned(),
    };
    let planned =
        purge::plan(&store, writer, parsed.from, parsed.to, &why).map_err(|e| e.to_string())?;
    let Planned::Ready(plan) = planned else {
        return Err("the retention purge was refused".to_owned());
    };
    purge::carry_out(&mut store, &node, &plan, None, &mut OsEntropy).map_err(|e| e.to_string())?;
    er(w).retained = Some((parsed.from.get(), parsed.to.get()));

    Ok(())
}

/// The recall shows the range a retention policy purged as a tombstone
/// of the given reason, and no item in it.
#[then(regex = r#"^a retention-purged range appears as a tombstone with reason "([^"]+)"$"#)]
fn retention_tombstone(w: &mut World, reason: String) -> Result<(), String> {
    let (from, to) = er(w)
        .retained
        .ok_or("no step had a retention policy purge")?;
    let (marks, _) = er(w)
        .followed
        .clone()
        .ok_or("no step followed the recall to its end")?;
    let mut shown = Vec::new();
    for (range, why) in &marks {
        let range = Range::parse(range).map_err(|e| e.to_string())?;
        if range.from.get() <= to && from <= range.to.get() {
            shown.push((range.from.get(), range.to.get(), why.as_str()));
        }
    }
    if shown != [(from, to, reason.as_str())] {
        return Err(format!(
            "the retention-purged {from}-{to} shows {shown:?}, wanting one tombstone of reason {reason:?}"
        ));
    }

    Ok(())
}

/// The items and tombstones of the last recall and of the pages its
/// cursor goes on to, in order: each item as its writer and seq, each
/// tombstone as its range and reason.
/// The items (writer, seq), tombstones (range, reason) and quarantine
/// markers (the whole entry) of a recall.
pub(crate) type Followed = (Vec<(WriterId, u64)>, Vec<(String, String)>, Vec<Value>);

pub(crate) fn follow(w: &mut World) -> Result<Followed, String> {
    let (mut seqs, mut marks, mut markers) = (Vec::new(), Vec::new(), Vec::new());
    let mut answer = last_reply(w)?;
    for _ in 0..200 {
        if answer.is_error {
            return Err(format!("the recall was refused: {}", answer.text));
        }
        let envelope: Value = serde_json::from_str(&answer.text).map_err(|e| e.to_string())?;
        for item in envelope["items"].as_array().into_iter().flatten() {
            let (address, _) = Address::parse(item["address"].as_str().unwrap_or_default())
                .map_err(|e| e.to_string())?;
            seqs.push((address.writer, address.seq.get()));
        }
        for mark in envelope["tombstones"].as_array().into_iter().flatten() {
            marks.push((
                mark["range"].as_str().unwrap_or_default().to_owned(),
                mark["reason"].as_str().unwrap_or_default().to_owned(),
            ));
        }
        markers.extend(
            envelope["quarantine_markers"]
                .as_array()
                .into_iter()
                .flatten()
                .cloned(),
        );
        let Some(cursor) = envelope["next_cursor"].as_str() else {
            return Ok((seqs, marks, markers));
        };
        answer = call_tool(w, "event_expand", json!({"cursor": cursor}))?;
    }

    Err("the cursor never reached the end of the range".to_owned())
}

/// Purges events `from` to `to` of the run whose transcript is `name`,
/// as a person does: the standing risk acceptance first.
pub fn purge_run_events(w: &mut World, name: &str, from: u64, to: u64) -> Result<(), String> {
    let writer = open(w)?
        .ingested_runs()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|r| r.path.ends_with(&format!("{name}.jsonl")))
        .ok_or_else(|| format!("{name} was not ingested"))?
        .writer;
    accept_risks(w)?;
    run_core_at_terminal(w, &["purge", "--range", &format!("{writer}:{from}-{to}")])?;

    expect_success(w, "parinn purge")
}

// ---------------------------------------------------------------- RCL-08

/// The address range the step showed, from RCL-08's steps: the writer,
/// its seqs and the text shown.
#[derive(Debug, Clone)]
pub struct ShownRange {
    pub writer: WriterId,
    pub from: u64,
    pub to: u64,
}

/// An address of a quarantined event resolves to a quarantine marker and
/// one of a purged event to a tombstone, wherever the address is taken
/// (RCL-08): the shown events are quarantined, then purged.
pub fn resolves_to_markers(
    w: &mut World,
    shown: &ShownRange,
    call: &dyn Fn(&mut World) -> Result<(), String>,
) -> Result<(), String> {
    accept_risks(w)?;
    let first = format!("{}:{}-{}", shown.writer, shown.from, shown.from);
    run_core_at_terminal(w, &["quarantine", "add", "--range", &first])?;
    expect_success(w, "parinn quarantine add")?;
    call(w)?;
    let held = envelope_marks(w)?;
    if !held.contains(&("quarantined".to_owned(), shown.from)) {
        return Err(format!(
            "the quarantined address resolves to {held:?}, wanting a quarantine marker"
        ));
    }
    let all = format!("{}:{}-{}", shown.writer, shown.from, shown.to);
    run_core_at_terminal(w, &["purge", "--range", &all])?;
    expect_success(w, "parinn purge")?;
    call(w)?;
    let gone = envelope_marks(w)?;
    let covered: BTreeSet<u64> = (shown.from..=shown.to).collect();
    let shown_gone: BTreeSet<u64> = gone
        .iter()
        .filter(|(reason, _)| reason == "purged")
        .map(|(_, seq)| *seq)
        .collect();
    if shown_gone != covered {
        return Err(format!(
            "the purged addresses resolve to {gone:?}, wanting a tombstone for each of {covered:?}"
        ));
    }
    if !last_reply(w)?.items().is_empty() {
        return Err("a purged address still returns an item".to_owned());
    }

    Ok(())
}

/// The reason and first seq of each seq a tombstone of the last recall
/// covers.
pub(crate) fn envelope_marks(w: &mut World) -> Result<Vec<(String, u64)>, String> {
    let answer = last_reply(w)?;
    if answer.is_error {
        return Err(format!("the call was refused: {}", answer.text));
    }
    let envelope: Value = serde_json::from_str(&answer.text).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for mark in envelope["tombstones"].as_array().into_iter().flatten() {
        let range =
            Range::parse(mark["range"].as_str().unwrap_or_default()).map_err(|e| e.to_string())?;
        let reason = mark["reason"].as_str().unwrap_or_default();
        out.extend((range.from.get()..=range.to.get()).map(|s| (reason.to_owned(), s)));
    }
    for mark in envelope["quarantine_markers"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let range =
            Range::parse(mark["range"].as_str().unwrap_or_default()).map_err(|e| e.to_string())?;
        out.extend((range.from.get()..=range.to.get()).map(|s| ("quarantined".to_owned(), s)));
    }

    Ok(out)
}

pub(crate) fn expect_success(w: &mut World, what: &str) -> Result<(), String> {
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }

    Err(format!(
        "{what} exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}
