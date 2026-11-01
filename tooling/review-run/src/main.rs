//! The `review-run` command: the review of a pull request run by code
//! (ENG-28). It starts each dispatched agent as a `claude` process
//! through the evaluation's process host, each with its deadline, and
//! nothing else. It is CI tooling and never ships.

use std::io;
use std::process::ExitCode;

use review_eval::process::ProcessHost;

fn main() -> ExitCode {
    // The access token this process holds, when it holds one: a form
    // every text it writes is scrubbed of. No agent it starts gets it
    // (ENG-28).
    let token = std::env::var(review_eval::credential::TOKEN_VAR).ok();
    let code = review_run::cli::run_with(
        &ProcessHost,
        std::env::args().skip(1),
        token.as_deref(),
        &mut io::stdout(),
        &mut io::stderr(),
    );

    ExitCode::from(code)
}
