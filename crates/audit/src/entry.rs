//! One audit entry (OPS-02): what happened, the counter it feeds, and
//! its place in the log's hash chain. Its line is the RFC 8785
//! encoding of its fields (§8.3); its hash is SHA-256 over the encoding
//! of every field but the hash, the previous entry's hash among them.

use record::canonical;
use record::digest::Digest;
use serde_json::{Map, Value, json};

use crate::AuditError;

/// The fields of an entry, the hash included; `counter` only on an
/// entry that feeds one.
const FIELDS: [&str; 8] = [
    "counter",
    "detail",
    "entry",
    "hash",
    "kind",
    "operation",
    "prev_hash",
    "time",
];

/// How an operation degraded (OPS-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Failed,
    Dropped,
    Rejected,
    Redacted,
    TimedOut,
    Truncated,
    Coalesced,
}

impl Kind {
    /// Its name in an entry.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Failed => "failed",
            Self::Dropped => "dropped",
            Self::Rejected => "rejected",
            Self::Redacted => "redacted",
            Self::TimedOut => "timed-out",
            Self::Truncated => "truncated",
            Self::Coalesced => "coalesced",
        }
    }
}

/// A named counter (OPS-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counter {
    HookRejected,
    HookPanicked,
    McpPanicked,
    /// A settings layer failed validation (ADM-04).
    SettingsLayerInvalid,
    /// A repository configuration key that does more than tighten was
    /// ignored (SEC-11).
    RepositoryConfigurationKeyIgnored,
    /// A settings layer's trust policy key was ignored (PRV-05).
    TrustPolicyKeyIgnored,
    /// A key only managed policy may set was ignored in the principal's
    /// configuration (OWN-22, SEC-22).
    ConfigurationKeyIgnored,
    /// Managed policy is writable by the principal, or does not parse
    /// or validate (ADM-04).
    ManagedPolicyInvalid,
    /// A principal act was refused: no terminal (OWN-12), or a widening
    /// act OWN-22 or managed policy does not permit (SEC-22).
    PrincipalActRefused,
    /// A secret or harness account identifier was redacted before it
    /// was written, once per redaction (SEC-08).
    Redacted,
    /// A principal's redaction pattern did not compile, or compiled past
    /// the size limit, and was refused (SEC-08).
    RedactionPatternRefused,
    /// A text's redaction reached its scan limit, so none of the text
    /// was written and a marker stands for it; once per text (SEC-08,
    /// I9, I6).
    RedactionScanLimited,
    /// A hook input exceeded 1 MiB and was refused (SEC-05).
    HookInputTooLarge,
    /// A hook input nested deeper than 64 levels and was refused
    /// (SEC-05).
    HookInputTooDeep,
    /// A path outside every allowed root was refused (SEC-18).
    PathOutsideAllowedRoots,
    /// A structured log line could not be written (ENG-05, I6).
    LogLineDropped,
    /// An ingest was refused while the principal's configuration or the
    /// repository configuration failed, so the redaction patterns it
    /// needs were unknown; the transcript stays for a later ingest
    /// (SEC-08, I1).
    IngestRefusedSettingsInvalid,
    /// A transcript line was stored as an `unparsed` event: it cannot be
    /// parsed, or its type is one no supported version records (REC-05).
    EventsUnparsed,
    /// A transcript line's shape differed from the one recorded for its
    /// harness version (REC-26).
    FormatDrift,
    /// The node identity changed, and a new device key and device seat
    /// replaced the old (REC-24).
    NodeIdentityChanged,
    /// The device key is missing from the key store beside this node's
    /// record (SEC-10, PRV-12).
    DeviceKeyMissing,
    /// The key store is locked or cannot be read (PRV-12).
    KeyStoreUnavailable,
    /// A transcript source shrank or its consumed prefix changed, and a
    /// new transcript generation started (REC-07).
    NewTranscriptGeneration,
    /// A transcript line repeated another source's `uuid` with other
    /// content, and was recorded as its own event (REC-04).
    RepeatConflicts,
    /// No fork observation named the owner of repeated lines, so the run
    /// that first recorded them owns them (REC-04).
    RepeatOwnerChosen,
    /// An ingest marker began to hold a harness fork's repeated lines
    /// until their owner's transcript is ingested (REC-04).
    RepeatHeld,
    /// Held repeated lines were released into their own run's writer,
    /// after 30 s or once their owner's transcript was gone (REC-04).
    RepeatReleased,
    /// A transcript source disappeared before it was fully ingested
    /// (REC-08).
    TranscriptSourceLost,
    /// A field of a hook input a handler uses failed the schema
    /// (SEC-16).
    HookInputRejected,
    /// A hook handler ran past its hook budget and answered empty
    /// (NFR-02).
    HookBudgetExceeded,
    /// A hook handler failed to record its hook observation (I6).
    HookFailed,
    /// A start marker was left behind by a hook handler the harness
    /// stopped before it finished (NFR-02).
    HookUnfinished,
    /// No MCP server took a harness session's session binding within
    /// 30 s of its `SessionStart` hook observation (REC-19).
    SessionBindingNotTaken,
    /// A prompt commitment was dropped while a settings layer failed,
    /// its redaction patterns unknown (PRV-08, SEC-08).
    PromptCommitmentDropped,
    /// A line carried the `prompt_id` of a prompt commitment already
    /// used up (PRV-08).
    PromptMatchConflicts,
    /// A matched `user` event's hook handler found no attended mark in
    /// deployment mode `interactive` (PRV-04).
    HarnessSessionNotAttended,
    /// The MCP server stopped short of a transcript's end, leaving the
    /// rest past its ingest marker: no failure, as `parinn ingest` or the
    /// next MCP server resumes it (REC-13).
    IngestRemainderLeft,
    /// A witnessed line repeating another transcript's event was
    /// recorded as its own event, never a repeat reference (REC-04).
    WitnessedRepeatRecorded,
    /// A tool call carried no tool-use id, so its run was not found and
    /// it was refused (PRV-14).
    ToolCallWithoutToolUseId,
    /// No transcript line carried a tool call's tool-use id within 2 s,
    /// and the call was refused (PRV-14).
    ToolCallRunNotFound,
    /// Transcript lines of two runs carried a tool call's tool-use id,
    /// and the call was refused (PRV-14).
    ToolCallRunAmbiguous,
    /// The MCP server failed to ingest a transcript of its harness
    /// session; the next read tries again (REC-13, I6).
    LiveIngestFailed,
    /// A search ran past its deadline and was cancelled (SEC-04).
    QueryCancelled,
    /// The search index leaves out the events of a writer whose slot a
    /// smaller writer id holds (REC-11, I6).
    SearchIndexWriterUnindexed,
    /// The search index leaves out the events of a writer from seq 2^28
    /// on (REC-11, I6).
    SearchIndexSeqBeyondLimit,
    /// A history held two different events at one (writer, seq): the
    /// later head was refused, the first kept (PEER-10, REC-21).
    ForkConflict,
    /// The search index's unreported gaps could not be looked up after
    /// an ingest recorded its events (REC-11, I6).
    SearchIndexGapLookupFailed,
    /// The identity a directory's repository resolves to changed since
    /// it was bound; no room's branch link is relinked for it (LANE-02,
    /// I6).
    RepositoryIdentityChanged,
    /// A worktree was bound after the node's device key was replaced (a
    /// node identity change, a rotation or a key minted anew): its kept
    /// names went with the old key, so a worktree bound before and one
    /// never bound are told apart by neither. A structural event, not a
    /// widening principal act (LANE-02, I6).
    RepositoryIdentityRebound,
    /// A clone's git data could not be read, so its repository identity
    /// stays as it was (LANE-02, I6).
    RepositoryReadFailed,
    /// The identity of a clone could not be bound: the record or the
    /// random source failed, not the clone (LANE-02, I6).
    RepositoryBindFailed,
    /// A look at a clone ran out of its time, so its identity stays as it
    /// was and the next look tries again (LANE-02, ENG-03, I6).
    RepositoryReadTimedOut,
    /// A recall tool stopped at its per-call cap and returned a
    /// continuation cursor (RCL-03).
    RecallTruncated,
    /// A recall call's event could not be appended to the record, so the
    /// call went unrecorded (RCL-07, I6).
    RecallNotRecorded,
    /// A recall call was refused: its scope was not served, or the record
    /// holds a gap inside the range it asked for (RCL-04, RCL-05, I6).
    RecallRefused,
    /// A purge of a range was refused: the range holds an event a purge
    /// keeps, none that holds content, or lies in a run with a pending
    /// ingest marker (ADM-07, I6).
    PurgeRefused,
    /// A purge erased what it was asked to, but a step after its
    /// transaction could not finish: the residue stays journalled for the
    /// next attempt (ADM-07, SEC-14, I6).
    PurgeUnfinished,
    /// A retention policy aged a range out and the node purged it
    /// (REC-15, SEC-22).
    RetentionPurged,
    /// Applying the retention policies failed: the store or the random
    /// source did; the next ingest tries again (SEC-22, I6).
    RetentionFailed,
    /// `parinn verify` ran out of time deriving the search index again:
    /// the index was left unchecked, neither passed nor failed (ADM-09,
    /// ENG-03, I6).
    VerifyTimedOut,
}

