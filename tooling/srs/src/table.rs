//! Pipe tables read from a Markdown document, and the code fences
//! that hide a table quoted inside them.

/// One pipe table: its header cells and its body rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Row>,
}

/// One body row of a [`Table`], with the line it sits on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub cells: Vec<String>,
    pub line: usize,
}

/// Reads every pipe table in `body`. A table is a header row, a
/// delimiter row of dashes, and the pipe rows that follow; lines inside
/// a fenced code block never count, so a table quoted in an example is
/// not a table.
#[must_use]
pub fn tables(body: &str) -> Vec<Table> {
    let lines: Vec<&str> = body.lines().collect();
    let mut out = Vec::new();
    let mut fence = None;
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if fence_marker(trimmed).is_some() {
            fence = fence_after(fence, trimmed);
            i += 1;
            continue;
        }
        if fence.is_some() || !opens_table(trimmed, lines.get(i + 1).copied()) {
            i += 1;
            continue;
        }

        let mut table = Table {
            header: split_row(trimmed),
            rows: Vec::new(),
        };
        i += 2;
        while let Some(line) = lines.get(i).map(|l| l.trim()).filter(|l| is_pipe_row(l)) {
            table.rows.push(Row {
                cells: split_row(line),
                line: i + 1,
            });
            i += 1;
        }
        out.push(table);
    }

    out
}

/// The run of three or more backticks or tildes `line` opens with, or
/// `None` when it is no fence.
#[must_use]
pub fn fence_marker(line: &str) -> Option<&str> {
    let first = line.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let n = line.len() - line.trim_start_matches(first).len();

    (n >= 3).then(|| &line[..n])
}

/// The fence open after `line`, given the fence `open` before it. A
/// fence closes only on a run of its own character at least as long as
/// the one that opened it, so a four-backtick fence can quote a
/// three-backtick one.
#[must_use]
pub fn fence_after<'a>(open: Option<&'a str>, line: &'a str) -> Option<&'a str> {
    match (open, fence_marker(line.trim())) {
        (open, None) => open,
        (None, Some(f)) => Some(f),
        (Some(open), Some(f)) if f.as_bytes()[0] == open.as_bytes()[0] && f.len() >= open.len() => {
            None
        }
        (open, Some(_)) => open,
    }
}

/// Whether `line`, with `next` under it, opens a table: a pipe row with
/// a delimiter row under it.
pub(crate) fn opens_table(line: &str, next: Option<&str>) -> bool {
    is_pipe_row(line.trim()) && next.is_some_and(is_delimiter_row)
}

fn is_pipe_row(line: &str) -> bool {
    line.starts_with('|')
}

/// Whether `line` is a table's `|---|:--:|` row.
fn is_delimiter_row(line: &str) -> bool {
    let line = line.trim();
    is_pipe_row(line)
        && split_row(line).iter().all(|c| {
            let c = c.trim_matches(':');
            !c.is_empty() && c.chars().all(|ch| ch == '-')
        })
}

/// Splits a pipe row into trimmed cells. An escaped pipe (`\|`) stays
/// inside its cell as a plain `|`.
fn split_row(line: &str) -> Vec<String> {
    let mut line = line.strip_prefix('|').unwrap_or(line);
    if line.ends_with('|') && !line.ends_with("\\|") {
        line = &line[..line.len() - 1];
    }

    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                cur.push('|');
                chars.next();
            }
            '|' => cells.push(std::mem::take(&mut cur).trim().to_owned()),
            _ => cur.push(c),
        }
    }
    cells.push(cur.trim().to_owned());

    cells
}

#[cfg(test)]
mod tests;
