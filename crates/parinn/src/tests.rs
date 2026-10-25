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
            query_deadline: None,
            working_dir: None,
        }
    }

    pub(crate) fn parinn_home(&self) -> std::path::PathBuf {
        self.dir.path().join(".parinn")
    }

    pub(crate) fn invoke(&self, args: &[&str], build: &Build) -> (u8, String, String) {
        let now = self.now;
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
            clock: &|| now,
            attended: self.attended,
            hook_budget: self.hook_budget,
            ancestry: self.ancestry,
            live_limit: self.live_limit,
            query_deadline: self.query_deadline,
        };
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(
            args.iter().map(|a| (*a).to_owned()),
            &ctx,
            &mut out,
            &mut err,
        );

        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
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
            entry("redacted", "redacted", "redacted by rule aws_access_key"),
            entry("redacted", "redacted", "redacted by rule extra_pattern_2"),
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

    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["lost_sources"],
        serde_json::json!([cut.display().to_string()])
    );
    let (_, text, _) = p.invoke(&["verify"], &DEV);
    assert!(
        text.contains("transcript sources gone before they were fully ingested: "),
        "{text}"
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

#[test]
fn ingest_all_goes_on_past_a_transcript_it_cannot_ingest_and_fails() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "bad.jsonl", &[0xff, b'\n']);
    transcript_in(&p, "-w-b", "good.jsonl", b"a\n");

    let (code, out, err) = p.invoke(&["ingest", "--all"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(out.starts_with("ingested 1 lines into writer "), "{out}");
    assert!(err.contains("is not UTF-8"), "{err}");
}

#[cfg(unix)]
#[test]
fn ingest_all_refuses_a_linked_transcript_outside_the_roots_and_goes_on() {
    let p = Process::new();
    let outside = p.dir.path().join("outside.jsonl");
    std::fs::write(&outside, "a\n").unwrap();
    let dir = p.dir.path().join(".claude").join("projects").join("-w-a");
    std::fs::create_dir_all(&dir).unwrap();
    std::os::unix::fs::symlink(&outside, dir.join("linked.jsonl")).unwrap();

    let (code, out, err) = p.invoke(&["ingest", "--all"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(err.contains("parinn ingest: "), "{err}");
    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    assert_eq!(counters, [Some("path_outside_allowed_roots".to_owned())]);
}

#[test]
fn ingest_all_reports_a_transcript_root_it_cannot_list() {
    let p = Process::new();
    p.configure(&format!(
        "[transcript]\nroots = [{:?}]\n",
        p.dir.path().join("file").display().to_string()
    ));
    std::fs::write(p.dir.path().join("file"), "").unwrap();

    let (code, _, err) = p.invoke(&["ingest", "--all"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("parinn ingest: list the transcripts in "),
        "{err}"
    );
}

#[test]
fn verify_checks_the_record_and_counts_it() {
    let p = Process::new();
    let path = p.transcript(&CLEAN_LINE.repeat(2));
    p.invoke(&["ingest", "--path", &path], &DEV);

    let text = p.invoke(&["verify"], &DEV);
    let json = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!(
        text,
        (
            EXIT_OK,
            "checked 2 writers and 6 events: every chain, commitment and seat certificate holds; \
             integrity unsigned, as no log is signed yet; the audit log's chain holds over 0 \
             entries\n"
                .into(),
            String::new()
        )
    );
    assert_eq!(json.0, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&json.1).unwrap();
    assert_eq!(
        (&v["integrity"], &v["writers"], &v["events"], &v["broken"]),
        (
            &serde_json::json!("unsigned"),
            &serde_json::json!(2),
            &serde_json::json!(6),
            &serde_json::Value::Null
        )
    );
    assert_eq!(v["audit"], serde_json::json!({"files": 0, "entries": 0}));
    let order: Vec<&str> = v["order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    assert_eq!(order.len(), 6);
    assert!(
        order.iter().all(|a| a.starts_with("parinn:room/")),
        "{order:?}"
    );
}

#[test]
fn verify_exits_3_naming_where_the_chain_breaks() {
    let p = Process::new();
    let path = p.transcript("a\nb\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute(
        "UPDATE events SET provenance = 'user' WHERE kind = 'unparsed_line' AND seq = 2",
        [],
    )
    .unwrap();

    let (code, out, err) = p.invoke(&["verify"], &DEV);
    let (json_code, json, _) = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_INTEGRITY, ""));
    assert!(
        err.starts_with("parinn verify: chain broken at parinn:room/"),
        "{err}"
    );
    assert!(
        err.ends_with("/2: the stored hash does not match the event's header\n"),
        "{err}"
    );
    assert_eq!(json_code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        format!(
            "parinn verify: {}\n",
            v["broken"]["message"].as_str().unwrap()
        ),
        err
    );
    assert_eq!(
        (&v["integrity"], &v["broken"]["class"], &v["order"]),
        (
            &serde_json::json!("broken"),
            &serde_json::json!("chain"),
            &serde_json::Value::Null
        )
    );
    assert!(
        err.contains(v["broken"]["address"].as_str().unwrap()),
        "{v}"
    );
}

/// Brings a second history of the writer of `p`'s transcript into its
/// home: a copy of that writer's log that disagrees from seq 2 on.
/// Returns the writer and what the refusal said.
fn refuse_a_disagreeing_copy(p: &Process) -> (String, crate::forks::Refused) {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = Store::open(&home.store()).unwrap();
    let writer = store
        .writers()
        .unwrap()
        .into_iter()
        .max_by_key(|w| store.log(*w).unwrap().len())
        .unwrap();
    let mut copy: Vec<record::chain::Link> = store
        .log(writer)
        .unwrap()
        .into_iter()
        .map(|s| s.link)
        .collect();
    for link in copy.iter_mut().skip(1) {
        link.hash = record::digest::Digest::of(&[b"another history"]);
    }
    drop(store);
    let audit = |o: &audit::entry::Occurrence| ctx_audit(p, o);
    let refused = crate::forks::refuse(&home, &copy, "restore", &mut { audit }).unwrap();

    (writer.to_string(), refused)
}

fn ctx_audit(p: &Process, o: &audit::entry::Occurrence) -> bool {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    Log::new(home.audit()).append(o, p.now).is_ok()
}

#[test]
fn verify_exits_3_reporting_a_refused_fork_by_writer_range_and_digests() {
    let p = Process::new();
    let path = p.transcript("a\nb\nc\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    let before = p.invoke(&["verify"], &DEV);
    assert_eq!(before.0, EXIT_OK);
    let (writer, refused) = refuse_a_disagreeing_copy(&p);
    let fork = refused.forks[0];

    let (code, out, err) = p.invoke(&["verify"], &DEV);
    let (json_code, json, _) = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_INTEGRITY, ""));
    assert_eq!(
        err,
        format!(
            "parinn verify: refused: writer {writer} holds two events at seq 2 to {}: {} held, {} offered\n",
            fork.last, fork.held, fork.offered
        )
    );
    assert!(!err.contains("fork choose"), "{err}");
    assert_eq!(json_code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["integrity"], "refused");
    assert_eq!(v["broken"], serde_json::Value::Null);
    assert_eq!(
        v["refused"],
        serde_json::json!([{
            "writer": writer,
            "first": 2,
            "last": fork.last.get(),
            "held": fork.held.to_hex(),
            "offered": fork.offered.to_hex(),
        }])
    );
}

#[test]
fn a_refused_fork_is_audited_counted_and_leaves_the_record_as_it_was() {
    let p = Process::new();
    let path = p.transcript("a\nb\nc\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    let digest = |p: &Process| {
        let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
        let store = Store::open(&home.store()).unwrap();
        store
            .writers()
            .unwrap()
            .into_iter()
            .map(|w| store.log(w).unwrap())
            .collect::<Vec<_>>()
    };
    let held = digest(&p);

    refuse_a_disagreeing_copy(&p);

    assert_eq!(digest(&p), held);
    let audit = audit_text(&p);
    assert_eq!(
        audit.matches("\"counter\":\"fork_conflict\"").count(),
        1,
        "{audit}"
    );
    let (_, status, _) = p.invoke(&["status", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(v["counters"]["fork_conflict"]["total"], 1);
}

#[test]
fn verify_names_a_broken_chain_before_a_refused_fork() {
    let p = Process::new();
    let path = p.transcript("a\nb\nc\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    refuse_a_disagreeing_copy(&p);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute(
        "UPDATE events SET provenance = 'user' WHERE kind = 'unparsed_line' AND seq = 2",
        [],
    )
    .unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_INTEGRITY);
    assert!(err.starts_with("parinn verify: chain broken at"), "{err}");
}

#[test]
fn verify_without_a_store_or_with_a_failing_one_fails() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        err,
        format!(
            "parinn verify: no store at {}\n",
            p.parinn_home().join("store.db").display()
        )
    );

    owner_only_dirs(&p.parinn_home());
    junk_store(&p);
    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("parinn verify: the store: "), "{err}");
}

#[test]
fn verify_reports_a_store_error_met_while_checking() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("UPDATE seats SET room = 'x'", []).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    let (_, json, _) = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!(code, EXIT_INTEGRITY);
    assert!(
        err.starts_with("parinn verify: the store is corrupt"),
        "{err}"
    );
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        (&v["broken"]["class"], &v["broken"]["address"]),
        (&serde_json::json!("corrupt"), &serde_json::Value::Null)
    );
}

