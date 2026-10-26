use super::*;

fn agent(name: &str, effort: &str, body: &str) -> (String, String) {
    (
        format!("{AGENTS}/{name}.md"),
        format!("---\nname: {name}\ndescription: d\neffort: {effort}\n---\n{body}\n"),
    )
}

/// A checkout with one SRS section, one domain-model file and every
/// agent the dispatch names for them.
fn files() -> Vec<(String, String)> {
    let mut files = vec![
        ("docs/srs/index.md".to_owned(), "x".to_owned()),
        ("docs/srs/06-security.md".to_owned(), "x".to_owned()),
        ("docs/domain-model/index.md".to_owned(), "x".to_owned()),
        ("docs/domain-model/record.md".to_owned(), "x".to_owned()),
        agent("srs-security", "low", "Read docs/srs/06-security.md."),
        agent(
            "domain-model-record",
            "low",
            "Read docs/domain-model/record.md.",
        ),
    ];
    files.extend(FIXED_SPECIALISTS.iter().map(|n| agent(n, "low", "x")));
    files.extend(GENERALISTS.iter().map(|n| agent(n, "medium", "x")));
    files
}

fn problems(files: &[(String, String)]) -> Vec<String> {
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, b)| (p.as_str(), b.as_str()))
        .collect();
    let (_d, c) = crate::tests::checkout(&refs);
    every_dispatched_agent_is_defined(&c)
}

fn without(name: &str) -> Vec<(String, String)> {
    files()
        .into_iter()
        .filter(|(p, _)| p != &format!("{AGENTS}/{name}.md"))
        .collect()
}

fn replaced(name: &str, effort: &str, body: &str) -> Vec<(String, String)> {
    let mut f = without(name);
    f.push(agent(name, effort, body));
    f
}

#[test]
fn a_complete_set_has_no_problem() {
    assert_eq!(problems(&files()), Vec::<String>::new());
}

#[test]
fn a_missing_agent_is_named() {
    for name in [
        "srs-security",
        "domain-model-record",
        "lang-rust",
        "review-code",
    ] {
        assert_eq!(
            problems(&without(name)),
            [format!("no agent {name} at {AGENTS}/{name}.md")],
            "{name}"
        );
    }
}

#[test]
fn an_agent_at_the_wrong_effort_is_named() {
    assert_eq!(
        problems(&replaced("lang-rust", "medium", "x")),
        ["lang-rust runs at effort medium, want low"]
    );
    assert_eq!(
        problems(&replaced("review-spec", "low", "x")),
        ["review-spec runs at effort low, want medium"]
    );
}

#[test]
fn an_agent_whose_name_is_not_its_file_is_named() {
    let mut f = without("lang-toml");
    f.push((
        format!("{AGENTS}/lang-toml.md"),
        "---\nname: lang-tom\neffort: low\n---\n".to_owned(),
    ));

    assert_eq!(problems(&f), ["lang-toml names itself lang-tom"]);
}

#[test]
fn a_specialist_that_does_not_name_its_file_is_named() {
    assert_eq!(
        problems(&replaced("srs-security", "low", "Read the section.")),
        ["srs-security does not name docs/srs/06-security.md"]
    );
}

#[test]
fn a_directory_that_cannot_be_read_is_named() {
    let f: Vec<(String, String)> = files()
        .into_iter()
        .filter(|(p, _)| !p.starts_with("docs/"))
        .collect();

    let got = problems(&f);

    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got[0].starts_with("read docs/srs: "), "{got:?}");
    assert!(got[1].starts_with("read docs/domain-model: "), "{got:?}");
}

#[test]
fn the_front_matter_reads_quoted_values_and_only_its_own_block() {
    let mut f = without("lang-toml");
    f.push((
        format!("{AGENTS}/lang-toml.md"),
        "---\nname: \"lang-toml\"\neffort: 'low'\n---\nname: other\n".to_owned(),
    ));
    assert_eq!(problems(&f), Vec::<String>::new());

    let mut f = without("lang-toml");
    f.push((
        format!("{AGENTS}/lang-toml.md"),
        "name: lang-toml\n".to_owned(),
    ));
    assert_eq!(
        problems(&f),
        ["lang-toml's front matter does not read: no front matter"]
    );
}
