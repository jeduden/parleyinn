//! The record: writers, events, addresses and segments.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::commit::{self, Commitment, Key};
use crate::{Error, Result, canon, hex, unhex32};

/// The segment format version every segment carries (REC-21).
pub const FORMAT: u64 = 1;

/// An event's address: (writer, seq) (REC-06).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address {
    pub writer: String,
    pub seq: u64,
}

/// Another writer's chain head at a seq, as this writer had seen it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainHead {
    pub seq: u64,
    pub chain_head: [u8; 32],
}

/// The chained header: only the address, kind, provenance, `prev_hash`,
/// the other writers' chain heads seen and the commitment (REC-10,
/// REC-17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub writer: String,
    pub seq: u64,
    pub kind: String,
    pub provenance: String,
    pub prev_hash: Option<[u8; 32]>,
    pub chain_heads_seen: BTreeMap<String, ChainHead>,
    pub commitment: Commitment,
}

impl Header {
    pub fn to_json(&self) -> Value {
        let heads: Map<String, Value> = self
            .chain_heads_seen
            .iter()
            .map(|(w, h)| {
                let head = json!({"seq": h.seq, "chain_head": hex(&h.chain_head)});
                (w.clone(), head)
            })
            .collect();
        json!({
            "writer": self.writer,
            "seq": self.seq,
            "kind": self.kind,
            "provenance": self.provenance,
            "prev_hash": self.prev_hash.map(|h| hex(&h)).unwrap_or_default(),
            "chain_heads_seen": heads,
            "commitment": hex(&self.commitment),
        })
    }

    pub fn from_json(v: &Value) -> Result<Header> {
        let mut heads = BTreeMap::new();
        for (w, h) in field(v, "chain_heads_seen")?
            .as_object()
            .ok_or_else(|| bad("chain_heads_seen"))?
        {
            let head = ChainHead {
                seq: uint(h, "seq")?,
                chain_head: unhex32(text(h, "chain_head")?)?,
            };
            heads.insert(w.clone(), head);
        }
        let prev = text(v, "prev_hash")?;
        Ok(Header {
            writer: text(v, "writer")?.to_string(),
            seq: uint(v, "seq")?,
            kind: text(v, "kind")?.to_string(),
            provenance: text(v, "provenance")?.to_string(),
            prev_hash: if prev.is_empty() {
                None
            } else {
                Some(unhex32(prev)?)
            },
            chain_heads_seen: heads,
            commitment: unhex32(text(v, "commitment")?)?,
        })
    }

    /// The event's hash: SHA-256 over the header's canonical encoding,
    /// which holds `prev_hash` (REC-10, §8.3).
    pub fn hash(&self) -> [u8; 32] {
        Sha256::digest(canon::encode(&self.to_json())).into()
    }
}

/// Content and its commitment key, kept beside the header and erased
/// together by a purge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    pub key: Key,
    pub content: Value,
}

/// One event: its header, the hash over it, and its body unless purged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub header: Header,
    pub hash: [u8; 32],
    pub body: Option<Body>,
}

impl Event {
    pub fn address(&self) -> Address {
        Address {
            writer: self.header.writer.clone(),
            seq: self.header.seq,
        }
    }

    /// Checks the hash over the header and, where the body is held, the
    /// commitment to it.
    pub fn verify(&self) -> Result<()> {
        if self.header.hash() != self.hash {
            return Err(Error::new(format!(
                "{:?}: hash does not match its header",
                self.address()
            )));
        }
        if let Some(b) = &self.body
            && commit::commit(&b.key, &canon::encode(&b.content)) != self.header.commitment
        {
            return Err(Error::new(format!(
                "{:?}: content does not open its commitment",
                self.address()
            )));
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        let body = match &self.body {
            Some(b) => json!({"key": hex(&b.key), "content": b.content}),
            None => Value::Null,
        };
        json!({"header": self.header.to_json(), "hash": hex(&self.hash), "body": body})
    }

    fn from_json(v: &Value) -> Result<Event> {
        let body = match field(v, "body")? {
            Value::Null => None,
            b => Some(Body {
                key: unhex32(text(b, "key")?)?,
                content: field(b, "content")?.clone(),
            }),
        };
        Ok(Event {
            header: Header::from_json(field(v, "header")?)?,
            hash: unhex32(text(v, "hash")?)?,
            body,
        })
    }
}

/// A closed range of one writer, the unit peers exchange.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub writer: String,
    pub events: Vec<Event>,
}

impl Segment {
    pub fn first(&self) -> u64 {
        self.events.first().map_or(0, |e| e.header.seq)
    }

    pub fn last(&self) -> u64 {
        self.events.last().map_or(0, |e| e.header.seq)
    }

    /// The segment's bytes: canonical JSON lines, a format line first.
    pub fn encode(&self) -> Vec<u8> {
        let head = json!({"format": FORMAT, "writer": self.writer, "first": self.first(), "last": self.last()});
        let mut out = canon::encode(&head);
        for e in &self.events {
            out.push(b'\n');
            out.extend(canon::encode(&e.to_json()));
        }
        out.push(b'\n');
        out
    }

