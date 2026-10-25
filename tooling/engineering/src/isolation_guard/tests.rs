use super::*;
use testkit::TempDir;

/// A probe that ended as `success` with `stdout` and `stderr`.
fn probe(success: bool, stdout: &str, stderr: &str) -> Probe {
    Probe {
        success,
        stdout: stdout.to_owned(),
        stderr: stderr.to_owned(),
    }
}

#[test]
fn a_refusal_that_prints_nothing_and_says_so_is_what_the_guard_does() {
    let got = not_refused("user-home", &probe(false, "", "refusing ... (ENG-14)\n"));

    assert_eq!(got, None);
}

#[test]
fn a_probe_that_resolved_the_real_home_was_not_refused() {
    let got = not_refused("user-home", &probe(true, "/real\n", ""));

    assert_eq!(
        got,
        Some(
            "place user-home: resolved real state instead of aborting: stdout \"/real\", stderr \"\""
                .to_owned()
        )
    );
}

#[test]
fn a_probe_that_printed_a_path_before_failing_was_not_refused() {
    let got = not_refused("user-home", &probe(false, "/real", "(ENG-14)"));

    assert!(got.is_some_and(|p| p.contains("printed a path before aborting")));
}

#[test]
fn a_probe_that_failed_for_another_reason_was_not_refused() {
    let got = not_refused("user-home", &probe(false, "", "panicked at main.rs"));

    assert!(got.is_some_and(|p| p.contains("ended without the guard's refusal")));
}

#[test]
fn a_probe_that_printed_a_path_under_the_isolated_home_was_allowed() {
    let got = not_allowed(
        "user-home",
        &probe(true, "/t/home/.parinn\n", ""),
        Path::new("/t/home"),
    );

    assert_eq!(got, None);
}

#[test]
fn a_probe_that_failed_or_printed_another_path_was_not_allowed() {
    let home = Path::new("/t/home");

    for p in [
        probe(false, "/t/home", "(ENG-14)"),
        probe(true, "/elsewhere", ""),
        probe(true, "", ""),
    ] {
        let got = not_allowed("user-home", &p, home);

        assert!(
            got.is_some_and(|m| m.starts_with("place user-home: refused an isolated place: ")),
            "{p:?}"
        );
    }
}

/// An executable shell script holding `body`, in `dir`, that has
/// started once and can start again.
///
/// A fork another test thread makes can hold the new file open for
/// writing until it execs, and starting the script meanwhile fails with
/// "text file busy". The script exits at once when asked to `--warm`
/// up, and is started that way until the start succeeds.
#[cfg(unix)]
fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join(name);
    let text = format!("#!/bin/sh\n[ \"$1\" = --warm ] && exit 0\n{body}\n");
    fs::write(&path, text).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    for _ in 0..500 {
        match Command::new(&path).arg("--warm").status() {
            Err(e) if e.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                thread::sleep(Duration::from_millis(10));
            }
            _ => break,
        }
    }
    path
}

