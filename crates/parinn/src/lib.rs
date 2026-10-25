//! The ParleyInn core executable, specified in docs/srs: the hook
//! handlers, the MCP server and the CLI so far.
//!
//! It runs `version`, `ingest`, `mcp`, `verify`, `hook`,
//! `status`, `doctor`, `purge`, `quarantine`, and the principal acts
//! `counter ack`, `configuration accept` and `risk-acceptance`. Every other
//! command in the SRS's §9.5 is a pending scenario under features/, and
//! lands with the plan that makes its scenario pass.

// A lint the core's code would quiet says so with an expectation and a
// reason, which fails once nothing trips it; a bare allow does not
// (SEC-01, ENG-16).
#![deny(clippy::allow_attributes)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use administration::act as act_record;
use administration::home::Home;
use administration::node::Node;
use administration::node_identity::IdentitySource;
use administration::verify::{Report, Shape, VerifyError, verify};
use allowed_root::{Access, AllowedPath, AllowedRoots};
use audit::AuditError;
use audit::counters::{Counts, counts};
use audit::entry::{Counter, Entry, Kind, Occurrence};
use audit::log::Log;
use configuration::source::OsUser;
use configuration::{Loaded, SettingsLayer};
use hook::Failure;
use key::key_store::KeyStoreKind;
use key::random::OsEntropy;
use redaction::Redactor;
use serde_json::{Value, json};
use store::Store;

/// The command did what it was asked.
pub const EXIT_OK: u8 = 0;
/// The command failed.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;
/// The record failed an integrity check.
pub const EXIT_INTEGRITY: u8 = 3;

/// How long `parinn mcp` waits for a line before it waits again.
const IDLE: Duration = Duration::from_secs(1);

/// The longest line the process's input may carry, the hook input's
/// limit (SEC-05); the MCP server's requests are held to it too.
pub const INPUT_LINE_LIMIT: usize = hook::HOOK_INPUT_LIMIT;

/// The longest `parinn hook` waits for its whole hook input, however its
/// lines arrive; its hook budget ends the wait sooner (NFR-02).
const HOOK_INPUT_WAIT: Duration = Duration::from_millis(500);

/// What a build stamps into the executable. The release build sets
/// `PARINN_VERSION` and `PARINN_COMMIT` (ENG-01); a local build reports
/// version "dev" and no commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Build<'a> {
    pub version: &'a str,
    pub commit: Option<&'a str>,
}

/// The version a build reports when the release pipeline stamps none.
const BUILD_DEV: &str = "dev";

/// This executable's build.
pub const BUILD: Build<'static> = Build {
    version: match option_env!("PARINN_VERSION") {
        Some(v) => v,
        None => BUILD_DEV,
    },
    commit: option_env!("PARINN_COMMIT"),
};

const USAGE: &str = "usage: parinn <command> [flags]

commands:
  configuration accept [--json]          accept the configuration (widening)
  counter ack [--json]                   acknowledge the counters (neutral)
  doctor [--json]                        fail while a counter rose since the last acknowledgement
  event search WORDS [--room ID] [--k N] find events by their text, as event_search does
  event expand RANGE [--room ID]         read an address range, as event_expand does; --cursor C continues one
  hook <hook>                            run the hook handler on the hook input on stdin
  ingest --all | --path P [--json]       record every transcript under the roots, or the one at P
  licenses                               print the third-party license texts built in
  mcp                                    serve the MCP tools over stdio
  purge --range RANGE [--reason TEXT]    erase the content of an address range (widening)
  quarantine add SELECTOR [--reason T]   hold events out of recall (cut)
  quarantine release SELECTOR            take a quarantine back (widening)
  quarantine list [--json]               show the quarantines in force
  repository show [DIR] [--json]         show the repository identity of DIR, the working directory by default
  repository bind [DIR] [--commit ID]    bind or rebind the repository identity (widening)
  rebuild [--json]                       derive the origins and the search index again from the record
  risk-acceptance show [--json]          show the residual risks open on this node
  risk-acceptance record [--json]        accept the open residual risks (widening)
  risk-acceptance withdraw [--json]      withdraw the risk acceptance (cut)
  status [--json]                        show the home, the store, the settings and the counters
  verify [--json]                        check the record and the audit log
  version [--json]                       print the build version
  help                                   print this message
";

/// What an invocation runs in: the build, the home variables and the
/// process's input, each injected so a test can stand in for them.
pub struct Context<'a> {
    pub build: &'a Build<'a>,
    /// `PARINN_HOME`, as the process found it.
    pub parinn_home: Option<OsString>,
    /// `HOME`, as the process found it.
    pub home: Option<OsString>,
    /// `PARINN_HOME_ID`, the home id the environment supplies (SEC-03).
    pub home_id: Option<OsString>,
    /// Whether the hook handler panics: a development build's seam for
    /// the `@ENG-04` scenario, never set in a release build.
    pub hook_panics: bool,
    /// Whether the hook handler hangs, as one the harness kills before
    /// it finishes: a development build's seam for the `@NFR-02`
    /// scenario, never set in a release build.
    pub hook_hangs: bool,
    /// Whether the process's standard input and output are both a
    /// terminal, which a verb that writes a principal act needs
    /// (OWN-12): a person's are, an agent's tool call's are not.
    pub terminal: bool,
    /// The working directory, whose repository's configuration is a
    /// settings layer (SEC-11).
    pub working_dir: Option<PathBuf>,
    /// Where managed policy is read from (ADM-04).
    pub managed_policy: PathBuf,
    /// Where the node identity is read from instead of this OS's source
    /// (REC-24): a development build's seam for the tests, never set in
    /// a release build.
    pub node_identity: Option<PathBuf>,
    /// The running OS user, whom managed policy's files must not let
    /// write (ADM-04).
    pub os_user: OsUser,
    /// Starts reading the process's input, one line at a time.
    pub input: &'a dyn Fn() -> Receiver<io::Result<String>>,
    /// The wall clock, in seconds since the Unix epoch, which only
    /// audit entries, ingest's holds, start markers and session
    /// bindings read.
    pub clock: &'a dyn Fn() -> u64,
    /// Whether the harness's attended mark is in the process's
    /// environment (ASM-22): only a hook handler reads it (PRV-04).
    pub attended: bool,
    /// A hook budget in place of each hook's own: a development build's
    /// seam for the tests, never set in a release build.
    pub hook_budget: Option<Duration>,
    /// Reads a hook handler's process ancestry (PRV-08), and the MCP
    /// server's own harness process (REC-13): this process's own, or
    /// what a test stands in for it.
    pub ancestry: fn() -> Option<hook::ancestry::Ancestry>,
    /// How many lines of a transcript the MCP server's live ingest reads
    /// at most: a development build's seam for the tests, never set in a
    /// release build.
    pub live_limit: Option<usize>,
    /// How long a search may run before it is cancelled, in place of
    /// the default 2 s (SEC-04): a development build's seam for the
    /// tests, never set in a release build.
    pub query_deadline: Option<Duration>,
}

impl Context<'_> {
    /// The home, checked before anything reads or writes in it: it and
    /// everything in it owner-only (SEC-02), and its home id the one the
    /// environment supplies (SEC-03).
    fn home(&self) -> Result<Home, administration::AdminError> {
        let mut home = Home::locate(self.parinn_home.as_deref(), self.home.as_deref())?;
        if let Some(path) = &self.node_identity {
            home = home.with_identity(IdentitySource::Files(vec![path.clone()]));
        }
        home.check()?;
        home.check_home_id(self.home_id.as_deref())?;

        Ok(home)
    }

    /// Appends `occurrence` to the checked home's audit log, making the
    /// home first where it does not exist yet.
    fn audit(&self, occurrence: &Occurrence) -> Result<Entry, Box<dyn std::error::Error>> {
        let home = self.home()?;
        home.create()?;

        Ok(Log::new(home.audit()).append(occurrence, (self.clock)())?)
    }
}

