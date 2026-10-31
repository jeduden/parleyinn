//! The `review-gate` command decides the review the reviewer app posts
//! on a pull request (ENG-28). The review workflow runs it from main's
//! copy of the repository, in the one job that holds the app's key,
//! after review-run wrote the review outcome. It prints the
//! review as the JSON GitHub's create-review endpoint takes, prints
//! nothing when the pull request moved past the reviewed head, and
//! exits non-zero on anything it cannot decide, so the job fails and
//! posts nothing.
//!
//! It reads files and writes stdout only; the workflow fetches the
//! check runs and posts the review. It is CI tooling and never ships.

use std::fs;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let code = review_gate::cli::run(
        std::env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
        &|path| fs::read_to_string(path),
    );

    ExitCode::from(code)
}
