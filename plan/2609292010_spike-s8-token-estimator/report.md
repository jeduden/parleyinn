---
title: "Spike S8 report"
summary: >-
  A conservative model-token estimator calibrated against Claude's own
  counts on 646 samples of record content, hook handler baselines on a
  1M-event store, and the fate of every initial target the SRS marks
  with a dagger.
---
# Spike S8 report: the model-token estimator and the initial targets

Spike S8 answers SRS §8.4 and freezes the † targets of §7 and the rest
of the SRS that it has evidence for. Every other † target is re-planned
to the spike or milestone that builds what measures it, in the SRS's
new §12.3.

## Decision

- **The estimator.** Each character adds the weight of one of eleven
  classes, split by script, and each switch between lowercase, uppercase
  and digit inside a sequence of letters and digits adds a twelfth weight.
  The sum, rounded up, plus 6 is the estimate. On the committed sample
  of 646 texts it is at or above Claude's count for all 646, and for
  99.85% of samples under five-fold cross validation. It overcounts by
  1.40× on the whole sample, from 1.34× on multilingual text to 1.63×
  on short English pins.
- **Claude's tokenizer is dense.** English prose runs at 3.0 bytes per
  model token, code at 2.4, transcript JSON at 1.9 and random
  identifiers at 1.3. "Bytes divided by four" falls under the count for
  97.4% of the sample; one model token per byte still misses 0.5%.
- **Budgets count the estimate.** §8.4 now says so. A restore block
  limit of 2,000 model tokens holds about 1,400 true ones; a recall tool's
  output under RCL-03's cap of 8,000 holds 10–16 KB of record content.
- **Hook handlers have room.** On emulated reference hardware with a
  1M-event store, ten appending processes and the room view reading,
  `UserPromptSubmit` takes 3.2 ms and `SessionStart` 4.2 ms at p95,
  against 50 and 150 ms. Each peaks at 10.8 MiB RSS against 50 MiB.
- **Frozen:** 9 † targets, the ones S2 and S8 measured and M1 needs
  from its start, values unchanged. **Re-planned:** the 44 left, each in
  §12.3 with what freezes it; a target freezes as late as it can.
- **One target does not fit its neighbour.** A 1,000-character pin
  (PIN-03) in Korean or Chinese estimates above PIN-08's whole pin
  budget, so it could never restore. OQ-56 asks M2 to bound pins in
  estimated model tokens too.

## Method

### Claude's counts

Each count is one headless call of Claude Code 2.1.296 in an isolated
home ([measure.py](calibrate/measure.py)): no tools, no settings, no MCP
servers, a one-line system prompt and a user message of `<t>\n`, the
text, `\n</t>`, sent as stream-json input. The count is the call's input
usage (input, cache read and cache creation tokens) minus the same
call's usage with an empty text: the text's tokens and the two
newlines that join it to its delimiters, as ParleyInn joins text into a
block. The empty-text baseline was measured before and after each run
and never moved.

Two things the collector guards against:

- Claude Code expands an `@` mention of a path that exists, even in
  stream-json input: `see @/etc/hostname` cost 50 tokens more than a
  mention of a missing path. The calls run in an empty working
  directory and home, and the collector drops any text with a mention
  that resolves there (one draw).
- Prompts that switch thinking modes. The collector drops texts naming
  them; none did.

`the ` repeated 500 times counts 502: one model token a word and the two
joining newlines. The text is counted once.

### The sample

[collect.py](calibrate/collect.py) draws 646 texts with a fixed seed from
commit `99e06e5`. Windows start at a line start, with a length drawn
log-uniformly between 8 bytes and 16 KiB, which covers pins, landmark
lines, previews and a recall tool's whole output.

| Stratum      | Samples | Source                                                                                                                        |
| ------------ | ------- | ----------------------------------------------------------------------------------------------------------------------------- |
| prose        | 110     | the repository's Markdown                                                                                                     |
| code         | 110     | its Rust, TOML, YAML, shell, Python, Go and Gherkin                                                                           |
| transcript   | 110     | the recorded Claude Code transcripts and hook inputs under testdata                                                           |
| tool-output  | 100     | `git log --stat`, `git log --format=fuller`, `git show`, `git ls-tree -r -l`, `git grep -n`, `git blame` and `ls -laR` output |
| high-entropy | 80      | generated base64, hex, UUIDs, `sha256sum` lines, `xxd` dumps and JWT-shaped tokens                                            |
| pin-en       | 55      | English pins written for the spike ([authored.txt](calibrate/authored.txt))                                                   |
| multilingual | 81      | 57 pins in 24 languages and scripts with emoji, symbols and rare CJK, written for the spike, and 24 joins of 2 to 8 of them   |

