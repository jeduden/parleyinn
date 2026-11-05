//! The line over the line ceiling a window ends before: once every line
//! before it is planned, its place is recorded, an unparsed line event
//! stands for it and the ingest position moves past it, so every later
//! line of the source is recorded. Its content is never read, held or
//! scanned, so the event holds a whole-line redaction marker, the
//! recorded redaction being I1's exception: its bytes are zeros in the
//! consumed-prefix hash, as its recorded place says (SEC-05, SEC-31,
//! I1).

use key::random::Entropy;
use record::id::WriterId;
use store::Position;
use store::source::Zeroed;

use super::Resume;
use super::View;
use super::as_index;
use super::plan::Plan;

/// Records `plan`'s place and event, in `writer`'s run, for the line
/// over the line ceiling `view` ends before, when its line break is
/// written and every line before it is planned: not held, and not left
/// for the next read by the planning budget. Its line number follows the
/// `lines` lines of the window `resume` starts at.
///
/// # Errors
///
/// Fails when the event's commitment key cannot be drawn.
pub(super) fn pass_over(
    plan: &mut Plan,
    view: &View<'_>,
    (resume, lines): (&Resume, usize),
    writer: WriterId,
    entropy: &mut dyn Entropy,
) -> Result<(), crate::IngestError> {
    let line = resume.first_line + lines as u64 + 1;
    if let Some(over) = view
        .refused
        .filter(|_| view.refused_ended && !plan.hold && !plan.resumable)
    {
        plan.pass_over(over, line, writer, entropy)?;
    }

    Ok(())
}

/// Hashes into `running` what a transaction ending at `end` consumes
/// after `position`: the window's `bytes` through `end`, and, when the
/// transaction ends past them with the line over the line ceiling, that
/// line's bytes as the zeros its span zeroes.
pub(super) fn hash_through(
    running: &mut Zeroed<'_>,
    bytes: &[u8],
    base: u64,
    position: &Position,
    end: u64,
    plan: &Plan,
) {
    let held = base + bytes.len() as u64;
    running.update(
        bytes
            .get(as_index(position.offset - base)..as_index(end.min(held) - base))
            .unwrap_or_default(),
    );
    if let Some(over) = plan
        .over
        .as_ref()
        .filter(|over| end == over.offset + over.length)
    {
        feed_zeroes(running, over.length);
    }
}

/// `position` moved past what `plan` refused after its last recorded
/// line, `running` hashing the bytes it passes: a refusal no recorded
/// line follows still moves the position, or the next read would refuse
/// it again.
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
