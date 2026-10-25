//! One ingest of one transcript source: where to resume it (REC-03,
//! REC-07), which run each new line belongs to (REC-02), what becomes
//! of a repeated line (REC-04), and the appends that move the ingest
//! position with the lines they cover.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use administration::home::Home;
use administration::node::Node;
use key::certificate::SeatCertificate;
use key::random::{Entropy, draw};
use provenance::{Classified, Classifier, Session};
use record::commitment::CommitmentKey;
use record::digest::{Digest, Running};
use record::event::{EventKind, Provenance};
use record::id::{Address, WriterId};
use redaction::Redactor;
use redaction::account::HarnessAccountIdentifiers;
use serde_json::{Value, json};
use store::{Held, IngestedRun, NewEvent, Place, Placed, Position, Source, Store};
use transcript::shape::Shapes;

use crate::live::RunSeats;
use crate::observed::{self, Observed};
use crate::prompt::{self, Outcome, PromptLine};
use crate::read::{self, Line};
use crate::repeat::{self, Decision, Facts, Owner, Rule};
use crate::run::{Defaults, RunKey};
use crate::{IngestError, Ingested, Inputs, RunReport, content, turn_end, well_formed};

/// Who records what an ingest reads, and in which seats.
pub(crate) enum Recorder<'r> {
    /// `parinn ingest`: each run in an ingest seat whose key the core
    /// keeps (REC-22).
    Cli,
    /// The harness session's MCP server: each run in a run seat whose
    /// key it holds in memory (REC-13, SEC-10).
    Live(&'r mut RunSeats),
}

/// Ingests the transcript at `path`, whose text is `text`, as
/// [`crate::ingest_with`] describes, `recorder` recording it.
pub(crate) fn ingest(
    home: &Home,
    path: &Path,
    text: &str,
    redactor: &Redactor,
    inputs: &Inputs,
    recorder: &mut Recorder<'_>,
    entropy: &mut dyn Entropy,
) -> Result<Ingested, IngestError> {
    let lines = read::lines(text);
    let parsed: Vec<Value> = lines
        .iter()
        .map(|l| serde_json::from_str(l.text).unwrap_or(Value::Null))
        .collect();
    let mut ids = HarnessAccountIdentifiers::default();
    lines.iter().for_each(|l| ids.learn(l.text));
    let redactor = redactor
        .with_identifiers(&ids)
        .map_err(IngestError::Redaction)?;
    home.create().map_err(IngestError::Admin)?;
    let mut store =
        Store::open(&home.store())?.with_payload_threshold(inputs.payload.payload_threshold);
    let node = Node::open(home, &mut store, entropy).map_err(IngestError::Admin)?;
    let observed = Observed::load(&store)?;
    let mut inputs = inputs.clone();
    inputs.forks.extend(observed.forks());
    let inputs = &inputs;
    let named = Named::of(path, &redactor);
    let resume = Resume::find(&mut store, &named, text.as_bytes(), &lines, &redactor)?;
    let defaults = Defaults::of(path);
    let mut runs = Runs::load(&store, &named.path, recorder)?;
    let own = runs.ensure(&mut store, &node, home, &defaults.own(), entropy)?;
    let mut planner = Planner::new(&store, &named, inputs, &resume.source, &runs)?;
    let plan = planner.plan(
        &mut store,
        &mut runs,
        &observed.session(&defaults.own().harness_session),
        Read {
            node: &node,
            home,
            lines: &lines,
            parsed: &parsed,
            from: resume.index,
            defaults: &defaults,
            redactor: &redactor,
        },
        entropy,
    )?;
    let main = defaults.own().agent_id.is_none().then_some(own);
    let seen = Seen {
        observed: &observed,
        session: &defaults.own().harness_session,
        main,
        interactive: inputs.interactive,
        live: live_observations(&observed, &defaults, main, runs.live()),
    };
    let landed = append(&mut store, &resume, text.as_bytes(), &plan, &seen, entropy)?;
    let (appended, addresses) = (landed.lines, landed.addresses);
    let hold_started = hold(&mut store, &resume.source, plan.hold, inputs.now_ms)?;
    let owners_chosen = choose_owners(&mut store, &node, &plan.chosen, entropy)?;
    let observations = record_observed(&mut store, &node, &seen, &plan, &addresses, entropy)?;

    let (index_gaps, index_gaps_unread) = gaps(store.unreported_search_index_gaps());

    Ok(Ingested {
        index_gaps,
        index_gaps_unread,
        witnessed_repeats: plan.witnessed_repeats,
        matched: observations.matched,
        prompt_conflicts: observations.conflicts,
        not_attended: observations.not_attended,
        observations: observations.appended + landed.observations,
        writer: own,
        seat: seat_of(&store, own)?,
        room: node.personal_room(),
        runs: reports(&store, own, &plan.events_by_writer(), &runs)?,
        events: appended,
        generation: resume.source.generation,
        new_generation: resume.new_generation,
        held: plan.hold,
        hold_started,
        released: plan.released,
        references: plan.references,
        conflicts: plan.conflicts,
        owners_chosen,
        redactions: plan.redactions,
        unparsed: plan.unparsed,
        drifts: plan.drifts,
        redacted_path: named.redacted_path,
        name: named.name,
    })
}