/// Where a run's managed policy is, beside the principal's
/// configuration in its home: what the structured log reads the
/// redaction patterns from. A repository's never apply (SEC-08).
#[derive(Debug, Clone)]
pub struct LogSettings {
    pub managed: PathBuf,
    pub os_user: configuration::source::OsUser,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            managed: PathBuf::new(),
            os_user: configuration::source::OsUser::Unchecked,
        }
    }
}

/// The structured log (ENG-05, OPS-05) for the home the environment
/// names: written to its log file once the home exists and passes its
/// checks (SEC-02, SEC-03), and nowhere before; each field redacted by
/// the maintained redaction rule set and the redaction patterns the
/// principal's configuration and managed policy name (SEC-08), compiled
/// at the first event, so a run that logs nothing starts no slower.
/// While the principal's configuration fails, its patterns are unknown,
/// so no line is written; each is counted as dropped.
#[must_use]
pub fn structured_log(
    parinn_home: Option<OsString>,
    home: Option<OsString>,
    home_id: Option<OsString>,
    settings: LogSettings,
    clock: fn() -> u64,
) -> structured_log::StructuredLog {
    let (compile_home, compile_os_home) = (parinn_home.clone(), home.clone());
    // With no home there are no settings layers to read and nowhere to
    // write: the rule set alone, and the log opens nothing.
    let compile: structured_log::Compile =
        Box::new(
            move || match Home::locate(compile_home.as_deref(), compile_os_home.as_deref()) {
                Ok(home) => log_redactor(&home, &settings),
                Err(_) => Redactor::new(&[])
                    .map(|(r, _)| r)
                    .map_err(|e| e.to_string()),
            },
        );
    let open: structured_log::Open = Box::new(move || {
        let Ok(home) = Home::locate(parinn_home.as_deref(), home.as_deref()) else {
            return Ok(None);
        };
        if !home.root().exists() || home.check().is_err() {
            return Ok(None);
        }
        if home.check_home_id(home_id.as_deref()).is_err() {
            return Ok(None);
        }
        let file = structured_log::open_log_file(&home.root().join("logs"))?;
        Ok(Some(Box::new(file)))
    });
    structured_log::StructuredLog::new(open, compile, Box::new(clock))
}

/// The redactor log fields pass through: the rule set and the redaction
/// patterns the principal's configuration and managed policy of `home`
/// name, as written, so a pattern still awaiting acceptance redacts the
/// log too; a repository's never (SEC-08). None while the principal's
/// configuration fails.
fn log_redactor(home: &Home, settings: &LogSettings) -> Result<Redactor, String> {
    let sources = configuration::Sources {
        configuration: home.root().join(configuration::source::CONFIGURATION),
        repository: None,
        managed: settings.managed.clone(),
        running: settings.os_user,
    };
    let loaded = configuration::load(&sources);
    if loaded.principal_layer_failed() {
        return Err("the configuration failed, so its redaction patterns are unknown".to_owned());
    }
    let mut patterns: Vec<String> = Vec::new();
    for layer in [&loaded.layers.configuration, &loaded.layers.managed] {
        let named = configuration::key::key("redaction.extra_patterns").and_then(|k| layer.get(&k));
        if let Some(configuration::Value::Texts(items)) = named {
            patterns.extend(
                items
                    .iter()
                    .filter(|p| !patterns.contains(p))
                    .cloned()
                    .collect::<Vec<_>>(),
            );
        }
    }

    Redactor::new(&patterns)
        .map(|(r, _)| r)
        .map_err(|e| e.to_string())
}

/// Audits the log lines a run could not write (I6).
pub fn audit_dropped_log_lines(ctx: &Context, dropped: u64) {
    for _ in 0..dropped {
        let _ = ctx.audit(&Occurrence::Degraded {
            kind: Kind::Dropped,
            counter: Counter::LogLineDropped,
            operation: "log",
            detail: "log line dropped",
        });
    }
}

/// Runs one invocation and returns its exit code. It is `main` without
/// the process: every test drives the command line through here.
pub fn run(
    args: impl IntoIterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        None => {
            let _ = stderr.write_all(USAGE.as_bytes());
            EXIT_USAGE
        }
        Some("configuration") => act::configuration(args, ctx, stdout, stderr),
        Some("counter") => act::counter(args, ctx, stdout, stderr),
        Some("doctor") => doctor(args, ctx, stdout, stderr),
        Some("event") => event::event(args, ctx, stdout, stderr),
        Some("hook") => hook_handlers::hook_command(args.next().as_deref(), ctx, stdout),
        Some("ingest") => ingest(args, ctx, stdout, stderr),
        Some("licenses") => licenses::licenses(args, stdout, stderr),
        Some("mcp") => mcp(args, ctx, stdout, stderr),
        Some("purge") => purge::purge(args, ctx, stdout, stderr),
        Some("quarantine") => quarantine::quarantine(args, ctx, stdout, stderr),
        Some("rebuild") => rebuild::rebuild(args, ctx, stdout, stderr),
        Some("repository") => repository_identity::repository(args, ctx, stdout, stderr),
        Some("risk-acceptance") => act::risk_acceptance(args, ctx, stdout, stderr),
        Some("status") => status(args, ctx, stdout, stderr),
        Some("verify") => verify_command(args, ctx, stdout, stderr),
        Some("version") => version(args, ctx.build, stdout, stderr),
        Some("help" | "-h" | "--help") => written(stdout.write_all(USAGE.as_bytes()), stderr),
        Some(other) => {
            let _ = write!(stderr, "parinn: unknown command {other:?}\n\n{USAGE}");
            EXIT_USAGE
        }
    }
}

/// `parinn version [--json]`: the stamped version as a bare line, or as
/// `{"version": ..., "commit": ...}`, the commit left out when the
/// build stamped none.
fn version(
    args: impl Iterator<Item = String>,
    build: &Build,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("version", args, stderr) else {
        return EXIT_USAGE;
    };
    let line = if as_json {
        version_json(build)
    } else {
        build.version.to_owned()
    };

    written(writeln!(stdout, "{line}"), stderr)
}

/// The JSON object `parinn version --json` prints.
fn version_json(build: &Build) -> String {
    let mut out = format!("{{\"version\":{}", json_string(build.version));
    if let Some(commit) = build.commit {
        out.push_str(&format!(",\"commit\":{}", json_string(commit)));
    }
    out.push('}');

    out
}

/// The flags a command takes, with `--json` among them; anything else
/// is a usage error, reported on `stderr`.
pub(crate) fn json_flag(
    command: &str,
    args: impl Iterator<Item = String>,
    stderr: &mut dyn Write,
) -> Option<bool> {
    let mut as_json = false;
    for arg in args {
        if arg != "--json" {
            let _ = writeln!(stderr, "parinn {command}: unknown flag {arg:?}");
            return None;
        }
        as_json = true;
    }

    Some(as_json)
}

/// Logs the failure of `result`, if it failed, as what `what` says: for
/// work that must not stop the command it runs in (I9).
pub(crate) fn logged<T, E: std::fmt::Display>(what: &str, result: Result<T, E>) {
    if let Err(e) = result {
        tracing::warn!(error = e.to_string().as_str(), what, "failed");
    }
}

/// Reports a failure of `command` on `stderr`.
pub(crate) fn failed(command: &str, e: &dyn std::fmt::Display, stderr: &mut dyn Write) -> u8 {
    let _ = writeln!(stderr, "parinn {command}: {e}");
    EXIT_FAILURE
}

