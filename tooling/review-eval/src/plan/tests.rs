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
