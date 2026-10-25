//! The statistics the gate decides with: the median, kept exact as twice
//! its value, the one-sided Mann–Whitney U test as a z score, and its
//! stratified form for samples taken on several runners. All are pure
//! and deterministic: no clock, no randomness, so the same samples
//! always yield the same decision.

/// Twice the median of `samples`: an even count's median is the mean of
/// its middle two, so twice it stays an integer and the gate compares
/// medians without rounding. `None` for no samples.
#[must_use]
pub fn twice_median(samples: &[u64]) -> Option<u128> {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    let low = sorted.get(n.checked_sub(1)? / 2)?;
    let high = sorted.get(n / 2)?;

    Some(u128::from(*low) + u128::from(*high))
}

/// The Mann–Whitney U statistic of `head` against `base` as a z score
/// under the normal approximation, with ties given their mean rank and
/// the variance corrected for them, and a continuity correction of one
/// half toward zero. Positive when the head's samples tend to be larger
/// than the base's. Zero when either side is empty or every sample ties,
/// since then nothing separates the two sides.
#[must_use]
pub fn mann_whitney_z(base: &[u64], head: &[u64]) -> f64 {
    rank_sum(base, head).map_or(0.0, |(above, variance)| (above - 0.5) / variance.sqrt())
}

/// The stratified rank test (van Elteren) of `head` against `base`, each
/// stratum a pair of the base's and the head's samples taken on one
/// runner. A sample is ranked only against samples of its own stratum,
/// so the runners' speed differences never widen the distributions;
/// each stratum's rank sum is weighted by one over its size plus one
/// and the weighted sums are combined into one z score. One stratum is
/// [`mann_whitney_z`]. A stratum with an empty side or every sample
/// tied separates nothing and is left out; zero when none remains.
#[must_use]
pub fn stratified_z(strata: &[(&[u64], &[u64])]) -> f64 {
    if let [(base, head)] = strata {
        return mann_whitney_z(base, head);
    }
    let (mut above, mut variance) = (0.0, 0.0);
    for (base, head) in strata {
        if let Some((a, v)) = rank_sum(base, head) {
            let weight = 1.0 / (count(base.len() + head.len()) + 1.0);
            above += weight * a;
            variance += weight * weight * v;
        }
    }
    if variance <= 0.0 {
        return 0.0;
    }

    above / variance.sqrt()
}

/// How far the sum of `head`'s ranks among `base` and `head` together
/// lies above its expectation when the sides do not differ, and the
/// variance of that sum, with ties given their mean rank and the
/// variance corrected for them. `None` when either side is empty or
/// every sample ties, since then nothing separates the two sides.
fn rank_sum(base: &[u64], head: &[u64]) -> Option<(f64, f64)> {
    if base.is_empty() || head.is_empty() {
        return None;
    }
    let mut pooled: Vec<(u64, bool)> = base
        .iter()
        .map(|&s| (s, false))
        .chain(head.iter().map(|&s| (s, true)))
        .collect();
    pooled.sort_unstable();

    // Each tie group of equal samples takes the mean of the ranks it spans,
    // and adds t^3 - t to the tie correction.
    let (mut ranked, mut head_ranks, mut ties) = (0.0, 0.0, 0.0);
    for tie_group in pooled.chunk_by(|a, b| a.0 == b.0) {
        let t = count(tie_group.len());
        let mean_rank = ranked + (t + 1.0) / 2.0;
        ranked += t;
        head_ranks += mean_rank * count(tie_group.iter().filter(|(_, on_head)| *on_head).count());
        ties += t * t * t - t;
    }

    let (n_base, n_head) = (count(base.len()), count(head.len()));
    let n = n_base + n_head;
    let expected = n_head * (n + 1.0) / 2.0;
    let variance = n_base * n_head / 12.0 * ((n + 1.0) - ties / (n * (n - 1.0)));
    if variance <= 0.0 {
        return None;
    }

    Some((head_ranks - expected, variance))
}

/// A count as a float. Sample counts stay far below 2^52, where the
/// conversion is exact.
fn count(n: usize) -> f64 {
    n as f64
}

#[cfg(test)]
mod tests;
