use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

use testkit::TempDir;

use super::*;

/// A stand-in for the Claude token; never a real one.
const TOKEN: &str = "dummy-access-token-0123456789-ABCDEFabcdef_xyz";

/// A token of the generic Claude shape, built at run time so this file
/// never holds one an agent could quote.
fn shaped() -> String {
    format!("sk-{}-oat01-{}", "ant", "Zz09_-".repeat(8))
}

#[test]
fn the_exact_token_is_replaced_and_counted() {
    let s = Scrubber::new(&[TOKEN]);

    let (text, n) = s.scrub(&format!("env: X={TOKEN}\nagain {TOKEN}."));

    assert_eq!(text, format!("env: X={REDACTED}\nagain {REDACTED}."));
    assert_eq!(n, 2);
}

#[test]
fn text_without_the_token_is_left_as_it_is() {
    let s = Scrubber::new(&[TOKEN]);

    assert_eq!(
        s.scrub("src/a.rs:3 — fine"),
        ("src/a.rs:3 — fine".to_owned(), 0)
    );
    assert_eq!(Scrubber::new(&[]).scrub("x"), ("x".to_owned(), 0));
}

#[test]
fn the_token_base64_encoded_at_any_offset_is_replaced() {
    let s = Scrubber::new(&[TOKEN]);
    for prefix in ["", "a", "ab", "abc"] {
        let raw = format!("{prefix}{TOKEN}!");
        for encoded in [
            encode(raw.as_bytes(), STANDARD),
            encode(raw.as_bytes(), URL_SAFE),
        ] {
            let (text, n) = s.scrub(&format!("blob {encoded} end"));
            assert_eq!(n, 1, "{prefix:?}: {encoded}");
            // Only the characters that hold a neighbouring byte's bits,
            // and the prefix's own, are left beside the replacement.
            let inner = text
                .strip_prefix("blob ")
                .and_then(|t| t.strip_suffix(" end"));
            let (head, tail) = inner.and_then(|t| t.split_once(REDACTED)).unwrap();
            assert!(encoded.starts_with(head) && head.len() <= 4, "{text}");
            assert!(encoded.ends_with(tail) && tail.len() <= 4, "{text}");
        }
    }
}

#[test]
fn the_token_percent_or_hex_encoded_is_replaced() {
    let odd = "tok/en+with=odd&chars-0123456789";
    let s = Scrubber::new(&[odd]);

    let (text, n) = s.scrub("q=tok%2Fen%2Bwith%3Dodd%26chars-0123456789&x");
    assert_eq!(text, format!("q={REDACTED}&x"));
    assert_eq!(n, 1);
    let hex: String = TOKEN.bytes().map(|b| format!("{b:02x}")).collect();
    let (text, n) = Scrubber::new(&[TOKEN]).scrub(&format!("[{hex}] [{}]", hex.to_uppercase()));
    assert_eq!(text, format!("[{REDACTED}] [{REDACTED}]"));
    assert_eq!(n, 2);
}

#[test]
fn the_token_percent_encoded_with_lowercase_hex_is_replaced() {
    let odd = "tok/en+with=odd&chars-0123456789";
    let s = Scrubber::new(&[odd]);

    let (text, n) = s.scrub("q=tok%2fen%2bwith%3dodd%26chars-0123456789&x");

    assert_eq!(text, format!("q={REDACTED}&x"));
    assert_eq!(n, 1);
}

#[test]
fn any_token_of_the_claude_shape_is_replaced_even_unknown() {
    let s = Scrubber::new(&[]);

    let (text, n) = s.scrub(&format!("leak: \"{}\".", shaped()));

    assert_eq!(text, format!("leak: \"{REDACTED}\"."));
    assert_eq!(n, 1);
    // The shape named in prose, too short to be a token, stays.
    let prose = format!("a token starts sk-{}-oat01-…", "ant");
    assert_eq!(s.scrub(&prose), (prose.clone(), 0));
}

/// Another token of that shape, which no tree holds.
fn unknown() -> String {
    format!("sk-{}-api03-{}", "ant", "Qq17_-".repeat(8))
}

