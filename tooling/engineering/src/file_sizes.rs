//! ENG-34: CI fails on a file past its kind's line limit that the
//! recorded sizes do not list, and on a listed file that has grown past
//! its recorded size; a listed file that now fits, or is gone, must
//! leave the record, so the record only shrinks. The limits are the
//! review bundle's, which the ENG-34 row states; the review bundle's own
//! finding for a changed file past its limit stays a nit.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use review_bundle::limits::{limit_of, line_count};

use crate::Checkout;

/// The recorded sizes, relative to the checkout: one file a line,
/// `<path> <lines>`, owned by the stakeholder (ENG-21).
pub const RECORD: &str = "tooling/engineering/oversize.txt";

/// The directories no file is read under, at any depth: git's own
/// store, build output and local state.
const NOT_SEARCHED: [&str; 3] = [".git", "target", ".tmp"];

/// Other checkouts of the repository, which are no part of this one,
/// relative to the root.
const OTHER_CHECKOUTS: &str = ".claude/worktrees";

/// The lines each file holds, by its path relative to the checkout.
pub type Sizes = BTreeMap<String, usize>;

/// The sizes `text`, the record, lists, and a problem for each line
/// that is no `<path> <lines>` or lists a path twice.
#[must_use]
pub fn parse_record(text: &str) -> (Sizes, Vec<String>) {
    let (mut sizes, mut problems) = (Sizes::new(), Vec::new());
    let entries = text
        .lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line.trim()))
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'));
    for (at, line) in entries {
        let words: Vec<&str> = line.split_whitespace().collect();
        let [path, lines] = words.as_slice() else {
            problems.push(format!(
                "{RECORD}:{at}: want `<path> <lines>`, got {line:?}"
            ));
            continue;
        };
        let Ok(lines) = lines.parse::<usize>() else {
            problems.push(format!("{RECORD}:{at}: {lines:?} is no line count"));
            continue;
        };
        if sizes.insert((*path).to_owned(), lines).is_some() {
            problems.push(format!("{RECORD}:{at}: {path} is listed twice"));
        }
    }

    (sizes, problems)
}

/// The lines of each file under `root` that has a limit.
///
/// # Errors
///
/// Fails when a directory or a file cannot be read: a limit is never
/// skipped unseen (I6).
pub fn measure(root: &Path) -> Result<Sizes, String> {
    let mut sizes = Sizes::new();
    walk(root, "", &mut sizes)?;

    Ok(sizes)
}

