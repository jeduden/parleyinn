use testkit::TempDir;

use super::*;

#[test]
fn the_text_names_what_was_spent_and_reads_back() {
    assert_eq!(text(1.25), "{\"spent_usd\": 1.25}\n");
    assert_eq!(text(0.0), "{\"spent_usd\": 0}\n");

    assert_eq!(check(&text(4.995)), Ok(4.995));
    assert_eq!(check(&text(0.0)), Ok(0.0));
}

#[test]
fn a_spend_file_past_the_size_limit_is_named() {
    let big = format!("{{\"spent_usd\": 1{}}}", " ".repeat(MAX_BYTES));

    assert_eq!(
        check(&big),
        Err(format!(
            "the spend file is {} bytes, past the 1024 a review reads",
            big.len()
        ))
    );
    let exact = format!("{{\"spent_usd\": 1}}{}", " ".repeat(MAX_BYTES - 16));
    assert_eq!(exact.len(), MAX_BYTES);
    assert_eq!(check(&exact), Ok(1.0));
}

#[test]
fn a_spend_file_that_does_not_read_is_named_never_taken_for_nothing() {
    for bad in [
        "",
        "{",
        "[]",
        "{}",
        r#"{"spent_usd":"1"}"#,
        r#"{"spent_usd":1,"x":2}"#,
    ] {
        let err = check(bad).unwrap_err();
        assert!(
            err.starts_with("the spend file does not read: "),
            "{bad}: {err}"
        );
    }
    assert_eq!(
        check(r#"{"spent_usd": -0.5}"#),
        Err("the spend file names -0.5 USD, no amount of zero or more".to_owned())
    );
    assert!(check(r#"{"spent_usd": 1e999}"#).is_err());
}

#[test]
fn the_file_is_written_whole_and_written_over() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("spend.json");

    write(&path, &text(1.0)).unwrap();
    write(&path, &text(2.0)).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), text(2.0));
    assert!(!dir.path().join("spend.json.part").exists());
}

#[test]
fn a_file_that_cannot_be_written_or_moved_is_named() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("spend.json");
    fs::create_dir(dir.path().join("spend.json.part")).unwrap();
    let err = write(&path, "x").unwrap_err();
    assert!(
        err.starts_with("write ") && err.contains("spend.json.part"),
        "{err}"
    );

    let other = dir.path().join("taken.json");
    fs::create_dir_all(other.join("full")).unwrap();
    let err = write(&other, "x").unwrap_err();
    assert!(
        err.starts_with("rename ") && err.contains("taken.json"),
        "{err}"
    );
}
