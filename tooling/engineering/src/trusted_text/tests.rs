use super::*;
use testkit::TempDir;

fn problems(text: &str, inside: bool) -> Vec<String> {
    check_file("src/lib.rs", text, inside)
}

#[test]
fn a_construction_outside_the_restore_block_crate_is_named_by_file_and_line() {
    let text = "fn a() {}\n\nfn leak(s: String) -> TrustedText {\n    TrustedText::new(s)\n}\n";

    let got = problems(text, false);

    assert_eq!(
        got,
        [
            "src/lib.rs:4: TrustedText is constructed (TrustedText::new) outside the restore_block crate"
        ]
    );
}

#[test]
fn every_way_to_build_one_is_refused_outside_the_crate() {
    for (text, how) in [
        ("let t = TrustedText::from(s);", "TrustedText::from"),
        ("let t = TrustedText::default();", "TrustedText::default"),
        ("let t = TrustedText::fixed(\"x\");", "TrustedText::fixed"),
        ("let t = TrustedText :: new(s);", "TrustedText::new"),
        ("let t = TrustedText(s);", "TrustedText(..)"),
        ("let t = TrustedText { text: s };", "a struct literal"),
        (
            "impl From<String> for TrustedText {}",
            "a trait that builds one",
        ),
        ("impl Default for TrustedText {}", "a trait that builds one"),
        (
            "impl std::str::FromStr for TrustedText {}",
            "a trait that builds one",
        ),
        (
            "#[derive(Default)]\npub struct TrustedText(String);",
            "a derive that builds one",
        ),
        (
            "#[derive(Clone, serde::Deserialize)]\nstruct TrustedText { s: String }",
            "a derive that builds one",
        ),
    ] {
        let got = problems(text, false);

        assert_eq!(got.len(), 1, "{text:?}: {got:?}");
        assert!(got[0].contains(how), "{text:?}: {got:?}");
        assert!(
            got[0].ends_with("outside the restore_block crate"),
            "{got:?}"
        );
    }
}

#[test]
fn a_type_position_builds_nothing() {
    let text = "\
use restore_block::TrustedText;
fn take(t: &TrustedText, v: Vec<TrustedText>) -> TrustedText { t.clone() }
fn give() -> Result<TrustedText, E> { todo() }
impl Display for TrustedText {}
impl TrustedText { fn len(&self) -> usize { 0 } }
struct TrustedTexts(String);
fn dynamic(_: &dyn Into<TrustedText>) {}
// TrustedText::new(s) in a comment
/* TrustedText(s) in a block comment */
const NOTE: &str = \"TrustedText::new(s) in a string\";
const RAW: &str = r#\"TrustedText { } in a raw string\"#;
";

    assert_eq!(problems(text, false), Vec::<String>::new());
    assert_eq!(problems(text, true), Vec::<String>::new());
}

