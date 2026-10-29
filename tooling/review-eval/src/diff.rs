//! A unified diff of the files a case edits, in the shape `git diff`
//! writes, so the review bundle reads a fixture's change as it reads a
//! pull request's. Lines are compared whole; three lines of context
//! surround each hunk.

use std::fmt::Write as _;

/// The lines of context around a change.
const CONTEXT: usize = 3;

/// One file's change: its path, its text on main and its text in the
/// fixture, none for a file that does not exist on that side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub path: String,
    pub old: Option<String>,
    pub new: Option<String>,
}

/// The unified diff of `changes`, in path order, each file as
/// [`unified`] writes it.
#[must_use]
pub fn all(changes: &[Change]) -> String {
    let mut sorted: Vec<&Change> = changes.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    sorted
        .iter()
        .map(|c| unified(&c.path, c.old.as_deref(), c.new.as_deref()))
        .collect()
}

/// The unified diff of one file at `path`, from `old` to `new`; empty
/// when the two sides are the same.
#[must_use]
pub fn unified(path: &str, old: Option<&str>, new: Option<&str>) -> String {
    if old == new {
        return String::new();
    }
    let mut out = format!("diff --git a/{path} b/{path}\n");
    match (old, new) {
        (None, Some(_)) => out.push_str("new file mode 100644\n"),
        (Some(_), None) => out.push_str("deleted file mode 100644\n"),
        _ => {}
    }
    let _ = writeln!(
        out,
        "--- {}",
        old.map_or("/dev/null".to_owned(), |_| format!("a/{path}"))
    );
    let _ = writeln!(
        out,
        "+++ {}",
        new.map_or("/dev/null".to_owned(), |_| format!("b/{path}"))
    );
    let a = lines(old.unwrap_or_default());
    let b = lines(new.unwrap_or_default());
    for hunk in hunks(&edits(&a, &b)) {
        out.push_str(&render(&hunk, &a, &b));
    }

    out
}

/// A file's lines, each with whether a newline ended it.
fn lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split_inclusive('\n').collect()
}

/// One step of an edit script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    /// `a[i]` is `b[j]`.
    Equal(usize, usize),
    /// `a[i]` is gone.
    Delete(usize),
    /// `b[j]` is new.
    Insert(usize),
}

/// The edit script from `a` to `b`: the common head and tail kept, and
/// the middle by a longest common subsequence.
fn edits(a: &[&str], b: &[&str]) -> Vec<Op> {
    let head = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let tail = a[head..]
        .iter()
        .rev()
        .zip(b[head..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (ma, mb) = (&a[head..a.len() - tail], &b[head..b.len() - tail]);
    let mut out: Vec<Op> = (0..head).map(|i| Op::Equal(i, i)).collect();
    out.extend(middle(ma, mb).into_iter().map(|op| match op {
        Op::Equal(i, j) => Op::Equal(i + head, j + head),
        Op::Delete(i) => Op::Delete(i + head),
        Op::Insert(j) => Op::Insert(j + head),
    }));
    let (na, nb) = (a.len() - tail, b.len() - tail);
    out.extend((0..tail).map(|k| Op::Equal(na + k, nb + k)));

    out
}

/// The edit script of the differing middle, by a longest common
/// subsequence table.
fn middle(a: &[&str], b: &[&str]) -> Vec<Op> {
    let (n, m) = (a.len(), b.len());
    // lcs[i][j]: the longest common subsequence of a[i..] and b[j..].
    let mut lcs = vec![vec![0_usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i] == b[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if a[i] == b[j] {
            out.push(Op::Equal(i, j));
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            out.push(Op::Delete(i));
            i += 1;
        } else {
            out.push(Op::Insert(j));
            j += 1;
        }
    }
    out.extend((i..n).map(Op::Delete));
    out.extend((j..m).map(Op::Insert));

    out
}

/// A run of the edit script that one hunk shows: its ops, with the
/// context around its changes.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hunk {
    ops: Vec<Op>,
}

/// Groups the changes of `ops` into hunks, each with [`CONTEXT`] lines
/// of equal ops around it; two changes closer than twice the context
/// share a hunk.
fn hunks(ops: &[Op]) -> Vec<Hunk> {
    let changed: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, op)| !matches!(op, Op::Equal(..)))
        .map(|(at, _)| at)
        .collect();
    let mut out: Vec<Hunk> = Vec::new();
    let mut from: Option<(usize, usize)> = None;
    for &at in &changed {
        match from {
            Some((start, end)) if at <= end + 2 * CONTEXT => from = Some((start, at)),
            Some((start, end)) => {
                out.push(cut(ops, start, end));
                from = Some((at, at));
            }
            None => from = Some((at, at)),
        }
    }
    if let Some((start, end)) = from {
        out.push(cut(ops, start, end));
    }

    out
}

/// The hunk of `ops` whose first change is at `start` and last at
/// `end`, with the context on each side.
fn cut(ops: &[Op], start: usize, end: usize) -> Hunk {
    let lo = start.saturating_sub(CONTEXT);
    let hi = (end + CONTEXT + 1).min(ops.len());
    Hunk {
        ops: ops[lo..hi].to_vec(),
    }
}

/// `hunk` as text: its header, then each line marked.
fn render(hunk: &Hunk, a: &[&str], b: &[&str]) -> String {
    let first_a = hunk.ops.iter().find_map(|op| match op {
        Op::Equal(i, _) | Op::Delete(i) => Some(i + 1),
        Op::Insert(_) => None,
    });
    let first_b = hunk.ops.iter().find_map(|op| match op {
        Op::Equal(_, j) | Op::Insert(j) => Some(j + 1),
        Op::Delete(_) => None,
    });
    let count_a = hunk
        .ops
        .iter()
        .filter(|op| !matches!(op, Op::Insert(_)))
        .count();
    let count_b = hunk
        .ops
        .iter()
        .filter(|op| !matches!(op, Op::Delete(_)))
        .count();
    let mut out = format!(
        "@@ -{},{count_a} +{},{count_b} @@\n",
        first_a.unwrap_or(0),
        first_b.unwrap_or(0)
    );
    for op in &hunk.ops {
        match op {
            Op::Equal(i, _) => push(&mut out, ' ', a[*i]),
            Op::Delete(i) => push(&mut out, '-', a[*i]),
            Op::Insert(j) => push(&mut out, '+', b[*j]),
        }
    }

    out
}

/// Appends `line` marked with `mark`, noting a missing final newline
/// as `git diff` does.
fn push(out: &mut String, mark: char, line: &str) {
    out.push(mark);
    out.push_str(line);
    if !line.ends_with('\n') {
        out.push_str("\n\\ No newline at end of file\n");
    }
}

#[cfg(test)]
mod tests;
