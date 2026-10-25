//! The scenario the test and the measurement share: two run seats'
//! writers, A and B, each appending spike S2's corpus and recording the
//! other's chain head at its newest closed segment whenever they close
//! one, while seats A and B each link one branch to the room.

use serde_json::{Map, Value, json};

use crate::Result;
use crate::corpus::{Corpus, Event as CorpusEvent};
use crate::record::{Address, Segment, Writer};

pub const ROOM: &str = "room-1";
pub const REPOSITORY: &str = "repository-1";
/// The branch seats A and B link concurrently: neither had seen the
/// other's link act.
pub const CONCURRENT_BRANCH: &str = "feature/x";
/// The branch seat B links after it had seen A's branch link.
pub const CAUSAL_BRANCH: &str = "feature/y";

/// Both writers and the segments each closed, in the order closed.
pub struct TwoWriters {
    pub a: Writer,
    pub b: Writer,
    pub a_segments: Vec<Segment>,
    pub b_segments: Vec<Segment>,
    /// A's and B's branch link of the concurrent branch.
    pub concurrent_links: (Address, Address),
    /// A's and B's branch link of the causal branch, A's first.
    pub causal_links: (Address, Address),
}

/// An S2 corpus event as (kind, provenance, content). The corpus's own
/// trust draw is ignored: trust derives on the reading node (PRV-02).
pub fn corpus_event(e: &CorpusEvent) -> (&'static str, String, Value) {
    let (kind, provenance) = match (e.kind, e.tool) {
        ("user", _) => ("message", "user".to_string()),
        ("assistant", _) => ("message", "assistant".to_string()),
        ("tool_use", _) => ("tool_call", "tool_call".to_string()),
        ("tool_result", Some(tool)) => ("tool_result", format!("tool_result:{tool}")),
        _ => ("metadata", "harness_meta".to_string()),
    };
    let mut content = Map::new();
    content.insert("run".into(), json!(e.session));
    content.insert("line".into(), json!(e.line));
    content.insert("ts".into(), json!(e.ts));
    content.insert("text".into(), json!(e.body));
    if let Some(tool) = e.tool {
        content.insert("tool".into(), json!(tool));
    }
    (kind, provenance, Value::Object(content))
}

/// A run seat's link act adding a branch link; a run seat's acts take
/// its pin class, `assistant` (PRV-01).
pub fn link(w: &mut Writer, branch: &str) -> Result<Address> {
    let act = json!({"act": "link", "link": "branch", "room": ROOM, "repository": REPOSITORY, "branch": branch});
    w.append("act", "assistant", act)
}

/// Builds both writers: `per_writer` corpus events each, a segment
/// closed every `per_segment` of them.
pub fn two_writers(per_writer: usize, per_segment: usize) -> Result<TwoWriters> {
    let mut a = Writer::open(b"seat key of A", "interactive")?;
    let mut b = Writer::open(b"seat key of B", "interactive")?;
    let (mut ca, mut cb) = (Corpus::from_seeds(1, 1), Corpus::from_seeds(1, 2));
    let (mut a_segments, mut b_segments) = (Vec::new(), Vec::new());
    let mut concurrent = None;
    let close = |w: &mut Writer, out: &mut Vec<Segment>| out.extend(w.close_segment());
    for i in 0..per_writer {
        let (k, p, c) = corpus_event(&ca.next());
        a.append(k, &p, c)?;
        let (k, p, c) = corpus_event(&cb.next());
        b.append(k, &p, c)?;
        if i == per_writer / 2 {
            concurrent = Some((
                link(&mut a, CONCURRENT_BRANCH)?,
                link(&mut b, CONCURRENT_BRANCH)?,
            ));
        }
        if (i + 1) % per_segment == 0 {
            close(&mut a, &mut a_segments);
            close(&mut b, &mut b_segments);
            a.see(&b);
            b.see(&a);
        }
    }
    let a_causal = link(&mut a, CAUSAL_BRANCH)?;
    close(&mut a, &mut a_segments);
    b.see(&a);
    let b_causal = link(&mut b, CAUSAL_BRANCH)?;
    close(&mut b, &mut b_segments);
    let concurrent_links = concurrent.ok_or_else(|| crate::Error::new("too few events"))?;
    Ok(TwoWriters {
        a,
        b,
        a_segments,
        b_segments,
        concurrent_links,
        causal_links: (a_causal, b_causal),
    })
}