/// The gaps a lookup found, or none and why it failed: the events are
/// recorded by then, so a failed lookup is reported beside the ingest,
/// never as its failure (I6).
fn gaps(
    found: Result<Vec<store::IndexGap>, store::StoreError>,
) -> (Vec<store::IndexGap>, Option<String>) {
    match found {
        Ok(gaps) => (gaps, None),
        Err(e) => (Vec::new(), Some(with_causes(&e))),
    }
}

/// `error` and each cause behind it, joined, so the cause chain survives
/// the string the caller logs.
fn with_causes(error: &dyn std::error::Error) -> String {
    let mut said = error.to_string();
    let mut cause = error.source();
    while let Some(c) = cause {
        said.push_str(": ");
        said.push_str(&c.to_string());
        cause = c.source();
    }

    said
}

/// A transcript source's names: its path as the store keys it, its
/// transcript directory, and what it may be shown as.
struct Named {
    path: String,
    directory: String,
    redacted_path: String,
    name: String,
}

impl Named {
    fn of(path: &Path, redactor: &Redactor) -> Self {
        let directory = path.parent().unwrap_or_else(|| Path::new(""));
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        Self {
            path: path.to_string_lossy().into_owned(),
            directory: directory.to_string_lossy().into_owned(),
            redacted_path: redactor.redact_text(&path.to_string_lossy()).text,
            name: redactor.redact_text(&stem).text,
        }
    }
}

/// Where an ingest resumes its transcript source.
struct Resume {
    /// The generation it reads.
    source: Source,
    /// The index of the first line it reads.
    index: usize,
    /// The position before that line.
    position: Position,
    new_generation: bool,
}