/// Reports a store that failed to open: one of a newer schema version is
/// an integrity failure, refused and named, exiting 3 (ADM-05); any other
/// a failure.
fn store_failed(command: &str, e: &store::StoreError, stderr: &mut dyn Write) -> u8 {
    let code = failed(command, e, stderr);
    if matches!(e, store::StoreError::Newer { .. }) {
        EXIT_INTEGRITY
    } else {
        code
    }
}

/// The payload threshold the settings in force set (REC-09).
fn payload_settings(settings: &configuration::resolve::Settings) -> ingest::PayloadSettings {
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
enum IngestTarget {
    /// Every transcript under the transcript roots (REC-01).
    All,
    /// The transcript at this path.
    Path(String),
}

/// `parinn ingest --all | --path P [--json]`: records every transcript
/// under the transcript roots, or the one at `P`, each in a new ingest
/// seat. A transcript that cannot be ingested is reported and the rest
/// still are; the command then fails.
fn ingest(
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
    let mut failures = 0;
    let mut allowed = Vec::new();
    for path in &candidates {
        match allowed_transcript(ctx, settings, path, stderr) {
            Ok(transcript) => allowed.push(transcript),
            Err(e) => {
                failed("ingest", &e, stderr);
                failures += 1;
            }
        }
    }
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
        // the store itself (REC-04).
        forks: std::collections::BTreeSet::new(),
        now_ms: (ctx.clock)().saturating_mul(1000),
        interactive: settings.deployment_mode() == "interactive",
    };
    let paths: Vec<&Path> = allowed.iter().map(AllowedPath::path).collect();
    let (reports, failed_ingests) = ingest_all(ctx, &home, &paths, &redactor, &inputs, stderr);
    failures += failed_ingests;
    let held = held_sources(ctx, settings, &home, stderr);
    let held: Vec<&Path> = held.iter().map(AllowedPath::path).collect();
    failures += ingest_all(ctx, &home, &held, &redactor, &inputs, stderr).1;
    apply_retention(ctx, &home, settings, stderr);
    if target == IngestTarget::All {
        audit_lost(ctx, &home, stderr);
    }
    let shown = ingest_report(&target, &reports, failures, as_json);
    let code = written(write!(stdout, "{shown}"), stderr);

    if failures > 0 { EXIT_FAILURE } else { code }
}

