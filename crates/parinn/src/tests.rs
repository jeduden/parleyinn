use super::*;
use configuration::source::OsUser;
use std::io;
use std::sync::mpsc;
use testkit::TempDir;

pub(crate) const DEV: Build<'static> = Build {
    version: "dev",
    commit: None,
};

/// A process's surroundings: a fresh home, and input that closes at
/// once unless a test hands lines.
pub(crate) struct Process {
    pub(crate) dir: TempDir,
    pub(crate) lines: Vec<String>,
    /// The input fails after its lines rather than closing.
    pub(crate) input_error: bool,
    /// The input stays open after its lines, as a harness that never
    /// closes it would.
    pub(crate) held_open: bool,
    /// The input fails after its lines as a line cut at the reader's
    /// limit does.
    pub(crate) input_too_large: bool,
    pub(crate) home_id: Option<String>,
    pub(crate) hook_panics: bool,
    /// The hook handler hangs until the harness would kill it.
    pub(crate) hook_hangs: bool,
    /// The standard input and output are a terminal, as a person's are;
    /// an agent's tool call has none.
    pub(crate) terminal: bool,
    /// The running OS user the core is told of: by default someone else
    /// than the owner of the test's files, so a managed policy the test
    /// writes passes as root's would.
    pub(crate) os_user: OsUser,
    /// Where the node identity is read from, as a development build's
    /// seam lets a test say; this OS's source when none.
    pub(crate) node_identity: Option<std::path::PathBuf>,
    /// Whether the harness's attended mark is in the environment.
    pub(crate) attended: bool,
    /// The hook budget in place of each hook's own.
    pub(crate) hook_budget: Option<std::time::Duration>,
    /// The time the clock reads.
    pub(crate) now: u64,
    /// How far the clock moves on each read: none by default.
    pub(crate) clock_step: u64,
    /// The process ancestry a hook handler reads.
    pub(crate) ancestry: fn() -> Option<hook::ancestry::Ancestry>,
    /// How many lines of a transcript live ingest reads at most.
    pub(crate) live_limit: Option<usize>,
    /// How long a search may run, as a development build's seam lets a
    /// test say; the default when none.
    pub(crate) query_deadline: Option<Duration>,
    /// The working directory, the process directory when none.
    pub(crate) working_dir: Option<std::path::PathBuf>,
}

/// The harness process the tests' hook handlers and MCP servers run
/// below.
pub(crate) const HARNESS: u32 = 4242;

/// The ancestry of a hook handler below its own harness alone.
pub(crate) fn own_harness() -> Option<hook::ancestry::Ancestry> {
    Some(hook::ancestry::Ancestry {
        harnesses: 1,
        nested: false,
        harness: Some(HARNESS),
    })
}

/// A hook budget no handler under test reaches, however loaded the
/// machine running the tests.
pub(crate) const LENIENT: std::time::Duration = std::time::Duration::from_secs(20);

/// The time every test's clock reads.
pub(crate) const NOW: u64 = 1_760_000_000;

/// A transcript line in Claude Code 2.1.296's recorded shape, which
/// ingest records with nothing to audit.
const CLEAN_LINE: &str = "{\"type\":\"atis-latch\",\"atis\":\"\",\"sessionId\":\"s\"}\n";