The sample, with each text and its counts, is
[sample.jsonl](sample.jsonl): 771 KB of text, 371,000 model tokens.

The sample, the authored pins ([authored.txt](calibrate/authored.txt)) and
the collector ([collect.py](calibrate/collect.py)) keep the product's
earlier name, Cairn, on purpose. The texts are verbatim from commit
`99e06e5`, and each count is the model's count for that exact text, so
renaming a text would leave a count that belongs to another one. The
collector reads that commit's files by their paths then, such as
`crates/cairn`. ParleyInn is the same product under its new name.

### One tokenizer

Every sample was counted on Haiku 5.5. Every 16th, 41 samples, was
also counted on Sonnet 5.5, Opus 5.5 and Fable 5.1, and 10 more
high-entropy samples on Fable 5.1. Sonnet and Opus gave Haiku's count
for every sample. Fable 5.1 gave it for 49 of 51, and for two, a base64
block and a JWT-shaped one, it gave 8,885 and 12,029 against 3,754 and
5,326, reproducibly, while Sonnet and Opus matched Haiku on both. Larger
samples of the same two kinds matched exactly. No tokenization of a
text gives more tokens than its bytes plus the two joining newlines,
and both counts exceed that bound twice over. So the extra tokens are
something Fable's API adds on that content, not the text's tokens. The
fit uses the Haiku count, which every sample has; the verifier reports
every model.

## The estimator

[estimate.rs](bench/src/estimate.rs) is the reference implementation,
with its unit tests. Weights are in 1/1024 of a model token:

| Feature                                                      | Weight | Tokens |
| ------------------------------------------------------------ | ------ | ------ |
| ASCII letter                                                 | 561    | 0.55   |
| ASCII digit                                                  | 965    | 0.94   |
| ASCII space                                                  | 110    | 0.11   |
| other ASCII whitespace and control                           | 213    | 0.21   |
| ASCII punctuation and symbols                                | 1143   | 1.12   |
| U+0080–U+02FF, Latin with diacritics                         | 2160   | 2.11   |
| U+0300–U+07FF: Greek, Cyrillic, Hebrew, Arabic               | 850    | 0.83   |
| other three-byte: Indic, Thai                                | 967    | 0.94   |
| U+2000–U+2BFF: punctuation, symbols, arrows, math, box lines | 4462   | 4.36   |
| CJK, kana, Hangul, fullwidth forms                           | 1577   | 1.54   |
| four-byte: emoji, mathematical alphanumerics, rare CJK       | 4166   | 4.07   |
| switch between lowercase, uppercase and digit                | 861    | 0.84   |
| added to every estimate                                      | 6      |        |

[fit.py](calibrate/fit.py) fits them by linear programming as a quantile
regression on estimate / count at τ = 0.995, so a short text weighs as
much as a long one. Each character class has a floor (a digit at least
a third of a model token, any non-ASCII character half of one, a
four-byte one a whole one), so a class the sample holds little of is never
free. The fitted weights are raised by 10% and rounded up.

The margin and the switch feature come from cross validation. Weights
by character class alone reached at best 98.3% held out and overcounted prose
2×: one letter weight had to cover random base64, where letters cost
three times what they cost in words. Counting switches gives random
text its own cost. The 10% margin lifts every stratum to at least 99%
on the samples each fold did not see. A variant counting runs of
letters and digits fitted tighter, 1.54× against 1.71× before the
switch feature, but it learned a cheap cost per sequence of digits that
undercounts long numbers. It was set aside.

Estimate against Claude's count, from `s8bench verify`
([verify.out](verify.out) holds every table):

| Stratum      | Samples | Estimate ≥ count | Held out | Estimate / count | Lowest ratio |
| ------------ | ------- | ---------------- | -------- | ---------------- | ------------ |
| code         | 110     | 100%             | 100%     | 1.36             | 1.09         |
| high-entropy | 80      | 100%             | 100%     | 1.36             | 1.07         |
| multilingual | 81      | 100%             | 100%     | 1.34             | 1.10         |
| pin-en       | 55      | 100%             | 100%     | 1.63             | 1.26         |
| prose        | 110     | 100%             | 100%     | 1.49             | 1.10         |
| tool-output  | 100     | 100%             | 99.0%    | 1.36             | 1.08         |
| transcript   | 110     | 100%             | 100%     | 1.50             | 1.10         |
| **all**      | 646     | **100%**         | 99.85%   | 1.40             | 1.07         |

Simpler rules on the same sample:

