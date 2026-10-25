use super::*;

fn ms(values: &[u64]) -> Vec<Duration> {
    values.iter().map(|v| Duration::from_millis(*v)).collect()
}

#[test]
fn the_median_of_an_odd_count_is_its_middle_sample() {
    assert_eq!(twice_median_nanos(&ms(&[9, 1, 5])), Some(10_000_000));
}

#[test]
fn the_median_of_an_even_count_is_the_mean_of_its_middle_two() {
    assert_eq!(twice_median_nanos(&ms(&[4, 1, 2, 30])), Some(6_000_000));
}

#[test]
fn no_samples_have_no_median() {
    assert_eq!(twice_median_nanos(&[]), None);
}

#[test]
fn a_program_at_the_limit_passes() {
    assert_eq!(within_multiple(&ms(&[50]), &ms(&[10]), 5), Ok(()));
}

#[test]
fn a_program_past_the_limit_fails_naming_the_medians() {
    let err = within_multiple(&ms(&[51]), &ms(&[10]), 5).unwrap_err();

    assert!(
        err.contains("51ms") && err.contains("10ms") && err.contains("5 times"),
        "{err}"
    );
}

#[test]
fn one_outlier_of_the_program_does_not_move_its_median() {
    let program = ms(&[40, 41, 900, 42, 43]);

    assert_eq!(within_multiple(&program, &ms(&[10; 5]), 5), Ok(()));
}

#[test]
fn noise_that_slows_both_cancels() {
    let (quiet_program, quiet_reference) = (ms(&[10]), ms(&[2]));
    let (slow_program, slow_reference) = (ms(&[36]), ms(&[28]));

    assert_eq!(within_multiple(&quiet_program, &quiet_reference, 5), Ok(()));
    assert_eq!(within_multiple(&slow_program, &slow_reference, 5), Ok(()));
}

#[test]
fn either_side_without_samples_fails() {
    assert!(within_multiple(&[], &ms(&[1]), 5).is_err());
    assert!(within_multiple(&ms(&[1]), &[], 5).is_err());
}

/// A timing double that logs `name` each time it runs and takes the
/// next of `times`.
fn double<'a>(
    name: &'static str,
    log: &'a std::cell::RefCell<Vec<&'static str>>,
    times: Vec<Result<u64, &'static str>>,
) -> impl FnMut() -> Result<Duration, String> + 'a {
    let mut times = times.into_iter();
    move || {
        log.borrow_mut().push(name);
        match times.next() {
            Some(Ok(t)) => Ok(Duration::from_millis(t)),
            Some(Err(e)) => Err(e.to_owned()),
            None => Err("out of times".to_owned()),
        }
    }
}

#[test]
fn rounds_alternate_which_of_the_pair_runs_first() {
    let log = std::cell::RefCell::new(Vec::new());
    let mut reference = double("ref", &log, vec![Ok(1); 3]);
    let mut program = double("prog", &log, vec![Ok(9); 3]);

    let (got_program, got_reference) = interleaved(3, &mut reference, &mut program).unwrap();

    assert_eq!(*log.borrow(), ["ref", "prog", "prog", "ref", "ref", "prog"]);
    assert_eq!((got_program.len(), got_reference.len()), (3, 3));
    assert_eq!(got_program[0], Duration::from_millis(9));
    assert_eq!(got_reference[0], Duration::from_millis(1));
}

#[test]
fn a_timing_that_fails_fails_the_rounds() {
    let log = std::cell::RefCell::new(Vec::new());
    let mut reference = double("ref", &log, vec![Ok(1), Ok(1)]);
    let mut program = double("prog", &log, vec![Ok(9), Err("hook exited 1")]);

    let err = interleaved(2, &mut reference, &mut program).unwrap_err();

    assert_eq!(err, "hook exited 1");
}

#[test]
fn a_hook_made_slow_against_the_reference_fails_the_check() {
    let log = std::cell::RefCell::new(Vec::new());
    // Confinement adds 27 ms to both; the slowed program adds 60 more.
    let mut reference = double("ref", &log, vec![Ok(29); 9]);
    let mut slowed = double("prog", &log, vec![Ok(27 + 60 + 9); 9]);
    let (slow, base) = interleaved(9, &mut reference, &mut slowed).unwrap();
    assert!(within_multiple(&slow, &base, 3).is_err());

    let mut reference = double("ref", &log, vec![Ok(29); 9]);
    let mut healthy = double("prog", &log, vec![Ok(27 + 9); 9]);
    let (fine, base) = interleaved(9, &mut reference, &mut healthy).unwrap();
    assert_eq!(within_multiple(&fine, &base, 3), Ok(()));
}
