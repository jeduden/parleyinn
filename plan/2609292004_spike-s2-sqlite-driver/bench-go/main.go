// Command bench is spike S2's harness: it builds the same synthetic
// ParleyInn store with modernc.org/sqlite and github.com/ncruces/go-sqlite3
// and measures each against NFR-03, NFR-04, NFR-05 and NFR-09.
//
// It is a separate module, so neither driver enters parinn's module
// graph before ADR-07's decision lands.
package main

import (
	"context"
	"flag"
	"fmt"
	"os"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "usage: bench <load|search|expand|ingest|hook|cancel|wal|walworker|fts5|optimize> [flags]")
		os.Exit(2)
	}
	cmd, args := os.Args[1], os.Args[2:]
	fs := flag.NewFlagSet(cmd, flag.ExitOnError)
	o := opts{}
	fs.StringVar(&o.driver, "driver", "modernc", "modernc or ncruces")
	fs.StringVar(&o.db, "db", "", "database path")
	fs.Int64Var(&o.n, "n", 1_000_000, "events (load), queries (search), or transactions per worker (walworker)")
	fs.IntVar(&o.batch, "batch", 100, "events per transaction")
	fs.IntVar(&o.workers, "workers", 50, "concurrent writer processes (wal)")
	fs.IntVar(&o.id, "id", 0, "worker id (walworker)")
	fs.Uint64Var(&o.seed, "seed", 1, "corpus seed")
	fs.Int64Var(&o.recent, "recent", 0, "search: rank only the most recent this many matches (0 ranks all)")
	fs.Int64Var(&o.window, "window", defaultWindow, "search: rank only the most recent this many events")
	fs.StringVar(&o.vfs, "vfs", "", "encrypting VFS for the OQ-04 probe: xts or adiantum (ncruces only)")
	fs.Parse(args)

	ctx := context.Background()
	var err error
	switch cmd {
	case "load":
		err = cmdLoad(ctx, o)
	case "search":
		err = cmdSearch(ctx, o)
	case "expand":
		err = cmdExpand(ctx, o)
	case "ingest":
		err = cmdIngest(ctx, o)
	case "hook":
		err = cmdHook(ctx, o)
	case "cancel":
		err = cmdCancel(ctx, o)
	case "wal":
		err = cmdWAL(ctx, o)
	case "walworker":
		err = cmdWALWorker(ctx, o)
	case "fts5":
		err = cmdFTS5(ctx, o)
	case "optimize":
		err = cmdOptimize(ctx, o)
	default:
		err = fmt.Errorf("unknown command %q", cmd)
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, "bench:", err)
		os.Exit(1)
	}
}

// defaultWindow leaves search unbounded: it exceeds any store.
const defaultWindow = 1 << 62

type opts struct {
	driver  string
	db      string
	n       int64
	batch   int
	workers int
	id      int
	seed    uint64
	window  int64
	recent  int64
	vfs     string // encrypting VFS for the OQ-04 probe (ncruces only); empty opens plain files
}
