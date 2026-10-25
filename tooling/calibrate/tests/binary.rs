//! The executable around the frozen workload, run as the scenario runs
//! it: it prints the workload's result and exits 0, and says why it
//! could not run when its workspace cannot be made.

use std::process::Command;

fn calibrate() -> Command {
    Command::new(env!("CARGO_BIN_EXE_calibrate"))
}

#[test]
fn it_prints_the_workload_s_result_and_exits_zero() {
    let out = calibrate().output().unwrap();

    let printed = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert_eq!(printed.trim().len(), 16, "{printed}");
    assert!(printed.trim().bytes().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn it_exits_non_zero_naming_itself_when_it_cannot_make_its_files() {
    let out = calibrate()
        .env("TMPDIR", "/no/such/directory/for/calibrate")
        .output()
        .unwrap();

    let said = String::from_utf8(out.stderr).unwrap();
    assert!(!out.status.success());
    assert!(said.starts_with("calibrate: "), "{said}");
}
