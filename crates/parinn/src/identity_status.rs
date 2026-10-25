//! What `parinn status` says of the node identity and of fork detection
//! (REC-24, PEER-10): whether this OS has a source the node identity is
//! checked against, and that finding a fork across nodes arrives with
//! the peer network, while `parinn verify` scans the writer logs held
//! here.

use administration::node_identity::IdentitySource;
use serde_json::{Value, json};

/// The line that says where forks are found.
pub(crate) const FORK_DETECTION_LINE: &str = "fork detection: a second event at one (writer, seq) is refused for the segments the node \
     holds, found by a local scan with parinn verify; the equivocated mark waits for seals; the \
     check between peers arrives with the peer network";

/// The line that says whether, and against what, the node identity is
/// checked on this OS.
pub(crate) fn node_identity_line(source: &IdentitySource) -> String {
    match source.describe() {
        Some(against) => format!("node identity: checked ({against})"),
        None => "node identity: not checked on this OS".to_owned(),
    }
}

/// The node identity's facts for `parinn status --json`.
pub(crate) fn json(source: &IdentitySource) -> Value {
    json!({"checked": source.describe().is_some(), "against": source.describe()})
}

/// Fork detection's facts for `parinn status --json`.
pub(crate) fn fork_detection_json() -> Value {
    json!({
        "local": "parinn verify",
        "refused": "a second event at one (writer, seq), for the segments the node holds",
        "equivocated": "the mark waits for seals",
        "peer": "the check between peers arrives with the peer network",
    })
}

#[cfg(test)]
mod tests;
