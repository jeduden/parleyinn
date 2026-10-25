//! ENG-14: no agent instruction file directs a destructive command at
//! non-isolated state. The files are the agent instructions the
//! repository carries: every `CLAUDE.md` and `AGENTS.md`, and
//! everything under a `.claude/` directory (agents, skills, settings).
//! A statement in them directs a destructive command when it names a
//! way to delete or overwrite, names real state (`~`, `HOME`,
//! `PARINN_HOME`, Claude Code's configuration directory), and neither
//! forbids it nor first points `HOME` at a temporary directory.
//!
//! A statement is a line of a code block, a line of a file that is not
//! Markdown, or a sentence of prose. A paragraph, a list item, a table
//! row and a heading each stand alone, so one item's "never" does not
//! excuse the next item's command.

use std::fs;
use std::io;
use std::path::Path;

use crate::Checkout;

/// The instruction files that sit anywhere in the repository.
const INSTRUCTION_NAMES: [&str; 3] = ["CLAUDE.md", "AGENTS.md", "CLAUDE.local.md"];

/// Directories that hold no instruction of this repository.
const SKIPPED_DIRS: [&str; 3] = [".git", "target", "node_modules"];

/// The directory that holds agents, skills and settings.
const CLAUDE_DIR: &str = ".claude";

/// Under [`CLAUDE_DIR`], other checkouts of the repository.
const OTHER_CHECKOUTS: &str = "worktrees";

/// A developer's own settings, kept out of the repository.
const PRIVATE: &str = "settings.local.json";

/// What names real state in a statement, as a word.
const HOME_NAMES: [&str; 4] = ["HOME", "PARINN_HOME", "CLAUDE_CONFIG_DIR", "USERPROFILE"];

/// Words that delete or overwrite, whole.
const DESTRUCTIVE: [&str; 7] = ["rm", "rmdir", "unlink", "shred", "tee", "trash", "-delete"];

/// Words that delete or overwrite, by their stem.
const DESTRUCTIVE_STEMS: [&str; 11] = [
    "delet",
    "remov",
    "eras",
    "wipe",
    "wiping",
    "purg",
    "destroy",
    "overwrit",
    "clobber",
    "truncat",
    "uninstall",
];

/// Commands that delete or overwrite in more than one word.
const DESTRUCTIVE_PHRASES: [&str; 2] = ["git clean", "sed -i"];

/// Words that forbid an act.
const NEGATIONS: [&str; 13] = [
    "never",
    "not",
    "no",
    "avoid",
    "without",
    "refuse",
    "refuses",
    "cannot",
    "forbid",
    "forbids",
    "forbidden",
    "prohibit",
    "prohibited",
];

/// What prose says to put the act in a temporary directory.
const ISOLATION_WORDS: [&str; 5] = ["temporary", "tempdir", "temp dir", "mktemp", "tmpdir"];

/// One statement of an instruction file.
struct Statement {
    /// The line it starts on.
    line: usize,
    text: String,
    /// Whether `HOME` points at a temporary directory by then.
    isolated: bool,
    /// Whether it is prose, where a "never" forbids the act. A line of
    /// code is not excused by a word in it.
    prose: bool,
}

/// Lists the instruction files under `root`, as paths relative to it,
/// sorted. Another checkout under `.claude/worktrees` is no part of
/// this one, nor is a file a developer keeps for themselves
/// (`settings.local.json`).
///
/// # Errors
///
/// Fails when a directory cannot be listed.
pub fn instruction_files(root: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    walk(root, "", false, &mut out)?;
    out.sort();

    Ok(out)
}

/// Adds the instruction files under `dir`, which sits at `rel`.
fn walk(dir: &Path, rel: &str, in_claude: bool, out: &mut Vec<String>) -> Result<(), String> {
    let list = |e: io::Error| format!("list {}: {e}", dir.display());
    for entry in fs::read_dir(dir).map_err(list)? {
        let entry = entry.map_err(list)?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let here = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        if entry.file_type().map_err(list)?.is_dir() {
            if !skipped(&name, rel) {
                walk(&entry.path(), &here, in_claude || name == CLAUDE_DIR, out)?;
            }
        } else if is_instruction(&name, in_claude) {
            out.push(here);
        }
    }

    Ok(())
}

