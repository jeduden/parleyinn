use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

/// Runs `waits` at `bound`, each task recording when it started and
/// ended and answering its number with the sum of its inputs.
fn run(waits: &[Vec<usize>], bound: usize) -> (Vec<usize>, Vec<String>) {
    let events = Mutex::new(Vec::new());
    let out = pipeline(waits, bound, &|_| true, &|at, inputs: Vec<usize>| {
        events.lock().unwrap().push(format!("start {at}"));
        let out = at * 100 + inputs.iter().sum::<usize>();
        events.lock().unwrap().push(format!("end {at}"));
        out
    });
    (out, events.into_inner().unwrap())
}

#[test]
fn results_come_in_the_tasks_order_each_given_what_it_waits_for() {
    let (out, _) = run(&[vec![], vec![], vec![0, 1]], 4);

    assert_eq!(out, [0, 100, 300]);
}

#[test]
fn a_task_starts_only_once_every_task_it_waits_for_ended() {
    let waits = [vec![], vec![], vec![0], vec![1], vec![2, 3]];
    for bound in [1, 2, 8] {
        let (_, events) = run(&waits, bound);
        let place = |e: String| events.iter().position(|x| *x == e).unwrap();
        for (at, waited) in waits.iter().enumerate() {
            for w in waited {
                assert!(
                    place(format!("end {w}")) < place(format!("start {at}")),
                    "{bound}: {events:?}"
                );
            }
        }
    }
}

#[test]
fn one_at_a_time_the_lowest_ready_task_runs_first() {
    let (_, events) = run(&[vec![], vec![0], vec![], vec![]], 1);

    let starts: Vec<&String> = events.iter().filter(|e| e.starts_with("start")).collect();
    assert_eq!(starts, ["start 0", "start 1", "start 2", "start 3"]);
}

#[test]
fn a_wait_on_a_later_task_or_on_itself_is_ignored() {
    let (out, _) = run(&[vec![1, 0], vec![1]], 2);

    assert_eq!(out, [0, 100]);
}

#[test]
fn nothing_runs_nothing_and_a_bound_of_zero_runs_one_at_a_time() {
    assert_eq!(run(&[], 3).0, Vec::<usize>::new());
    assert_eq!(run(&[vec![], vec![]], 0).0, [0, 100]);
}

#[test]
fn the_bound_holds() {
    let active = AtomicUsize::new(0);
    let most = AtomicUsize::new(0);
    let waits: Vec<Vec<usize>> = (0..12).map(|_| Vec::new()).collect();

    pipeline(&waits, 3, &|_| true, &|_, _: Vec<()>| {
        let now = active.fetch_add(1, Ordering::SeqCst) + 1;
        most.fetch_max(now, Ordering::SeqCst);
        std::thread::yield_now();
        active.fetch_sub(1, Ordering::SeqCst);
    });

    assert!(most.load(Ordering::SeqCst) <= 3);
}

#[test]
fn a_task_that_panics_stops_the_schedule_rather_than_hang_its_waiters() {
    let ran = AtomicUsize::new(0);
    let waits = [vec![], vec![0], vec![1]];

    let out = catch_unwind(AssertUnwindSafe(|| {
        pipeline(&waits, 2, &|_| true, &|at, _: Vec<()>| {
            ran.fetch_add(1, Ordering::SeqCst);
            assert!(at != 0, "task 0 fails");
        })
    }));

    assert!(out.is_err());
    assert_eq!(ran.load(Ordering::SeqCst), 1);
}

#[test]
fn a_gated_task_starts_once_its_gate_opens_after_a_task_ends() {
    let ended = AtomicUsize::new(0);
    let order = Mutex::new(Vec::new());
    let waits = [vec![], vec![], vec![]];

    // Task 0 opens only once both others ended.
    pipeline(
        &waits,
        3,
        &|at| at != 0 || ended.load(Ordering::SeqCst) == 2,
        &|at, _: Vec<()>| {
            order.lock().unwrap().push(at);
            ended.fetch_add(1, Ordering::SeqCst);
        },
    );

    assert_eq!(order.into_inner().unwrap().last(), Some(&0));
}

#[test]
fn a_gate_that_never_opens_still_lets_the_tasks_run_once_nothing_else_does() {
    let out = pipeline(&[vec![], vec![0]], 2, &|_| false, &|at, _: Vec<usize>| at);

    assert_eq!(out, [0, 1]);
}