| Rule             | Estimate ≥ count | Estimate / count | Lowest ratio |
| ---------------- | ---------------- | ---------------- | ------------ |
| bytes / 4        | 2.6%             | 0.52             | 0.24         |
| bytes / 3        | 12.5%            | 0.69             | 0.32         |
| bytes / 2        | 61.6%            | 1.04             | 0.48         |
| bytes            | 99.5%            | 2.08             | 0.95         |
| the S8 estimator | 100%             | 1.40             | 1.07         |

On Sonnet 5.5 and Opus 5.5 the estimator covers all 41 samples, and on
Fable 5.1 49 of 51: the two counts above.

### What the budgets and limits hold

| Budget                                      | Default | Holds                                                                    |
| ------------------------------------------- | ------- | ------------------------------------------------------------------------ |
| PIN-08 pin budget                           | 1,000   | about 12 of the spike's English pins (mean 132 characters, estimate 78)  |
| INJ-07 restore block limit                  | 2,000   | about 1,400 true model tokens                                            |
| RCL-03 recall output, CMP-06 printed output | 8,000   | 16 KB of prose, 14 KB of code, 13 KB of tool output, 10 KB of transcript |

A pin at PIN-03's 1,000-character limit estimates at about 510 in
English, 1,190 in Korean, 1,550 in Chinese and 4,075 in emoji. The last
three exceed the whole pin budget: PIN-08 truncates no pin, so such a
pin is named omitted at every restore and never restores. OQ-56 carries
this to M2.

## Baselines

### NFR-01: hook handlers on a 1M-event store

[s8bench](bench/src/main.rs) runs two hook handler shapes as processes,
hook input from the S1 fixtures on stdin, against S2's store of 1M
events built by `s2bench load`:

- `UserPromptSubmit`: open, record the prompt and ingest five
  transcript lines in one `BEGIN IMMEDIATE` transaction, close.
- `SessionStart`: open, record the start, read the room's pins and the
  run's newest 200 events, build a restore block within 2,000 estimated
  model tokens, and print it.

Each ran 1,000 times, timed from spawn to exit, inside a cgroup pinning
two CPUs and capping memory at 4 GiB ([ref.sh](bench/ref.sh)), as S2
emulated the reference hardware. Under load, `s8bench fleet` ran
alongside: ten appending processes, each adding five events every 200 ms (247
events per second in all), as ten harnesses' MCP servers ingesting,
and a reader of the newest 100 events every 250 ms, as the room view.

| Hook               | Target | Cold cache, idle p95 | Under write load p50 / p95 / p99 | Peak RSS |
| ------------------ | ------ | -------------------- | -------------------------------- | -------- |
| `UserPromptSubmit` | 50 ms  | 9.7 ms               | 1.7 / 3.2 / 24.8 ms              | 10.7 MiB |
| `SessionStart`     | 150 ms | 7.3 ms               | 1.8 / 4.2 / 38.4 ms              | 10.8 MiB |

The idle runs began right after the page cache was dropped, which is
why they are slower. The worst single run under load took 118 ms. The
harness's own process start comes on top. The shapes leave out what M1
and M2 add: the commitment, the seat key's signature and REC-04's
attribution. The margin is wide enough to freeze the target.

`PermissionRequest`'s targets wait for S9, which records how the held
request behaves.

### NFR-03, NFR-04, NFR-05 and NFR-09 from S2

S2 measured on the same reference hardware: bounded search 172 ms p95
at 10M events and expand 0.87 ms (NFR-03), ingest 26,143 events per
second on one connection at 100 per transaction (NFR-04), and a hook
process peak of 12.5 MiB (NFR-09). NFR-05 carries no †; S2's 10M-event
store and its 50-writer WAL check stand as its evidence.

NFR-09's store overhead stays open: S2 measured 1.96× stored text, the
`events` table alone 1.49×. M1 adds REC-17's commitment and 256-bit
secret per event, about 64 bytes, and moves content above 8 KiB to
payload files; its schema decides the number.

### Input limits on recorded transcripts