/// Applies the retention policies in force once the ingest is done
/// (SEC-22); a failure to open the store again is audited and counted,
/// and stops nothing (I9).
fn apply_retention(
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
fn ingest_all(
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
fn count_untaken_bindings(ctx: &Context, store: &mut Store) {
    hook_handlers::count_untaken(store, (ctx.clock)(), &|o| {
        let _ = ctx.audit(o);
    });
}

/// The transcript sources an ingest marker holds, each resolved again
/// within the transcript roots: what a harness fork read before its
/// owner left waiting, read again once its owner is ingested or gone
/// (REC-04, REC-08).
fn held_sources(
    ctx: &Context,
    settings: &configuration::resolve::Settings,
    home: &Home,
    stderr: &mut dyn Write,
) -> Vec<AllowedPath> {
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

    held.iter()
        .filter_map(|path| allowed_transcript(ctx, settings, path, stderr).ok())
        .collect()
}

/// Audits and counts, once each, every transcript source `parinn ingest
/// --all` finds gone before it was fully ingested (REC-08, I6).
fn audit_lost(ctx: &Context, home: &Home, stderr: &mut dyn Write) {
    let found = Store::open(&home.store()).and_then(|mut store| {
        let sources = store.sources()?;
        let fresh: Vec<store::Source> = sources::lost(&sources)
            .into_iter()
            .filter(|s| !s.lost)
            .cloned()
            .collect();
        for source in &fresh {
            store.lose(source.id)?;
        }
        Ok(fresh)
    });
    let fresh = match found {
        Ok(fresh) => fresh,
        Err(e) => {
            let _ = writeln!(stderr, "parinn ingest: find lost transcript sources: {e}");
            return;
        }
    };
    for source in fresh {
        let name = Path::new(&source.path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let occurrence = Occurrence::Keyed {
            kind: Kind::Dropped,
            counter: Counter::TranscriptSourceLost,
            operation: "ingest",
            detail: "transcript source gone before it was fully ingested:",
            key: &name,
        };
        if let Err(e) = ctx.audit(&occurrence) {
            let _ = writeln!(stderr, "parinn ingest: audit a lost transcript source: {e}");
        }
    }
}

/// What `parinn ingest`'s command line asks for, or why it is refused.
fn ingest_args(mut args: impl Iterator<Item = String>) -> Result<(IngestTarget, bool), String> {
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
fn root_transcripts(
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
fn audit_index_gaps(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
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
fn record_ingested(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
    audit_redactions(ctx, &ingested.redactions, stderr);
    audit_scan_limited(ctx, ingested.scan_limited, stderr);
    audit_parsing(ctx, ingested, stderr);
    audit_sourcing(ctx, ingested, stderr);
    audit_prompts(ctx, ingested, stderr);
    audit_index_gaps(ctx, ingested, stderr);
    tracing::info!(
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

/// What `parinn ingest` prints of the transcripts it recorded: for
/// `--path`, the one; for `--all`, each, or that there was none.
fn ingest_report(
    target: &IngestTarget,
    reports: &[ingest::Ingested],
    failures: usize,
    as_json: bool,
) -> String {
    let one = |i: &ingest::Ingested| {
        json!({
            "writer": i.writer.to_string(),
            "seat": i.seat.to_string(),
            "room": i.room.to_string(),
            "events": i.events,
            "generation": i.generation,
            "held": i.held,
            "runs": runs_json(&i.runs),
        })
    };
    if as_json && *target == IngestTarget::All {
        let all: Vec<Value> = reports.iter().map(one).collect();
        return format!("{}\n", json!({"transcripts": all}));
    }
    if reports.is_empty() && failures == 0 && *target == IngestTarget::All {
        return "ingested no transcript\n".to_owned();
    }
    reports
        .iter()
        .map(|i| {
            if as_json {
                format!("{}\n", one(i))
            } else {
                format!("ingested {} lines into writer {}\n", i.events, i.writer)
            }
        })
        .collect()
}

/// Each run an ingest appended to, as `parinn ingest --json` shows it:
/// its writer and seat, the harness's agent id, the run its parent link
/// ties it to, and how many events it gained (REC-02).
fn runs_json(runs: &[ingest::RunReport]) -> Value {
    runs.iter()
        .map(|r| {
            json!({
                "writer": r.writer.to_string(),
                "seat": r.seat.to_string(),
                "agent_id": r.agent_id,
                "parent": r.parent.map(|p| p.to_string()),
                "events": r.events,
            })
        })
        .collect()
}

/// Audits what an ingest found of its transcript source, one entry
/// each, naming the source by its redacted name: a new transcript
/// generation (REC-07); each line repeating a `uuid` with other
/// content, each repeat owner no fork observation chose, a hold begun
/// and each line released from one (REC-04).
fn audit_sourcing(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
    let keyed = |kind, counter, detail| Occurrence::Keyed {
        kind,
        counter,
        operation: "ingest",
        detail,
        key: &ingested.name,
    };
    let occurrences = [
        (
            usize::from(ingested.new_generation),
            keyed(
                Kind::Truncated,
                Counter::NewTranscriptGeneration,
                "new transcript generation of",
            ),
        ),
        (
            ingested.conflicts,
            keyed(
                Kind::Rejected,
                Counter::RepeatConflicts,
                "repeated uuid with other content, recorded as its own event, in",
            ),
        ),
        (
            ingested.owners_chosen,
            keyed(
                Kind::Coalesced,
                Counter::RepeatOwnerChosen,
                "repeated lines owned by the run that first recorded them, repeated in",
            ),
        ),
        (
            usize::from(ingested.hold_started),
            keyed(
                Kind::Truncated,
                Counter::RepeatHeld,
                "repeated lines held at an ingest marker in",
            ),
        ),
        (
            ingested.released,
            keyed(
                Kind::TimedOut,
                Counter::RepeatReleased,
                "repeated line released into its own run in",
            ),
        ),
    ];
    for (times, occurrence) in &occurrences {
        for _ in 0..*times {
            if let Err(e) = ctx.audit(occurrence) {
                let _ = writeln!(stderr, "parinn ingest: audit the transcript source: {e}");
            }
        }
    }
}

/// Audits each prompt line an ingest found carrying a `prompt_id`
/// already used up (PRV-08), and each matched `user` event whose hook
/// handler found no attended mark in deployment mode `interactive`,
/// which stays untrusted (PRV-04).
fn audit_prompts(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
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
fn audit_parsing(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
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
fn start_node(ctx: &Context, home: &Home, stderr: &mut dyn Write) -> Result<Store, String> {
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
fn refuse_ingest(ctx: &Context, stderr: &mut dyn Write) -> String {
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

/// The transcript at `path` resolved within the transcript roots, the
/// only allowed roots `parinn ingest` reads from (SEC-18): nothing in
/// the home, its keys and store among it, is a transcript. A path
/// outside them is refused, audited and counted, and nothing is read
/// from it.
fn allowed_transcript(
    ctx: &Context,
    settings: &configuration::resolve::Settings,
    path: &Path,
    stderr: &mut dyn Write,
) -> Result<AllowedPath, String> {
    let os_home = ctx.home.as_deref().map(Path::new);
    let allowed = settings
        .texts("transcript.roots")
        .iter()
        .fold(AllowedRoots::default(), |allowed, root| {
            allowed.with(&allowed_root::expand_home(root, os_home), Access::Read)
        });
    allowed.resolve(path, Access::Read).map_err(|refused| {
        let occurrence = Occurrence::Degraded {
            kind: Kind::Rejected,
            counter: Counter::PathOutsideAllowedRoots,
            operation: "ingest",
            detail: "path outside allowed roots",
        };
        if let Err(e) = ctx.audit(&occurrence) {
            let _ = writeln!(
                stderr,
                "parinn ingest: audit a path outside allowed roots: {e}"
            );
        }
        refused.to_string()
    })
}

/// The redactor for the settings in force: the rule set and the
/// principal's redaction patterns (SEC-08), each pattern refused
/// audited and counted while the rest still apply.
fn redactor(
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
fn audit_redactions(ctx: &Context, rules: &[String], stderr: &mut dyn Write) {
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
fn audit_scan_limited(ctx: &Context, texts: usize, stderr: &mut dyn Write) {
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
fn audit_settings(ctx: &Context, home: &Home, stderr: &mut dyn Write) -> settings::InForce {
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

/// `parinn mcp`: serves the MCP tools over the process's input and
/// `stdout` until the input closes.
fn mcp(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    if let Some(arg) = args.into_iter().next() {
        let _ = writeln!(stderr, "parinn mcp: unknown flag {arg:?}");
        return EXIT_USAGE;
    }
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("mcp", &e, stderr),
    };
    // The MCP server counts each start marker a stopped hook handler
    // left behind, as it starts (NFR-02).
    if home.root().exists() {
        hook_handlers::count_unfinished(&home, (ctx.clock)(), None);
    }
    // It records its harness session's runs, so it starts the node as
    // ingest does (REC-13).
    let served = start_node(ctx, &home, stderr).and_then(|mut store| {
        repository_identity::bind_observed(ctx, &home, &mut store);
        let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
        let server = mcp::Server::new(
            store,
            ctx.build.version,
            node.device().public(),
            node.personal_room(),
        );
        Ok((server, node.device_writer()))
    });
    let (server, device_writer) = match served {
        Ok(served) => served,
        Err(e) => return failed("mcp", &e, stderr),
    };
    // What an MCP server killed before it stopped left is audited by
    // the next one, as it starts (REC-13).
    capture::count_stopped(&home, "mcp", &|o| {
        let _ = ctx.audit(o);
    });
    let shared = std::sync::Arc::new(capture::Shared::default());
    // Neither a recall event nor a line of a transcript may be written
    // without the redaction patterns in force (SEC-08).
    let in_force = audit_settings(ctx, &home, stderr);
    let redactor = recall::redactor(ctx, &in_force, stderr);
    let mut server = server
        .with_runs(Box::new(capture::Finder(shared.clone())))
        .with_recall_writer(recall::recall_writer(device_writer, redactor.clone()));
    let live = redactor
        .map(|r| live_capture(ctx, &home, &shared, &in_force, r))
        .map(capture::Capture::start)
        .map(spawn_capture);
    if let Some(deadline) = ctx.query_deadline {
        server = server.with_query_deadline(deadline);
    }
    // The panic and the cancelled search each audit, and say on stderr
    // when the audit log takes no entry, so both write through it.
    let stderr = RefCell::new(stderr);
    let (mut panic_err, mut cancel_err) = (Shared(&stderr), Shared(&stderr));
    let mut cancelled = on_query_cancelled(&home, ctx.clock, &mut cancel_err);
    let served = mcp::serve(
        &mut |l| {
            let reply = server.handle(l);
            record_found(ctx, &server.take_found());
            cancelled(server.take_cancelled());
            recall::audit_notes(ctx, &server.take_notes());
            reply
        },
        &(ctx.input)(),
        stdout,
        IDLE,
        &mut on_mcp_panic(&home, ctx.clock, &mut panic_err),
    );
    // Its input closed: live ingest stops, audits what it left and closes
    // its segments, within a bound; past it the next reader audits it.
    shared.stop();
    if let Some(done) = live {
        let _ = done.recv_timeout(CAPTURE_STOP);
    }
    match served {
        Ok(()) => EXIT_OK,
        Err(e) => failed("mcp", &e, &mut **stderr.borrow_mut()),
    }
}

/// How long `parinn mcp` waits for live ingest to stop once its input
/// closes.
const CAPTURE_STOP: Duration = Duration::from_secs(5);

/// The MCP server's live ingest in `home` (REC-13): its own harness
/// process's session bindings, its transcripts resolved within the
/// transcript roots, redacted by `redactor`, the patterns of the
/// settings in force.
fn live_capture(
    ctx: &Context,
    home: &Home,
    shared: &std::sync::Arc<capture::Shared>,
    in_force: &settings::InForce,
    redactor: Redactor,
) -> capture::Capture {
    let settings = &in_force.resolved.settings;
    let os_home = ctx.home.as_deref().map(Path::new);
    let roots = settings
        .texts("transcript.roots")
        .iter()
        .fold(AllowedRoots::default(), |allowed, root| {
            allowed.with(&allowed_root::expand_home(root, os_home), Access::Read)
        });
    let base = (ctx.clock)();
    let started = now_instant();

    capture::Capture {
        ceiling: ctx.home.as_deref().map(PathBuf::from),
        home: home.clone(),
        roots,
        redactor,
        inputs: ingest::Inputs {
            payload: payload_settings(settings),
            forks: std::collections::BTreeSet::new(),
            now_ms: base.saturating_mul(1000),
            interactive: settings.deployment_mode() == "interactive",
        },
        harness: (ctx.ancestry)().and_then(|a| a.harness),
        limit: ctx.live_limit,
        clock: Box::new(move || base + started.elapsed().as_secs()),
        shared: shared.clone(),
    }
}

/// Now, for live ingest's audit clock.
#[expect(
    clippy::disallowed_methods,
    reason = "live ingest's audit entries record when they were made; no derived artifact \
              reads it (I10)"
)]
fn now_instant() -> Instant {
    Instant::now()
}

/// Runs `capture` on a thread of its own; what it sends tells it ended.
fn spawn_capture(capture: capture::Running) -> Receiver<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        capture.run();
        let _ = tx.send(());
    });

    rx
}

/// Logs what finding each tool call's run came to (PRV-14, OPS-05),
/// through redaction: its tool-use id, the run or why none was found,
/// and how long it waited; each refusal counted and audited (I6).
fn record_found(ctx: &Context, found: &[mcp::tool_call::Found]) {
    use mcp::tool_call::Cause;
    for f in found {
        let id = f.tool_use_id.as_deref().unwrap_or_default();
        let waited_ms = u64::try_from(f.waited.as_millis()).unwrap_or(u64::MAX);
        let cause = match &f.run {
            Ok(run) => {
                tracing::info!(tool_use_id = id, run = run.as_str(), waited_ms, "tool call");
                continue;
            }
            Err(cause) => *cause,
        };
        tracing::info!(
            tool_use_id = id,
            cause = cause.as_str(),
            waited_ms,
            "tool call refused"
        );
        let counter = match cause {
            Cause::NoToolUseId => Counter::ToolCallWithoutToolUseId,
            Cause::NotFound => Counter::ToolCallRunNotFound,
            Cause::Ambiguous => Counter::ToolCallRunAmbiguous,
        };
        let _ = ctx.audit(&Occurrence::Degraded {
            kind: Kind::Rejected,
            counter,
            operation: "mcp",
            detail: "a tool call whose run could not be found was refused",
        });
    }
}

/// A writer two closures share, each writing in turn.
struct Shared<'a, 'w>(&'a RefCell<&'w mut dyn Write>);

impl Write for Shared<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.borrow_mut().flush()
    }
}

/// What the MCP server does with each search the deadline cancelled:
/// audit and count it, saying on `stderr` when the audit log takes no
/// entry (SEC-04, I6).
fn on_query_cancelled<'a>(
    home: &'a Home,
    clock: &'a dyn Fn() -> u64,
    stderr: &'a mut dyn Write,
) -> impl FnMut(u64) + 'a {
    move |cancelled| {
        for _ in 0..cancelled {
            if let Err(e) = query_cancelled(home, clock()) {
                let _ = writeln!(stderr, "parinn mcp: audit a cancelled search: {e}");
            }
        }
    }
}

/// Audits a search the deadline cancelled.
fn query_cancelled(home: &Home, time: u64) -> Result<Entry, AuditError> {
    let occurrence = Occurrence::Degraded {
        kind: Kind::TimedOut,
        counter: Counter::QueryCancelled,
        operation: "mcp",
        detail: "query cancelled at its deadline",
    };

    Log::new(home.audit()).append(&occurrence, time)
}

/// `parinn verify [--json]`: checks the record against this node's
/// device key, exiting 3 at the first failure, which it names. No log
/// is signed yet, so a record that passes is `unsigned` (§9.7.5). It
/// changes nothing it checks: a purge a kill cut short is reported,
/// with the search index its residue leaves unchecked, for the next
/// purge, ingest or rebuild to finish (ADM-09, ENG-06).
fn verify_command(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("verify", args, stderr) else {
        return EXIT_USAGE;
    };
    let (home, store) = match verified_store(ctx, stderr) {
        Ok(opened) => opened,
        Err(code) => return code,
    };
    let incomplete = match completeness(ctx, &home, &store) {
        Ok(found) => found,
        Err(e) => return failed("verify", &e, stderr),
    };
    let read = store
        .refused_forks()
        .and_then(|forks| Ok((forks, unfinished_purges(ctx, &store)?)));
    let (forks, unfinished) = match read {
        Ok(read) => read,
        Err(e) => return failed("verify", &e, stderr),
    };
    let mut shape = Shape::Checked;
    let verified = match verify_record(&home, &store) {
        Ok(verified) => verified.and_then(|(r, staged)| {
            shape = checked_shape(ctx, &store)?;
            Ok((r, staged))
        }),
        Err(e) => return failed("verify", &e, stderr),
    };
    let checked = match verified {
        Ok((r, staged)) => Log::new(home.audit())
            .verify()
            .map(|a| (r, staged, a))
            .map_err(Err),
        Err(e) => Err(Ok(e)),
    };
    let (code, line) = match checked {
        Ok((r, staged, _)) if !forks.is_empty() => {
            refused_report(as_json, &r, staged, &forks, stderr)
        }
        Ok(_) if incomplete.failure().is_some() => broken_json(
            as_json,
            "transcript_source_completeness",
            "transcript source completeness",
            None,
            &incomplete.failure().unwrap_or_default(),
            stderr,
        ),
        Ok((r, staged, a)) => (
            EXIT_OK,
            verified_line(
                as_json,
                &r,
                staged,
                &a,
                &incomplete,
                Checks {
                    shape,
                    unfinished: &unfinished,
                },
            ),
        ),
        Err(Ok(VerifyError::Store(e))) => return failed("verify", &e, stderr),
        Err(Err(e @ AuditError::Broken { .. })) => {
            broken_json(as_json, "audit", "hash chain", None, &e.to_string(), stderr)
        }
        Err(Err(e)) => return failed("verify", &e, stderr),
        Err(Ok(broken)) => broken_json(
            as_json,
            broken.class(),
            broken.check(),
            broken.address(),
            &broken.to_string(),
            stderr,
        ),
    };
    if line.is_empty() {
        return code;
    }

    match written(writeln!(stdout, "{line}"), stderr) {
        EXIT_OK => code,
        other => other,
    }
}

/// The home and the store `parinn verify` checks, or the exit code of
/// the failure it reported on `stderr`.
fn verified_store(ctx: &Context, stderr: &mut dyn Write) -> Result<(Home, Store), u8> {
    let home = ctx.home().map_err(|e| failed("verify", &e, stderr))?;
    if !home.store().exists() {
        let missing = format!("no store at {}", home.store().display());
        return Err(failed("verify", &missing, stderr));
    }
    let store = Store::open(&home.store()).map_err(|e| store_failed("verify", &e, stderr))?;

    Ok((home, store))
}

/// How far the transcript sources of `store` fall short (ADM-09), as
/// `parinn verify` finds them now.
fn completeness(
    ctx: &Context,
    home: &Home,
    store: &Store,
) -> Result<sources::Completeness, store::StoreError> {
    Ok(sources::completeness(
        &store.sources()?,
        &capture::served(home),
        &sources::look,
        (ctx.clock)().saturating_mul(1000),
    ))
}

/// The ranges a purge a kill cut short has still to clear from the
/// search index, audited and counted when there are any (I6): the next
/// purge, ingest or rebuild finishes them.
fn unfinished_purges(ctx: &Context, store: &Store) -> Result<Vec<String>, store::StoreError> {
    let unfinished = store.unfinished_purges()?;
    if !unfinished.is_empty() {
        purge::audit_residue(ctx);
    }

    Ok(unfinished)
}

/// What `parinn verify` checked beside the record: the store's shape,
/// and the ranges an unfinished purge leaves in the search index.
#[derive(Debug, Clone, Copy)]
struct Checks<'a> {
    shape: Shape,
    unfinished: &'a [String],
}

/// How long `parinn verify` derives the search index again before it
/// leaves the index unchecked (ADM-09, ENG-03), in seconds on its clock.
const INDEX_CHECK_SECS: u64 = 60;

/// The shape of `store`, its search index derived again while
/// [`INDEX_CHECK_SECS`] last on `ctx`'s clock and compared outside the
/// ranges a pending purge has still to clear, whose residue it holds; a
/// derivation that ran out of time is audited and counted (I6).
fn checked_shape(ctx: &Context, store: &Store) -> Result<Shape, VerifyError> {
    let ends = (ctx.clock)().saturating_add(INDEX_CHECK_SECS);
    let shape = administration::verify::verify_shape(store, &|| (ctx.clock)() < ends)?;
    if shape == Shape::IndexUnfinished {
        let _ = ctx.audit(&Occurrence::Degraded {
            kind: Kind::TimedOut,
            counter: Counter::VerifyTimedOut,
            operation: "verify",
            detail: "the search index was not derived again in time and is unchecked",
        });
    }

    Ok(shape)
}

/// The record in `store`, verified against the device key `home` keeps,
/// or against the key an interrupted first start staged, which its next
/// start places, with whether it was staged; an empty record while that
/// start has recorded nothing yet, as a crash can leave a home (ENG-06).
fn verify_record(
    home: &Home,
    store: &Store,
) -> Result<Result<(Report, bool), VerifyError>, administration::AdminError> {
    Ok(match administration::node::verifying_key(home, store)? {
        Some((key, staged)) => {
            let verified = verify(store, &key);
            // The worktrees' names derive from the device key, which a
            // staged first start has not placed yet.
            if verified.is_ok() && !staged {
                let node = Node::find(home, store)?;
                if let Err(e) = administration::verify::kept_worktrees_agree(store, &node) {
                    return Ok(Err(e));
                }
            }
            verified.map(|r| (r, staged))
        }
        None => Ok((
            Report {
                writers: 0,
                events: 0,
                order: Vec::new(),
                rooms: BTreeMap::new(),
                seats: Vec::new(),
                open_segments: Vec::new(),
            },
            true,
        )),
    })
}

/// What `parinn verify` says of a record that verified, `staged` while
/// the node's first start is still to finish, as JSON when asked.
fn verified_line(
    as_json: bool,
    r: &Report,
    staged: bool,
    a: &audit::log::Checked,
    incomplete: &sources::Completeness,
    checks: Checks<'_>,
) -> String {
    let index_checked = checks.shape == Shape::Checked;
    if as_json {
        return json!({
            "integrity": "unsigned",
            "writers": r.writers,
            "events": r.events,
            "order": r.full_order(),
            "seats": seats_json(&r.seats),
            "open_segments": segments::open_json(&r.open_segments),
            "short_sources": incomplete.short,
            "pending_purges": checks.unfinished,
            "index_checked": index_checked,
            "index_checked_outside_pending_purges": index_checked && !checks.unfinished.is_empty(),
            "staged": staged,
            "audit": {"files": a.files, "entries": a.entries},
            "refused": [],
            "broken": null,
        })
        .to_string();
    }
    let staged = if staged {
        "; the node's first start was interrupted, and its next start finishes it"
    } else {
        ""
    };
    let pending = if checks.unfinished.is_empty() {
        String::new()
    } else if index_checked {
        format!(
            "; a purge was cut short before it finished, so its residue stays until the next \
             purge, ingest or rebuild clears it, and the search index is checked outside its \
             ranges: {}",
            checks.unfinished.join(", ")
        )
    } else {
        format!(
            "; a purge was cut short before it finished, so its residue stays until the next \
             purge, ingest or rebuild clears it: {}",
            checks.unfinished.join(", ")
        )
    };
    let unchecked = if index_checked {
        ""
    } else {
        "; the search index was not derived again in time, so it is unchecked: \
         run parinn verify again"
    };

    format!(
        "checked {} writers and {} events: every chain, commitment and seat certificate \
         holds; integrity unsigned, as no log is signed yet, so a log cut at its tail or \
         rewritten whole goes unseen until seals; completeness covers only the transcript \
         sources the record names; the audit log's chain holds over {} entries{}{}{pending}{unchecked}\
         {staged}",
        r.writers,
        r.events,
        a.entries,
        open_text(&r.open_segments),
        incomplete.short_text()
    )
}

/// What `parinn verify` says of a record whose chains hold but which
/// refused a second event at one (writer, seq): each fork on `stderr`,
/// exiting 3, with the integrity status `refused` and the forks as JSON
/// when asked (PEER-10, VIEW-10). The writer is not marked, and no
/// choice between forks is offered: only the two digests are shown.
fn refused_report(
    as_json: bool,
    r: &Report,
    staged: bool,
    forks: &[store::Fork],
    stderr: &mut dyn Write,
) -> (u8, String) {
    for line in forks::lines(forks) {
        let _ = writeln!(stderr, "parinn verify: {line}");
    }
    if !as_json {
        return (EXIT_INTEGRITY, String::new());
    }
    let line = json!({
        "integrity": "refused",
        "writers": r.writers,
        "events": r.events,
        "staged": staged,
        "refused": forks::json(forks),
        "broken": null,
    });

    (EXIT_INTEGRITY, line.to_string())
}

/// What `parinn verify` adds for writer chains that ended without a
/// closed segment: nothing when none did (REC-19).
fn open_text(open: &[store::Segment]) -> String {
    if open.is_empty() {
        return String::new();
    }

    format!(
        "; writer chains ended without a closed segment: {}",
        segments::open_list(open)
    )
}

/// Each seat as `parinn verify --json` reports it: its writer, and its
/// seat id, room, seat kind, ingest flag and seat certificate, all read
/// from that certificate (PRV-11).
fn seats_json(seats: &[administration::verify::SeatReport]) -> Value {
    seats
        .iter()
        .map(|s| {
            json!({
                "writer": s.writer.to_string(),
                "seat": s.seat.to_string(),
                "room": s.room.to_string(),
                "seat_kind": s.seat_kind.as_str(),
                "ingest_seat": s.ingest_seat,
                "certificate": s.certificate,
            })
        })
        .collect()
}

/// Reports a broken record or audit log on `stderr`, exiting 3, with
/// the failure's class, address and message as JSON when asked; the
/// line is empty when no JSON was asked for.
fn broken_json(
    as_json: bool,
    class: &str,
    check: &str,
    address: Option<&str>,
    message: &str,
    stderr: &mut dyn Write,
) -> (u8, String) {
    let _ = writeln!(stderr, "parinn verify: {check}: {message}");
    if !as_json {
        return (EXIT_INTEGRITY, String::new());
    }
    let line = json!({
        "integrity": "broken",
        "writers": null,
        "events": null,
        "order": null,
        "seats": null,
        "audit": null,
        "broken": {"class": class, "check": check, "address": address, "message": message},
    });

    (EXIT_INTEGRITY, line.to_string())
}

/// The development build's seam for the `@ENG-04` scenario: the hook
/// handler panics.
#[expect(
    clippy::panic,
    reason = "the seam's whole purpose is a panic the hook entry point must catch"
)]
fn seam_panic() {
    panic!("the hook handler panics, as PARINN_TEST_FAULT asked");
}

/// The hook input on stdin, its lines joined, once the input closes
/// by `deadline`. Reading stops once it holds more than 1 MiB (SEC-05).
///
/// # Errors
///
/// [`Failure::TooLarge`] past 1 MiB or for a line the reader cut at its
/// limit; [`Failure::Malformed`] when the input cannot be read or does
/// not close in time.
#[expect(
    clippy::disallowed_methods,
    reason = "the deadline on reading the hook input (ENG-03); no derived artifact reads it (I10)"
)]
fn hook_input(ctx: &Context, deadline: Instant) -> Result<String, Failure> {
    let input = (ctx.input)();
    let (mut text, mut lines) = (String::new(), 0usize);
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .ok_or(Failure::Malformed)?;
        match input.recv_timeout(left) {
            Ok(Ok(line)) => {
                if lines > 0 {
                    text.push('\n');
                }
                text.push_str(&line);
                lines += 1;
                if text.len() > hook::HOOK_INPUT_LIMIT {
                    return Err(Failure::TooLarge);
                }
            }
            Ok(Err(e)) if e.kind() == io::ErrorKind::FileTooLarge => return Err(Failure::TooLarge),
            Ok(Err(_)) | Err(RecvTimeoutError::Timeout) => return Err(Failure::Malformed),
            Err(RecvTimeoutError::Disconnected) => return Ok(text),
        }
    }
}

