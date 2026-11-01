use std::time::Duration;

use super::*;

#[test]
fn the_system_clock_moves_forward_waits_and_draws_within_the_unit() {
    let clock = SystemClock::default();
    let before = clock.now();

    clock.sleep(Duration::from_millis(1));

    assert!(clock.now() >= before + Duration::from_millis(1));
    for _ in 0..100 {
        let draw = clock.jitter();
        assert!((0.0..1.0).contains(&draw), "{draw}");
    }
}

#[test]
fn a_draw_is_its_bits_in_millionths_below_one() {
    assert!(unit(0).abs() < f64::EPSILON);
    assert!((unit(500_000) - 0.5).abs() < f64::EPSILON);
    assert!((unit(1_999_999) - 0.999_999).abs() < f64::EPSILON);
}

#[test]
fn the_budget_refuses_a_run_once_its_finished_runs_reach_the_cap() {
    let budget = Budget::new(1.0);
    assert_eq!(budget.refusal(), None);

    budget.charge(0.6);
    assert_eq!(budget.refusal(), None);
    budget.charge(0.4);

    assert!((budget.spent() - 1.0).abs() < 1e-9);
    assert_eq!(
        budget.refusal().as_deref(),
        Some(
            "over budget: the review's runs cost 1.0000 USD, its cap is 1.00 USD, so it was not \
             started"
        )
    );
}

#[test]
fn the_breaker_is_decided_once_it_tripped_or_can_no_longer_trip() {
    let breaker = Breaker::new(BREAKER_FRACTION, 6);
    assert!(!breaker.decided(), "six may still fail");
    for _ in 0..3 {
        breaker.end(false);
    }
    assert!(!breaker.decided(), "the three running may still fail");
    breaker.end(false);
    assert!(breaker.decided(), "two of six is a third, no more");

    let tripped = Breaker::new(BREAKER_FRACTION, 6);
    for _ in 0..3 {
        tripped.end(true);
    }
    assert!(tripped.decided());
    assert!(Breaker::new(BREAKER_FRACTION, 0).decided());
}

#[test]
fn the_breaker_trips_once_more_than_its_fraction_of_specialists_failed() {
    let breaker = Breaker::new(BREAKER_FRACTION, 6);
    breaker.end(true);
    breaker.end(true);
    breaker.end(false);
    assert_eq!(breaker.refusal(), None, "two of six is a third, not more");

    breaker.end(true);

    assert_eq!(
        breaker.refusal().as_deref(),
        Some(
            "aborted: 3 of 6 specialists failed (mass failure; likely an outage or a broken \
             prompt), so it was not started"
        )
    );
}

#[test]
fn a_breaker_over_no_specialist_never_trips() {
    let zero = Fraction { num: 0, den: 1 };
    assert!(!trips(0, 0, BREAKER_FRACTION));
    assert!(trips(1, 3, zero));
    assert!(!trips(1, 3, BREAKER_FRACTION));
    assert!(trips(2, 3, BREAKER_FRACTION));
    assert!(!trips(3, 3, Fraction::WHOLE));
}

#[test]
fn more_than_a_third_trips_in_whole_numbers_and_exactly_a_third_does_not() {
    assert!(trips(17, 50, BREAKER_FRACTION), "17 of 50 is over a third");
    assert!(!trips(16, 50, BREAKER_FRACTION));
    assert!(trips(18, 53, BREAKER_FRACTION), "18 of 53 is over a third");
    assert!(!trips(17, 53, BREAKER_FRACTION));
    assert!(!trips(2, 6, BREAKER_FRACTION), "exactly a third");
    assert!(!trips(17, 51, BREAKER_FRACTION), "exactly a third");
    assert!(trips(18, 51, BREAKER_FRACTION));
}

#[test]
fn a_breaker_of_fifty_trips_on_the_seventeenth_failure() {
    let breaker = Breaker::new(BREAKER_FRACTION, 50);
    for _ in 0..16 {
        breaker.end(true);
    }
    assert_eq!(breaker.refusal(), None);
    breaker.end(true);
    assert!(breaker.refusal().is_some());
}