impl Resume {
    /// Where to resume the source `named`, whose bytes are `bytes`: past
    /// its position while its consumed prefix stands; else in a new
    /// generation, past the longest prefix equal by content to the
    /// earlier generation's ingested lines (REC-07).
    fn find(
        store: &mut Store,
        named: &Named,
        bytes: &[u8],
        lines: &[Line<'_>],
        redactor: &Redactor,
    ) -> Result<Self, IngestError> {
        let Some(source) = store.source(&named.path)? else {
            let source = store.start_generation(&named.path, &named.directory)?;
            return Ok(Self::at(source, 0, lines, bytes, false));
        };
        if continues(&source.position, bytes) {
            let offset = source.position.offset;
            let index = lines
                .iter()
                .position(|l| l.offset >= offset)
                .unwrap_or(lines.len());
            let position = source.position.clone();
            return Ok(Self {
                source,
                index,
                position,
                new_generation: false,
            });
        }
        // Bytes read before another ingest moved the position past their
        // end are no rewrite: the source only grew since (REC-06).
        if behind(&source.position, bytes, Path::new(&named.path)) {
            return Err(IngestError::Store(store::StoreError::PositionMoved {
                source: source.id,
            }));
        }
        let earlier = earlier_texts(store, &named.path)?;
        let same = lines
            .iter()
            .zip(&earlier)
            .take_while(|(line, text)| {
                repeat::same_content(&redactor.redact_line(line.text).text, text)
            })
            .count();
        let source = store.start_generation(&named.path, &named.directory)?;

        Ok(Self::at(source, same, lines, bytes, true))
    }

    /// Resuming `source` at the line `index` of `lines`.
    fn at(
        source: Source,
        index: usize,
        lines: &[Line<'_>],
        bytes: &[u8],
        new_generation: bool,
    ) -> Self {
        let offset = index
            .checked_sub(1)
            .and_then(|i| lines.get(i))
            .map_or(0, |l| l.end);
        let consumed = bytes.get(..as_index(offset)).unwrap_or_default();

        Self {
            source,
            index,
            position: Position {
                offset,
                prefix: Digest::of(&[consumed]),
                lines: index as u64,
                seen: 0,
            },
            new_generation,
        }
    }
}

/// A byte offset as an index into the bytes it lies within.
fn as_index(offset: u64) -> usize {
    usize::try_from(offset).unwrap_or(usize::MAX)
}

/// Whether `bytes` still begin with the prefix `position` consumed.
fn continues(position: &Position, bytes: &[u8]) -> bool {
    bytes
        .get(..as_index(position.offset))
        .is_some_and(|prefix| Digest::of(&[prefix]) == position.prefix)
}

/// Whether `bytes`, which end before `position`, were read before another
/// ingest moved it: the source at `path` now reads on from the prefix
/// `position` consumed. A source that cannot be read again, or that no
/// longer holds that prefix, was rewritten: REC-07's.
fn behind(position: &Position, bytes: &[u8], path: &Path) -> bool {
    bytes.len() < as_index(position.offset)
        && read::transcript(path).is_ok_and(|now| continues(position, &now))
}

/// The redacted lines the newest generation of the source at `path`
/// ingested, in order: those it skipped as equal to its predecessor's,
/// then its own.
fn earlier_texts(store: &Store, path: &str) -> Result<Vec<String>, IngestError> {
    let mut texts = Vec::new();
    for generation in store.generations(path)? {
        let placed = store.placed(generation.id)?;
        let kept = placed
            .first()
            .map_or(generation.position.lines, |p| p.place.line - 1);
        texts.truncate(as_index(kept));
        for p in &placed {
            texts.push(text_of(store, p)?);
        }
    }

    Ok(texts)
}

/// The redacted line the event `placed` holds, or, for a repeat
/// reference, the event it refers to; empty where purge removed it.
fn text_of(store: &Store, placed: &Placed) -> Result<String, IngestError> {
    let text = |address: Address| -> Result<Option<String>, IngestError> {
        Ok(store
            .event(address)?
            .and_then(|e| e.content)
            .and_then(|c| c["text"].as_str().map(str::to_owned)))
    };
    if placed.kind != EventKind::RepeatReference {
        return Ok(text(placed.address)?.unwrap_or_default());
    }
    let referred = store
        .event(placed.address)?
        .and_then(|e| e.content)
        .and_then(|c| c["repeats"].as_str().map(str::to_owned))
        .and_then(|a| Address::parse(&a).ok())
        .map(|(a, _)| a);

    Ok(referred
        .map(text)
        .transpose()?
        .flatten()
        .unwrap_or_default())
}

/// The ingested runs of one transcript source, by the harness's agent
/// id, and the harness session of every ingested run, by writer.
struct Runs<'r> {
    path: String,
    of_source: BTreeMap<Option<String>, WriterId>,
    sessions: BTreeMap<WriterId, String>,
    /// The writers whose seat this node's device key is known to certify.
    certified: BTreeSet<WriterId>,
    /// The run seats of the MCP server ingesting live, when one is.
    live: Option<&'r mut RunSeats>,
}

impl<'r> Runs<'r> {
    fn load(
        store: &Store,
        path: &str,
        recorder: &'r mut Recorder<'_>,
    ) -> Result<Self, IngestError> {
        let live = match recorder {
            Recorder::Live(seats) => Some(&mut **seats),
            Recorder::Cli => None,
        };
        let mut runs = Self {
            path: path.to_owned(),
            of_source: BTreeMap::new(),
            sessions: BTreeMap::new(),
            certified: BTreeSet::new(),
            live,
        };
        for run in store.ingested_runs()? {
            runs.keep(&run);
        }

        Ok(runs)
    }

    /// Whether the MCP server ingests live.
    fn live(&self) -> bool {
        self.live.is_some()
    }

    fn keep(&mut self, run: &IngestedRun) {
        // Live, a run's lines go to its run seat, never an ingested run's
        // seat (REC-19).
        if run.path == self.path && self.live.is_none() {
            self.of_source.insert(run.agent_id.clone(), run.writer);
        }
        self.sessions
            .insert(run.writer, run.harness_session.clone());
    }

    /// The writer of the run `key` names in this source, its ingest seat
    /// started in the personal room when it has none yet.
    fn ensure(
        &mut self,
        store: &mut Store,
        node: &Node,
        home: &Home,
        key: &RunKey,
        entropy: &mut dyn Entropy,
    ) -> Result<WriterId, IngestError> {
        if let Some(seats) = self.live.as_mut() {
            let writer = seats.ensure(store, node, key, entropy)?;
            self.of_source.insert(key.agent_id.clone(), writer);
            return Ok(writer);
        }
        // After a node identity change, what remains of a run goes to a
        // new ingest seat the new device key certifies (REC-19, REC-24):
        // the old writer gets nothing more.
        if let Some(writer) = self.of_source.get(&key.agent_id).copied()
            && (self.certified.contains(&writer) || certified_by(store, writer, node)?)
        {
            self.certified.insert(writer);
            return Ok(writer);
        }
        let seat = node
            .certify_ingest_seat(home, store, entropy)
            .map_err(IngestError::Admin)?;
        let run = IngestedRun {
            writer: seat.writer,
            path: self.path.clone(),
            harness_session: key.harness_session.clone(),
            agent_id: key.agent_id.clone(),
        };
        store.add_run(&run)?;
        self.keep(&run);
        self.certified.insert(seat.writer);

        Ok(seat.writer)
    }
}

/// Whether the seat of `writer` was certified by `node`'s device key.
fn certified_by(store: &Store, writer: WriterId, node: &Node) -> Result<bool, IngestError> {
    let device = node.device().public();

    Ok(store.seat(writer)?.is_some_and(|seat| {
        SeatCertificate::from_value(&seat.certificate).is_ok_and(|c| c.certifying_key == device)
    }))
}

/// What one ingest reads.
struct Read<'r, 'a> {
    node: &'r Node,
    home: &'r Home,
    lines: &'r [Line<'a>],
    parsed: &'r [Value],
    /// The index of the first line to read.
    from: usize,
    defaults: &'r Defaults,
    redactor: &'r Redactor,
}

