//! The `review-bundle` command builds what the review workflow hands
//! the reviewing agent (ENG-21, ENG-28): the review bundle of touched
//! requirements, scenarios and invariants, the deterministic
//! findings the review gate weighs, and the review class printed as
//! step outputs. The workflow runs it from main's copy of the
//! repository, with main's tree, the pull request's tree and the
//! stakeholder's comments as data.
//!
//! It reads files and writes the two outputs and stdout only. It is CI
//! tooling and never ships.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let code = review_bundle::cli::run(
        std::env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
    );

    ExitCode::from(code)
}
