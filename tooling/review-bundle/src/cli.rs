//! The `review-bundle` command line: two trees, a diff and the
//! stakeholder's comments in; the review bundle and the deterministic
//! findings written to a directory, and the review class printed for the
//! workflow's step outputs.
//!
//! `--consent` names a JSON array of the comments the CODEOWNERS owners
//! wrote on the pull request, each `{"author", "at", "body"}` and, for a
//! review, its `state`, which the workflow filters before the command
//! reads it. One with a non-blank body, other than a review that
//! requests changes, consents: to each owned path it names (ENG-21),
//! and, whatever it names, to the change ENG-29 guards.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::Value;

use crate::{Snapshot, analyse, render};

/// The review bundle and the findings were written.
pub const EXIT_OK: u8 = 0;
/// A tree, the diff, the consent or an output could not be read or
/// written.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str =
    "usage: review-bundle --base DIR --head DIR --diff FILE --out DIR --consent FILE";

/// The flags, in the order the usage names them, each required.
const FLAGS: [&str; 5] = ["base", "head", "diff", "out", "consent"];

/// The state of a review that requests changes, as GitHub names it.
const CHANGES_REQUESTED: &str = "CHANGES_REQUESTED";

/// The review bundle's file name under `--out`.
pub const BUNDLE: &str = "bundle.md";
/// The deterministic findings' file name under `--out`.
pub const CHECKED: &str = "checked.json";
/// The agents the review runs, each generalist with its specialists,
/// which the review gate holds the review to (ENG-28).
pub const DISPATCH: &str = "dispatch.json";
/// The directory under `--out` that holds each specialist's diff,
/// `<its name>.diff`: only the changed files it reads.
pub const DIFFS: &str = "diffs";

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let [base, head, diff, out, consent] = match parse_flags(args) {
        Ok(values) => values,
        Err(msg) => {
            let _ = writeln!(stderr, "review-bundle: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    match build(
        Path::new(&base),
        Path::new(&head),
        Path::new(&diff),
        Path::new(&out),
        Path::new(&consent),
    ) {
        Ok(outputs) => {
            let _ = write!(stdout, "{outputs}");
            EXIT_OK
        }
        Err(msg) => {
            let _ = writeln!(stderr, "{msg}");
            EXIT_FAILURE
        }
    }
}

/// Reads both trees, the diff and the consent, writes the review bundle
/// and the findings under `out`, and returns the step outputs to print.
fn build(
    base: &Path,
    head: &Path,
    diff: &Path,
    out: &Path,
    consent: &Path,
) -> Result<String, String> {
    let base_tree = Snapshot::read(base).map_err(|e| format!("{e} (main's tree)"))?;
    let head_tree = Snapshot::read(head).map_err(|e| format!("{e} (the pull request's tree)"))?;
    // The diff is read as bytes: git shows a file it takes for text, though
    // it is not UTF-8 (a fuzz seed, a fixture), and a review must not fail
    // for it. Each such byte reads as U+FFFD; every line of text is whole.
    let diff = fs::read(diff).map_err(failed("read", diff))?;
    let diff = String::from_utf8_lossy(&diff);
    let read = |path: &Path| fs::read_to_string(path).map_err(failed("read", path));
    let consent = consents(&read(consent)?).map_err(failed("read", consent))?;
    let a = analyse(&base_tree, &head_tree, &diff, &consent);
    let write = |name: &str, body: &str| {
        let path = out.join(name);
        fs::write(&path, body).map_err(failed("write", &path))
    };
    write(BUNDLE, &render::bundle(&a, &base_tree, &head_tree))?;
    let findings = serde_json::to_string_pretty(&a.findings).map_err(failed("encode", out))?;
    write(CHECKED, &format!("{findings}\n"))?;
    let dispatch = serde_json::to_string_pretty(&a.dispatch).map_err(failed("encode", out))?;
    write(DISPATCH, &format!("{dispatch}\n"))?;
    let diffs = out.join(DIFFS);
    fs::create_dir_all(&diffs).map_err(failed("write", &diffs))?;
    let tree = |path: &str| tree_file(head, path);
    for (specialist, scope) in &a.scopes {
        write(
            &format!("{DIFFS}/{specialist}.diff"),
            &diff_for(&diff, scope, &tree),
        )?;
    }

    Ok(format!(
        "class={}\ntouched={}\nfindings={}\nconsults={}\n",
        a.class.name(),
        a.touched.len(),
        a.findings.len(),
        crate::dispatch::agents(&a.dispatch).len()
    ))
}

/// The part of `diff` a specialist whose files are `scope` reads: those
/// files' sections, or the whole diff when it has none of its own, as a
/// specialist a row brings may not; either without the hunks inside a
/// copy an `<?include?>` makes of its source in the tree `tree` reads.
fn diff_for(diff: &str, scope: &BTreeSet<String>, tree: &dyn Fn(&str) -> Option<String>) -> String {
    let own = if scope.is_empty() {
        diff.to_owned()
    } else {
        crate::diff::only(diff, scope)
    };

    crate::copies::prune(&own, tree)
}

/// The text of the regular file at `path` in the tree at `root`, or
/// `None` when it is no regular file there or does not read.
fn tree_file(root: &Path, path: &str) -> Option<String> {
    let file = root.join(path);
    let regular = fs::symlink_metadata(&file).is_ok_and(|m| m.is_file());
    regular.then(|| fs::read_to_string(&file).ok()).flatten()
}

/// The message for a failure to `verb` the file at `path`.
fn failed<E: std::fmt::Display>(verb: &'static str, path: &Path) -> impl FnOnce(E) -> String {
    let path = path.display().to_string();
    move |e| format!("review-bundle: {verb} {path}: {e}")
}

/// The bodies of the comments in `json`, the owners' comments, that
/// consent: each non-blank one that is no review requesting changes.
/// Each comment must carry a string `author`, `at` and `body`.
fn consents(json: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let comments = value.as_array().ok_or("not an array of comments")?;
    let mut consent = Vec::new();
    for comment in comments {
        let field = |name: &str| {
            comment
                .get(name)
                .and_then(Value::as_str)
                .ok_or_else(|| format!("a comment without a string {name:?}"))
        };
        field("author")?;
        field("at")?;
        let objects = comment.get("state").and_then(Value::as_str) == Some(CHANGES_REQUESTED);
        let body = field("body")?;
        if !objects && !body.trim().is_empty() {
            consent.push(body.to_owned());
        }
    }

    Ok(consent)
}

/// Reads `--name value` and `--name=value` pairs into the values of
/// [`FLAGS`], each required.
fn parse_flags(args: impl IntoIterator<Item = String>) -> Result<[String; 5], String> {
    let mut values: [Option<String>; 5] = Default::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let Some(flag) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument {arg:?}"));
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (flag, None),
        };
        let slot = FLAGS
            .iter()
            .position(|f| *f == name)
            .ok_or_else(|| format!("unknown flag {arg:?}"))?;
        let value = inline
            .or_else(|| args.next())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("--{name} needs a value"))?;
        values[slot] = Some(value);
    }

    let mut out: [String; 5] = Default::default();
    for (i, value) in values.into_iter().enumerate() {
        out[i] = value.ok_or_else(|| format!("--{} is required", FLAGS[i]))?;
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