/// What an ingest will append, line by line, with what reading found.
#[derive(Default)]
struct Plan {
    events: Vec<NewEvent>,
    places: Vec<Place>,
    /// For each event: its writer, whether a segment closes after it,
    /// and the byte offset just past its line.
    ends: Vec<(WriterId, bool, u64)>,
    /// Whether an ingest marker holds the line after the last.
    hold: bool,
    released: usize,
    references: usize,
    conflicts: usize,
    /// Each (owner, repeating) pair of writers whose repeat owner no
    /// fork observation chose.
    chosen: BTreeSet<(WriterId, WriterId)>,
    redactions: Vec<String>,
    unparsed: usize,
    drifts: Vec<String>,
    /// Each prompt line's event, by its index, with its `prompt_id` and
    /// what became of the prompt commitment it names (PRV-08).
    prompts: Vec<(usize, String, Outcome)>,
    /// How many witnessed lines repeating another transcript's event
    /// were recorded as their own (REC-04).
    witnessed_repeats: usize,
}

impl Plan {
    /// How many events it appends to each writer.
    fn events_by_writer(&self) -> BTreeMap<WriterId, usize> {
        let mut out = BTreeMap::new();
        for (writer, _, _) in &self.ends {
            *out.entry(*writer).or_insert(0) += 1;
        }
        out
    }
}

/// Decides what becomes of each line an ingest reads.
struct Planner<'a> {
    named: &'a Named,
    inputs: &'a Inputs,
    held_since: Option<u64>,
    /// The `uuid`s the directory's other transcripts hold, read once a
    /// fork's line needs them.
    siblings: Option<BTreeSet<String>>,
    /// Whether a fork observation names each ingested run's harness
    /// session, by writer.
    forked: BTreeMap<WriterId, bool>,
    /// Whether ingest has read another transcript of the directory,
    /// whose events a line could repeat.
    others: bool,
    /// The prompt commitments, by harness session and `prompt_id`, a
    /// line this ingest read used up.
    used: BTreeSet<(String, String)>,
    /// Whether the MCP server ingests live, its lines witnessed.
    live: bool,
    /// How many witnessed lines repeat another transcript's event.
    witnessed_repeats: usize,
}

impl<'a> Planner<'a> {
    fn new(
        store: &Store,
        named: &'a Named,
        inputs: &'a Inputs,
        source: &Source,
        runs: &Runs<'_>,
    ) -> Result<Self, IngestError> {
        let others = store
            .sources()?
            .iter()
            .any(|s| s.directory == named.directory && s.path != named.path);
        let forked = runs
            .sessions
            .iter()
            .map(|(w, s)| (*w, inputs.forks.contains(s)))
            .collect();

        Ok(Self {
            named,
            inputs,
            held_since: source.held_since,
            siblings: None,
            forked,
            others,
            used: BTreeSet::new(),
            live: runs.live(),
            witnessed_repeats: 0,
        })
    }

    /// Plans the lines `read` reads, in order, until one is held.
    fn plan(
        &mut self,
        store: &mut Store,
        runs: &mut Runs<'_>,
        session: &Session,
        read: Read<'_, '_>,
        entropy: &mut dyn Entropy,
    ) -> Result<Plan, IngestError> {
        let shapes = Shapes::supported();
        let mut classifier = Classifier::new(&shapes, *session, read.redactor);
        // The lines read before, replayed, so a result finds its call.
        for line in &read.lines[..read.from] {
            classifier.line(line.text);
        }
        let mut plan = Plan::default();
        for (i, line) in read.lines.iter().enumerate().skip(read.from) {
            let key = read.defaults.key(&read.parsed[i]);
            let writer = runs.ensure(store, read.node, read.home, &key, entropy)?;
            self.forked
                .insert(writer, self.inputs.forks.contains(&key.harness_session));
            let classified = classifier.line(line.text);
            let redacted = read.redactor.redact_line(line.text);
            let decision = self.decide(store, &read.parsed[i], &key, &redacted.text)?;
            if decision == Decision::Hold {
                plan.hold = true;
                break;
            }
            let user = matches!(classified, Classified::Line(Provenance::User, _));
            let prompt = self.prompt(store, &read.parsed[i], &key, user, &decision, &read)?;
            let at = Planned {
                writer,
                number: i as u64 + 1,
                line,
                parsed: &read.parsed[i],
            };
            plan.add(at, classified, redacted, decision, read.redactor, entropy)?;
            if let Some((prompt_id, outcome)) = prompt {
                plan.prompts
                    .push((plan.events.len() - 1, prompt_id, outcome));
            }
        }
        plan.witnessed_repeats = self.witnessed_repeats;

        Ok(plan)
    }