/// The lines `reader` holds, each without its line ending. A line
/// longer than `limit` bytes is never held whole (SEC-05): an error of
/// kind [`io::ErrorKind::FileTooLarge`] stands in its place, its rest
/// skipped, and the lines after it go on, as they do after one that is
/// not UTF-8. A read that fails ends them.
pub fn bounded_lines<R: io::BufRead>(
    reader: R,
    limit: usize,
) -> impl Iterator<Item = io::Result<String>> {
    let mut reader = Some(reader);
    std::iter::from_fn(move || {
        let r = reader.as_mut()?;
        let mut buf = Vec::new();
        // The limit, and room for a line ending of two bytes.
        let read = io::BufRead::read_until(
            &mut io::Read::take(r.by_ref(), limit as u64 + 2),
            b'\n',
            &mut buf,
        );
        match read {
            Ok(0) => None,
            Ok(_) if buf.last() != Some(&b'\n') && buf.len() > limit => {
                let skipped = io::BufRead::skip_until(r, b'\n');
                Some(skipped.and(Err(io::ErrorKind::FileTooLarge.into())))
            }
            Ok(_) => Some(line_of(buf, limit)),
            Err(e) => {
                reader = None;
                Some(Err(e))
            }
        }
    })
}

/// One line read with its ending, checked against `limit`.
fn line_of(mut buf: Vec<u8>, limit: usize) -> io::Result<String> {
    if buf.last() == Some(&b'\n') {
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop();
        }
    }
    if buf.len() > limit {
        return Err(io::ErrorKind::FileTooLarge.into());
    }

    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// What the MCP server does with each panic it catches: audit it,
