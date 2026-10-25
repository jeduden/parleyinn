//! `event_search` (RCL-02, §9.2): a search of the record's text, its
//! hits in the envelope, each with its preview and score. The caller's
//! text is compiled by the query compiler into literal terms and never
//! reaches the index as syntax (SEC-04). Every refusal is fixed text
//! (RCL-04).
//!
//! The recall scope is `room` until the session binding names a current
//! run (phase 9 of plan 2609292012) and RCL-05 builds the rest, so
//! `run` is refused as `event_get` refuses it, and `rooms` is not served.
//! The filter `trust` keeps the hits of one derived trust level
//! (PRV-02), applied after the ranking.

use std::time::Duration;

use administration::trust_policy::TrustLevel;

use record::id::{RoomId, WriterId};
use search::Request;
use search::rank::MAX_HITS;
use serde_json::Value;
use store::Search;

/// The hits a search returns when it asks for no number (RCL-02).
pub const DEFAULT_HITS: usize = 10;

/// How long a search may run before it is cancelled (SEC-04).
pub const DEADLINE: Duration = Duration::from_secs(2);

/// The most values a filter list holds, and the most characters each of
/// them.
const FILTER_VALUES: usize = 16;
const FILTER_CHARS: usize = 64;

/// The refusals, each fixed text.
pub const NEEDS_RUN: &str = "this server serves no run yet, so pass scope \"room\"";
pub const NO_ROOMS: &str = "scope \"rooms\" is not served yet";
const BAD_SCOPE: &str = "the scope is none of run, room or rooms";
const BAD_ROOM: &str = "the room is not a room id";
const BAD_RUN: &str =
    "the run is not a writer id: a run is named by its writer until runs derive from seats";
const BAD_QUERY: &str = "query, phrase must be strings and any_of, all_of lists of strings";
const BAD_FILTER: &str = "provenance and kind must be lists of at most 16 short strings";
const BAD_TRUST: &str = "the trust is neither trusted nor untrusted";
const BAD_TIME: &str = "since and until must be a date or a UTC time like 2026-06-30T12:00:00Z";
const BAD_K: &str = "k must be a whole number from 1";

/// What a call asks for, checked.
#[derive(Debug, PartialEq, Eq)]
pub struct Params {
    pub text: String,
    pub any_of: Vec<String>,
    pub all_of: Vec<String>,
    pub phrase: Option<String>,
    pub room: Option<RoomId>,
    pub writer: Option<WriterId>,
    pub provenance: Vec<String>,
    pub kinds: Vec<String>,
    /// Only hits of this trust level, when set.
    pub trust: Option<TrustLevel>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub k: usize,
}

impl Params {
    /// The compiled query's request.
    #[must_use]
    pub fn request(&self) -> Request<'_> {
        Request {
            text: &self.text,
            any_of: &self.any_of,
            all_of: &self.all_of,
            phrase: self.phrase.as_deref(),
        }
    }

    /// The search over `query` in `room`, cancelled after `deadline`.
    #[must_use]
    pub fn search<'a>(
        &'a self,
        query: &'a search::Compiled,
        room: RoomId,
        deadline: Duration,
    ) -> Search<'a> {
        Search {
            query,
            room,
            writer: self.writer,
            provenance: &self.provenance,
            kinds: &self.kinds,
            since: self.since.as_deref(),
            until: self.until.as_deref(),
            k: self.k,
            deadline,
        }
    }
}

/// Checks the arguments of a call.
///
/// # Errors
///
/// Fails with fixed text naming what is wrong.
pub fn parse(arguments: &Value) -> Result<Params, &'static str> {
    match arguments["scope"].as_str().unwrap_or("run") {
        "room" => {}
        "run" => return Err(NEEDS_RUN),
        "rooms" => return Err(NO_ROOMS),
        _ => return Err(BAD_SCOPE),
    }
    let text = optional_string(&arguments["query"])?.unwrap_or_default();
    let trust = optional_string(&arguments["trust"]).map_err(|_| BAD_TRUST)?;
    let trust = match trust.as_deref() {
        None => None,
        Some("untrusted") => Some(TrustLevel::Untrusted),
        Some("trusted") => Some(TrustLevel::Trusted),
        Some(_) => return Err(BAD_TRUST),
    };

    Ok(Params {
        text,
        any_of: strings(&arguments["any_of"], BAD_QUERY)?,
        all_of: strings(&arguments["all_of"], BAD_QUERY)?,
        phrase: optional_string(&arguments["phrase"])?,
        room: id(&arguments["room"], BAD_ROOM, RoomId::parse)?,
        writer: id(&arguments["run"], BAD_RUN, WriterId::parse)?,
        provenance: filter(&arguments["provenance"])?,
        kinds: filter(&arguments["kind"])?,
        trust,
        since: time(&arguments["since"])?,
        until: time(&arguments["until"])?,
        k: hits(&arguments["k"])?,
    })
}

/// A string argument that may be absent.
fn optional_string(value: &Value) -> Result<Option<String>, &'static str> {
    match value {
        Value::Null => Ok(None),
        Value::String(s) => Ok(Some(s.clone())),
        _ => Err(BAD_QUERY),
    }
}

/// A list of strings that may be absent.
fn strings(value: &Value, refusal: &'static str) -> Result<Vec<String>, &'static str> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::Array(items) => items
            .iter()
            .map(|v| v.as_str().map(str::to_owned).ok_or(refusal))
            .collect(),
        _ => Err(refusal),
    }
}

/// A list of short strings, a few of them.
fn filter(value: &Value) -> Result<Vec<String>, &'static str> {
    let values = strings(value, BAD_FILTER)?;
    let fits =
        values.len() <= FILTER_VALUES && values.iter().all(|v| v.chars().count() <= FILTER_CHARS);

    fits.then_some(values).ok_or(BAD_FILTER)
}

/// An id argument that may be absent.
fn id<T, E>(
    value: &Value,
    refusal: &'static str,
    parse: fn(&str) -> Result<T, E>,
) -> Result<Option<T>, &'static str> {
    match value {
        Value::Null => Ok(None),
        Value::String(s) => parse(s).map(Some).map_err(|_| refusal),
        _ => Err(refusal),
    }
}

/// A date or a UTC time, as transcripts record them.
fn time(value: &Value) -> Result<Option<String>, &'static str> {
    let Some(text) = optional_string(value).map_err(|_| BAD_TIME)? else {
        return Ok(None);
    };
    let (date, rest) = text.split_at_checked(10).ok_or(BAD_TIME)?;
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    let shaped = date.len() == 10
        && digits(&date[..4])
        && &date[4..5] == "-"
        && digits(&date[5..7])
        && &date[7..8] == "-"
        && digits(&date[8..]);
    let clock = rest.is_empty()
        || (rest.starts_with('T')
            && (2..=24).contains(&rest.len())
            && rest[1..]
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b':' | b'.' | b'Z')));

    (shaped && clock).then_some(Some(text)).ok_or(BAD_TIME)
}

/// How many hits the call asks for, at most [`MAX_HITS`].
fn hits(value: &Value) -> Result<usize, &'static str> {
    match value {
        Value::Null => Ok(DEFAULT_HITS),
        Value::Number(n) => n
            .as_u64()
            .filter(|k| *k >= 1)
            .map(|k| usize::try_from(k).map_or(MAX_HITS, |k| k.min(MAX_HITS)))
            .ok_or(BAD_K),
        _ => Err(BAD_K),
    }
}

#[cfg(test)]
mod tests;
