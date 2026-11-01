//! ENG-33: every workflow and composite action pins each action and
//! reusable workflow it uses from outside the repository to a full
//! commit sha, with its version in a comment, and no `run:` script
//! expands a value a pull-request author can set; CI runs zizmor over
//! them as well.

use std::fs;

use crate::Checkout;
use crate::review_workflow::uncomment;

/// Where the workflows live, relative to the checkout.
pub const WORKFLOWS: &str = ".github/workflows";

/// Where the composite actions live, one directory each.
pub const ACTIONS: &str = ".github/actions";

/// The CI workflow that runs zizmor.
pub const CI: &str = ".github/workflows/ci.yml";

/// The command CI runs zizmor over the workflows and actions with.
pub const ZIZMOR: &str = "zizmor .github";

/// The contexts a `run:` script may not expand: a pull-request author
/// sets every field of the event (a pull request's title, a branch
/// name, a comment's body), the head branch, and what a caller passes
/// as an input.
pub const UNTRUSTED: [&str; 3] = ["github.event", "github.head_ref", "inputs"];

/// The length of a full commit sha in hex.
const SHA_LEN: usize = 40;

/// The length of a sha256 digest in hex.
const DIGEST_LEN: usize = 64;

/// Every workflow and composite action in the checkout, relative to
/// it, sorted.
///
/// # Errors
///
/// Fails when the workflow directory, or an action directory present,
/// cannot be read.
pub fn files(c: &Checkout) -> Result<Vec<String>, String> {
    // One read of the directory and its entries, so an error in either
    // is the same named problem.
    let read = |dir: &str| {
        fs::read_dir(c.root.join(dir))
            .and_then(|entries| entries.collect::<Result<Vec<_>, _>>())
            .map(|entries| {
                entries
                    .iter()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
            })
            .map_err(|e| format!("read {dir}: {e}"))
    };
    let mut found: Vec<String> = read(WORKFLOWS)?
        .into_iter()
        .filter(|n| is_yaml(n))
        .map(|n| format!("{WORKFLOWS}/{n}"))
        .collect();
    if c.root.join(ACTIONS).is_dir() {
        for name in read(ACTIONS)? {
            found.extend(
                ["action.yml", "action.yaml"]
                    .iter()
                    .map(|f| format!("{ACTIONS}/{name}/{f}"))
                    .filter(|f| c.root.join(f).is_file()),
            );
        }
    }
    found.sort();

    Ok(found)
}

fn is_yaml(name: &str) -> bool {
    name.ends_with(".yml") || name.ends_with(".yaml")
}

/// What `line` names after `uses:`, unquoted; none for any other line.
fn uses_target(line: &str) -> Option<String> {
    let code = uncomment(line);
    let key = code.trim();
    let key = key.strip_prefix("- ").map_or(key, str::trim_start);
    let target = key
        .strip_prefix("uses:")?
        .trim()
        .trim_matches(|c| c == '"' || c == '\'');

    (!target.is_empty()).then(|| target.to_owned())
}

fn is_hex(s: &str, len: usize) -> bool {
    s.len() == len && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Why `target`, which `line` uses, is not pinned; none when it is: a
/// local action or workflow, a container image by its sha256 digest,
/// or anything else by a full commit sha with its version in a comment
/// after it.
fn pin_problem(target: &str, line: &str) -> Option<&'static str> {
    if target.starts_with("./") {
        return None;
    }
    if let Some(image) = target.strip_prefix("docker://") {
        let digest = image.split_once("@sha256:").map(|(_, d)| d);
        return match digest {
            Some(d) if is_hex(d, DIGEST_LEN) => None,
            _ => Some("not pinned to a sha256 digest"),
        };
    }
    let reference = target.rsplit_once('@').map(|(_, r)| r);
    if !reference.is_some_and(|r| is_hex(r, SHA_LEN)) {
        return Some("not pinned to a full commit sha");
    }
    let comment = line.split_once(" #").map(|(_, c)| c.trim());
    let first = comment.and_then(|c| c.split_whitespace().next());
    if first.is_some_and(is_version) {
        None
    } else {
        Some("with no version in a comment")
    }
}