/// saying on `stderr` when the audit log takes no entry.
fn on_mcp_panic<'a>(
    home: &'a Home,
    clock: &'a dyn Fn() -> u64,
    stderr: &'a mut dyn Write,
) -> impl FnMut() + 'a {
    move || {
        if let Err(e) = mcp_panicked(home, clock()) {
            let _ = writeln!(stderr, "parinn mcp: audit a caught panic: {e}");
        }
    }
}

/// Audits a panic the MCP server caught.
fn mcp_panicked(home: &Home, time: u64) -> Result<Entry, AuditError> {
    let occurrence = Occurrence::Degraded {
        kind: Kind::Failed,
        counter: Counter::McpPanicked,
        operation: "mcp",
        detail: "panic recovered",
    };

    Log::new(home.audit()).append(&occurrence, time)
}

/// The counts the home's audit log gives, their rise counted after the
/// entry the counters were last acknowledged through, and its number of
/// entries; else the exit code, its failure reported: 3 for a broken
/// chain.
fn audit_counts(
    command: &str,
    home: &Home,
    acknowledged: Option<(u64, record::digest::Digest)>,
    stderr: &mut dyn Write,
) -> Result<(Counts, u64), u8> {
    match Log::new(home.audit()).entries() {
        Ok(entries) => Ok((counts(&entries, acknowledged), entries.len() as u64)),
        Err(e @ AuditError::Broken { .. }) => {
            let _ = writeln!(stderr, "parinn {command}: {e}");
            Err(EXIT_INTEGRITY)
        }
        Err(e) => Err(failed(command, &e, stderr)),
    }
}

