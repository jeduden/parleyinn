//! The `review-eval` command: the evaluation of the review's own
//! reviewers (ENG-32). It starts the processes the library asks for,
//! `git` for the fixture worktrees, `mdsmith` for its pattern catalog
//! and `claude` for each run, each with a deadline, through the
//! library's process host, and nothing else. It is CI tooling and
//! never ships.

use std::io;
use std::process::ExitCode;

use review_eval::process::ProcessHost;

fn main() -> ExitCode {
    // The access token this process holds, when it holds one: what
    // `stage-credential` stages and a form the scrub replaces. No
    // command it starts gets it (ENG-28).
    let token = std::env::var(review_eval::credential::TOKEN_VAR).ok();
    let code = review_eval::cli::run_with(
        &ProcessHost,
        std::env::args().skip(1),
        token.as_deref(),
        &mut io::stdout(),
        &mut io::stderr(),
    );

    ExitCode::from(code)
}
