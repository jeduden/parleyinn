use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use testkit::TempDir;

use super::*;
use crate::host::Output;
use crate::host::fake::FakeHost;

fn component(kind: Kind, name: &str) -> Component {
    Component {
        name: name.to_owned(),
        kind,
        file: String::new(),
        text: String::new(),
        model: if kind == Kind::Orchestrator {
            "claude-sonnet-5-5".to_owned()
        } else {
            "haiku".to_owned()
        },
        effort: "low".to_owned(),
        max_turns: 20,
        tools: Vec::new(),
        strict_lines: true,
    }
}

fn world(class: &str, consults: usize) -> World {
    World {
        main: PathBuf::from("/work/main"),
        class: class.to_owned(),
        touched: 1,
        consults,
        dispatched: BTreeSet::new(),
        seeded: vec!["review/reports/lang-rust.md".to_owned()],
        notes: Vec::new(),
    }
}

#[test]
fn each_kind_gets_its_prompt() {
    let w = world("standard", 2);

    let specialist = prompt(&component(Kind::Specialist, "lang-rust"), &w);
    assert!(
        specialist.contains("You are `lang-rust`, a specialist"),
        "{specialist}"
    );
    assert!(
        specialist.contains("`review/diffs/lang-rust.diff`"),
        "{specialist}"
    );
    assert!(
        specialist.contains("`review/reports/lang-rust.md`"),
        "{specialist}"
    );

    let generalist = prompt(&component(Kind::Generalist, "review-code"), &w);
    assert!(
        generalist.contains("You are `review-code`, a generalist"),
        "{generalist}"
    );
    assert!(
        generalist
            .contains("The review class is standard; `review/bundle.md` lists the 1 touched ids."),
        "{generalist}"
    );
    assert!(
        generalist.contains("reports are `review/reports/lang-rust.md`."),
        "{generalist}"
    );

    let mut bare = world("light", 0);
    bare.touched = 0;
    bare.seeded.clear();
    let generalist = prompt(&component(Kind::Generalist, "review-code"), &bare);
    assert!(
        generalist.contains("lists the 0 touched ids."),
        "{generalist}"
    );
    assert!(generalist.contains("reports are none."), "{generalist}");

    assert_eq!(
        prompt(&component(Kind::Orchestrator, "review"), &w),
        ORCHESTRATOR_PROMPT
    );
}

fn args_of(c: &Command) -> Vec<&str> {
    c.args.iter().map(String::as_str).collect()
}

fn has(args: &[&str], pair: [&str; 2]) -> bool {
    args.windows(2).any(|w| w == pair)
}

#[test]
fn a_specialist_runs_as_the_session_s_agent_with_the_review_s_tools() {
    let w = world("standard", 2);
    let log = PathBuf::from("/logs/run.jsonl");

    let c = command(
        &component(Kind::Specialist, "lang-rust"),
        &w,
        None,
        &log,
        Duration::from_secs(9),
    );

    assert_eq!(c.program, "claude");
    assert_eq!(c.cwd, PathBuf::from("/work/main"));
    assert_eq!(c.stdout_to, Some(log));
    assert_eq!(c.deadline, Some(Duration::from_secs(9)));
    let args = args_of(&c);
    assert_eq!(args[0], "-p");
    assert!(args[1].starts_with("You are `lang-rust`"));
    assert!(has(&args, ["--max-turns", "20"]));
    assert!(has(&args, ["--allowedTools", SPECIALIST_TOOLS]));
    assert!(has(&args, ["--disallowedTools", SPECIALIST_DISALLOWED]));
    assert!(has(&args, ["--agent", "lang-rust"]));
    assert!(!args.contains(&"--model"));
    assert!(!args.contains(&"--json-schema"));
}

#[test]
fn every_run_logs_stream_json_without_a_session_from_the_project_s_settings() {
    let w = world("standard", 2);
    let log = PathBuf::from("/logs/run.jsonl");

    let c = command(
        &component(Kind::Specialist, "lang-rust"),
        &w,
        None,
        &log,
        Duration::from_secs(9),
    );

    let args = args_of(&c);
    assert!(has(&args, ["--output-format", "stream-json"]));
    assert!(args.contains(&"--verbose"));
    assert!(args.contains(&"--no-session-persistence"));
    assert!(has(&args, ["--setting-sources", "project"]));
    assert!(has(&args, ["--add-dir", "pr-head"]));
}

#[test]
fn a_model_override_and_a_generalist_s_tools_are_passed() {
    let w = world("standard", 2);
    let log = PathBuf::from("/logs/run.jsonl");
    let mut generalist = component(Kind::Generalist, "review-code");
    generalist.max_turns = 40;

    let c = command(
        &generalist,
        &w,
        Some("claude-opus-5-5"),
        &log,
        Duration::from_secs(9),
    );

    let args = args_of(&c);
    assert!(has(&args, ["--agent", "review-code"]));
    assert!(has(&args, ["--model", "claude-opus-5-5"]));
    assert!(has(&args, ["--max-turns", "40"]));
    assert!(has(&args, ["--allowedTools", GENERALIST_TOOLS]));
    assert!(has(&args, ["--disallowedTools", GENERALIST_DISALLOWED]));
}

