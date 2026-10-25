//! The `benchmark` executable: the suite NFR-01..05 register against
//! and the gate the CI job runs on it (ENG-15). Everything it does is
//! [`benchmark::cli::run`]; this entry point hands it the process's
//! arguments and streams, the file system, and the timer that starts a
//! program. It is CI tooling and never ships.

use std::fs;
use std::io;
use std::process::ExitCode;

use benchmark::measure;

fn main() -> ExitCode {
    let code = benchmark::cli::run(
        std::env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
        &|path| fs::read_to_string(path),
        &|program, benchmark| measure::time_benchmark(program, benchmark, measure::DEADLINE),
    );

    ExitCode::from(code)
}