#[test]
fn a_key_shaped_text_the_tree_holds_is_replaced_and_is_no_leak() {
    let s = Scrubber::new(&[]).sparing(BTreeSet::from([shaped()]));

    let (text, n) = s.scrub(&format!("a.rs:3: \"{}\", then \"{}\"", shaped(), unknown()));

    // The tree's own text is no token of the review's: it is replaced,
    // so nothing kept holds the shape, and only the unknown one counts.
    assert_eq!(
        text,
        format!("a.rs:3: \"{TREE_TEXT}\", then \"{REDACTED}\"")
    );
    assert_eq!(n, 1);
    // A longer run of token characters is another text than the tree's.
    let longer = format!("{}x", shaped());
    assert_eq!(s.scrub(&longer), (REDACTED.to_owned(), 1));
}

#[test]
fn the_review_s_own_token_is_a_leak_even_where_a_tree_holds_it() {
    let s = Scrubber::new(&[&shaped()]).sparing(BTreeSet::from([shaped()]));

    assert_eq!(s.scrub(&shaped()), (REDACTED.to_owned(), 1));
}

#[test]
fn the_key_shaped_texts_under_a_tree_are_collected() {
    let dir = TempDir::new().unwrap();
    let write = |rel: &str, body: String| {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    write(
        "crates/a/src/tests.rs",
        format!("let key = \"{}\";\n", shaped()),
    );
    write("notes.md", format!("a token starts sk-{}-oat01-…\n", "ant"));
    // What a build or git wrote is no text of the tree.
    write("target/debug/x", unknown());
    write(".git/objects/y", unknown());
    let mut bytes = vec![0xff, 0xfe];
    bytes.extend_from_slice(format!("{}z", shaped()).as_bytes());
    fs::write(dir.path().join("bin"), bytes).unwrap();

    // Nor is what a link of the tree points at: a link is never followed,
    // so a file outside the tree stays outside.
    let outside = TempDir::new().unwrap();
    let elsewhere = format!("{}y", shaped());
    fs::write(outside.path().join("key"), &elsewhere).unwrap();
    std::os::unix::fs::symlink(outside.path(), dir.path().join("linked-dir")).unwrap();
    std::os::unix::fs::symlink(outside.path().join("key"), dir.path().join("linked-file")).unwrap();
    // Nor is the staged credential, which holds tokens that are no fixture.
    write("cred/.credentials.json", unknown());
    let cred = fs::canonicalize(dir.path().join("cred")).unwrap();

    assert_eq!(
        shaped_under(dir.path(), Some(&cred)),
        Ok(BTreeSet::from([shaped(), format!("{}z", shaped())]))
    );
    assert_eq!(
        shaped_under(dir.path(), None).map(|found| found.contains(&unknown())),
        Ok(true)
    );
    assert_eq!(
        shaped_under(dir.path(), None).map(|found| found.contains(&elsewhere)),
        Ok(false)
    );
    let err = shaped_under(&dir.path().join("missing"), None).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
    assert_eq!(
        unread(Path::new("a/b"), &io::Error::other("gone")),
        "read a/b: gone"
    );
}

#[test]
fn a_file_holding_only_the_tree_s_text_is_rewritten_and_is_no_leak() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("log.jsonl");
    fs::write(&path, format!("{{\"text\":\"{}\"}}\n", shaped())).unwrap();
    let s = Scrubber::new(&[TOKEN]).sparing(BTreeSet::from([shaped()]));

    assert_eq!(s.scrub_tree(dir.path()).unwrap(), 0);

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        format!("{{\"text\":\"{TREE_TEXT}\"}}\n")
    );
    // A second scrub that knows no tree finds nothing left to take for a token.
    assert_eq!(Scrubber::new(&[TOKEN]).scrub_tree(dir.path()).unwrap(), 0);
}

#[test]
fn a_secret_too_short_to_tell_from_text_is_not_a_form() {
    let s = Scrubber::new(&["", "  ", "short"]);

    assert_eq!(s.scrub("a short note"), ("a short note".to_owned(), 0));
}

