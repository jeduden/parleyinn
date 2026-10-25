use std::env;
use std::fs;
use std::process::Command;

use super::*;
use crate::TempDir;

/// The variables of a process whose environment is `pairs`.
fn vars(pairs: &[(&str, &Path)]) -> impl Fn(&str) -> Option<OsString> + use<> {
    let pairs: Vec<(String, PathBuf)> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.to_path_buf()))
        .collect();
    move |name| {
        pairs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone().into_os_string())
    }
}

fn refused(place: Place, var: Var<'_>, temp: &Path) -> Refusal {
    check(place, var, temp).unwrap_err()
}

#[test]
fn a_home_under_the_temporary_directory_resolves() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");

    let got = check(Place::UserHome, &vars(&[("HOME", &home)]), temp.path());

    assert_eq!(got, Ok(home));
}

#[test]
fn a_real_home_is_refused_with_the_path_it_would_have_reached() {
    let temp = TempDir::new().unwrap();
    let real = Path::new("/home/someone");

    let got = refused(Place::UserHome, &vars(&[("HOME", real)]), temp.path());

    assert_eq!(
        got,
        Refusal::Outside {
            place: Place::UserHome,
            path: real.to_path_buf(),
            temp: temp.path().to_path_buf()
        }
    );
    assert_eq!(
        got.to_string(),
        format!(
            "refusing to resolve the user's `HOME` directory at /home/someone: it is not under the temporary directory {}; point HOME and PARINN_HOME at a testkit::TempDir first (ENG-14)",
            temp.path().display()
        )
    );
}

#[test]
fn an_unset_home_is_refused() {
    let temp = TempDir::new().unwrap();

    for place in Place::ALL {
        assert_eq!(refused(place, &vars(&[]), temp.path()), Refusal::Unset);
    }
    assert_eq!(
        Refusal::Unset.to_string(),
        "refusing to resolve a path: HOME is not set (ENG-14)"
    );
}

#[test]
fn the_temporary_directory_itself_is_no_home() {
    let temp = TempDir::new().unwrap();

    let got = refused(
        Place::UserHome,
        &vars(&[("HOME", temp.path())]),
        temp.path(),
    );

    assert!(matches!(got, Refusal::Outside { .. }));
}

#[test]
fn a_relative_home_is_refused() {
    let temp = TempDir::new().unwrap();

    let got = refused(
        Place::UserHome,
        &vars(&[("HOME", Path::new("home"))]),
        temp.path(),
    );

    assert!(matches!(got, Refusal::Outside { .. }));
}

#[test]
fn a_home_that_climbs_out_of_the_temporary_directory_is_refused() {
    let temp = TempDir::new().unwrap();
    let sneaky = temp.path().join("a/../../elsewhere");

    let got = refused(Place::UserHome, &vars(&[("HOME", &sneaky)]), temp.path());

    assert!(matches!(got, Refusal::Outside { .. }));
}

#[cfg(unix)]
#[test]
fn a_home_that_is_a_symlink_out_of_the_temporary_directory_is_refused() {
    let (temp, outside) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let link = temp.path().join("home");
    std::os::unix::fs::symlink(outside.path(), &link).unwrap();

    let at_link = refused(Place::UserHome, &vars(&[("HOME", &link)]), temp.path());
    let below = link.join("not/yet/there");
    let below_link = refused(Place::UserHome, &vars(&[("HOME", &below)]), temp.path());

    assert!(matches!(at_link, Refusal::Outside { .. }));
    assert!(matches!(below_link, Refusal::Outside { .. }));
}

#[test]
fn a_home_that_does_not_exist_yet_resolves_inside_the_temporary_directory() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("not/yet/there");

    let got = check(Place::UserHome, &vars(&[("HOME", &home)]), temp.path());

    assert_eq!(got, Ok(home));
}

#[test]
fn the_parinn_home_defaults_to_dot_parinn_in_the_home() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");

    let got = check(Place::ParleyInnHome, &vars(&[("HOME", &home)]), temp.path());

    assert_eq!(got, Ok(home.join(".parinn")));
}

#[test]
fn an_explicit_real_parinn_home_is_refused_beside_an_isolated_home() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");
    let real = Path::new("/home/x/.parinn");
    let var = vars(&[("HOME", &home), ("PARINN_HOME", real)]);

    let got = refused(Place::ParleyInnHome, &var, temp.path());

    assert!(matches!(
        got,
        Refusal::Outside {
            place: Place::ParleyInnHome,
            ..
        }
    ));
}

#[test]
fn an_explicit_isolated_parinn_home_resolves_beside_a_real_home() {
    let temp = TempDir::new().unwrap();
    let parinn = temp.path().join("parinn");
    let var = vars(&[("HOME", Path::new("/home/x")), ("PARINN_HOME", &parinn)]);

    assert_eq!(check(Place::ParleyInnHome, &var, temp.path()), Ok(parinn));
}

