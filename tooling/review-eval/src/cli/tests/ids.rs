//! A run's findings are held to the ids of its world's two trees, as
//! the review holds them (ENG-32).

use super::*;

/// The obedient claude, its log changed by `change` and `then` run in
/// its work directory before it returns.
fn claude_that(change: fn(String) -> String, then: fn(&Path)) -> crate::host::fake::Claude {
    let obedient = obedient_claude();
    Box::new(move |c: &Command| {
        let out = obedient(c)?;
        let log_path = c.stdout_to.as_ref().unwrap();
        fs::write(log_path, change(fs::read_to_string(log_path).unwrap())).unwrap();
        then(&c.cwd);
        Ok(out)
    })
}

fn run_lang_rust(host: &FakeHost, dir: &Path, out_dir: &Path) -> (u8, String, String) {
    run_cli(
        host,
        &[
            "run",
            "--checkout",
            &text(dir),
            "--component",
            "lang-rust",
            "--out",
            &text(out_dir),
            "--repetitions",
            "1",
            "--passes",
            "1",
        ],
    )
}

#[test]
fn run_passes_a_run_whose_finding_cites_an_id_its_world_has_and_the_invariants_lack() {
    let dir = checkout();
    sound_cases(dir.path());
    let mut host = FakeHost::new(dir.path());
    host.claude = claude_that(|log| log, |_| {});
    let out = TempDir::new().unwrap();
    let out_dir = out.path().join("out");

    let (code, out, err) = run_lang_rust(&host, dir.path(), &out_dir);

    assert_eq!(code, EXIT_OK, "{out}\n{err}");
    let report = fs::read_to_string(out_dir.join("runs/lang-rust/defect/1/report.md")).unwrap();
    assert!(report.contains(" — ENG-16\n"), "{report}");
}

#[test]
fn run_fails_a_run_whose_finding_cites_an_id_no_requirement_of_its_world_has() {
    let dir = checkout();
    sound_cases(dir.path());
    let mut host = FakeHost::new(dir.path());
    host.claude = claude_that(|log| log.replace("ENG-16", "ENG-99"), |_| {});
    let out = TempDir::new().unwrap();
    let out_dir = out.path().join("out");

    let (code, out, err) = run_lang_rust(&host, dir.path(), &out_dir);

    assert_eq!(code, EXIT_FAILURE, "{out}\n{err}");
    assert!(
        err.contains("cites ENG-99, which no requirement or invariant has"),
        "{err}"
    );
    let report = fs::read_to_string(out_dir.join("runs/lang-rust/defect/1/report.md")).unwrap();
    assert!(!report.contains("ENG-99"), "{report}");
}

#[test]
fn run_fails_a_run_whose_world_s_requirements_do_not_read() {
    let dir = checkout();
    sound_cases(dir.path());
    let mut host = FakeHost::new(dir.path());
    host.claude = claude_that(
        |log| log,
        |world| {
            let table = "| ID | Pri | Requirement | Ver | Traces |\n|---|---|---|---|---|\n";
            fs::write(
                world.join("docs/srs/99.md"),
                format!("{table}| eng-1 | P0 | Bad. | T | I1 |\n"),
            )
            .unwrap();
        },
    );
    let out = TempDir::new().unwrap();
    let out_dir = out.path().join("out");

    let (code, out, err) = run_lang_rust(&host, dir.path(), &out_dir);

    assert_eq!(code, EXIT_FAILURE, "{out}\n{err}");
    assert!(
        err.contains("docs/srs/99.md:1: a requirement table"),
        "{err}"
    );
}
