use super::*;

/// `review-run previous` only picks and checks a findings file: it
/// returns before the command line reads an agent, so no spelling of it
/// starts one (ENG-28).
#[test]
fn the_previous_command_of_review_run_starts_no_agent() {
    for text in [
        "cargo run --locked --quiet --offline -p review-run -- previous select --pull 7",
        "cargo run --package review-run -- previous check f",
        "cargo run --bin review-run -- previous check f",
        "target/debug/review-run previous check f",
        "target/release/review-run   previous check f",
        "cargo run -p review-run --   previous check f",
    ] {
        assert!(!runs_agent_step(text), "{text}");
        assert!(!runs_agents(text), "{text}");
    }
}

#[test]
fn every_other_use_of_review_run_still_starts_agents() {
    for text in [
        "cargo run --locked --quiet --offline -p review-run -- --workdir . previous",
        "cargo run -p review-run -- previously",
        "cargo run -p review-run -- previousx select",
        "cargo run -p review-run -- --previous f --workdir .",
        "target/debug/review-run --previous f",
        "cargo run -p review-run -- answers drop K",
        "cargo run -p review-run --",
        "target/debug/review-run",
        "target/debug/review-run --workdir .",
        // One start among many: the exemption is for the one command.
        "cargo run -p review-run -- previous check f && cargo run -p review-run -- --workdir .",
        "target/debug/review-run previous check f\ntarget/debug/review-run --workdir .",
        // The word must follow on the same line: a bare start with its
        // arguments on the next line is a start with none.
        "cargo run -p review-run --\nprevious check f",
        "target/debug/review-run\nprevious check f",
    ] {
        assert!(runs_agent_step(text), "{text}");
        assert!(runs_agents(text), "{text}");
    }
}