impl Counter {
    /// Its name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HookRejected => "hook_rejected",
            Self::HookPanicked => "hook_panicked",
            Self::McpPanicked => "mcp_panicked",
            Self::SettingsLayerInvalid => "settings_layer_invalid",
            Self::RepositoryConfigurationKeyIgnored => "repository_configuration_key_ignored",
            Self::TrustPolicyKeyIgnored => "trust_policy_key_ignored",
            Self::ConfigurationKeyIgnored => "configuration_key_ignored",
            Self::ManagedPolicyInvalid => "managed_policy_invalid",
            Self::PrincipalActRefused => "principal_act_refused",
            Self::Redacted => "redactions",
            Self::RedactionPatternRefused => "redaction_pattern_refused",
            Self::RedactionScanLimited => "redaction_scan_limited",
            Self::HookInputTooLarge => "hook_input_too_large",
            Self::HookInputTooDeep => "hook_input_too_deep",
            Self::PathOutsideAllowedRoots => "path_outside_allowed_roots",
            Self::LogLineDropped => "log_line_dropped",
            Self::IngestRefusedSettingsInvalid => "ingest_refused_settings_invalid",
            Self::EventsUnparsed => "events_unparsed",
            Self::FormatDrift => "format_drift",
            Self::NodeIdentityChanged => "node_identity_changed",
            Self::DeviceKeyMissing => "device_key_missing",
            Self::KeyStoreUnavailable => "key_store_unavailable",
            Self::NewTranscriptGeneration => "new_transcript_generation",
            Self::RepeatConflicts => "repeat_conflicts",
            Self::RepeatOwnerChosen => "repeat_owner_chosen",
            Self::RepeatHeld => "repeat_held",
            Self::RepeatReleased => "repeat_released",
            Self::TranscriptSourceLost => "transcript_source_lost",
            Self::HookInputRejected => "hook_input_rejected",
            Self::HookBudgetExceeded => "hook_budget_exceeded",
            Self::HookFailed => "hook_failed",
            Self::HookUnfinished => "hook_unfinished",
            Self::SessionBindingNotTaken => "session_binding_not_taken",
            Self::PromptCommitmentDropped => "prompt_commitment_dropped",
            Self::PromptMatchConflicts => "prompt_match_conflicts",
            Self::HarnessSessionNotAttended => "harness_session_not_attended",
            Self::IngestRemainderLeft => "ingest_remainder_left",
            Self::WitnessedRepeatRecorded => "witnessed_repeat_recorded",
            Self::ToolCallWithoutToolUseId => "tool_call_without_tool_use_id",
            Self::ToolCallRunNotFound => "tool_call_run_not_found",
            Self::ToolCallRunAmbiguous => "tool_call_run_ambiguous",
            Self::LiveIngestFailed => "live_ingest_failed",
            Self::QueryCancelled => "query_cancelled",
            Self::SearchIndexWriterUnindexed => "search_index_writer_unindexed",
            Self::SearchIndexSeqBeyondLimit => "search_index_seq_beyond_limit",
            Self::ForkConflict => "fork_conflict",
            Self::SearchIndexGapLookupFailed => "search_index_gap_lookup_failed",
            Self::RepositoryIdentityChanged => "repository_identity_changed",
            Self::RepositoryIdentityRebound => "repository_identity_rebound",
            Self::RepositoryReadFailed => "repository_read_failed",
            Self::RepositoryBindFailed => "repository_bind_failed",
            Self::RepositoryReadTimedOut => "repository_read_timed_out",
            Self::RecallTruncated => "recall_truncated",
            Self::RecallNotRecorded => "recall_not_recorded",
            Self::RecallRefused => "recall_refused",
            Self::PurgeRefused => "purge_refused",
            Self::PurgeUnfinished => "purge_unfinished",
            Self::RetentionPurged => "retention_purged",
            Self::RetentionFailed => "retention_failed",
            Self::VerifyTimedOut => "verify_timed_out",
        }
    }
}