    /// What the parsed line `line` of the run `key`, classed `user` when
    /// `user`, does to the prompt commitment its `prompt_id` names, when
    /// it is a prompt line recorded as its own event (PRV-08).
    fn prompt(
        &mut self,
        store: &Store,
        line: &Value,
        key: &RunKey,
        user: bool,
        decision: &Decision,
        read: &Read<'_, '_>,
    ) -> Result<Option<(String, Outcome)>, IngestError> {
        if matches!(decision, Decision::Reference { .. }) {
            return Ok(None);
        }
        let Some(prompt_line) = PromptLine::of(line) else {
            return Ok(None);
        };
        let id = (key.harness_session.clone(), prompt_line.prompt_id.clone());
        let commitment = store.prompt_commitment(&id.0, &id.1)?;
        let redact = |text: &str| read.redactor.redact_text(text).text;
        let outcome = prompt::outcome(
            commitment.as_ref(),
            self.used.contains(&id),
            user,
            &prompt_line,
            &redact,
        );
        if outcome.is_some() {
            self.used.insert(id);
        }

        Ok(outcome.map(|o| (prompt_line.prompt_id, o)))
    }

    /// What becomes of the parsed line `line` of the run `key`, whose
    /// redacted text is `redacted` (REC-04).
    fn decide(
        &mut self,
        store: &Store,
        line: &Value,
        key: &RunKey,
        redacted: &str,
    ) -> Result<Decision, IngestError> {
        let Some(uuid) = line["uuid"].as_str() else {
            return Ok(Decision::Own);
        };
        if self.live {
            // A witnessed line is never a repeat reference, nor held
            // (REC-04): one another transcript recorded is its own event.
            let named = self.named;
            let repeats = self.others
                && store
                    .line_owner(&named.directory, uuid, &named.path)?
                    .is_some();
            self.witnessed_repeats += usize::from(repeats);
            return Ok(Decision::Own);
        }
        let owner = self.owner(store, uuid)?;
        let fork = self.inputs.forks.contains(&key.harness_session);
        let in_sibling = fork && owner.is_none() && self.siblings().contains(uuid);
        let facts = Facts {
            owner,
            fork,
            in_sibling,
            held_since: self.held_since,
            now: self.inputs.now_ms,
        };

        Ok(repeat::decide(&facts, redacted))
    }

    /// The event of another transcript of the directory that first
    /// recorded the line `uuid` names, if an ingested run holds one.
    fn owner(&self, store: &Store, uuid: &str) -> Result<Option<Owner>, IngestError> {
        let named = self.named;
        if !self.others {
            return Ok(None);
        }
        let Some(address) = store.line_owner(&named.directory, uuid, &named.path)? else {
            return Ok(None);
        };
        // An event a place names is an ingested run's, and holds its
        // line until purge removes it.
        let fork = self.forked.get(&address.writer).copied();
        let event = store.event(address)?;

        Ok(fork.zip(event).map(|(fork, event)| Owner {
            address,
            provenance: event.link.header.provenance.clone(),
            text: event
                .content
                .as_ref()
                .and_then(|c| c["text"].as_str())
                .unwrap_or_default()
                .to_owned(),
            fork,
        }))
    }

    /// The `uuid`s the directory's other transcripts hold.
    fn siblings(&mut self) -> &BTreeSet<String> {
        let named = self.named;
        self.siblings
            .get_or_insert_with(|| sibling_uuids(&named.directory, &named.path))
    }
}

/// Every `uuid` the transcripts in `directory` but `path` hold; a
/// transcript that cannot be read holds none.
fn sibling_uuids(directory: &str, path: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let other = entry.path();
        if other.to_string_lossy() == path || other.extension().is_none_or(|e| e != "jsonl") {
            continue;
        }
        let Ok(bytes) = read::transcript(&other) else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        for line in read::lines(&text) {
            let parsed: Value = serde_json::from_str(line.text).unwrap_or(Value::Null);
            if let Some(uuid) = parsed["uuid"].as_str() {
                found.insert(uuid.to_owned());
            }
        }
    }

    found
}

/// One line a plan takes.
struct Planned<'p, 'a> {
    writer: WriterId,
    number: u64,
    line: &'p Line<'a>,
    parsed: &'p Value,
}

