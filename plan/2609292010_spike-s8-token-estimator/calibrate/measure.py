"""Count each sample's model tokens with Claude's own counts.

Each count is one headless Claude Code call in an isolated home whose user
message is "<t>\\n" + text + "\\n</t>". The call's input usage (input, cache
read and cache creation tokens) minus the same call's usage with an empty
text is the sample's model-token count: the text and the two newlines that
join it to its delimiters, as ParleyInn inserts text into a block. The baseline
is measured before and after the run and must not move.

Usage: python3 -I measure.py SAMPLE.jsonl MODEL [EVERY]
  Counts every EVERY-th sample (default 1) on MODEL and rewrites the file.
Needs: claude on PATH, signed in; S8_HOME, an empty scratch home.
"""

import concurrent.futures
import json
import os
import subprocess
import sys

WORKERS = 6
SYSTEM = "Reply with the single word ok."


def env():
    home = os.environ["S8_HOME"]
    e = dict(os.environ)
    e["HOME"] = home
    e["CLAUDE_CONFIG_DIR"] = os.path.join(home, ".claude")
    return e, os.path.join(home, "work")


def total_input(model, text):
    e, work = env()
    os.makedirs(work, exist_ok=True)
    msg = {"type": "user", "message": {"role": "user", "content": [
        {"type": "text", "text": "<t>\n" + text + "\n</t>"}]}}
    cmd = ["claude", "-p", "--model", model, "--tools", "", "--effort", "low",
           "--system-prompt", SYSTEM, "--setting-sources", "", "--strict-mcp-config",
           "--input-format", "stream-json", "--output-format", "stream-json",
           "--verbose", "--no-session-persistence"]
    for attempt in range(4):
        p = subprocess.run(cmd, input=json.dumps(msg) + "\n", text=True, cwd=work,
                           env=e, capture_output=True, timeout=600)
        for line in p.stdout.splitlines():
            d = json.loads(line)
            if d.get("type") == "result" and not d.get("is_error"):
                u = d["usage"]
                return u["input_tokens"] + u["cache_read_input_tokens"] + u["cache_creation_input_tokens"]
    raise RuntimeError(f"no usable result after retries: {p.stdout[-400:]} {p.stderr[-400:]}")


def main():
    path, model = sys.argv[1], sys.argv[2]
    every = int(sys.argv[3]) if len(sys.argv) > 3 else 1
    samples = [json.loads(l) for l in open(path, encoding="utf-8")]
    todo = [s for i, s in enumerate(samples)
            if i % every == 0 and model not in s.get("tokens", {})]
    before = total_input(model, "")
    print(f"{model}: baseline {before}, {len(todo)} to count", file=sys.stderr)
    done = 0
    with concurrent.futures.ThreadPoolExecutor(WORKERS) as pool:
        futures = {pool.submit(total_input, model, s["text"]): s for s in todo}
        for f in concurrent.futures.as_completed(futures):
            s = futures[f]
            s.setdefault("tokens", {})[model] = f.result() - before
            done += 1
            if done % 50 == 0:
                print(f"  {done}/{len(todo)}", file=sys.stderr)
    after = total_input(model, "")
    if after != before:
        sys.exit(f"baseline moved from {before} to {after}; counts discarded")
    with open(path, "w", encoding="utf-8") as f:
        for s in samples:
            f.write(json.dumps(s, ensure_ascii=False) + "\n")
    print(f"{model}: baseline held at {before}", file=sys.stderr)


if __name__ == "__main__":
    main()
