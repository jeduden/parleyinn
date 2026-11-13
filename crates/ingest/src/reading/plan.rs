//! What an ingest will append, line by line (REC-04): which run each
//! line goes to, what becomes of a repeated line, and what each
//! decision leaves for the caller to audit.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use administration::home::Home;
use administration::node::Node;
use key::random::{Entropy, draw};
use provenance::{Classified, Classifier, Session};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::WriterId;
use redaction::Redactor;
use serde_json::{Value, json};
use store::{NewEvent, Place, Source, Store};
use transcript::shape::Shapes;

use super::{Named, Runs, with_causes};
use crate::prompt::{self, Outcome, PromptLine};
use crate::read::{self, Line};
use crate::repeat::{self, Decision, Facts, Owner, Rule};
use crate::run::{Defaults, RunKey};
use crate::{IngestError, Inputs, content, turn_end, well_formed};

/// What one ingest reads.
pub(super) struct Read<'r, 'a> {
    pub(super) node: &'r Node,
    pub(super) home: &'r Home,
    pub(super) lines: &'r [Line<'a>],
    pub(super) parsed: &'r [Value],
    /// The index of the first line to read.
    pub(super) from: usize,
    pub(super) defaults: &'r Defaults,
    pub(super) redactor: &'r Redactor,
}

/// What an ingest will append, line by line, with what reading found.
#[derive(Default)]
pub(super) struct Plan {
    pub(super) events: Vec<NewEvent>,
    pub(super) places: Vec<Place>,
    /// For each event: its writer, whether a segment closes after it,
    /// and the byte offset just past its line.
    pub(super) ends: Vec<(WriterId, bool, u64)>,
    /// Whether an ingest marker holds the line after the last.
    pub(super) hold: bool,
    pub(super) released: usize,
    pub(super) references: usize,
    pub(super) conflicts: usize,
    /// How many lines were refused: their `uuid` names a purged event
    /// (REC-04).
    pub(super) refused: usize,
    /// The positions, lengths and harness UUIDs of those refused copies,
    /// whose bytes the consumed-prefix hash takes as zeros (SEC-31).
    pub(super) refused_places: Vec<Place>,
    /// The line number and end offset of the last line refused, so the
    /// ingest position moves past a refusal no recorded line follows.
    pub(super) refused_to: Option<(u64, u64)>,
    /// Each (owner, repeating) pair of writers whose repeat owner no
    /// fork observation chose.
    pub(super) chosen: BTreeSet<(WriterId, WriterId)>,
    pub(super) redactions: Vec<String>,
    /// How many lines' redaction scan limit was reached (I9).
    pub(super) scan_limited: usize,
    pub(super) unparsed: usize,
    pub(super) drifts: Vec<String>,
    /// Each prompt line's event, by its index, with its `prompt_id` and
    /// what became of the prompt commitment it names (PRV-08).
    pub(super) prompts: Vec<(usize, String, Outcome)>,
    /// How many witnessed lines repeating another transcript's event
    /// were recorded as their own (REC-04).
    pub(super) witnessed_repeats: usize,
    /// The transcripts of the directory that could not be read to learn
    /// the `uuid`s they hold, by redacted name, with why (I6).
    pub(super) unread: Vec<crate::UnreadSource>,
}

impl Plan {
    /// How many events it appends to each writer.
    pub(super) fn events_by_writer(&self) -> BTreeMap<WriterId, usize> {
        let mut out = BTreeMap::new();
        for (writer, _, _) in &self.ends {
            *out.entry(*writer).or_insert(0) += 1;
        }
        out
    }
}

