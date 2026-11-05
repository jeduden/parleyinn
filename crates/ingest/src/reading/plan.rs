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

/// Parsed JSON is transient but can be much larger than its source
/// syntax. Preflight keeps its conservative heap bound to two maximum
/// lines; `read::open` reserves that plus one whole-line redaction.
pub(super) const PARSED_LINE_LIMIT: usize = 2 * read::bounded::LINE_LIMIT;
const JSON_TOKEN_BYTES: usize = 256;

/// What one ingest reads.
pub(super) struct Read<'r, 'a> {
    pub(super) node: &'r Node,
    pub(super) home: &'r Home,
    pub(super) lines: &'r [Line<'a>],
    /// The index of the first line to read.
    pub(super) from: usize,
    /// How many lines of the source lie before the first of `lines`, when
    /// it is a window past the source's start (SEC-05).
    pub(super) first_line: u64,
    /// The source, whose lines before the window are replayed from it.
    pub(super) path: &'r Path,
    pub(super) defaults: &'r Defaults,
    pub(super) redactor: &'r Redactor,
    /// Bytes left under SEC-05's source-memory ceiling for planned
    /// event content and its metadata.
    pub(super) memory_limit: usize,
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
    /// The place of the line over the line ceiling the window ends
    /// before, once the ingest goes on past it: its content is never
    /// read, its bytes are zeros in the consumed-prefix hash (SEC-05,
    /// SEC-31).
    pub(super) over: Option<Place>,
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
    /// A conservative upper bound for source-derived bytes retained by
    /// this plan.
    pub(super) memory: usize,
    /// Whether planning stopped at its memory budget before the window
    /// ended, so the next read resumes at the ingest marker.
    pub(super) resumable: bool,
    /// The transcripts of the directory that could not be read to learn
    /// the `uuid`s they hold, by redacted name, with why (I6).
    pub(super) unread: Vec<crate::UnreadSource>,
}

