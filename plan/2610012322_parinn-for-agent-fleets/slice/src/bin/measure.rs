//! Spike S12 at scale. Run from the slice's directory:
//!
//! ```text
//! cargo run --release --bin measure -- [events per writer] [events per segment] [purges]
//! ```
//!
//! Prints Markdown table rows for the report: rebuild cost, whether an
//! index kept incrementally in arrival order matches a rebuild, and
//! what an in-place purge leaves in the index file.

use std::path::{Path, PathBuf};
use std::time::Instant;

use s12slice::fixture::{self, TwoWriters};
use s12slice::index::{self, Ids, Incremental};
use s12slice::node::Node;
use s12slice::record::{Address, Event, Segment};
use s12slice::{Result, room};

fn fresh(dir: &Path, name: &str) -> Result<PathBuf> {
    let p = dir.join(name);
    let _ = std::fs::remove_dir_all(&p);
    let _ = std::fs::remove_file(&p);
    Ok(p)
}

fn row(what: &str, value: impl std::fmt::Display) {
    println!("| {what} | {value} |");
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

fn orders(f: &TwoWriters) -> [(&'static str, Vec<&Segment>); 2] {
    [
        (
            "A then B",
            f.a_segments.iter().chain(&f.b_segments).collect(),
        ),
        (
            "B then A",
            f.b_segments.iter().chain(&f.a_segments).collect(),
        ),
    ]
}

fn main() -> Result<()> {
    let args: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let per_writer = args.first().copied().unwrap_or(100_000);
    let per_segment = args.get(1).copied().unwrap_or(500);
    let purges = args.get(2).copied().unwrap_or(40);
    let dir = std::env::temp_dir().join("s12-measure");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;

    let t = Instant::now();
    let f = fixture::two_writers(per_writer, per_segment)?;
    let secs = t.elapsed().as_secs_f64();
    let all: Vec<&Event> = f
        .a_segments
        .iter()
        .chain(&f.b_segments)
        .flat_map(|s| &s.events)
        .collect();
    let n = all.len();
    println!("| Measure | Value |\n| --- | --- |");
    row("events, both writers", n);
    row(
        "segments, both writers",
        f.a_segments.len() + f.b_segments.len(),
    );
    row(
        "append, with a commitment key each",
        format!("{:.0} events/s", n as f64 / secs),
    );
    let a = f.a.id().to_string();

    // A full rebuild in each arrival order, on A's node.
    let mut rebuilt = Vec::new();
    for (name, order) in orders(&f) {
        let mut node = Node::new(&fresh(&dir, &format!("node {name}"))?, &[&a])?;
        let t = Instant::now();
        for s in &order {
            node.receive(s)?;
        }
        let receive = t.elapsed().as_secs_f64();
        let t = Instant::now();
        let p = node.rebuild()?;
        let rebuild = t.elapsed().as_secs_f64();
        row(
            &format!("receive and store, {name}"),
            format!("{receive:.2} s, {:.0} events/s", n as f64 / receive),
        );
        row(
            &format!("full rebuild, {name}"),
            format!("{rebuild:.2} s, {:.0} events/s", n as f64 / rebuild),
        );
        rebuilt.push((std::fs::read(&p)?, index::logical_digest(&p)?, node));
    }
    row(
        "index file size",
        format!("{:.1} MiB", rebuilt[0].0.len() as f64 / 1048576.0),
    );
    row(
        "full rebuild: both orders byte-identical",
        yes(rebuilt[0].0 == rebuilt[1].0),
    );

    // The same index kept incrementally, segment by segment, with row
    // ids in arrival order or from the event's hash.
    for ids in [Ids::Arrival, Ids::FromHash] {
        let reference = {
            let p = fresh(&dir, &format!("rebuilt {ids:?}.db"))?;
            let node = &rebuilt[0].2;
            index::build_with(&p, node, &room::derive(node), ids)?;
            index::optimize_and_vacuum(&p)?;
            std::fs::read(&p)?
        };
        let mut kept = Vec::new();
        for (name, order) in orders(&f) {
            let mut node = Node::new(&fresh(&dir, &format!("incremental node {name}"))?, &[&a])?;
            let p = fresh(&dir, &format!("incremental {name}.db"))?;
            let mut inc = Incremental::create(&p, ids)?;
            let t = Instant::now();
            for s in &order {
                node.receive(s)?;
                inc.add(&node, s)?;
            }
            inc.set_room(&room::derive(&node))?;
            let secs = t.elapsed().as_secs_f64();
            row(
                &format!("incremental, ids {ids:?}, {name}"),
                format!("{secs:.2} s with receive"),
            );
            let bytes = std::fs::read(&p)?;
            let digest = index::logical_digest(&p)?;
            drop(inc);
            index::optimize_and_vacuum(&p)?;
            kept.push((bytes, digest, std::fs::read(&p)?));
        }
        let tag = format!("incremental, ids {ids:?}");
        row(
            &format!("{tag}: both orders byte-identical"),
            yes(kept[0].0 == kept[1].0),
        );
        row(
            &format!("{tag}, optimized and vacuumed: both orders byte-identical"),
            yes(kept[0].2 == kept[1].2),
        );
        row(
            &format!("{tag}, optimized and vacuumed: byte-identical to a rebuild optimized alike"),
            yes(kept[0].2 == reference),
        );
        let logical = kept.iter().all(|k| k.1 == rebuilt[0].1);
        row(
            &format!("{tag}: logical content equal to the rebuild's"),
            yes(logical),
        );
    }

    // In-place purges against a purge by rebuild, over A's first user
    // messages (40 unless the third argument says): what does each
    // leave in the index file?
    let victims: Vec<(Address, String)> = all
        .iter()
        .filter(|e| e.header.provenance == "user" && e.address().writer == a)
        .filter_map(|e| {
            let text = e.body.as_ref()?.content.get("text")?.as_str()?;
            Some((e.address(), text.to_string()))
        })
        .take(purges)
        .collect();
    if victims.is_empty() {
        return Ok(());
    }
    let opening = |t: &str| t.as_bytes()[..t.len().min(48)].to_vec();
    let rebuilt_path = dir.join("node A then B").join("index.db");
    let before = std::fs::read(&rebuilt_path)?;
    let held = victims
        .iter()
        .filter(|v| contains(&before, &opening(&v.1)))
        .count();
    row(
        "purged events, each from a fresh copy of the rebuilt index",
        format!("{}; the index holds the opening of {held}", victims.len()),
    );
    let fts_before = index::fts5_data_bytes(&rebuilt_path)?;
    for (name, secure, fts5_secure) in [
        ("secure_delete off", false, false),
        ("secure_delete on", true, false),
        ("secure_delete on, FTS5 secure-delete on", true, true),
    ] {
        let (mut left, mut ms, mut grew) = (0, 0.0, 0i64);
        let p = fresh(&dir, "purged in place.db")?;
        for (address, text) in &victims {
            std::fs::copy(&rebuilt_path, &p)?;
            let t = Instant::now();
            index::purge_in_place(&p, address, secure, fts5_secure)?;
            ms += t.elapsed().as_secs_f64() * 1000.0;
            left += usize::from(contains(&std::fs::read(&p)?, &opening(text)));
            grew += index::fts5_data_bytes(&p)? - fts_before;
        }
        let k = victims.len() as f64;
        row(
            &format!("in-place purge, {name}"),
            format!(
                "{:.1} ms each; {left} leave the text's opening in the file; FTS5 data {:+.0} bytes each",
                ms / k,
                grew as f64 / k
            ),
        );
    }
    Ok(())
}