/// What one entry reports. Every field is text ParleyInn ships, or a
/// settings key's name, so no record content or secret can reach the
/// log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occurrence<'a> {
    Degraded {
        kind: Kind,
        counter: Counter,
        operation: &'static str,
        detail: &'static str,
    },
    /// An operation degraded over one settings key: its detail is
    /// `detail` followed by `key`, a settings key's name, a provenance
    /// class or a format drift's description, shown only when it holds
    /// nothing but the characters those can.
    Keyed {
        kind: Kind,
        counter: Counter,
        operation: &'static str,
        detail: &'static str,
        key: &'a str,
    },
    /// A principal act was taken, such as acknowledging the counters;
    /// it feeds no counter.
    PrincipalAct {
        operation: &'static str,
        detail: &'static str,
    },
    /// A recall call whose scope extends past the calling run was served
    /// (RCL-05); it feeds no counter, as nothing degraded.
    RecallExtended {
        operation: &'static str,
        detail: &'static str,
    },
    /// A purge erased the content of the range `scope`, for `reason`,
    /// `by` the principal act or retention policy `behind` it; a
    /// retention policy's purge feeds `counter` (ADM-07, SEC-22). The
    /// entry names no content.
    Purged {
        scope: &'a str,
        reason: &'static str,
        by: &'static str,
        behind: &'a str,
        counter: Option<Counter>,
    },
}

