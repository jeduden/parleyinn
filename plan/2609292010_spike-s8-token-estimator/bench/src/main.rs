//! Spike S8's harness: the conservative model-token estimator SRS §8.4
//! asks for, its check against the committed sample of Claude's
//! counts, and the hook handler timings NFR-01 needs, on the store
//! spike S2 builds with `s2bench load`.
//!
//! A standalone crate: `rusqlite` enters parinn's workspace only with
//! the M1 change that links the store.
//!
//!   s8bench verify SAMPLE.jsonl
//!   s8bench pins -db DB [-n 100]
//!   s8bench hook-ups -db DB [-source 150]      (hook input on stdin)
//!   s8bench hook-start -db DB [-source 150]    (hook input on stdin)
//!   s8bench lat -fixture F -n 1000 -- hook-ups -db DB
//!   s8bench fleet -db DB [-writers 10] [-secs 60]

mod estimate;
mod hook;
mod lat;
mod store;
mod verify;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(String::as_str)
}

fn int(args: &[String], name: &str, default: i64) -> Result<i64, String> {
    flag(args, name).map_or(Ok(default), |v| v.parse().map_err(|e| format!("{name}: {e}")))
}

fn run(args: &[String]) -> store::Result<()> {
    let cmd = args.first().map(String::as_str).unwrap_or_default();
    let rest = &args[1.min(args.len())..];
    let db = || flag(rest, "-db").map(PathBuf::from).ok_or("-db is required");
    match cmd {
        "verify" => {
            let path = rest.first().ok_or("verify needs the sample")?;
            if verify::verify(Path::new(path))? {
                Ok(())
            } else {
                Err("the S8 estimator covers under 99% of the sample".into())
            }
        }
        "pins" => store::add_pins(&store::open(&db()?)?, int(rest, "-n", 100)?),
        "hook-ups" => hook::user_prompt_submit(&db()?, int(rest, "-source", 150)?),
        "hook-start" => hook::session_start(&db()?, int(rest, "-source", 150)?, "r0"),
        "lat" => {
            let sep = rest.iter().position(|a| a == "--").ok_or("lat needs -- and a hook command")?;
            let fixture = flag(&rest[..sep], "-fixture").ok_or("-fixture is required")?;
            let n = int(&rest[..sep], "-n", 1000)? as usize;
            lat::lat(&rest[sep + 1..], Path::new(fixture), n)
        }
        "fleet" => lat::fleet(&db()?, int(rest, "-writers", 10)?, int(rest, "-secs", 60)? as u64),
        _ => Err(format!("unknown command {cmd:?}").into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("s8bench: {e}");
            ExitCode::FAILURE
        }
    }
}
