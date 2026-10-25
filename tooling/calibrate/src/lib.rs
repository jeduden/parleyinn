//! The calibration workload (NFR-02): a fixed computation and a fixed
//! number of file system calls shaped like a hook handler's. The
//! `@NFR-02` hook budget rows time it beside a hook handler, under the
//! same confinement, and hold the handler to a multiple of it. It reads
//! nothing of ParleyInn and imports no crate, so it stays the same when
//! the product changes.
//!
//! One calibration unit is a nominal 10 ms, by definition. The
//! constants below were tuned once to take about that on a release
//! build, and never change: a gate test in `engineering` fails when
//! this crate's source differs from the recorded hash.

use std::hint::black_box;
use std::io::{self, Read as _, Write as _};
use std::path::Path;
use std::time::Instant;

/// The seed of the mixing computation.
pub const SEED: u64 = 0x9e37_79b9_7f4a_7c15;

/// How many rounds of mixing the workload runs.
pub const MIX_ROUNDS: u64 = 1_200_000;

/// How many times the workload creates, reads and removes its file.
pub const FILE_ROUNDS: u32 = 50;

/// The bytes the workload writes to its file each round.
const CHUNK: usize = 64;

/// `rounds` rounds of integer mixing from `seed`.
#[must_use]
pub fn mix(seed: u64, rounds: u64) -> u64 {
    let mut x = seed;
    for i in 0..rounds {
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^= x >> 31;
        x = x.wrapping_add(i);
    }

    x
}

/// `rounds` rounds of creating, writing, reading and inspecting a file
/// in `dir`, then removing it: open, write, close, open, read, close,
/// stat, process id and clock. The sum of what it read and measured.
///
/// # Errors
///
/// Fails when the file cannot be created, read or removed.
pub fn touch_files(dir: &Path, rounds: u32) -> io::Result<u64> {
    let path = dir.join(format!("calibrate-{}.tmp", std::process::id()));
    let mut sum = 0u64;
    for round in 0..rounds {
        // A new file each round: rewriting an existing one makes the
        // file system flush it on close, which times the disk and not
        // the work.
        let step = one_round(&path, round, &mut sum);
        let removed = std::fs::remove_file(&path);
        step?;
        removed?;
    }

    Ok(sum)
}

/// One round of [`touch_files`] on `path`, adding what it read and
/// measured to `sum`.
fn one_round(path: &Path, round: u32, sum: &mut u64) -> io::Result<()> {
    let chunk = [u8::try_from(round % 251).unwrap_or(0); CHUNK];
    std::fs::File::create(path)?.write_all(&chunk)?;
    let mut back = [0u8; CHUNK];
    std::fs::File::open(path)?.read_exact(&mut back)?;
    *sum += back.iter().map(|b| u64::from(*b)).sum::<u64>();
    *sum += std::fs::metadata(path)?.len();
    black_box((std::process::id(), Instant::now()));

    Ok(())
}

/// The workload with its own parameters: the mixed seed combined with
/// the files' sum.
///
/// # Errors
///
/// Fails as [`touch_files`] does.
pub fn run_with(dir: &Path, seed: u64, mix_rounds: u64, file_rounds: u32) -> io::Result<u64> {
    let mixed = mix(black_box(seed), black_box(mix_rounds));
    let files = touch_files(dir, file_rounds)?;

    Ok(black_box(mixed ^ files))
}

/// The frozen workload, in `dir`.
///
/// # Errors
///
/// Fails as [`touch_files`] does.
pub fn run(dir: &Path) -> io::Result<u64> {
    run_with(dir, SEED, MIX_ROUNDS, FILE_ROUNDS)
}

#[cfg(test)]
mod tests;