impl Plan {
    /// Adds the line `at`, classed as `classified` and redacted to
    /// `redacted`, as `decision` says.
    fn add(
        &mut self,
        at: Planned<'_, '_>,
        classified: Classified,
        redacted: redaction::Redacted,
        decision: Decision,
        redactor: &Redactor,
        entropy: &mut dyn Entropy,
    ) -> Result<(), IngestError> {
        let describe = |text: &str| redactor.redact_text(text).text;
        let (kind, provenance, drift) = match classified {
            Classified::Unparsed(drift) => {
                self.unparsed += 1;
                (EventKind::UnparsedLine, Provenance::Unparsed, drift)
            }
            Classified::Line(provenance, drift) => (EventKind::TranscriptLine, provenance, drift),
        };
        self.drifts.extend(drift.map(|d| d.describe(&describe)));
        let end = turn_end(at.parsed) || provenance == Provenance::SubagentResult;
        let (kind, provenance, content) = match decision {
            Decision::Reference {
                owner,
                provenance,
                rule,
            } => {
                self.references += 1;
                if rule == Rule::FirstRecorded {
                    self.chosen.insert((owner.writer, at.writer));
                }
                let content = json!({"line": at.number, "repeats": owner.to_string()});
                (EventKind::RepeatReference, provenance, content)
            }
            other => {
                self.conflicts += usize::from(other == Decision::Conflict);
                self.released += usize::from(other == Decision::Released);
                let content = content(at.number, &redacted.text, redacted.rules.len());
                self.redactions.extend(redacted.rules);
                (kind, provenance, content)
            }
        };
        self.events.push(NewEvent {
            kind,
            provenance,
            content,
            commitment_key: CommitmentKey::from_bytes(draw(entropy).map_err(IngestError::Key)?),
            // Read from the line's own structure, as its class is; the
            // timestamp only when well formed, so it carries no secret.
            transcript_timestamp: at.parsed["timestamp"]
                .as_str()
                .filter(|t| well_formed(t))
                .map(str::to_owned),
        });
        self.places.push(Place {
            line: at.number,
            offset: at.line.offset,
            uuid: at.parsed["uuid"].as_str().map(str::to_owned),
        });
        self.ends.push((at.writer, end, at.line.end));

        Ok(())
    }
}

/// What [`append`] appended.
struct Landed {
    /// How many line events, and the address of each.
    lines: usize,
    addresses: Vec<Address>,
    /// How many hook observations, live, in the main run's run seat.
    observations: usize,
}

/// Appends `plan` to its writers a transaction at a time, each moving
/// the source's position past the lines it covers, then records how many
/// bytes the source held. A transaction holds one writer's run of
/// consecutive lines, at most a batch; its segments close at each turn
/// end and subagent end, and after a writer's last line. Live, each hook
/// observation `hooks` holds goes to the main run's run seat in a
/// transaction of its own, before the first line its hook fired before
/// (REC-13, REC-19).
fn append(
    store: &mut Store,
    resume: &Resume,
    bytes: &[u8],
    plan: &Plan,
    hooks: &Seen<'_>,
    entropy: &mut dyn Entropy,
) -> Result<Landed, IngestError> {
    let seen = bytes.len() as u64;
    let mut running = Running::default();
    running.update(
        bytes
            .get(..as_index(resume.position.offset))
            .unwrap_or_default(),
    );
    let mut position = Position {
        seen,
        ..resume.position.clone()
    };
    let last = last_of_each_writer(&plan.ends);
    // A segment closes at each turn end and subagent end, and after each
    // writer's last line.
    let closes: Vec<bool> = plan
        .ends
        .iter()
        .enumerate()
        .map(|(i, (writer, end, _))| *end || last.get(writer) == Some(&i))
        .collect();
    let mut landed = 0;
    let mut addresses = Vec::with_capacity(plan.events.len());
    let mut observing = Observing::new(hooks);
    let starts = observing.breaks(&plan.places);
    for range in transactions(&plan.ends, &starts) {
        let (writer, _, end) = plan.ends[range.end - 1];
        let first = plan.places[range.start].offset;
        observing.flush(store, Some(first), entropy)?;
        running.update(
            bytes
                .get(as_index(first)..as_index(end))
                .unwrap_or_default(),
        );
        position = Position {
            offset: end,
            prefix: running.digest(),
            lines: plan.places[range.end - 1].line,
            seen,
        };
        let appended = store.append_lines(
            writer,
            &plan.events[range.clone()],
            &plan.places[range.clone()],
            &closes[range.clone()],
            resume.source.id,
            &position,
        );
        addresses.extend(appended.map_err(|source| IngestError::Partial {
            writer,
            landed,
            lines: plan.events.len(),
            source,
        })?);
        landed += range.len();
    }
    store.advance(resume.source.id, &position)?;
    observing.flush(store, Some(position.offset), entropy)?;

    Ok(Landed {
        lines: landed,
        addresses,
        observations: observing.appended,
    })
}

/// The hook observations a live ingest appends to the main run's run
/// seat, each before the first line written after its hook fired.
struct Observing<'s> {
    /// The main run's writer, and its observations by offset.
    pending: Option<(WriterId, Vec<&'s Held>)>,
    appended: usize,
}