/// Adds the files of the directory `rel` of `root` to `sizes`.
fn walk(root: &Path, rel: &str, sizes: &mut Sizes) -> Result<(), String> {
    let dir = root.join(rel);
    let failed = |what: &str, e: std::io::Error| format!("{what} {}: {e}", dir.display());
    for entry in fs::read_dir(&dir).map_err(|e| failed("read", e))? {
        let entry = entry.map_err(|e| failed("read", e))?;
        let kind = entry.file_type().map_err(|e| failed("stat", e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        if kind.is_dir() && !NOT_SEARCHED.contains(&name.as_str()) && path != OTHER_CHECKOUTS {
            walk(root, &path, sizes)?;
        } else if kind.is_file() && limit_of(&path).is_some() {
            let bytes = fs::read(entry.path()).map_err(|e| failed("read", e))?;
            sizes.insert(path, line_count(&bytes));
        }
    }

    Ok(())
}

/// The problems of `sizes` against `record`: those of the files past
/// their limits, then those of the record.
#[must_use]
pub fn compare(sizes: &Sizes, record: &Sizes) -> Vec<String> {
    let (past, stale) = compare_parts(sizes, record);

    [past, stale].concat()
}

/// The problems of `sizes` against `record`, in two parts: each file
/// past its limit that the record does not list and each listed file
/// grown past its recorded lines; then each listed file that fits its
/// limit or is gone, which must leave the record.
#[must_use]
pub fn compare_parts(sizes: &Sizes, record: &Sizes) -> (Vec<String>, Vec<String>) {
    let mut problems = Vec::new();
    for (path, &lines) in sizes {
        let Some((kind, limit)) = limit_of(path).filter(|_| past_limit(path, lines)) else {
            continue;
        };
        match record.get(path) {
            None => problems.push(format!(
                "{path} holds {lines} lines, past the limit of {limit} for {kind}, and {RECORD} does not list it: split it — ENG-34"
            )),
            Some(&was) if lines > was => problems.push(format!(
                "{path} grew from its recorded {was} lines to {lines}, past the limit of {limit} for {kind}: split it — ENG-34"
            )),
            Some(_) => {}
        }
    }
    let mut stale = Vec::new();
    for path in record.keys() {
        match sizes.get(path) {
            None => stale.push(format!(
                "{path} is listed in {RECORD} but is gone or has no limit: remove it — ENG-34"
            )),
            Some(&lines) if !past_limit(path, lines) => stale.push(format!(
                "{path} now fits its limit with {lines} lines: remove it from {RECORD} — ENG-34"
            )),
            Some(_) => {}
        }
    }

    (problems, stale)
}

/// Whether a file at `path` holding `lines` is past its kind's limit.
fn past_limit(path: &str, lines: usize) -> bool {
    limit_of(path).is_some_and(|(_, limit)| lines > limit)
}

/// The problems of the checkout: its record against the sizes of its
/// files.
#[must_use]
pub fn check(checkout: &Checkout) -> Vec<String> {
    let (past, stale) = check_parts(checkout);

    [past, stale].concat()
}

/// The problems of the checkout in two parts, as [`compare_parts`]
/// gives them; a record or a tree that cannot be read is in the first.
#[must_use]
pub fn check_parts(checkout: &Checkout) -> (Vec<String>, Vec<String>) {
    check_with(checkout, measure)
}

/// [`check_parts`] with the tree measured by `measure`.
fn check_with(
    checkout: &Checkout,
    measure: impl Fn(&Path) -> Result<Sizes, String>,
) -> (Vec<String>, Vec<String>) {
    let text = match checkout.read(RECORD) {
        Ok(text) => text,
        Err(e) => return (vec![e], Vec::new()),
    };
    let (record, mut problems) = parse_record(&text);
    match measure(&checkout.root) {
        Ok(sizes) => {
            let (past, stale) = compare_parts(&sizes, &record);
            problems.extend(past);
            (problems, stale)
        }
        Err(e) => {
            problems.push(e);
            (problems, Vec::new())
        }
    }
}

/// The tree of the fixture, each file with its lines: one listed file
/// grown, one past its limit and not listed, one listed at its size,
/// one listed that fits and one within its limit, not listed.
const FIXTURE: [(&str, usize); 5] = [
    (GROWN, 1002),
    (UNLISTED, 1001),
    (HELD, 1001),
    (FITS, 10),
    (WITHIN, 1000),
];

/// The listed file that grew.
pub const GROWN: &str = "crates/grown/src/lib.rs";
/// The file past its limit that the record does not list.
pub const UNLISTED: &str = "crates/unlisted/src/lib.rs";
/// The listed file held at its recorded size.
pub const HELD: &str = "crates/held/src/lib.rs";
/// The listed file that fits its limit.
pub const FITS: &str = "crates/fits/src/lib.rs";
/// The file within its limit, which the record does not list.
pub const WITHIN: &str = "crates/within/src/lib.rs";
/// The listed file that is gone.
pub const GONE: &str = "crates/gone/src/lib.rs";

/// Writes the fixture and its record under `work` and returns the
/// problems the check finds in it.
///
/// # Errors
///
/// Fails when the tree cannot be written.
pub fn fixture_problems(work: &Path) -> Result<Vec<String>, String> {
    let mut files: Vec<(String, String)> = FIXTURE
        .iter()
        .map(|(path, lines)| ((*path).to_owned(), "x\n".repeat(*lines)))
        .collect();
    let record = format!("{GROWN} 1001\n{HELD} 1001\n{FITS} 1200\n{GONE} 1100\n");
    files.push((RECORD.to_owned(), record));
    for (rel, body) in files {
        let path = work.join(&rel);
        let failed = |e: std::io::Error| format!("write {}: {e}", path.display());
        fs::create_dir_all(path.parent().unwrap_or(work)).map_err(failed)?;
        fs::write(&path, body).map_err(failed)?;
    }

    Ok(check(&Checkout::new(work)))
}

/// The problems when `found`, the check's problems on the fixture, do
/// not name the grown, unlisted, fitting and gone files.
#[must_use]
pub fn names_each_drift(found: &[String]) -> Vec<String> {
    [GROWN, UNLISTED, FITS, GONE]
        .into_iter()
        .filter(|path| !found.iter().any(|p| p.starts_with(path)))
        .map(|path| {
            format!(
                "the size check does not name {path}, among: {}",
                found.join("; ")
            )
        })
        .collect()
}

/// The problems when `found`, the check's problems on the fixture, name
/// a file held at its size or within its limit.
#[must_use]
pub fn names_no_held_file(found: &[String]) -> Vec<String> {
    [HELD, WITHIN]
        .into_iter()
        .filter(|path| found.iter().any(|p| p.starts_with(path)))
        .map(|path| format!("the size check names {path}, which is within its record"))
        .collect()
}

#[cfg(test)]
mod tests;
