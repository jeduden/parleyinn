use std::io;
use std::sync::{Arc, Mutex};

use redaction::Redactor;
use serde_json::{Value, json};
use testkit::TempDir;

use super::{LOG_FILE, Open, StructuredLog, level, open_log_file};

/// A writer whose bytes a test reads back.
#[derive(Clone, Default)]
struct Shared(Arc<Mutex<Vec<u8>>>);

impl io::Write for Shared {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn log_to(open: Open) -> StructuredLog {
    StructuredLog::new(open, Box::new(rule_set), Box::new(|| 7))
}

fn rule_set() -> Result<Redactor, String> {
    Redactor::new(&[])
        .map(|(r, _)| r)
        .map_err(|e| e.to_string())
}

fn lines(shared: &Shared) -> Vec<Value> {
    String::from_utf8(shared.0.lock().unwrap().clone())
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn an_event_is_one_json_line_with_its_fields_redacted() {
    let shared = Shared::default();
    let out = shared.clone();
    let log = log_to(Box::new(move || Ok(Some(Box::new(out.clone())))));

    tracing::subscriber::with_default(log, || {
        tracing::info!(
            transcript = "/p/AKIAIOSFODNN7EXAMPLE.jsonl",
            events = 3u64,
            delta = -1i64,
            ok = true,
            ratio = 0.5f64,
            shown = ?"AKIAIOSFODNN7EXAMPLE",
            "ingested"
        );
        tracing::debug!("left out below info");
        let span = tracing::info_span!("a span", n = 1);
        let _entered = span.enter();
        span.record("n", 2);
        span.follows_from(tracing::info_span!("before").id());
        tracing::warn!("in a span");
    });

    let lines = lines(&shared);
    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines[0],
        json!({
            "time": 7,
            "level": "info",
            "target": "structured_log::tests",
            "fields": {
                "message": "ingested",
                "transcript": "/p/[REDACTED:aws_access_key].jsonl",
                "events": 3,
                "delta": -1,
                "ok": true,
                "ratio": 0.5,
                "shown": "\"[REDACTED:aws_access_key]\"",
            },
        })
    );
    assert_eq!(lines[1]["level"], "warn");
}

#[test]
fn nowhere_to_write_logs_nothing_and_drops_nothing() {
    let log = Arc::new(log_to(Box::new(|| Ok(None))));
    let shared: Arc<dyn tracing::Subscriber + Send + Sync> = log.clone();

    tracing::subscriber::with_default(shared, || tracing::error!("no home yet"));

    assert_eq!(log.dropped(), 0);
}

#[test]
fn a_line_that_cannot_be_written_is_counted() {
    let log = Arc::new(log_to(Box::new(|| Err(io::Error::other("refused")))));
    let shared: Arc<dyn tracing::Subscriber + Send + Sync> = log.clone();

    tracing::subscriber::with_default(shared, || {
        tracing::error!("one");
        tracing::error!("two");
    });

    assert_eq!(log.dropped(), 2);
}

#[test]
fn level_names_every_level() {
    use tracing::Level;
    let names: Vec<&str> = [
        Level::ERROR,
        Level::WARN,
        Level::INFO,
        Level::DEBUG,
        Level::TRACE,
    ]
    .into_iter()
    .map(level)
    .collect();
    assert_eq!(names, ["error", "warn", "info", "debug", "trace"]);
}

#[test]
fn the_log_file_is_owner_only_and_appended_to() {
    let dir = TempDir::new().unwrap();
    let logs = dir.path().join("home").join("logs");
    {
        use std::io::Write;
        writeln!(open_log_file(&logs).unwrap(), "a").unwrap();
        writeln!(open_log_file(&logs).unwrap(), "b").unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(logs.join(LOG_FILE)).unwrap(),
        "a\nb\n"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&logs), 0o700);
        assert_eq!(mode(&logs.join(LOG_FILE)), 0o600);
    }
    std::fs::write(dir.path().join("file"), "").unwrap();
    assert!(open_log_file(&dir.path().join("file").join("logs")).is_err());
}

#[test]
fn a_redactor_that_does_not_compile_drops_every_line_unwritten() {
    let shared = Shared::default();
    let out = shared.clone();
    let log = Arc::new(StructuredLog::new(
        Box::new(move || Ok(Some(Box::new(out.clone())))),
        Box::new(|| Err("a settings layer failed".to_owned())),
        Box::new(|| 7),
    ));
    let subscriber: Arc<dyn tracing::Subscriber + Send + Sync> = log.clone();

    tracing::subscriber::with_default(subscriber, || tracing::info!(key = "AKIAIOSFODNN7EXAMPLE"));

    assert_eq!(log.dropped(), 1);
    assert!(shared.0.lock().unwrap().is_empty());
}