/// Decides what becomes of each line an ingest reads.
pub(super) struct Planner<'a> {
    named: &'a Named,
    inputs: &'a Inputs,
    held_since: Option<u64>,
    /// The `uuid`s the directory's other transcripts hold, read once a
    /// fork's line needs them.
    siblings: Option<Siblings>,
    /// Harness UUIDs whose event content a purge erased, in any
    /// transcript source of this directory, loaded once for this plan.
    purged: BTreeSet<String>,
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
    pub(super) fn new(
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
        let purged = store.purged_line_uuids(&named.directory)?;
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
            purged,
            forked,
            others,
            used: BTreeSet::new(),
            live: runs.live(),
            witnessed_repeats: 0,
        })
    }

    /// Plans the lines `read` reads, in order, until one is held.
    pub(super) fn plan(
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
            if decision == Decision::Refused {
                plan.refused += 1;
                plan.refused_places.push(Place {
                    line: i as u64 + 1,
                    offset: line.offset,
                    length: line.end - line.offset,
                    uuid: kept_uuid(&read.parsed[i], read.redactor),
                });
                plan.refused_to = Some((i as u64 + 1, line.end));
                continue;
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
        plan.unread = self.unread(read.redactor);

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
        if self.purged.contains(uuid) {
            return Ok(Decision::Refused);
        }
        if self.live {
            return self.decide_witnessed(store, uuid, key, redacted);
        }
        let owner = self.owner(store, uuid)?;
        let fork = self.inputs.forks.contains(&key.harness_session);
        let in_other_transcript = fork && owner.is_none() && self.siblings().all.contains(uuid);
        let facts = Facts {
            owner,
            harness_fork: fork,
            in_other_transcript,
            held_since: self.held_since,
            now: self.inputs.now_ms,
        };

        Ok(repeat::decide(&facts, redacted))
    }

    /// What becomes of a witnessed line: never a repeat reference
    /// (REC-04). One whose `uuid` another transcript's event holds is its
    /// own event, counted for the caller to audit, and refused when that
    /// event was purged. Before any event holds it, a fork's line is held
    /// for a transcript another MCP server holds a session binding for,
    /// and never for one no server does: it is its own event at once,
    /// counted when another transcript holds its `uuid`.
    fn decide_witnessed(
        &mut self,
        store: &Store,
        uuid: &str,
        key: &RunKey,
        redacted: &str,
    ) -> Result<Decision, IngestError> {
        let named = self.named;
        let held = if self.others {
            store.line_owner(&named.directory, uuid, &named.path)?
        } else {
            None
        };
        if let Some(address) = held {
            let purged = store.event(address)?.is_some_and(|e| e.content.is_none());
            if purged {
                return Ok(Decision::Refused);
            }
            self.witnessed_repeats += 1;
            return Ok(Decision::Own);
        }
        let fork = self.inputs.forks.contains(&key.harness_session);
        let repeated = fork && self.siblings().all.contains(uuid);
        let facts = Facts {
            owner: None,
            harness_fork: fork,
            in_other_transcript: repeated && self.siblings().served.contains(uuid),
            held_since: self.held_since,
            now: self.inputs.now_ms,
        };
        let decision = repeat::decide(&facts, redacted);
        // A fork's line recorded at once, though another transcript
        // holds its `uuid`, is counted for the caller to audit.
        self.witnessed_repeats += usize::from(repeated && decision == Decision::Own);

        Ok(decision)
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
            purged: event.content.is_none(),
            address,
            provenance: event.link.header.provenance.clone(),
            text: event
                .content
                .as_ref()
                .and_then(|c| c["text"].as_str())
                .unwrap_or_default()
                .to_owned(),
            harness_fork: fork,
        }))
    }

    /// What the planner could not read of the directory's other
    /// transcripts, named as the redactor shows them.
    fn unread(&self, redactor: &Redactor) -> Vec<crate::UnreadSource> {
        let Some(siblings) = &self.siblings else {
            return Vec::new();
        };
        siblings
            .unread
            .iter()
            .map(|(path, why)| crate::UnreadSource {
                name: redactor
                    .redact_text(&path.file_stem().unwrap_or_default().to_string_lossy())
                    .text,
                why: why.clone(),
            })
            .collect()
    }

    /// The `uuid`s the directory's other transcripts hold.
    fn siblings(&mut self) -> &Siblings {
        let (named, inputs) = (self.named, self.inputs);
        self.siblings
            .get_or_insert_with(|| sibling_uuids(&named.directory, &named.path, &inputs.served))
    }
}

