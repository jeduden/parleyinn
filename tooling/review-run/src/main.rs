//! The `review-run` command: the review of a pull request run by code
//! (ENG-28). It starts each dispatched agent as a `claude` process
//! through the evaluation's process host, each with its deadline, and
//! nothing else. It is CI tooling and never ships. In a GitHub Actions
//! job it adds the table of every run to the job's summary file, the
//! one `GITHUB_STEP_SUMMARY` names, unless `--summary` names another.

use std::io;
use std::process::ExitCode;

use review_eval::process::ProcessHost;

fn main() -> ExitCode {
    let summary = std::env::var("GITHUB_STEP_SUMMARY")
        .ok()
        .filter(|path| !path.is_empty())
        .map(|path| format!("--summary={path}"));
    let code = review_run::cli::run(
        &ProcessHost,
        summary.into_iter().chain(std::env::args().skip(1)),
        &mut io::stdout(),
        &mut io::stderr(),
    );

    ExitCode::from(code)
}
