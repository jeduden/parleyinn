//! A relative time check (NFR-02): a program's time judged against a
//! reference program's, both timed in interleaved pairs under the same
//! confinement at the same moment. The fastest of the rounds stands for
//! each: noise only ever adds time, so the fastest round is the least
//! disturbed, and a program that is slow in every round is slow in its
//! fastest one too.

use std::time::Duration;

/// The fastest of `samples`, in nanoseconds. `None` for no samples.
#[must_use]
pub fn fastest_nanos(samples: &[Duration]) -> Option<u128> {
    samples.iter().map(Duration::as_nanos).min()
}

/// Judges the fastest time of `program` against `limit` times the
/// fastest time of `reference`, in whole multiples.
///
/// # Errors
///
/// Fails, naming both fastest times and the limit, when the program's is
/// more than `limit` times the reference's, or when either has no
/// samples.
pub fn within_multiple(
    program: &[Duration],
    reference: &[Duration],
    limit: u32,
) -> Result<(), String> {
    let (Some(own), Some(base)) = (fastest_nanos(program), fastest_nanos(reference)) else {
        return Err("a relative time check needs samples of both programs".to_owned());
    };
    if own <= base * u128::from(limit) {
        return Ok(());
    }
    Err(format!(
        "the fastest time {:?} is more than {limit} times the reference's {:?}",
        Duration::from_nanos(u64::try_from(own).unwrap_or(u64::MAX)),
        Duration::from_nanos(u64::try_from(base).unwrap_or(u64::MAX)),
    ))
}

/// A timing a step takes: how long one run of a program took.
pub type Timing<'a> = &'a mut dyn FnMut() -> Result<Duration, String>;

/// The times of `rounds` rounds, each timing the reference and the
/// program once, one right after the other, the order alternating from
/// round to round so a drift in the machine's speed favours neither.
/// Returns the program's times and the reference's.
///
/// # Errors
///
/// Fails with the first timing that fails.
pub fn interleaved(
    rounds: usize,
    reference: Timing<'_>,
    program: Timing<'_>,
) -> Result<(Vec<Duration>, Vec<Duration>), String> {
    let (mut own, mut base) = (Vec::new(), Vec::new());
    for round in 0..rounds {
        if round % 2 == 0 {
            base.push(reference()?);
            own.push(program()?);
        } else {
            own.push(program()?);
            base.push(reference()?);
        }
    }

    Ok((own, base))
}

#[cfg(test)]
mod tests;