#[test]
fn the_orchestrator_runs_as_the_workflow_runs_it() {
    let w = world("deep", 4);
    let log = PathBuf::from("/logs/run.jsonl");

    let c = command(
        &component(Kind::Orchestrator, "review"),
        &w,
        None,
        &log,
        Duration::from_secs(9),
    );

    let args = args_of(&c);
    assert_eq!(args[1], ORCHESTRATOR_PROMPT);
    assert!(has(&args, ["--max-turns", "34"]));
    assert!(has(&args, ["--allowedTools", ORCHESTRATOR_TOOLS]));
    assert!(has(&args, ["--disallowedTools", ORCHESTRATOR_DISALLOWED]));
    assert!(has(&args, ["--model", "claude-sonnet-5-5"]));
    assert!(has(&args, ["--effort", "low"]));
    assert!(has(&args, ["--json-schema", OUTCOME_SCHEMA]));
    assert!(!args.contains(&"--agent"));

    let c = command(
        &component(Kind::Orchestrator, "review"),
        &w,
        Some("claude-opus-5-5"),
        &log,
        Duration::from_secs(9),
    );
    assert!(has(&args_of(&c), ["--model", "claude-opus-5-5"]));
}

#[test]
fn the_class_sets_the_orchestrator_s_turns() {
    assert_eq!(class_turns("light"), 12);
    assert_eq!(class_turns("standard"), 20);
    assert_eq!(class_turns("deep"), 30);
    assert_eq!(class_turns("other"), 20);
}

#[test]
fn run_reads_the_log_the_command_wrote() {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("run.jsonl");
    let mut host = FakeHost::new(dir.path());
    host.claude = Box::new(|c: &Command| {
        fs::write(
            c.stdout_to.as_ref().unwrap(),
            "{\"type\":\"result\",\"subtype\":\"success\",\"result\":\"outcome: sound\",\"num_turns\":2}\n",
        )
        .unwrap();
        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    });
    let c = Command::new("claude", &["-p", "x"], dir.path()).logging_to(&log);

    let got = run(&host, &c).unwrap();

    assert!(got.answered);
    assert_eq!(got.answer, "outcome: sound");
    assert_eq!(got.turns, 2);
}

#[test]
fn run_marks_a_run_the_deadline_killed() {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("run.jsonl");
    let mut host = FakeHost::new(dir.path());
    host.claude = Box::new(|c: &Command| {
        fs::write(c.stdout_to.as_ref().unwrap(), "{\"type\":\"system\"}\n").unwrap();
        Ok(Output {
            code: None,
            timed_out: true,
            ..Output::default()
        })
    });
    let c = Command::new("claude", &[], dir.path()).logging_to(&log);

    let got = run(&host, &c).unwrap();

    assert!(!got.answered);
    assert_eq!(got.subtype, "deadline");
}

#[test]
fn run_names_what_it_cannot_do() {
    let dir = TempDir::new().unwrap();
    let mut host = FakeHost::new(dir.path());
    host.claude = Box::new(|_| {
        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    });

    let err = run(&host, &Command::new("claude", &[], dir.path())).unwrap_err();
    assert_eq!(err, "the run logs to no file");

    let log = dir.path().join("none.jsonl");
    let err = run(
        &host,
        &Command::new("claude", &[], dir.path()).logging_to(&log),
    )
    .unwrap_err();
    assert!(
        err.starts_with(&format!("read {}: ", log.display())),
        "{err}"
    );

    let err = run(
        &host,
        &Command::new("other", &[], dir.path()).logging_to(&log),
    )
    .unwrap_err();
    assert_eq!(err, "other: not found");
}

#[test]
fn a_consultant_gets_the_target_a_skill_hands_it() {
    let w = world("standard", 2);

    let consultant = prompt(&component(Kind::Consultant, "persona-agent"), &w);

    assert!(
        consultant.starts_with("You are `persona-agent`. Review this pull request"),
        "{consultant}"
    );
    assert!(consultant.contains("`pr-head/`"), "{consultant}");
    assert!(consultant.contains("`pr.diff`"), "{consultant}");
    assert!(consultant.contains("`review/bundle.md`"), "{consultant}");
    assert!(consultant.contains("write none"), "{consultant}");
    assert!(!consultant.contains("review/reports"), "{consultant}");
}

#[test]
fn a_consultant_runs_as_the_session_s_agent_on_the_model_it_resolved() {
    let w = world("standard", 2);
    let log = PathBuf::from("/logs/run.jsonl");
    let mut consultant = component(Kind::Consultant, "persona-agent");
    consultant.model = "claude-sonnet-5-5".to_owned();
    consultant.max_turns = 40;

    let c = command(&consultant, &w, None, &log, Duration::from_secs(9));
    let args = args_of(&c);
    assert!(has(&args, ["--agent", "persona-agent"]));
    assert!(has(&args, ["--model", "claude-sonnet-5-5"]));
    assert!(has(&args, ["--max-turns", "40"]));
    assert!(has(&args, ["--allowedTools", GENERALIST_TOOLS]));
    assert!(has(&args, ["--disallowedTools", GENERALIST_DISALLOWED]));
    assert!(!args.contains(&"--json-schema"));

    let c = command(
        &consultant,
        &w,
        Some("claude-opus-5-5"),
        &log,
        Duration::from_secs(9),
    );
    assert!(has(&args_of(&c), ["--model", "claude-opus-5-5"]));
}
