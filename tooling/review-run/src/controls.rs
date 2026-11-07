//! What a review's runs are held to beside each run's own deadline:
//! the clock it reads, waits on and draws jitter from, which a test
//! hands in so it never sleeps; the pull request's cost cap, which a
//! run starts within only once its tier's cap fits beside what the
//! earlier reviews and this review's finished runs cost and the caps
//! of the runs in flight; and the circuit
//! breaker, which counts the specialists that ended in a gap and, once
//! more than its fraction of those dispatched did, starts no
//! generalist. A run the cap or the breaker keeps from starting is
//! named with why, never taken as clean (I6).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::live::Live;

/// The cost cap of a pull request, counted over all of its reviews,
/// unless the run says otherwise, in US dollars: the stakeholder's
/// budget for one pull request.
pub const PULL_REQUEST_COST_USD: f64 = 5.0;
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

/// The pull request's cost cap, what its earlier reviews spent, what
/// this review's finished runs were charged, and the tier caps of its
/// runs in flight, reserved until each ends. A run starts only once its
/// own tier's cap fits beside all of those, so runs started together
/// can never spend past the cap; one that does not fit waits for runs
/// in flight to end, and is refused once it cannot fit whatever they
/// cost.
#[derive(Debug)]
pub struct Budget {
    cap: f64,
    before: f64,
    ledger: Mutex<Ledger>,
    freed: Condvar,
}

/// What a review's runs were charged, and what its runs in flight
/// hold, in US dollars.
#[derive(Debug, Default)]
struct Ledger {
    spent: f64,
    reserved: f64,
}

/// What a sum of costs may pass the cap by and still fit, so that
/// adding up tier caps in floating point never refuses what fits.
const SLACK: f64 = 1e-9;

impl Budget {
    /// A cap of `cap` US dollars, `before` of it spent by the pull
    /// request's earlier reviews.
    #[must_use]
    pub fn new(cap: f64, before: f64) -> Self {
        Self {
            cap,
            before,
            ledger: Mutex::new(Ledger::default()),
            freed: Condvar::new(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Ledger> {
        self.ledger.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether `own`, of this review's, fits beside what the earlier
    /// reviews spent.
    fn within(&self, own: f64) -> bool {
        self.before + own <= self.cap + SLACK
    }

    /// Whether a run whose tier caps it at `most` US dollars may start
    /// now: its cap fits beside what was spent and what the runs in
    /// flight hold.
    #[must_use]
    pub fn fits(&self, most: f64) -> bool {
        let ledger = self.lock();
        self.within(ledger.spent + ledger.reserved + most)
    }

    /// Why a run capped at `most` US dollars may never start: its cap
    /// does not fit beside what was spent, whatever the runs in flight
    /// cost; none while it may.
    #[must_use]
    pub fn refusal(&self, most: f64) -> Option<String> {
        self.refused(&self.lock(), most)
    }

    fn refused(&self, ledger: &Ledger, most: f64) -> Option<String> {
        (!self.within(ledger.spent + most)).then(|| {
            format!(
                "{OVER_BUDGET} the pull request's reviews cost {:.4} USD, and a run capped at \
                 {most:.2} USD does not fit in its cap of {:.2} USD, so it was not started",
                self.before + ledger.spent,
                self.cap
            )
        })
    }

    /// Reserves `most` US dollars for a run about to start, once its
    /// cap fits: while runs in flight hold what it needs it waits for
    /// one to end, and once it cannot fit whatever they cost it is
    /// refused, why given. Each reservation is released by
    /// [`Budget::charge`].
    ///
    /// # Errors
    ///
    /// Fails, saying why, when the run may never start.
    pub fn reserve(&self, most: f64) -> Result<(), String> {
        let mut ledger = self.lock();
        loop {
            if let Some(why) = self.refused(&ledger, most) {
                return Err(why);
            }
            // Not refused, so what does not fit is held by runs in
            // flight, each of which ends and wakes this wait.
            if self.within(ledger.spent + ledger.reserved + most) {
                ledger.reserved += most;
                return Ok(());
            }
            ledger = self
                .freed
                .wait(ledger)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// Releases the `most` a run that ended reserved, charges it `cost`
    /// and hands `keep` the pull request's total so far, while no other
    /// run's cost is added, so what it keeps never goes back; then
    /// wakes the runs waiting for room.
    pub fn charge(&self, most: f64, cost: f64, keep: &dyn Fn(f64)) {
        let mut ledger = self.lock();
        ledger.reserved -= most;
        ledger.spent += cost;
        keep(self.before + ledger.spent);
        drop(ledger);
        self.freed.notify_all();
    }

    /// What this review's finished runs were charged.
    #[must_use]
    pub fn spent(&self) -> f64 {
        self.lock().spent
    }

    /// What the pull request's reviews cost so far, this one's finished
    /// runs and the earlier reviews'.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.before + self.spent()
    }
}

/// The circuit breaker: how many of the dispatched specialists ended,
/// how many of them in a gap, and whether that is more than its
/// fraction of them.
#[derive(Debug)]
pub struct Breaker {
    fraction: Fraction,
    dispatched: usize,
    /// `(failed, ended)`, kept together so every read is one snapshot.
    counts: Mutex<(usize, usize)>,
}

impl Breaker {
    /// A breaker over `dispatched` specialists that trips past
    /// `fraction` of them.
    #[must_use]
    pub fn new(fraction: Fraction, dispatched: usize) -> Self {
        Self {
            fraction,
            dispatched,
            counts: Mutex::new((0, 0)),
        }
    }

    fn snapshot(&self) -> (usize, usize) {
        *self.counts.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Counts one more specialist ended, in a gap when `failed`.
    pub fn end(&self, failed: bool) {
        let mut counts = self.counts.lock().unwrap_or_else(PoisonError::into_inner);
        counts.0 += usize::from(failed);
        counts.1 += 1;
    }

    /// Whether the breaker has tripped, or can no longer trip however
    /// the specialists still running end: until then no generalist
    /// starts, so whether one does never hangs on which ended first.
    #[must_use]
    pub fn decided(&self) -> bool {
        let (failed, ended) = self.snapshot();
        let running = self.dispatched.saturating_sub(ended);
        trips(failed, self.dispatched, self.fraction)
            || !trips(failed + running, self.dispatched, self.fraction)
    }

    /// Why a generalist may not start, once the breaker tripped; none
    /// before.
    #[must_use]
    pub fn refusal(&self) -> Option<String> {
        let (failed, _) = self.snapshot();
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
