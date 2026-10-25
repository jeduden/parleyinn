package main

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"slices"
	"strings"
	"time"
)

// cmdFTS5 probes the FTS5 features ParleyInn's recall relies on (ADR-05,
// RCL-01) on a small fresh store: BM25 ranking, snippet, phrase,
// prefix and NEAR queries, the unicode61 and trigram tokenizers, and
// rebuilding the projection from the record (I10).
func cmdFTS5(ctx context.Context, o opts) error {
	for _, suf := range []string{"", "-wal", "-shm"} {
		os.Remove(o.db + suf)
	}
	db, err := open(o)
	if err != nil {
		return err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	var version string
	if err := db.QueryRowContext(ctx, `SELECT sqlite_version()`).Scan(&version); err != nil {
		return fmt.Errorf("sqlite version: %w", err)
	}
	fmt.Printf("RESULT fts5 driver=%s sqlite_version=%s\n", o.driver, version)
	if err := createSchema(ctx, db); err != nil {
		return err
	}
	docs := fts5Docs()
	if err := appendBodies(ctx, db, docs); err != nil {
		return err
	}
	for _, p := range fts5Probes() {
		got, err := probeHits(ctx, db, p.q)
		ids := hitIDs(got)
		fmt.Printf("RESULT fts5 driver=%s probe=%s ok=%v ids=%v want=%v err=%q hits=%q\n", o.driver, p.name,
			err == nil && slices.Equal(ids, p.want), ids, p.want, errString(err), strings.Join(hitStrings(got), " | "))
	}
	// Each tokenizer is exercised, not just created: a query only that
	// tokenizer can answer must hit the one document it targets.
	for _, t := range fts5Tokenizers() {
		hits, err := probeTokenizer(ctx, db, t.tok, t.q, docs)
		fmt.Printf("RESULT fts5 driver=%s tokenizer=%q query=%q ok=%v hits=%d err=%q\n", o.driver, t.tok, t.q, err == nil && hits == 1, hits, errString(err))
	}
	_, err = db.ExecContext(ctx, `INSERT INTO events_fts(events_fts) VALUES('delete-all')`)
	if err == nil {
		_, err = db.ExecContext(ctx, `INSERT INTO events_fts(events_fts) VALUES('rebuild')`)
	}
	var n int64
	db.QueryRowContext(ctx, `SELECT count(*) FROM events_fts WHERE events_fts MATCH 'driver'`).Scan(&n)
	fmt.Printf("RESULT fts5 driver=%s probe=rebuild_from_record ok=%v hits_after=%d err=%q\n", o.driver, err == nil && n == 3, n, errString(err))
	var opts []string
	rows, err := db.QueryContext(ctx, `PRAGMA compile_options`)
	if err == nil {
		for rows.Next() {
			var s string
			rows.Scan(&s)
			if strings.Contains(s, "FTS") || strings.Contains(s, "THREADSAFE") || strings.Contains(s, "DQS") || strings.Contains(s, "DBSTAT") {
				opts = append(opts, s)
			}
		}
		rows.Close()
	}
	fmt.Printf("RESULT fts5 driver=%s compile_options=%q\n", o.driver, strings.Join(opts, ","))

	return nil
}

// fts5Docs is the probe store's text, one event per document, seq
// counting from 1.
func fts5Docs() []string {
	return []string{
		"the migration failed because the sqlite driver was missing FTS5",
		"we pinned the constraint: never touch the production database",
		"driver benchmark: modernc versus ncruces at ten million events",
		// ở carries two marks: only remove_diacritics 2 folds it.
		"Ünïcödé naïve café résumé phở text with diacritics",
		"the driver the driver the driver repeated term frequency",
		// Filler without the probe terms: BM25's IDF floors at zero
		// for a term in half the documents or more, which would tie
		// every bm25_order score and prove no ranking at all.
		"compaction keeps the pinned constraints verbatim",
		"the hook failed open and the agent continued",
		"recall stays pull-only and every result is enveloped",
		"payload files are content-addressed by hash",
	}
}

// fts5Tokenizers pairs each tokenizer the probe exercises with a query
// only that tokenizer answers: the store's default unicode61, which
// is remove_diacritics 1, finds no document for it.
func fts5Tokenizers() []struct{ tok, q string } {
	return []struct{ tok, q string }{
		{"unicode61 remove_diacritics 2", `pho`},
		{"trigram", `igrati`},
		{"porter unicode61", `repeating`},
	}
}

// fts5Probe is one FTS5 query and the seqs it must return, best rank
// first. A probe passes only on exactly those documents in that order,
// so a driver whose FTS5 parses a query but answers it wrongly fails.
type fts5Probe struct {
	name, q string
	want    []int64
}

func fts5Probes() []fts5Probe {
	return []fts5Probe{
		// The three-fold repeat outranks the shorter document, which
		// outranks the longer one.
		{"bm25_order", `driver`, []int64{5, 3, 1}},
		{"phrase", `"sqlite driver"`, []int64{1}},
		{"prefix", `bench*`, []int64{3}},
		{"near", `NEAR(migration FTS5, 8)`, []int64{1}},
		{"diacritics_folded", `cafe`, []int64{4}},
		{"boolean_not", `driver NOT benchmark`, []int64{5, 1}},
	}
}

// appendBodies appends one event per body in a single transaction,
// seq counting from 1.
func appendBodies(ctx context.Context, db *sql.DB, bodies []string) error {
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return err
	}
	defer tx.Rollback()
	a, err := newAppender(ctx, tx)
	if err != nil {
		return err
	}
	defer a.close()
	prev := make([]byte, 32)
	for i, b := range bodies {
		ev := event{session: "s", source: 1, line: int64(i), kind: "user", body: b}
		if prev, err = a.append(ctx, int64(i+1), ev, prev); err != nil {
			return err
		}
	}

	return tx.Commit()
}