/// Whether `word` reads as a version: dot-separated numbers, `v` before
/// them or not, as `v7.0.2` or `4`.
fn is_version(word: &str) -> bool {
    let numbers = word.strip_prefix('v').unwrap_or(word);
    numbers
        .split('.')
        .all(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// Every action or reusable workflow `body`, the text of `file`, uses
/// from outside the repository without a full commit sha and a
/// version comment, one problem each.
#[must_use]
pub fn unpinned(file: &str, body: &str) -> Vec<String> {
    body.lines()
        .enumerate()
        .filter_map(|(n, line)| {
            let target = uses_target(line)?;
            let problem = pin_problem(&target, line)?;
            Some(format!("{file}:{} uses {target}, {problem}", n + 1))
        })
        .collect()
}

/// The column and value of a `run:` key on `line`, a list item's
/// included; none for any other line.
fn run_key(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let mut column = line.len() - trimmed.len();
    let key = match trimmed.strip_prefix("- ") {
        Some(rest) => {
            let item = rest.trim_start();
            column += 2 + rest.len() - item.len();
            item
        }
        None => trimmed,
    };

    key.strip_prefix("run:").map(|value| (column, value))
}

/// Every `run:` script of `body`, each with the line it starts on,
/// counted from one: a block scalar's lines, or an inline value
/// without its comment.
#[must_use]
pub fn run_scripts(body: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = body.lines().collect();
    let mut scripts = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let Some((column, value)) = run_key(lines[i]) else {
            i += 1;
            continue;
        };
        let value = uncomment(value);
        let value = value.trim();
        if !(value.starts_with('|') || value.starts_with('>')) {
            scripts.push((i + 1, value.to_owned()));
            i += 1;
            continue;
        }
        let start = i + 2;
        let mut script = Vec::new();
        i += 1;
        while i < lines.len() {
            let line = lines[i];
            let indent = line.len() - line.trim_start().len();
            if !line.trim().is_empty() && indent <= column {
                break;
            }
            script.push(line);
            i += 1;
        }
        scripts.push((start, script.join("\n")));
    }

    scripts
}

/// Whether `expression`, the inside of a `${{ }}`, reads a context of
/// [`UNTRUSTED`], in any case, as GitHub reads contexts, and through a
/// quoted index as through a dot: `github['head_ref']` reads
/// `github.head_ref`.
fn reads_untrusted(expression: &str) -> bool {
    dotted(&expression.to_ascii_lowercase())
        .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')))
        .any(|word| {
            UNTRUSTED.iter().any(|context| {
                word == *context
                    || word
                        .strip_prefix(context)
                        .is_some_and(|rest| rest.starts_with('.'))
            })
        })
}

/// `expression` with each quoted index, `['x']` or `["x"]`, spaces
/// around it aside, written `.x`; any other bracket stays.
fn dotted(expression: &str) -> String {
    let mut out = String::new();
    let mut rest = expression;
    while let Some(open) = rest.find('[') {
        out.push_str(&rest[..open]);
        let inside = rest[open + 1..].trim_start();
        let quoted = inside.chars().next().filter(|q| matches!(q, '\'' | '"'));
        let index = quoted.and_then(|q| {
            let (name, after) = inside[1..].split_once(q)?;
            Some((name, after.trim_start().strip_prefix(']')?))
        });
        match index {
            Some((name, after)) => {
                out.truncate(out.trim_end().len());
                out.push('.');
                out.push_str(name);
                rest = after;
            }
            None => {
                out.push('[');
                rest = &rest[open + 1..];
            }
        }
    }
    out.push_str(rest);

    out
}

/// Every `${{ }}` expression a `run:` script of `body`, the text of
/// `file`, expands that reads a context of [`UNTRUSTED`], one problem
/// each.
#[must_use]
pub fn untrusted_expansions(file: &str, body: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (start, script) in run_scripts(body) {
        let mut rest = script.as_str();
        let mut offset = 0;
        while let Some(open) = rest.find("${{") {
            let after = &rest[open + 3..];
            let Some(close) = after.find("}}") else {
                break;
            };
            let expression = after[..close].trim();
            if reads_untrusted(expression) {
                let line = start + script[..offset + open].matches('\n').count();
                problems.push(format!(
                    "{file}:{line} expands {expression} in a run script, not through env"
                ));
            }
            let used = open + 3 + close + 2;
            offset += used;
            rest = &rest[used..];
        }
    }

    problems
}

/// Requires `ci`, the CI workflow's text, to run [`ZIZMOR`] in a step
/// that a comment does not merely name.
#[must_use]
pub fn runs_zizmor(ci: &str) -> Vec<String> {
    let runs = run_scripts(ci)
        .iter()
        .any(|(_, script)| script.lines().any(|l| l.trim() == ZIZMOR));
    if runs {
        Vec::new()
    } else {
        vec![format!("the CI workflow does not run {ZIZMOR:?}")]
    }
}

/// Every problem of the workflows and composite actions in the
/// checkout: an unpinned use, or an untrusted expansion in a script.
#[must_use]
pub fn check(c: &Checkout) -> Vec<String> {
    let files = match files(c) {
        Ok(f) if f.is_empty() => return vec![format!("{WORKFLOWS} holds no workflow")],
        Ok(f) => f,
        Err(e) => return vec![e],
    };
    let mut problems = Vec::new();
    for file in files {
        match c.read(&file) {
            Ok(body) => {
                problems.extend(unpinned(&file, &body));
                problems.extend(untrusted_expansions(&file, &body));
            }
            Err(e) => problems.push(e),
        }
    }

    problems
}

#[cfg(test)]
mod tests;
