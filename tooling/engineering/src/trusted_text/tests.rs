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
fn a_name_that_hides_the_type_is_refused_wherever_it_stands() {
    for (text, how) in [
        (
            "use restore_block::TrustedText as Safe;",
            "a `use .. as` alias",
        ),
        (
            "use restore_block::{Other, TrustedText as Safe};",
            "a `use .. as` alias",
        ),
        ("type Safe = TrustedText;", "a type alias"),
        (
            "pub type Safe = restore_block::TrustedText;",
            "a type alias",
        ),
        ("let t = <TrustedText>::new(s);", "a qualified path"),
        (
            "let t = <TrustedText as From<String>>::from(s);",
            "a qualified path",
        ),
        ("let t = (<TrustedText>::fixed(\"x\"));", "a qualified path"),
    ] {
        for inside in [false, true] {
            let got = problems(text, inside);

            assert_eq!(got.len(), 1, "{text:?} inside {inside}: {got:?}");
            assert!(got[0].contains(how), "{text:?}: {got:?}");
            assert!(got[0].starts_with("src/lib.rs:1: "), "{got:?}");
        }
    }
    assert!(
        problems("type Safe = TrustedText;", false)[0].ends_with("outside the restore_block crate")
    );
    assert!(problems("type Safe = TrustedText;", true)[0].contains("hides how it is built"));
}

#[test]
fn naming_the_type_inside_another_type_is_no_alias() {
    let text = "\
type Pins = Vec<TrustedText>;
fn f(x: Option<TrustedText>) -> Box<TrustedText> { todo() }
let v = Vec::<TrustedText>::new();
let w = Vec::<TrustedText>::with_capacity(1);
fn g() -> impl Iterator<Item = <TrustedText as Block>::Out> { todo() }
";

    // Only the last, a qualified path of the type, is refused.
    let got = problems(text, false);

    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].starts_with("src/lib.rs:5: "), "{got:?}");
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