#[test]
fn verify_reports_a_store_that_fails_while_checking() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("DROP TABLE event_heads", []).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("parinn verify: the store: no such table"),
        "{err}"
    );
}

#[test]
fn verify_fails_naming_the_store_when_its_refused_forks_cannot_be_read() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("CREATE TABLE fork_refusals (unrelated TEXT)", [])
        .unwrap();

    let (code, out, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(out, "");
    assert!(err.starts_with("parinn verify: the store: "), "{err}");
}

#[test]
fn verify_of_a_home_without_its_device_key_fails() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    std::fs::remove_file(p.parinn_home().join("keys/device.key")).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("no key in the key store entry device"),
        "{err}"
    );
}

#[test]
fn verify_refuses_an_unknown_flag() {
    assert_eq!(
        invoke(&["verify", "--yaml"], &DEV),
        (
            EXIT_USAGE,
            String::new(),
            "parinn verify: unknown flag \"--yaml\"\n".into()
        )
    );
}

#[test]
fn mcp_answers_each_line_of_input() {
    let mut p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    p.lines = vec![
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.into(),
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#.into(),
    ];

    let (code, out, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let lines: Vec<serde_json::Value> = out
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines[0]["result"]["serverInfo"]["version"], "dev");
    assert_eq!(lines[1]["id"], 2);
}

