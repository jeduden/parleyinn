//! REC-07's steps for a purged line: a transcript appended to after a
//! purge stays in its transcript generation, its consumed prefix read
//! with each purged line's bytes zeroed, and one rewritten after a purge
//! starts a new transcript generation that refuses each purged line by
//! its `uuid`, audits each refusal and records neither again (REC-04,
//! SEC-31). A purged line that carries no `uuid` cannot be refused: the
//! MCP server serving the rewritten transcript leaves it waiting for
//! `parinn ingest`, audited, and the transcript generation that command
//! starts records the line again, counted and audited as unchecked
//! against purged lines.

use std::fs;

use cucumber::{then, when};

use crate::World;
use crate::live::wait_until;
use crate::record::home;
use crate::record_ingest::{generations, ingest_all, lines, open, placed, rewritten};
use crate::record_live::{live, serve};
use crate::security_redaction::transcript;

/// What the audit log says of each line a purge erased that ingest
/// refuses, before the transcript it names.
const REFUSED: &str = "refused a repeated line whose `uuid` names a purged event";

/// What the audit log says of each line recorded without a check
/// against purged lines, before the transcript it names.
const UNCHECKED: &str = "recorded a line that keeps no `uuid`, unchecked against purged lines";

/// What the audit log says of a rewritten transcript the MCP server
/// leaves waiting, before the transcript it names.
const WAITS: &str =
    "a rewritten transcript with a purged line that kept no `uuid` waits for `parinn ingest`:";

/// How many transcript generations the transcript had when the MCP
/// server had audited leaving it waiting.
#[derive(Debug, Default)]
struct Waited {
    generations: usize,
}

/// A transcript line that carries no `uuid`, of a length no other line
/// of these steps has.
const NO_UUID: &str = "{\"type\":\"x\",\"note\":\"this line carries no uuid\"}\n";

/// Purges the events read from lines `from` to `to` of the transcript
/// `name`, as a person does.
fn purge_lines(w: &mut World, name: &str, from: u64, to: u64) -> Result<(), String> {
    let placed = placed(w, name)?;
    let seq_of = |line: u64| {
        placed
            .iter()
            .find(|p| p.place.line == line)
            .map(|p| p.address.seq.get())
            .ok_or_else(|| format!("line {line} of {name:?} is not in the record"))
    };
    let (first, last) = (seq_of(from)?, seq_of(to)?);

    crate::erasure::purge_run_events(w, name, first, last)
}

#[when(
    regex = r#"^"([^"]+)" is appended to by (\d+) lines after a purge of its lines (\d+)-(\d+)$"#
)]
fn purged_then_appended(
    w: &mut World,
    name: String,
    more: u64,
    from: u64,
    to: u64,
) -> Result<(), String> {
    purge_lines(w, &name, from, to)?;
    let path = transcript(w, &name)?;
    let mut text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let count = text.lines().count() as u64;
    text.push_str(&lines(&name, count + 1, count + more));
    fs::write(&path, text).map_err(|e| e.to_string())
}

#[then(
    regex = r#"^"([^"]+)" stays in transcript generation 1, its consumed prefix matching the stored hash with the bytes of each purged line zeroed, and its (\d+) new lines are recorded$"#
)]
fn resumed_past_a_purge(w: &mut World, name: String, more: u64) -> Result<(), String> {
    let generations = generations(w, &name)?;
    let [source] = generations.as_slice() else {
        return Err(format!("{name:?} has {} generations", generations.len()));
    };
    let bytes = fs::read(transcript(w, &name)?).map_err(|e| e.to_string())?;
    let offset = source.position.offset;
    let zeroed = store::source::prefix_digest(&bytes, offset, &source.purged);
    let as_written = store::source::prefix_digest(&bytes, offset, &[]);
    if source.purged.is_empty() || zeroed != Some(source.position.prefix) || zeroed == as_written {
        return Err(format!(
            "{name:?}: the stored hash is not that of the prefix with its {} purged lines zeroed",
            source.purged.len()
        ));
    }
    let total = bytes
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .count() as u64;
    let recorded: Vec<u64> = placed(w, &name)?.iter().map(|p| p.place.line).collect();
    let new: Vec<u64> = (total.saturating_sub(more) + 1..=total).collect();
    let all_new = total >= more && new.iter().all(|n| recorded.contains(n));
    if offset == bytes.len() as u64 && all_new {
        return Ok(());
    }
    Err(format!(
        "{name:?} stands at {offset} of {} bytes and records lines {recorded:?}",
        bytes.len()
    ))
}

#[when(
    regex = r#"^"([^"]+)" is rewritten with a different first line after a purge of its lines (\d+)-(\d+)$"#
)]
fn purged_then_rewritten(w: &mut World, name: String, from: u64, to: u64) -> Result<(), String> {
    purge_lines(w, &name, from, to)?;
    let path = transcript(w, &name)?;
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;

    fs::write(&path, rewritten(&name, text.lines().count() as u64)).map_err(|e| e.to_string())
}

/// How many events of the first transcript generation of `name` still
/// hold their content.
fn kept_of_the_first(w: &mut World, name: &str) -> Result<usize, String> {
    let generations = generations(w, name)?;
    let first = generations.first().ok_or("no generation")?;
    let store = open(w)?;
    let mut present = 0;
    for p in store.placed(first.id).map_err(|e| e.to_string())? {
        let event = store.event(p.address).map_err(|e| e.to_string())?;
        present += usize::from(event.is_some_and(|e| e.content.is_some()));
    }

    Ok(present)
}

