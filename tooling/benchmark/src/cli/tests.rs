use super::*;
use std::cell::RefCell;

/// `n` samples of `name` near `ns`, as a samples file.
fn file(name: &str, ns: u64, n: u64) -> String {
    let taken: Vec<u64> = (0..n).map(|i| ns + i % 7).collect();
    format!("{}\n{}", samples::HEADER, samples::render(name, &taken))
}

/// Runs the command over `files` with a timer that answers 1 µs, and
/// returns its code, stdout and stderr.
fn invoke(args: &[&str], files: &BTreeMap<&str, String>) -> (u8, String, String) {
    let time = |_: &Path, _: &suite::Benchmark| Ok(Duration::from_micros(1));
    invoke_timed(args, files, &time)
}

fn invoke_timed(
    args: &[&str],
    files: &BTreeMap<&str, String>,
    time: Time<'_>,
) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let read = |path: &str| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    };
    let code = run(
        args.iter().map(|a| (*a).to_owned()),
        &mut out,
        &mut err,
        &read,
        time,
    );
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

fn gate_files(base_ns: u64, head_ns: u64) -> BTreeMap<&'static str, String> {
    BTreeMap::from([
        ("base", file("core_start_up", base_ns, 30)),
        ("head", file("core_start_up", head_ns, 30)),
    ])
}

#[test]
fn run_writes_every_benchmark_s_samples_against_the_core() {
    let seen = RefCell::new(Vec::new());
    let time = |core: &Path, b: &suite::Benchmark| {
        seen.borrow_mut()
            .push(format!("{} {}", core.display(), b.args.join(" ")));
        Ok(Duration::from_nanos(812))
    };

    let (code, out, err) = invoke_timed(
        &["run", "--core", "target/release/parinn", "--samples=2"],
        &BTreeMap::new(),
        &time,
    );

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert_eq!(
        out,
        "# benchmark nanoseconds\ncore_start_up 812\ncore_start_up 812\n\
         ingest_transcript 812\ningest_transcript 812\n\
         hook_session_start 812\nhook_session_start 812\n\
         hook_user_prompt_submit 812\nhook_user_prompt_submit 812\n\
         search_events 812\nsearch_events 812\n"
    );
    // One warm-up and the two samples, of each benchmark.
    assert_eq!(seen.borrow().len(), 15);
    assert_eq!(seen.borrow()[0], "target/release/parinn version");
    assert_eq!(seen.borrow()[3], "target/release/parinn ingest --all");
    assert_eq!(seen.borrow()[6], "target/release/parinn hook SessionStart");
    assert_eq!(seen.borrow()[12], "target/release/parinn mcp");
}

#[test]
fn run_takes_ten_samples_unless_told() {
    let (code, out, _) = invoke(&["run", "--core", "parinn"], &BTreeMap::new());

    assert_eq!(code, EXIT_OK);
    assert_eq!(
        samples::parse(&out).unwrap()["core_start_up"].len(),
        DEFAULT_SAMPLES
    );
}

#[test]
fn run_fails_naming_the_benchmark_it_could_not_sample() {
    let time = |_: &Path, _: &suite::Benchmark| Err("parinn version exited 3".to_owned());

    let (code, out, err) = invoke_timed(&["run", "--core", "parinn"], &BTreeMap::new(), &time);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(out, "");
    assert_eq!(err, "benchmark: core_start_up: parinn version exited 3\n");
}

#[test]
fn run_refuses_a_sample_count_that_is_not_positive() {
    for n in ["0", "-1", "ten"] {
        let (code, _, err) = invoke(
            &["run", "--core", "parinn", "--samples", n],
            &BTreeMap::new(),
        );

        assert_eq!(code, EXIT_USAGE);
        assert!(
            err.starts_with(&format!(
                "benchmark: --samples {n:?} is not a positive whole number\nusage:"
            )),
            "{err}"
        );
    }
}

#[test]
fn gate_passes_and_reports_when_nothing_regressed() {
    let (code, out, err) = invoke(
        &["gate", "--base", "base", "--head", "head"],
        &gate_files(1000, 1050),
    );

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.starts_with("| Benchmark |"), "{out}");
    assert!(out.contains("| core_start_up |"), "{out}");
    assert!(out.ends_with("| within the threshold |\n"), "{out}");
}

#[test]
fn gate_fails_naming_the_benchmark_that_regressed() {
    let (code, out, err) = invoke(
        &["gate", "--base", "base", "--head", "head"],
        &gate_files(1000, 1200),
    );

    assert_eq!(code, EXIT_FAILURE);
    assert!(out.ends_with("| regressed |\n"), "{out}");
    assert!(
        err.starts_with("benchmark: core_start_up regressed by "),
        "{err}"
    );
}

#[test]
fn gate_fails_naming_a_benchmark_missing_on_the_head() {
    let files = BTreeMap::from([
        ("base", file("core_start_up", 1000, 30)),
        ("head", samples::HEADER.to_owned()),
    ]);

    let (code, _, err) = invoke(&["gate", "--base", "base", "--head", "head"], &files);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        err,
        "benchmark: core_start_up was sampled on the base but not on the head\n"
    );
}