#[test]
fn mcp_refuses_flags_and_reports_a_store_it_cannot_open() {
    for flag in ["--x", "--json"] {
        assert_eq!(
            invoke(&["mcp", flag], &DEV),
            (
                EXIT_USAGE,
                String::new(),
                format!("parinn mcp: unknown flag {flag:?}\n")
            )
        );
    }
    let p = Process::new();
    owner_only_dirs(&p.parinn_home());
    junk_store(&p);
    let (code, _, err) = p.invoke(&["mcp"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("parinn mcp: the store: "), "{err}");
}

#[test]
fn mcp_starts_the_node_it_records_in_and_refuses_one_whose_device_key_is_missing() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(p.parinn_home().join("store.db").exists());
    std::fs::remove_file(p.parinn_home().join("keys/device.key")).unwrap();
    let (code, _, err) = p.invoke(&["mcp"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("the device key is missing"), "{err}");
}

/// A `tools/call` of `event_get`, carrying the tool-use id `tool_use_id`
/// when given.
fn tool_call(id: u64, tool_use_id: Option<&str>) -> String {
    let mut params = serde_json::json!({"name": "event_get", "arguments": {"address": "x"}});
    if let Some(t) = tool_use_id {
        params["_meta"] = serde_json::json!({"claudecode/toolUseId": t});
    }
    serde_json::json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": params})
        .to_string()
}

#[test]
fn a_bound_mcp_server_serves_a_call_under_its_tool_use_ids_run_and_counts_each_refusal() {
    let mut p = Process::new();
    let session = "5e55a0de-0000-4000-8000-0000000000e1";
    let start = crate::hook_handlers::tests::session_start(&p, session, "startup");
    let path: String =
        serde_json::from_str::<serde_json::Value>(&start).unwrap()["transcript_path"]
            .as_str()
            .unwrap()
            .to_owned();
    let call = serde_json::json!({"type": "assistant", "message": {"content": [
        {"type": "tool_use", "id": "toolu_1", "name": "event_get"}
    ]}});
    std::fs::write(&path, format!("{call}\n")).unwrap();
    p.lines = vec![start];
    p.invoke(&["hook", "SessionStart"], &DEV);
    p.lines = vec![
        tool_call(1, Some("toolu_1")),
        tool_call(2, None),
        tool_call(3, Some("toolu_none")),
    ];

    let (code, out, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let texts: Vec<String> = out
        .lines()
        .map(|l| {
            let reply: serde_json::Value = serde_json::from_str(l).unwrap();
            reply["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert!(!texts[0].contains("tool-use id"), "served: {}", texts[0]);
    assert!(texts[1].contains("carries no tool-use id"), "{}", texts[1]);
    assert!(texts[2].contains("within 2 s"), "{}", texts[2]);
    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    for counter in ["tool_call_without_tool_use_id", "tool_call_run_not_found"] {
        assert_eq!(
            counters
                .iter()
                .filter(|c| c.as_deref() == Some(counter))
                .count(),
            1,
            "{counter}: {counters:?}"
        );
    }
    assert!(
        !counters.contains(&Some("session_binding_not_taken".to_owned())),
        "taken"
    );
}

/// Leaves, in `p`'s home, the start marker an MCP server killed before
/// it stopped would, naming the transcript at `path`.
fn stale_marker(p: &Process, path: &str) {
    let dir = p.parinn_home().join("mcp");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("0011223344556677.marker"), format!("{path}\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::set_permissions(
            dir.join("0011223344556677.marker"),
            std::fs::Permissions::from_mode(0o600),
        )
        .unwrap();
    }
}

#[test]
fn the_next_reader_audits_what_a_killed_mcp_server_left() {
    for command in [&["ingest", "--all"][..], &["mcp"]] {
        let p = Process::new();
        let path = p.transcript(CLEAN_LINE);
        p.invoke(&["counter", "ack"], &DEV);
        let resolved = std::fs::canonicalize(&path).unwrap();
        stale_marker(&p, &resolved.display().to_string());

        let (code, _, err) = p.invoke(command, &DEV);

        assert_eq!((code, err.as_str()), (EXIT_OK, ""), "{command:?}");
        let entries: Vec<(Option<String>, String)> = audited(&p)
            .into_iter()
            .map(|e| (e.counter, e.operation))
            .collect();
        let operation = if command[0] == "mcp" { "mcp" } else { "ingest" };
        assert!(
            entries.contains(&(
                Some("ingest_remainder_left".to_owned()),
                operation.to_owned()
            )),
            "{command:?}: {entries:?}"
        );
        assert!(
            !p.parinn_home()
                .join("mcp")
                .join("0011223344556677.marker")
                .exists()
        );
    }
}

#[test]
fn ambiguous_calls_are_counted_on_their_own() {
    use mcp::tool_call::{Cause, Found};
    let p = Process::new();
    p.invoke(&["counter", "ack"], &DEV);
    let input = || mpsc::channel().1;
    let ctx = Context {
        build: &DEV,
        parinn_home: Some(p.parinn_home().into_os_string()),
        home: None,
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

    log_with(&p, None, || {
        record_found(
            &ctx,
            &[Found {
                tool_use_id: Some("toolu_2".to_owned()),
                run: Err(Cause::Ambiguous),
                waited: std::time::Duration::ZERO,
            }],
        );
    });

    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    assert!(counters.contains(&Some("tool_call_run_ambiguous".to_owned())));
    let refused = logged_lines(&p)
        .into_iter()
        .find(|l| l["fields"]["tool_use_id"] == "toolu_2")
        .expect("the refusal is logged");
    assert_eq!(refused["fields"]["cause"], Cause::Ambiguous.as_str());
}

#[test]
fn an_mcp_server_serves_without_live_ingest_while_a_settings_layer_fails() {
    let p = Process::new();
    p.configure("recal.max_k = 10\n");

    let (code, _, _) = p.invoke(&["mcp"], &DEV);

    assert_eq!(code, EXIT_OK);
    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    assert!(
        counters.contains(&Some("ingest_refused_settings_invalid".to_owned())),
        "{counters:?}"
    );
}

#[test]
fn mcp_reports_a_reply_it_cannot_write() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let lines = vec![r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#.to_owned()];
    let input = move || {
        let (tx, rx) = mpsc::channel();
        for line in lines.clone() {
            tx.send(Ok(line)).unwrap();
        }
        rx
    };
    let ctx = Context {
        build: &DEV,
        parinn_home: Some(p.parinn_home().into_os_string()),
        home: None,
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
    let mut err = Vec::new();

    let code = run(["mcp".to_owned()], &ctx, &mut Closed, &mut err);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(String::from_utf8(err).unwrap(), "parinn mcp: closed\n");
}

#[test]
fn a_command_without_a_home_fails() {
    let input = || mpsc::channel().1;
    let ctx = Context {
        build: &DEV,
        parinn_home: None,
        home: None,
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
    for command in ["ingest", "mcp", "verify", "status", "doctor", "counter"] {
        let mut args = vec![command.to_owned()];
        if command == "ingest" {
            args.extend(["--path".to_owned(), "t".to_owned()]);
        }
        if command == "counter" {
            args.push("ack".to_owned());
        }
        let (mut out, mut err) = (Vec::new(), Vec::new());

        let code = run(args, &ctx, &mut out, &mut err);

        assert_eq!(code, EXIT_FAILURE, "{command}");
        let name = if command == "counter" {
            "counter ack"
        } else {
            command
        };
        assert_eq!(
            String::from_utf8(err).unwrap(),
            format!("parinn {name}: no home: set PARINN_HOME or HOME\n")
        );
    }
}

/// Makes `path` and the directories above it that do not exist yet,
/// each readable by its owner alone, as the home's check wants (SEC-02).
pub(crate) fn owner_only_dirs(path: &std::path::Path) {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(path).unwrap();
}

/// Writes a store file that is no database, readable by its owner alone
/// so the store reads it rather than refuse its permissions (SEC-02).
fn junk_store(p: &Process) {
    let path = p.parinn_home().join("store.db");
    std::fs::write(&path, "not a database, but long enough to read a header").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

/// The audit log's entries in `p`'s home.
pub(crate) fn audited(p: &Process) -> Vec<audit::entry::Entry> {
    audit::log::Log::new(p.parinn_home().join("audit"))
        .entries()
        .unwrap()
}

const SESSION_START: &str =
    r#"{"session_id":"s","hook_event_name":"SessionStart","source":"startup"}"#;

#[test]
fn a_hook_with_a_valid_hook_input_injects_nothing_and_audits_nothing() {
    let mut p = Process::new();
    p.lines = vec![crate::hook_handlers::tests::session_start(
        &p, "s1", "startup",
    )];

    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert_eq!(audited(&p), Vec::new());
}

#[test]
fn a_hook_input_that_is_no_json_object_is_audited_and_counted() {
    let mut p = Process::new();
    p.lines = vec!["{\"session_id\": ".to_owned()];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(
        (
            e.kind.as_str(),
            e.counter.as_deref(),
            e.operation.as_str(),
            e.detail.as_str(),
            e.time
        ),
        (
            "rejected",
            Some("hook_rejected"),
            "hook UserPromptSubmit",
            "not a JSON object",
            NOW
        )
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode =
            |path: &std::path::Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&p.parinn_home()), 0o700);
        assert_eq!(
            mode(&p.parinn_home().join("audit/audit-000001.jsonl")),
            0o600
        );
    }
}

#[test]
fn a_hook_input_that_cannot_be_read_or_never_ends_is_rejected() {
    let mut p = Process::new();
    p.input_error = true;
    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    p.input_error = false;
    p.held_open = true;
    p.lines = vec![SESSION_START.to_owned()];
    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let details: Vec<String> = audited(&p).into_iter().map(|e| e.detail).collect();
    assert_eq!(details, ["not a JSON object", "not a JSON object"]);
}

#[test]
fn a_hook_input_that_trickles_past_its_wait_is_rejected() {
    let p = Process::new();
    // A line every 50 ms, each inside the wait, until the hook stops
    // listening: the whole input must still end within the wait.
    let input = || {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            while tx.send(Ok("{".to_owned())).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
        rx
    };
    let ctx = Context {
        build: &DEV,
        parinn_home: Some(p.parinn_home().into_os_string()),
        home: None,
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
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = run(
        ["hook".to_owned(), "SessionStart".to_owned()],
        &ctx,
        &mut out,
        &mut err,
    );

    assert_eq!(
        (code, out.is_empty(), err.is_empty()),
        (EXIT_OK, true, true)
    );
    let details: Vec<String> = audited(&p).into_iter().map(|e| e.detail).collect();
    assert_eq!(details, ["not a JSON object"]);
}

#[test]
fn a_panic_while_auditing_a_hook_failure_still_exits_0() {
    let p = Process::new();
    let input = || {
        let (tx, rx) = mpsc::channel();
        tx.send(Ok("not json".to_owned())).unwrap();
        rx
    };
    let ctx = Context {
        build: &DEV,
        parinn_home: Some(p.parinn_home().into_os_string()),
        home: None,
        home_id: None,
        hook_panics: false,
        hook_hangs: false,
        terminal: true,
        working_dir: None,
        managed_policy: std::path::PathBuf::from("/nonexistent/managed_policy.toml"),
        node_identity: None,
        os_user: OsUser::Known(u32::MAX),
        input: &input,
        clock: &|| panic!("the clock fails"),
        attended: false,
        hook_budget: Some(LENIENT),
        ancestry: own_harness,
        live_limit: None,
        query_deadline: None,
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = run(
        ["hook".to_owned(), "UserPromptSubmit".to_owned()],
        &ctx,
        &mut out,
        &mut err,
    );

    assert_eq!(
        (code, out.is_empty(), err.is_empty()),
        (EXIT_OK, true, true)
    );
}

#[test]
fn a_hook_parinn_does_not_install_is_rejected_and_exits_0() {
    let mut p = Process::new();
    p.lines = vec![SESSION_START.to_owned()];
    for args in [&["hook", "Stop"][..], &["hook"]] {
        assert_eq!(
            p.invoke(args, &DEV),
            (EXIT_OK, String::new(), String::new()),
            "{args:?}"
        );
    }
    let entries = audited(&p);
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .all(|e| e.operation == "hook" && e.detail == "no such hook"),
        "{entries:?}"
    );
}

#[test]
fn a_hook_handler_that_panics_fails_open_and_is_audited() {
    let mut p = Process::new();
    p.lines = vec![SESSION_START.to_owned()];
    p.hook_panics = true;

    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(
        (
            entries[0].kind.as_str(),
            entries[0].counter.as_deref(),
            entries[0].detail.as_str()
        ),
        ("failed", Some("hook_panicked"), "panic recovered")
    );
}

#[cfg(unix)]
#[test]
fn a_hook_in_a_home_it_must_refuse_writes_nothing_and_exits_0() {
    use std::os::unix::fs::PermissionsExt;
    let mut p = Process::new();
    std::fs::create_dir(p.parinn_home()).unwrap();
    std::fs::set_permissions(p.parinn_home(), std::fs::Permissions::from_mode(0o755)).unwrap();
    p.lines = vec!["not json".to_owned()];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert!(!p.parinn_home().join("audit").exists());
    p.home_id = Some("other".into());
    std::fs::set_permissions(p.parinn_home(), std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(p.parinn_home().join("home.json"), r#"{"id":"mine"}"#).unwrap();
    std::fs::set_permissions(
        p.parinn_home().join("home.json"),
        std::fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert!(!p.parinn_home().join("audit").exists());
}

#[test]
fn a_hook_whose_audit_log_cannot_take_an_entry_still_exits_0() {
    let mut p = Process::new();
    owner_only_dirs(&p.parinn_home().join("audit/audit-000001.jsonl"));
    p.lines = vec!["not json".to_owned()];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
}

#[test]
fn counter_ack_records_a_neutral_principal_act_through_the_last_audit_entry() {
    let mut p = Process::new();

    assert_eq!(
        p.invoke(&["counter", "ack"], &DEV),
        (
            EXIT_OK,
            "counters acknowledged; the audit log held no entry\n".into(),
            String::new()
        )
    );
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    let (code, out, _) = p.invoke(&["counter", "ack", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["acknowledged"], 2);
    let acts = recorded(&p);
    assert_eq!(acts.len(), 2);
    assert_eq!(acts[0]["audit_entry"], serde_json::Value::Null);
    assert_eq!(acts[1]["act"], "counter_ack");
    assert_eq!(acts[1]["class"], "neutral");
    assert_eq!(acts[1]["principal_surface"], "cli");
    assert_eq!(acts[1]["audit_entry"], 2);
    let entries = audited(&p);
    assert_eq!(acts[1]["audit_hash"], entries[1].hash.to_hex());
    let kinds: Vec<String> = entries.into_iter().map(|e| e.kind).collect();
    assert_eq!(kinds, ["principal_act", "rejected", "principal_act"]);
}

#[test]
fn counter_ack_keeps_its_recorded_act_when_the_audit_log_takes_no_note() {
    let mut p = Process::new();
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    let lock = p.parinn_home().join("audit/lock");
    std::fs::remove_file(&lock).unwrap();
    owner_only_dirs(&lock);

    let (code, out, err) = p.invoke(&["counter", "ack"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(
        err.starts_with("parinn counter ack: the acknowledgement is recorded, but the audit log took no note of it: "),
        "{err}"
    );
    assert_eq!(recorded(&p)[0]["audit_entry"], 1);
    assert_eq!(audited(&p).len(), 1);
    std::fs::remove_dir(&lock).unwrap();
    assert_eq!(p.invoke(&["doctor"], &DEV).0, EXIT_OK);
}

#[test]
fn an_acknowledgement_hides_no_failure_once_the_audit_log_began_again() {
    let mut p = Process::new();
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    assert_eq!(p.invoke(&["counter", "ack"], &DEV).0, EXIT_OK);
    std::fs::remove_dir_all(p.parinn_home().join("audit")).unwrap();
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);

    let (code, out, _) = p.invoke(&["doctor"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        out,
        "counter hook_rejected rose by 1 since the counters were last acknowledged\n"
    );
}

/// The content of every principal act the home's store holds.
pub(crate) fn recorded(p: &Process) -> Vec<serde_json::Value> {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = Store::open(&home.store()).unwrap();
    let node = Node::find(&home, &store).unwrap();
    act_record::acts(&store, &node)
        .unwrap()
        .into_iter()
        .map(|a| a.content)
        .collect()
}

#[test]
fn counter_ack_without_a_terminal_is_refused_audited_and_acknowledges_nothing() {
    let mut p = Process::new();
    p.terminal = false;

    assert_eq!(
        p.invoke(&["counter", "ack"], &DEV),
        (
            EXIT_FAILURE,
            String::new(),
            "parinn counter ack: refused: a principal act needs a terminal on standard input and \
             output (OWN-12)\n"
                .into()
        )
    );
    assert!(!p.parinn_home().join("store.db").exists());
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].counter.as_deref(), Some("principal_act_refused"));
    assert_eq!(entries[0].operation, "counter ack");
}

#[test]
fn counter_needs_ack_and_refuses_other_flags() {
    for (args, why) in [
        (&["counter"][..], "parinn counter: want ack\n"),
        (&["counter", "reset"], "parinn counter: want ack\n"),
        (
            &["counter", "ack", "--all"],
            "parinn counter ack: unknown flag \"--all\"\n",
        ),
    ] {
        assert_eq!(
            invoke(args, &DEV),
            (EXIT_USAGE, String::new(), why.into()),
            "{args:?}"
        );
    }
}

#[test]
fn counter_ack_reports_an_audit_log_it_cannot_append_to() {
    let p = Process::new();
    owner_only_dirs(&p.parinn_home().join("audit/audit-000001.jsonl"));

    let (code, _, err) = p.invoke(&["counter", "ack"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("parinn counter ack: ") && err.contains("audit-000001.jsonl"),
        "{err}"
    );
}

#[test]
fn doctor_fails_while_a_counter_rose_since_the_last_acknowledgement() {
    let mut p = Process::new();
    assert_eq!(
        p.invoke(&["doctor"], &DEV),
        (
            EXIT_OK,
            format!(
                "no home at {} yet: nothing to diagnose\n",
                p.parinn_home().display()
            ),
            String::new()
        )
    );
    p.invoke(&["counter", "ack"], &DEV);
    assert_eq!(
        p.invoke(&["doctor"], &DEV),
        (
            EXIT_OK,
            "no counter rose since the counters were last acknowledged\n".into(),
            String::new()
        )
    );
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);

    assert_eq!(
        p.invoke(&["doctor"], &DEV),
        (
            EXIT_FAILURE,
            "counter hook_rejected rose by 2 since the counters were last acknowledged\n".into(),
            String::new()
        )
    );
    let (code, out, _) = p.invoke(&["doctor", "--json"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(out, "{\"risen\":{\"hook_rejected\":2}}\n");

    p.invoke(&["counter", "ack"], &DEV);
    let (code, out, _) = p.invoke(&["doctor", "--json"], &DEV);
    assert_eq!((code, out.as_str()), (EXIT_OK, "{\"risen\":{}}\n"));
}

#[test]
fn doctor_and_status_exit_3_on_a_broken_audit_log() {
    let mut p = Process::new();
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    let file = p.parinn_home().join("audit/audit-000001.jsonl");
    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, text.replace("rejected", "rejecteD")).unwrap();

    for command in ["doctor", "status"] {
        let (code, _, err) = p.invoke(&[command], &DEV);
        assert_eq!(code, EXIT_INTEGRITY, "{command}");
        assert_eq!(
            err,
            format!("parinn {command}: audit log audit-000001.jsonl line 1: hash does not match\n")
        );
    }
}

#[test]
fn doctor_and_status_refuse_unknown_flags() {
    for command in ["doctor", "status"] {
        assert_eq!(
            invoke(&[command, "--fix"], &DEV),
            (
                EXIT_USAGE,
                String::new(),
                format!("parinn {command}: unknown flag \"--fix\"\n")
            )
        );
    }
}

#[test]
fn status_shows_the_home_the_store_the_audit_log_and_the_counters() {
    let mut p = Process::new();
    let home = p.parinn_home().display().to_string();
    let id = identity_lines();
    assert_eq!(
        p.invoke(&["status"], &DEV),
        (
            EXIT_OK,
            format!(
                "home: {home} (none yet)\ndeployment mode: automation\nkeys: files only this OS user can read; an unsandboxed agent of the same OS user could read them\n{id}"
            ),
            String::new()
        )
    );
    Home::locate(Some(p.parinn_home().as_os_str()), None)
        .unwrap()
        .create()
        .unwrap();
    assert_eq!(
        p.invoke(&["status"], &DEV).1,
        format!(
            "home: {home}\nstore: {home}/store.db (none yet)\naudit log: {home}/audit (0 entries)\n\
             counters: none\ndeployment mode: automation\nkeys: files only this OS user can read; an unsandboxed agent of the same OS user could read them\n{id}"
        )
    );
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);

    let (code, out, err) = p.invoke(&["status"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let bytes = std::fs::metadata(p.parinn_home().join("store.db"))
        .unwrap()
        .len();
    assert_eq!(
        out,
        format!(
            "home: {home}\nstore: {home}/store.db ({bytes} bytes)\naudit log: {home}/audit (1 entries)\n\
             counters: hook_rejected 1 (1 since acknowledged)\ndeployment mode: automation\nkeys: files only this OS user can read; an unsandboxed agent of the same OS user could read them\n{id}"
        )
    );
    let (code, out, _) = p.invoke(&["status", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v,
        serde_json::json!({
            "home": home,
            "store": {"path": format!("{home}/store.db"), "bytes": bytes},
            "audit": {"path": format!("{home}/audit"), "entries": 1},
            "counters": {"hook_rejected": {"total": 1, "since_ack": 1}},
            "deployment_mode": "automation",
            "awaiting_acceptance": [],
            "managed_policy": {"refused": null},
            "keys": {
                "store": "file",
                "readable_by_agents": true,
                "where": "files only this OS user can read; an unsandboxed agent of the same OS user could read them",
            },
            "node_identity": identity_json(),
            "fork_detection": fork_detection_json(),
            "segments": {"open": [], "newer": []},
            "transcript_sources": {"held": [], "lost": []},
        })
    );
}

#[test]
fn status_of_a_home_not_made_yet_shows_no_store() {
    let fresh = Process::new();
    let (_, out, _) = fresh.invoke(&["status", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v,
        serde_json::json!({
            "home": fresh.parinn_home().display().to_string(),
            "store": null,
            "audit": null,
            "counters": {},
            "deployment_mode": "automation",
            "awaiting_acceptance": [],
            "managed_policy": {"refused": null},
            "keys": {
                "store": "file",
                "readable_by_agents": true,
                "where": "files only this OS user can read; an unsandboxed agent of the same OS user could read them",
            },
            "node_identity": identity_json(),
            "fork_detection": fork_detection_json(),
            "segments": null,
            "transcript_sources": null,
        })
    );
}

#[cfg(unix)]
#[test]
fn every_command_refuses_a_home_others_can_reach_naming_it() {
    use std::os::unix::fs::PermissionsExt;
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    std::fs::set_permissions(p.parinn_home(), std::fs::Permissions::from_mode(0o750)).unwrap();
    let path = p.transcript("b\n");
    for args in [
        &["ingest", "--path", path.as_str()][..],
        &["mcp"],
        &["verify"],
        &["status"],
        &["doctor"],
        &["counter", "ack"],
    ] {
        let (code, out, err) = p.invoke(args, &DEV);
        let name = if args[0] == "counter" {
            "counter ack"
        } else {
            args[0]
        };
        assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""), "{args:?}");
        assert_eq!(
            err,
            format!(
                "parinn {name}: {}: mode 0750 lets other OS users in; the home wants owner-only \
                 permissions\n",
                p.parinn_home().display()
            ),
            "{args:?}"
        );
    }
}

#[test]
fn every_command_refuses_a_home_bound_to_another_home_id() {
    let mut p = Process::new();
    p.invoke(&["counter", "ack"], &DEV);
    let binding = p.parinn_home().join("home.json");
    std::fs::write(&binding, r#"{"id":"home-a"}"#).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binding, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    p.home_id = Some("home-b".into());
    for command in ["verify", "status", "doctor", "mcp"] {
        let (code, _, err) = p.invoke(&[command], &DEV);
        assert_eq!(code, EXIT_FAILURE, "{command}");
        assert_eq!(
            err,
            format!(
                "parinn {command}: the home id PARINN_HOME_ID supplies does not match the home's\n"
            )
        );
    }
    p.home_id = Some("home-a".into());
    assert_eq!(p.invoke(&["doctor"], &DEV).0, EXIT_OK);
}

#[test]
fn verify_checks_the_audit_chain_too() {
    let mut p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    let (code, out, _) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_OK);
    assert!(
        out.ends_with("the audit log's chain holds over 1 entries\n"),
        "{out}"
    );
    let file = p.parinn_home().join("audit/audit-000001.jsonl");
    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, text.replace("rejected", "rejecteD")).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    assert_eq!(
        err,
        "parinn verify: audit log audit-000001.jsonl line 1: hash does not match\n"
    );
    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["broken"],
        serde_json::json!({
            "class": "audit",
            "address": null,
            "message": "audit log audit-000001.jsonl line 1: hash does not match",
        })
    );
}

#[test]
fn a_panic_in_the_mcp_server_is_audited() {
    let p = Process::new();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();

    mcp_panicked(&home, NOW).unwrap();

    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(
        (
            entries[0].counter.as_deref(),
            entries[0].operation.as_str(),
            entries[0].detail.as_str()
        ),
        (Some("mcp_panicked"), "mcp", "panic recovered")
    );
}

#[test]
fn a_caught_mcp_panic_the_audit_log_cannot_take_is_said_on_stderr() {
    let p = Process::new();
    owner_only_dirs(&p.parinn_home().join("audit/audit-000001.jsonl"));
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut err = Vec::new();

    on_mcp_panic(&home, &|| NOW, &mut err)();

    let err = String::from_utf8(err).unwrap();
    assert!(
        err.starts_with("parinn mcp: audit a caught panic: ") && err.contains("audit-000001.jsonl"),
        "{err}"
    );
}

#[test]
fn a_caught_mcp_panic_is_audited_quietly() {
    let p = Process::new();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut err = Vec::new();

    on_mcp_panic(&home, &|| NOW, &mut err)();

    assert!(err.is_empty());
    assert_eq!(audited(&p).len(), 1);
}

#[test]
fn an_audit_log_that_cannot_be_read_fails_verify_doctor_and_status() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    let audit = p.parinn_home().join("audit");
    std::fs::write(&audit, "not a directory").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&audit, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    for command in ["verify", "doctor", "status"] {
        let (code, _, err) = p.invoke(&[command], &DEV);
        assert_eq!(code, EXIT_FAILURE, "{command}");
        assert!(
            err.starts_with(&format!("parinn {command}: {}", audit.display())),
            "{command}: {err}"
        );
    }
}

#[test]
fn ingest_says_which_redaction_it_cannot_audit_and_still_records() {
    let p = Process::new();
    p.configure("[redaction]\nextra_patterns = [\"(\"]\n");
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(code, EXIT_OK);
    // An audit log that takes no entry.
    let audit_dir = p.parinn_home().join("audit");
    std::fs::remove_dir_all(&audit_dir).unwrap();
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let (code, out, err) = p.invoke(
        &["ingest", "--path", &p.transcript("AKIAIOSFODNN7EXAMPLE\n")],
        &DEV,
    );

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.starts_with("ingested 1 lines"), "{out}");
    assert!(
        err.contains("parinn ingest: audit a refused redaction pattern: "),
        "{err}"
    );
    assert!(err.contains("parinn ingest: audit a redaction: "), "{err}");
}

#[test]
fn a_hook_input_past_1_mib_is_audited_and_injects_nothing() {
    let mut p = Process::new();
    p.lines = vec![
        "{\"p\":\"".to_owned(),
        format!("{}\"}}", "x".repeat(1 << 20)),
    ];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].detail, "hook input exceeds 1 MiB");
    assert_eq!(entries[0].counter.as_deref(), Some("hook_input_too_large"));
}

#[test]
fn a_line_the_reader_cut_at_its_limit_is_a_hook_input_past_1_mib() {
    let mut p = Process::new();
    p.lines = vec!["{".to_owned()];
    p.input_too_large = true;

    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert_eq!(audited(&p)[0].detail, "hook input exceeds 1 MiB");
}

#[test]
fn a_hook_input_nested_past_depth_64_is_audited_and_injects_nothing() {
    let mut p = Process::new();
    p.lines = vec![format!("{}{}", "[".repeat(65), "]".repeat(65))];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert_eq!(audited(&p)[0].detail, "hook input nesting exceeds depth 64");
}

#[test]
fn bounded_lines_refuse_a_line_past_the_limit_without_holding_it_and_go_on() {
    let input = "ab\ncdefgh\nij\n".as_bytes();
    let lines: Vec<io::Result<String>> = bounded_lines(input, 4).collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].as_ref().unwrap(), "ab");
    assert_eq!(
        lines[1].as_ref().unwrap_err().kind(),
        io::ErrorKind::FileTooLarge
    );
    assert_eq!(lines[2].as_ref().unwrap(), "ij");
    let whole: Vec<String> = bounded_lines("a\r\nabcd\r\nb".as_bytes(), 4)
        .map(Result::unwrap)
        .collect();
    assert_eq!(whole, ["a", "abcd", "b"]);
    // One byte past the limit, its line ending read with it.
    let just_over: Vec<io::Result<String>> = bounded_lines("abcde\nx".as_bytes(), 4).collect();
    assert!(just_over[0].is_err());
    assert_eq!(just_over[1].as_ref().unwrap(), "x");
    let at_end: Vec<io::Result<String>> = bounded_lines("abcdef".as_bytes(), 4).collect();
    assert_eq!(at_end.len(), 1);
    assert!(at_end[0].is_err());
    struct Failing;
    impl io::Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("gone"))
        }
    }
    let failed: Vec<io::Result<String>> = bounded_lines(io::BufReader::new(Failing), 4).collect();
    assert_eq!(failed.len(), 1);
    assert!(failed[0].is_err());
    let bad: Vec<io::Result<String>> = bounded_lines(&[0xff, b'\n', b'a'][..], 4).collect();
    assert_eq!(
        bad[0].as_ref().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(bad[1].as_ref().unwrap(), "a");
}

/// Logs one event carrying an AWS access key through the structured
/// log of `p`'s home, with `home_id` supplied.
fn log_one(p: &Process, home_id: Option<&str>) -> std::sync::Arc<structured_log::StructuredLog> {
    log_with(p, home_id, || {
        tracing::info!(key = "AKIAIOSFODNN7EXAMPLE", "logged");
    })
}

/// The structured log of `p`'s home, with `home_id` supplied, its
/// settings layers where `p`'s commands find them, as it stands after
/// `logging` ran under it.
fn log_with(
    p: &Process,
    home_id: Option<&str>,
    logging: impl FnOnce(),
) -> std::sync::Arc<structured_log::StructuredLog> {
    let ctx = crate::settings::tests::ctx_of(p);
    let (managed, os_user) = (ctx.managed_policy.clone(), ctx.os_user);
    let log = std::sync::Arc::new(structured_log(
        None,
        Some(p.dir.path().as_os_str().to_owned()),
        home_id.map(Into::into),
        LogSettings { managed, os_user },
        || NOW,
    ));
    let dispatch = tracing::Dispatch::from(log.clone());
    tracing::dispatcher::with_default(&dispatch, logging);
    log
}

fn logged_lines(p: &Process) -> Vec<Value> {
    std::fs::read_to_string(log_file(p))
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// The log redacts each field with the principal's redaction patterns
/// too, and writes no line while the principal's configuration fails,
/// so its patterns are unknown (SEC-08).
#[test]
fn the_structured_log_redacts_with_the_principal_patterns_or_not_at_all() {
    let p = Process::new();
    p.configure("[redaction]\nextra_patterns = [\"ACME-[0-9]{8}\"]\n");
    let log = log_with(&p, None, || {
        tracing::info!(key = "id ACME-12345678", "logged")
    });
    assert_eq!(log.dropped(), 0);
    assert_eq!(
        logged_lines(&p)[0]["fields"]["key"],
        "id [REDACTED:extra_pattern_1]"
    );

    p.configure("recal.max_k = 10\n");
    let log = log_with(&p, None, || {
        tracing::info!(key = "id ACME-12345678", "logged")
    });
    assert_eq!(log.dropped(), 1);
    assert_eq!(logged_lines(&p).len(), 1);
}

/// A repository's redaction patterns are never applied, not even to the
/// log, and a failing repository configuration leaves the log written
/// (SEC-08, SEC-11).
#[test]
fn the_structured_log_ignores_a_repository_configuration() {
    let p = Process::new();
    p.configure("recall.max_k = 10\n");
    p.repository("[redaction]\nextra_patterns = [\"keep-me\"]\n");
    log_with(&p, None, || tracing::info!(key = "keep-me", "logged"));
    assert_eq!(logged_lines(&p)[0]["fields"]["key"], "keep-me");

    p.repository("recall.max_k = 'ten'\n");
    let log = log_with(&p, None, || tracing::info!(key = "x", "logged"));
    assert_eq!(log.dropped(), 0);
    assert_eq!(logged_lines(&p).len(), 2);
}

/// The transcript path ingest logs is redacted with the harness account
/// identifiers the transcript taught it (SEC-08).
#[test]
fn the_logged_transcript_path_is_redacted_with_the_harness_account_identifiers() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let dir = p.dir.path().join(".claude/projects/someone@example.com");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("t.jsonl");
    let context = r#"{"attachment":{"type":"session_context","context":{"userEmail":"someone@example.com"}}}"#;
    std::fs::write(&path, format!("{context}\n")).unwrap();
    let path = path.display().to_string();

    log_with(&p, None, || {
        assert_eq!(p.invoke(&["ingest", "--path", &path], &DEV).0, 0);
    });

    let logged = logged_lines(&p);
    let transcript = logged[0]["fields"]["transcript"].as_str().unwrap();
    assert!(!transcript.contains("someone@example.com"), "{transcript}");
}

fn log_file(p: &Process) -> std::path::PathBuf {
    p.parinn_home().join("logs").join(structured_log::LOG_FILE)
}

#[test]
fn the_structured_log_waits_for_the_home_and_redacts_each_field() {
    let p = Process::new();
    log_one(&p, None);
    assert!(!p.parinn_home().exists(), "a log makes no home");

    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let log = log_one(&p, None);

    assert_eq!(log.dropped(), 0);
    let text = std::fs::read_to_string(log_file(&p)).unwrap();
    let line: Value = serde_json::from_str(text.trim_end()).unwrap();
    assert_eq!(line["fields"]["key"], "[REDACTED:aws_access_key]");
    assert_eq!(line["time"], NOW);
    assert!(
        Home::locate(Some(p.parinn_home().as_os_str()), None)
            .unwrap()
            .check()
            .is_ok()
    );
}

#[test]
fn a_home_the_core_refuses_gets_no_log() {
    let mut p = Process::new();
    p.home_id = Some("h1".to_owned());
    std::fs::create_dir_all(p.parinn_home()).unwrap();
    set_mode(&p.parinn_home(), 0o700);
    std::fs::write(p.parinn_home().join("home.json"), r#"{"id":"h1"}"#).unwrap();
    set_mode(&p.parinn_home().join("home.json"), 0o600);

    log_one(&p, Some("h2"));
    assert!(!log_file(&p).exists());
    set_mode(&p.parinn_home(), 0o755);
    log_one(&p, Some("h1"));
    assert!(!log_file(&p).exists());
    let none = structured_log(None, None, None, LogSettings::default(), || NOW);
    let dispatch = tracing::Dispatch::new(none);
    tracing::dispatcher::with_default(&dispatch, || tracing::info!("nowhere"));
}

#[test]
fn dropped_log_lines_are_audited_one_entry_each() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let before = audited(&p).len();
    let ctx = crate::settings::tests::ctx_of(&p);

    audit_dropped_log_lines(&ctx, 2);

    let entries = audited(&p);
    assert_eq!(entries.len(), before + 2);
    assert_eq!(entries[before].counter.as_deref(), Some("log_line_dropped"));
    assert_eq!(entries[before].detail, "log line dropped");
}

#[test]
fn ingest_refuses_a_path_outside_the_allowed_roots_and_reads_nothing() {
    let p = Process::new();
    let outside = p.dir.path().join("t.jsonl");
    std::fs::write(&outside, "AKIAIOSFODNN7EXAMPLE\n").unwrap();
    let climbing = format!(
        "{}/../../t.jsonl",
        p.transcript("a\n").trim_end_matches("/t.jsonl")
    );

    for path in [outside.display().to_string(), climbing] {
        let (code, out, err) = p.invoke(&["ingest", "--path", &path], &DEV);

        assert_eq!(
            (code, out.as_str(), err.as_str()),
            (
                EXIT_FAILURE,
                "",
                "parinn ingest: path outside allowed roots\n"
            )
        );
    }
    let entries = audited(&p);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].detail, "path outside allowed roots");
    assert_eq!(
        entries[0].counter.as_deref(),
        Some("path_outside_allowed_roots")
    );
    assert!(!p.parinn_home().join("store.db").exists());
}

/// While the principal's configuration fails, ingest cannot know the
/// redaction patterns the principal asked for, so it records nothing:
/// the refusal is audited and counted, and the transcript stays for an
/// ingest once the configuration is fixed (SEC-08, I1, I6).
#[test]
fn ingest_refuses_while_a_settings_layer_fails() {
    let p = Process::new();
    p.configure("recal.max_k = 10\n");
    let transcript = p.transcript("{\"a\":1}\n");

    let (code, out, err) = p.invoke(&["ingest", "--path", &transcript], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(
        err.contains("parinn ingest: a settings layer failed"),
        "{err}"
    );
    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    assert!(
        counters.contains(&Some("settings_layer_invalid".to_owned())),
        "{counters:?}"
    );
    assert!(
        counters.contains(&Some("ingest_refused_settings_invalid".to_owned())),
        "{counters:?}"
    );
    assert!(!p.parinn_home().join("store.db").exists());

    p.configure("recall.max_k = 10\n");
    assert_eq!(p.invoke(&["ingest", "--path", &transcript], &DEV).0, 0);
}

#[test]
fn ingest_reads_within_the_configured_transcript_roots_only() {
    let p = Process::new();
    let elsewhere = p.dir.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::fs::write(elsewhere.join("t.jsonl"), "a\n").unwrap();
    p.configure(&format!(
        "[transcript]\nroots = [{:?}]\n",
        elsewhere.display().to_string()
    ));
    let (code, _, err) = p.invoke(
        &[
            "ingest",
            "--path",
            &elsewhere.join("t.jsonl").display().to_string(),
        ],
        &DEV,
    );
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));

    // Nothing in the home is a transcript: not its keys, not its store.
    let device_key = p.parinn_home().join("keys").join("device.key");
    let store = p.parinn_home().join("store.db");
    for path in [device_key, store] {
        let (code, _, err) = p.invoke(&["ingest", "--path", &path.display().to_string()], &DEV);
        assert_eq!(
            (code, err.as_str()),
            (EXIT_FAILURE, "parinn ingest: path outside allowed roots\n")
        );
    }
    // The default root no longer applies once the roots are configured.
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(code, EXIT_FAILURE);
}

