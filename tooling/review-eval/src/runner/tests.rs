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
        model: "haiku".to_owned(),
        effort: "low".to_owned(),
        max_turns: 20,
        tools: Vec::new(),
        strict_lines: true,
    }
}

fn world(class: &str) -> World {
    World {
        main: PathBuf::from("/work/main"),
        class: class.to_owned(),
        touched: 1,
        seeded: vec!["review/reports/lang-rust.md".to_owned()],
        notes: Vec::new(),
    }
}

#[test]
fn a_specialist_run_over_a_part_is_told_to_read_that_part_alone() {
    let w = world("standard");
    let c = component(Kind::Specialist, "lang-rust");

    let part = prompt_over(&c, &w, Some("review/diffs/lang-rust.2.diff"));

    assert!(
        part.contains("Read only your own diff, `review/diffs/lang-rust.2.diff`, alone first;"),
        "{part}"
    );
    assert!(!part.contains("`review/diffs/lang-rust.diff`"), "{part}");
    assert_eq!(prompt(&c, &w), prompt_over(&c, &w, None));
    let general = component(Kind::Generalist, "review-code");
    assert_eq!(
        prompt_over(&general, &w, Some("x.diff")),
        prompt(&general, &w)
    );
}

#[test]
fn a_command_over_a_part_carries_the_part_s_prompt() {
    let w = world("standard");
    let c = component(Kind::Specialist, "lang-rust");
    let log = Path::new("log.jsonl");
    let deadline = Duration::from_secs(1);

    let over = command_over(
        &c,
        &w,
        None,
        log,
        deadline,
        Some("review/diffs/lang-rust.1.diff"),
    );

    assert_eq!(
        over.args[1],
        prompt_over(&c, &w, Some("review/diffs/lang-rust.1.diff"))
    );
    assert_eq!(
        command_over(&c, &w, None, log, deadline, None),
        super::command(&c, &w, None, log, deadline)
    );
}

#[test]
fn each_kind_gets_its_prompt() {
    let w = world("standard");

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
        specialist.contains("You write no file: state each finding as one line in your text"),
        "{specialist}"
    );
    assert!(
        specialist.ends_with(
            "and answer with your outcome line alone, `outcome: findings — <sentence>` \
             or `outcome: none — <sentence>`. The review assembles your report from \
             the lines you stated."
        ),
        "{specialist}"
    );
    assert!(!specialist.contains("review/reports"), "{specialist}");
    // The bundle is too large for one read on a large change, so a
    // specialist reads its own diff and its own small context alone.
    assert!(
        specialist.contains(
            "Read only your own diff, `review/diffs/lang-rust.diff`, alone first; \
             then `review/context/lang-rust.md`, which names the review class, your \
             dispatch row and the deterministic findings on your files; never \
             `review/bundle.md`."
        ),
        "{specialist}"
    );
    assert!(
        !specialist.contains("Read `review/bundle.md`"),
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

    let mut bare = world("light");
    bare.touched = 0;
    bare.seeded.clear();
    let generalist = prompt(&component(Kind::Generalist, "review-code"), &bare);
    assert!(
        generalist.contains("lists the 0 touched ids."),
        "{generalist}"
    );
    assert!(generalist.contains("reports are none."), "{generalist}");
}