[limits.py](calibrate/limits.py) prints aggregates over the transcripts
and hook inputs under testdata (S1, S3, S4 and S13's recordings) and
over the transcript of the harness session that ran this spike:

| Limit                                   | Seen                                                                                                       |
| --------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| SEC-05 hook stdin ≤ 1 MiB               | largest of 153 hook inputs 47 KB                                                                           |
| SEC-05 JSON nesting depth ≤ 64          | 6 in hook inputs, 15 in transcript lines                                                                   |
| REC-09 payload threshold 8 KiB          | 3–7% of content blocks are above it and hold 33–91% of the content bytes; the median block is 75–251 bytes |
| REC-11 indexing cap 1 MiB               | largest content block 49 KB                                                                                |
| LMK-03 structural field ≤ 80 characters | tool names up to 37, MCP server names up to 18, file paths up to 85                                        |
| SEC-04 terms of at most 64 characters   | a SHA-256 digest in hex, the longest identifier the sample holds, is 64                                    |

## Every † target

The stakeholder's rule: a target freezes as late as it can, when the
spike or milestone that first uses or measures it is built. So S8 froze
only what it or S2 measured and M1 needs from its start, values
unchanged:

| Target                                              | Evidence                                               |
| --------------------------------------------------- | ------------------------------------------------------ |
| REC-09 payload threshold 8 KiB, preview ≤ 512 bytes | input limits above                                     |
| SEC-04 query deadline 2 s                           | S2: bounded search p99 548 ms, interrupt within 1.2 ms |
| SEC-05 hook stdin 1 MiB, JSON depth 64              | input limits above                                     |
| NFR-01 store of 1M events                           | NFR-01 above                                           |
| NFR-03 at 10M events                                | S2                                                     |
| NFR-04 on one core                                  | S2                                                     |
| NFR-09 hook peak RSS 50 MiB                         | S2 and NFR-01 above                                    |

These are 9 numbers that lost their †. The policy choices S8 has
evidence for but no measurement of stay initial, the evidence above
handed to the milestone that first uses them: REC-17's 256 bits,
RCL-02's 50 hits, RCL-03's 8,000-model-token cap and SEC-04's 16 terms
of 64 characters to M1; PIN-08's pin budget and INJ-07's restore block
limit to M2, with OQ-56; CMP-06's output cap to M4.

The remaining 44 are listed in §12.3 with what freezes them: six with
S9, seven with M1, three with M2, three with M3, three with M4, three
with M5, and nineteen with M7, M8 and M9, which build seals, the room
view, the launcher, the peer network and the publish and bridge
components.

## Handed on

- M1 ports [estimate.rs](bench/src/estimate.rs) into the core with its
  unit tests, moves the committed sample out of `plan/` into
  `testdata/`, and proves §8.4's bound on it in a test, so the
  normative estimator does not live in a plan folder (M1's task 9). A
  new tokenizer is a new calibration: rerun
  [measure.py](calibrate/measure.py) and [fit.py](calibrate/fit.py) and
  check the bound.
- M2 answers OQ-56 before PIN-03 ships.
- M1 measures the store overhead on its schema with payload files and
  sets NFR-09's number; it also times indexing one event at REC-11's
  cap.
- M3 decides LMK-03's field length: an absolute path in a real
  checkout already passes 80 characters.
- Fable 5.1's extra input tokens on some base64 content are not text
  tokens and no estimator of the text bounds them. Whether they count
  against the context the restore block shares is a question for the
  model, not for ParleyInn.

No requirement closes and no scenario leaves `@pending`. The spike
gathers evidence for §8.4, NFR-01, NFR-03, NFR-04 and NFR-09.

## Reproducing

```sh
cd plan/2609292010_spike-s8-token-estimator
# the sample: needs a checkout at the commit and an empty scratch home
S8_HOME=$(mktemp -d) S8_WORK=$S8_HOME/work \
  python3 -I calibrate/collect.py ../.. 99e06e596a084ca33b7b1294025faa89850c156e sample.jsonl
S8_HOME=$(mktemp -d) python3 -I calibrate/measure.py sample.jsonl claude-haiku-5-5
S8_HOME=$(mktemp -d) python3 -I calibrate/measure.py sample.jsonl claude-sonnet-5-5 16
python3 calibrate/fit.py sample.jsonl          # needs numpy and scipy
cd bench && cargo build --release && cargo test --release
B=$PWD/target/release/s8bench
$B verify ../sample.jsonl
# the hook baselines, on S2's store
../../2609292004_spike-s2-sqlite-driver/bench/target/release/s2bench load -db /tmp/s8.db -n 1000000
$B pins -db /tmp/s8.db
F=../../../testdata/claude-code/2.1.296/hooks/startup
./ref.sh $B lat -fixture $F/02-UserPromptSubmit.json -n 1000 -- hook-ups -db /tmp/s8.db
./ref.sh $B lat -fixture $F/01-SessionStart.json -n 1000 -- hook-start -db /tmp/s8.db
./ref.sh $B fleet -db /tmp/s8.db -writers 10 -secs 150 &   # then the two lat runs again
python3 -I ../calibrate/limits.py ../../../testdata/claude-code
```

The measure runs sign in as the person running them and use their
model quota: the whole sample cost about 1.2M input tokens on Haiku,
most of them the call's own fixed overhead.
