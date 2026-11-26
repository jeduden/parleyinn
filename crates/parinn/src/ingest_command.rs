//! The `parinn ingest` command: what it reads, records and audits.

use super::*;

/// The payload threshold the settings in force set (REC-09).
pub(crate) fn payload_settings(
    settings: &configuration::resolve::Settings,
) -> ingest::PayloadSettings {
    // The key has a default and takes only an integer it bounds.
    let threshold = settings
        .get("payload.threshold_bytes")
        .and_then(|v| v.to_json().as_u64())
        .and_then(|n| usize::try_from(n).ok());

    ingest::PayloadSettings {
        payload_threshold: threshold.unwrap_or(store::DEFAULT_PAYLOAD_THRESHOLD),
    }
}

/// What `parinn ingest` reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IngestTarget {
    /// Every transcript under the transcript roots (REC-01).
    All,
    /// The transcript at this path.
    Path(String),
}

/// `parinn ingest --all | --path P [--json]`: records every transcript
/// under the transcript roots, or the one at `P`, each in a new ingest
/// seat. A transcript that cannot be ingested is reported and the rest
/// still are; the command then fails.
pub(crate) fn ingest(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let (target, as_json) = match ingest_args(args) {
        Ok(parsed) => parsed,
        Err(why) => {
            let _ = writeln!(stderr, "parinn ingest: {why}");
            return EXIT_USAGE;
        }
    };
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("ingest", &e, stderr),
    };
    let in_force = audit_settings(ctx, &home, stderr);
    if in_force.loaded.principal_layer_failed() {
        return failed("ingest", &refuse_ingest(ctx, stderr), stderr);
    }
    let settings = &in_force.resolved.settings;
    let candidates = match &target {
        IngestTarget::Path(path) => vec![PathBuf::from(path)],
        IngestTarget::All => match root_transcripts(ctx, settings) {
            Ok(found) => found,
            Err(e) => return failed("ingest", &e, stderr),
        },
    };
    let named = matches!(target, IngestTarget::Path(_));
    let (allowed, mut failures) = ingest_path::allowed(ctx, settings, &candidates, named, stderr);
    if allowed.is_empty() && failures > 0 {
        return EXIT_FAILURE;
    }
    // The rule set always compiles (its unit test proves it); were it
    // not to, each transcript would fail with why.
    let redactor = redactor(ctx, settings, stderr);
    match start_node(ctx, &home, stderr) {
        Ok(mut store) => {
            purge::finish(ctx, &mut store);
            count_untaken_bindings(ctx, &mut store);
        }
        Err(e) => return failed("ingest", &e, stderr),
    }
    // What an MCP server killed before it stopped left is audited by
    // the reader that finds its ingest marker (REC-13).
    capture::count_stopped(&home, "ingest", &|o| {
        let _ = ctx.audit(o);
    });
    let inputs = ingest::Inputs {
        payload: payload_settings(settings),
        // Ingest reads the fork observations the hook handlers held from
        // the store itself, and holds no line for a server (REC-04).
        now_ms: (ctx.clock)().saturating_mul(1000),
        interactive: settings.deployment_mode() == "interactive",
        ..ingest::Inputs::default()
    };
    let paths: Vec<&Path> = allowed.iter().map(PathBuf::as_path).collect();
    let (reports, failed_ingests) = ingest_all(ctx, &home, &paths, &redactor, &inputs, stderr);
    failures += failed_ingests;
    let held = held_sources(ctx, settings, &home, stderr);
    let held: Vec<&Path> = held.iter().map(PathBuf::as_path).collect();
    failures += ingest_all(ctx, &home, &held, &redactor, &inputs, stderr).1;
    apply_retention(ctx, &home, settings, stderr);
    if target == IngestTarget::All {
        sources::audit_lost(ctx, &home, &redactor, stderr);
    }
    let shown = ingest_report::ingest_report(&target, &reports, failures, as_json);
    let code = written(write!(stdout, "{shown}"), stderr);

    if failures > 0 { EXIT_FAILURE } else { code }
}

