//! The `review-run` command: the review of a pull request run by code
//! (ENG-28). It starts each dispatched agent as a `claude` process
//! through the evaluation's process host, each with its deadline, and
//! nothing else. It is CI tooling and never ships.

use std::io;
use std::process::ExitCode;

use review_eval::process::ProcessHost;

fn main() -> ExitCode {
    let code = review_run::cli::run(
        &ProcessHost,
        std::env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
    );

    ExitCode::from(code)
}