/// The `uuid`s the transcripts of a directory but one hold.
#[derive(Debug, Default)]
struct Siblings {
    /// Every one.
    all: BTreeSet<String>,
    /// Those of transcripts another MCP server holds a session binding
    /// for.
    served: BTreeSet<String>,
    /// Each directory or transcript that could not be read, with why:
    /// the lines read before a failure stay in the sets above.
    unread: Vec<(PathBuf, String)>,
}

/// Every `uuid` the transcripts in `directory` but `path` hold, and
/// those of the transcripts in `served`. What cannot be read holds none,
/// and is kept in [`Siblings::unread`] for the caller to count (I6, I9).
fn sibling_uuids(directory: &str, path: &str, served: &BTreeSet<PathBuf>) -> Siblings {
    let listing = std::fs::read_dir(directory).map(|e| e.map(|entry| entry.map(|e| e.path())));

    sibling_uuids_of(listing, directory, path, served)
}

/// [`sibling_uuids`] of the entries `listing` gives for `directory`.
fn sibling_uuids_of(
    listing: std::io::Result<impl Iterator<Item = std::io::Result<PathBuf>>>,
    directory: &str,
    path: &str,
    served: &BTreeSet<PathBuf>,
) -> Siblings {
    let mut found = Siblings::default();
    let entries = match listing {
        Ok(entries) => entries,
        Err(e) => {
            found
                .unread
                .push((PathBuf::from(directory), with_causes(&e)));
            return found;
        }
    };
    let served: BTreeSet<PathBuf> = served.iter().map(|p| canonical(p)).collect();
    for entry in entries {
        let other = match entry {
            Ok(other) => other,
            Err(e) => {
                found
                    .unread
                    .push((PathBuf::from(directory), with_causes(&e)));
                continue;
            }
        };
        if other.to_string_lossy() == path || other.extension().is_none_or(|e| e != "jsonl") {
            continue;
        }
        let bytes = match read::transcript(&other) {
            Ok(bytes) => bytes,
            Err(e) => {
                found.unread.push((other, with_causes(&e)));
                continue;
            }
        };
        let text = String::from_utf8_lossy(&bytes);
        let bound = served.contains(&canonical(&other));
        for line in read::lines(&text) {
            let parsed: Value = serde_json::from_str(line.text).unwrap_or(Value::Null);
            if let Some(uuid) = parsed["uuid"].as_str() {
                found.all.insert(uuid.to_owned());
                if bound {
                    found.served.insert(uuid.to_owned());
                }
            }
        }
    }

    found
}

/// The line's `uuid`, kept beside its place only when redaction leaves
/// it whole, so a place carries no secret (SEC-08).
fn kept_uuid(parsed: &Value, redactor: &Redactor) -> Option<String> {
    let uuid = parsed["uuid"].as_str()?;

    (redactor.redact_text(uuid).text == uuid).then(|| uuid.to_owned())
}

/// `path` with its links resolved, or as it is when it cannot be.
fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
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
        let limited = Cell::new(0);
        let describe = |text: &str| {
            let redacted = redactor.redact_text(text);
            limited.set(limited.get() + usize::from(redacted.limited));
            redacted.text
        };
        let (kind, provenance, drift) = match classified {
            Classified::Unparsed(drift) => {
                self.unparsed += 1;
                (EventKind::UnparsedLine, Provenance::Unparsed, drift)
            }
            Classified::Line(provenance, drift) => (EventKind::TranscriptLine, provenance, drift),
        };
        self.drifts.extend(drift.map(|d| d.describe(&describe)));
        self.scan_limited += limited.get();
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
                self.scan_limited += usize::from(redacted.limited);
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
            length: at.line.end - at.line.offset,
            uuid: kept_uuid(at.parsed, redactor),
        });
        self.ends.push((at.writer, end, at.line.end));

        Ok(())
    }
}

#[cfg(test)]
mod tests;
