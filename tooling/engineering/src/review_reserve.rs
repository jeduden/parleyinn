//! ENG-28: a generalist starts on what is left of the pull request's
//! cost cap. Its tier's cap is twice what the dearest generalist on
//! record cost, so holding all of it refused a review its generalists
//! while a third of the pull request's cap was unspent. `review-run`
//! now starts a generalist whose tier's cap can never fit again once
//! half that cap is left, and passes it what is left as its own cost
//! cap, so the pull request's cap holds whatever the run costs. The
//! checks run `review-run` with the stand-in `claude` of
//! [`review_controls`], each run costing 0.01 US dollars, after earlier
//! reviews that spent what pull request 136's had, 4.10 of 5 US
//! dollars: every agent starts and each generalist is passed what was
//! left when it started; with 4.30 spent no generalist starts; and a
//! generalist killed in flight is charged what it held, not its tier's
//! cap.

use std::collections::BTreeMap;
use std::path::Path;

use review_eval::component::{Kind, kind_of};
use review_eval::host::Command;
use review_eval::runner::{GENERALIST_BUDGET_USD, GENERALIST_RESERVE_USD, budget_usd};
use review_run::controls::PULL_REQUEST_COST_USD;

use crate::Checkout;
use crate::review_controls::{ControlHost, gaps, review};
use crate::review_runs::agent_of;
use crate::review_spend::spend_of;

/// What the earlier reviews spent where a generalist's reserve is left
/// and its tier's cap is not: pull request 136's 4.10 US dollars.
const RESERVE_LEFT: f64 = 4.10;
/// What they spent where not even its reserve is left.
const NO_RESERVE_LEFT: f64 = 4.30;
/// How many agents the work directory dispatches, and what one run of
/// the stand-in costs.
const AGENTS: usize = 5;
const RUN_COST: f64 = 0.01;
/// The generalist killed in flight, and the log of its run: model calls
/// that used tokens and no result.
const KILLED: &str = "review-code";
const UNBILLED: &str = r#"{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":10,"output_tokens":5},"content":[{"type":"text","text":"weighing"}]}}"#;

/// The cost cap `command` passes its run, in US dollars; none when it
/// passes none that reads.
fn cap_of(command: &Command) -> Option<f64> {
    let mut after = (command.args.iter()).skip_while(|a| *a != "--max-budget-usd");
    after.nth(1).and_then(|cap| cap.parse().ok())
}

/// What is wrong with the cost cap one run of `agent` was passed,
/// `cap`, when `left` US dollars of the pull request's cap were left: a
/// specialist off its tier's cap, or a generalist passed other than
/// what was left, in whole cents.
#[must_use]
pub fn cap_problems(agent: &str, cap: Option<f64>, left: f64) -> Option<String> {
    let kind = kind_of(agent);
    let fits = cap.is_some_and(|cap| match kind {
        Kind::Specialist => budget_usd(kind).parse() == Ok(cap),
        Kind::Generalist | Kind::Consultant => (cap - left).abs() < RUN_COST / 2.0,
    });

    (!fits).then(|| {
        format!(
            "{agent} ran with the cost cap {cap:?}, though {left:.2} USD was left of the pull \
             request's cap"
        )
    })
}

/// What is left of `left` US dollars once `ended` runs of the stand-in
/// were charged [`RUN_COST`] each: what the next run starts on, as the
/// runs start one at a time.
fn left_after(left: f64, ended: usize) -> f64 {
    #[allow(clippy::cast_precision_loss, reason = "a handful of runs")]
    let charged = RUN_COST * ended as f64;
    left - charged
}

