use std::fs;

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
