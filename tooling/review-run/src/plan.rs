//! What the review runs: the dispatch the review bundle wrote, each
//! generalist with the specialists whose reports it weighs, and the
//! review class and touched ids the bundle printed, which a
//! generalist's prompt names.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use review_bundle::dispatch::Dispatch;
use review_eval::component::{Kind, kind_of};

use crate::{CLASS, REPORTS, REVIEW};

/// The review classes the bundle sets.
const CLASSES: [&str; 3] = ["light", "standard", "deep"];

/// One review's plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub class: String,
    /// How many ids the change touches, as the bundle counts them.
    pub touched: usize,
    pub dispatch: Dispatch,
}

impl Plan {
    /// Every dispatched specialist, sorted, each once.
    #[must_use]
    pub fn specialists(&self) -> Vec<String> {
        let all: BTreeSet<&String> = self.dispatch.values().flatten().collect();
        all.into_iter().cloned().collect()
    }

    /// Every dispatched generalist, sorted.
    #[must_use]
    pub fn generalists(&self) -> Vec<String> {
        self.dispatch.keys().cloned().collect()
    }

    /// Every dispatched agent.
    #[must_use]
    pub fn agents(&self) -> BTreeSet<String> {
        review_bundle::dispatch::agents(&self.dispatch)
    }

    /// The report files of `generalist`'s specialists, relative to the
    /// work directory.
    #[must_use]
    pub fn reports_of(&self, generalist: &str) -> Vec<String> {
        self.dispatch
            .get(generalist)
            .into_iter()
            .flatten()
            .map(|s| report_file(s))
            .collect()
    }
}

/// `specialist`'s report file, relative to the work directory.
#[must_use]
pub fn report_file(specialist: &str) -> String {
    format!("{REPORTS}/{specialist}.md")
}

/// Reads the plan from the work directory `workdir`: the dispatch
/// under `review/` and the bundle's printed output beside it.
///
/// # Errors
///
/// Fails when either file cannot be read or does not read as its shape.
pub fn read(workdir: &Path) -> Result<Plan, String> {
    let load = |rel: &str| {
        let path = workdir.join(rel);
        fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))
            .map(|text| (path, text))
    };
    let (path, text) = load(&format!("{REVIEW}/{}", review_bundle::cli::DISPATCH))?;
    let dispatch = parse_dispatch(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let (path, text) = load(CLASS)?;
    let (class, touched) = parse_class(&text).map_err(|e| format!("{}: {e}", path.display()))?;

    Ok(Plan {
        class,
        touched,
        dispatch,
    })
}

/// Reads a dispatch: a map of generalists to lists of specialists,
/// each name an agent's, so it can name a file, and each in its place.
///
/// # Errors
///
/// Fails on anything else.
pub fn parse_dispatch(text: &str) -> Result<Dispatch, String> {
    let dispatch: Dispatch =
        serde_json::from_str(text).map_err(|e| format!("malformed dispatch: {e}"))?;
    for (generalist, specialists) in &dispatch {
        agent_name(generalist)?;
        if kind_of(generalist) != Kind::Generalist {
            return Err(format!("malformed dispatch: {generalist} is no generalist"));
        }
        for specialist in specialists {
            agent_name(specialist)?;
            if kind_of(specialist) != Kind::Specialist {
                return Err(format!("malformed dispatch: {specialist} is no specialist"));
            }
        }
    }

    Ok(dispatch)
}

/// Refuses `name` unless it is lowercase letters, digits and hyphens.
fn agent_name(name: &str) -> Result<(), String> {
    let fits = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if fits {
        Ok(())
    } else {
        Err(format!("malformed dispatch: agent name {name:?}"))
    }
}

/// Reads the review class and the touched count from the bundle's
/// printed output, `key=value` per line.
///
/// # Errors
///
/// Fails when either is missing, the class is not one the bundle sets,
/// or the count is not a number.
pub fn parse_class(text: &str) -> Result<(String, usize), String> {
    let values: BTreeMap<&str, &str> = text
        .lines()
        .filter_map(|l| l.trim().split_once('='))
        .collect();
    let class = *values.get("class").ok_or("names no class")?;
    if !CLASSES.contains(&class) {
        return Err(format!(
            "class {class:?} is none of light, standard and deep"
        ));
    }
    let touched = values.get("touched").ok_or("names no touched count")?;
    let touched = touched
        .parse()
        .map_err(|_| format!("touched {touched:?} is not a number"))?;

    Ok((class.to_owned(), touched))
}

#[cfg(test)]
mod tests;
