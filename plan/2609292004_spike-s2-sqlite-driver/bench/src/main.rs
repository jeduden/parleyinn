//! Spike S2's harness for the Rust core: it builds the synthetic
//! ParleyInn store with `rusqlite` and bundled SQLite and measures it
//! against NFR-03, NFR-04, NFR-05 and NFR-09. It ports ../bench-go,
//! the Go harness that compared two pure-Go drivers, command for
//! command, and generates the same corpus from the same seed.
//!
//! It is a standalone crate, so `rusqlite` enters parinn's workspace
//! only with the M1 change that links the store.

mod cmds;
mod corpus;
mod fts5;
mod store;
mod wal;

use std::path::PathBuf;
use std::process::ExitCode;

use sha2::{Digest, Sha256};

/// Leaves search unbounded: it exceeds any store.
pub const DEFAULT_WINDOW: i64 = 1 << 62;

#[derive(Clone)]
pub struct Opts {
    pub db: PathBuf,
    pub n: i64,
    pub batch: i64,
    pub workers: i64,
    pub id: i64,
    pub seed: u64,
    pub window: i64,
    pub recent: i64,
    /// Key the store with SQLCipher (the `cipher` build only).
    pub key: bool,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        db: PathBuf::new(),
        n: 1_000_000,
        batch: 100,
        workers: 50,
        id: 0,
        seed: 1,
        window: DEFAULT_WINDOW,
        recent: 0,
        key: false,
    };
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        if flag == "-key" {
            o.key = true;
            continue;
        }
        let v = it.next().ok_or(format!("{flag} needs a value"))?;
        let int = || v.parse::<i64>().map_err(|e| format!("{flag}: {e}"));
        match flag.as_str() {
            "-db" => o.db = PathBuf::from(v),
            "-n" => o.n = int()?,
            "-batch" => o.batch = int()?,
            "-workers" => o.workers = int()?,
            "-id" => o.id = int()?,
            "-seed" => o.seed = v.parse().map_err(|e| format!("-seed: {e}"))?,
            "-window" => o.window = int()?,
            "-recent" => o.recent = int()?,
            _ => return Err(format!("unknown flag {flag}")),
        }
    }
    Ok(o)
}

/// Hashes the first o.n events of the corpus, field by field, for
/// comparison with the Go harness's generator.
fn corpus_hash(o: &Opts) -> store::Result<()> {
    let mut c = corpus::Corpus::new(o.seed);
    let mut h = Sha256::new();
    for _ in 0..o.n {
        let e = c.next();
        h.update(format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
            e.session, e.source, e.line, e.kind, e.trust, e.tool.unwrap_or(""), e.body, e.payload, e.ts
        ));
    }
    println!("RESULT corpus events={} sha256={:x}", o.n, h.finalize());
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        eprintln!("usage: s2bench <corpus|load|search|expand|ingest|hook|cancel|wal|walworker|fts5|optimize> [flags]");
        return ExitCode::from(2);
    };
    let o = match parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("s2bench: {e}");
            return ExitCode::from(2);
        }
    };
    let res = match cmd.as_str() {
        "corpus" => corpus_hash(&o),
        "load" => cmds::load(&o),
        "search" => cmds::search(&o),
        "expand" => cmds::expand(&o),
        "ingest" => cmds::ingest(&o),
        "hook" => cmds::hook(&o),
        "cancel" => cmds::cancel(&o),
        "optimize" => cmds::optimize(&o),
        "wal" => wal::wal(&o),
        "walworker" => wal::worker(&o),
        "fts5" => fts5::fts5(&o),
        other => Err(format!("unknown command {other:?}").into()),
    };
    match res {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("s2bench: {e}");
            ExitCode::FAILURE
        }
    }
}
