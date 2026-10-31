use std::fs;

use super::*;
use crate::review_workflow::parse;
use testkit::TempDir;

/// A checkout whose workspace holds the crates `names`, one directory
/// each under `crates/`.
fn checkout(names: &[&str]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();
    for name in names {
        let at = dir.path().join("crates").join(name);
        fs::create_dir_all(&at).unwrap();
        fs::write(
            at.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\n"),
        )
        .unwrap();
    }
    let c = Checkout::new(dir.path());
    (dir, c)
}

/// A CI workflow whose test job lists `matrix` shards and the package
/// `lists`, each a line of the shards array.
fn workflow(matrix: &str, lists: &[&str]) -> Workflow {
    let lists: String = lists.iter().map(|l| format!("            {l}\n")).collect();
    parse(&format!(
        "jobs:\n  test:\n    strategy:\n      matrix:\n        shard: [{matrix}]\n    env:\n      {}\n      {}\n    steps:\n      - run: |\n          shards=(\n{lists}          )\n          {} echo bad; exit 1; fi\n          {}\n          HOME=\"$decoy\" {SHARD_COMMAND}\n",
        SELECTION[0], SELECTION[1], SELECTION[2], SELECTION[3],
    ))
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

const CRATES: [&str; 4] = ["a", "b", "c", SCENARIO];

#[test]
fn shards_that_run_every_crate_once_pass() {
    let (_dir, c) = checkout(&CRATES);
    let flow = workflow(
        "1, 2",
        &[
            "\"-p a\"",
            "\"--workspace --exclude scenario \\",
            "  --exclude a\"",
        ],
    );

    assert!(
        shards_cover_the_workspace(&c, &flow).is_empty(),
        "{:?}",
        shards_cover_the_workspace(&c, &flow)
    );
}

#[test]
fn a_crate_added_to_the_workspace_and_to_no_shard_fails() {
    let (_dir, c) = checkout(&["a", "b", "new", SCENARIO]);
    let flow = workflow("1, 2", &["\"-p a\"", "\"-p b\""]);

    assert_eq!(
        shards_cover_the_workspace(&c, &flow),
        [
            "the test job's shards: crate new runs in no shard: add -p new to one in .github/workflows/ci.yml"
        ]
    );
}

#[test]
fn a_crate_in_two_shards_or_the_scenario_crate_in_one_fails() {
    let packages = names(&CRATES);
    let shards = [names(&["a", "b", "c"]), names(&["b", SCENARIO])];

    assert_eq!(
        coverage(&shards, &packages),
        [
            "the test job's shards: crate b runs in shards 1 and 2: keep it in one in .github/workflows/ci.yml",
            "the test job's shards: shard 2 runs the scenario crate, which the scenarios job runs: exclude it in .github/workflows/ci.yml",
        ]
    );
}

#[test]
fn a_shard_list_selects_named_packages_or_the_workspace_less_its_excludes() {
    let packages = names(&CRATES);

    assert_eq!(
        selected("-p b --package a -p b", &packages),
        Ok(names(&["a", "b"]))
    );
    assert_eq!(
        selected("--workspace --exclude a", &packages),
        Ok(names(&["b", "c", SCENARIO]))
    );
}

#[test]
fn a_shard_list_that_cargo_would_read_otherwise_fails() {
    let packages = names(&CRATES);
    let either = Err("list -p packages, or --workspace with its --exclude packages".to_owned());

    assert_eq!(selected("", &packages), either);
    assert_eq!(selected("--workspace -p a", &packages), either);
    assert_eq!(selected("-p a --exclude b", &packages), either);
    assert_eq!(
        selected("-p a --all", &packages),
        Err("`--all`: list only -p, --workspace and --exclude".to_owned())
    );
    assert_eq!(
        selected("-p", &packages),
        Err("`-p` names no package".to_owned())
    );
    assert_eq!(
        selected("-p z", &packages),
        Err("z is no workspace crate".to_owned())
    );
}

#[test]
fn a_bad_shard_list_is_named_by_its_shard() {
    let (_dir, c) = checkout(&CRATES);
    let flow = workflow("1, 2", &["\"-p a -p b -p c\"", "\"-p y\""]);

    assert_eq!(
        shards_cover_the_workspace(&c, &flow),
        ["the test job's shards: shard 2: y is no workspace crate"]
    );
}

#[test]
fn a_matrix_with_more_shards_than_lists_fails() {
    let (_dir, c) = checkout(&CRATES);
    let flow = workflow("1, 2, 3", &["\"-p a -p b -p c\""]);

    assert_eq!(
        shards_cover_the_workspace(&c, &flow),
        ["the test job's shards: the matrix does not list one shard per package list (1 lists)"]
    );
}

#[test]
fn a_job_that_picks_its_shard_otherwise_fails() {
    let (_dir, c) = checkout(&CRATES);
    let mut flow = workflow("1", &["\"-p a -p b -p c\""]);
    let job = &mut flow.jobs[0];
    job.text = job
        .text
        .replace("${shards[$JOB_INDEX]}", "${shards[0]}")
        .replace(SHARD_COMMAND, "cargo test --locked -p a");

    assert_eq!(
        shards_cover_the_workspace(&c, &flow),
        [
            format!(
                "the test job's shards: the job does not choose its list by its place in the matrix: {}",
                SELECTION[3]
            ),
            format!("the test job's shards: the job does not run {SHARD_COMMAND}"),
        ]
    );
}

#[test]
fn a_shards_array_holding_more_than_quoted_lists_fails() {
    let (_dir, c) = checkout(&CRATES);
    let flow = workflow("1", &["-p a"]);
    let open = workflow("1", &["\"-p a"]);

    let want = [
        "the test job's shards: the shards=( ... ) array holds more than double-quoted package lists",
    ];
    assert_eq!(shards_cover_the_workspace(&c, &flow), want);
    assert_eq!(shards_cover_the_workspace(&c, &open), want);
}

#[test]
fn a_job_without_a_shards_array_fails() {
    let (_dir, c) = checkout(&CRATES);
    let mut flow = workflow("1", &["\"-p a\""]);
    let mut open = workflow("1", &["\"-p a\""]);
    flow.jobs[0].text = flow.jobs[0].text.replace("shards=(", "lists=(");
    open.jobs[0].text = open.jobs[0].text.replace("          )\n", "");

    let want = ["the test job's shards: no shards=( ... ) array lists their packages"];
    assert_eq!(shards_cover_the_workspace(&c, &flow), want);
    assert_eq!(shards_cover_the_workspace(&c, &open), want);
}

#[test]
fn a_workflow_without_a_test_job_fails() {
    let (_dir, c) = checkout(&CRATES);

    assert_eq!(
        shards_cover_the_workspace(&c, &parse("jobs:\n  lint:\n    steps:\n")),
        ["the CI workflow has no test job"]
    );
}

#[test]
fn the_workspace_s_packages_are_read_from_each_member_s_manifest() {
    let (_dir, c) = checkout(&["b", "a"]);

    assert_eq!(packages(&c), Ok(names(&["a", "b"])));
}

#[test]
fn a_member_without_a_package_name_or_a_root_manifest_fails() {
    let (_dir, c) = checkout(&["a"]);
    fs::write(c.root.join("crates/a/Cargo.toml"), "[lib]\n").unwrap();
    let flow = workflow("1", &["\"-p a\""]);

    assert_eq!(
        shards_cover_the_workspace(&c, &flow),
        ["crates/a/Cargo.toml names no package"]
    );
    fs::remove_file(c.root.join("Cargo.toml")).unwrap();
    assert!(packages(&c).unwrap_err().starts_with("read Cargo.toml: "));
}