#[test]
fn inside_the_crate_only_the_four_constructors_build_one() {
    let allowed = "\
let a = TrustedText::fixed(\"restore block follows\");
let b = TrustedText::fixed(RECALL_HINT);
let c = TrustedText::pin(&pin);
let d = TrustedText::field(&field);
let e = TrustedText::join(&[a, b]);
let f = TrustedText::fixed(r#\"raw\"#);
pub struct TrustedText(String);
impl TrustedText { fn fixed(text: &'static str) -> Self { Self(text.to_owned()) } }
";

    assert_eq!(problems(allowed, true), Vec::<String>::new());
    assert!(
        problems("let a = TrustedText::fixed(\"x\");", false).len() == 1,
        "outside the crate even those are refused"
    );
}

#[test]
fn inside_the_crate_anything_else_is_refused_with_why() {
    for (text, how) in [
        ("let a = TrustedText::new(recalled);", "TrustedText::new"),
        ("let a = TrustedText::from(recalled);", "TrustedText::from"),
        ("let a = TrustedText(recalled);", "TrustedText(..)"),
        ("let a = TrustedText { text };", "a struct literal"),
        (
            "impl From<String> for TrustedText {}",
            "a trait that builds one",
        ),
        (
            "#[derive(Deserialize)]\nstruct TrustedText(String);",
            "a derive that builds one",
        ),
    ] {
        let got = problems(text, true);

        assert_eq!(got.len(), 1, "{text:?}: {got:?}");
        assert!(
            got[0].contains(&format!("builds TrustedText by {how}")),
            "{got:?}"
        );
        assert!(got[0].contains("qualifying pins, sanitized structural fields or fixed text"));
    }
    for text in [
        "let a = TrustedText::fixed(recalled);",
        "let a = TrustedText::fixed(format!(\"{x}\"));",
        "let a = TrustedText::fixed(Recalled);",
        "let a = TrustedText::fixed(A + b);",
        "let a = TrustedText::fixed();",
    ] {
        let got = problems(text, true);

        assert_eq!(got.len(), 1, "{text:?}: {got:?}");
        assert!(
            got[0].contains("fixed from something but a string literal"),
            "{got:?}"
        );
    }
}

#[test]
fn comments_and_literals_are_blanked_with_their_lines_kept() {
    let text = "a // TrustedText\nb /* x\n TrustedText */ c \"TrustedText\\\"\" d 'x' '\"' 'a: &'a str r\"TrustedText\" e";

    let blanked = blank(text);

    assert!(!blanked.contains("TrustedText"), "{blanked:?}");
    assert_eq!(blanked.matches('\n').count(), text.matches('\n').count());
    assert!(
        blanked.contains("'a: &'a str"),
        "a lifetime is code: {blanked:?}"
    );
    assert!(blank("/* a /* nested */ still */ code").trim() == "code");
    assert_eq!(
        blank("'\\n' x").trim_start(),
        "' ' x".replace("' '", "'  '")
    );
}

fn checkout(files: &[(&str, &str)]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    for (path, body) in files {
        let full = dir.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, body).unwrap();
    }
    let c = Checkout::new(dir.path());

    (dir, c)
}

#[test]
fn the_check_reads_every_crate_and_names_the_offending_file() {
    let (_dir, c) = checkout(&[
        ("crates/mcp/src/lib.rs", "fn a() {}\n"),
        (
            "crates/mcp/src/leak.rs",
            "fn b(s: String) {\n    let _ = TrustedText::new(s);\n}\n",
        ),
        ("crates/mcp/tests/t.rs", "let _ = TrustedText(s);\n"),
        ("crates/mcp/target/debug/x.rs", "let _ = TrustedText(s);\n"),
        (
            "crates/restore_block/src/lib.rs",
            "let _ = TrustedText::fixed(\"ok\");\n",
        ),
        (
            "crates/restore_block/src/bad.rs",
            "\nlet _ = TrustedText::from(s);\n",
        ),
        ("crates/restore_block/notes.txt", "TrustedText::new(s)\n"),
    ]);

    let got = constructions(&c);

    assert_eq!(got.len(), 3, "{got:?}");
    assert!(got[0].starts_with("crates/mcp/src/leak.rs:2: "), "{got:?}");
    assert!(got[1].starts_with("crates/mcp/tests/t.rs:1: "), "{got:?}");
    assert!(
        got[2].starts_with("crates/restore_block/src/bad.rs:2: "),
        "{got:?}"
    );
}

#[test]
fn a_repository_that_builds_nothing_outside_the_crate_passes() {
    let (_dir, c) = checkout(&[
        ("crates/mcp/src/lib.rs", "fn a(t: &TrustedText) {}\n"),
        (
            "crates/restore_block/src/lib.rs",
            "let _ = TrustedText::pin(&p);\n",
        ),
    ]);

    assert_eq!(constructions(&c), Vec::<String>::new());
}

#[test]
fn a_checkout_without_crates_or_with_an_unreadable_file_is_a_problem() {
    let (_dir, none) = checkout(&[("README.md", "x")]);
    assert_eq!(
        constructions(&none),
        ["list crates: cannot read the crates directory"]
    );
    let (dir, c) = checkout(&[("crates/mcp/src/lib.rs", "x")]);
    std::fs::write(dir.path().join("crates/mcp/src/bad.rs"), [0xff, 0xfe]).unwrap();

    let got = constructions(&c);

    assert_eq!(got.len(), 1);
    assert!(
        got[0].starts_with("read crates/mcp/src/bad.rs: "),
        "{got:?}"
    );
}

#[test]
fn the_repository_itself_builds_no_trusted_text_outside_its_crate() {
    let c = Checkout::new(&testkit::repo_root());

    assert_eq!(constructions(&c), Vec::<String>::new());
}

#[test]
fn a_comment_or_literal_the_file_ends_in_is_blanked_to_its_end() {
    for text in [
        "a /* open",
        "a \"open",
        "a \"open\\",
        "a r#\"open",
        "a // open",
    ] {
        let blanked = blank(text);

        assert_eq!(blanked.chars().count(), text.chars().count(), "{text:?}");
        assert!(!blanked.contains("open"), "{text:?}");
    }
}