/// Whether the directory `name` under `parent` holds no instruction.
fn skipped(name: &str, parent: &str) -> bool {
    let under_claude = parent.rsplit('/').next() == Some(CLAUDE_DIR);

    SKIPPED_DIRS.contains(&name) || (under_claude && name == OTHER_CHECKOUTS)
}

/// Whether the file `name` is an instruction file, inside a `.claude`
/// directory or not.
fn is_instruction(name: &str, in_claude: bool) -> bool {
    if in_claude {
        name != PRIVATE
    } else {
        INSTRUCTION_NAMES.contains(&name)
    }
}

/// The statements of `body`, the text of the file `file`, that direct
/// a destructive command at non-isolated state, one line each:
/// `file:line: ...`.
#[must_use]
pub fn directives(file: &str, body: &str) -> Vec<String> {
    let statements = if file.ends_with(".md") {
        markdown(body)
    } else {
        plain(body)
    };

    statements
        .iter()
        .filter(|s| directs(s))
        .map(|s| {
            format!(
                "{file}:{}: directs a destructive command at non-isolated state: {}",
                s.line,
                s.text.trim()
            )
        })
        .collect()
}

/// Requires every instruction file of the checkout to be free of a
/// destructive command aimed at non-isolated state.
#[must_use]
pub fn no_destructive_instructions(c: &Checkout) -> Vec<String> {
    let files = match instruction_files(&c.root) {
        Ok(files) => files,
        Err(e) => return vec![e],
    };

    files
        .iter()
        .flat_map(|file| match fs::read(c.root.join(file)) {
            Ok(bytes) => directives(file, &String::from_utf8_lossy(&bytes)),
            Err(e) => vec![format!("read {file}: {e}")],
        })
        .collect()
}

/// The lines of a file that is not Markdown, one statement each.
fn plain(body: &str) -> Vec<Statement> {
    body.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(i, line)| Statement {
            line: i + 1,
            text: line.to_owned(),
            isolated: assigns_temp_home(line),
            prose: false,
        })
        .collect()
}

/// The statements of a Markdown file: the lines of its code blocks and
/// the sentences of its prose.
fn markdown(body: &str) -> Vec<Statement> {
    let mut out = Vec::new();
    let mut para = Paragraph::default();
    // Whether a code block is open, and `HOME` isolated in it so far.
    let mut block: Option<bool> = None;
    for (i, raw) in body.lines().enumerate() {
        let (line, text) = (i + 1, raw.trim_start());
        if text.starts_with("```") || text.starts_with("~~~") {
            para.end(&mut out);
            block = block.map_or(Some(false), |_| None);
        } else if let Some(isolated) = block.as_mut() {
            *isolated |= assigns_temp_home(raw);
            out.push(Statement {
                line,
                text: raw.to_owned(),
                isolated: *isolated,
                prose: false,
            });
        } else if text.is_empty() {
            para.end(&mut out);
        } else {
            if starts_item(text) {
                para.end(&mut out);
            }
            para.add(line, text);
            if is_single_line(text) {
                para.end(&mut out);
            }
        }
    }
    para.end(&mut out);

    out
}

/// Prose being gathered: the lines of one paragraph or list item.
#[derive(Default)]
struct Paragraph {
    line: usize,
    text: String,
}

impl Paragraph {
    fn add(&mut self, line: usize, text: &str) {
        if self.text.is_empty() {
            self.line = line;
        } else {
            self.text.push(' ');
        }
        self.text.push_str(text);
    }

    /// Ends the paragraph: its sentences join `out`.
    fn end(&mut self, out: &mut Vec<Statement>) {
        let text = std::mem::take(&mut self.text);
        out.extend(sentences(&text).into_iter().map(|s| Statement {
            line: self.line,
            isolated: assigns_temp_home(&s) || says_isolated(&s),
            prose: true,
            text: s,
        }));
    }
}

/// Whether `text` opens a list item, a heading, a table row or front
/// matter: a line that does not continue the paragraph before it.
fn starts_item(text: &str) -> bool {
    let digits = text.trim_start_matches(|c: char| c.is_ascii_digit());
    let numbered =
        digits.len() < text.len() && (digits.starts_with(". ") || digits.starts_with(") "));

    ["- ", "* ", "+ "].iter().any(|m| text.starts_with(m)) || numbered || is_single_line(text)
}

