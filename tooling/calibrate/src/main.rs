//! The `calibrate` executable: runs the frozen workload once and prints
//! its result, so the work cannot be optimised away. It is test tooling
//! and never ships.

use std::process::ExitCode;

fn main() -> ExitCode {
    match calibrate::run(&std::env::temp_dir()) {
        Ok(result) => {
            println!("{result:016x}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("calibrate: {e}");
            ExitCode::FAILURE
        }
    }
}
