use super::*;

/// A writer two threads share, so a test reads what a run has written
/// so far.
#[derive(Clone, Default)]
struct Shared(std::sync::Arc<Mutex<Vec<u8>>>);

impl Write for Shared {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Shared {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

/// A run of `lang-rust`'s three cases one at a time: what its progress
/// showed as the first agent started, and all it showed.
fn progress_run() -> (String, String) {
    let dir = checkout();
    sound_cases(dir.path());
    let err = Shared::default();
    let seen: std::sync::Arc<Mutex<Vec<String>>> = std::sync::Arc::default();
    let (watched, log) = (err.clone(), seen.clone());
    let inner = obedient_claude();
    let mut host = FakeHost::new(dir.path());
    host.claude = Box::new(move |c: &Command| {
        log.lock().unwrap().push(watched.text());
        inner(c)
    });
    let out = TempDir::new().unwrap();
    let out_dir = out.path().join("out");
    let args = [
        "run",
        "--checkout",
        &text(dir.path()),
        "--component",
        "lang-rust",
        "--out",
        &text(&out_dir),
        "--parallel",
        "1",
    ];

    let code = run(
        &host,
        args.iter().map(|a| (*a).to_owned()),
        &mut Vec::new(),
        &mut err.clone(),
    );

    assert_eq!(code, EXIT_OK);
    let first = seen.lock().unwrap()[0].clone();
    (first, err.text())
}

#[test]
fn run_names_what_it_evaluates_before_the_first_agent_runs() {
    let (first, _) = progress_run();

    assert!(
        first.contains("evaluating 3 cases of 1 component, 1 at a time"),
        "{first}"
    );
    assert!(
        first.contains("] lang-rust/") && first.contains(" run 1: started"),
        "{first}"
    );
}

#[test]
fn run_counts_each_decided_case_on_a_clocked_line() {
    let (_, all) = progress_run();

    assert!(all.contains("1/3 cases decided"), "{all}");
    assert!(all.contains("3/3 cases decided"), "{all}");
    assert!(all.lines().all(|l| l.starts_with("[+")), "{all}");
}

#[test]
fn estimate_line_counts_the_cases_of_the_components_named() {
    let dir = checkout();

    assert_eq!(
        estimate_line(dir.path(), &["lang-rust".to_owned()]),
        format!(
            "estimate: 3 cases, {}, {PARALLEL} at a time",
            plan::estimate(3, PARALLEL)
        )
    );
    assert_eq!(
        estimate_line(dir.path(), &["lang-toml".to_owned()]),
        format!(
            "estimate: 0 cases, {}, {PARALLEL} at a time",
            plan::estimate(0, PARALLEL)
        )
    );
}

#[test]
fn estimate_line_names_a_tree_whose_cases_cannot_be_derived() {
    let none = TempDir::new().unwrap();
    let why = check::all_cases_from(none.path(), &none.path().join(CASES), None).unwrap_err();

    assert_eq!(
        estimate_line(none.path(), &["lang-rust".to_owned()]),
        format!("estimate: none, the cases do not load: {why}")
    );
}

#[test]
fn estimate_line_names_cases_that_do_not_load() {
    let dir = TempDir::new().unwrap();
    let bad = dir.path().join(CASES).join("lang-rust/bad.toml");
    fs::create_dir_all(bad.parent().unwrap()).unwrap();
    fs::write(&bad, "nope").unwrap();
    let why = check::all_cases_from(dir.path(), &dir.path().join(CASES), None).unwrap_err();

    assert_eq!(
        estimate_line(dir.path(), &["lang-rust".to_owned()]),
        format!("estimate: none, the cases do not load: {why}")
    );
}

#[test]
fn clock_reads_minutes_seconds_and_hours() {
    assert_eq!(clock(Duration::from_secs(0)), "00:00");
    assert_eq!(clock(Duration::from_secs(245)), "04:05");
    assert_eq!(clock(Duration::from_secs(3_725)), "1:02:05");
}

#[test]
fn counted_uses_the_plural_unless_one() {
    assert_eq!(counted(1, "case"), "1 case");
    assert_eq!(counted(0, "case"), "0 cases");
    assert_eq!(counted(3, "component"), "3 components");
}
