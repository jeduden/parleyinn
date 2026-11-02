//! What `parinn ingest` prints of the transcripts it recorded, and how it
//! reads a source of more than one window (SEC-05, REC-13).

use serde_json::{Value, json};

/// What `parinn ingest` prints of the transcripts it recorded: for
/// `--path`, the one; for `--all`, each, or that there was none.
pub(crate) fn ingest_report(
    target: &crate::IngestTarget,
    reports: &[ingest::Ingested],
    failures: usize,
    as_json: bool,
) -> String {
    let one = |i: &ingest::Ingested| {
        json!({
            "writer": i.writer.to_string(),
            "seat": i.seat.to_string(),
            "room": i.room.to_string(),
            "events": i.events,
            "read_window_high_water": i.read_window_high_water,
            "generation": i.generation,
            "held": i.held,
            "runs": runs_json(&i.runs),
        })
    };
    if as_json && *target == crate::IngestTarget::All {
        let all: Vec<Value> = reports.iter().map(one).collect();
        return format!("{}\n", json!({"transcripts": all}));
    }
    if reports.is_empty() && failures == 0 && *target == crate::IngestTarget::All {
        return "ingested no transcript\n".to_owned();
    }
    reports
        .iter()
        .map(|i| {
            if as_json {
                format!("{}\n", one(i))
            } else {
                format!("ingested {} lines into writer {}\n", i.events, i.writer)
            }
        })
        .collect()
}

/// Each run an ingest appended to, as `parinn ingest --json` shows it:
/// its writer and seat, the harness's agent id, the run its parent link
/// ties it to, and how many events it gained (REC-02).
fn runs_json(runs: &[ingest::RunReport]) -> Value {
    runs.iter()
        .map(|r| {
            json!({
                "writer": r.writer.to_string(),
                "seat": r.seat.to_string(),
                "agent_id": r.agent_id,
                "parent": r.parent.map(|p| p.to_string()),
                "events": r.events,
            })
        })
        .collect()
}