/// Applies the retention policies in force once the ingest is done
/// (SEC-22); a failure to open the store again is audited and counted,
/// and stops nothing (I9).
pub(crate) fn apply_retention(
    ctx: &Context,
    home: &Home,
    settings: &configuration::resolve::Settings,
    stderr: &mut dyn Write,
) {
    if retention::policies(settings).is_empty() {
        return;
    }
    let opened = start_node(ctx, home, stderr).and_then(|store| {
        let node = Node::find(home, &store).map_err(|e| e.to_string())?;
        Ok((store, node))
    });
    retention::apply_opened(ctx, opened, settings, &mut OsEntropy);
}

/// Ingests each transcript source at `paths`. Returns a report of each
/// ingested, and how many failed, each said on `stderr`.
pub(crate) fn ingest_all(
    ctx: &Context,
    home: &Home,
    paths: &[&Path],
    redactor: &Result<Redactor, String>,
    inputs: &ingest::Inputs,
    stderr: &mut dyn Write,
) -> (Vec<ingest::Ingested>, usize) {
    let mut reports = Vec::new();
    let mut failures = 0;
    for path in paths {
        let ingested = redactor.as_ref().map_err(Clone::clone).and_then(|r| {
            ingest::ingest_with(home, path, r, inputs, &mut OsEntropy).map_err(|e| e.to_string())
        });
        match ingested {
            Ok(i) => {
                record_ingested(ctx, &i, stderr);
                reports.push(i);
            }
            Err(e) => {
                failed("ingest", &e, stderr);
                failures += 1;
            }
        }
    }

    (reports, failures)
}

/// Counts, once each, every session binding no MCP server took within
/// 30 s of its `SessionStart` hook observation (REC-19, I6).
pub(crate) fn count_untaken_bindings(ctx: &Context, store: &mut Store) {
    hook_handlers::count_untaken(store, (ctx.clock)(), &|o| {
        let _ = ctx.audit(o);
    });
}

/// The transcript sources an ingest marker holds, each resolved again
/// within the transcript roots: what a harness fork read before its
/// owner left waiting, read again once its owner is ingested or gone
/// (REC-04, REC-08).
pub(crate) fn held_sources(
    ctx: &Context,
    settings: &configuration::resolve::Settings,
    home: &Home,
    stderr: &mut dyn Write,
) -> Vec<PathBuf> {
    let held: Vec<PathBuf> = Store::open(&home.store())
        .and_then(|store| store.sources())
        .map(|sources| {
            sources
                .into_iter()
                .filter(|s| s.held_since.is_some() && !s.lost)
                .map(|s| PathBuf::from(s.path))
                .collect()
        })
        .unwrap_or_default();

    ingest_path::within(ctx, settings, &held, stderr)
}

/// What `parinn ingest`'s command line asks for, or why it is refused.
pub(crate) fn ingest_args(
    mut args: impl Iterator<Item = String>,
) -> Result<(IngestTarget, bool), String> {
    let (mut all, mut path, mut as_json) = (false, None, false);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => as_json = true,
            "--all" => all = true,
            "--path" => path = Some(args.next().ok_or("--path needs a transcript")?),
            other => match other.strip_prefix("--path=") {
                Some(p) => path = Some(p.to_owned()),
                None => return Err(format!("unknown flag {other:?}")),
            },
        }
    }
    match (all, path) {
        (true, Some(_)) => Err("want --all or --path P, not both".to_owned()),
        (true, None) => Ok((IngestTarget::All, as_json)),
        (false, Some(path)) => Ok((IngestTarget::Path(path), as_json)),
        (false, None) => Err("want --all or --path P".to_owned()),
    }
}

