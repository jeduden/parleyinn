//! Requirement blocks read from a Markdown document. A block is a
//! heading whose text is a requirement id, the list of fields right
//! under it, and its text, which runs to the next heading or table.

use crate::is_requirement_id;
use crate::table::{fence_after, fence_marker, opens_table};

/// One requirement written as a block.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Block {
    /// The id its heading holds.
    pub id: String,
    /// Each item of its field list as its label and its value, in
    /// order. An item with no colon is all label.
    pub fields: Vec<(String, String)>,
    /// Its lines after the field list, each trimmed, joined by single
    /// spaces.
    pub text: String,
    /// The line its heading sits on.
    pub line: usize,
}

/// Where a block's reading stands.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    /// Between the heading and the first item or text line.
    Head,
    /// In the field list.
    Fields,
    /// In the text.
    Text,
}

/// Reads every block in `body`. A line inside a fenced code block is
/// never a heading or a table, so a block quoted in an example is none.
/// A table opens on a pipe row with a delimiter row under it, as
/// [`crate::tables`] reads one, so a line of the text that starts with
/// a pipe stays text, as it does in Markdown.
#[must_use]
pub fn blocks(body: &str) -> Vec<Block> {
    let lines: Vec<&str> = body.lines().collect();
    let mut out = Vec::new();
    let mut open: Option<(Block, Part)> = None;
    let mut fence = None;
    for (i, line) in lines.iter().copied().enumerate() {
        let trimmed = line.trim();
        let fenced = fence.is_some() || fence_marker(trimmed).is_some();
        fence = fence_after(fence, trimmed);
        if let Some(text) = heading(line).filter(|_| !fenced) {
            out.extend(open.take().map(|(block, _)| block));
            open = is_requirement_id(text).then(|| {
                let block = Block {
                    id: text.to_owned(),
                    line: i + 1,
                    ..Block::default()
                };
                (block, Part::Head)
            });
            continue;
        }
        if !fenced && opens_table(trimmed, lines.get(i + 1).copied()) {
            out.extend(open.take().map(|(block, _)| block));
        }
        if let Some((block, part)) = open.as_mut() {
            *part = take(block, *part, line);
        }
    }
    out.extend(open.map(|(block, _)| block));

    out
}

/// Adds `line` to `block`, read in `part`, and returns the part the
/// next line is read in: an item under the heading opens the field
/// list, any other line of the list continues its last field, as
/// Markdown reads a list item's lines, a blank line ends the list, and
/// any other line that is not blank is text.
fn take(block: &mut Block, part: Part, line: &str) -> Part {
    let trimmed = line.trim();
    let item = line.strip_prefix("- ");
    match (part, item) {
        (Part::Head | Part::Fields, Some(item)) => {
            let (label, value) = item.split_once(':').unwrap_or((item, ""));
            block
                .fields
                .push((label.trim().to_owned(), value.trim().to_owned()));
            Part::Fields
        }
        (_, _) if trimmed.is_empty() => match part {
            Part::Head => Part::Head,
            Part::Fields | Part::Text => Part::Text,
        },
        (Part::Fields, None) => {
            if let Some((_, value)) = block.fields.last_mut() {
                value.push(' ');
                value.push_str(trimmed);
            }
            Part::Fields
        }
        _ => {
            if !block.text.is_empty() {
                block.text.push(' ');
            }
            block.text.push_str(trimmed);
            Part::Text
        }
    }
}

/// The text of `line` when it is an ATX heading: one to six `#` and a
/// space or the line's end.
fn heading(line: &str) -> Option<&str> {
    let rest = line.trim_start_matches('#');
    let level = line.len() - rest.len();
    ((1..=6).contains(&level) && (rest.is_empty() || rest.starts_with(' '))).then(|| rest.trim())
}

#[cfg(test)]
mod tests;
