use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::*;

fn tree(root: &Path) {
    let write = |rel: &str, body: &str| {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    write("docs/srs/06-security.md", "# 6\n");
    write("docs/domain-model/index.md", "# hub\n");
    write(".claude/agents/lang-rust.md", "rust v1\n");
    write(".claude/agents/review-code.md", "code v1\n");
    write(".claude/agents/review-code-deep.md", "deep v1\n");
    write(".claude/skills/review/SKILL.md", "skill v1\n");
    write(
        "tooling/review-eval/cases/lang-rust/unwrap.toml",
        "case v1\n",
    );
    write(
        "tooling/review-eval/cases/review-code/keeps.toml",
        "case v1\n",
    );
}

#[test]
fn between_names_each_changed_component_and_why() {
    let base = TempDir::new().unwrap();
    let head = TempDir::new().unwrap();
    tree(base.path());
    tree(head.path());
    let h = |rel: &str, body: &str| {
        let path = head.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    h(".claude/agents/lang-rust.md", "rust v2\n");
    h(".claude/agents/lang-toml.md", "toml v1\n");
    // A skill is no reviewer component: changing one plans nothing.
    h(".claude/skills/review/SKILL.md", "skill v2\n");
    h(
        "tooling/review-eval/cases/review-code/keeps.toml",
        "case v2\n",
    );
    fs::create_dir_all(
        head.path()
            .join("tooling/review-eval/cases/lang-rust/notes"),
    )
    .unwrap();

    let got = between(base.path(), head.path()).unwrap();

    assert_eq!(
        got,
        vec![
            Planned {
                component: "lang-rust".to_owned(),
                reason: "instructions changed".to_owned(),
            },
            Planned {
                component: "lang-toml".to_owned(),
                reason: "new component".to_owned(),
            },
            Planned {
                component: "review-code".to_owned(),
                reason: "cases changed".to_owned(),
            },
            Planned {
                component: "review-code-deep".to_owned(),
                reason: "cases changed".to_owned(),
            },
        ]
    );
}

#[test]
fn between_plans_nothing_for_the_same_tree() {
    let base = TempDir::new().unwrap();
    tree(base.path());

    assert_eq!(between(base.path(), base.path()), Ok(Vec::new()));
}

#[test]
fn between_fails_on_a_head_without_the_sections() {
    let base = TempDir::new().unwrap();
    let head = TempDir::new().unwrap();
    tree(base.path());

    let err = between(base.path(), head.path()).unwrap_err();

    assert!(err.starts_with("read docs/srs: "), "{err}");
}

#[test]
fn files_read_a_directory_or_nothing() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.toml"), "a").unwrap();
    fs::create_dir_all(dir.path().join("sub")).unwrap();

    let got = files(dir.path()).unwrap();

    assert_eq!(got.len(), 1);
    assert_eq!(got["a.toml"], "a");
    assert_eq!(files(&dir.path().join("none")), Ok(BTreeMap::new()));
    let err = files(&dir.path().join("a.toml")).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
}

#[test]
fn a_planned_component_round_trips_through_json() {
    let planned = Planned {
        component: "lang-rust".to_owned(),
        reason: "instructions changed".to_owned(),
    };

    let text = serde_json::to_string(&planned).unwrap();

    assert_eq!(
        text,
        "{\"component\":\"lang-rust\",\"reason\":\"instructions changed\"}"
    );
    assert_eq!(serde_json::from_str::<Planned>(&text).unwrap(), planned);
}

/// An agent file with `own` text and an included section `file` whose
/// expanded body is `body`.
fn with_sections(own: &str, sections: &[(&str, &str)]) -> String {
    let mut text = format!("{own}\n");
    for (file, body) in sections {
        text.push_str(&format!(
            "<?include\nfile: ../../docs/agents/{file}\n?>\n{body}\n<?/include?>\n"
        ));
    }
    text
}

/// A tree whose agents `a`, `b` and `c` include section `s`, and `d`
/// includes `s` and `t`, with `body` as the expanded text of `s`.
fn shared_tree(root: &Path, own_b: &str, body: &str) {
    let write = |name: &str, text: String| {
        let path = root.join(format!(".claude/agents/{name}.md"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    };
    write("lang-a", with_sections("own a", &[("s.md", body)]));
    write("lang-b", with_sections(own_b, &[("s.md", body)]));
    write("lang-c", with_sections("own c", &[("s.md", body)]));
    write(
        "lang-d",
        with_sections("own d", &[("s.md", body), ("t.md", "t body")]),
    );
    let docs = root.join("docs/srs");
    fs::create_dir_all(&docs).unwrap();
    fs::create_dir_all(root.join("docs/domain-model")).unwrap();
    fs::write(root.join("docs/domain-model/index.md"), "# hub\n").unwrap();
    fs::write(docs.join("06-security.md"), "# 6\n").unwrap();
}

#[test]
fn a_shared_section_change_plans_one_component_per_set_of_sections() {
    let (base, head) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    shared_tree(base.path(), "own b", "s v1");
    shared_tree(head.path(), "own b", "s v2");

    let got = between(base.path(), head.path()).unwrap();

    assert_eq!(
        got,
        vec![
            Planned {
                component: "lang-a".to_owned(),
                reason: "shared sections changed; also covers lang-b, lang-c".to_owned(),
            },
            Planned {
                component: "lang-d".to_owned(),
                reason: "shared sections changed".to_owned(),
            },
        ]
    );
}

#[test]
fn a_component_whose_own_text_changed_is_always_planned() {
    let (base, head) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    shared_tree(base.path(), "own b", "s v1");
    shared_tree(head.path(), "own b changed", "s v2");

    let got = between(base.path(), head.path()).unwrap();

    let names: Vec<(&str, &str)> = got
        .iter()
        .map(|p| (p.component.as_str(), p.reason.as_str()))
        .collect();
    assert_eq!(
        names,
        [
            ("lang-a", "shared sections changed; also covers lang-c"),
            ("lang-b", "instructions changed"),
            ("lang-d", "shared sections changed"),
        ]
    );
}

#[test]
fn strip_includes_drops_only_the_expanded_bodies() {
    let text = with_sections("own", &[("s.md", "body one\nbody two")]);

    let got = strip_includes(&text);

    assert_eq!(
        got,
        "own\n<?include\nfile: ../../docs/agents/s.md\n?>\n<?/include?>\n"
    );
    assert_eq!(strip_includes("no sections\n"), "no sections\n");
}

#[test]
fn sections_of_names_each_included_file_once_sorted() {
    let text = with_sections("own", &[("t.md", "x"), ("s.md", "y"), ("t.md", "z")]);

    assert_eq!(
        sections_of(&text),
        ["../../docs/agents/s.md", "../../docs/agents/t.md"]
    );
    assert!(sections_of("none\n").is_empty());
}
