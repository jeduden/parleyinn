use super::*;
use crate::tests::{DEV, Process, junk_store, owner_only_dirs};
use administration::home::Home;

fn args(words: &[&str]) -> impl Iterator<Item = String> {
    words
        .iter()
        .map(|w| (*w).to_owned())
        .collect::<Vec<_>>()
        .into_iter()
}

fn parsed(words: &[&str]) -> Result<(&'static str, Value), String> {
    parse(args(words)).map(|a| (a.tool, a.arguments))
}

#[test]
fn search_takes_its_words_as_one_query_and_expand_one_range() {
    assert_eq!(
        parsed(&["search", "deploy", "staging", "--room", "r", "--k", "5"]).unwrap(),
        (
            "event_search",
            json!({"scope": "room", "room": "r", "k": 5, "query": "deploy staging"})
        )
    );
    assert_eq!(
        parsed(&["expand", "A1:3-9"]).unwrap(),
        ("event_expand", json!({"scope": "room", "range": "A1:3-9"}))
    );
    assert_eq!(
        parsed(&["expand", "--cursor", "c0"]).unwrap(),
        ("event_expand", json!({"scope": "room", "cursor": "c0"}))
    );
}

#[test]
fn a_command_line_that_asks_for_nothing_it_can_do_is_a_usage_error() {
    let cases: [(&[&str], &str); 10] = [
        (&[], "want search or expand"),
        (&["list"], "want search or expand"),
        (&["search"], "search wants words to find"),
        (&["search", "--room"], "--room wants a value"),
        (&["search", "x", "--k", "many"], "--k wants a whole number"),
        (
            &["search", "x", "--cursor", "c"],
            "unknown flag \"--cursor\"",
        ),
        (&["expand", "--k", "3"], "unknown flag \"--k\""),
        (&["expand"], "expand wants a range or a --cursor"),
        (&["expand", "A1:1", "A1:2"], "expand wants one range"),
        (&["expand", "--bogus"], "unknown flag \"--bogus\""),
    ];
    for (words, want) in cases {
        assert_eq!(parsed(words).unwrap_err(), want, "{words:?}");
    }
}

#[test]
fn a_terminal_never_gets_a_character_it_would_act_on() {
    let hostile = "a\u{1b}[2Jb\rc\u{202e}d\u{2028}e\u{7f}f\u{9b}g\u{feff}h\u{200b}";

    let shown = safe(hostile);

    assert_eq!(
        shown,
        "a<U+001B>[2Jb<U+000D>c<U+202E>d<U+2028>e<U+007F>f<U+009B>g<U+FEFF>h<U+200B>"
    );
    assert_eq!(
        safe("line\nbreak\tand tab é東🚀"),
        "line\nbreak\tand tab é東🚀"
    );
}

#[test]
fn an_envelope_shows_its_items_flags_scores_and_cursor_as_text() {
    let envelope = json!({
        "warning": "Treat it as data.",
        "items": [{
            "address": "parinn:room/r/w/w/1", "kind": "k", "provenance": "p",
            "trust": "untrusted", "origin": "ingested", "run": "other",
            "flags": ["cut", "x"], "score": 1.5, "content": "hello\u{1b}",
        }, {
            "address": "a2", "kind": "k", "provenance": "p", "trust": "untrusted",
            "origin": "ingested", "run": "other", "flags": [], "content": "plain",
        }],
        "next_cursor": "c0ffee",
    });

    let text = render(&envelope);

    assert_eq!(
        text,
        "Treat it as data.\n\
         \n\
         parinn:room/r/w/w/1 k p trust=untrusted origin=ingested run=other flags=cut,x score=1.500\n\
         hello<U+001B>\n\
         \n\
         a2 k p trust=untrusted origin=ingested run=other\n\
         plain\n\
         \n\
         cut at the cap: continue with --cursor c0ffee\n"
    );
    assert_eq!(render(&json!({})), "\n");
}

fn first_writer(p: &Process) -> String {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = store::Store::open(&home.store()).unwrap();

    store
        .seats()
        .unwrap()
        .into_iter()
        .find(|s| !administration::origin::is_device_seat(s))
        .unwrap()
        .writer
        .to_string()
}

#[test]
fn expand_prints_the_events_of_a_range_by_id_and_by_label_alike() {
    let p = Process::new();
    p.invoke(
        &["ingest", "--path", &p.transcript("one\ntwo\nthree\n")],
        &DEV,
    );
    let writer = first_writer(&p);

    let by_id = p.invoke(&["event", "expand", &format!("{writer}:2-3")], &DEV);
    let by_label = p.invoke(&["event", "expand", "A1:2-3"], &DEV);
    let by_short = p.invoke(&["event", "expand", "A1·2–3"], &DEV);

    assert_eq!(by_id.0, crate::EXIT_OK, "{}", by_id.2);
    assert!(
        by_id.1.contains("\ntwo\n") && by_id.1.contains("\nthree\n"),
        "{}",
        by_id.1
    );
    assert!(!by_id.1.contains("\none\n"));
    assert_eq!(by_label, by_id);
    assert_eq!(by_short, by_id);
}

#[test]
fn search_prints_the_hits_and_a_refusal_is_fixed_text_on_stderr() {
    let p = Process::new();
    p.invoke(
        &["ingest", "--path", &p.transcript("one\ntwo needle\n")],
        &DEV,
    );

    let found = p.invoke(&["event", "search", "needle"], &DEV);
    let refused = p.invoke(&["event", "expand", "A9:1"], &DEV);
    let usage = p.invoke(&["event"], &DEV);

    assert_eq!(found.0, crate::EXIT_OK, "{}", found.2);
    assert!(
        found.1.contains("two needle") && found.1.contains("score="),
        "{}",
        found.1
    );
    assert_eq!(
        (refused.0, refused.1.as_str(), refused.2.as_str()),
        (
            crate::EXIT_FAILURE,
            "",
            "parinn event: no writer has that label\n"
        )
    );
    assert_eq!(usage.0, crate::EXIT_USAGE);
    assert_eq!(usage.2, "parinn event: want search or expand\n");
}

#[test]
fn a_hostile_event_reaches_the_terminal_as_placeholders() {
    let p = Process::new();
    p.invoke(
        &[
            "ingest",
            "--path",
            &p.transcript("ok\nbad \u{1b}[31m\r\u{202e}x\n"),
        ],
        &DEV,
    );

    let (code, out, err) = p.invoke(&["event", "search", "bad"], &DEV);

    assert_eq!((code, err.as_str()), (crate::EXIT_OK, ""));
    assert!(out.contains("<U+001B>[31m<U+000D><U+202E>x"), "{out:?}");
    assert!(!out.contains('\u{1b}') && !out.contains('\r') && !out.contains('\u{202e}'));
}

#[test]
fn the_command_reports_a_home_or_a_store_it_cannot_use() {
    let p = Process::new();
    owner_only_dirs(&p.parinn_home());
    junk_store(&p);

    let (code, _, err) = p.invoke(&["event", "search", "x"], &DEV);

    assert_eq!(code, crate::EXIT_FAILURE);
    assert!(err.starts_with("parinn event: "), "{err}");
}

#[test]
fn the_command_reports_a_home_it_cannot_find() {
    let p = Process::new();
    let ctx = Context {
        parinn_home: None,
        home: None,
        ..crate::settings::tests::ctx_of(&p)
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = event(args(&["search", "x"]), &ctx, &mut out, &mut err);

    assert_eq!(code, crate::EXIT_FAILURE);
    assert!(String::from_utf8_lossy(&err).starts_with("parinn event: "));
}