/// The counts of `home`'s audit log since the counters were last
/// acknowledged, a neutral principal act the record holds (OWN-11), and
/// its number of entries; none before the home exists.
fn home_counts(
    command: &str,
    home: &Home,
    acts: &[act_record::Recorded],
    stderr: &mut dyn Write,
) -> Result<Option<(Counts, u64)>, u8> {
    if !home.root().exists() {
        return Ok(None);
    }

    audit_counts(command, home, act_record::acknowledged(acts), stderr).map(Some)
}

/// `parinn doctor [--json]`: read-only (ADM-10); exits 1 while a counter
/// rose since the counters were last acknowledged, naming each
/// (OPS-03).
fn doctor(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("doctor", args, stderr) else {
        return EXIT_USAGE;
    };
    let read = ctx
        .home()
        .map_err(|e| e.to_string())
        .and_then(|home| Ok((settings::recorded_acts(&home)?, home)));
    let (acts, home) = match read {
        Ok(read) => read,
        Err(e) => return failed("doctor", &e, stderr),
    };
    let counts = match home_counts("doctor", &home, &acts, stderr) {
        Ok(c) => c.map(|(c, _)| c),
        Err(code) => return code,
    };
    let code = if counts.as_ref().is_some_and(Counts::risen) {
        EXIT_FAILURE
    } else {
        EXIT_OK
    };
    let risen = counts.map(|c| c.since_ack);
    let text = if as_json {
        json!({"risen": risen.unwrap_or_default()}).to_string()
    } else {
        match risen {
            None => format!(
                "no home at {} yet: nothing to diagnose",
                home.root().display()
            ),
            Some(r) if r.is_empty() => {
                "no counter rose since the counters were last acknowledged".to_owned()
            }
            Some(r) => r
                .iter()
                .map(|(name, n)| {
                    format!("counter {name} rose by {n} since the counters were last acknowledged")
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    };

    match written(writeln!(stdout, "{text}"), stderr) {
        EXIT_OK => code,
        other => other,
    }
}

/// What `parinn status` shows.
struct Status {
    home: String,
    /// The store's path and size, once the home exists.
    store: Option<(String, Option<u64>)>,
    /// The schema version of the store, once it is made.
    schema_version: Option<u32>,
    /// The audit log's path and number of entries, once the home exists.
    audit: Option<(String, u64)>,
    /// Each counter's total and rise since the last acknowledgement.
    counters: Vec<(String, u64, u64)>,
    deployment_mode: String,
    awaiting_acceptance: Vec<String>,
    /// Why managed policy is refused, when it is.
    managed_policy: Option<String>,
    /// The key store the home's keys are kept in (SEC-10).
    keys: KeyStoreKind,
    /// Where the node identity is read from (REC-24).
    identity: IdentitySource,
    /// The store's open and newer segments, once there is a store.
    segments: Option<segments::SegmentsStatus>,
    /// The transcript sources held or lost, once there is a store.
    sources: Option<sources::SourcesStatus>,
    /// How many session bindings an MCP server took and did not take
    /// (REC-13, REC-19), once any `SessionStart` was observed.
    bindings: Option<(u64, u64)>,
}

impl Status {
    fn json(&self) -> String {
        let counters: BTreeMap<&String, Value> = self
            .counters
            .iter()
            .map(|(name, total, since)| (name, json!({"total": total, "since_ack": since})))
            .collect();
        let mut status = json!({
            "home": self.home,
            "store": self.store.as_ref().map(|(p, b)| {
                json!({"path": p, "bytes": b, "schema_version": self.schema_version})
            }),
            "audit": self.audit.as_ref().map(|(p, n)| json!({"path": p, "entries": n})),
            "counters": counters,
            "deployment_mode": self.deployment_mode,
            "awaiting_acceptance": self.awaiting_acceptance,
            "managed_policy": {"refused": self.managed_policy},
            "keys": {
                "store": self.keys.as_str(),
                "readable_by_agents": self.keys.readable_by_agents(),
                "where": self.keys.describe(),
            },
            "node_identity": identity_status::json(&self.identity),
            "fork_detection": identity_status::fork_detection_json(),
            "segments": self.segments.as_ref().map(segments::SegmentsStatus::json),
            "transcript_sources": self.sources.as_ref().map(sources::SourcesStatus::json),
        });
        // Named once any session binding was observed.
        if let Some((taken, untaken)) = self.bindings {
            status["mcp_server"] = json!({"bindings_taken": taken, "bindings_untaken": untaken});
        }

        status.to_string()
    }

    fn text(&self) -> String {
        let mut lines = Vec::new();
        match (&self.store, &self.audit) {
            (Some((store, bytes)), Some((audit, entries))) => {
                let size = bytes.map_or("none yet".to_owned(), |b| {
                    let schema = self
                        .schema_version
                        .map_or(String::new(), |v| format!(", schema version {v}"));
                    format!("{b} bytes{schema}")
                });
                lines.push(format!("home: {}", self.home));
                lines.push(format!("store: {store} ({size})"));
                lines.push(format!("audit log: {audit} ({entries} entries)"));
            }
            _ => lines.push(format!("home: {} (none yet)", self.home)),
        }
        let counters: Vec<String> = self
            .counters
            .iter()
            .map(|(name, total, since)| format!("{name} {total} ({since} since acknowledged)"))
            .collect();
        if self.store.is_some() {
            let counters = if counters.is_empty() {
                "none".to_owned()
            } else {
                counters.join(", ")
            };
            lines.push(format!("counters: {counters}"));
        }
        lines.push(format!("deployment mode: {}", self.deployment_mode));
        if !self.awaiting_acceptance.is_empty() {
            lines.push(format!(
                "awaiting acceptance (parinn configuration accept): {}",
                self.awaiting_acceptance.join(", ")
            ));
        }
        if let Some(why) = &self.managed_policy {
            lines.push(format!("managed policy refused: {why}"));
        }
        lines.push(format!("keys: {}", self.keys.describe()));
        lines.push(identity_status::node_identity_line(&self.identity));
        lines.push(identity_status::FORK_DETECTION_LINE.to_owned());
        lines.extend(
            self.segments
                .iter()
                .flat_map(segments::SegmentsStatus::lines),
        );
        lines.extend(self.sources.iter().flat_map(sources::SourcesStatus::lines));
        lines.extend(
            self.bindings
                .map(|(taken, untaken)| binding_line(taken, untaken)),
        );

        lines.join("\n")
    }
}

/// What `parinn status` says of the MCP server: whether session bindings
/// are taken (REC-13, REC-19).
fn binding_line(taken: u64, untaken: u64) -> String {
    match (taken, untaken) {
        (0, untaken) => {
            format!("MCP server: none has taken a session binding on this node ({untaken} untaken)")
        }
        (taken, 0) => format!("MCP server: took {taken} session bindings"),
        (taken, untaken) => {
            format!("MCP server: took {taken} session bindings, {untaken} untaken")
        }
    }
}

/// `parinn status [--json]`: the home, the store, the audit log, the
/// non-zero counters and the settings in force (ADM-11, which phase 15
/// completes). It exits 2 while the configuration or the repository
/// configuration fails validation (ADM-04).
fn status(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("status", args, stderr) else {
        return EXIT_USAGE;
    };
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("status", &e, stderr),
    };
    // The store's version is read first, and nothing in a newer store is
    // opened for writing (ADM-05).
    let segments = match segments::read(&home) {
        Ok(s) => s,
        Err(e) => return store_failed("status", &e, stderr),
    };
    let (mut store, acts) = match settings::opened_acts(&home) {
        Ok(opened) => opened,
        Err(e) => return failed("status", &e, stderr),
    };
    let in_force = settings::in_force(ctx, &home, &acts);
    if in_force.loaded.principal_layer_failed() {
        return invalid_layers("status", &in_force.loaded, stderr);
    }
    // Each start marker left behind and each session binding left
    // untaken is counted before the counters are read (NFR-02, REC-19).
    if home.root().exists() {
        hook_handlers::count_unfinished(&home, (ctx.clock)(), None);
    }
    if let Some(store) = store.as_mut() {
        count_untaken_bindings(ctx, store);
    }
    let counted = match home_counts("status", &home, &acts, stderr) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let refused = segments.as_ref().is_some_and(|s| !s.newer.is_empty());
    let mut status = status_of(&home, counted, &in_force);
    status.schema_version = segments.as_ref().map(|s| s.schema_version);
    status.segments = segments;
    status.bindings = store
        .as_ref()
        .and_then(|store| store.binding_counts().ok())
        .filter(|(taken, untaken)| taken + untaken > 0);
    status.sources = match sources::read(&home) {
        Ok(s) => s,
        Err(e) => return store_failed("status", &e, stderr),
    };
    let text = if as_json {
        status.json()
    } else {
        status.text()
    };

    // A refused segment of a newer format version is named and exits 3
    // (ADM-05).
    match written(writeln!(stdout, "{text}"), stderr) {
        EXIT_OK if refused => EXIT_INTEGRITY,
        other => other,
    }
}

/// The status of `home`, given its audit log's counts.
fn status_of(home: &Home, counted: Option<(Counts, u64)>, in_force: &settings::InForce) -> Status {
    let (store, audit) = (home.store(), home.audit());
    let mut status = Status {
        home: home.root().display().to_string(),
        store: None,
        schema_version: None,
        audit: None,
        counters: Vec::new(),
        deployment_mode: in_force.resolved.settings.deployment_mode().to_owned(),
        awaiting_acceptance: in_force
            .resolved
            .awaiting_acceptance
            .iter()
            .map(|k| k.name().to_owned())
            .collect(),
        managed_policy: in_force
            .loaded
            .failures
            .iter()
            .find(|f| f.layer == SettingsLayer::ManagedPolicy)
            .map(ToString::to_string),
        keys: home.key_store().kind(),
        identity: home.identity().clone(),
        segments: None,
        sources: None,
        bindings: None,
    };
    if let Some((counts, entries)) = counted {
        let bytes = fs::metadata(&store).ok().map(|m| m.len());
        status.store = Some((store.display().to_string(), bytes));
        status.audit = Some((audit.display().to_string(), entries));
        status.counters = counts
            .total
            .iter()
            .map(|(name, total)| {
                let since = counts.since_ack.get(name).copied().unwrap_or(0);
                (name.clone(), *total, since)
            })
            .collect();
    }

    status
}

/// `s` as a JSON string literal.
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');

    out
}

/// Turns the result of writing the command's output into its exit
/// code: a write that fails, to a closed stdout say, is a failure.
pub(crate) fn written(result: std::io::Result<()>, stderr: &mut dyn Write) -> u8 {
    match result {
        Ok(()) => EXIT_OK,
        Err(e) => {
            let _ = writeln!(stderr, "parinn: write the output: {e}");
            EXIT_FAILURE
        }
    }
}

/// Reports each failed settings layer but managed policy, exiting 2
/// (ADM-04): the error names the layer, the key and the problem.
pub(crate) fn invalid_layers(command: &str, loaded: &Loaded, stderr: &mut dyn Write) -> u8 {
    for failure in loaded
        .failures
        .iter()
        .filter(|f| f.layer != SettingsLayer::ManagedPolicy)
    {
        let _ = writeln!(stderr, "parinn {command}: {failure}");
    }

    EXIT_USAGE
}

mod act;
mod capture;
mod event;
pub mod forks;
mod hook_handlers;
mod identity_status;
mod index_gaps;
mod licenses;
mod purge;
mod quarantine;
mod rebuild;
mod recall;
mod repository_identity;
mod retention;
mod segments;
mod settings;
mod sources;

pub use hook_handlers::budget_override;
pub use settings::managed_policy;

#[cfg(test)]
mod tests;
