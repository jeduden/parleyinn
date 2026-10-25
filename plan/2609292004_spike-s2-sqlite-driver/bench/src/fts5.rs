//! The FTS5 features ParleyInn's recall relies on (ADR-05, RCL-01), on a
//! small fresh store: BM25 ranking, snippet and highlight, phrase,
//! prefix, NEAR and NOT queries, the unicode61, trigram and porter
//! tokenizers, and rebuilding the projection from the record (I10).

use rusqlite::Connection;

use crate::Opts;
use crate::corpus::Event;
use crate::store::{self, Result};

const DOCS: [&str; 9] = [
    "the migration failed because the sqlite driver was missing FTS5",
    "we pinned the constraint: never touch the production database",
    "driver benchmark: modernc versus ncruces at ten million events",
    // ở carries two marks: only remove_diacritics 2 folds it.
    "Ünïcödé naïve café résumé phở text with diacritics",
    "the driver the driver the driver repeated term frequency",
    // Filler without the probe terms, so BM25's IDF stays above zero.
    "compaction keeps the pinned constraints verbatim",
    "the hook failed open and the agent continued",
    "recall stays pull-only and every result is enveloped",
    "payload files are content-addressed by hash",
];

/// Each query and the seqs it must return, best rank first.
const PROBES: [(&str, &str, &[i64]); 6] = [
    ("bm25_order", "driver", &[5, 3, 1]),
    ("phrase", "\"sqlite driver\"", &[1]),
    ("prefix", "bench*", &[3]),
    ("near", "NEAR(migration FTS5, 8)", &[1]),
    ("diacritics_folded", "cafe", &[4]),
    ("boolean_not", "driver NOT benchmark", &[5, 1]),
];

/// Each tokenizer with a query only it answers.
const TOKENIZERS: [(&str, &str); 3] = [
    ("unicode61 remove_diacritics 2", "pho"),
    ("trigram", "igrati"),
    ("porter unicode61", "repeating"),
];

pub fn fts5(o: &Opts) -> Result<()> {
    for suf in ["", "-wal", "-shm"] {
        let mut p = o.db.clone().into_os_string();
        p.push(suf);
        let _ = std::fs::remove_file(p);
    }
    let mut db = store::open(&o.db, o.key, false)?;
    let version: String = db.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
    println!("RESULT fts5 sqlite_version={version}");
    store::create_schema(&db)?;
    {
        let tx = store::begin(&mut db)?;
        let mut prev = vec![0u8; 32];
        for (i, b) in DOCS.iter().enumerate() {
            let e = Event {
                session: "s".into(),
                source: 1,
                line: i as i64,
                kind: "user",
                trust: 0,
                tool: None,
                body: (*b).into(),
                payload: false,
                ts: 0,
            };
            prev = store::append(&tx, i as i64 + 1, &e, &prev)?;
        }
        tx.commit()?;
    }
    for (name, q, want) in PROBES {
        let got = probe(&db, q);
        let ids: Vec<i64> = got.as_ref().map(|h| h.iter().map(|x| x.0).collect()).unwrap_or_default();
        let hits: Vec<String> = got
            .as_ref()
            .map(|h| h.iter().map(|(id, s, sn, hl)| format!("{id}({s:.3}){sn}{{{hl}}}")).collect())
            .unwrap_or_default();
        println!(
            "RESULT fts5 probe={name} ok={} ids={ids:?} want={want:?} err={:?} hits={:?}",
            got.is_ok() && ids == want,
            got.as_ref().err().map(|e| e.to_string()).unwrap_or_default(),
            hits.join(" | ")
        );
    }
    for (tok, q) in TOKENIZERS {
        let got = tokenizer(&db, tok, q);
        println!(
            "RESULT fts5 tokenizer={tok:?} query={q:?} ok={} hits={} err={:?}",
            matches!(got, Ok(1)),
            got.as_ref().copied().unwrap_or(-1),
            got.as_ref().err().map(|e| e.to_string()).unwrap_or_default()
        );
    }
    let rebuilt = db
        .execute("INSERT INTO events_fts(events_fts) VALUES('delete-all')", [])
        .and_then(|_| db.execute("INSERT INTO events_fts(events_fts) VALUES('rebuild')", []));
    let n: i64 = db.query_row("SELECT count(*) FROM events_fts WHERE events_fts MATCH 'driver'", [], |r| r.get(0))?;
    println!(
        "RESULT fts5 probe=rebuild_from_record ok={} hits_after={n} err={:?}",
        rebuilt.is_ok() && n == 3,
        rebuilt.err().map(|e| e.to_string()).unwrap_or_default()
    );
    let mut st = db.prepare("PRAGMA compile_options")?;
    let opts: Vec<String> = st
        .query_map([], |r| r.get::<_, String>(0))?
        .filter_map(|x| x.ok())
        .filter(|s| ["FTS", "THREADSAFE", "DQS", "DBSTAT", "LOAD_EXTENSION", "HAS_CODEC", "CRYPTO"].iter().any(|k| s.contains(k)))
        .collect();
    println!("RESULT fts5 compile_options={:?}", opts.join(","));
    Ok(())
}

type Hit = (i64, f64, String, String);

fn probe(db: &Connection, q: &str) -> rusqlite::Result<Vec<Hit>> {
    let mut st = db.prepare(
        "SELECT rowid, round(bm25(events_fts), 3), snippet(events_fts, 0, '[', ']', '…', 6),
         highlight(events_fts, 0, '[', ']')
         FROM events_fts WHERE events_fts MATCH ? ORDER BY rank",
    )?;
    st.query_map([q], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect()
}

fn tokenizer(db: &Connection, tok: &str, q: &str) -> rusqlite::Result<i64> {
    let table = format!("t_{}", tok.split_whitespace().next().unwrap_or("x"));
    db.execute_batch(&format!("CREATE VIRTUAL TABLE {table} USING fts5(x, tokenize='{tok}')"))?;
    for d in DOCS {
        db.execute(&format!("INSERT INTO {table}(x) VALUES(?)"), [d])?;
    }
    db.query_row(&format!("SELECT count(*) FROM {table} WHERE {table} MATCH ?"), [q], |r| r.get(0))
}
