use super::*;

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| (*s).to_owned()).collect()
}

fn tags(t: &[&str]) -> Vec<String> {
    args(t)
}

#[test]
fn parse_args_reads_tags_filters_and_ignores_libtest_flags() {
    let got = parse_args(&args(&[
        "--tags",
        "@ENG-24",
        "--nocapture",
        "--test-threads=2",
        "--color",
        "never",
        "title",
    ]))
    .unwrap();

    let expr = |e: &str| format!("{:?}", Some(e.parse::<TagOperation>().unwrap()));
    assert_eq!(format!("{:?}", got.tags), expr("@ENG-24"));
    assert_eq!(got.filters, ["title"]);
    let short = parse_args(&args(&["-t=@a"])).unwrap();
    assert_eq!(format!("{:?}", short.tags), expr("@a"));
    let none = parse_args(&[]).unwrap();
    assert!(none.tags.is_none() && none.filters.is_empty());
}

#[test]
fn parse_args_reads_skips_as_libtest_does() {
    let got = parse_args(&args(&["--skip", "disclosure", "--skip=@ENG-01"])).unwrap();

    assert_eq!(got.skips, ["disclosure", "@ENG-01"]);
    assert!(got.filters.is_empty());
    assert_eq!(
        parse_args(&args(&["--skip"])).unwrap_err(),
        "--skip needs a value"
    );
}

#[test]
fn selects_leaves_out_what_a_skip_names() {
    let skipping = parse_args(&args(&["--skip", "disclosure", "--skip", "@ENG-01"])).unwrap();
    let bound = tags(&["ENG-24", "P0"]);

    assert!(!skipping.selects("a 90-day disclosure policy", bound.iter()));
    assert!(!skipping.selects("x", tags(&["ENG-01"]).iter()));
    assert!(skipping.selects("x", bound.iter()));
}

#[test]
fn parse_args_refuses_what_it_does_not_know() {
    assert_eq!(
        parse_args(&args(&["--bogus"])).unwrap_err(),
        r#"unknown flag "--bogus""#
    );
    assert_eq!(
        parse_args(&args(&["--tags"])).unwrap_err(),
        "--tags needs a value"
    );
    assert_eq!(
        parse_args(&args(&["--test-threads"])).unwrap_err(),
        "--test-threads needs a value"
    );
    assert!(
        parse_args(&args(&["--tags", "@a and"]))
            .unwrap_err()
            .starts_with(r#"--tags "@a and": "#)
    );
}

#[test]
fn selects_skips_pending_and_applies_tags_and_filters() {
    let all = Options::default();
    let eng24 = parse_args(&args(&["--tags", "@ENG-24"])).unwrap();
    let named = parse_args(&args(&["disclosure"])).unwrap();
    let by_tag = parse_args(&args(&["@ENG-24"])).unwrap();
    let bound = tags(&["ENG-24", "P0"]);
    let pending = tags(&["ENG-25", "P1", "pending"]);
    let title = "SECURITY.md names a private channel and a 90-day disclosure policy";

    assert!(all.selects(title, bound.iter()));
    assert!(!all.selects("x", pending.iter()));
    assert!(eng24.selects(title, bound.iter()));
    assert!(!eng24.selects("x", tags(&["ENG-01"]).iter()));
    assert!(!eng24.selects("x", tags(&["ENG-24", "pending"]).iter()));
    assert!(named.selects(title, bound.iter()));
    assert!(!named.selects("another", bound.iter()));
    assert!(by_tag.selects("another", bound.iter()));
}

/// A future that is pending until polled `left` more times, waking
/// itself each time: by reference on this thread, or by value from
/// another thread.
struct Countdown {
    left: u8,
}

impl Future for Countdown {
    type Output = &'static str;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.left {
            0 => Poll::Ready("done"),
            n => {
                self.left -= 1;
                if n % 2 == 0 {
                    cx.waker().wake_by_ref();
                } else {
                    let waker = cx.waker().clone();
                    thread::spawn(move || waker.wake());
                }
                Poll::Pending
            }
        }
    }
}

#[test]
fn block_on_drives_a_future_through_its_waits() {
    assert_eq!(block_on(Countdown { left: 3 }), "done");
    assert_eq!(block_on(async { 7 }), 7);
}

#[test]
fn is_isolated_wants_the_marked_home_under_the_temporary_directory() {
    let temp = Path::new("/tmp");
    let os = |s: &'static str| Some(OsStr::new(s));

    assert!(is_isolated(os("/tmp/parinn-1"), os("/tmp/parinn-1"), temp));
    assert!(
        !is_isolated(os("/home/me"), os("/home/me"), temp),
        "an exported marker cannot isolate a real home"
    );
    assert!(!is_isolated(os("/tmp/parinn-1"), os("/tmp/parinn-2"), temp));
    assert!(!is_isolated(os("/tmp/parinn-1"), None, temp));
    assert!(!is_isolated(None, os("/tmp/parinn-1"), temp));
}

#[test]
fn isolate_points_home_parinn_home_and_the_marker_at_the_home() {
    let mut command = Command::new("x");

    isolate(&mut command, Path::new("/tmp/h"));

    let envs: Vec<(String, String)> = command
        .get_envs()
        .map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.unwrap().to_string_lossy().into_owned(),
            )
        })
        .collect();
    assert_eq!(
        envs,
        [
            ("HOME".to_owned(), "/tmp/h".to_owned()),
            ("PARINN_BDD_HOME".to_owned(), "/tmp/h".to_owned()),
            ("PARINN_HOME".to_owned(), "/tmp/h/.parinn".to_owned()),
        ]
    );
}

#[test]
fn exit_code_passes_a_byte_and_fails_the_rest() {
    assert_eq!(exit_code(Some(0)), 0);
    assert_eq!(exit_code(Some(2)), 2);
    assert_eq!(exit_code(Some(300)), 1);
    assert_eq!(exit_code(None), 1);
}

#[test]
fn sections_hold_one_value_per_type() {
    #[derive(Default)]
    struct Store(u8);
    let mut sections = Sections::default();

    sections.get::<Store>().0 = 7;

    assert_eq!(sections.get::<Store>().0, 7);
    assert_eq!(*sections.get::<u32>(), 0);
    assert_eq!(format!("{sections:?}"), "Sections(2)");
    assert_eq!(
        Sections::default().get::<Store>().0,
        0,
        "another scenario, another value"
    );
}
