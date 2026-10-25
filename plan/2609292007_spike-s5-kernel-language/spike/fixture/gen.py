"""Generate the spike S5 fixture record, deterministically.

Six runs, each in its own run-seat writer (A1 to A6), and one device-seat
writer (D1) holding the operator's pin acts. Every event is a flat object
with the same twelve fields; a field that does not apply is null.

    python3 gen.py > record.json
"""

import json
import random
import sys
from datetime import datetime, timezone

SEED = 2609292007
BASE = int(datetime(2026, 10, 1, tzinfo=timezone.utc).timestamp())

FILES = [
    "crates/parinn/src/main.rs",
    "crates/parinn/src/store/mod.rs",
    "crates/parinn/src/store/segment.rs",
    "crates/parinn/src/recall/search.rs",
    "crates/parinn/src/recall/envelope.rs",
    "crates/parinn/src/restore/block.rs",
    "crates/parinn/src/hook/handler.rs",
    "crates/parinn/tests/reach.rs",
    "docs/srs/05-functional-requirements.md",
    "docs/testing.md",
    "features/recall.feature",
    "features/restore.feature",
    "tooling/scenario/tests/bdd/recall.rs",
    "Cargo.toml",
    "README.md",
]
COMMANDS = [
    "cargo test -p parinn",
    "cargo clippy --all-targets",
    "cargo fmt --check",
    "git status",
    "git diff --stat",
    "mdsmith check .",
    "cargo test -p scenario",
    "cargo build --release",
]
PATTERNS = ["TrustedText", "event_search", "fn restore", "seq", "quarantine", "TODO"]
URLS = [
    "https://docs.rs/rusqlite",
    "https://www.sqlite.org/lang_with.html",
    "https://www.lua.org/manual/5.4/",
]
PROMPTS = [
    "Run the tests and fix what fails",
    "Add a unit test for the envelope builder",
    "Why does the restore block omit pin p3?",
    "Refactor the segment writer into smaller functions",
    "Check clippy and fix the warnings",
    "Update the SRS row for RCL-07 and its scenario",
    "Make the recall test deterministic",
    "Look up how rusqlite cancels a query",
    "Summarize what changed in the store today",
    "Rename the handler module and update its tests",
]
RECALL_QUERIES = [
    "restore block budget",
    "pin p3",
    "segment seal",
    "clippy warning",
    "test failure recall",
    "TrustedText",
]
ERRORS = {
    "Bash": None,  # exit codes, below
    "Read": "file not found",
    "Edit": "old_string not found in file",
    "Write": "permission denied",
    "Grep": "invalid regex",
    "Glob": "no matches",
    "WebFetch": "timeout after 30s",
}
ERROR_RATE = {
    "Bash": 0.30,
    "Read": 0.05,
    "Edit": 0.20,
    "Write": 0.03,
    "Grep": 0.04,
    "Glob": 0.10,
    "WebFetch": 0.25,
}
TOOL_WEIGHTS = [
    ("Bash", 30),
    ("Read", 28),
    ("Edit", 18),
    ("Grep", 10),
    ("Glob", 5),
    ("Write", 5),
    ("WebFetch", 4),
]
DURATION = {
    "Bash": (2, 180),
    "Read": (0, 2),
    "Edit": (0, 3),
    "Write": (0, 2),
    "Grep": (0, 5),
    "Glob": (0, 2),
    "WebFetch": (1, 30),
}


class Writer:
    def __init__(self, label, run):
        self.label = label
        self.run = run
        self.seq = 0
        self.events = []

    def add(self, ts, provenance, kind, tokens, text, tool=None, call_id=None,
            status=None, args=None):
        self.seq += 1
        self.events.append({
            "writer": self.label,
            "seq": self.seq,
            "run": self.run,
            "ts": ts,
            "provenance": provenance,
            "kind": kind,
            "tool": tool,
            "call_id": call_id,
            "status": status,
            "tokens": tokens,
            "text": text,
            "args": None if args is None else json.dumps(args, sort_keys=True),
        })


def pick_tool(rng):
    total = sum(w for _, w in TOOL_WEIGHTS)
    r = rng.uniform(0, total)
    for name, w in TOOL_WEIGHTS:
        r -= w
        if r <= 0:
            return name
    return TOOL_WEIGHTS[-1][0]


def tool_args(rng, tool):
    if tool == "Bash":
        return {"command": rng.choice(COMMANDS)}
    if tool in ("Read", "Edit", "Write"):
        return {"file_path": rng.choice(FILES)}
    if tool == "Grep":
        return {"pattern": rng.choice(PATTERNS), "path": "crates"}
    if tool == "Glob":
        return {"pattern": rng.choice(["**/*.rs", "features/*.feature", "docs/**/*.md"])}
    return {"url": rng.choice(URLS)}