// probeHit is one row a probe query returned.
type probeHit struct {
	id              int64
	score           float64
	snippet, marked string
}

// probeHits runs q with BM25, snippet and highlight, best rank first.
func probeHits(ctx context.Context, db *sql.DB, q string) ([]probeHit, error) {
	rows, err := db.QueryContext(ctx, `SELECT rowid, round(bm25(events_fts), 3), snippet(events_fts, 0, '[', ']', '…', 6),
		highlight(events_fts, 0, '[', ']')
		FROM events_fts WHERE events_fts MATCH ? ORDER BY rank`, q)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []probeHit
	for rows.Next() {
		var h probeHit
		if err := rows.Scan(&h.id, &h.score, &h.snippet, &h.marked); err != nil {
			return nil, err
		}
		out = append(out, h)
	}

	return out, rows.Err()
}

// probeIDs returns the seqs q matches, best rank first.
func probeIDs(ctx context.Context, db *sql.DB, q string) ([]int64, error) {
	hits, err := probeHits(ctx, db, q)

	return hitIDs(hits), err
}

func hitIDs(hits []probeHit) []int64 {
	ids := make([]int64, len(hits))
	for i, h := range hits {
		ids[i] = h.id
	}

	return ids
}

func hitStrings(hits []probeHit) []string {
	out := make([]string, len(hits))
	for i, h := range hits {
		out[i] = fmt.Sprintf("%d(%.3f)%s{%s}", h.id, h.score, h.snippet, h.marked)
	}

	return out
}

// probeTokenizer indexes docs in a fresh FTS5 table with tokenizer tok
// and counts the documents query q matches.
func probeTokenizer(ctx context.Context, db *sql.DB, tok, q string, docs []string) (int64, error) {
	table := "t_" + strings.Fields(tok)[0]
	if _, err := db.ExecContext(ctx, `CREATE VIRTUAL TABLE `+table+` USING fts5(x, tokenize='`+tok+`')`); err != nil {
		return 0, err
	}
	for _, d := range docs {
		if _, err := db.ExecContext(ctx, `INSERT INTO `+table+`(x) VALUES(?)`, d); err != nil {
			return 0, err
		}
	}
	var n int64
	err := db.QueryRowContext(ctx, `SELECT count(*) FROM `+table+` WHERE `+table+` MATCH ?`, q).Scan(&n)

	return n, err
}

// cmdOptimize reports the FTS5 index's segment count, merges it into
// one segment with the 'optimize' command, and reports the time and
// the store's size afterwards. Bulk loading leaves many segments, and
// every query reads each one.
func cmdOptimize(ctx context.Context, o opts) error {
	db, err := open(o)
	if err != nil {
		return err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	segs := func() int64 {
		var n int64
		db.QueryRowContext(ctx, `SELECT count(DISTINCT segid) FROM events_fts_idx`).Scan(&n)
		return n
	}
	before := segs()
	t := time.Now()
	if _, err := db.ExecContext(ctx, `INSERT INTO events_fts(events_fts) VALUES('optimize')`); err != nil {
		return err
	}
	el := time.Since(t)
	if _, err := db.ExecContext(ctx, `PRAGMA wal_checkpoint(TRUNCATE)`); err != nil {
		return err
	}
	fmt.Printf("RESULT optimize driver=%s segments_before=%d segments_after=%d elapsed=%s\n", o.driver, before, segs(), el.Round(time.Second))

	return reportSize(ctx, db, o)
}