/// What is wrong with a review that started with a generalist's
/// reserve left, its runs' `commands` and what it printed, `out`, and
/// kept as spent, `spend`: an agent not started, a run capped wrongly,
/// or a spend file that is not what its runs cost.
#[must_use]
pub fn started_problems(
    commands: &[Command],
    out: &str,
    spend: &Result<f64, String>,
) -> Vec<String> {
    let left = PULL_REQUEST_COST_USD - RESERVE_LEFT;
    let mut problems = Vec::new();
    if commands.len() != AGENTS || !out.contains("\nover_budget=0\n") {
        problems.push(format!(
            "{} of {AGENTS} agents started with {left:.2} USD left, a generalist's reserve and \
             more: {out}",
            commands.len()
        ));
    }
    problems.extend(
        (commands.iter().enumerate()).filter_map(|(ended, c)| {
            cap_problems(&agent_of(c), cap_of(c), left_after(left, ended))
        }),
    );
    #[allow(clippy::cast_precision_loss, reason = "a handful of runs")]
    let cost = RUN_COST * AGENTS as f64;
    match spend {
        Ok(got) if (got - cost).abs() < 1e-9 => {}
        Ok(got) => problems.push(format!(
            "review/spend.json says {got} USD, not the {cost} its runs cost"
        )),
        Err(e) => problems.push(e.clone()),
    }

    problems
}

/// What is wrong with a review that started with less than a
/// generalist's reserve left, its runs' `commands` and its `gaps`: a
/// generalist started, or one not named over budget with the reserve
/// it starts with.
#[must_use]
pub fn refused_problems(commands: &[Command], gaps: &BTreeMap<String, String>) -> Vec<String> {
    let reserve = format!("starts with {GENERALIST_RESERVE_USD:.2} USD at least");
    let mut problems: Vec<String> = commands
        .iter()
        .map(agent_of)
        .filter(|agent| kind_of(agent) == Kind::Generalist)
        .map(|agent| format!("{agent} started with less than its reserve left"))
        .collect();
    let named = gaps
        .iter()
        .filter(|(name, _)| kind_of(name) == Kind::Generalist)
        .filter(|(_, why)| why.starts_with("over budget: ") && why.contains(&reserve))
        .count();
    if named != 2 {
        problems.push(format!(
            "{named} generalists named over budget with the reserve they start with, not 2: \
             {gaps:?}"
        ));
    }

    problems
}

/// What is wrong with a review one of whose generalists was killed in
/// flight on what was left, what it printed, `out`, and kept as spent,
/// `spend`: a spend past what was left, as its tier's cap charged would
/// be, or under it, or a review that does not request changes.
#[must_use]
pub fn killed_problems(out: &str, spend: &Result<f64, String>) -> Vec<String> {
    let left = PULL_REQUEST_COST_USD - RESERVE_LEFT;
    let mut problems = Vec::new();
    match spend {
        Ok(got) if (got - left).abs() < 1e-9 => {}
        Ok(got) => problems.push(format!(
            "review/spend.json says {got} USD after a generalist was killed on what was left, \
             not the {left:.2} left: its tier's cap is {GENERALIST_BUDGET_USD:.2}"
        )),
        Err(e) => problems.push(e.clone()),
    }
    if !out.starts_with("verdict=request_changes\n") {
        problems.push(format!(
            "a review whose generalist was killed with nothing left does not request changes: \
             {out}"
        ));
    }

    problems
}

/// What is wrong with how `checkout`'s `review-run` starts generalists
/// on what is left of the pull request's cap, run in `workdir`.
#[must_use]
pub fn reserve_problems(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    let run = |name: &str, before: f64, host: &ControlHost| {
        let dir = workdir.join(name);
        let before = before.to_string();
        let flags = ["--parallel", "1", "--spent-before", before.as_str()];
        review(checkout, &dir, host, &flags).map(|out| (dir, out))
    };
    let (started, refused) = (ControlHost::default(), ControlHost::default());
    let mut killed = ControlHost::default();
    killed.fails.insert(KILLED, (1, Some(UNBILLED)));
    let checked = [
        run("started", RESERVE_LEFT, &started)
            .map(|(dir, out)| started_problems(&started.commands(), &out, &spend_of(&dir))),
        run("refused", NO_RESERVE_LEFT, &refused)
            .and_then(|(dir, _)| Ok(refused_problems(&refused.commands(), &gaps(&dir)?))),
        run("killed", RESERVE_LEFT, &killed)
            .map(|(dir, out)| killed_problems(&out, &spend_of(&dir))),
    ];

    checked
        .into_iter()
        .flat_map(|found| found.unwrap_or_else(|e| vec![e]))
        .collect()
}

#[cfg(test)]
mod tests;
