//! The schedule: tasks run at most a bound at once, each once every
//! task it waits for has ended and its gate lets it, the lowest ready
//! one first, and their results come back in the tasks' order, not the
//! order they ended. A gate is asked again each time a task ends. A
//! task waits only for tasks before it, and when nothing runs and no
//! gate opens the lowest task whose waits are over starts all the same,
//! so the schedule always moves. A task that panics stops the schedule,
//! so no task waits for its result forever.

use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;

/// What the workers share.
struct State<R> {
    started: Vec<bool>,
    done: Vec<Option<R>>,
    /// How many tasks are running.
    running: usize,
    /// Set once a task panicked: no other task starts.
    abandoned: bool,
}

/// The schedule's shared state and the signal a worker waits on.
struct Board<'a, R> {
    waits: &'a [Vec<usize>],
    gate: &'a (dyn Fn(usize) -> bool + Sync),
    state: Mutex<State<R>>,
    changed: Condvar,
}

/// Stops the schedule when the task it guards panics.
struct Guard<'a, 'b, R> {
    board: &'b Board<'a, R>,
}

impl<R> Drop for Guard<'_, '_, R> {
    fn drop(&mut self) {
        if thread::panicking() {
            let board = self.board;
            board
                .state
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .abandoned = true;
            board.changed.notify_all();
        }
    }
}

impl<R: Clone> Board<'_, R> {
    fn lock(&self) -> MutexGuard<'_, State<R>> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The lowest task not started whose every earlier task it waits
    /// for has ended and, unless nothing runs, whose gate is open.
    fn ready(&self, state: &State<R>) -> Option<usize> {
        let waited =
            |at: usize| !state.started[at] && self.waits_of(at).all(|w| state.done[w].is_some());
        let open = (0..state.started.len()).find(|&at| waited(at) && (self.gate)(at));
        match open {
            None if state.running == 0 => (0..state.started.len()).find(|&at| waited(at)),
            open => open,
        }
    }

    /// The tasks `at` waits for: those before it alone.
    fn waits_of(&self, at: usize) -> impl Iterator<Item = usize> + '_ {
        self.waits[at].iter().copied().filter(move |&w| w < at)
    }

    /// The next task for a worker, marked started, with the results of
    /// the tasks it waits for, once one is ready; none once every task
    /// has started or the schedule was abandoned.
    fn next(&self) -> Option<(usize, Vec<R>)> {
        let mut state = self.lock();
        loop {
            if state.abandoned || state.started.iter().all(|s| *s) {
                return None;
            }
            if let Some(at) = self.ready(&state) {
                state.started[at] = true;
                state.running += 1;
                let inputs = self
                    .waits_of(at)
                    .filter_map(|w| state.done[w].clone())
                    .collect();
                return Some((at, inputs));
            }
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// Keeps task `at`'s result and wakes the waiting workers.
    fn finish(&self, at: usize, out: R) {
        let mut state = self.lock();
        state.done[at] = Some(out);
        state.running -= 1;
        drop(state);
        self.changed.notify_all();
    }
}

/// `f` of each task, task `at` given the results of the tasks
/// `waits[at]` names before it, started once `gate` of it holds, at
/// most `bound` at once, in the order of the tasks.
pub fn pipeline<R: Clone + Send>(
    waits: &[Vec<usize>],
    bound: usize,
    gate: &(dyn Fn(usize) -> bool + Sync),
    f: &(dyn Fn(usize, Vec<R>) -> R + Sync),
) -> Vec<R> {
    let count = waits.len();
    let board = Board {
        waits,
        gate,
        state: Mutex::new(State {
            started: vec![false; count],
            done: (0..count).map(|_| None).collect(),
            running: 0,
            abandoned: false,
        }),
        changed: Condvar::new(),
    };
    let worker = || {
        while let Some((at, inputs)) = board.next() {
            let guard = Guard { board: &board };
            let out = f(at, inputs);
            drop(guard);
            board.finish(at, out);
        }
    };
    thread::scope(|scope| {
        for _ in 0..bound.max(1).min(count) {
            scope.spawn(worker);
        }
    });

    board
        .state
        .into_inner()
        .unwrap_or_else(PoisonError::into_inner)
        .done
        .into_iter()
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests;
