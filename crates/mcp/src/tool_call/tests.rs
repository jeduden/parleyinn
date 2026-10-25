use std::cell::RefCell;

use serde_json::json;

use super::*;

/// Runs that answer each tool-use id from a table, recording each wait.
struct Table {
    runs: Vec<(&'static str, &'static str)>,
    asked: RefCell<Vec<(String, Duration)>>,
}

impl Table {
    fn new(runs: &[(&'static str, &'static str)]) -> Self {
        Self {
            runs: runs.to_vec(),
            asked: RefCell::new(Vec::new()),
        }
    }
}

impl Runs for Table {
    fn serving(&self) -> bool {
        true
    }

    fn find(&self, tool_use_id: &str, wait: Duration) -> (Vec<String>, Duration) {
        self.asked.borrow_mut().push((tool_use_id.to_owned(), wait));
        let found: Vec<String> = self
            .runs
            .iter()
            .filter(|(id, _)| *id == tool_use_id)
            .map(|(_, run)| (*run).to_owned())
            .collect();
        let waited = if found.is_empty() {
            wait
        } else {
            Duration::ZERO
        };
        (found, waited)
    }
}

fn params(tool_use_id: Option<&str>) -> Value {
    let mut params = json!({"name": "event_search", "arguments": {"run": "s/other"}});
    if let Some(id) = tool_use_id {
        params["_meta"] = json!({"claudecode/toolUseId": id, "progressToken": 2});
    }
    params
}

#[test]
fn the_run_is_the_one_whose_line_carries_the_tool_use_id_never_the_arguments() {
    let runs = Table::new(&[("toolu_1", "s/main"), ("toolu_1", "s/main")]);

    let found = find(&runs, &params(Some("toolu_1")));

    assert_eq!(
        found,
        Found {
            tool_use_id: Some("toolu_1".to_owned()),
            run: Ok("s/main".to_owned()),
            waited: Duration::ZERO,
        }
    );
    assert_eq!(*runs.asked.borrow(), [("toolu_1".to_owned(), WAIT)]);
}

#[test]
fn a_call_with_no_tool_use_id_is_refused_without_waiting() {
    let runs = Table::new(&[]);

    for params in [params(None), params(Some(""))] {
        let found = find(&runs, &params);
        assert_eq!(found.tool_use_id, None);
        assert_eq!(found.run, Err(Cause::NoToolUseId));
        assert_eq!(found.waited, Duration::ZERO);
    }
    assert!(runs.asked.borrow().is_empty());
}

#[test]
fn no_line_in_time_or_lines_of_two_runs_are_refused_naming_the_cause() {
    let runs = Table::new(&[("toolu_2", "s/main"), ("toolu_2", "s/agent")]);

    let missing = find(&runs, &params(Some("toolu_9")));
    let ambiguous = find(&runs, &params(Some("toolu_2")));

    assert_eq!((missing.run, missing.waited), (Err(Cause::NotFound), WAIT));
    assert_eq!(ambiguous.run, Err(Cause::Ambiguous));
}

#[test]
fn each_cause_has_its_own_name_and_fixed_refusal() {
    let causes = [Cause::NoToolUseId, Cause::NotFound, Cause::Ambiguous];
    let names: std::collections::BTreeSet<&str> = causes.iter().map(|c| c.as_str()).collect();
    let refusals: std::collections::BTreeSet<&str> = causes.iter().map(|c| c.refusal()).collect();

    assert_eq!((names.len(), refusals.len()), (3, 3));
    assert!(refusals.iter().all(|r| r.contains("tool-use id")));
    assert!(runs_serving(&Table::new(&[])));
}

fn runs_serving(runs: &dyn Runs) -> bool {
    runs.serving()
}