#[test]
fn inside_the_crate_self_builds_one_only_in_the_four_constructors() {
    let allowed = "\
pub struct TrustedText(String);
impl TrustedText {
    pub fn fixed(text: &'static str) -> Self { Self(text.to_owned()) }
    pub fn pin(pin: &Pin) -> Self { Self(pin.text().to_owned()) }
    pub fn field(field: &Field) -> Self { Self { s: field.sanitized() } }
    pub fn join(parts: &[Self]) -> Self { Self(parts.iter().map(|p| p.0.as_str()).collect()) }
    pub fn hint() -> Self { Self::fixed(\"recall with event_get\") }
    pub fn as_str(&self) -> &str { let Self(s) = self; s }
    pub fn len(&self) -> usize { self.0.len() }
}
";

    assert_eq!(problems(allowed, true), Vec::<String>::new());
}

#[test]
fn inside_the_crate_self_anywhere_else_is_refused() {
    for (text, how) in [
        (
            "impl TrustedText {\n    pub fn wrap(s: &str) -> Self { Self(s.into()) }\n}",
            "Self(..) in wrap",
        ),
        (
            "impl TrustedText {\n    pub fn wrap(s: &str) -> Self { Self { s: s.into() } }\n}",
            "Self { .. } in wrap",
        ),
        (
            "impl TrustedText {\n    pub fn pin(p: &Pin) -> Self { Self::wrap(p) }\n}",
            "Self::wrap",
        ),
        (
            "impl restore_block::TrustedText {\n    fn hint() -> Self { Self::new(x) }\n}",
            "Self::new",
        ),
        (
            "impl Clone for TrustedText {\n    fn clone(&self) -> Self { Self(self.0.clone()) }\n}",
            "Self(..) in clone",
        ),
    ] {
        let got = problems(text, true);

        assert_eq!(got.len(), 1, "{text:?}: {got:?}");
        assert!(got[0].starts_with("src/lib.rs:2: "), "{got:?}");
        assert!(
            got[0].contains(&format!("builds TrustedText by {how}")),
            "{how}: {got:?}"
        );
    }
    let got = problems(
        "impl TrustedText {\n    fn hint(recalled: &str) -> Self { Self::fixed(recalled) }\n}",
        true,
    );
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(
        got[0].contains("fixed from something but a string literal"),
        "{got:?}"
    );
}

#[test]
fn every_method_of_an_impl_is_checked_not_the_first_alone() {
    let text = "\
impl TrustedText {
    pub fn fixed(text: &'static str) -> Self { Self(text.to_owned()) }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn wrap(recalled: &str) -> Self { Self(recalled.into()) }
    pub fn append(&mut self, recalled: &str) {}
}
";

    let got = problems(text, true);

    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got[0].starts_with("src/lib.rs:4: "), "{got:?}");
    assert!(got[0].contains("Self(..) in wrap"), "{got:?}");
    assert!(got[1].starts_with("src/lib.rs:5: "), "{got:?}");
    assert!(got[1].contains("the method append"), "{got:?}");
}

#[test]
fn an_impl_that_builds_and_changes_nothing_passes_however_it_is_cut() {
    for text in [
        "impl TrustedText {\n    fn len(&self) -> usize;\n    fn max() -> u8 { Self::MAX }\n    fn same(&self) -> bool { let x: Self = y; x.len() == 0 }\n}",
        "impl<T: Into<String>> Other<T> for Mine { fn f() { Self(t) } }",
        "impl Other for Mine;",
        "struct S; impl",
        "impl<T TrustedText",
        "impl<T TrustedText {}",
        "fn simple() {} struct Implied;",
        "impl TrustedText {\n    const C: P = P { a: 1 };\n    fn me() -> u8 { MySelf::new() }\n}",
        "impl TrustedText {\n    fn len(&self) -> usize { 0",
    ] {
        assert_eq!(problems(text, true), Vec::<String>::new(), "{text:?}");
    }
}

#[test]
fn a_change_to_trusted_text_is_refused_inside_the_crate_and_out() {
    for (text, what) in [
        (
            "impl TrustedText {\n    pub fn append(&mut self, s: &str) { self.0.push_str(s) }\n}",
            "the method append, which takes mut self",
        ),
        (
            "impl TrustedText {\n    pub fn then(mut self, s: &str) -> Self { self.0 += s; self }\n}",
            "the method then, which takes mut self",
        ),
        (
            "impl TrustedText {\n    pub fn grow(self: &mut Self, s: &str) {}\n}",
            "the method grow, which takes mut self",
        ),
        (
            "impl TrustedText {\n    pub fn grow(self: &mut TrustedText, s: &str) {}\n}",
            "the method grow, which takes mut self",
        ),
        (
            "impl TrustedText {\n    pub fn grow<'a>(&'a mut self, s: &str) {}\n}",
            "the method grow, which takes mut self",
        ),
        (
            "impl Clone for TrustedText {\n    fn clone_from(&mut self, r: &Self) {}\n}",
            "the method clone_from, which takes mut self",
        ),
        (
            "impl TrustedText {\n    pub fn push_str(&self, s: &str) {}\n}",
            "the method push_str",
        ),
        (
            "fn grow(t: &mut TrustedText, r: &str) {\n    more(t, r)\n}",
            "a &mut TrustedText",
        ),
        (
            "fn grow(t: & 'a mut restore_block::TrustedText) {}",
            "a &mut TrustedText",
        ),
        (
            "fn grow(mut t: TrustedText) -> TrustedText { t }",
            "a mut binding of TrustedText",
        ),
        ("fn grow(p: *mut TrustedText) {}", "a mut TrustedText"),
        (
            "impl TrustedText {\n    fn grow(other: &mut Self) {}\n}",
            "a &mut Self",
        ),
    ] {
        for inside in [true, false] {
            let got = problems(text, inside);

            assert_eq!(got.len(), 1, "{text:?} inside {inside}: {got:?}");
            assert!(
                got[0].contains(&format!("TrustedText is changed by {what}")),
                "{what}: {got:?}"
            );
        }
    }
}

#[test]
fn the_review_s_two_forms_are_refused() {
    // A custom trait whose method takes `&mut self`: refused for the
    // trait and for the receiver.
    let custom =
        "impl Grow for TrustedText {\n    fn grow(&mut self, r: &str) { self.0.push_str(r) }\n}";
    // A free function taking `&mut TrustedText`.
    let free = "fn grow(t: &mut TrustedText, r: &str) {\n    t.0.push_str(r)\n}";

    for inside in [true, false] {
        let got = problems(custom, inside);
        assert_eq!(got.len(), 2, "{got:?}");
        assert!(got[0].contains("implements the trait Grow"), "{got:?}");
        assert!(
            got[1].contains("the method grow, which takes mut self"),
            "{got:?}"
        );
        let got = problems(free, inside);
        assert!(
            got[0].starts_with("src/lib.rs:1: TrustedText is changed by a &mut TrustedText"),
            "{got:?}"
        );
    }
}

#[test]
fn only_the_traits_that_cannot_add_text_may_be_implemented() {
    let allowed = "\
impl Clone for TrustedText { fn clone(&self) -> Self { todo() } }
impl std::fmt::Debug for TrustedText {}
impl fmt::Display for TrustedText { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) } }
impl PartialEq for TrustedText {}
impl Eq for TrustedText {}
impl std::hash::Hash for TrustedText {}
impl PartialOrd for TrustedText {}
impl Ord for TrustedText {}
impl AsRef<str> for TrustedText { fn as_ref(&self) -> &str { &self.0 } }
impl std::ops::Deref for TrustedText { type Target = str; fn deref(&self) -> &str { &self.0 } }
";
    let clone_builds = problems(allowed, true);
    assert_eq!(clone_builds, Vec::<String>::new(), "{clone_builds:?}");

    for (text, name) in [
        (
            "impl std::ops::Add<&str> for TrustedText {\n    type Output = Self;\n}",
            "Add",
        ),
        ("impl AddAssign<String> for TrustedText {}", "AddAssign"),
        ("impl<'a> Extend<&'a str> for TrustedText {}", "Extend"),
        ("impl fmt::Write for TrustedText {}", "Write"),
        ("impl DerefMut for TrustedText {}", "DerefMut"),
        ("impl Drop for TrustedText {}", "Drop"),
        ("impl Iterator for TrustedText {}", "Iterator"),
        ("impl Grow for TrustedText {}", "Grow"),
        ("impl AsRef<[u8]> for TrustedText {}", "AsRef"),
        (
            "impl Deref for TrustedText { type Target = String; }",
            "Deref",
        ),
    ] {
        for inside in [true, false] {
            let got = problems(text, inside);

            assert_eq!(got.len(), 1, "{text:?} inside {inside}: {got:?}");
            assert!(got[0].starts_with("src/lib.rs:1: "), "{text:?}: {got:?}");
            assert!(
                got[0].contains(&format!(
                    "TrustedText implements the trait {name}, but only {ALLOWED_TRAITS_SHOWN} may be"
                )),
                "{name}: {got:?}"
            );
        }
    }
}

