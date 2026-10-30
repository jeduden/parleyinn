use std::fs;

use testkit::TempDir;

use super::*;

/// A tree with one SRS section, one model file, an agent of each kind
/// and the orchestrator's skill.
fn tree() -> TempDir {
    let dir = TempDir::new().unwrap();
    let write = |rel: &str, body: &str| {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    write("docs/srs/06-security.md", "# 6\n");
    write("docs/srs/index.md", "# index\n");
    write("docs/srs/notes.txt", "no section");
    write("docs/domain-model/index.md", "# hub\n");
    write("docs/domain-model/places.md", "# places\n");
    write(
        ".claude/agents/lang-rust.md",
        "---\nname: lang-rust\ndescription: >-\n  Rust.\ntools: Read, Grep, Glob, Write\nmodel: haiku\neffort: low\nmaxTurns: 20\ngeneralist: review-code\n---\n# Rust\n\nfindings, each on one line.\n",
    );
    write(
        ".claude/agents/review-code.md",
        "---\nname: review-code\ndescription: code\ntools: Read, Grep, Glob\nmodel: sonnet\neffort: medium\n---\n# Code\n",
    );
    write(
        ".claude/agents/review-code-deep.md",
        "---\nname: review-code-deep\ndescription: code\ntools: Read, Grep, Glob\nmodel: opus\neffort: high\nmaxTurns: 40\n---\n# Code\n",
    );
    write(
        ".claude/skills/review/SKILL.md",
        "---\nname: review\ndescription: review\n---\n# review\n",
    );
    dir
}

#[test]
fn names_list_the_orchestrator_the_generalists_and_every_specialist() {
    let dir = tree();

    let got = names(dir.path()).unwrap();

    assert!(got.contains(&"review".to_owned()));
    assert!(got.contains(&"review-code".to_owned()));
    assert!(got.contains(&"review-spec-deep".to_owned()));
    assert!(got.contains(&"lang-rust".to_owned()));
    assert!(got.contains(&"srs-security".to_owned()));
    assert!(got.contains(&"domain-model-hub".to_owned()));
    assert!(got.contains(&"domain-model-places".to_owned()));
    assert!(!got.contains(&"srs-index".to_owned()));
    let mut sorted = got.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(got, sorted);
}

#[test]
fn names_fail_on_a_tree_without_the_sections() {
    let dir = TempDir::new().unwrap();

    let err = names(dir.path()).unwrap_err();

    assert!(err.starts_with("read docs/srs: "), "{err}");
}

#[test]
fn names_fail_on_a_tree_without_the_model() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("docs/srs")).unwrap();

    let err = names(dir.path()).unwrap_err();

    assert!(err.starts_with("read docs/domain-model: "), "{err}");
}

#[cfg(unix)]
#[test]
fn names_fail_on_a_section_whose_name_is_not_utf_8() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let dir = tree();
    fs::write(
        dir.path()
            .join("docs/srs")
            .join(OsStr::from_bytes(b"\xff.md")),
        "x",
    )
    .unwrap();

    let err = names(dir.path()).unwrap_err();

    assert!(err.ends_with(" is not UTF-8"), "{err}");
}

#[test]
fn each_name_has_its_kind_suite_and_file() {
    assert_eq!(kind_of("review"), Kind::Orchestrator);
    assert_eq!(kind_of("review-spec"), Kind::Generalist);
    assert_eq!(kind_of("review-spec-deep"), Kind::Generalist);
    assert_eq!(kind_of("lang-rust"), Kind::Specialist);
    assert_eq!(suite_of("review-code-deep"), "review-code");
    assert_eq!(suite_of("review-code"), "review-code");
    assert_eq!(suite_of("lang-rust"), "lang-rust");
    assert_eq!(file_of("review"), ".claude/skills/review/SKILL.md");
    assert_eq!(file_of("lang-rust"), ".claude/agents/lang-rust.md");
}

#[test]
fn hash_is_short_stable_and_text_sensitive() {
    assert_eq!(hash("a").len(), 16);
    assert_eq!(hash("a"), hash("a"));
    assert_ne!(hash("a"), hash("b"));
}

#[test]
fn load_reads_a_specialist_s_front_matter() {
    let dir = tree();

    let c = load(dir.path(), "lang-rust").unwrap();

    assert_eq!(c.name, "lang-rust");
    assert_eq!(c.kind, Kind::Specialist);
    assert_eq!(c.file, ".claude/agents/lang-rust.md");
    assert_eq!(c.model, "haiku");
    assert_eq!(c.effort, "low");
    assert_eq!(c.max_turns, 20);
    assert_eq!(c.tools, vec!["Read", "Grep", "Glob", "Write"]);
    assert!(c.strict_lines);
    assert!(c.text.starts_with("---\n"));
}