#[test]
fn every_kind_is_told_to_read_the_pull_request_s_tree_and_not_main_s() {
    let w = world("standard");

    for (kind, name) in [
        (Kind::Specialist, "srs-security"),
        (Kind::Generalist, "review-spec"),
        (Kind::Consultant, "persona-agent"),
    ] {
        let prompt = prompt(&component(kind, name), &w);

        // A path a diff or an agent's instructions name is relative to the
        // repository, and the work directory's root is main's tree: an
        // agent that reads it there judges the change against main.
        assert!(prompt.contains(HEAD_TREE), "{prompt}");
        assert!(!prompt.contains("  "), "{prompt}");
    }
    // The whole wording is pinned: the middle clause is what stops an
    // agent from opening a path as its diff or its instructions give it.
    assert_eq!(
        HEAD_TREE,
        "The pull request's tree is in `pr-head/`: read and search the repository's files \
         there, each as `pr-head/<its path>`, whatever path your diff or your instructions \
         name. The tree around it is main's: it shows only what the change replaced, and \
         nothing the pull request adds."
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
    let w = world("standard");
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
    assert!(has(&args, ["--allowedTools", TOOLS]));
    assert!(has(&args, ["--disallowedTools", DISALLOWED]));
    assert!(has(&args, ["--agent", "lang-rust"]));
    assert!(!args.contains(&"--model"));
    assert!(!args.contains(&"--effort"));
}

#[test]
fn every_run_logs_stream_json_without_a_session_from_the_project_s_settings() {
    let w = world("standard");
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
    let w = world("standard");
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
    assert!(has(&args, ["--allowedTools", TOOLS]));
    assert!(has(&args, ["--disallowedTools", DISALLOWED]));
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
    let w = world("standard");

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
    assert!(!consultant.contains("  "), "{consultant}");
}

#[test]
fn a_consultant_runs_as_the_session_s_agent_on_the_model_it_resolved() {
    let w = world("standard");
    let log = PathBuf::from("/logs/run.jsonl");
    let mut consultant = component(Kind::Consultant, "persona-agent");
    consultant.model = "claude-sonnet-5-5".to_owned();
    consultant.max_turns = 40;

    let c = command(&consultant, &w, None, &log, Duration::from_secs(9));
    let args = args_of(&c);
    assert!(has(&args, ["--agent", "persona-agent"]));
    assert!(has(&args, ["--model", "claude-sonnet-5-5"]));
    assert!(has(&args, ["--max-turns", "40"]));
    assert!(has(&args, ["--allowedTools", TOOLS]));
    assert!(has(&args, ["--disallowedTools", DISALLOWED]));
    assert!(!args.contains(&"--effort"));

    let c = command(
        &consultant,
        &w,
        Some("claude-opus-5-5"),
        &log,
        Duration::from_secs(9),
    );
    assert!(has(&args_of(&c), ["--model", "claude-opus-5-5"]));
}

#[test]
fn every_agent_reads_alone_and_is_refused_every_writing_tool() {
    // Pinned as text, so a writing tool re-added to either list fails
    // here rather than passing every test that compares the constants.
    assert_eq!(TOOLS, "Read,Glob,Grep");
    assert_eq!(
        DISALLOWED,
        "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch,Agent,\
         Read(//proc/**),Read(//sys/**),Read(//dev/**)"
    );
}

#[test]
fn every_run_is_capped_at_its_tier_s_budget() {
    // Pinned as text, so a cap raised or dropped fails here rather than
    // passing every test that compares the constants.
    assert_eq!(budget_usd(Kind::Specialist), "0.30");
    assert_eq!(budget_usd(Kind::Generalist), "1.50");
    let w = world("standard");
    let log = PathBuf::from("/logs/run.jsonl");
    for (kind, name, want) in [
        (Kind::Specialist, "lang-rust", "0.30"),
        (Kind::Generalist, "review-code", "1.50"),
        (Kind::Consultant, "persona-agent", "1.50"),
    ] {
        let c = command(
            &component(kind, name),
            &w,
            None,
            &log,
            Duration::from_secs(9),
        );

        assert!(has(&args_of(&c), ["--max-budget-usd", want]), "{name}");
        assert_eq!(budget_usd(kind), want, "{name}");
    }
}

#[test]
fn every_agent_is_refused_reads_outside_its_trees_and_of_the_process_files() {
    // Pinned as text: the process files hold every environment on the
    // machine, the review's own among them, and nothing outside the two
    // trees is the agent's to read (ENG-28).
    assert_eq!(
        SETTINGS,
        r#"{"permissions":{"blockReadsOutsideWorkingDirectories":true}}"#
    );
    for root in ["//proc/**", "//sys/**", "//dev/**"] {
        assert!(DISALLOWED.contains(&format!("Read({root})")), "{root}");
    }
    let c = command(
        &component(Kind::Specialist, "lang-rust"),
        &world("standard"),
        None,
        &PathBuf::from("/l"),
        Duration::from_secs(1),
    );
    assert!(has(&args_of(&c), ["--settings", SETTINGS]));
}