/// A guard as it should be: it resolves a home under the temporary
/// directory, and refuses any other.
#[cfg(unix)]
const GOOD_GUARD: &str = r#"case "$HOME" in
  "$TMPDIR"/*) echo "$HOME/$PARINN_GUARD_PROBE"; exit 0 ;;
  *) echo "refusing to resolve $HOME (ENG-14)" >&2; exit 134 ;;
esac"#;

#[cfg(unix)]
#[test]
fn a_probe_runs_with_nothing_but_its_home_its_temporary_directory_and_its_place() {
    let dir = TempDir::new().unwrap();
    let exe = script(dir.path(), "env.sh", "env | sort");

    let got = run_probe(
        &exe,
        "parinn-home",
        Path::new("/h"),
        Path::new("/t"),
        Duration::from_secs(30),
    )
    .unwrap();

    assert!(got.success);
    assert_eq!(
        got.stdout
            .lines()
            // A coverage run's profile file, which the probe keeps.
            .filter(|l| !l.starts_with("PWD=") && !l.starts_with("LLVM_PROFILE_FILE="))
            .collect::<Vec<_>>(),
        ["HOME=/h", "PARINN_GUARD_PROBE=parinn-home", "TMPDIR=/t"]
    );
}

#[cfg(unix)]
#[test]
fn a_probe_reports_how_it_ended_and_what_it_said() {
    let dir = TempDir::new().unwrap();
    let exe = script(dir.path(), "fail.sh", "echo out; echo err >&2; exit 3");

    let got = run_probe(
        &exe,
        "user-home",
        dir.path(),
        dir.path(),
        Duration::from_secs(30),
    )
    .unwrap();

    assert_eq!(got, probe(false, "out\n", "err\n"));
}

#[test]
fn a_probe_that_cannot_start_is_named() {
    let dir = TempDir::new().unwrap();

    let err = run_probe(
        &dir.path().join("missing"),
        "user-home",
        dir.path(),
        dir.path(),
        Duration::from_secs(30),
    )
    .unwrap_err();

    assert!(err.starts_with("start probe "), "{err}");
}

#[cfg(unix)]
#[test]
fn a_probe_that_outlives_its_deadline_is_killed() {
    let dir = TempDir::new().unwrap();
    let exe = script(dir.path(), "slow.sh", "exec sleep 30");

    let err = run_probe(
        &exe,
        "user-home",
        dir.path(),
        dir.path(),
        Duration::from_millis(50),
    )
    .unwrap_err();

    assert!(err.contains("home of "), "{err}");
    assert!(err.ends_with("still running after 50ms"), "{err}");
}

#[cfg(unix)]
#[test]
fn a_guard_that_refuses_the_real_home_and_allows_an_isolated_one_holds() {
    let dir = TempDir::new().unwrap();
    let exe = script(dir.path(), "guard.sh", GOOD_GUARD);
    let scratch = dir.path().join("scratch");

    let got = guard_holds(&exe, &scratch, &["user-home", "claude-json"]);

    assert_eq!(got, Vec::<String>::new());
}

#[cfg(unix)]
#[test]
fn a_guard_that_lets_the_real_home_through_does_not_hold() {
    let dir = TempDir::new().unwrap();
    let exe = script(dir.path(), "open.sh", r#"echo "$HOME""#);
    let scratch = dir.path().join("scratch");

    let got = guard_holds(&exe, &scratch, &["user-home", "parinn-home"]);

    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got[0].starts_with("place user-home: resolved real state"));
    assert!(got[1].starts_with("place parinn-home: resolved real state"));
}

#[cfg(unix)]
#[test]
fn a_guard_that_refuses_everything_does_not_hold() {
    let dir = TempDir::new().unwrap();
    let exe = script(dir.path(), "shut.sh", r#"echo "no (ENG-14)" >&2; exit 134"#);

    let got = guard_holds(&exe, &dir.path().join("scratch"), &["user-home"]);

    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].starts_with("place user-home: refused an isolated place"));
}

#[test]
fn a_probe_executable_that_is_missing_fails_every_run() {
    let dir = TempDir::new().unwrap();

    let got = guard_holds(
        &dir.path().join("missing"),
        &dir.path().join("scratch"),
        &["user-home"],
    );

    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got.iter().all(|p| p.starts_with("start probe ")));
}

#[test]
fn homes_that_cannot_be_created_are_reported() {
    let dir = TempDir::new().unwrap();
    let blocker = dir.path().join("file");
    fs::write(&blocker, "x").unwrap();

    let got = guard_holds(Path::new("/bin/true"), &blocker, &["user-home"]);

    assert_eq!(got.len(), 1);
    assert!(
        got[0].starts_with("create the probe's directories: "),
        "{got:?}"
    );
}

#[test]
fn a_read_of_the_home_by_name_is_found_with_its_line() {
    let body = "fn a() {\n    let h = std::env::var_os(\"HOME\");\n}\n";

    assert_eq!(
        unguarded_reads("x.rs", body),
        vec![
            "x.rs:2: reads real state outside testkit::guard: let h = std::env::var_os(\"HOME\");"
                .to_owned()
        ]
    );
}

#[test]
fn every_variable_that_leads_to_the_real_home_is_found() {
    for (read, name) in [
        ("var", "HOME"),
        ("var_os", "PARINN_HOME"),
        ("var", "CLAUDE_CONFIG_DIR"),
        ("var_os", "USERPROFILE"),
        ("env!", "HOME"),
        ("option_env!", "HOME"),
    ] {
        let body = format!("let _ = {read}(\"{name}\");\n");

        assert_eq!(unguarded_reads("x.rs", &body).len(), 1, "{body}");
    }
}

#[test]
fn a_read_wrapped_over_lines_is_found_at_its_name() {
    let body = "let h = std::env::var_os(\n    \"HOME\",\n);\n";

    let got = unguarded_reads("x.rs", body);

    assert_eq!(got.len(), 1);
    assert!(got[0].starts_with("x.rs:2: "), "{got:?}");
}

#[test]
fn the_standard_library_s_home_lookup_is_found() {
    let body = format!("let h = std::env::{}());\n", ["home", "_dir("].concat());

    assert_eq!(unguarded_reads("x.rs", &body).len(), 1);
}

#[test]
fn two_hits_on_one_line_are_one_finding() {
    let body = "let _ = (var(\"HOME\"), var(\"PARINN_HOME\"));\n";

    assert_eq!(unguarded_reads("x.rs", body).len(), 1);
}

#[test]
fn setting_the_home_or_naming_it_is_no_read() {
    for body in [
        "command.env(\"HOME\", home);",
        "let name = \"HOME\";",
        "let pairs = [(\"HOME\", \"/tmp/h\")];",
        "var(NAME)",
        "var_os(CHILD)",
        "let _ = var(\"PATH\");",
    ] {
        assert_eq!(
            unguarded_reads("x.rs", body),
            Vec::<String>::new(),
            "{body}"
        );
    }
}

/// A checkout holding `files`, each a read of the home.
fn checkout(files: &[&str]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    for dir_name in SOURCES {
        fs::create_dir_all(dir.path().join(dir_name)).unwrap();
    }
    for file in files {
        let path = dir.path().join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "let _ = var(\"HOME\");\n").unwrap();
    }
    let c = Checkout::new(dir.path());
    (dir, c)
}

#[test]
fn the_sources_are_scanned_but_the_resolvers_and_build_output() {
    let mut files = vec![
        "crates/a/src/lib.rs",
        "crates/a/tests/t.rs",
        "tooling/b/src/tests.rs",
        "crates/a/notes.txt",
        "tooling/b/target/debug/x.rs",
    ];
    files.extend(RESOLVERS);
    let (_dir, c) = checkout(&files);

    let got = no_unguarded_reads(&c);

    let named: Vec<&str> = got.iter().map(|p| p.split(':').next().unwrap()).collect();
    assert_eq!(
        named,
        [
            "crates/a/src/lib.rs",
            "crates/a/tests/t.rs",
            "tooling/b/src/tests.rs"
        ]
    );
}

#[test]
fn a_checkout_whose_sources_cannot_be_listed_is_reported() {
    let c = Checkout::new(Path::new("/no/such/dir"));

    let got = no_unguarded_reads(&c);

    assert_eq!(got.len(), 2);
    assert!(got[0].starts_with("list /no/such/dir/crates: "), "{got:?}");
}

#[cfg(unix)]
#[test]
fn a_source_that_cannot_be_read_is_reported() {
    let (dir, c) = checkout(&[]);
    std::os::unix::fs::symlink("nowhere", dir.path().join("crates/x.rs")).unwrap();

    let got = no_unguarded_reads(&c);

    assert_eq!(got.len(), 1);
    assert!(got[0].starts_with("read crates/x.rs: "), "{got:?}");
}
