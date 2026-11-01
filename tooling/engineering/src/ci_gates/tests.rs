use std::fs;

use testkit::TempDir;

use super::*;

const LIST: &str = "# gates\n\nfmt|fast|-|ws|cargo fmt --all --check|=\nclippy|fast|-|pkgs|cargo clippy {pkgs} --locked|=\ntests|fast|-|pkgs|@job:test|cargo test {pkgs}\nsize|fast|-|ws|@workflow:review|@size\n";

const CI_TEXT: &str = "jobs:\n  test:\n    steps:\n  lint:\n    steps:\n      - run: scripts/ci-gate.sh fmt\n      - run: scripts/ci-gate.sh clippy\n";

fn exists(path: &str) -> bool {
    path == ".github/workflows/review.yml"
}

#[test]
fn parse_reads_each_gate_and_skips_comments_and_blank_lines() {
    let (gates, problems) = parse(LIST);

    assert!(problems.is_empty());
    assert_eq!(gates.len(), 4);
    assert_eq!(gates[1].name, "clippy");
    assert_eq!(gates[1].scope, "pkgs");
    assert_eq!(gates[2].local, "cargo test {pkgs}");
}

#[test]
fn parse_names_a_short_line_a_bad_field_and_a_repeat() {
    let (gates, problems) = parse(
        "fmt|fast|-|ws|cargo fmt\nodd|sometimes|-|ws|x|=\nodd|fast|-|nowhere|x|=\nempty|fast|-|ws||=\nok|fast|-|ws|x|=\nok|full|-|ws|y|=\n",
    );

    assert_eq!(gates.len(), 1);
    assert_eq!(
        problems,
        [
            "scripts/ci-gates.txt:1 is not name|fit|needs|scope|ci|local",
            "scripts/ci-gates.txt:2 gate \"odd\" has a fit, scope or empty field it may not have",
            "scripts/ci-gates.txt:3 gate \"odd\" has a fit, scope or empty field it may not have",
            "scripts/ci-gates.txt:4 gate \"empty\" has a fit, scope or empty field it may not have",
            "scripts/ci-gates.txt:6 gate ok is defined twice",
        ]
    );
}

#[test]
fn command_is_the_ci_field_with_the_workspace_for_pkgs_and_none_for_a_job() {
    let (gates, _) = parse(LIST);

    assert_eq!(
        gates[0].command().as_deref(),
        Some("cargo fmt --all --check")
    );
    assert_eq!(
        gates[1].command().as_deref(),
        Some("cargo clippy --workspace --locked")
    );
    assert_eq!(gates[2].command(), None);
}

#[test]
fn expand_replaces_each_call_with_the_command_and_keeps_the_rest() {
    let (gates, _) = parse(LIST);
    let ci = "      - run: scripts/ci-gate.sh clippy\n        run: scripts/ci-gate.sh fmt # note\n      - run: scripts/ci-gate.sh tests\n      - run: scripts/ci-gate.sh nothing\n      - run: cargo build\n";

    assert_eq!(
        expand(ci, &gates),
        "      - run: cargo clippy --workspace --locked\n        run: cargo fmt --all --check # note\n      - run: scripts/ci-gate.sh tests\n      - run: scripts/ci-gate.sh nothing\n      - run: cargo build\n"
    );
    assert_eq!(expand("a\nb", &gates), "a\nb");
}

#[test]
fn expand_handles_a_call_at_the_end_of_a_line_and_one_with_no_name() {
    let (gates, _) = parse(LIST);

    assert_eq!(
        expand("x: scripts/ci-gate.sh fmt", &gates),
        "x: cargo fmt --all --check"
    );
    assert_eq!(
        expand("x: scripts/ci-gate.sh ", &gates),
        "x: scripts/ci-gate.sh "
    );
}

#[test]
fn check_passes_when_every_gate_has_a_job_and_nothing_runs_inline() {
    assert_eq!(check(CI_TEXT, LIST, &exists), Vec::<String>::new());
}

#[test]
fn check_names_a_gate_with_no_job_a_missing_job_and_a_missing_workflow() {
    let ci = "jobs:\n  lint:\n    steps:\n      - run: scripts/ci-gate.sh fmt\n";
    let gone = |_: &str| false;

    assert_eq!(
        check(ci, LIST, &gone),
        [
            "gate clippy has no job: run scripts/ci-gate.sh clippy in .github/workflows/ci.yml",
            "gate tests names job test, which .github/workflows/ci.yml does not have",
            "gate size names workflow review, which does not exist",
        ]
    );
}

#[test]
fn check_names_a_call_of_no_gate_and_a_call_of_a_job_defined_gate() {
    let ci = format!(
        "{CI_TEXT}      - run: scripts/ci-gate.sh nothing\n      - run: scripts/ci-gate.sh tests\n      - run: scripts/ci-gate.sh \n"
    );

    assert_eq!(
        check(&ci, LIST, &exists),
        [
            "scripts/ci-gate.sh nothing",
            "scripts/ci-gate.sh tests",
            "scripts/ci-gate.sh ",
        ]
        .map(|call| format!(
            ".github/workflows/ci.yml calls {call}, which scripts/ci-gates.txt does not define as a command"
        ))
    );
}

#[test]
fn check_names_a_gate_s_command_written_inline() {
    let ci = format!(
        "{CI_TEXT}      - run: cargo fmt --all --check\n      - run: cargo clippy --workspace --locked\n      - run: cargo test\n"
    );

    assert_eq!(
        check(&ci, LIST, &exists),
        [
            ".github/workflows/ci.yml:8 runs the command of gate fmt inline: run scripts/ci-gate.sh fmt instead",
            ".github/workflows/ci.yml:9 runs the command of gate clippy inline: run scripts/ci-gate.sh clippy instead",
        ]
    );
}

#[test]
fn check_carries_the_list_s_own_problems() {
    assert_eq!(check(CI_TEXT, "junk\n", &exists).len(), 3);
}

fn checkout(ci: Option<&str>, gates: Option<&str>) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
    fs::create_dir_all(dir.path().join("scripts")).unwrap();
    fs::write(dir.path().join(".github/workflows/review.yml"), "").unwrap();
    for (path, text) in [(CI, ci), (GATES, gates)] {
        if let Some(text) = text {
            fs::write(dir.path().join(path), text).unwrap();
        }
    }
    let c = Checkout::new(dir.path());
    (dir, c)
}

#[test]
fn check_repository_reads_the_checkout_and_names_a_file_it_cannot_read() {
    let (_dir, c) = checkout(None, None);
    assert_eq!(check_repository(&c).len(), 2);

    let (_dir, c) = checkout(Some(CI_TEXT), Some(LIST));
    assert_eq!(check_repository(&c), Vec::<String>::new());
}

#[test]
fn a_checkout_reads_the_ci_workflow_as_the_commands_it_runs() {
    let (_dir, c) = checkout(Some(CI_TEXT), None);
    assert_eq!(
        c.read(CI).unwrap(),
        CI_TEXT,
        "with no list it reads as written"
    );

    let (_dir, c) = checkout(Some(CI_TEXT), Some(LIST));
    let read = c.read(CI).unwrap();
    assert!(read.contains("- run: cargo fmt --all --check\n"));
    assert_eq!(c.read_raw(CI).unwrap(), CI_TEXT);
    assert_eq!(c.read(GATES).unwrap(), LIST);
}