#[test]
fn ingest_says_when_it_cannot_audit_its_refusal() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    p.configure("recal.max_k = 10\n");
    let audit_dir = p.parinn_home().join("audit");
    let _ = std::fs::remove_dir_all(&audit_dir);
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("parinn ingest: audit a refused ingest: "),
        "{err}"
    );
}

#[test]
fn ingest_says_when_it_cannot_audit_a_path_outside_the_roots() {
    let p = Process::new();
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(code, EXIT_OK);
    let audit_dir = p.parinn_home().join("audit");
    let _ = std::fs::remove_dir_all(&audit_dir);
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let (code, _, err) = p.invoke(&["ingest", "--path", "/etc/passwd"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("parinn ingest: audit a path outside allowed roots: "),
        "{err}"
    );
    assert!(
        err.ends_with("parinn ingest: path outside allowed roots\n"),
        "{err}"
    );
}

/// Every audit entry the home at `p` holds, as text.
fn audit_text(p: &Process) -> String {
    let dir = p.parinn_home().join("audit");
    let mut text = String::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return text;
    };
    for entry in entries {
        text.push_str(&std::fs::read_to_string(entry.unwrap().path()).unwrap());
    }
    text
}

#[test]
fn ingest_audits_and_counts_a_node_identity_change_before_it_appends() {
    let mut p = Process::new();
    let id = p.dir.path().join("machine-id");
    std::fs::write(&id, "first").unwrap();
    p.node_identity = Some(id.clone());
    assert_eq!(
        p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV)
            .0,
        EXIT_OK
    );
    std::fs::write(&id, "second").unwrap();

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("b\n")], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let (_, out, _) = p.invoke(&["status", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["counters"]["node_identity_changed"]["total"], 1);
    assert!(
        audit_text(&p)
            .contains("node identity changed: a new device key and device seat replace the old")
    );
    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let kinds: Vec<&str> = v["seats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["seat_kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds.iter().filter(|k| **k == "device").count(), 2);
    assert_eq!(kinds.iter().filter(|k| **k == "run").count(), 2);
}