#[test]
fn a_report_that_must_be_all_findings_holds_a_specialist_to_one_line_each() {
    for sentence in [
        "Every line of the report is a finding or the outcome: no heading.",
        "Every line is a finding or the\noutcome, never a heading.",
        "Findings, each on one line.",
    ] {
        assert!(one_finding_per_line(sentence), "{sentence}");
    }
    assert!(!one_finding_per_line("Report each finding you keep."));
}

#[test]
fn load_gives_a_generalist_without_a_turn_budget_the_default() {
    let dir = tree();

    let c = load(dir.path(), "review-code").unwrap();

    assert_eq!(c.kind, Kind::Generalist);
    assert_eq!(c.max_turns, GENERALIST_TURNS);
    assert!(!c.strict_lines);
    assert_eq!(load(dir.path(), "review-code-deep").unwrap().max_turns, 40);
}

#[test]
fn load_gives_the_orchestrator_the_workflow_s_settings() {
    let dir = tree();

    let c = load(dir.path(), "review").unwrap();

    assert_eq!(c.kind, Kind::Orchestrator);
    assert_eq!(c.model, ORCHESTRATOR_MODEL);
    assert_eq!(c.effort, ORCHESTRATOR_EFFORT);
    assert_eq!(c.max_turns, ORCHESTRATOR_TURNS);
    assert_eq!(c.tools, vec!["Read", "Glob", "Grep", "Agent", "Write"]);
}

#[test]
fn load_names_each_problem() {
    let dir = tree();
    let agents = dir.path().join(".claude/agents");

    let err = load(dir.path(), "lang-toml").unwrap_err();
    assert!(
        err.starts_with("read .claude/agents/lang-toml.md: "),
        "{err}"
    );

    fs::write(agents.join("lang-toml.md"), "no front matter\n").unwrap();
    assert_eq!(
        load(dir.path(), "lang-toml").unwrap_err(),
        ".claude/agents/lang-toml.md: no front matter"
    );

    fs::write(
        agents.join("lang-toml.md"),
        "---\nname: lang-toml\ndescription: t\ntools: Read\n---\n# t\n",
    )
    .unwrap();
    assert_eq!(
        load(dir.path(), "lang-toml").unwrap_err(),
        ".claude/agents/lang-toml.md: names no model"
    );

    fs::write(
        agents.join("lang-toml.md"),
        "---\nname: lang-toml\ndescription: t\ntools: Read\nmodel: haiku\nmaxTurns: many\n---\n# t\n",
    )
    .unwrap();
    assert_eq!(
        load(dir.path(), "lang-toml").unwrap_err(),
        ".claude/agents/lang-toml.md: maxTurns many is not a number"
    );
}

#[test]
fn load_refuses_a_key_or_a_tool_a_component_may_not_carry() {
    let dir = tree();
    let agents = dir.path().join(".claude/agents");
    let write = |rel: &str, body: &str| fs::write(dir.path().join(rel), body).unwrap();

    write(
        ".claude/agents/lang-toml.md",
        "---\nname: lang-toml\ndescription: t\ntools: Read\nmodel: haiku\nhooks: []\n---\n# t\n",
    );
    assert_eq!(
        load(dir.path(), "lang-toml").unwrap_err(),
        ".claude/agents/lang-toml.md: front matter names hooks, which a reviewer component may not"
    );
    write(
        ".claude/agents/lang-toml.md",
        "---\nname: lang-toml\ndescription: t\ntools: Read, Bash\nmodel: haiku\n---\n# t\n",
    );
    assert_eq!(
        load(dir.path(), "lang-toml").unwrap_err(),
        ".claude/agents/lang-toml.md: lists tool Bash, which a specialist may not"
    );
    write(
        ".claude/agents/review-code.md",
        "---\nname: review-code\ndescription: c\ntools: Read, Write\nmodel: sonnet\n---\n# c\n",
    );
    assert_eq!(
        load(dir.path(), "review-code").unwrap_err(),
        ".claude/agents/review-code.md: lists tool Write, which a generalist may not"
    );
    write(
        ".claude/skills/review/SKILL.md",
        "---\nname: review\ndescription: r\nhooks: []\n---\n# r\n",
    );
    assert_eq!(
        load(dir.path(), "review").unwrap_err(),
        ".claude/skills/review/SKILL.md: front matter names hooks, which a reviewer component may not"
    );
    // A nested block and a flow mapping are what an agent's hooks look
    // like in YAML; the parser refuses the one and the allow-list the
    // other, so neither reaches a run.
    write(
        ".claude/agents/lang-toml.md",
        "---\nname: lang-toml\ndescription: t\ntools: Read\nmodel: haiku\nhooks:\n  PreToolUse:\n    - command: curl evil\n---\n# t\n",
    );
    let err = load(dir.path(), "lang-toml").unwrap_err();
    assert!(
        err.starts_with(".claude/agents/lang-toml.md: line 7: unsupported front matter"),
        "{err}"
    );
    write(
        ".claude/agents/lang-toml.md",
        "---\nname: lang-toml\ndescription: t\ntools: Read\nmodel: haiku\nhooks: {PreToolUse: [{command: curl evil}]}\n---\n# t\n",
    );
    assert_eq!(
        load(dir.path(), "lang-toml").unwrap_err(),
        ".claude/agents/lang-toml.md: front matter names hooks, which a reviewer component may not"
    );
    assert!(agents.join("lang-rust.md").is_file());
    assert!(load(dir.path(), "lang-rust").is_ok());
}