    /// Reads a segment, refusing an unknown format version (REC-21), a
    /// range that does not match its events, and a broken chain inside.
    pub fn decode(bytes: &[u8]) -> Result<Segment> {
        let mut lines = bytes.split(|&b| b == b'\n').filter(|l| !l.is_empty());
        let head: Value = serde_json::from_slice(lines.next().ok_or_else(|| bad("format line"))?)?;
        let format = uint(&head, "format")?;
        if format != FORMAT {
            return Err(Error::new(format!(
                "segment format {format} is not supported"
            )));
        }
        let events = lines
            .map(|l| Event::from_json(&serde_json::from_slice(l)?))
            .collect::<Result<Vec<_>>>()?;
        let s = Segment {
            writer: text(&head, "writer")?.to_string(),
            events,
        };
        if (s.first(), s.last()) != (uint(&head, "first")?, uint(&head, "last")?) {
            return Err(Error::new("segment range does not match its events"));
        }
        s.check()?;
        Ok(s)
    }

    /// Every event verifies, belongs to the segment's writer, and chains
    /// to the one before it.
    pub fn check(&self) -> Result<()> {
        if self.events.is_empty() {
            return Err(Error::new("empty segment"));
        }
        for (i, e) in self.events.iter().enumerate() {
            e.verify()?;
            if e.header.writer != self.writer {
                return Err(Error::new("event of another writer in segment"));
            }
            if i > 0 {
                let p = &self.events[i - 1];
                if e.header.seq != p.header.seq + 1 || e.header.prev_hash != Some(p.hash) {
                    return Err(Error::new(format!(
                        "{:?}: chain broken inside segment",
                        e.address()
                    )));
                }
            } else if (e.header.seq == 1) != e.header.prev_hash.is_none() {
                return Err(Error::new("only seq 1 has no prev_hash"));
            }
        }
        Ok(())
    }
}

/// One seat's append-only log.
pub struct Writer {
    id: String,
    /// `log[i]` is the event at seq `i + 1`.
    log: Vec<Event>,
    chain_heads_seen: BTreeMap<String, ChainHead>,
    open_from: usize,
    closed: Vec<(usize, usize)>,
}

impl Writer {
    /// Opens a writer whose writer id derives from its seat's first key,
    /// recording the node's deployment mode as its first, structural
    /// event.
    pub fn open(seat_key: &[u8], deployment_mode: &str) -> Result<Writer> {
        let mut w = Writer {
            id: hex(&Sha256::digest(seat_key)),
            log: Vec::new(),
            chain_heads_seen: BTreeMap::new(),
            open_from: 0,
            closed: Vec::new(),
        };
        w.append(
            "deployment_mode",
            "structural",
            json!({"deployment_mode": deployment_mode}),
        )?;
        Ok(w)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// Appends one event under a fresh commitment key and returns its
    /// address; the seq is assigned here, inside the append (REC-06).
    pub fn append(&mut self, kind: &str, provenance: &str, content: Value) -> Result<Address> {
        let key = commit::fresh_key()?;
        let header = Header {
            writer: self.id.clone(),
            seq: self.log.len() as u64 + 1,
            kind: kind.to_string(),
            provenance: provenance.to_string(),
            prev_hash: self.log.last().map(|e| e.hash),
            chain_heads_seen: self.chain_heads_seen.clone(),
            commitment: commit::commit(&key, &canon::encode(&content)),
        };
        let event = Event {
            hash: header.hash(),
            header,
            body: Some(Body { key, content }),
        };
        let address = event.address();
        self.log.push(event);
        Ok(address)
    }

    /// The chain head at the end of the newest closed segment.
    pub fn closed_chain_head(&self) -> Option<ChainHead> {
        let &(_, end) = self.closed.last()?;
        let e = &self.log[end - 1];
        Some(ChainHead {
            seq: e.header.seq,
            chain_head: e.hash,
        })
    }

    /// Records the other writer's chain head at its newest closed
    /// segment as seen.
    pub fn see(&mut self, other: &Writer) {
        if let Some(h) = other.closed_chain_head() {
            self.chain_heads_seen.insert(other.id.clone(), h);
        }
    }

    /// Closes the open segment, if it holds any event.
    pub fn close_segment(&mut self) -> Option<Segment> {
        if self.open_from == self.log.len() {
            return None;
        }
        let range = (self.open_from, self.log.len());
        self.closed.push(range);
        self.open_from = range.1;
        Some(self.segment(range))
    }

    fn segment(&self, (from, to): (usize, usize)) -> Segment {
        Segment {
            writer: self.id.clone(),
            events: self.log[from..to].to_vec(),
        }
    }

    /// Every closed segment as it stands now, purges applied.
    pub fn segments(&self) -> Vec<Segment> {
        self.closed.iter().map(|&r| self.segment(r)).collect()
    }

    /// The event at `seq`, if appended.
    pub fn event(&self, seq: u64) -> Option<&Event> {
        self.log.get(usize::try_from(seq).ok()?.checked_sub(1)?)
    }

    /// Purges the addresses by a principal act: erases each one's
    /// content and commitment key, and appends a tombstone holding only
    /// addresses, count, reason and commitments (ADM-07).
    pub fn purge(&mut self, addresses: &[Address], reason: &str) -> Result<Address> {
        let mut purged = Vec::new();
        for a in addresses {
            let i = usize::try_from(a.seq).ok().and_then(|s| s.checked_sub(1));
            let e = match i.and_then(|i| self.log.get_mut(i)) {
                Some(e) if a.writer == self.id => e,
                _ => return Err(Error::new(format!("{a:?} is not in this writer"))),
            };
            e.body = None;
            purged.push(
                json!({"writer": a.writer, "seq": a.seq, "commitment": hex(&e.header.commitment)}),
            );
        }
        let tombstone = json!({"purged": purged, "count": addresses.len(), "reason": reason});
        self.append("tombstone", "structural", tombstone)
    }
}

/// The addresses a tombstone's content names.
pub fn purged_addresses(tombstone: &Value) -> Vec<Address> {
    let list = tombstone.get("purged").and_then(Value::as_array);
    list.into_iter()
        .flatten()
        .filter_map(|p| {
            Some(Address {
                writer: p.get("writer")?.as_str()?.to_string(),
                seq: p.get("seq")?.as_u64()?,
            })
        })
        .collect()
}

fn bad(what: &str) -> Error {
    Error::new(format!("segment: missing or malformed {what}"))
}

fn field<'a>(v: &'a Value, k: &str) -> Result<&'a Value> {
    v.get(k).ok_or_else(|| bad(k))
}