impl Plan {
    /// Records the place of `line`, the line number of the line over the
    /// line ceiling `over`, so the ingest goes on past it. Its length
    /// takes its line break, as every place's does.
    pub(super) fn pass_over(&mut self, over: crate::read::bounded::Refused, line: u64) {
        let place = Place {
            line,
            offset: over.offset,
            length: over.length + 1,
            uuid: None,
        };
        self.refused_places.push(place.clone());
        self.over = Some(place);
    }

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
    /// Harness UUIDs whose event content was purged from another
    /// transcript in this directory, loaded once for this plan.
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
        let forked = runs
            .sessions
            .iter()
            .map(|(w, s)| (*w, inputs.forks.contains(s)))
            .collect();
        let purged = if others {
            store.purged_line_uuids(&named.directory, &named.path)?
        } else {
            BTreeSet::new()
        };

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
            if json_heap_bound(line.text) <= PARSED_LINE_LIMIT {
                classifier.line(line.text);
            }
        }
        if let (true, Some(first)) = (read.first_line > 0, read.lines.first()) {
            read::open::for_each_line(read.path, first.offset, |text| {
                if json_heap_bound(text) <= PARSED_LINE_LIMIT {
                    classifier.line(text);
                }
            })
            .map_err(IngestError::read(read.path))?;
        }
        let mut plan = Plan::default();
        for (i, line) in read.lines.iter().enumerate().skip(read.from) {
            let charge = plan_charge(line.text.len());
            if plan.memory.saturating_add(charge) > read.memory_limit {
                plan.resumable = true;
                break;
            }
            let parse = json_heap_bound(line.text) <= PARSED_LINE_LIMIT;
            let parsed: Value = if parse {
                serde_json::from_str(line.text).unwrap_or(Value::Null)
            } else {
                Value::Null
            };
            let key = read.defaults.key(&parsed);
            let writer = runs.ensure(store, read.node, read.home, &key, entropy)?;
            self.forked
                .insert(writer, self.inputs.forks.contains(&key.harness_session));
            let classified = if parse {
                classifier.line(line.text)
            } else {
                Classified::Unparsed(None)
            };
            let redacted = read.redactor.redact_line(line.text);
            let decision = self.decide(store, &parsed, &key, &redacted.text)?;
            if decision == Decision::Hold {
                plan.hold = true;
                break;
            }
            if decision == Decision::Refused {
                plan.refused += 1;
                plan.refused_places.push(Place {
                    line: read.first_line + i as u64 + 1,
                    offset: line.offset,
                    length: line.end - line.offset,
                    uuid: parsed["uuid"].as_str().map(str::to_owned),
                });
                plan.refused_to = Some((read.first_line + i as u64 + 1, line.end));
                plan.memory = plan.memory.saturating_add(charge);
                continue;
            }
            let user = matches!(classified, Classified::Line(Provenance::User, _));
            let prompt = self.prompt(store, &parsed, &key, user, &decision, &read)?;
            let at = Planned {
                writer,
                number: read.first_line + i as u64 + 1,
                line,
                parsed: &parsed,
            };
            plan.add(at, classified, redacted, decision, read.redactor, entropy)?;
            plan.memory = plan.memory.saturating_add(charge);
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

/// A conservative bound for the source-derived memory one planned line
/// can retain: redacted content, UUID/timestamp copies and container
/// metadata. The parsed JSON is transient and has a separate one-line
/// reserve in `read::open` (SEC-05).
fn plan_charge(line: usize) -> usize {
    line.saturating_mul(3).saturating_add(2 * 1024)
}

/// A conservative bound on the heap `serde_json::Value` may retain for
/// `text`, found without allocating: every string's source bytes plus a
/// generous node/container allowance for each JSON token. Large plain
/// strings stay parseable; dense arrays and objects become unparsed
/// lines before serde can allocate past SEC-05's transient budget.
pub(super) fn json_heap_bound(text: &str) -> usize {
    let bytes = text.as_bytes();
    let (mut at, mut tokens, mut strings) = (0, 0_usize, 0_usize);
    while at < bytes.len() {
        match bytes[at] {
            b'"' => {
                tokens = tokens.saturating_add(1);
                at += 1;
                let start = at;
                while at < bytes.len() {
                    match bytes[at] {
                        b'\\' => at = at.saturating_add(2),
                        b'"' => break,
                        _ => at += 1,
                    }
                }
                strings = strings.saturating_add(at.saturating_sub(start));
                at = at.saturating_add(1);
            }
            b'{' | b'[' => {
                tokens = tokens.saturating_add(1);
                at += 1;
            }
            b'}' | b']' | b':' | b',' | b' ' | b'\t' | b'\r' | b'\n' => at += 1,
            _ => {
                tokens = tokens.saturating_add(1);
                at += 1;
                while at < bytes.len()
                    && !matches!(bytes[at], b'}' | b']' | b',' | b' ' | b'\t' | b'\r' | b'\n')
                {
                    at += 1;
                }
            }
        }
    }

    strings.saturating_add(tokens.saturating_mul(JSON_TOKEN_BYTES))
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
        let bound = served.contains(&canonical(&other));
        // A line at a time, so a long transcript is never held (SEC-05).
        let read = read::open::for_each_line(&other, u64::MAX, |text| {
            if json_heap_bound(text) > PARSED_LINE_LIMIT {
                return;
            }
            let parsed: Value = serde_json::from_str(text).unwrap_or(Value::Null);
            if let Some(uuid) = parsed["uuid"].as_str() {
                found.all.insert(uuid.to_owned());
                if bound {
                    found.served.insert(uuid.to_owned());
                }
            }
        });
        if let Err(e) = read {
            found.unread.push((other, with_causes(&e)));
        }
    }

    found
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
            uuid: at.parsed["uuid"].as_str().map(str::to_owned),
        });
        self.ends.push((at.writer, end, at.line.end));

        Ok(())
    }
}

#[cfg(test)]
mod tests;
