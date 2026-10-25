//! What an event is and what produced its content: its event kind, and
//! its provenance class from PRV-01's closed set. Both sit in the
//! chained header; neither is content.

/// Which kind of entry an event is, recorded apart from its
/// provenance. The record grows this set as the plans land.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventKind {
    /// A key event of a key event log (PRV-12).
    KeyEvent,
    /// A create room act (LANE-01).
    CreateRoom,
    /// A certificate record: a seat certificate's digest, recorded in a
    /// device seat (PRV-11, PRV-12).
    CertificateRecord,
    /// A transcript line recorded whole that the transcript parser
    /// cannot parse, or whose type no supported version records
    /// (REC-05).
    UnparsedLine,
    /// A transcript line recorded whole, of the provenance class its
    /// structure gives (PRV-08).
    TranscriptLine,
    /// A principal act, recorded in the device seat of the device it
    /// was taken on (OWN-02, OWN-11).
    PrincipalAct,
    /// A node identity change event, the first event of the device seat
    /// a node whose node identity changed starts (REC-24).
    NodeIdentityChange,
    /// A repeat reference: a transcript line a harness fork repeats,
    /// recorded by a run that does not own it as the address of the
    /// event that holds it, never a copy (REC-04).
    RepeatReference,
    /// A repeat owner event: which run's writer owns a repeated line
    /// when no fork observation names one, recorded in the device seat
    /// (REC-04).
    RepeatOwner,
    /// A hook observation: the structural fields a hook handler kept,
    /// held at a transcript's ingest marker until it reached the record
    /// (REC-19, PRV-08).
    HookObservation,
    /// A prompt match: what became of a prompt commitment the first line
    /// carrying its `prompt_id` used up (PRV-08).
    PromptMatch,
    /// A repository's identity, bound once at the first hook event that
    /// shows it (LANE-02): a structural event, never an `operator` one.
    RepositoryBind,
}

impl EventKind {
    /// The kind's name in a header.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeyEvent => "key_event",
            Self::CreateRoom => "create_room",
            Self::CertificateRecord => "certificate_record",
            Self::UnparsedLine => "unparsed_line",
            Self::TranscriptLine => "transcript_line",
            Self::PrincipalAct => "principal_act",
            Self::NodeIdentityChange => "node_identity_change",
            Self::RepeatReference => "repeat_reference",
            Self::RepeatOwner => "repeat_owner",
            Self::HookObservation => "hook_observation",
            Self::PromptMatch => "prompt_match",
            Self::RepositoryBind => "repository_bind",
        }
    }

    /// The kind `name` names.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        [
            Self::KeyEvent,
            Self::CreateRoom,
            Self::CertificateRecord,
            Self::UnparsedLine,
            Self::TranscriptLine,
            Self::PrincipalAct,
            Self::NodeIdentityChange,
            Self::RepeatReference,
            Self::RepeatOwner,
            Self::HookObservation,
            Self::PromptMatch,
            Self::RepositoryBind,
        ]
        .into_iter()
        .find(|k| k.as_str() == name)
    }
}

/// What produced an event's content: PRV-01's closed set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Provenance {
    User,
    Assistant,
    ToolCall,
    /// The result of the named tool.
    ToolResult(String),
    Web,
    /// What the named MCP server returned.
    Mcp(String),
    File,
    SubagentResult,
    HarnessMeta,
    HarnessText,
    Operator,
    Post,
    Summary,
    Structural,
    Unparsed,
}

/// The classes that carry no name, by their names.
const PLAIN: [(&str, Provenance); 13] = [
    ("user", Provenance::User),
    ("assistant", Provenance::Assistant),
    ("tool_call", Provenance::ToolCall),
    ("web", Provenance::Web),
    ("file", Provenance::File),
    ("subagent_result", Provenance::SubagentResult),
    ("harness_meta", Provenance::HarnessMeta),
    ("harness_text", Provenance::HarnessText),
    ("operator", Provenance::Operator),
    ("post", Provenance::Post),
    ("summary", Provenance::Summary),
    ("structural", Provenance::Structural),
    ("unparsed", Provenance::Unparsed),
];

impl Provenance {
    /// The class's name in a header: `tool_result:<tool>` and
    /// `mcp:<server>` carry their name.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::ToolResult(tool) => format!("tool_result:{tool}"),
            Self::Mcp(server) => format!("mcp:{server}"),
            plain => PLAIN
                .iter()
                .find(|(_, p)| p == plain)
                .map_or_else(String::new, |(n, _)| (*n).to_owned()),
        }
    }

    /// The class `name` names.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        let named = |prefix: &str| name.strip_prefix(prefix).filter(|n| !n.is_empty());
        if let Some(tool) = named("tool_result:") {
            return Some(Self::ToolResult(tool.to_owned()));
        }
        if let Some(server) = named("mcp:") {
            return Some(Self::Mcp(server.to_owned()));
        }

        PLAIN
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, p)| p.clone())
    }
}

#[cfg(test)]
mod tests;