#[test]
fn ingest_refuses_a_missing_or_unreadable_device_key_audited_and_counted() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let key = p.parinn_home().join("keys/device.key");
    std::fs::remove_file(&key).unwrap();

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("b\n")], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("no second genesis starts"), "{err}");
    assert!(audit_text(&p).contains("device key missing from the key store"));

    std::fs::create_dir(&key).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("c\n")], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(
        audit_text(&p).contains("key store locked or unreadable"),
        "{err}"
    );
}

#[test]
fn a_principal_act_audits_a_node_identity_change_and_counts_no_act_from_before_it() {
    let mut p = Process::new();
    let id = p.dir.path().join("machine-id");
    std::fs::write(&id, "first").unwrap();
    p.node_identity = Some(id.clone());
    p.invoke(&["counter", "ack"], &DEV);
    std::fs::write(&id, "second").unwrap();

    let (code, _, err) = p.invoke(&["counter", "ack"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(audit_text(&p).contains("node identity changed"));
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![id]));
    let store = Store::open(&home.store()).unwrap();
    let node = Node::find(&home, &store).unwrap();
    assert_eq!(act_record::acts(&store, &node).unwrap().len(), 1);
}

#[test]
fn ingest_fails_on_a_store_the_node_cannot_be_found_in_and_audits_no_key_failure() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("UPDATE seats SET certificate = '{'", [])
        .unwrap();
    let before = audit_text(&p);

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("b\n")], &DEV);

    assert_eq!(code, EXIT_FAILURE, "{err}");
    let after = audit_text(&p);
    assert!(!after.contains("key store"), "{after}");
    assert!(!after.contains("device key missing"), "{after}");
    assert!(after.len() >= before.len());
}

