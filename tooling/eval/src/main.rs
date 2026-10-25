//! `parinn-eval recall [--published FILE]`: runs the recall-after-
//! compaction evaluation on every seed of the dataset, prints its
//! report, and exits 1 when a depth's recall is below the target or the
//! published result differs from the one measured.

use std::process::ExitCode;

use parinn_eval::cli;
use parinn_eval::dataset::{self, SEEDS};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (code, out) = cli::run(&args, &testkit::repo_root(), &SEEDS, dataset::generate);
    print!("{out}");
    ExitCode::from(code)
}
