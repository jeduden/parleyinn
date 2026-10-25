//! ParleyInn's structured logs (ENG-05, OPS-05): a `tracing` subscriber
//! that writes each event as one JSON object per line, every field
//! value redacted first by the redactor its caller compiles, so no
//! secret a log field carries reaches the log file. It writes only to
//! the home's log file, `logs/parinn.jsonl`, which a hook handler logs
//! to and nowhere else; ParleyInn ships no log anywhere (OPS-05).
//!
//! The log file opens at the first event, through a closure the caller
//! hands in, so a home that does not exist yet, or that the caller
//! refuses, gets no log. A line that cannot be written is counted, for
//! the caller to audit (I6).

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use redaction::Redactor;
use serde_json::{Map, Value, json};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

/// The log file's name in the home's `logs` directory.
pub const LOG_FILE: &str = "parinn.jsonl";

/// Opens where log lines go: none when there is nowhere yet, as before
/// the home exists.
pub type Open = Box<dyn Fn() -> io::Result<Option<Box<dyn Write + Send>>> + Send + Sync>;

/// Compiles the redactor a log's fields pass through, or says why it
/// cannot.
pub type Compile = Box<dyn Fn() -> Result<Redactor, String> + Send + Sync>;

/// The subscriber.
pub struct StructuredLog {
    open: Open,
    out: Mutex<Option<Box<dyn Write + Send>>>,
    compile: Compile,
    redactor: OnceLock<Option<Redactor>>,
    clock: Box<dyn Fn() -> u64 + Send + Sync>,
    spans: AtomicU64,
    dropped: AtomicU64,
}

impl StructuredLog {
    /// A log written where `open` says, its lines stamped by `clock` in
    /// seconds since the Unix epoch, its fields redacted by the redactor
    /// `compile` makes. It compiles at the first event, so a run that
    /// logs nothing spends nothing on it.
    #[must_use]
    pub fn new(open: Open, compile: Compile, clock: Box<dyn Fn() -> u64 + Send + Sync>) -> Self {
        Self {
            open,
            out: Mutex::new(None),
            compile,
            redactor: OnceLock::new(),
            clock,
            spans: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
        }
    }

    /// How many lines could not be written.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// The JSON line for `event`, its fields redacted; none when the
    /// redactor does not compile, so no field goes unredacted.
    fn line(&self, event: &Event<'_>) -> Option<String> {
        let redactor = self
            .redactor
            .get_or_init(|| (self.compile)().ok())
            .as_ref()?;
        let mut fields = Fields {
            map: Map::new(),
            redactor,
        };
        event.record(&mut fields);
        let meta = event.metadata();

        let line = json!({
            "time": (self.clock)(),
            "level": level(*meta.level()),
            "target": meta.target(),
            "fields": fields.map,
        });

        Some(line.to_string())
    }

    /// Writes `line`, opening the log first if it is not open.
    fn write(&self, line: &str) -> io::Result<()> {
        let mut out = self
            .out
            .lock()
            .map_err(|_| io::Error::other("log poisoned"))?;
        if out.is_none() {
            *out = (self.open)()?;
        }
        match out.as_mut() {
            Some(w) => writeln!(w, "{line}").and_then(|()| w.flush()),
            None => Ok(()),
        }
    }
}

impl Subscriber for StructuredLog {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        *metadata.level() <= Level::INFO
    }

    fn new_span(&self, _span: &Attributes<'_>) -> Id {
        Id::from_u64(self.spans.fetch_add(1, Ordering::Relaxed) + 1)
    }

    fn record(&self, _span: &Id, _values: &Record<'_>) {}

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let written = self.line(event).map(|line| self.write(&line));
        if !matches!(written, Some(Ok(()))) {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn enter(&self, _span: &Id) {}

    fn exit(&self, _span: &Id) {}
}

/// A level's name in a log line.
fn level(level: Level) -> &'static str {
    match level {
        Level::ERROR => "error",
        Level::WARN => "warn",
        Level::INFO => "info",
        Level::DEBUG => "debug",
        Level::TRACE => "trace",
    }
}

/// An event's fields as JSON, each text redacted.
struct Fields<'a> {
    map: Map<String, Value>,
    redactor: &'a Redactor,
}

impl Fields<'_> {
    fn text(&mut self, field: &Field, text: &str) {
        let redacted = self.redactor.redact_text(text).text;
        self.map
            .insert(field.name().to_owned(), Value::String(redacted));
    }
}

impl Visit for Fields<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.text(field, &format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.text(field, value);
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.map.insert(field.name().to_owned(), json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.map.insert(field.name().to_owned(), json!(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.map.insert(field.name().to_owned(), json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.map.insert(field.name().to_owned(), json!(value));
    }
}

/// Opens the log file in `logs`, making the directory owner-only and
/// the file readable and writable by its owner alone (SEC-02).
///
/// # Errors
///
/// Fails when the directory cannot be made or the file opened.
pub fn open_log_file(logs: &Path) -> io::Result<File> {
    let mut dir = fs::DirBuilder::new();
    dir.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut dir, 0o700);
    dir.create(logs)?;
    let mut file = OpenOptions::new();
    file.create(true).append(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut file, 0o600);

    file.open(logs.join(LOG_FILE))
}

#[cfg(test)]
mod tests;