/// How many audit entries' detail starts with `said`.
fn audited(w: &World, said: &str) -> Result<usize, String> {
    let entries = audit::log::Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;

    Ok(entries
        .iter()
        .filter(|e| e.detail.starts_with(said))
        .count())
}

#[then(
    regex = r#"^"([^"]+)" starts transcript generation (\d+), its purged lines (\d+)-(\d+) are each refused with an audit entry and never recorded again, and the (\d+) other events of transcript generation 1 remain in the record$"#
)]
fn refused_in_a_new_generation(
    w: &mut World,
    name: String,
    generation: u64,
    from: u64,
    to: u64,
    kept: usize,
) -> Result<(), String> {
    let generations = generations(w, &name)?;
    let newest = generations.last().ok_or("no generation")?;
    if newest.generation != generation {
        return Err(format!(
            "{name:?} is at generation {}, not {generation}",
            newest.generation
        ));
    }
    let again: Vec<u64> = open(w)?
        .placed(newest.id)
        .map_err(|e| e.to_string())?
        .iter()
        .map(|p| p.place.line)
        .filter(|line| (from..=to).contains(line))
        .collect();
    let (present, refused) = (kept_of_the_first(w, &name)?, audited(w, REFUSED)?);
    let purged = usize::try_from(to - from + 1).map_err(|e| e.to_string())?;
    if again.is_empty() && present == kept && refused == purged {
        return Ok(());
    }

    Err(format!(
        "{name:?}: generation {generation} records lines {again:?} again, generation 1 keeps {present} events and {refused} refusals are audited"
    ))
}

#[when(
    regex = r#"^"([^"]+)" is rewritten with a different first line, while its MCP server serves it, after a purge of its ingested line (\d+), which carries no uuid$"#
)]
fn purged_without_uuid_then_rewritten_while_served(
    w: &mut World,
    name: String,
    line: u64,
) -> Result<(), String> {
    let path = transcript(w, &name)?;
    let mut text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let count = text.lines().count() as u64;
    if line != count + 1 {
        return Err(format!(
            "{name:?} has {count} lines, so it gains line {}",
            count + 1
        ));
    }
    text.push_str(NO_UUID);
    fs::write(&path, text).map_err(|e| e.to_string())?;
    ingest_all(w)?;
    purge_lines(w, &name, line, line)?;
    live(w).transcripts.insert(name.clone(), path.clone());
    serve(w, &name, "startup", &[])?;
    // Renamed into place, so the server never reads half of it.
    let staged = path.with_extension("staged");
    fs::write(&staged, format!("{}{NO_UUID}", rewritten(&name, count)))
        .map_err(|e| e.to_string())?;
    fs::rename(&staged, &path).map_err(|e| e.to_string())?;
    wait_until("the MCP server to leave the transcript waiting", || {
        Ok(audited(w, WAITS)? > 0)
    })?;
    w.section::<Waited>().generations = generations(w, &name)?.len();
    let harness = live(w).harness.take().ok_or("no MCP server is running")?;

    harness.stop()
}

/// Whether the newest transcript generation of `name` records `line`
/// again without a `uuid`, at the byte length of a line the purge of
/// the first left without one, beside a line that keeps no `uuid` and
/// has another length: REC-07 counts the first as unchecked and leaves
/// the second out.
fn repeats_at_a_purged_length(w: &mut World, name: &str, line: u64) -> Result<bool, String> {
    let generations = generations(w, name)?;
    let first = generations.first().ok_or("no generation")?;
    let newest = generations.last().ok_or("no generation")?;
    let store = open(w)?;
    let purged: Vec<u64> = store
        .purged_places(first.id)
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|place| place.uuid.is_none())
        .map(|place| place.length)
        .collect();
    let placed = store.placed(newest.id).map_err(|e| e.to_string())?;
    let without_uuid = || placed.iter().filter(|p| p.place.uuid.is_none());
    let again = without_uuid().any(|p| p.place.line == line && purged.contains(&p.place.length));
    let other = without_uuid().any(|p| p.place.line != line && !purged.contains(&p.place.length));

    Ok(again && other)
}

#[then(
    regex = r#"^the MCP server left "([^"]+)" in transcript generation (\d+), waiting for "parinn ingest" with an audit entry, and now "[^"]+" starts transcript generation (\d+), its line (\d+) is recorded again and counted as unchecked against purged lines with an audit entry, and the (\d+) other events of transcript generation 1 remain in the record$"#
)]
fn unchecked_in_a_new_generation(
    w: &mut World,
    name: String,
    waited_in: usize,
    generation: u64,
    line: u64,
    kept: usize,
) -> Result<(), String> {
    let (waited, waits) = (w.section::<Waited>().generations, audited(w, WAITS)?);
    if waited != waited_in || waits != 1 {
        return Err(format!(
            "{name:?} had {waited} generations when the MCP server left it waiting, audited {waits} times"
        ));
    }
    let generations = generations(w, &name)?;
    let newest = generations.last().ok_or("no generation")?;
    if newest.generation != generation {
        return Err(format!(
            "{name:?} is at generation {}, not {generation}",
            newest.generation
        ));
    }
    let again = repeats_at_a_purged_length(w, &name, line)?;
    let (present, unchecked) = (kept_of_the_first(w, &name)?, audited(w, UNCHECKED)?);
    let refused = audited(w, REFUSED)?;
    if again && present == kept && unchecked == 1 && refused == 0 {
        return Ok(());
    }

    Err(format!(
        "{name:?}: generation {generation} records line {line} again at a purged line's byte length, beside a line without a `uuid` of another length: {again}; generation 1 keeps {present} events; {unchecked} unchecked lines and {refused} refusals are audited"
    ))
}