impl Process {
    pub(crate) fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
            lines: Vec::new(),
            input_error: false,
            held_open: false,
            input_too_large: false,
            home_id: None,
            hook_panics: false,
            hook_hangs: false,
            terminal: true,
            os_user: OsUser::Known(u32::MAX),
            node_identity: None,
            attended: false,
            hook_budget: Some(LENIENT),
            ancestry: own_harness,
            live_limit: None,
            now: NOW,
            clock_step: 0,
            query_deadline: None,
            working_dir: None,
        }
    }

    pub(crate) fn parinn_home(&self) -> std::path::PathBuf {
        self.dir.path().join(".parinn")
    }

    pub(crate) fn invoke(&self, args: &[&str], build: &Build) -> (u8, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = self.with_context(build, |ctx| {
            run(
                args.iter().map(|a| (*a).to_owned()),
                ctx,
                &mut out,
                &mut err,
            )
        });

        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    /// Runs `f` with the context an invocation of this process gets.
    pub(crate) fn with_context<T>(&self, build: &Build, f: impl FnOnce(&Context) -> T) -> T {
        let (now, step) = (std::cell::Cell::new(self.now), self.clock_step);
        let clock = move || {
            let t = now.get();
            now.set(t.saturating_add(step));
            t
        };
        let (lines, input_error, held_open, too_large) = (
            self.lines.clone(),
            self.input_error,
            self.held_open,
            self.input_too_large,
        );
        let input = move || {
            let (tx, rx) = mpsc::channel();
            for line in lines.clone() {
                tx.send(Ok(line)).unwrap();
            }
            if input_error {
                tx.send(Err(io::Error::new(io::ErrorKind::InvalidData, "not UTF-8")))
                    .unwrap();
            }
            if too_large {
                tx.send(Err(io::Error::from(io::ErrorKind::FileTooLarge)))
                    .unwrap();
            }
            if held_open {
                std::mem::forget(tx);
            }
            rx
        };
        let ctx = Context {
            build,
            parinn_home: None,
            home: Some(self.dir.path().as_os_str().to_owned()),
            home_id: self.home_id.as_ref().map(Into::into),
            hook_panics: self.hook_panics,
            hook_hangs: self.hook_hangs,
            terminal: self.terminal,
            working_dir: Some(
                self.working_dir
                    .clone()
                    .unwrap_or_else(|| self.dir.path().to_path_buf()),
            ),
            managed_policy: self.dir.path().join("managed").join("managed_policy.toml"),
            node_identity: self.node_identity.clone(),
            os_user: self.os_user,
            input: &input,
            clock: &clock,
            attended: self.attended,
            hook_budget: self.hook_budget,
            ancestry: self.ancestry,
            live_limit: self.live_limit,
            query_deadline: self.query_deadline,
        };

        f(&ctx)
    }

    /// Writes a transcript under the default transcript root,
    /// `~/.claude/projects` (SEC-18).
    pub(crate) fn transcript(&self, text: &str) -> String {
        let dir = self.dir.path().join(".claude").join("projects").join("p");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.jsonl");
        std::fs::write(&path, text).unwrap();
        path.display().to_string()
    }

    /// Writes the principal's configuration, owner-only as the home
    /// wants it, making the home first.
    pub(crate) fn configure(&self, text: &str) {
        let home = Home::locate(Some(self.parinn_home().as_os_str()), None).unwrap();
        home.create().unwrap();
        let path = self.parinn_home().join("config.toml");
        std::fs::write(&path, text).unwrap();
        set_mode(&path, 0o600);
    }

    /// Writes managed policy where the core reads it, as root's: no
    /// group or other write, and the running OS user not its owner.
    pub(crate) fn manage(&self, text: &str) {
        let dir = self.dir.path().join("managed");
        std::fs::create_dir_all(&dir).unwrap();
        set_mode(&dir, 0o755);
        let path = dir.join("managed_policy.toml");
        std::fs::write(&path, text).unwrap();
        set_mode(&path, 0o644);
    }

    /// Writes a repository configuration at the top of a repository
    /// containing the working directory.
    pub(crate) fn repository(&self, text: &str) {
        std::fs::create_dir_all(self.dir.path().join(".git")).unwrap();
        std::fs::write(self.dir.path().join(".parinn.toml"), text).unwrap();
    }
}

