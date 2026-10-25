use super::*;

#[test]
fn a_temp_dir_exists_until_dropped() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_path_buf();
    fs::write(path.join("a"), "x").unwrap();

    assert!(path.is_dir());
    drop(dir);
    assert!(!path.exists());
}

#[test]
fn two_temp_dirs_never_share_a_path() {
    let (a, b) = (TempDir::new().unwrap(), TempDir::new().unwrap());

    assert_ne!(a.path(), b.path());
}

#[cfg(unix)]
#[test]
fn a_temp_dir_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();

    assert_eq!(
        fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777,
        0o700
    );
}

#[test]
fn create_unique_skips_a_name_that_exists() {
    let base = TempDir::new().unwrap();
    fs::create_dir(base.path().join("a")).unwrap();

    let got = create_unique(base.path(), &mut ["a".into(), "b".into()].into_iter()).unwrap();

    assert_eq!(got, base.path().join("b"));
}

#[test]
fn create_unique_reports_running_out_of_names() {
    let base = TempDir::new().unwrap();
    fs::create_dir(base.path().join("a")).unwrap();

    let err = create_unique(base.path(), &mut ["a".into()].into_iter()).unwrap_err();

    assert!(err.to_string().contains("no free directory name"), "{err}");
}

#[test]
fn create_unique_passes_other_errors_on() {
    let base = TempDir::new().unwrap();

    let err =
        create_unique(&base.path().join("missing"), &mut ["a".into()].into_iter()).unwrap_err();

    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}

#[test]
fn isolate_points_home_parinn_home_and_the_working_directory_at_the_directory() {
    let mut command = Command::new("x");

    isolate(&mut command, Path::new("/tmp/h"));

    let envs: Vec<(String, String)> = command
        .get_envs()
        .map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.unwrap().to_string_lossy().into_owned(),
            )
        })
        .collect();
    assert_eq!(
        envs,
        [
            ("HOME".into(), "/tmp/h".into()),
            ("PARINN_HOME".into(), "/tmp/h/.parinn".into())
        ]
    );
    assert_eq!(command.get_current_dir(), Some(Path::new("/tmp/h")));
}

#[test]
fn repo_root_is_two_levels_above_the_crate() {
    assert_eq!(
        repo_root(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    );
}

#[test]
fn an_executable_is_found_beside_the_running_test_s_deps_directory() {
    let dir = TempDir::new().unwrap();
    let deps = dir.path().join("debug/deps");
    fs::create_dir_all(&deps).unwrap();
    let test = deps.join("bdd-0123");
    let exe = dir
        .path()
        .join(format!("debug/parinn{}", std::env::consts::EXE_SUFFIX));

    let missing = sibling_executable(&test, "parinn").unwrap_err();
    fs::write(&exe, "").unwrap();
    let found = sibling_executable(&test, "parinn").unwrap();

    assert_eq!(found, exe);
    assert_eq!(
        missing,
        format!(
            "no parinn executable at {}: build it first, with cargo build -p parinn",
            exe.display()
        )
    );
    assert_eq!(
        sibling_executable(Path::new("bdd"), "parinn").unwrap_err(),
        "the running test bdd lies in no deps directory"
    );
}

#[test]
fn core_executable_looks_beside_the_running_test() {
    let test = std::env::current_exe().unwrap();

    assert_eq!(core_executable(), sibling_executable(&test, "parinn"));
}

#[test]
fn place_transcript_copies_under_the_default_transcript_root() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("src").join("-w-app");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("s.jsonl"), "{}\n").unwrap();
    let home = dir.path().join("home");

    let placed = place_transcript(&home, &source.join("s.jsonl")).unwrap();

    assert_eq!(placed, home.join(".claude/projects/-w-app/s.jsonl"));
    assert_eq!(std::fs::read_to_string(&placed).unwrap(), "{}\n");
    assert!(place_transcript(&home, Path::new("/")).is_err());
    assert!(place_transcript(&home, &source.join("missing.jsonl")).is_err());
    let top = place_transcript(&home, Path::new("relative.jsonl"));
    assert!(top.is_err());
}

#[test]
fn the_release_executable_is_the_named_path_else_the_checkout_s_release_build() {
    let dir = TempDir::new().unwrap();
    let exe = format!("parinn{}", std::env::consts::EXE_SUFFIX);
    let built = dir.path().join("target/release").join(&exe);
    let named = dir.path().join("elsewhere").join(&exe);
    fs::create_dir_all(built.parent().unwrap()).unwrap();
    fs::create_dir_all(named.parent().unwrap()).unwrap();

    let missing = release_executable(None, dir.path(), "parinn").unwrap_err();
    fs::write(&built, "").unwrap();
    let found = release_executable(None, dir.path(), "parinn").unwrap();
    let unnamed = release_executable(Some(named.clone().into()), dir.path(), "parinn").unwrap_err();
    fs::write(&named, "").unwrap();
    let chosen = release_executable(Some(named.clone().into()), dir.path(), "parinn").unwrap();

    assert_eq!(
        missing,
        format!(
            "no release parinn executable at {}: build it first, with cargo build --release -p parinn, or name it in PARINN_RELEASE_CORE",
            built.display()
        )
    );
    assert_eq!(found, built);
    assert!(unnamed.contains(&named.display().to_string()), "{unnamed}");
    assert_eq!(chosen, named);
}

#[test]
fn the_calibration_executable_is_found_as_the_release_core_is() {
    let base = TempDir::new().unwrap();
    let named = base.path().join("calibrate-here");
    std::fs::write(&named, "").unwrap();

    let found = release_executable(
        Some(named.clone().into_os_string()),
        base.path(),
        "calibrate",
    );
    let missing = release_executable(None, base.path(), "calibrate").unwrap_err();

    assert_eq!(found, Ok(named));
    assert!(
        missing.contains("cargo build --release -p calibrate"),
        "{missing}"
    );
    assert!(missing.contains(CALIBRATE_VAR), "{missing}");
    assert_eq!(CALIBRATE_VAR, "PARINN_CALIBRATE");
    match calibrate_executable() {
        Ok(path) => assert!(path.is_file()),
        Err(e) => assert!(e.contains("cargo build --release -p calibrate"), "{e}"),
    }
}

#[test]
fn the_release_core_executable_reads_the_variable_and_the_checkout() {
    // The unit run builds no release core, so the answer is a file or
    // the error that says how to build one.
    match release_core_executable() {
        Ok(path) => assert!(path.is_file()),
        Err(e) => assert!(e.contains("cargo build --release -p parinn"), "{e}"),
    }
}
