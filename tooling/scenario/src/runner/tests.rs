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

fn shard(index: u64, total: u64) -> Shard {
    Shard { index, total }
}

#[test]
fn parse_args_reads_a_shard() {
    let got = parse_args(&args(&["--shard", "2/4"])).unwrap();
    let inline = parse_args(&args(&["--shard=4/4", "x"])).unwrap();

    assert_eq!(got.shard, Some(shard(2, 4)));
    assert_eq!(inline.shard, Some(shard(4, 4)));
    assert_eq!(inline.filters, ["x"]);
    assert_eq!(parse_args(&[]).unwrap().shard, None);
    assert_eq!(
        parse_args(&args(&["--shard"])).unwrap_err(),
        "--shard needs a value"
    );
}

#[test]
fn parse_args_refuses_a_shard_that_is_not_k_of_n() {
    for bad in [
        "", "2", "0/4", "5/4", "1/0", "-1/4", "+1/4", "a/4", "1/b", "1/2/3", "1/",
    ] {
        let err = parse_args(&args(&["--shard", bad])).unwrap_err();
        assert_eq!(
            err,
            format!("--shard: want K/N with 1 <= K <= N, not {bad:?}")
        );
    }
}

#[test]
fn options_take_the_shard_from_the_environment_unless_the_flag_names_one() {
    let env = |s: &'static str| Some(OsStr::new(s));

    assert_eq!(options(&[], env("3/4")).unwrap().shard, Some(shard(3, 4)));
    assert_eq!(options(&[], None).unwrap().shard, None);
    assert_eq!(
        options(&args(&["--shard", "1/2"]), env("3/4"))
            .unwrap()
            .shard,
        Some(shard(1, 2)),
        "the flag wins"
    );
    assert_eq!(
        options(&[], env("5/4")).unwrap_err(),
        format!(r#"{SHARD_ENV}: want K/N with 1 <= K <= N, not "5/4""#)
    );
    assert_eq!(
        options(&args(&["--bogus"]), None).unwrap_err(),
        r#"unknown flag "--bogus""#
    );
}

#[cfg(unix)]
#[test]
fn options_refuse_a_shard_variable_that_is_not_text() {
    use std::os::unix::ffi::OsStrExt as _;

    let err = options(&[], Some(OsStr::from_bytes(b"\xff/2"))).unwrap_err();

    assert_eq!(err, format!("{SHARD_ENV}: not unicode"));
}

#[test]
fn fnv1a_is_the_published_hash() {
    // The reference vectors of the FNV-1a 64-bit hash.
    assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(fnv1a(b"foobar"), 0x8594_4171_f739_67e8);
}

#[test]
fn shard_key_joins_path_name_and_row() {
    let path = Path::new("administration.feature");

    assert_eq!(
        shard_key(path, "a name", None),
        "administration.feature\0a name"
    );
    assert_eq!(
        shard_key(path, "a name", Some(7)),
        "administration.feature\0a name\x007"
    );
    assert_eq!(
        shard_key(Path::new("sub/x.feature"), "n", None),
        "sub/x.feature\0n"
    );
}

#[test]
fn an_unsharded_run_holds_every_scenario() {
    assert!(Options::default().in_shard("anything"));
}

/// A suite of scenarios across files, some outlines with many rows.
fn suite() -> Vec<String> {
    let mut keys = Vec::new();
    for file in 0..12 {
        let path = std::path::PathBuf::from(format!("section{file}.feature"));
        for scenario in 0..9 {
            let name = format!("scenario {scenario}");
            keys.push(shard_key(&path, &name, None));
            keys.push(shard_key(&path, &name, Some(10 + scenario)));
            keys.push(shard_key(&path, &name, Some(20 + scenario)));
        }
    }

    keys
}

#[test]
fn the_shards_partition_the_suite_in_any_order() {
    let keys = suite();
    let mut reversed = keys.clone();
    reversed.reverse();
    let mut want: Vec<&String> = keys.iter().collect();
    want.sort();

    for total in 1..=8 {
        for order in [&keys, &reversed] {
            let mut held: Vec<&String> = Vec::new();
            for index in 1..=total {
                let options = parse_args(&args(&["--shard", &format!("{index}/{total}")])).unwrap();
                held.extend(order.iter().filter(|k| options.in_shard(k)));
            }
            held.sort();
            assert_eq!(
                held, want,
                "{total} shards hold every scenario exactly once"
            );
        }
    }
}

#[test]
fn the_shards_each_hold_a_fair_share() {
    let keys = suite();
    for total in 2..=4 {
        for index in 1..=total {
            let mine = keys.iter().filter(|k| shard(index, total).holds(k)).count();
            let share = keys.len() / usize::try_from(total).unwrap();
            assert!(
                mine * 2 > share && mine < share * 2,
                "shard {index}/{total} holds {mine} of {}",
                keys.len()
            );
        }
    }
}

#[test]
fn a_shard_composes_with_tags_filters_and_skips() {
    let key = "k";
    let index = if shard(1, 2).holds(key) { 1 } else { 2 };
    let bound = tags(&["ENG-24"]);
    let run = |extra: &[&str], index: u64| {
        let mut all = vec!["--shard".to_owned(), format!("{index}/2")];
        all.extend(args(extra));
        let options = parse_args(&all).unwrap();
        options.selects("name", bound.iter()) && options.in_shard(key)
    };

    assert!(run(&["--tags", "@ENG-24"], index));
    assert!(!run(&["--tags", "@ENG-24"], 3 - index));
    assert!(!run(&["--tags", "@ENG-01"], index));
    assert!(!run(&["--skip", "name"], index));
    assert!(run(&["name"], index));
}