/// The transcripts under every transcript root (REC-01), root by root.
pub(crate) fn root_transcripts(
    ctx: &Context,
    settings: &configuration::resolve::Settings,
) -> Result<Vec<PathBuf>, String> {
    let os_home = ctx.home.as_deref().map(Path::new);
    let mut found = Vec::new();
    for root in settings.texts("transcript.roots") {
        let root = allowed_root::expand_home(root, os_home);
        found.extend(ingest::transcripts(&root).map_err(|e| e.to_string())?);
    }

    Ok(found)
}

/// Audits, once each, the events the search index leaves out that no
/// ingest reported yet (REC-11, I6), saying on `stderr` what it cannot.
pub(crate) fn audit_index_gaps(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
    if let Some(why) = &ingested.index_gaps_unread {
        let _ = ctx.audit(&index_gaps::lookup_failed("ingest"));
        let _ = writeln!(
            stderr,
            "parinn ingest: look up the search index gaps: {why}"
        );
    }
    if ingested.index_gaps.is_empty() {
        return;
    }
    let problem = match ctx.home() {
        Ok(home) => index_gaps::problem(&index_gaps::report(
            &home,
            &ingested.index_gaps,
            "ingest",
            &mut |o| ctx.audit(o).is_ok(),
        )),
        Err(e) => Some(e.to_string()),
    };
    if let Some(e) = problem {
        let _ = writeln!(stderr, "parinn ingest: audit the search index gaps: {e}");
    }
}

/// Audits what one ingest found, each redaction, unparsed line and
/// format drift, and logs it.
pub(crate) fn record_ingested(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
    audit_redactions(ctx, &ingested.redactions, stderr);
    audit_scan_limited(ctx, ingested.scan_limited, stderr);
    audit_parsing(ctx, ingested, stderr);
    sourcing::audit(ctx, ingested, stderr);
    audit_prompts(ctx, ingested, stderr);
    audit_index_gaps(ctx, ingested, stderr);
    tracing::info!(
        target: "parinn",
        transcript = %ingested.redacted_path,
        writer = %ingested.writer,
        events = ingested.events as u64,
        redactions = ingested.redactions.len() as u64,
        redaction_scan_limited = ingested.scan_limited as u64,
        unparsed = ingested.unparsed as u64,
        format_drift = ingested.drifts.len() as u64,
        "ingested"
    );
}

/// Audits each prompt line an ingest found carrying a `prompt_id`
/// already used up (PRV-08), and each matched `user` event whose hook
/// handler found no attended mark in deployment mode `interactive`,
/// which stays untrusted (PRV-04).
pub(crate) fn audit_prompts(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
    let occurrences = [
        (
            ingested.prompt_conflicts,
            Occurrence::Degraded {
                kind: Kind::Rejected,
                counter: Counter::PromptMatchConflicts,
                operation: "ingest",
                detail: "a line carried the prompt_id of a prompt commitment already used up",
            },
        ),
        (
            ingested.not_attended,
            Occurrence::Degraded {
                kind: Kind::Rejected,
                counter: Counter::HarnessSessionNotAttended,
                operation: "ingest",
                detail: "a matched user event stays untrusted: its hook handler found no \
                         attended mark",
            },
        ),
    ];
    for (times, occurrence) in &occurrences {
        for _ in 0..*times {
            if let Err(e) = ctx.audit(occurrence) {
                let _ = writeln!(stderr, "parinn ingest: audit a prompt match: {e}");
            }
        }
    }
}

