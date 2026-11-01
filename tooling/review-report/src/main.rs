//! The `review-report` command: a review's run list and report from
//! its work directory, and the stability of repeated reviews of one
//! pull-request snapshot (ENG-28, ENG-32). It reads and writes files only.
//! It is CI tooling and never ships.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let code = review_report::cli::run(
        std::env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
    );

    ExitCode::from(code)
}