#[test]
fn catalog_reads_every_component_or_names_the_missing_one() {
    let dir = tree();

    let err = catalog(dir.path(), dir.path()).unwrap_err();

    // The tree defines only four of the components the root names.
    assert!(err.starts_with("read .claude/agents/"), "{err}");
}

/// A consultant's file: an agent under `.claude/agents` the dispatch
/// never names, with no model of its own.
const PERSONA: &str = "---\nname: persona-agent\ndescription: >-\n  Claude as a user.\ntools: Read, Grep, Glob\n---\n# Persona: agent\n";

#[test]
fn names_list_every_other_agent_under_the_agents_directory_as_a_consultant() {
    let dir = tree();
    fs::write(dir.path().join(".claude/agents/persona-agent.md"), PERSONA).unwrap();
    fs::write(dir.path().join(".claude/agents/notes.txt"), "no agent").unwrap();

    let got = names(dir.path()).unwrap();

    assert!(got.contains(&"persona-agent".to_owned()), "{got:?}");
    assert!(!got.contains(&"notes".to_owned()), "{got:?}");
    assert_eq!(got.iter().filter(|n| *n == "lang-rust").count(), 1);
    assert_eq!(kind_of("persona-agent"), Kind::Consultant);
    assert_eq!(kind_of("domain-model"), Kind::Consultant);
    assert_eq!(kind_of("test-engineer"), Kind::Consultant);
    assert_eq!(kind_of("domain-model-hub"), Kind::Specialist);
    assert_eq!(kind_of("domain-model-places"), Kind::Specialist);
    assert_eq!(kind_of("srs-security"), Kind::Specialist);
    assert_eq!(kind_of("ci-engineer"), Kind::Specialist);
    assert_eq!(suite_of("persona-agent"), "persona-agent");
    assert_eq!(file_of("persona-agent"), ".claude/agents/persona-agent.md");
}

#[test]
fn each_kind_has_its_word() {
    assert_eq!(Kind::Orchestrator.word(), "orchestrator");
    assert_eq!(Kind::Generalist.word(), "generalist");
    assert_eq!(Kind::Specialist.word(), "specialist");
    assert_eq!(Kind::Consultant.word(), "consultant");
}

#[test]
fn load_gives_a_consultant_without_a_model_the_default_and_no_write_tool() {
    let dir = tree();
    let file = dir.path().join(".claude/agents/persona-agent.md");
    fs::write(&file, PERSONA).unwrap();

    let c = load(dir.path(), "persona-agent").unwrap();

    assert_eq!(c.kind, Kind::Consultant);
    assert_eq!(c.model, CONSULTANT_MODEL);
    assert_eq!(c.effort, "");
    assert_eq!(c.max_turns, GENERALIST_TURNS);
    assert_eq!(c.tools, vec!["Read", "Grep", "Glob"]);

    fs::write(
        &file,
        PERSONA.replace(
            "tools: Read, Grep, Glob\n",
            "tools: Read, Grep, Glob\nmodel: opus\n",
        ),
    )
    .unwrap();
    assert_eq!(load(dir.path(), "persona-agent").unwrap().model, "opus");

    fs::write(
        &file,
        PERSONA.replace(
            "tools: Read, Grep, Glob\n",
            "tools: Read, Grep, Glob, Write\n",
        ),
    )
    .unwrap();
    assert_eq!(
        load(dir.path(), "persona-agent").unwrap_err(),
        ".claude/agents/persona-agent.md: lists tool Write, which a consultant may not"
    );
}
