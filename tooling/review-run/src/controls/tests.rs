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
fn the_budget_refuses_a_run_whose_cap_no_longer_fits_beside_what_was_spent() {
    let budget = Budget::new(1.0, 0.0);
    assert_eq!(budget.refusal(0.5), None);

    budget.reserve(0.5).unwrap();
    budget.charge(0.5, 0.4, &|_| {});
    assert_eq!(budget.refusal(0.6), None, "0.4 and 0.6 is the cap, no more");
    budget.reserve(0.6).unwrap();
    budget.charge(0.6, 0.2, &|_| {});

    assert!((budget.spent() - 0.6).abs() < 1e-9);
    assert_eq!(
        budget.refusal(0.5).as_deref(),
        Some(
            "over budget: the pull request's reviews cost 0.6000 USD, and a run capped at 0.50 \
             USD does not fit in its cap of 1.00 USD, so it was not started"
        )
    );
    assert_eq!(budget.reserve(0.5), Err(budget.refusal(0.5).unwrap()));
}

#[test]
fn a_run_fits_only_beside_the_caps_the_runs_in_flight_hold() {
    let budget = Budget::new(1.0, 0.0);

    budget.reserve(0.3).unwrap();
    budget.reserve(0.3).unwrap();
    budget.reserve(0.3).unwrap();

    assert!(!budget.fits(0.3), "three in flight hold 0.9");
    assert!(budget.fits(0.1));
    assert_eq!(budget.refusal(0.3), None, "it waits, it is not refused");
    budget.charge(0.3, 0.05, &|_| {});
    assert!(budget.fits(0.3), "0.05 spent and 0.6 held leave room");
    // Sixteen specialists' caps fit in a pull request's cap, summed in
    // floating point; a seventeenth does not.
    let sixteen = Budget::new(PULL_REQUEST_COST_USD, 0.0);
    for _ in 0..16 {
        assert!(sixteen.fits(0.3));
        sixteen.reserve(0.3).unwrap();
    }
    assert!(!sixteen.fits(0.3));
}

#[test]
fn a_reservation_that_does_not_fit_waits_for_a_run_in_flight_to_end() {
    let budget = Budget::new(1.0, 0.0);
    budget.reserve(0.8).unwrap();

    std::thread::scope(|scope| {
        let waiting = scope.spawn(|| budget.reserve(0.5));
        // It cannot be reserved while 0.8 is held, so it waits until the
        // run in flight ends, however long that takes here.
        std::thread::sleep(Duration::from_millis(20));
        assert!(!waiting.is_finished(), "it waits for room");
        budget.charge(0.8, 0.1, &|_| {});
        assert_eq!(waiting.join().unwrap(), Ok(()));
    });
    assert!(!budget.fits(0.5), "0.1 spent and 0.5 held");
}

#[test]
fn a_waiting_reservation_is_refused_once_what_ended_leaves_no_room() {
    let budget = Budget::new(1.0, 0.0);
    budget.reserve(0.8).unwrap();

    std::thread::scope(|scope| {
        let waiting = scope.spawn(|| budget.reserve(0.5));
        std::thread::sleep(Duration::from_millis(20));
        budget.charge(0.8, 0.7, &|_| {});
        let refused = waiting.join().unwrap().unwrap_err();
        assert!(refused.starts_with("over budget: "), "{refused}");
    });
}

#[test]
fn the_budget_counts_what_the_pull_request_s_earlier_reviews_spent() {
    let budget = Budget::new(1.0, 0.5);
    assert_eq!(budget.refusal(0.5), None);
    let kept = std::sync::Mutex::new(Vec::new());

    budget.reserve(0.5).unwrap();
    budget.charge(0.5, 0.25, &|total| kept.lock().unwrap().push(total));

    assert!((budget.spent() - 0.25).abs() < 1e-9, "this review's own");
    assert!((budget.total() - 0.75).abs() < 1e-9);
    assert_eq!(kept.into_inner().unwrap(), [0.25], "this review's own");
    assert!(budget.refusal(0.3).is_some());
    assert!(
        Budget::new(1.0, 1.0).refusal(0.01).is_some(),
        "spent before it ran"
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