def result_text(rng, tool, ok):
    if ok:
        return {
            "Bash": "finished",
            "Read": "file contents",
            "Edit": "edit applied",
            "Write": "file written",
            "Grep": "%d matches" % rng.randint(1, 40),
            "Glob": "%d files" % rng.randint(1, 60),
            "WebFetch": "page fetched",
        }[tool]
    if tool == "Bash":
        return "command failed: exit code %d" % rng.choice([1, 1, 1, 2, 101, 127])
    return ERRORS[tool]


def gen_run(rng, label, run, start, turns):
    w = Writer(label, run)
    ts = start
    ctx = 0
    calls = 0
    for turn in range(turns):
        # Idle before the prompt: mostly short, sometimes a long break.
        if turn:
            ts += rng.choice([rng.randint(20, 600)] * 4 + [rng.randint(2400, 20000)])
        prompt = rng.choice(PROMPTS)
        w.add(ts, "user", "message", 10 + len(prompt) // 4, prompt)
        ctx += 300
        steps = rng.randint(2, 9)
        last_failed = None
        for _ in range(steps):
            ts += rng.randint(2, 40)
            toks = rng.randint(40, 900)
            w.add(ts, "assistant", "message", toks, "Working on it")
            ctx += toks
            # A retry of the last failed call, or one or two fresh calls.
            if last_failed and rng.random() < 0.6:
                batch = [last_failed]
            else:
                n = 2 if rng.random() < 0.2 else 1
                batch = []
                for _ in range(n):
                    tool = pick_tool(rng)
                    batch.append((tool, tool_args(rng, tool)))
            issued = []
            for tool, args in batch:
                calls += 1
                cid = "%s-c%03d" % (run, calls)
                ts += rng.randint(0, 2)
                w.add(ts, "assistant", "tool_call", rng.randint(20, 120),
                      "call " + tool, tool=tool, call_id=cid, args=args)
                issued.append((tool, args, cid, ts))
            # Results; a parallel pair may come back in either order.
            if len(issued) == 2 and rng.random() < 0.5:
                issued.reverse()
            last_failed = None
            for tool, args, cid, call_ts in issued:
                lo, hi = DURATION[tool]
                ts = max(ts, call_ts + rng.randint(lo, hi))
                ok = rng.random() >= ERROR_RATE[tool]
                toks = rng.randint(20, 4000) if ok else rng.randint(10, 80)
                w.add(ts, "tool_result:" + tool, "tool_result", toks,
                      result_text(rng, tool, ok), tool=tool, call_id=cid,
                      status="ok" if ok else "error")
                ctx += toks
                if not ok:
                    last_failed = (tool, args)
            if ctx > 60000:
                ts += rng.randint(5, 30)
                trigger = "auto" if rng.random() < 0.75 else "manual"
                w.add(ts, "harness_meta", "compaction", 0, "compaction",
                      args={"trigger": trigger, "pre_tokens": ctx})
                ctx = rng.randint(8000, 15000)
                if rng.random() < 0.7:
                    ts += rng.randint(3, 20)
                    q = rng.choice(RECALL_QUERIES)
                    w.add(ts, "structural", "recall", 0, "recall",
                          args={"tool": "event_search", "query": q})
        ts += rng.randint(2, 30)
        w.add(ts, "assistant", "message", rng.randint(40, 600), "Done")
    return w, ts


def gen_orphans(rng, w, ts):
    # A run cut off mid-call: tool calls with no result.
    for _ in range(rng.randint(1, 2)):
        ts += rng.randint(1, 20)
        tool = pick_tool(rng)
        cid = "%s-x%02d" % (w.run, w.seq)
        w.add(ts, "assistant", "tool_call", rng.randint(20, 120),
              "call " + tool, tool=tool, call_id=cid, args=tool_args(rng, tool))
    return ts


def gen_pins(rng, ends):
    d = Writer("D1", None)
    active = []
    n = 0
    ts = BASE + 3600
    for _ in range(22):
        ts += rng.randint(600, 30000)
        if active and rng.random() < 0.4:
            pid = rng.choice(active)
            active.remove(pid)
            d.add(ts, "operator", "pin", 15, "unpin " + pid,
                  args={"action": "remove", "pin_id": pid})
        else:
            n += 1
            pid = "p%d" % n
            active.append(pid)
            d.add(ts, "operator", "pin", 25, "Never push to main without review",
                  args={"action": "add", "pin_id": pid})
    return d


def main():
    rng = random.Random(SEED)
    writers = []
    ends = []
    start = BASE + 7 * 3600
    for i in range(6):
        label = "A%d" % (i + 1)
        run = "r%d" % (i + 1)
        w, end = gen_run(rng, label, run, start, rng.randint(18, 34))
        if i in (2, 4):
            end = gen_orphans(rng, w, end)
        writers.append(w)
        ends.append(end)
        start += rng.randint(4 * 3600, 30 * 3600)
    writers.append(gen_pins(rng, ends))
    events = [e for w in writers for e in w.events]
    json.dump(events, sys.stdout, indent=0, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
