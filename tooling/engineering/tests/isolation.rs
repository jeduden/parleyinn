//! ENG-14's checks against the repository checkout itself: the agent
//! instruction files, and the one place tests resolve real state.

use engineering::{Checkout, agent_instructions, isolation_guard};

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

/// No agent instruction file directs a destructive command at
/// non-isolated state (ENG-14).
#[test]
fn the_agent_instructions_direct_no_destructive_command_at_real_state() {
    let problems = agent_instructions::no_destructive_instructions(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The scan reads the instruction files this repository carries.
#[test]
fn the_agent_instructions_scan_reads_the_repository_s_files() {
    let files = agent_instructions::instruction_files(&checkout().root).unwrap();

    for want in ["CLAUDE.md", "AGENTS.md", ".claude/settings.json"] {
        assert!(files.iter().any(|f| f == want), "{want} not in {files:?}");
    }
    assert!(files.iter().any(|f| f.starts_with(".claude/agents/")));
    assert!(files.iter().any(|f| f.starts_with(".claude/skills/")));
}

/// CI runs the suite with `HOME` pointed at a decoy, and fails when a
/// test left anything under it (ENG-14): the guard stops a test that
/// resolves real state, and the decoy catches a test that writes under
/// the user's `HOME` without going through the guard. Every shard of
/// the job runs its package list behind it, and
/// `the_ci_test_shards_run_every_crate_once` holds the lists to every
/// crate.
#[test]
fn the_ci_test_job_runs_with_a_decoy_home() {
    let c = checkout();
    let wants = [
        r#"HOME="$decoy" cargo test --locked "${packages[@]}""#,
        r#"[ -n "$(ls -A "$decoy")" ]"#,
    ];

    let problems: Vec<String> = wants
        .iter()
        .filter_map(|want| c.contains(".github/workflows/ci.yml", want).err())
        .collect();

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Nothing reads the home but the isolation guard (ENG-14): a test
/// that resolves it goes through `testkit::guard`, so the guard covers
/// every test and not only those that opt in.
#[test]
fn only_the_isolation_guard_reads_the_home() {
    let problems = isolation_guard::no_unguarded_reads(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
