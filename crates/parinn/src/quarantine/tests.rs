use administration::home::Home;
use record::id::{RoomId, SeatId, WriterId};
use serde_json::Value;
use store::Store;

use super::*;
use crate::tests::{DEV, Process, audited};
use crate::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};

const LINES: &str = "one\ntwo\nthree\n";

/// A process whose node ingested `LINES`, and the run's writer.
fn with_a_run() -> (Process, WriterId) {
    let p = Process::new();
    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript(LINES)], &DEV);
    assert_eq!(code, EXIT_OK, "{err}");
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let writer = Store::open(&home.store()).unwrap().ingested_runs().unwrap()[0].writer;

    (p, writer)
}

fn parsed(args: &[&str]) -> Result<Asked, String> {
    parse(args.iter().map(|a| (*a).to_owned()))
}

#[test]
fn the_command_line_names_a_verb_one_selector_and_a_reason() {
    let writer = WriterId::derive(&[1; 32]).to_string();
    let seat = SeatId::derive(&RoomId::derive(&[1; 32], &[2; 16]), &[1; 32]).to_string();
    let room = RoomId::derive(&[1; 32], &[2; 16]).to_string();

    assert_eq!(
        parsed(&["add", "--run", "r-1", "--reason", "suspect content"]),
        Ok(Asked::Add {
            selector: Wanted::Done(Selector::run("r-1").unwrap()),
            reason: "suspect content".to_owned()
        })
    );
    assert_eq!(
        parsed(&["add", "--range", "A1:100-200"]),
        Ok(Asked::Add {
            selector: Wanted::Range("A1:100-200".to_owned()),
            reason: String::new()
        })
    );
    assert_eq!(
        parsed(&["release", "--writer", "A2"]),
        Ok(Asked::Release {
            selector: Wanted::Writer("A2".to_owned())
        })
    );
    assert_eq!(
        parsed(&["list", "--json"]),
        Ok(Asked::List { as_json: true })
    );
    assert_eq!(parsed(&["list"]), Ok(Asked::List { as_json: false }));
    for (flag, value) in [
        ("--author", seat.as_str()),
        ("--room", room.as_str()),
        ("--provenance", "web"),
        ("--flag", "instruction_like"),
        ("--since", "2026-01-01T00:00:00Z"),
    ] {
        assert!(
            matches!(
                parsed(&["add", flag, value]),
                Ok(Asked::Add {
                    selector: Wanted::Done(_),
                    ..
                })
            ),
            "{flag}"
        );
    }
    let _ = writer;
}

#[test]
fn a_command_line_that_names_no_verb_or_a_wrong_selector_is_a_usage_error() {
    for (args, why) in [
        (&[][..], "want add, release or list"),
        (&["nothing"], "want add, release or list"),
        (
            &["show", "--run", "r"],
            "show, the forensic view, waits for the room view (VIEW-11)",
        ),
        (&["add"], "add wants a selector, such as --run ID"),
        (&["release"], "release wants a selector, such as --run ID"),
        (&["add", "--run", "r", "--room", "x"], "want one selector"),
        (&["add", "--run"], "--run wants a value"),
        (
            &["add", "--run", "two words"],
            "a run name is letters, digits and - _ . / :",
        ),
        (&["add", "--author", "x"], "--author wants a seat id"),
        (&["add", "--room", "x"], "--room wants a room id"),
        (&["add", "--provenance", "x"], "not a provenance class"),
        (
            &["add", "--flag", "X"],
            "a flag is lowercase letters and underscores",
        ),
        (
            &["add", "--since", "x"],
            "a time is RFC 3339, such as 2026-01-01T00:00:00Z",
        ),
        (
            &["add", "--run", "r", "--bogus"],
            "unknown flag \"--bogus\"",
        ),
        (&["add", "--run", "r", "stray"], "unexpected \"stray\""),
        (
            &["release", "--run", "r", "--reason", "x"],
            "release states no reason",
        ),
        (&["list", "--run", "r"], "list takes no selector"),
        (&["list", "--reason", "x"], "list takes no selector"),
    ] {
        assert_eq!(parsed(args), Err(why.to_owned()), "{args:?}");
    }
    assert_eq!(
        Process::new().invoke(&["quarantine", "add"], &DEV),
        (
            EXIT_USAGE,
            String::new(),
            "parinn quarantine: add wants a selector, such as --run ID\n".into()
        )
    );
}

