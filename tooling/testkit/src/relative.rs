//! A relative time check (NFR-02): a program's time judged against a
//! reference program's, both timed in interleaved pairs under the same
//! confinement at the same moment, so what slows one slows the other
//! and cancels in the ratio of their medians.

use std::time::Duration;

/// Twice the median of `samples`, in nanoseconds: an even count's median
/// is the mean of its middle two, so twice it stays an integer. `None`
/// for no samples.
#[must_use]
pub fn twice_median_nanos(samples: &[Duration]) -> Option<u128> {
    let mut sorted: Vec<u128> = samples.iter().map(Duration::as_nanos).collect();
    sorted.sort_unstable();
    let n = sorted.len();
    let low = sorted.get(n.checked_sub(1)? / 2)?;
    let high = sorted.get(n / 2)?;

    Some(low + high)
}

/// Judges the median of `program` against `limit` times the median of
/// `reference`, in whole multiples.
///
/// # Errors
///
/// Fails, naming both medians and the limit, when the program's median
/// is more than `limit` times the reference's, or when either has no
/// samples.
pub fn within_multiple(
    program: &[Duration],
    reference: &[Duration],
    limit: u32,
) -> Result<(), String> {
    let (Some(own), Some(base)) = (twice_median_nanos(program), twice_median_nanos(reference))
    else {
        return Err("a relative time check needs samples of both programs".to_owned());
    };
    if own <= base * u128::from(limit) {
        return Ok(());
    }
    Err(format!(
        "the median time {:?} is more than {limit} times the reference's {:?}",
        Duration::from_nanos(u64::try_from(own / 2).unwrap_or(u64::MAX)),
        Duration::from_nanos(u64::try_from(base / 2).unwrap_or(u64::MAX)),
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