impl<'s> Observing<'s> {
    fn new(seen: &Seen<'s>) -> Self {
        let pending = seen.main.zip(seen.live.clone()).map(|(run, mut held)| {
            held.sort_by_key(|h| h.observation.offset);
            (run, held)
        });

        Self {
            pending,
            appended: 0,
        }
    }

    /// The indices of `places` a transaction must start at, so an
    /// observation lands between the lines on either side of its offset.
    fn breaks(&self, places: &[Place]) -> BTreeSet<usize> {
        let offsets = self
            .pending
            .iter()
            .flat_map(|(_, held)| held.iter().filter_map(|h| h.observation.offset));
        offsets
            .filter_map(|o| places.iter().position(|p| p.offset >= o))
            .collect()
    }

    /// Appends each observation whose hook fired at or before `upto`,
    /// none when it has no offset, to the main run's run seat.
    fn flush(
        &mut self,
        store: &mut Store,
        upto: Option<u64>,
        entropy: &mut dyn Entropy,
    ) -> Result<(), IngestError> {
        let Some((run, held)) = self.pending.as_mut() else {
            return Ok(());
        };
        let due = held
            .iter()
            .take_while(|h| h.observation.offset.zip(upto).is_none_or(|(o, u)| o <= u))
            .count();
        if due == 0 {
            return Ok(());
        }
        let ready: Vec<&Held> = held.drain(..due).collect();
        let events = ready
            .iter()
            .map(|h| {
                structural(
                    EventKind::HookObservation,
                    observed::content(h, *run),
                    entropy,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let ids: Vec<i64> = ready.iter().map(|h| h.id).collect();
        store.append_observed(*run, &events, &ids, &[])?;
        self.appended += ids.len();

        Ok(())
    }
}

/// A structural event of `kind` holding `content`.
fn structural(
    kind: EventKind,
    content: Value,
    entropy: &mut dyn Entropy,
) -> Result<NewEvent, IngestError> {
    Ok(NewEvent {
        kind,
        provenance: Provenance::Structural,
        content,
        commitment_key: CommitmentKey::from_bytes(draw(entropy).map_err(IngestError::Key)?),
        transcript_timestamp: None,
    })
}

/// What one ingest found of the hook observations of the harness
/// session it read.
struct Seen<'s> {
    observed: &'s Observed,
    /// The harness session of the transcript's own run.
    session: &'s str,
    /// The writer of that harness session's main run, when the
    /// transcript is its main run's: its hook observations go with it.
    main: Option<WriterId>,
    /// Whether the deployment mode is `interactive` (PRV-04).
    interactive: bool,
    /// Live, the hook observations the main run's run seat takes; none
    /// when `parinn ingest` reads, which puts them in the device seat.
    live: Option<Vec<&'s Held>>,
}

/// The hook observations a live ingest of the transcript whose main run
/// is `main` appends to that run's run seat: those its harness session
/// holds; none for `parinn ingest`.
fn live_observations<'o>(
    observed: &'o Observed,
    defaults: &Defaults,
    main: Option<WriterId>,
    live: bool,
) -> Option<Vec<&'o Held>> {
    let pending = match main {
        Some(_) => observed.pending(&defaults.own().harness_session),
        None => Vec::new(),
    };

    live.then_some(pending)
}

/// What [`record_observed`] recorded.
#[derive(Debug, Default)]
struct Recorded {
    appended: usize,
    matched: usize,
    conflicts: usize,
    not_attended: usize,
}

/// Records, in the device seat, a prompt match event for each prompt
/// line of `plan`, whose events landed at `addresses`, and a hook
/// observation event for each hook observation the transcript's main
/// run still held, using up the prompt commitments and marking the
/// observations appended in the same transaction (REC-19, PRV-08).
fn record_observed(
    store: &mut Store,
    node: &Node,
    seen: &Seen<'_>,
    plan: &Plan,
    addresses: &[Address],
    entropy: &mut dyn Entropy,
) -> Result<Recorded, IngestError> {
    let mut recorded = Recorded::default();
    let mut contents = Vec::new();
    let mut used = Vec::new();
    let prompts = plan
        .prompts
        .iter()
        .filter_map(|(index, id, outcome)| Some((addresses.get(*index)?, id, outcome)));
    for (address, prompt_id, outcome) in prompts {
        let attended = seen.observed.attended(seen.session, prompt_id);
        recorded.matched += usize::from(*outcome == Outcome::Matched);
        recorded.conflicts += usize::from(*outcome == Outcome::Conflict);
        recorded.not_attended +=
            usize::from(seen.interactive && *outcome == Outcome::Matched && !attended);
        used.push((seen.session.to_owned(), prompt_id.clone()));
        contents.push((
            EventKind::PromptMatch,
            json!({
                "event": address.to_string(),
                "prompt_id": prompt_id,
                "result": outcome.as_str(),
                "attended": attended,
            }),
        ));
    }
    // Live, the main run's run seat took them (REC-19).
    let pending = seen
        .main
        .filter(|_| seen.live.is_none())
        .map(|run| (run, seen.observed.pending(seen.session)));
    let mut ids = Vec::new();
    for (run, held) in pending
        .iter()
        .flat_map(|(r, h)| h.iter().map(move |h| (*r, h)))
    {
        contents.push((EventKind::HookObservation, observed::content(held, run)));
        ids.push(held.id);
    }
    recorded.appended = ids.len();
    if contents.is_empty() {
        return Ok(recorded);
    }
    let events = contents
        .into_iter()
        .map(|(kind, content)| structural(kind, content, entropy))
        .collect::<Result<Vec<_>, IngestError>>()?;
    store.append_observed(node.device_writer(), &events, &ids, &used)?;

    Ok(recorded)
}