fn text<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    field(v, k)?.as_str().ok_or_else(|| bad(k))
}

fn uint(v: &Value, k: &str) -> Result<u64> {
    field(v, k)?.as_u64().ok_or_else(|| bad(k))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn writer() -> Writer {
        keyed(b"k")
    }

    fn keyed(seat_key: &[u8]) -> Writer {
        let mut w = Writer::open(seat_key, "interactive").unwrap();
        w.append("message", "user", json!({"text": "hello"}))
            .unwrap();
        w
    }

    #[test]
    fn a_segment_round_trips_and_its_chain_checks() {
        let mut w = writer();
        let s = w.close_segment().unwrap();
        assert_eq!((s.first(), s.last()), (1, 2));
        assert_eq!(Segment::decode(&s.encode()).unwrap(), s);
    }

    #[test]
    fn an_unknown_format_is_refused() {
        let mut w = writer();
        let bytes = w.close_segment().unwrap().encode();
        let text = String::from_utf8(bytes)
            .unwrap()
            .replacen("\"format\":1", "\"format\":2", 1);
        let err = Segment::decode(text.as_bytes()).unwrap_err();
        assert!(err.0.contains("format 2"), "{err}");
    }

    #[test]
    fn changed_content_no_longer_opens_its_commitment() {
        let mut w = writer();
        let mut s = w.close_segment().unwrap();
        s.events[1].body.as_mut().unwrap().content = json!({"text": "hellO"});
        assert!(s.check().unwrap_err().0.contains("does not open"));
    }

    #[test]
    fn purge_erases_content_and_key_and_keeps_the_chain() {
        let mut w = writer();
        w.close_segment();
        let c = w.event(2).unwrap().header.commitment;
        let t = w
            .purge(
                &[Address {
                    writer: w.id().to_string(),
                    seq: 2,
                }],
                "r",
            )
            .unwrap();
        assert_eq!(t.seq, 3);
        assert_eq!(w.event(2).unwrap().body, None);
        let tomb = &w.event(3).unwrap().body.as_ref().unwrap().content;
        assert_eq!(
            purged_addresses(tomb),
            vec![Address {
                writer: w.id().to_string(),
                seq: 2
            }]
        );
        assert_eq!(tomb["purged"][0]["commitment"], json!(hex(&c)));
        w.close_segment();
        for s in w.segments() {
            s.check().unwrap();
        }
    }

    #[test]
    fn purge_refuses_an_address_of_another_writer() {
        let mut w = writer();
        let other = Address {
            writer: "x".into(),
            seq: 1,
        };
        assert!(w.purge(&[other], "r").is_err());
    }

    #[test]
    fn see_records_only_the_closed_chain_head() {
        let (mut a, mut b) = (keyed(b"a"), keyed(b"b"));
        a.see(&b);
        assert!(a.chain_heads_seen.is_empty());
        b.close_segment();
        b.append("message", "user", json!({"text": "open"}))
            .unwrap();
        a.see(&b);
        assert_eq!(a.chain_heads_seen[b.id()].seq, 2);
    }
}
