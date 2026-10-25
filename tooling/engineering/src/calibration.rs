//! NFR-02: the reference a hook handler's time is judged against is the
//! frozen calibration workload of the `calibrate` crate, one unit of
//! which is a nominal 10 ms by definition. It stays a constant
//! measurement only while it never changes, so this check holds its
//! source to a recorded hash and its constants to their recorded
//! values: a change to either fails the build, and moving the record
//! is a change the stakeholder reviews.

use std::fs;
use std::path::Path;

/// Where the workload's source lies, relative to the checkout.
pub const SOURCE_DIR: &str = "tooling/calibrate/src";

/// The hash of the workload's source, recorded when it was tuned.
pub const RECORDED_HASH: u64 = 0xb0a1_a524_60b6_90c2;

/// The workload's constants as recorded: the seed, the rounds of
/// mixing and the rounds of file calls.
pub const RECORDED_CONSTANTS: (u64, u64, u32) = (0x9e37_79b9_7f4a_7c15, 1_200_000, 50);

/// The 64-bit FNV-1a hash of `bytes`.
#[must_use]
pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The hash of every file under `root`'s [`SOURCE_DIR`], each by name
/// and content, in name order.
///
/// # Errors
///
/// Fails when the directory or a file in it cannot be read.
pub fn source_hash(root: &Path) -> Result<u64, String> {
    let dir = root.join(SOURCE_DIR);
    let read = |e: std::io::Error| format!("read {SOURCE_DIR}: {e}");
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).map_err(read)? {
        names.push(entry.map_err(read)?.file_name());
    }
    names.sort();
    let mut bytes = Vec::new();
    for name in names {
        bytes.extend_from_slice(name.as_encoded_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&fs::read(dir.join(&name)).map_err(read)?);
        bytes.push(0);
    }

    Ok(fnv1a(&bytes))
}

/// What differs between the workload `root` holds, with `constants`, and
/// the record: its hash against `recorded_hash`, its constants against
/// [`RECORDED_CONSTANTS`].
#[must_use]
pub fn problems(root: &Path, recorded_hash: u64, constants: (u64, u64, u32)) -> Vec<String> {
    let mut found = Vec::new();
    match source_hash(root) {
        Ok(hash) if hash == recorded_hash => {}
        Ok(hash) => found.push(format!(
            "the calibration workload's source hashes to {hash:016x}, not the recorded {recorded_hash:016x}: \
             the workload is frozen, and one calibration unit is 10 ms by definition"
        )),
        Err(e) => found.push(e),
    }
    let (seed, mixing, files) = constants;
    let (want_seed, want_mixing, want_files) = RECORDED_CONSTANTS;
    if seed != want_seed {
        found.push(format!(
            "the calibration seed is {seed:#x}, not the recorded {want_seed:#x}"
        ));
    }
    if mixing != want_mixing {
        found.push(format!(
            "the calibration mixing rounds are {mixing}, not the recorded {want_mixing}"
        ));
    }
    if files != want_files {
        found.push(format!(
            "the calibration file rounds are {files}, not the recorded {want_files}"
        ));
    }

    found
}

/// [`problems`] for the checkout's workload against the record.
#[must_use]
pub fn frozen(root: &Path) -> Vec<String> {
    problems(
        root,
        RECORDED_HASH,
        (
            calibrate::SEED,
            calibrate::MIX_ROUNDS,
            calibrate::FILE_ROUNDS,
        ),
    )
}

#[cfg(test)]
mod tests;