/// Runs `statement` on the store in `p`'s home.
fn store_sql(p: &Process, statement: &str) {
    rusqlite::Connection::open(p.parinn_home().join("store.db"))
        .unwrap()
        .execute_batch(statement)
        .unwrap();
}

/// Appends two events to a writer of the store in `p`'s home, leaving
/// its segment open, as a writer that stopped before closing it would;
/// its writer.
fn writer_left_open(p: &Process) -> store::WriterId {
    let writer = store::WriterId::derive(&[7; 32]);
    let new = |n: u8| store::NewEvent {
        kind: record::event::EventKind::UnparsedLine,
        provenance: record::event::Provenance::Unparsed,
        content: serde_json::json!({"line": n}),
        commitment_key: store::CommitmentKey::from_bytes([n; 32]),
        transcript_timestamp: None,
    };
    Store::open(&p.parinn_home().join("store.db"))
        .unwrap()
        .append_open(writer, &[new(1), new(2)])
        .unwrap();
    writer
}

#[test]
fn status_reports_each_writer_chain_ended_without_a_closed_segment() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    let writer = writer_left_open(&p);

    let (code, out, err) = p.invoke(&["status"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(
        out.contains(&format!(
            "\nwriter chains ended without a closed segment: {writer} (seq 1 to 2)\n"
        )),
        "{out}"
    );
    let (code, out, _) = p.invoke(&["status", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["segments"],
        serde_json::json!({
            "open": [{"writer": writer.to_string(), "first": 1, "last": 2}],
            "newer": [],
        })
    );
}

#[test]
fn verify_reports_each_writer_chain_ended_without_a_closed_segment_and_exits_0() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    let (_, out, _) = p.invoke(&["verify", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["open_segments"], serde_json::json!([]));
    let writer = writer_left_open(&p);
    // The writer has no seat certificate: verify names that first, so
    // give the open segment to the ingest seat's writer instead.
    store_sql(
        &p,
        &format!(
            "DELETE FROM segments WHERE writer = '{writer}'; DELETE FROM events WHERE writer = '{writer}'"
        ),
    );
    let ingest = Store::open(&p.parinn_home().join("store.db"))
        .unwrap()
        .open_segments()
        .unwrap();
    assert!(ingest.is_empty());
    store_sql(
        &p,
        "UPDATE segments SET closed = 0 WHERE (writer, first_seq) IN \
         (SELECT writer, max(first_seq) FROM segments WHERE writer IN \
          (SELECT writer FROM events WHERE kind = 'transcript_line') GROUP BY writer)",
    );

    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    let text = p.invoke(&["verify"], &DEV).1;

    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let open = v["open_segments"].as_array().unwrap();
    assert_eq!(open.len(), 1, "{v}");
    assert_eq!(
        (&open[0]["first"], &open[0]["last"]),
        (&serde_json::json!(1), &serde_json::json!(1))
    );
    assert!(
        text.contains(&format!(
            "; writer chains ended without a closed segment: {} (seq 1 to 1)",
            open[0]["writer"].as_str().unwrap()
        )),
        "{text}"
    );
}

#[test]
fn status_and_verify_exit_3_naming_a_newer_store_and_leave_it_unchanged() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    store_sql(
        &p,
        "PRAGMA journal_mode = DELETE; UPDATE meta SET value = '2' WHERE key = 'schema_version'",
    );
    let path = p.parinn_home().join("store.db");
    let before = std::fs::read(&path).unwrap();

    for command in ["status", "verify"] {
        let (code, out, err) = p.invoke(&[command], &DEV);
        assert_eq!(
            (code, out.as_str(), err.as_str()),
            (
                EXIT_INTEGRITY,
                "",
                format!(
                    "parinn {command}: the store is schema version 2, newer than this parinn's 1\n"
                )
                .as_str()
            )
        );
    }
    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("the store is schema version 2"), "{err}");

    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn status_and_verify_exit_3_naming_a_newer_segment() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    store_sql(
        &p,
        "UPDATE segments SET format_version = 2 WHERE writer IN \
         (SELECT writer FROM events WHERE kind = 'transcript_line')",
    );
    let newer = Store::open(&p.parinn_home().join("store.db"))
        .unwrap()
        .newer_segments()
        .unwrap();
    assert_eq!(newer.len(), 1);
    let named = store::StoreError::NewerSegment(newer[0].clone()).to_string();

    let (code, out, err) = p.invoke(&["status"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_INTEGRITY, ""));
    assert!(out.contains(&format!("\nrefused: {named}\n")), "{out}");
    let (code, out, _) = p.invoke(&["status", "--json"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["segments"]["newer"][0]["format_version"], 2);

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(
        (code, err),
        (EXIT_INTEGRITY, format!("parinn verify: {named}\n"))
    );
}

#[test]
fn status_of_a_store_it_cannot_read_fails() {
    let p = Process::new();
    owner_only_dirs(&p.parinn_home());
    Home::locate(Some(p.parinn_home().as_os_str()), None)
        .unwrap()
        .create()
        .unwrap();
    junk_store(&p);

    let (code, _, err) = p.invoke(&["status"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("parinn status: the store: "), "{err}");
}

#[test]
fn ingest_puts_a_line_above_the_configured_payload_threshold_in_the_payload_store() {
    let p = Process::new();
    p.configure("payload.threshold_bytes = 100\n");
    let long = format!(
        "{{\"type\":\"atis-latch\",\"atis\":\"{}\",\"sessionId\":\"s\"}}\n",
        "a".repeat(200)
    );

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript(&long)], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let held = std::fs::read_dir(p.parinn_home().join("payloads"))
        .unwrap()
        .any(|e| {
            std::fs::read_to_string(e.unwrap().path())
                .unwrap()
                .contains(&"a".repeat(200))
        });
    assert!(held, "the line's content is in the payload store");
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
}

#[test]
fn verify_of_a_home_whose_first_start_was_cut_short_before_it_recorded_exits_0() {
    use key::key_store::Entry;
    use key::signing::{DeviceKey, RecoveryKey, SeatKey};
    let p = Process::new();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    home.create().unwrap();
    drop(Store::open(&home.store()).unwrap());
    // A first start killed after it staged its keys, before it recorded.
    let keys = home.key_store();
    DeviceKey::mint(&mut OsEntropy)
        .unwrap()
        .keep(keys.as_ref(), &Entry::staged(&Entry::device()))
        .unwrap();
    RecoveryKey::mint(&mut OsEntropy)
        .unwrap()
        .keep(keys.as_ref(), &Entry::staged(&Entry::recovery()))
        .unwrap();
    SeatKey::mint(&mut OsEntropy)
        .unwrap()
        .keep(keys.as_ref(), &Entry::staged_seat())
        .unwrap();
    key::file::write_secret(&home.staged(), &format!("genesis {}", "0".repeat(32))).unwrap();

    let (code, out, err) = p.invoke(&["verify"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.starts_with("checked 0 writers and 0 events: "), "{out}");
    assert!(
        out.ends_with("; the node's first start was interrupted, and its next start finishes it\n"),
        "{out}"
    );
    let (_, out, _) = p.invoke(&["verify", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        (&v["writers"], &v["staged"]),
        (&serde_json::json!(0), &serde_json::json!(true))
    );

    // Its next start finishes it, and the record verifies against the
    // key it placed.
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    assert_eq!(code, EXIT_OK);
    let (_, out, _) = p.invoke(&["verify", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["staged"], serde_json::json!(false));
}

#[test]
fn a_search_the_deadline_cancels_is_told_in_fixed_text_and_audited() {
    let mut p = Process::new();
    p.invoke(
        &[
            "ingest",
            "--path",
            &p.transcript(&"alpha pad\n".repeat(2100)),
        ],
        &DEV,
    );
    p.query_deadline = Some(Duration::ZERO);
    p.lines = vec![
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.into(),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"event_search","arguments":{"query":"alpha","scope":"room"}}}"#.into(),
    ];

    let (code, out, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let reply: serde_json::Value = serde_json::from_str(out.lines().nth(1).unwrap()).unwrap();
    assert_eq!(reply["result"]["isError"], true);
    let cancelled: Vec<_> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("query_cancelled"))
        .collect();
    assert_eq!(cancelled.len(), 1);
    assert_eq!(
        (
            cancelled[0].operation.as_str(),
            cancelled[0].detail.as_str()
        ),
        ("mcp", "query cancelled at its deadline")
    );
}

#[test]
fn a_cancelled_search_the_audit_log_cannot_take_is_said_on_stderr() {
    let p = Process::new();
    owner_only_dirs(&p.parinn_home().join("audit/audit-000001.jsonl"));
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut err = Vec::new();

    on_query_cancelled(&home, &|| NOW, &mut err)(2);

    let err = String::from_utf8(err).unwrap();
    assert_eq!(
        err.matches("parinn mcp: audit a cancelled search: ")
            .count(),
        2,
        "{err}"
    );
}

#[test]
fn no_cancelled_search_is_audited_when_none_was_cancelled() {
    let p = Process::new();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut err = Vec::new();

    on_query_cancelled(&home, &|| NOW, &mut err)(0);

    assert!(err.is_empty());
    assert!(!p.parinn_home().join("audit").exists());
}

#[test]
fn two_closures_share_one_writer_through_shared() {
    let mut sink: Vec<u8> = Vec::new();
    {
        let cell = RefCell::new(&mut sink as &mut dyn Write);
        let (mut a, mut b) = (Shared(&cell), Shared(&cell));

        a.write_all(b"one ").unwrap();
        b.write_all(b"two").unwrap();
        a.flush().unwrap();
    }

    assert_eq!(sink, b"one two");
}

#[test]
fn ingest_audits_each_gap_of_the_search_index_once_and_status_counts_it() {
    let p = Process::new();
    let path = p.transcript("a\n");
    let (_, out, _) = p.invoke(&["ingest", &format!("--path={path}"), "--json"], &DEV);
    let first: serde_json::Value = serde_json::from_str(&out).unwrap();
    let small = record::id::WriterId::parse(first["writer"].as_str().unwrap()).unwrap();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let large = index_gaps::tests::leave_out_a_writer_sharing_the_slot_of(&home, small);

    for name in ["b.jsonl", "c.jsonl"] {
        let other = transcript_in(&p, "p", name, b"b\n");
        let (code, _, err) = p.invoke(&["ingest", "--path", &other.display().to_string()], &DEV);
        assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    }

    let gaps: Vec<_> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("search_index_writer_unindexed"))
        .collect();
    assert_eq!(gaps.len(), 1, "once, not per ingest");
    assert_eq!(gaps[0].kind, "dropped");
    assert_eq!(gaps[0].operation, "ingest");
    assert!(gaps[0].detail.ends_with(&large.to_string()));
    let (_, status, _) = p.invoke(&["status"], &DEV);
    assert!(
        status.contains("search_index_writer_unindexed 1 (1 since acknowledged)"),
        "{status}"
    );
}

#[test]
fn ingest_says_which_search_index_gap_it_cannot_audit_and_reports_it_at_the_next_ingest() {
    let p = Process::new();
    let path = p.transcript("a\n");
    let (_, out, _) = p.invoke(&["ingest", &format!("--path={path}"), "--json"], &DEV);
    let first: serde_json::Value = serde_json::from_str(&out).unwrap();
    let small = record::id::WriterId::parse(first["writer"].as_str().unwrap()).unwrap();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    index_gaps::tests::leave_out_a_writer_sharing_the_slot_of(&home, small);
    let audit_dir = p.parinn_home().join("audit");
    let kept = std::fs::read_dir(&audit_dir).unwrap().count();
    assert!(kept > 0);
    std::fs::rename(&audit_dir, p.parinn_home().join("audit-kept")).unwrap();
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let other = transcript_in(&p, "p", "b.jsonl", b"b\n");
    let (code, _, err) = p.invoke(&["ingest", "--path", &other.display().to_string()], &DEV);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(
        err.contains(
            "parinn ingest: audit the search index gaps: the audit log refused 1 entries; \
             the next ingest tries again"
        ),
        "{err}"
    );
    std::fs::remove_file(&audit_dir).unwrap();
    std::fs::rename(p.parinn_home().join("audit-kept"), &audit_dir).unwrap();
    let again = transcript_in(&p, "p", "c.jsonl", b"c\n");
    p.invoke(&["ingest", "--path", &again.display().to_string()], &DEV);
    let gaps = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("search_index_writer_unindexed"))
        .count();
    assert_eq!(gaps, 1);
}