/// Sets `path`'s mode.
pub(crate) fn set_mode(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

fn invoke(args: &[&str], build: &Build) -> (u8, String, String) {
    Process::new().invoke(args, build)
}

#[test]
fn licenses_is_a_command_that_prints_the_embedded_texts() {
    let (code, out, err) = invoke(&["licenses"], &DEV);

    assert_eq!((code, err.as_str()), (0, ""));
    assert_eq!(out, crate::licenses::THIRD_PARTY_LICENSES);
}

#[test]
fn no_command_is_a_usage_error() {
    let (code, out, err) = invoke(&[], &DEV);

    assert_eq!(code, EXIT_USAGE);
    assert_eq!(out, "");
    assert!(err.starts_with("usage: parinn"), "{err}");
}

#[test]
fn help_prints_the_usage_to_stdout() {
    for arg in ["help", "-h", "--help"] {
        let (code, out, err) = invoke(&[arg], &DEV);

        assert_eq!(code, EXIT_OK, "{arg}");
        assert!(out.starts_with("usage: parinn"), "{arg}: {out}");
        assert_eq!(err, "", "{arg}");
    }
}

#[test]
fn an_unknown_command_is_a_usage_error() {
    let (code, _, err) = invoke(&["restructure"], &DEV);

    assert_eq!(code, EXIT_USAGE);
    assert!(
        err.starts_with("parinn: unknown command \"restructure\"\n\nusage: parinn"),
        "{err}"
    );
}

#[test]
fn version_prints_the_stamped_version() {
    let build = Build {
        version: "v1.2.3",
        commit: Some("abc"),
    };

    assert_eq!(
        invoke(&["version"], &build),
        (EXIT_OK, "v1.2.3\n".into(), String::new())
    );
}

#[test]
fn version_json_carries_the_version_and_the_commit() {
    let build = Build {
        version: "v1.2.3",
        commit: Some("abc"),
    };

    let (code, out, _) = invoke(&["version", "--json"], &build);

    assert_eq!(code, EXIT_OK);
    assert_eq!(out, "{\"version\":\"v1.2.3\",\"commit\":\"abc\"}\n");
}

#[test]
fn version_json_leaves_out_a_commit_the_build_did_not_stamp() {
    assert_eq!(
        invoke(&["version", "--json"], &DEV).1,
        "{\"version\":\"dev\"}\n"
    );
}

#[test]
fn version_refuses_an_unknown_flag() {
    let (code, out, err) = invoke(&["version", "--yaml"], &DEV);

    assert_eq!(code, EXIT_USAGE);
    assert_eq!(out, "");
    assert_eq!(err, "parinn version: unknown flag \"--yaml\"\n");
}

#[test]
fn json_string_escapes_quotes_backslashes_and_control_characters() {
    assert_eq!(
        json_string("a\"b\\c\nd\u{1}é"),
        r#""a\"b\\c\u000ad\u0001é""#
    );
}

/// Refuses every write, standing in for a closed stdout.
struct Closed;

impl io::Write for Closed {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_failed_write_is_a_failure() {
    let p = Process::new();
    let path = p.transcript("x\n");
    for args in [
        &["version"][..],
        &["version", "--json"],
        &["help"],
        &["ingest", "--path", &path],
        &["verify"],
        &["counter", "ack"],
        &["doctor"],
        &["status"],
    ] {
        let mut err = Vec::new();
        let input = || mpsc::channel().1;
        let ctx = Context {
            build: &DEV,
            parinn_home: Some(p.parinn_home().into_os_string()),
            home: Some(p.dir.path().as_os_str().to_owned()),
            home_id: None,
            hook_panics: false,
            hook_hangs: false,
            terminal: true,
            working_dir: None,
            managed_policy: std::path::PathBuf::from("/nonexistent/managed_policy.toml"),
            node_identity: None,
            os_user: OsUser::Known(u32::MAX),
            input: &input,
            clock: &|| NOW,
            attended: false,
            hook_budget: Some(LENIENT),
            ancestry: own_harness,
            live_limit: None,
            query_deadline: None,
        };

        let code = run(
            args.iter().map(|a| (*a).to_owned()),
            &ctx,
            &mut Closed,
            &mut err,
        );

        assert_eq!(code, EXIT_FAILURE, "{args:?}");
        assert!(
            String::from_utf8(err).unwrap().contains("closed"),
            "{args:?}"
        );
    }
}

#[test]
fn the_build_is_stamped_from_the_environment_or_reports_dev() {
    assert_eq!(
        BUILD.version,
        option_env!("PARINN_VERSION").unwrap_or("dev")
    );
    assert_eq!(BUILD.commit, option_env!("PARINN_COMMIT"));
}

#[test]
fn ingest_records_each_line_and_names_the_writer() {
    let p = Process::new();
    let path = p.transcript("a\nb\n");

    let (code, out, err) = p.invoke(&["ingest", "--path", &path], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.starts_with("ingested 2 lines into writer "), "{out}");
    assert!(!out.contains(&path), "the output repeats no path: {out}");
}

#[test]
fn a_release_build_ingests_now_that_redaction_comes_first() {
    let p = Process::new();
    let path = p.transcript("a\n");
    let release = Build {
        version: "1.0.0",
        commit: Some("abc"),
    };

    let (code, out, err) = p.invoke(&["ingest", "--path", &path], &release);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.starts_with("ingested 1 lines into writer "), "{out}");
}

#[test]
fn ingest_redacts_by_the_principals_patterns_and_audits_each_redaction() {
    let p = Process::new();
    p.configure("[redaction]\nextra_patterns = [\"(\", \"ACME-[0-9]{8}\"]\n");
    let path = p.transcript("{\"t\":\"AKIAIOSFODNN7EXAMPLE ACME-12345678\"}\n");

    let (code, _, err) = p.invoke(&["ingest", "--path", &path], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let entries: Vec<(String, Option<String>, String)> = audited(&p)
        .into_iter()
        .map(|e| (e.kind, e.counter, e.detail))
        .collect();
    let entry = |kind: &str, counter: &str, detail: &str| {
        (kind.to_owned(), Some(counter.to_owned()), detail.to_owned())
    };
    assert_eq!(
        entries,
        [
            entry(
                "rejected",
                "redaction_pattern_refused",
                "refused redaction pattern extra_pattern_1"
            ),
            entry("redacted", "redactions", "redacted by rule aws_access_key"),
            entry("redacted", "redactions", "redacted by rule extra_pattern_2"),
            // The line has no type, so it is stored unparsed (REC-05).
            entry(
                "failed",
                "events_unparsed",
                "stored a transcript line it cannot parse as an unparsed event"
            ),
        ]
    );
    let store = std::fs::read(p.parinn_home().join("store.db")).unwrap();
    assert!(!store.windows(13).any(|w| w == b"ACME-12345678"));
}

#[test]
fn ingest_json_names_the_writer_seat_room_and_count() {
    let p = Process::new();
    let path = p.transcript("a\n");

    let (code, out, _) = p.invoke(&["ingest", &format!("--path={path}"), "--json"], &DEV);

    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["events"], 1);
    for field in ["writer", "seat", "room"] {
        assert_eq!(v[field].as_str().unwrap().len(), 64, "{field}");
    }
}

#[test]
fn ingest_refuses_a_wrong_command_line() {
    for (args, want) in [
        (&["ingest"][..], "parinn ingest: want --all or --path P\n"),
        (
            &["ingest", "--path"],
            "parinn ingest: --path needs a transcript\n",
        ),
        (
            &["ingest", "--all", "--path", "t.jsonl"],
            "parinn ingest: want --all or --path P, not both\n",
        ),
        (
            &["ingest", "--path=t.jsonl", "--all"],
            "parinn ingest: want --all or --path P, not both\n",
        ),
        (
            &["ingest", "--yaml"],
            "parinn ingest: unknown flag \"--yaml\"\n",
        ),
    ] {
        let (code, _, err) = invoke(args, &DEV);
        assert_eq!((code, err.as_str()), (EXIT_USAGE, want), "{args:?}");
    }
}

#[test]
fn ingest_reports_a_transcript_it_cannot_read() {
    let p = Process::new();
    let missing = p.transcript("a\n").replace("t.jsonl", "missing.jsonl");

    let (code, _, err) = p.invoke(&["ingest", "--path", &missing], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("parinn ingest: read the transcript "),
        "{err}"
    );
}

#[test]
fn ingest_audits_a_limited_redaction_apart_from_a_redaction() {
    let p = Process::new();
    let key = format!("AIza{}", "a".repeat(35));
    let path = p.transcript(&format!("{{\"t\":\"{}\"}}\n", key.repeat(5_000)));

    let (code, _, err) = p.invoke(&["ingest", "--path", &path], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let limited: Vec<_> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("redaction_scan_limited"))
        .map(|e| (e.kind, e.detail))
        .collect();
    assert_eq!(
        limited,
        [("dropped".to_owned(), crate::SCAN_LIMITED_DETAIL.to_owned())]
    );
}

#[test]
fn ingest_audits_and_counts_each_unparsed_line_and_format_drift() {
    let p = Process::new();
    let path = p.transcript("{not json\n{\"type\":\"future_kind\"}\n");

    let (code, _, err) = p.invoke(&["ingest", "--path", &path], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let entries: Vec<(String, Option<String>, String)> = audited(&p)
        .into_iter()
        .map(|e| (e.kind, e.counter, e.detail))
        .collect();
    let entry = |kind: &str, counter: &str, detail: &str| {
        (kind.to_owned(), Some(counter.to_owned()), detail.to_owned())
    };
    let unparsed = || {
        entry(
            "failed",
            "events_unparsed",
            "stored a transcript line it cannot parse as an unparsed event",
        )
    };
    assert_eq!(
        entries,
        [
            unparsed(),
            unparsed(),
            entry(
                "rejected",
                "format_drift",
                "format drift claude-code:unversioned:future_kind:unrecorded-type"
            ),
        ]
    );
    let (_, status, _) = p.invoke(&["status"], &DEV);
    assert!(
        status.contains("events_unparsed 2 (2 since acknowledged)"),
        "{status}"
    );
    assert!(
        status.contains("format_drift 1 (1 since acknowledged)"),
        "{status}"
    );
}

#[test]
fn ingest_says_which_unparsed_line_or_drift_it_cannot_audit_and_still_records() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let audit_dir = p.parinn_home().join("audit");
    std::fs::remove_dir_all(&audit_dir).unwrap();
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let (code, out, err) = p.invoke(
        &[
            "ingest",
            "--path",
            &p.transcript("{\"type\":\"future_kind\"}\n"),
        ],
        &DEV,
    );

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.starts_with("ingested 1 lines"), "{out}");
    assert!(
        err.contains("parinn ingest: audit an unparsed line: "),
        "{err}"
    );
    assert!(
        err.contains("parinn ingest: audit a format drift: "),
        "{err}"
    );
}

