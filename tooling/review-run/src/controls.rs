//! What a review's runs are held to beside each run's own deadline:
//! the clock it reads, waits on and draws jitter from, which a test
//! hands in so it never sleeps; the review's cost cap, the sum of what
//! its finished runs cost, past which no run starts; and the circuit
//! breaker, which counts the specialists that ended in a gap and, once
//! more than its fraction of those dispatched did, starts no
//! generalist. A run the cap or the breaker keeps from starting is
//! named with why, never taken as clean (I6).

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::live::Live;

/// The run-level cost cap unless the run says otherwise, in US
/// dollars: 56 specialists at the dearest a specialist of pull request
/// 98's reviews cost, 0.30, and three generalists at theirs, 2.00, come
/// to about 23, so 40 leaves room before a review stops.
pub const MAX_COST_USD: f64 = 40.0;
/// The fraction of the dispatched specialists that may end in a gap
/// before no generalist starts, unless the run says otherwise: more
/// than a third, exactly one third not tripping it.
pub const BREAKER_FRACTION: Fraction = Fraction { num: 1, den: 3 };
/// How a gap's reason starts for a run the cost caps stopped or kept
/// from starting.
pub const OVER_BUDGET: &str = "over budget:";
/// How a gap's reason starts for a generalist the breaker kept from
/// starting.
pub const ABORTED: &str = "aborted:";

/// A share of a whole, whole numbers so that "more than" is exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fraction {
    pub num: u64,
    pub den: u64,
}

impl Fraction {
    /// All of them: more than all of the specialists never ended in a
    /// gap, so a breaker of it never trips.
    pub const WHOLE: Self = Self { num: 1, den: 1 };
}

/// What one review's runs are held to and watched by.
#[derive(Debug)]
pub struct Controls {
    pub budget: Budget,
    pub breaker: Breaker,
    pub live: Live,
}

/// The time a review reads, waits on and draws jitter from.
pub trait Clock: Sync {
    /// How long since the review started.
    fn now(&self) -> Duration;
    /// Waits `duration`.
    fn sleep(&self, duration: Duration);
    /// A draw in `[0, 1)`, spreading the waits of runs that failed
    /// together.
    fn jitter(&self) -> f64;
}

/// The machine's clock: real time, real waits, and jitter from the
/// time of day mixed with a count of draws.
#[derive(Debug)]
pub struct SystemClock {
    start: Instant,
    draws: AtomicU64,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            draws: AtomicU64::new(0),
        }
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.start.elapsed()
    }

    fn sleep(&self, duration: Duration) {
        thread::sleep(duration);
    }

    fn jitter(&self) -> f64 {
        let draw = self.draws.fetch_add(1, Ordering::SeqCst);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| u64::from(d.subsec_nanos()));
        unit(nanos ^ draw.wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }
}

/// `bits` as a draw in `[0, 1)`.
#[must_use]
pub fn unit(bits: u64) -> f64 {
    let millionths = u32::try_from(bits % 1_000_000).unwrap_or_default();
    f64::from(millionths) / 1_000_000.0
}

/// The review's cost cap and what its finished runs cost.
#[derive(Debug)]
pub struct Budget {
    cap: f64,
    spent: Mutex<f64>,
}

impl Budget {
    /// A cap of `cap` US dollars, nothing spent.
    #[must_use]
    pub fn new(cap: f64) -> Self {
        Self {
            cap,
            spent: Mutex::new(0.0),
        }
    }

    /// Adds what a finished run cost.
    pub fn charge(&self, cost: f64) {
        *self.spent.lock().unwrap_or_else(PoisonError::into_inner) += cost;
    }

    /// What the finished runs cost.
    #[must_use]
    pub fn spent(&self) -> f64 {
        *self.spent.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Why no further run may start, once what was spent reached the
    /// cap; none below it.
    #[must_use]
    pub fn refusal(&self) -> Option<String> {
        let spent = self.spent();
        (spent >= self.cap).then(|| {
            format!(
                "{OVER_BUDGET} the review's runs cost {spent:.4} USD, its cap is {:.2} USD, so \
                 it was not started",
                self.cap
            )
        })
    }
}

/// The circuit breaker: how many of the dispatched specialists ended,
/// how many of them in a gap, and whether that is more than its
/// fraction of them.
#[derive(Debug)]
pub struct Breaker {
    fraction: Fraction,
    dispatched: usize,
    failed: AtomicUsize,
    ended: AtomicUsize,
}

impl Breaker {
    /// A breaker over `dispatched` specialists that trips past
    /// `fraction` of them.
    #[must_use]
    pub fn new(fraction: Fraction, dispatched: usize) -> Self {
        Self {
            fraction,
            dispatched,
            failed: AtomicUsize::new(0),
            ended: AtomicUsize::new(0),
        }
    }

    /// Counts one more specialist ended, in a gap when `failed`.
    pub fn end(&self, failed: bool) {
        if failed {
            self.failed.fetch_add(1, Ordering::SeqCst);
        }
        self.ended.fetch_add(1, Ordering::SeqCst);
    }

    /// Whether the breaker has tripped, or can no longer trip however
    /// the specialists still running end: until then no generalist
    /// starts, so whether one does never hangs on which ended first.
    #[must_use]
    pub fn decided(&self) -> bool {
        let failed = self.failed.load(Ordering::SeqCst);
        let running = self
            .dispatched
            .saturating_sub(self.ended.load(Ordering::SeqCst));
        trips(failed, self.dispatched, self.fraction)
            || !trips(failed + running, self.dispatched, self.fraction)
    }

    /// Why a generalist may not start, once the breaker tripped; none
    /// before.
    #[must_use]
    pub fn refusal(&self) -> Option<String> {
        let failed = self.failed.load(Ordering::SeqCst);
        trips(failed, self.dispatched, self.fraction).then(|| {
            format!(
                "{ABORTED} {}, so it was not started",
                mass_failure(failed, self.dispatched)
            )
        })
    }
}

/// Whether `failed` of `dispatched` specialists is more than `fraction`
/// of them, compared in whole numbers.
#[must_use]
pub fn trips(failed: usize, dispatched: usize, fraction: Fraction) -> bool {
    let wide = |n: usize| u128::try_from(n).unwrap_or(u128::MAX);
    let (failed, dispatched) = (wide(failed), wide(dispatched));
    failed * u128::from(fraction.den) > dispatched * u128::from(fraction.num)
}

/// How the review names a mass failure of `failed` of `dispatched`
/// specialists.
#[must_use]
pub fn mass_failure(failed: usize, dispatched: usize) -> String {
    format!(
        "{failed} of {dispatched} specialists failed (mass failure; likely an outage or a broken \
         prompt)"
    )
}

#[cfg(test)]
mod tests;