#[test]
fn ingest_counts_and_says_a_lookup_of_the_search_index_gaps_that_failed() {
    let p = Process::new();
    let path = p.transcript("a\n");
    p.invoke(&["ingest", &format!("--path={path}")], &DEV);
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut ingested = ingest::ingest_path(
        &home,
        std::path::Path::new(&path),
        &Redactor::new(&[]).unwrap().0,
        &mut key::random::OsEntropy,
    )
    .unwrap();
    ingested.index_gaps_unread = Some("the store has no search index".to_owned());
    let input = || mpsc::channel().1;
    let ctx = Context {
        build: &DEV,
        parinn_home: Some(p.parinn_home().into_os_string()),
        home: None,
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
    let mut err = Vec::new();

    audit_index_gaps(&ctx, &ingested, &mut err);

    assert_eq!(
        String::from_utf8(err).unwrap(),
        "parinn ingest: look up the search index gaps: the store has no search index\n"
    );
    let counted: Vec<_> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("search_index_gap_lookup_failed"))
        .collect();
    assert_eq!(counted.len(), 1);
    assert_eq!(
        (counted[0].kind.as_str(), counted[0].operation.as_str()),
        ("failed", "ingest")
    );
}

#[test]
fn ingest_says_a_search_index_gap_it_cannot_audit_when_the_home_cannot_be_found() {
    let p = Process::new();
    let path = p.transcript("a\n");
    p.invoke(&["ingest", &format!("--path={path}")], &DEV);
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut ingested = ingest::ingest_path(
        &home,
        std::path::Path::new(&path),
        &Redactor::new(&[]).unwrap().0,
        &mut key::random::OsEntropy,
    )
    .unwrap();
    ingested.index_gaps = vec![store::IndexGap {
        writer: record::id::WriterId::derive(&[7; 32]),
        class: store::GapClass::WriterUnindexed,
    }];
    let input = || mpsc::channel().1;
    let ctx = Context {
        build: &DEV,
        parinn_home: None,
        home: None,
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
    let mut err = Vec::new();

    audit_index_gaps(&ctx, &ingested, &mut err);

    let said = String::from_utf8(err).unwrap();
    assert!(
        said.starts_with("parinn ingest: audit the search index gaps: "),
        "{said}"
    );
}

/// The two lines `parinn status` ends its facts with on this OS, each
/// followed by a newline.
fn identity_lines() -> String {
    let against = match identity_against() {
        Some(against) => format!("checked ({against})"),
        None => "not checked on this OS".to_owned(),
    };

    format!(
        "node identity: {against}\nfork detection: local scan by parinn verify; peer detection with the peer network\n"
    )
}

/// What the node identity is checked against on this OS.
fn identity_against() -> Option<&'static str> {
    if cfg!(target_os = "linux") {
        Some("Linux machine id")
    } else if cfg!(target_os = "macos") {
        Some("macOS hardware UUID")
    } else if cfg!(windows) {
        Some("Windows MachineGuid")
    } else {
        None
    }
}