/// Whether `text` is a heading, a table row or a front-matter fence: a
/// line that is a paragraph of its own.
fn is_single_line(text: &str) -> bool {
    text.starts_with('#') || text.starts_with('|') || text == "---"
}

/// The sentences of `text`: each runs to a `.`, `!` or `?` that is
/// followed by a space or the end.
fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        current.push(c);
        if matches!(c, '.' | '!' | '?') && chars.peek().is_none_or(|n| n.is_whitespace()) {
            out.push(std::mem::take(&mut current));
        }
    }
    out.push(current);
    out.retain(|s| !s.trim().is_empty());

    out
}

/// Whether `statement` directs a destructive command at real state.
fn directs(statement: &Statement) -> bool {
    let text = &statement.text;

    let forbidden = statement.prose && forbids(text);

    !statement.isolated && destroys(text) && names_real_home(text) && !forbidden
}

/// The words of `text`, lowercase: letters, digits, `_`, `-` and `'`.
fn words(text: &str) -> Vec<String> {
    text.replace('\u{2019}', "'")
        .to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '-' | '\'')))
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether `text` names a way to delete or overwrite.
fn destroys(text: &str) -> bool {
    let lower = text.to_lowercase();

    words(text).iter().any(|w| {
        DESTRUCTIVE.contains(&w.as_str()) || DESTRUCTIVE_STEMS.iter().any(|s| w.starts_with(s))
    }) || DESTRUCTIVE_PHRASES.iter().any(|p| lower.contains(p))
        || redirects_to_home(text)
}

/// Whether `text` forbids what it names.
fn forbids(text: &str) -> bool {
    words(text)
        .iter()
        .any(|w| NEGATIONS.contains(&w.as_str()) || w.ends_with("n't"))
}

/// Whether `text` says, in prose, that the act happens in a temporary
/// or isolated place.
fn says_isolated(text: &str) -> bool {
    let lower = text.to_lowercase();

    lower.contains("isolat") || ISOLATION_WORDS.iter().any(|w| lower.contains(w))
}

/// Whether `text` sets `HOME` or `PARINN_HOME` to a temporary directory.
fn assigns_temp_home(text: &str) -> bool {
    text.match_indices("HOME=").any(|(i, assign)| {
        let value = text[i + assign.len()..].split_whitespace().next();
        value.is_some_and(|v| {
            let v = v.to_lowercase();
            v.contains("tmp") || v.contains("temp")
        })
    })
}

/// Whether `text` names real state: `~`, or one of [`HOME_NAMES`].
fn names_real_home(text: &str) -> bool {
    has_tilde(text)
        || text
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .any(|w| HOME_NAMES.contains(&w))
}

/// Whether `text` holds a `~` that stands for the user's `HOME`: at the start
/// of a path, and alone or followed by a `/`.
fn has_tilde(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();

    (0..chars.len()).any(|i| {
        let before = i.checked_sub(1).map(|b| chars[b]);
        chars[i] == '~'
            && before.is_none_or(|c| c.is_whitespace() || "`\"'(=:>".contains(c))
            && path_ends_after(chars.get(i + 1).copied())
    })
}

/// Whether a `~` followed by `next` is the user's `HOME` and not part of a word.
fn path_ends_after(next: Option<char>) -> bool {
    next.is_none_or(|c| c == '/' || c.is_whitespace() || "`\"')],.;:".contains(c))
}

/// Whether `text` redirects output (`>`, `>>`, `>|`) into real
/// state.
fn redirects_to_home(text: &str) -> bool {
    text.match_indices('>').any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let rest = text[i + 1..].trim_start_matches(['>', '|', ' ', '\t', '"', '\'']);
        !matches!(before, Some('-' | '=' | '>')) && starts_at_home(rest)
    })
}

/// Whether `rest` starts with real state: `~`, or a variable of
/// [`HOME_NAMES`].
fn starts_at_home(rest: &str) -> bool {
    match rest.strip_prefix('~') {
        Some(after) => path_ends_after(after.chars().next()),
        None => rest.strip_prefix('$').is_some_and(|v| {
            let v = v.trim_start_matches('{');
            HOME_NAMES.iter().any(|n| v.starts_with(n))
        }),
    }
}

#[cfg(test)]
mod tests;