#[test]
fn claude_code_s_configuration_defaults_to_the_home() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");
    let var = vars(&[("HOME", &home)]);

    assert_eq!(
        check(Place::ClaudeConfig, &var, temp.path()),
        Ok(home.join(".claude"))
    );
    assert_eq!(
        check(Place::ClaudeJson, &var, temp.path()),
        Ok(home.join(".claude.json"))
    );
}

#[test]
fn claude_code_s_configuration_directory_overrides_the_home() {
    let temp = TempDir::new().unwrap();
    let config = temp.path().join("config");
    let var = vars(&[
        ("HOME", Path::new("/home/x")),
        ("CLAUDE_CONFIG_DIR", &config),
    ]);

    assert_eq!(
        check(Place::ClaudeConfig, &var, temp.path()),
        Ok(config.clone())
    );
    assert_eq!(
        check(Place::ClaudeJson, &var, temp.path()),
        Ok(config.join(".claude.json"))
    );
}

#[test]
fn a_real_claude_code_configuration_is_refused_under_an_isolated_home() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");
    let real = Path::new("/home/x/.claude");
    let var = vars(&[("HOME", &home), ("CLAUDE_CONFIG_DIR", real)]);

    for place in [Place::ClaudeConfig, Place::ClaudeJson] {
        assert!(matches!(
            refused(place, &var, temp.path()),
            Refusal::Outside { .. }
        ));
    }
}

#[test]
fn a_refusal_says_which_place_it_kept_out() {
    let temp = TempDir::new().unwrap();
    let real = Path::new("/home/x");
    let var = vars(&[("HOME", real)]);

    for (place, what) in [
        (
            Place::ParleyInnHome,
            "the ParleyInn home at /home/x/.parinn",
        ),
        (
            Place::ClaudeConfig,
            "Claude Code's configuration directory at /home/x/.claude",
        ),
        (
            Place::ClaudeJson,
            "Claude Code's state file at /home/x/.claude.json",
        ),
    ] {
        let got = refused(place, &var, temp.path()).to_string();

        assert!(
            got.starts_with(&format!("refusing to resolve {what}: ")),
            "{got}"
        );
    }
}

#[test]
fn a_path_with_no_existing_ancestor_resolves_as_written() {
    let path = Path::new("no-such-dir-here/below");

    assert_eq!(resolved(path), path);
}

#[test]
fn a_place_is_named_and_parsed_both_ways() {
    for place in Place::ALL {
        assert_eq!(Place::parse(place.name()), Some(place));
    }
    assert_eq!(Place::parse("elsewhere"), None);
}

fn abort_by_panic() -> ! {
    panic!("aborted")
}

#[test]
fn a_refusal_is_reported_and_aborts() {
    let temp = TempDir::new().unwrap();
    let mut report = Vec::new();
    let var = vars(&[("HOME", Path::new("/home/x"))]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        check_or_abort(
            Place::UserHome,
            &var,
            temp.path(),
            &mut report,
            abort_by_panic,
        )
    }));

    assert!(result.is_err(), "the guard returned a path");
    assert!(String::from_utf8(report).unwrap().contains("/home/x"));
}

#[test]
fn an_isolated_place_is_returned_without_a_report() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");
    let mut report = Vec::new();

    let got = check_or_abort(
        Place::UserHome,
        &vars(&[("HOME", &home)]),
        temp.path(),
        &mut report,
        abort_by_panic,
    );

    assert_eq!(got, home);
    assert!(report.is_empty());
}

/// Set on the process the test below starts: its value is the home that
/// process must resolve.
const CHILD: &str = "PARINN_TESTKIT_GUARD_CHILD";

/// `resolve` reads this process's environment. The test starts itself
/// with an isolated one and has the child resolve every place.
#[cfg(unix)]
#[test]
fn resolve_reads_the_environment_of_the_process() {
    if let Some(home) = env::var_os(CHILD) {
        let home = PathBuf::from(home);
        assert_eq!(resolve(Place::UserHome), home);
        assert_eq!(resolve(Place::ParleyInnHome), home.join(".parinn"));
        assert_eq!(resolve(Place::ClaudeConfig), home.join(".claude"));
        assert_eq!(resolve(Place::ClaudeJson), home.join(".claude.json"));
        return;
    }
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");
    fs::create_dir(&home).unwrap();

    let out = Command::new(env::current_exe().unwrap())
        .args([
            "--exact",
            "guard::tests::resolve_reads_the_environment_of_the_process",
        ])
        .env_clear()
        .env("HOME", &home)
        .env("TMPDIR", temp.path())
        .env(CHILD, &home)
        // A coverage run measures the child too, if it can find where
        // to write its profile.
        .envs(env::var_os("LLVM_PROFILE_FILE").map(|file| ("LLVM_PROFILE_FILE", file)))
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