/// `parinn status --json`'s node identity on this OS.
fn identity_json() -> serde_json::Value {
    let against = identity_against();

    serde_json::json!({"checked": against.is_some(), "against": against})
}

/// `parinn status --json`'s fork detection.
fn fork_detection_json() -> serde_json::Value {
    serde_json::json!({"local": "parinn verify", "peer": "with the peer network"})
}

/// How many events the store holds.
fn event_count(p: &Process) -> i64 {
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.query_row("SELECT count(*) FROM events", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn rebuild_derives_the_origins_again_and_appends_no_event() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "s1.jsonl", CLEAN_LINE.as_bytes());
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    let before = event_count(&p);

    let (code, out, err) = p.invoke(&["rebuild", "--json"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["origins"], 2);
    assert_eq!(v["worktrees"], 0);
    assert_eq!(v["indexed"], before);
    assert_eq!(event_count(&p), before);
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
    let (_, text, _) = p.invoke(&["rebuild"], &DEV);
    assert!(
        text.starts_with("rebuilt: 2 origins, 0 worktrees"),
        "{text}"
    );
}

#[test]
fn a_kept_worktree_identity_the_record_lacks_fails_verify_until_rebuild_derives_it_again() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "s1.jsonl", CLEAN_LINE.as_bytes());
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_OK);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute(
        "INSERT INTO repository_worktrees (worktree, identity) VALUES ('stray', 'x')",
        [],
    )
    .unwrap();
    drop(conn);

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    assert!(
        err.contains("the identity kept for worktree stray"),
        "{err}"
    );

    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
}

#[test]
fn a_kept_origin_changed_in_place_fails_verify_until_rebuild_derives_it_again() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "s1.jsonl", CLEAN_LINE.as_bytes());
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_OK);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("UPDATE origins SET origin = 'peer'", [])
        .unwrap();
    drop(conn);

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    assert!(err.contains("derived artifact consistency"), "{err}");
    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    assert!(out.contains("derived_artifact_consistency"), "{out}");

    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
}

#[test]
fn rebuild_without_a_store_fails_and_an_unknown_flag_is_usage() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["rebuild"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("no store at "), "{err}");
    let (code, _, err) = p.invoke(&["rebuild", "--now"], &DEV);
    assert_eq!(code, EXIT_USAGE);
    assert!(err.contains("unknown flag"), "{err}");
}

#[test]
fn rebuild_refuses_a_home_it_cannot_open_a_newer_store_and_a_store_with_no_node() {
    let mut p = Process::new();
    p.invoke(&["counter", "ack"], &DEV);
    let db = p.parinn_home().join("store.db");

    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch("DROP TABLE seats").unwrap();
    drop(conn);
    let (code, _, err) = p.invoke(&["rebuild"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("parinn rebuild: "), "{err}");

    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute(
        "UPDATE meta SET value = '99' WHERE key = 'schema_version'",
        [],
    )
    .unwrap();
    drop(conn);
    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_INTEGRITY);

    p.home_id = Some("home-b".into());
    let binding = p.parinn_home().join("home.json");
    std::fs::write(&binding, r#"{"id":"home-a"}"#).unwrap();
    set_mode(&binding, 0o600);
    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_FAILURE);
}

#[test]
fn status_says_whether_an_mcp_server_took_the_session_bindings() {
    assert_eq!(
        binding_line(0, 2),
        "MCP server: none has taken a session binding on this node (2 untaken)"
    );
    assert_eq!(binding_line(3, 0), "MCP server: took 3 session bindings");
    assert_eq!(
        binding_line(3, 1),
        "MCP server: took 3 session bindings, 1 untaken"
    );
}

#[test]
fn status_reports_the_session_bindings_in_text_and_json_once_any_was_observed() {
    let mut p = Process::new();
    let (_, none, _) = p.invoke(&["status", "--json"], &DEV);
    let none: serde_json::Value = serde_json::from_str(&none).unwrap();
    assert!(none.get("mcp_server").is_none());

    let app = p.dir.path().join("app");
    std::fs::create_dir_all(&app).unwrap();
    p.lines = vec![
        serde_json::json!({
            "session_id": "s1",
            "transcript_path": p.transcript("a\n"),
            "cwd": app,
            "hook_event_name": "SessionStart",
            "source": "startup",
        })
        .to_string(),
    ];
    assert_eq!(p.invoke(&["hook", "SessionStart"], &DEV).0, EXIT_OK);
    p.lines = Vec::new();

    let (_, json, _) = p.invoke(&["status", "--json"], &DEV);
    let json: serde_json::Value = serde_json::from_str(&json).unwrap();
    let (_, text, _) = p.invoke(&["status"], &DEV);

    assert_eq!(json["mcp_server"]["bindings_untaken"], 1);
    assert_eq!(json["mcp_server"]["bindings_taken"], 0);
    assert!(
        text.contains("MCP server: none has taken a session binding on this node (1 untaken)"),
        "{text}"
    );
}
