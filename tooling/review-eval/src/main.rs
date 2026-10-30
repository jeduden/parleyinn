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
    let code = review_eval::cli::run(
        &ProcessHost,
        std::env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
    );

    ExitCode::from(code)
}
