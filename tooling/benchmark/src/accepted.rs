//! Reviewed acceptances of a regression (ENG-15): a committed record
//! that lets one benchmark's head run slower than the gate's 10%, up to
//! a ceiling, for a stated reason. A change that makes ParleyInn slower on
//! purpose, such as linking the store into the core, records it in
//! `tooling/benchmark/accepted.txt`, where review sees it.
//!
//! An acceptance counts only while the base does not carry the same
//! record: once the change lands, the next change's base has it, so it
//! accepts nothing more, and the step it accepted becomes the new base.
//!
//! The format is one acceptance a line, `<benchmark> <percent>
//! <reason>`; blank lines and lines starting with `#` are left out.

use crate::gate::THRESHOLD_PERCENT;

/// One accepted regression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acceptance {
    pub benchmark: String,
    /// How much slower than the base, in percent of its median, the
    /// head may run.
    pub up_to_percent: u128,
    pub reason: String,
}

/// The acceptances `text` records.
///
/// # Errors
///
/// Fails, naming the line, on one that names no benchmark, ceiling and
/// reason, or whose ceiling the gate's threshold already allows.
pub fn parse(text: &str) -> Result<Vec<Acceptance>, String> {
    let mut out = Vec::new();
    for (n, line) in (1..).zip(text.lines()) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut words = line.splitn(3, char::is_whitespace);
        let (Some(benchmark), Some(percent), Some(reason)) =
            (words.next(), words.next(), words.next())
        else {
            return Err(format!("line {n}: want <benchmark> <percent> <reason>"));
        };
        let up_to_percent: u128 = percent
            .parse()
            .map_err(|_| format!("line {n}: {percent:?} is not a whole percent"))?;
        if up_to_percent <= THRESHOLD_PERCENT {
            return Err(format!(
                "line {n}: {up_to_percent}% is no more than the gate's {THRESHOLD_PERCENT}%, so it accepts nothing"
            ));
        }
        out.push(Acceptance {
            benchmark: benchmark.to_owned(),
            up_to_percent,
            reason: reason.trim().to_owned(),
        });
    }

    Ok(out)
}

/// The acceptances in `head` for a benchmark no line of `base` names.
/// A base line, however a later change rewords it or raises its ceiling,
/// keeps its benchmark's acceptance spent; accepting that benchmark
/// again takes removing its line first.
#[must_use]
pub fn fresh(head: &[Acceptance], base: &[Acceptance]) -> Vec<Acceptance> {
    head.iter()
        .filter(|a| !base.iter().any(|b| b.benchmark == a.benchmark))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests;