/// Audits each line an ingest stored as `unparsed` (REC-05) and each
/// format drift it found (REC-26), one entry each, naming the harness
/// version and what differed and nothing of the line's content.
pub(crate) fn audit_parsing(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
    let unparsed = Occurrence::Degraded {
        kind: Kind::Failed,
        counter: Counter::EventsUnparsed,
        operation: "ingest",
        detail: "stored a transcript line it cannot parse as an unparsed event",
    };
    for _ in 0..ingested.unparsed {
        if let Err(e) = ctx.audit(&unparsed) {
            let _ = writeln!(stderr, "parinn ingest: audit an unparsed line: {e}");
        }
    }
    for drift in &ingested.drifts {
        let occurrence = Occurrence::Keyed {
            kind: Kind::Rejected,
            counter: Counter::FormatDrift,
            operation: "ingest",
            detail: "format drift",
            key: drift,
        };
        if let Err(e) = ctx.audit(&occurrence) {
            let _ = writeln!(stderr, "parinn ingest: audit a format drift: {e}");
        }
    }
}

/// Opens the node before anything is appended: its first start, or the
/// node identity check every later start makes first (REC-24). A change
/// is audited and counted. A device key missing from the key store, or
/// a key store that is locked or cannot be read, refuses the command,
/// audited and counted (I6).
pub(crate) fn start_node(
    ctx: &Context,
    home: &Home,
    stderr: &mut dyn Write,
) -> Result<Store, String> {
    home.create().map_err(|e| e.to_string())?;
    let mut store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    open_node(ctx, home, &mut store, stderr)?;

    Ok(store)
}

/// The node in `home`, opened in `store` as [`start_node`] opens it:
/// a node identity change audited and counted, and a missing device key
/// or a key store that cannot be read refused, audited and counted.
pub(crate) fn open_node(
    ctx: &Context,
    home: &Home,
    store: &mut Store,
    stderr: &mut dyn Write,
) -> Result<Node, String> {
    let opened = Node::open(home, store, &mut OsEntropy);
    let occurrence = match &opened {
        Ok(node) if node.identity_change().is_some() => Some(Occurrence::Degraded {
            kind: Kind::Dropped,
            counter: Counter::NodeIdentityChanged,
            operation: "start",
            detail: "node identity changed: a new device key and device seat replace the old",
        }),
        Ok(_) => None,
        Err(administration::AdminError::DeviceKeyMissing) => Some(Occurrence::Degraded {
            kind: Kind::Failed,
            counter: Counter::DeviceKeyMissing,
            operation: "start",
            detail: "device key missing from the key store",
        }),
        Err(administration::AdminError::Key(e)) if e.is_unavailable() => {
            Some(Occurrence::Degraded {
                kind: Kind::Failed,
                counter: Counter::KeyStoreUnavailable,
                operation: "start",
                detail: "key store locked or unreadable",
            })
        }
        Err(_) => None,
    };
    if let Some(occurrence) = occurrence
        && let Err(e) = ctx.audit(&occurrence)
    {
        let _ = writeln!(stderr, "parinn: audit the node's start: {e}");
    }

    opened.map_err(|e| e.to_string())
}

/// Refuses an ingest while the principal's configuration or the
/// repository configuration fails: the redaction patterns they name are
/// unknown, so nothing may be written (SEC-08). The refusal is audited
/// and counted (I6), and the transcript stays where it is for an ingest
/// once the layer is fixed (I1).
pub(crate) fn refuse_ingest(ctx: &Context, stderr: &mut dyn Write) -> String {
    let occurrence = Occurrence::Degraded {
        kind: Kind::Rejected,
        counter: Counter::IngestRefusedSettingsInvalid,
        operation: "ingest",
        detail: "ingest refused while a settings layer fails",
    };
    if let Err(e) = ctx.audit(&occurrence) {
        let _ = writeln!(stderr, "parinn ingest: audit a refused ingest: {e}");
    }

    "a settings layer failed, so its redaction patterns are unknown; fix it and ingest again"
        .to_owned()
}