/// The kind of an entry noting a principal act.
pub const PRINCIPAL_ACT: &str = "principal_act";

/// The kind of an entry noting a recall whose scope extends past the
/// calling run.
pub const RECALL_EXTENDED: &str = "recall_extended";

/// The kind of an entry noting a purge.
pub const PURGE: &str = "purge";

/// `key` as an entry may hold it: as it is when it holds only the
/// characters a settings key or a provenance class can, else a fixed
/// phrase. The audit log owns this filter, so it trusts no caller to
/// have checked the key (T5).
fn shown(key: &str) -> &str {
    let plain = !key.is_empty()
        && key.len() <= 160
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.:*-".contains(c));
    if plain {
        key
    } else {
        "a key no settings key spells"
    }
}

/// One entry of the audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub number: u64,
    pub time: u64,
    pub kind: String,
    pub counter: Option<String>,
    pub operation: String,
    pub detail: String,
    pub prev_hash: Digest,
    pub hash: Digest,
}

impl Entry {
    /// The entry numbered `number` reporting `occurrence` at `time`
    /// (seconds since the Unix epoch), chained to `prev_hash`.
    ///
    /// # Errors
    ///
    /// Fails when `number` or `time` lies past the integers the
    /// canonical encoding takes.
    pub fn next(
        occurrence: &Occurrence<'_>,
        number: u64,
        time: u64,
        prev_hash: Digest,
    ) -> Result<Self, AuditError> {
        let (kind, counter, operation, detail) = match *occurrence {
            Occurrence::Degraded {
                kind,
                counter,
                operation,
                detail,
            } => (
                kind.as_str(),
                Some(counter.as_str()),
                operation,
                detail.to_owned(),
            ),
            Occurrence::Keyed {
                kind,
                counter,
                operation,
                detail,
                key,
            } => (
                kind.as_str(),
                Some(counter.as_str()),
                operation,
                format!("{detail} {}", shown(key)),
            ),
            Occurrence::PrincipalAct { operation, detail } => {
                (PRINCIPAL_ACT, None, operation, detail.to_owned())
            }
            Occurrence::RecallExtended { operation, detail } => {
                (RECALL_EXTENDED, None, operation, detail.to_owned())
            }
            Occurrence::Purged {
                scope,
                reason,
                by,
                behind,
                counter,
            } => (
                PURGE,
                counter.map(Counter::as_str),
                "purge",
                format!(
                    "purge of {} ({reason}) by {by} {}",
                    shown(scope),
                    shown(behind)
                ),
            ),
        };
        let mut entry = Self {
            number,
            time,
            kind: kind.to_owned(),
            counter: counter.map(str::to_owned),
            operation: operation.to_owned(),
            detail,
            prev_hash,
            hash: Digest::ZERO,
        };
        entry.hash = entry.digest()?;

        Ok(entry)
    }