#[test]
fn a_quarantine_is_added_as_a_cut_act_that_states_its_selector_and_reason() {
    let (p, writer) = with_a_run();

    let (code, out, err) = p.invoke(
        &[
            "quarantine",
            "add",
            "--range",
            "A1:2-2",
            "--reason",
            "suspect\u{1b}[31m content",
        ],
        &DEV,
    );

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(
        out.starts_with(&format!("quarantined range {writer}:2-2; act ")),
        "{out}"
    );
    let acts = crate::tests::recorded(&p);
    let added = acts.iter().find(|a| a["act"] == "quarantine_add").unwrap();
    assert_eq!(added["class"], "cut");
    assert_eq!(added["selector"]["writer"], writer.to_string());
    assert_eq!(added["reason"], "suspect\u{1b}[31m content");
}

#[test]
fn the_list_shows_the_quarantines_in_force_safely_and_in_json() {
    let (p, writer) = with_a_run();
    p.invoke(
        &[
            "quarantine",
            "add",
            "--range",
            "A1:2-2",
            "--reason",
            "suspect\u{1b}[31m content",
        ],
        &DEV,
    );

    let (code, listed, _) = p.invoke(&["quarantine", "list"], &DEV);

    assert_eq!(code, EXIT_OK);
    assert!(
        listed.starts_with(&format!("range {writer}:2-2; act ")),
        "{listed}"
    );
    assert!(
        listed
            .trim_end()
            .ends_with("reason: suspect<U+001B>[31m content"),
        "{listed}"
    );
    let (_, json, _) = p.invoke(&["quarantine", "list", "--json"], &DEV);
    let got: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(got["quarantines"][0]["reason"], "suspect\u{1b}[31m content");
}

#[test]
fn a_release_is_a_widening_act_and_the_list_is_empty_after_it() {
    let (p, writer) = with_a_run();
    p.invoke(&["quarantine", "add", "--range", "A1:2-2"], &DEV);
    assert_eq!(p.invoke(&["risk-acceptance", "record"], &DEV).0, EXIT_OK);

    let (code, out, _) = p.invoke(&["quarantine", "release", "--range", "A1:2-2"], &DEV);

    assert_eq!(code, EXIT_OK);
    assert!(
        out.starts_with(&format!("released range {writer}:2-2; act ")),
        "{out}"
    );
    let released = crate::tests::recorded(&p)
        .into_iter()
        .find(|a| a["act"] == "quarantine_release")
        .unwrap();
    assert_eq!(released["class"], "widening");
    let (_, nothing, _) = p.invoke(&["quarantine", "list"], &DEV);
    assert_eq!(nothing, "nothing is quarantined\n");
    let notes: Vec<String> = audited(&p)
        .into_iter()
        .filter(|e| e.kind == "principal_act")
        .map(|e| e.detail)
        .collect();
    assert!(notes.ends_with(&[
        "quarantine added".to_owned(),
        "quarantine released".to_owned()
    ]));
}

#[test]
fn a_writer_is_selected_by_its_label_or_id() {
    let (p, writer) = with_a_run();

    for text in ["A1", &writer.to_string()] {
        let (code, out, err) = p.invoke(&["quarantine", "add", "--writer", text], &DEV);

        assert_eq!((code, err.as_str()), (EXIT_OK, ""), "{text}");
        assert!(
            out.starts_with(&format!("quarantined writer {writer}; act ")),
            "{out}"
        );
    }
    let (code, _, err) = p.invoke(&["quarantine", "add", "--writer", "A99"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("no writer has that label"), "{err}");
}

#[test]
fn a_release_of_a_selector_no_quarantine_holds_is_refused() {
    let (p, _) = with_a_run();

    let (code, out, err) = p.invoke(&["quarantine", "release", "--run", "nobody"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert_eq!(
        err,
        "parinn quarantine release: no quarantine holds that selector\n"
    );
}

#[test]
fn a_cut_quarantine_needs_a_terminal_alone_and_its_release_the_risk_acceptance_too() {
    let (mut p, _) = with_a_run();
    p.terminal = false;
    let (code, _, err) = p.invoke(&["quarantine", "add", "--run", "r-1"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("refused: a principal act needs a terminal"),
        "{err}"
    );
    p.terminal = true;

    let (code, _, err) = p.invoke(&["quarantine", "add", "--run", "r-1"], &DEV);
    assert_eq!(
        (code, err.as_str()),
        (EXIT_OK, ""),
        "a cut act needs no risk acceptance"
    );
    let (code, _, err) = p.invoke(&["quarantine", "release", "--run", "r-1"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("residual risks are open"), "{err}");
    let (code, _, _) = p.invoke(&["risk-acceptance", "record"], &DEV);
    assert_eq!(code, EXIT_OK);
    let (code, _, err) = p.invoke(&["quarantine", "release", "--run", "r-1"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
}

#[test]
fn a_list_in_a_home_that_never_started_starts_it_and_finds_nothing() {
    let p = Process::new();

    assert_eq!(
        p.invoke(&["quarantine", "list"], &DEV),
        (EXIT_OK, "nothing is quarantined\n".into(), String::new())
    );
}
