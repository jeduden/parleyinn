//! The line over the line ceiling a window ends before: once every line
//! before it is planned, its place is recorded and the ingest position
//! moves past it, so every later line of the source is recorded. Its
//! content is never read: its bytes are zeros in the consumed-prefix
//! hash, as its recorded place says (SEC-05, SEC-31, I1).

use store::Position;
use store::source::Zeroed;

use super::View;
use super::as_index;
use super::plan::Plan;

/// Records `plan`'s place for the line over the line ceiling `view` ends
/// before, `line` being its line number, when its line break is written
/// and every line before it is planned: not held, and not left for the
/// next read by the planning budget.
pub(super) fn pass_over(plan: &mut Plan, view: &View<'_>, line: u64) {
    if let Some(over) = view
        .refused
        .filter(|_| view.refused_ended && !plan.hold && !plan.resumable)
    {
        plan.pass_over(over, line);
    }
}

/// `position` moved past what `plan` refused after its last recorded
/// line, `running` hashing the bytes it passes: a refusal no recorded
/// line follows still moves the position, or the next read would refuse
/// it again; the line over the line ceiling is passed as zeros.
pub(super) fn move_past(
    running: &mut Zeroed<'_>,
    bytes: &[u8],
    base: u64,
    mut position: Position,
    plan: &Plan,
    seen: u64,
) -> Position {
    if let Some((line, end)) = plan.refused_to.filter(|(_, end)| *end > position.offset) {
        running.update(
            bytes
                .get(as_index(position.offset - base)..as_index(end - base))
                .unwrap_or_default(),
        );
        position = Position {
            offset: end,
            prefix: running.digest(),
            lines: line,
            seen,
        };
    }
    if let Some(over) = &plan.over {
        feed_zeroes(running, over.length);
        position = Position {
            offset: over.offset + over.length,
            prefix: running.digest(),
            lines: over.line,
            seen,
        };
    }

    position
}

/// Feeds `count` zero bytes to `running`: the bytes of a line over the
/// line ceiling, which its recorded place zeroes in the hash.
fn feed_zeroes(running: &mut Zeroed<'_>, mut count: u64) {
    const ZEROES: [u8; 4096] = [0; 4096];
    while count > 0 {
        let take = usize::try_from(count).map_or(ZEROES.len(), |c| c.min(ZEROES.len()));
        running.update(&ZEROES[..take]);
        count -= take as u64;
    }
}

#[cfg(test)]
mod tests;