/// Writes a transcript `name` holding `text` in project `project` under
/// the default transcript root.
pub(crate) fn transcript_in(
    p: &Process,
    project: &str,
    name: &str,
    text: &[u8],
) -> std::path::PathBuf {
    let dir = p.dir.path().join(".claude").join("projects").join(project);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn ingest_all_reads_every_transcript_under_the_transcript_roots() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "s1.jsonl", b"a\n");
    transcript_in(&p, "-w-b", "s2.jsonl", b"b\nc\n");

    let (code, out, err) = p.invoke(&["ingest", "--all"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert!(
        lines[0].starts_with("ingested 1 lines into writer "),
        "{out}"
    );
    assert!(
        lines[1].starts_with("ingested 2 lines into writer "),
        "{out}"
    );

    let (code, out, _) = p.invoke(&["ingest", "--all", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let events: Vec<u64> = v["transcripts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["events"].as_u64().unwrap())
        .collect();
    // Read again, each transcript source gains no event (REC-03).
    assert_eq!(events, [0, 0]);
}

#[test]
fn ingest_audits_a_new_transcript_generation_and_repeated_lines() {
    let p = Process::new();
    let line = |uuid: &str, text: &str| {
        format!("{{\"type\":\"x\",\"uuid\":\"{uuid}\",\"t\":\"{text}\"}}\n")
    };
    let main = transcript_in(
        &p,
        "-w-a",
        "main-run.jsonl",
        format!("{}{}", line("u1", "a"), line("u2", "b")).as_bytes(),
    );
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    std::fs::write(&main, line("u1", "a")).unwrap();
    transcript_in(
        &p,
        "-w-a",
        "repeating.jsonl",
        format!("{}{}", line("u1", "a"), line("u2", "other")).as_bytes(),
    );

    let (code, out, err) = p.invoke(&["ingest", "--all", "--json"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let main_report = &v["transcripts"][0];
    assert_eq!(main_report["generation"], 2);
    assert_eq!(main_report["held"], false);
    assert_eq!(main_report["runs"][0]["agent_id"], serde_json::Value::Null);
    assert_eq!(main_report["runs"][0]["parent"], serde_json::Value::Null);
    let audit = audit_text(&p);
    assert!(
        audit.contains("new transcript generation of main-run"),
        "{audit}"
    );
    assert!(
        audit.contains("repeated uuid with other content, recorded as its own event, in repeating"),
        "{audit}"
    );
    assert!(
        audit.contains(
            "repeated lines owned by the run that first recorded them, repeated in repeating"
        ),
        "{audit}"
    );
    let (_, out, _) = p.invoke(&["status", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    for counter in [
        "new_transcript_generation",
        "repeat_conflicts",
        "repeat_owner_chosen",
    ] {
        assert_eq!(v["counters"][counter]["total"], 1, "{counter}");
    }
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
}

#[test]
fn a_transcript_source_gone_before_it_was_fully_ingested_is_reported_and_audited_once() {
    let p = Process::new();
    let cut = transcript_in(&p, "-w-a", "cut-short.jsonl", b"{\"a\":1}\n{\"b\":");
    let whole = transcript_in(&p, "-w-a", "main-run.jsonl", b"{\"a\":1}\n");
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    std::fs::remove_file(&cut).unwrap();
    std::fs::remove_file(&whole).unwrap();

    let (code, out, err) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["broken"]["check"], "transcript source completeness");
    let named = format!(
        "transcript sources fall short: {} disappeared before it was fully ingested",
        cut.display()
    );
    assert_eq!(v["broken"]["message"], named);
    assert_eq!(
        err,
        format!("parinn verify: transcript source completeness: {named}\n")
    );
    let (_, status, _) = p.invoke(&["status"], &DEV);
    assert!(
        status.contains("transcript sources gone before they were fully ingested: "),
        "{status}"
    );

    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    let audit = audit_text(&p);
    assert_eq!(
        audit
            .matches("transcript source gone before it was fully ingested: cut-short")
            .count(),
        1,
        "{audit}"
    );
}

#[test]
fn ingest_says_which_lost_transcript_source_it_cannot_audit() {
    let p = Process::new();
    let cut = transcript_in(&p, "-w-a", "cut-short.jsonl", b"{\"a\":1}\n{\"b\":");
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    std::fs::remove_file(&cut).unwrap();
    let audit_dir = p.parinn_home().join("audit");
    std::fs::remove_dir_all(&audit_dir).unwrap();
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let (_, _, err) = p.invoke(&["ingest", "--all"], &DEV);

    assert!(
        err.contains("parinn ingest: audit a lost transcript source: "),
        "{err}"
    );
}

/// What `parinn verify --json` says, and its exit code.
fn verified(p: &Process) -> (u8, serde_json::Value) {
    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);

    (code, serde_json::from_str(&out).unwrap())
}

#[test]
fn a_source_an_ingest_left_short_fails_verify_once_its_marker_is_old() {
    let mut p = Process::new();
    let path = transcript_in(&p, "-w-a", "stopped.jsonl", b"{\"a\":1}\n{\"b\":2}\n");
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    // As an ingest killed after its first line's transaction leaves it.
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute(
        "UPDATE transcript_sources SET byte_offset = 8, prefix_hash = ?1, lines = 1, seen = 16",
        [record::digest::Digest::of(&[b"{\"a\":1}\n"]).to_hex()],
    )
    .unwrap();
    drop(conn);

    let (code, v) = verified(&p);
    assert_eq!(code, EXIT_OK, "{v}");
    assert_eq!(v["short_sources"], json!([path.display().to_string()]));

    p.now = NOW + 10 * 60 + 1;
    let (code, v) = verified(&p);
    assert_eq!(code, EXIT_INTEGRITY);
    assert_eq!(v["broken"]["check"], "transcript source completeness");
    assert_eq!(
        v["broken"]["message"],
        format!(
            "transcript sources fall short: {} {}",
            path.display(),
            sources::STALE
        )
    );
}

#[test]
fn verify_that_runs_out_of_time_deriving_the_index_leaves_it_unchecked_and_counts_it() {
    let mut p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    p.clock_step = 100;

    let (code, out, err) = p.invoke(&["verify"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(
        out.contains("; the search index was not derived again in time, so it is unchecked"),
        "{out}"
    );
    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["index_checked"], false);
    let counted = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("verify_timed_out"))
        .count();
    assert_eq!(counted, 2);

    p.clock_step = 0;
    let (_, out, _) = p.invoke(&["verify", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["index_checked"], true);
}

#[test]
fn a_transcript_source_cut_or_rewritten_after_ingest_fails_verify() {
    let p = Process::new();
    let path = transcript_in(&p, "-w-a", "main-run.jsonl", b"{\"a\":1}\n{\"b\":2}\n");
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    assert_eq!(verified(&p).0, EXIT_OK);

    for damage in [&b"{\"a\":1}\n"[..], b"{\"a\":9}\n{\"b\":2}\n", b""] {
        std::fs::write(&path, damage).unwrap();
        let (code, v) = verified(&p);

        assert_eq!(code, EXIT_INTEGRITY, "{damage:?}");
        assert_eq!(
            v["broken"]["message"],
            format!(
                "transcript sources fall short: {} {}",
                path.display(),
                sources::REPLACED
            )
        );
    }
}

#[test]
fn transcript_sources_that_cannot_be_read_fail_verify_and_status_and_are_said_by_ingest() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "s1.jsonl", b"a\n");
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    std::fs::remove_file(p.dir.path().join(".claude/projects/-w-a/s1.jsonl")).unwrap();
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute_batch("DROP TABLE transcript_sources").unwrap();
    drop(conn);

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("transcript_sources"), "{err}");
    let (code, _, err) = p.invoke(&["status"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("transcript_sources"), "{err}");
    let (_, _, err) = p.invoke(&["ingest", "--all"], &DEV);
    assert!(
        err.contains("parinn ingest: find lost transcript sources: "),
        "{err}"
    );
}

#[test]
fn ingest_all_with_no_transcript_ingests_nothing() {
    let p = Process::new();

    assert_eq!(
        p.invoke(&["ingest", "--all"], &DEV),
        (
            EXIT_OK,
            "ingested no transcript\n".to_owned(),
            String::new()
        )
    );
    assert_eq!(
        p.invoke(&["ingest", "--all", "--json"], &DEV),
        (EXIT_OK, "{\"transcripts\":[]}\n".to_owned(), String::new())
    );
}

mod part2;
mod part3;
mod part4;

pub(crate) use part2::{audited, junk_store, owner_only_dirs};
pub(crate) use part3::recorded;
pub(crate) use part4::audit_text;
