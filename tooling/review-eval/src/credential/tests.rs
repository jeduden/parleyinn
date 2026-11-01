use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Mutex;

use testkit::TempDir;

use super::*;
use crate::host::Output;

/// A stand-in for the access token; never a real one.
const TOKEN: &str = "dummy-access-token-0123456789";

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn a_staged_token_is_read_back_from_a_file_only_its_owner_reads() {
    let root = TempDir::new().unwrap();
    let dir = root.path().join("cred");

    stage(&dir, &format!("{TOKEN}\n")).unwrap();

    assert_eq!(mode(&dir), 0o700);
    assert_eq!(mode(&dir.join(FILE)), 0o600);
    assert_eq!(read(&dir).unwrap(), TOKEN);
    let body: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join(FILE)).unwrap()).unwrap();
    assert_eq!(body["claudeAiOauth"]["accessToken"], TOKEN);
    assert_eq!(
        body["claudeAiOauth"]["refreshToken"],
        serde_json::Value::Null
    );
    assert_eq!(body["claudeAiOauth"]["expiresAt"], serde_json::Value::Null);
    assert_eq!(body["claudeAiOauth"]["scopes"][0], "user:inference");

    // Staged again, it replaces the first.
    stage(&dir, "dummy-second-token-0123").unwrap();
    assert_eq!(read(&dir).unwrap(), "dummy-second-token-0123");
}

#[test]
fn a_token_that_is_empty_or_not_one_line_of_token_characters_is_refused_unnamed() {
    let root = TempDir::new().unwrap();
    for bad in ["", "  \n", "two words", "a\"quote", "line\nbreak"] {
        let err = stage(&root.path().join("x"), bad).unwrap_err();
        assert_eq!(
            err, "the access token is empty or holds a character no token has",
            "{bad:?}"
        );
    }
}

#[test]
fn a_credential_that_cannot_be_staged_or_read_fails_naming_its_path() {
    let root = TempDir::new().unwrap();
    let file = root.path().join("f");
    fs::write(&file, "x").unwrap();

    let err = stage(&file.join("under"), TOKEN).unwrap_err();
    assert!(
        err.starts_with(&format!("make {}", file.join("under").display())),
        "{err}"
    );
    let err = read(&root.path().join("missing")).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
    let bad = root.path().join("bad");
    fs::create_dir(&bad).unwrap();
    fs::write(bad.join(FILE), "{}").unwrap();
    assert_eq!(
        read(&bad).unwrap_err(),
        format!("{} holds no access token", bad.join(FILE).display())
    );
    fs::write(bad.join(FILE), "nope").unwrap();
    assert!(read(&bad).unwrap_err().contains("is not JSON"));
    // A directory where the file goes cannot be written.
    let blocked = root.path().join("blocked");
    fs::create_dir_all(blocked.join(FILE)).unwrap();
    let err = stage(&blocked, TOKEN).unwrap_err();
    assert!(err.starts_with("write "), "{err}");
}

#[test]
fn the_error_never_holds_the_token() {
    let root = TempDir::new().unwrap();
    let bad = root.path().join("bad");
    fs::create_dir(&bad).unwrap();
    fs::write(bad.join(FILE), format!("{{\"x\":\"{TOKEN}\"")).unwrap();

    let err = read(&bad).unwrap_err();

    assert!(!err.contains(TOKEN), "{err}");
}

struct Recording {
    seen: Mutex<Vec<Command>>,
}

impl Host for Recording {
    fn run(&self, command: &Command) -> Result<Output, String> {
        self.seen.lock().unwrap().push(command.clone());
        Ok(Output::default())
    }
}

fn claude() -> Command {
    Command::new(
        "claude",
        &["-p", "x", "--disallowedTools", "Bash,Edit"],
        Path::new("/work"),
    )
}

#[test]
fn a_confined_claude_reads_the_staged_config_and_may_not_read_it() {
    let dir = PathBuf::from("/run/cred");
    let inner = Recording {
        seen: Mutex::new(Vec::new()),
    };
    let host = Confined::new(&inner, &dir);

    host.run(&claude()).unwrap();
    host.run(&Command::new("git", &["status"], Path::new("/work")))
        .unwrap();

    let seen = inner.seen.lock().unwrap();
    assert_eq!(
        seen[0].env,
        [(CONFIG_VAR.to_owned(), "/run/cred".to_owned())]
    );
    assert_eq!(
        seen[0].args,
        [
            "-p",
            "x",
            "--disallowedTools",
            "Bash,Edit,Read(//run/cred/**),Read(//run/cred)"
        ]
    );
    assert_eq!(
        seen[1],
        Command::new("git", &["status"], Path::new("/work"))
    );
}

#[test]
fn a_claude_with_no_refused_tools_gets_the_rule_as_its_own() {
    let mut command = Command::new("claude", &["-p", "x"], Path::new("/w"));

    confine(&mut command, Path::new("/c"));

    assert_eq!(
        command.args,
        ["-p", "x", "--disallowedTools", "Read(//c/**),Read(//c)"]
    );
}