    /// Its line: the canonical encoding of its fields, its hash among
    /// them, with no line break.
    ///
    /// # Errors
    ///
    /// Fails when it has no canonical encoding.
    pub fn line(&self) -> Result<String, AuditError> {
        let mut value = self.body();
        value.insert("hash".to_owned(), json!(self.hash.to_hex()));

        canonical::encode(&Value::Object(value)).map_err(AuditError::Encode)
    }

    /// The entry `line` holds, its hash checked.
    ///
    /// # Errors
    ///
    /// Fails, saying why, on a line that is not the canonical encoding
    /// of a whole entry, or whose hash does not match.
    pub fn parse(line: &str) -> Result<Self, &'static str> {
        let value: Value = serde_json::from_str(line).map_err(|_| "not JSON")?;
        let entry = Self::from_value(&value).ok_or("not an entry")?;
        if canonical::encode(&value).ok().as_deref() != Some(line) {
            return Err("not canonical JSON");
        }
        if entry.digest().ok() != Some(entry.hash) {
            return Err("hash does not match");
        }

        Ok(entry)
    }

    /// The entry `value` spells, every field present and typed, and no
    /// other.
    fn from_value(value: &Value) -> Option<Self> {
        let object = value.as_object()?;
        let fields = object.keys().all(|k| FIELDS.contains(&k.as_str()));
        let text = |k: &str| object.get(k)?.as_str().map(str::to_owned);
        let digest = |k: &str| Digest::from_hex(object.get(k)?.as_str()?).ok();
        let counter = match object.get("counter") {
            None => None,
            Some(c) => Some(c.as_str()?.to_owned()),
        };
        let number = object.get("entry")?.as_u64().filter(|n| *n > 0)?;

        fields.then_some(Self {
            number,
            time: object.get("time")?.as_u64()?,
            kind: text("kind")?,
            counter,
            operation: text("operation")?,
            detail: text("detail")?,
            prev_hash: digest("prev_hash")?,
            hash: digest("hash")?,
        })
    }

    /// Every field but the hash.
    fn body(&self) -> Map<String, Value> {
        let mut body = Map::new();
        body.insert("entry".to_owned(), json!(self.number));
        body.insert("time".to_owned(), json!(self.time));
        body.insert("kind".to_owned(), json!(self.kind));
        if let Some(counter) = &self.counter {
            body.insert("counter".to_owned(), json!(counter));
        }
        body.insert("operation".to_owned(), json!(self.operation));
        body.insert("detail".to_owned(), json!(self.detail));
        body.insert("prev_hash".to_owned(), json!(self.prev_hash.to_hex()));

        body
    }

    /// SHA-256 over the canonical encoding of every field but the hash.
    fn digest(&self) -> Result<Digest, AuditError> {
        let encoded = canonical::encode(&Value::Object(self.body())).map_err(AuditError::Encode)?;

        Ok(Digest::of(&[encoded.as_bytes()]))
    }
}

#[cfg(test)]
mod tests;