#[test]
fn a_guarded_scrubber_refuses_a_secret_too_short_to_be_a_form() {
    let err = Scrubber::guarded(&[TOKEN, "short"]).unwrap_err();

    assert_eq!(
        err,
        format!(
            "the Claude token is shorter than the {SHORTEST} characters the scrub can match; \
             refusing to use it"
        )
    );
    // An empty secret is none at all: there is nothing of it to leak.
    assert_eq!(
        Scrubber::guarded(&[TOKEN, "", "  "]),
        Ok(Scrubber::new(&[TOKEN]))
    );
}

#[test]
fn a_file_is_rewritten_only_when_it_held_the_token() {
    let dir = TempDir::new().unwrap();
    let clean = dir.path().join("clean.md");
    let leaky = dir.path().join("sub/leaky.jsonl");
    fs::create_dir_all(leaky.parent().unwrap()).unwrap();
    fs::write(&clean, "nothing here\n").unwrap();
    fs::write(&leaky, format!("{{\"text\":\"{TOKEN}\"}}\n")).unwrap();
    let s = Scrubber::new(&[TOKEN]);

    assert_eq!(s.scrub_file(&clean).unwrap(), 0);
    assert_eq!(s.scrub_tree(dir.path()).unwrap(), 1);

    assert_eq!(fs::read_to_string(&clean).unwrap(), "nothing here\n");
    assert_eq!(
        fs::read_to_string(&leaky).unwrap(),
        format!("{{\"text\":\"{REDACTED}\"}}\n")
    );
    assert_eq!(s.scrub_tree(&dir.path().join("missing")).unwrap(), 0);
    assert_eq!(s.scrub_file(&dir.path().join("missing")).unwrap(), 0);
}

#[test]
fn a_file_that_is_not_text_is_still_scrubbed() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bin");
    let mut bytes = vec![0xff, 0xfe];
    bytes.extend_from_slice(TOKEN.as_bytes());
    fs::write(&path, &bytes).unwrap();

    assert_eq!(Scrubber::new(&[TOKEN]).scrub_file(&path).unwrap(), 1);

    let after = fs::read(&path).unwrap();
    assert!(!String::from_utf8_lossy(&after).contains(TOKEN));
}

#[test]
fn a_file_or_directory_that_cannot_be_read_fails_naming_it() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("f");
    fs::write(&file, "x").unwrap();
    let s = Scrubber::new(&[TOKEN]);

    let err = s.scrub_tree(&file.join("under")).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
    let err = s.scrub_file(dir.path()).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
}

#[test]
fn a_file_that_cannot_be_written_back_fails_naming_it() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("ro");
    fs::write(&path, TOKEN).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    // Root writes through any mode; the case needs a user it binds.
    if fs::OpenOptions::new().write(true).open(&path).is_ok() {
        return;
    }

    let err = Scrubber::new(&[TOKEN]).scrub_file(&path).unwrap_err();
    assert!(err.starts_with("write "), "{err}");
}

#[test]
fn base64_encodes_as_the_standard_says() {
    assert_eq!(encode(b"", STANDARD), "");
    assert_eq!(encode(b"f", STANDARD), "Zg");
    assert_eq!(encode(b"fo", STANDARD), "Zm8");
    assert_eq!(encode(b"foo", STANDARD), "Zm9v");
    assert_eq!(encode(&[0xfb, 0xff], STANDARD), "+/8");
    assert_eq!(encode(&[0xfb, 0xff], URL_SAFE), "-_8");
}

#[test]
fn a_scrubber_says_how_many_texts_of_the_trees_it_spares() {
    assert_eq!(Scrubber::new(&[TOKEN]).spares(), None);
    let one = Scrubber::new(&[]).sparing(BTreeSet::from([shaped()]));
    assert_eq!(
        one.spares(),
        Some(format!(
            "the scrub spares 1 key-shaped text the trees hold: \
             replaced with {TREE_TEXT}, no leak"
        ))
    );
    let two = Scrubber::new(&[]).sparing(BTreeSet::from([shaped(), unknown()]));
    assert_eq!(
        two.spares(),
        Some(format!(
            "the scrub spares 2 key-shaped texts the trees hold: \
             replaced with {TREE_TEXT}, no leak"
        ))
    );
}