#[test]
fn inside_the_crate_its_field_is_read_only_where_it_cannot_change() {
    let allowed = "\
pub struct TrustedText(String);
impl TrustedText {
    pub fn join(parts: &[Self]) -> Self { Self(parts.iter().map(|p| p.0.as_str()).collect()) }
    pub fn len(&self) -> usize { self.0.len() }
}
fn version() -> f64 { 1.0 }
";
    assert_eq!(problems(allowed, true), Vec::<String>::new());

    for (text, how) in [
        (
            "pub fn grow(t: TrustedText, r: &str) -> usize {\n    t.0.len()\n}",
            "a field outside TrustedText's impl",
        ),
        (
            "impl TrustedText {\n    pub fn grown(self, r: &str) -> usize { self.0.len() }\n}",
            "its field in grown, which takes no &self",
        ),
        (
            "impl TrustedText {\n    pub fn grow(&self, r: &str) { let Self(ref mut s) = self; }\n}",
            "a mut binding in grow",
        ),
        (
            "impl TrustedText {\n    pub fn grow(self, r: &str) -> Self { let mut s = self; s }\n}",
            "a mut binding in grow",
        ),
        (
            "fn grow(t: TrustedText) {\n    let TrustedText(ref mut s) = t;\n}",
            "a ref mut binding outside TrustedText's impl",
        ),
    ] {
        let got = problems(text, true);

        assert_eq!(got.len(), 1, "{text:?}: {got:?}");
        assert!(got[0].starts_with("src/lib.rs:2: "), "{text:?}: {got:?}");
        assert!(got[0].contains(how), "{how}: {got:?}");
    }
    // Outside the crate the field is private: reading a tuple field is
    // how any other type is read.
    assert_eq!(
        problems("fn f(p: (u8, u8)) -> u8 { p.0 }", false),
        Vec::<String>::new()
    );
}

#[test]
fn inside_the_crate_trusted_text_is_one_private_field_of_immutable_text() {
    for allowed in [
        "pub struct TrustedText(String);",
        "pub struct TrustedText(Box<str>);",
        "pub struct TrustedText(std::sync::Arc<str>);",
        "pub struct TrustedText(Rc<str>);",
        "pub struct TrustedText(Cow<'static, str>);",
        "pub struct TrustedText(&'static str);",
        "#[derive(Clone)]\npub struct TrustedText(String);",
    ] {
        assert_eq!(problems(allowed, true), Vec::<String>::new(), "{allowed}");
    }
    for refused in [
        "pub struct TrustedText(pub String);",
        "pub struct TrustedText(pub(crate) String);",
        "pub struct TrustedText(RefCell<String>);",
        "pub struct TrustedText(String, String);",
        "pub struct TrustedText { text: String }",
        "pub struct TrustedText;",
    ] {
        let got = problems(refused, true);

        assert_eq!(got.len(), 1, "{refused}: {got:?}");
        assert!(
            got[0].contains(
                "TrustedText must be a tuple struct of one private String, Box<str>, Arc<str>, Rc<str>, Cow<'static, str> or &'static str"
            ),
            "{refused}: {got:?}"
        );
    }
}