/// The index of each writer's last event in `ends`.
fn last_of_each_writer(ends: &[(WriterId, bool, u64)]) -> BTreeMap<WriterId, usize> {
    ends.iter()
        .enumerate()
        .map(|(i, (writer, _, _))| (*writer, i))
        .collect()
}

/// The ranges of `ends` each append transaction takes: consecutive
/// events of one writer, at most a batch, each index in `starts`
/// beginning one.
fn transactions(
    ends: &[(WriterId, bool, u64)],
    starts: &BTreeSet<usize>,
) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, (writer, _, _)) in ends.iter().enumerate() {
        let next_differs = ends.get(i + 1).is_none_or(|(w, _, _)| w != writer);
        if next_differs || starts.contains(&(i + 1)) || i + 1 - start == store::BATCH {
            out.push(start..i + 1);
            start = i + 1;
        }
    }

    out
}

/// Marks `source` held since `now` when `held` and it was not, or no
/// longer held when not; returns whether a hold began.
fn hold(store: &mut Store, source: &Source, held: bool, now: u64) -> Result<bool, IngestError> {
    match (held, source.held_since) {
        (true, None) => store.hold(source.id, Some(now))?,
        (false, Some(_)) => store.hold(source.id, None)?,
        _ => return Ok(false),
    }

    Ok(held)
}

/// Records, in the device seat, a repeat owner event for each pair of
/// writers in `chosen`: the owner of repeated lines no fork observation
/// chose, and the run that repeats them (REC-04).
fn choose_owners(
    store: &mut Store,
    node: &Node,
    chosen: &BTreeSet<(WriterId, WriterId)>,
    entropy: &mut dyn Entropy,
) -> Result<usize, IngestError> {
    for (owner, repeating) in chosen {
        let event = NewEvent {
            kind: EventKind::RepeatOwner,
            provenance: Provenance::Structural,
            content: json!({
                "owner": owner.to_string(),
                "repeating": repeating.to_string(),
                "reason": "first recorded on this node",
            }),
            commitment_key: CommitmentKey::from_bytes(draw(entropy).map_err(IngestError::Key)?),
            transcript_timestamp: None,
        };
        store.append(node.device_writer(), &[event])?;
    }

    Ok(chosen.len())
}

/// The seat of `writer`.
fn seat_of(store: &Store, writer: WriterId) -> Result<record::id::SeatId, IngestError> {
    let seat = store.seat(writer)?;

    seat.map(|s| s.seat).ok_or_else(|| no_seat(writer))
}

/// The error of a run whose writer has no seat in the store.
fn no_seat(writer: WriterId) -> IngestError {
    IngestError::Store(store::StoreError::Corrupt {
        address: None,
        what: format!("writer {writer} has no seat"),
    })
}

/// A report of each run this ingest appended to, the transcript's own
/// run `own` first, each subagent's with its parent link.
fn reports(
    store: &Store,
    own: WriterId,
    appended: &BTreeMap<WriterId, usize>,
    runs: &Runs<'_>,
) -> Result<Vec<RunReport>, IngestError> {
    let all = store.ingested_runs()?;
    let mut writers: Vec<WriterId> = runs.of_source.values().copied().collect();
    writers.sort_by_key(|w| (*w != own, *w));
    writers
        .into_iter()
        .filter(|w| *w == own || appended.contains_key(w))
        .map(|writer| {
            let run = all.iter().find(|r| r.writer == writer);
            let agent_id = run.and_then(|r| r.agent_id.clone());
            let parent = run.filter(|r| r.agent_id.is_some()).and_then(|r| {
                all.iter()
                    .find(|p| p.harness_session == r.harness_session && p.agent_id.is_none())
                    .map(|p| p.writer)
            });
            Ok(RunReport {
                writer,
                seat: seat_of(store, writer)?,
                agent_id,
                parent,
                events: appended.get(&writer).copied().unwrap_or(0),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