#[test]
fn gate_names_a_file_it_cannot_read_or_parse() {
    let mut files = gate_files(1000, 1000);
    files.insert("head", "core_start_up soon\n".to_owned());

    let (code, out, err) = invoke(&["gate", "--base", "base", "--head", "head"], &files);
    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert_eq!(
        err,
        "benchmark: head: line 1: \"core_start_up soon\" is not a benchmark name and a sample in nanoseconds\n"
    );

    let (code, _, err) = invoke(&["gate", "--base", "nowhere", "--head", "head"], &files);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("benchmark: read nowhere: "), "{err}");
}

#[test]
fn the_command_line_is_checked() {
    for (args, want) in [
        (&[][..], r#"unknown command """#),
        (&["bench"][..], r#"unknown command "bench""#),
        (&["run"][..], "--core is required"),
        (&["run", "parinn"][..], r#"unexpected argument "parinn""#),
        (&["run", "--core"][..], "--core needs a value"),
        (&["run", "--core="][..], "--core needs a value"),
        (&["run", "--base", "x"][..], r#"unknown flag "--base""#),
        (&["gate", "--base", "b"][..], "--head is required"),
        (&["gate", "--head", "h"][..], "--base is required"),
    ] {
        let (code, out, err) = invoke(args, &BTreeMap::new());

        assert_eq!((code, out.as_str()), (EXIT_USAGE, ""), "{args:?}");
        assert_eq!(err, format!("benchmark: {want}\n{USAGE}\n"), "{args:?}");
    }
}

/// A writer that refuses every write.
struct Refusing;

impl Write for Refusing {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("disk full"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_refused_write_fails_the_command() {
    let files = gate_files(1000, 1000);
    let read = |path: &str| Ok(files[path].clone());
    let time = |_: &Path, _: &suite::Benchmark| Ok(Duration::from_micros(1));

    for (args, want) in [
        (
            ["run", "--core", "parinn", "", ""],
            "write the samples: disk full",
        ),
        (
            ["gate", "--base", "base", "--head", "head"],
            "write the report: disk full",
        ),
    ] {
        let args = args
            .into_iter()
            .filter(|a| !a.is_empty())
            .map(str::to_owned);
        let mut err = Vec::new();

        let code = run(args, &mut Refusing, &mut err, &read, &time);

        assert_eq!(code, EXIT_FAILURE);
        assert_eq!(
            String::from_utf8(err).unwrap(),
            format!("benchmark: {want}\n")
        );
    }
}

#[test]
fn refusing_flushes_nothing() {
    assert!(Refusing.flush().is_ok());
}

#[test]
fn gate_accepts_a_regression_a_fresh_acceptance_covers() {
    let mut files = gate_files(1000, 1200);
    files.insert(
        "accepted",
        "core_start_up 30 the core links the store\n".to_owned(),
    );

    let (code, out, err) = invoke(
        &[
            "gate",
            "--base",
            "base",
            "--head",
            "head",
            "--accepted",
            "accepted",
            "--accepted-on-base",
            "none",
        ],
        &files,
    );

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.ends_with("| accepted |\n"), "{out}");
}

#[test]
fn gate_ignores_an_acceptance_the_base_already_carries() {
    let mut files = gate_files(1000, 1200);
    let entry = "core_start_up 30 the core links the store\n".to_owned();
    files.insert("accepted", entry.clone());
    files.insert("accepted-base", entry);

    let (code, _, err) = invoke(
        &[
            "gate",
            "--base",
            "base",
            "--head",
            "head",
            "--accepted",
            "accepted",
            "--accepted-on-base",
            "accepted-base",
        ],
        &files,
    );

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("core_start_up regressed by "), "{err}");
}

#[test]
fn gate_refuses_an_acceptance_file_it_cannot_read_or_parse() {
    let mut files = gate_files(1000, 1000);
    files.insert("bad", "core_start_up 5 too small\n".to_owned());

    let (missing, _, err_missing) = invoke(
        &[
            "gate",
            "--base",
            "base",
            "--head",
            "head",
            "--accepted",
            "none",
        ],
        &files,
    );
    let (bad, _, err_bad) = invoke(
        &[
            "gate",
            "--base",
            "base",
            "--head",
            "head",
            "--accepted",
            "bad",
        ],
        &files,
    );
    let (bad_base, _, err_bad_base) = invoke(
        &[
            "gate",
            "--base",
            "base",
            "--head",
            "head",
            "--accepted-on-base",
            "bad",
        ],
        &files,
    );

    assert_eq!(missing, EXIT_FAILURE);
    assert!(
        err_missing.starts_with("benchmark: read none: "),
        "{err_missing}"
    );
    assert_eq!(bad, EXIT_FAILURE);
    assert!(
        err_bad.starts_with("benchmark: bad: line 1: 5% is no more"),
        "{err_bad}"
    );
    assert_eq!(bad_base, EXIT_FAILURE);
    assert!(
        err_bad_base.starts_with("benchmark: bad: line 1:"),
        "{err_bad_base}"
    );
}
