//! Which failed runs are run again, and after how long. A run the API
//! turned away as overloaded or rate-limited, as its result or its
//! errors say (`overloaded_error`, `rate_limit_error`, a 429 or 529),
//! waits with an exponential backoff and jitter before it runs again,
//! within [`MAX_ATTEMPTS`], so at most twice, each wait cut to what is
//! left of its deadline. A run that can never succeed by running again is not run
//! again, and why is named: one a cost cap stopped or kept from
//! starting, and one its credential was refused for (a 401 or 403, an
//! invalid token or key). Any other failure is run once more.

use std::time::Duration;

use review_eval::runner::OVER_BUDGET as OVER_BUDGET_SUBTYPE;

use crate::controls::OVER_BUDGET;

/// The first wait after an overloaded run, doubled for the second.
pub const BASE: Duration = Duration::from_secs(15);
/// How many attempts one agent's run takes at most: its first, the one
/// more every failure but one that can never succeed gets, and one more
/// after an attempt the API turned away; so at most two waits.
pub const MAX_ATTEMPTS: u32 = 3;

/// What a failed run's failure and errors say of running it again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// The API was overloaded or rate-limited: wait, then run again.
    Overloaded,
    /// Running it again can never succeed, for the reason given.
    Never(&'static str),
    /// Run it once more.
    Again,
}

/// What a run's errors or result say when the API was overloaded or
/// rate-limited, lowercased.
const OVERLOADED: [&str; 7] = [
    "overloaded",
    "rate_limit",
    "rate limit",
    "api error: 429",
    "api error: 529",
    "status 429",
    "status 529",
];

/// What they say when its credential was refused, lowercased.
const REFUSED: [&str; 8] = [
    "api error: 401",
    "api error: 403",
    "status 401",
    "status 403",
    "authentication_error",
    "permission_error",
    "invalid token",
    "invalid api key",
];

/// What running again would do for a run that failed with `failure`,
/// its errors `stderr`.
#[must_use]
pub fn class(failure: &str, stderr: &str) -> Class {
    let text = format!("{failure}\n{stderr}").to_lowercase();
    let says = |words: &[&str]| words.iter().any(|w| text.contains(w));
    if failure.starts_with(OVER_BUDGET) || failure.contains(OVER_BUDGET_SUBTYPE) {
        Class::Never("a cost cap stopped it, and another run would cost more")
    } else if says(&REFUSED) {
        Class::Never("its credential was refused, and no run with it can succeed")
    } else if says(&OVERLOADED) {
        Class::Overloaded
    } else {
        Class::Again
    }
}

/// The wait before the run after `waited` waits, `jitter` a draw in
/// `[0, 1)` adding up to half as much again, cut to `left` of the
/// deadline; none when nothing is left.
#[must_use]
pub fn wait(waited: u32, jitter: f64, left: Duration) -> Option<Duration> {
    if left.is_zero() {
        return None;
    }
    let step = BASE.saturating_mul(1 << waited.min(16));
    let spread = step.mul_f64(jitter.clamp(0.0, 1.0) / 2.0);

    Some(step.saturating_add(spread).min(left))
}

#[cfg(test)]
mod tests;