/// The redactor for the settings in force: the rule set and the
/// principal's redaction patterns (SEC-08), each pattern refused
/// audited and counted while the rest still apply.
pub(crate) fn redactor(
    ctx: &Context,
    settings: &configuration::resolve::Settings,
    stderr: &mut dyn Write,
) -> Result<Redactor, String> {
    let patterns = settings.texts("redaction.extra_patterns");
    let (redactor, refused) = Redactor::new(patterns).map_err(|e| e.to_string())?;
    for r in &refused {
        let occurrence = Occurrence::Keyed {
            kind: Kind::Rejected,
            counter: Counter::RedactionPatternRefused,
            operation: "ingest",
            detail: "refused redaction pattern",
            key: &r.rule,
        };
        if let Err(e) = ctx.audit(&occurrence) {
            let _ = writeln!(
                stderr,
                "parinn ingest: audit a refused redaction pattern: {e}"
            );
        }
    }

    Ok(redactor)
}

/// Audits each redaction an ingest made, one entry per redaction
/// (SEC-08, I6), naming its rule and nothing of what it removed.
pub(crate) fn audit_redactions(ctx: &Context, rules: &[String], stderr: &mut dyn Write) {
    for rule in rules {
        let occurrence = Occurrence::Keyed {
            kind: Kind::Redacted,
            counter: Counter::Redacted,
            operation: "ingest",
            detail: "redacted by rule",
            key: rule,
        };
        if let Err(e) = ctx.audit(&occurrence) {
            let _ = writeln!(stderr, "parinn ingest: audit a redaction: {e}");
        }
    }
}

/// What the audit says of a text whose redaction scan limit was reached.
pub(crate) const SCAN_LIMITED_DETAIL: &str =
    "a text's redaction scan limit was reached; none of it was written, a marker stands for it";

/// The occurrence of a text whose redaction scan limit was reached, in
/// `operation` (SEC-08, I9, I6).
pub(crate) fn scan_limited_occurrence(operation: &'static str) -> Occurrence<'static> {
    Occurrence::Degraded {
        kind: Kind::Dropped,
        counter: Counter::RedactionScanLimited,
        operation,
        detail: SCAN_LIMITED_DETAIL,
    }
}

/// Audits each text whose redaction scan limit was reached in `ingest`,
/// a dropped text and not a redaction (SEC-08, I9, I6).
pub(crate) fn audit_scan_limited(ctx: &Context, texts: usize, stderr: &mut dyn Write) {
    let occurrence = scan_limited_occurrence("ingest");
    for _ in 0..texts {
        if let Err(e) = ctx.audit(&occurrence) {
            let _ = writeln!(stderr, "parinn ingest: audit a limited redaction: {e}");
        }
    }
}

/// Audits each log field that reached the redaction scan limit.
pub fn audit_limited_log_fields(ctx: &Context, limited: u64) {
    for _ in 0..limited {
        let _ = ctx.audit(&scan_limited_occurrence("log"));
    }
}

/// Audits each settings layer that fails and each key ignored before a
/// recording command goes on recording whatever they say (ADM-04, I9);
/// says on `stderr` what cannot be read or audited. Returns the
/// settings in force.
pub(crate) fn audit_settings(
    ctx: &Context,
    home: &Home,
    stderr: &mut dyn Write,
) -> settings::InForce {
    let acts = settings::recorded_acts(home).unwrap_or_else(|e| {
        let _ = writeln!(
            stderr,
            "parinn ingest: read the accepted configuration: {e}"
        );
        Vec::new()
    });
    let in_force = settings::in_force(ctx, home, &acts);
    // Managed policy that fails leaves the core recording (ADM-04); a
    // failing principal layer refuses the ingest after this.
    for failure in &in_force.loaded.failures {
        let line = match failure.layer {
            configuration::SettingsLayer::ManagedPolicy => format!("recording without {failure}"),
            _ => failure.to_string(),
        };
        let _ = writeln!(stderr, "parinn ingest: {line}");
    }
    for e in settings::audit(ctx, "ingest", &in_force) {
        let _ = writeln!(stderr, "parinn ingest: audit the settings: {e}");
    }

    in_force
}
