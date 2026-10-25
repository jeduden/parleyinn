---
title: "9.7 Room vocabulary"
summary: >-
  The one vocabulary every surface uses (VIEW-14): run status and
  freshness, room status, evidence and proof classes, the Needs you
  order, integrity status, trust marks and the keymap. Part of §9.
---
# 9.7 Room vocabulary

One vocabulary for every surface (VIEW-14): status words, marks, queue
order and keys. The reasons behind each choice are in plan 2610012322's
proposal, §8.

## 9.7.1 Run status and freshness

| Status       | Meaning                                                | Detail                                      |
| ------------ | ------------------------------------------------------ | ------------------------------------------- |
| **Starting** | launched, no first event yet                           | —                                           |
| **Working**  | a turn is running                                      | —                                           |
| **Asking**   | waiting on its principal                               | `permission`, `question`, `hand-off`        |
| **Idle**     | turn ended, nothing pending                            | `says done` when the agent says so (a mark) |
| **Paused**   | paused by its principal at a safe point, acknowledged  | —                                           |
| **Blocked**  | cannot proceed without something other than a decision | `rate limit`, `error`                       |
| **Ended**    | run ended, acknowledged                                | `by agent`, `by principal`, `crashed`       |

| Freshness mark | Meaning                                                                                                       |
| -------------- | ------------------------------------------------------------------------------------------------------------- |
| (none)         | live: events current on this node                                                                             |
| `behind`       | on a writer whose log has not reached this node lately: "as of 14:02 on desktop"                              |
| `unrecorded`   | the run is live but ingest has not kept up with its transcript since a time                                   |
| `ingested`     | an ingested run: from a transcript no MCP server ingested as it grew (REC-22)                                 |
| `stuck?`       | a watchdog observation (OWN-09), computed where shown, never recorded: Working with no event past a threshold |

## 9.7.2 Room status

Room status is computed where shown from room state, the heads, landings and
check states of the branches it names, its runs' statuses and their freshness
marks, never recorded: no act sets it directly, though OWN-21's ready and
abandoned marks feed it (LANE-09). Rooms do not land; branches do,
merged by git or the forge into the repository's default branch or a branch the
forge protects (LANE-06, LANE-08). First match wins, in this order:
**Abandoned** (the owner marked the room abandoned) and **Landed** (the room
names at least one branch, and every one has landed), **Asking** (a run waits on
its principal), **Failing** (a check failed on the head of a branch the room
names, evidence `own check` or stronger), **Blocked**, **Running**, **Ready for
review** (only after the principal act of OWN-21 marking the room ready),
**Quiet** (never while a run is `unrecorded` or `behind`, VIEW-04). The
pull-request states a forge reports (Pre-review, In review, Changes requested,
Approved, Queued, Merging) live on the pull-request pill (P2, LANE-08), marked
`asserted`, not in the room status; ParleyInn keeps no approval state of its
own. A
check's check state, computed where shown, is one of Pending, Executing, Passed,
Failed, Stale and Absent; a result resting on edits not fully recorded is marked
`unbound` (LANE-05), a mark like `from checkpoint` and `outside intent`, not a
check state. No check state shares a name with a room status, a draft pull
request or a landing.

## 9.7.3 Evidence and proof classes

| Evidence class  | Glyph | Rank (LANE-05)    |
| --------------- | ----- | ----------------- |
| `claim`         | `○`   | lowest            |
| `own check`     | `◐`   | above `claim`     |
| `witness check` | `◑`   | above `own check` |
| `CI attested`   | `●`   | highest           |

| Proof class           | Counts as proven |
| --------------------- | ---------------- |
| `same commit`         | yes              |
| `same patch`          | yes              |
| `same tree`           | yes              |
| `likely`              | no               |
| `asserted`            | no               |
| `not proven` + reason | no               |

## 9.7.4 Needs you order

Queue classes, named Q1–Q4 so they never read as priorities P0–P2: **Q1 blocking
now** (held requests: permission, question, hand-off), **Q2 blocking the room**
(failed check the forge requires, crash, rate limit with no resume, refused
segment, overlap), **Q3 waiting on you** (a pull-request review the forge asks
of you, an outcome awaiting a verdict, a role request, a directed post awaiting
your endorsement (LANE-12), a pin version you stamped that was edited, unpinned
or taken off the pin list (LANE-32), your device-seat pin a list removal took
off the pin list (LANE-26), a held request past its hold window under a
keep-going away policy (OWN-07), a quota crossed), **Q4 for your record** (never
alerts). Dismissing a directed post is a cut principal act, dismissing any other
Q3 or Q4 item a neutral one, and each room's owner acknowledges an overlap for
its room (LANE-13). Order: class; inside Q1, the number of agents blocked on the
same answer, then causal order of raising; inside Q2 and Q3, rooms in the focus
set first, then causal order. The focus set changes only by a recorded neutral
principal act, so every device of the principal shows one order. Causal order,
not wall-clock age, because age differs between nodes and reads a clock (I10);
it is oldest first wherever clocks agree. Pull-request reviews the forge asks of
the principal enter the same queue.

## 9.7.5 Integrity status

The integrity status of a room or a writer, shown on every surface and
in the envelope's `integrity` field (§9.3), is one of seven values
(VIEW-10): `verified` ◆ (chains and seals check), `unsigned`
(hash-chained, before REC-18 or past the newest seal), `incomplete` ◇ (a
writer's events are missing here), `unverified` (an event after a
break), `broken` ✕ (a chain or seal verification failed), `refused` (a
segment was refused, which an audit entry records, and a structural
event in this node's personal room once per writer and reason, REC-21),
`equivocated` (PEER-10). The UI never
says "secure".

## 9.7.6 Trust marks

No mark, for the principal's own agents, on this node's own `operator` and
structural events (hook observations only as PRV-08 allows), the `harness_meta`
events its launcher recorded, and, while the deployment mode is `interactive`
and no run on the node leaves R3 or R8 open, the `user` events matched to a
prompt (PRV-08) (an ingested one is untrusted, REC-22, and a line the launcher
carried in is untrusted `harness_text`, never a `user` event, LANE-14); on
principal acts a device key the principal certified signed, and posts and pins
written from a device seat such a key certified, within that key's scope, from
any of its nodes (shown with a device glyph); on pin versions the principal
stamped (shown with the stamper, LANE-32); and on posts and pins a trust grant
of the principal covers outside a foreign room, written from device seats a
device key of the principal the trust grant names certified, never a
token-key-only node's (shown with the granted key's petname, OWN-29). In a
foreign room only the principal's own device-seat pins and the versions it
stamped go unmarked, as in a restore block (PIN-10); in recall every item from a
foreign room is marked untrusted (RCL-10). The free text of a cut or neutral
principal act, such as a stated reason, shows `○` whatever its event's trust
level, on this node's own `operator` events too (OWN-11). `○` plus petname on
anything untrusted from another principal, agent, node or bundle, a `user` event
another node recorded included; a key with no petname shows its fingerprint. `⚑`
flagged (PRV-07). `▒` quarantined. `▬` from a seat that is no longer a member:
it left, was kicked or was barred. `ingested`. `new key`. "from a revoked
device". Timeline rails: solid for the agent's principal, hollow for other
principals, dotted for agents and the forge. Events from a token-key-only node's
writers are untrusted on every other node, and its pins restore only once a
principal stamps a version (PRV-02). Unsandboxed agents are one line on Health,
not a mark on every tile.

## 9.7.7 Keymap

One map, no key bound to two actions. Typing in a composer or text
field sends text, not shortcuts.

| Key                   | Action                                                          | Key             | Action                                                                   |
| --------------------- | --------------------------------------------------------------- | --------------- | ------------------------------------------------------------------------ |
| `?`                   | all shortcuts                                                   | `a` / `A`       | allow once / `allow-session`                                             |
| `/`                   | search and filter                                               | `d`             | deny                                                                     |
| `⌘K` / `Ctrl+K`       | command palette                                                 | `r`             | reply                                                                    |
| `⌘G` / `Ctrl+G`       | go to address                                                   | `e` / `E`       | endorse / edit, then endorse                                             |
| `g i`                 | Needs you                                                       | `Home` / `End`  | replay start / end                                                       |
| `g f`                 | Fleet                                                           | `x`             | dismiss a Q3 or Q4 item (a directed post: a cut); acknowledge an overlap |
| `g c`                 | Catch up                                                        | `y`             | copy address                                                             |
| `g q` / `g v`         | quarantine list / verify panel                                  | `o`             | open the Room page at this item                                          |
| `g h` / `g p`         | Health / Peers                                                  | `n` / `N`       | next / previous item that needs you                                      |
| `1` / `2` / `3`       | Room page tabs: Timeline / Review / Replay                      | `m`             | steer an agent; the principal-typed steer names this address (OWN-03)    |
| `j` / `k`             | next / previous row, item or file                               | `q`             | quarantine selection                                                     |
| `Enter`               | open                                                            | `⌘S` / `Ctrl+S` | save search                                                              |
| `Space`               | preview or peek                                                 | `+` / `-`       | more / less context around a hit                                         |
| `Esc`                 | close sheet                                                     | `w` / `b`       | why panel / blame gutter                                                 |
| `I`                   | interrupt (run pane)                                            | `W`             | witness check                                                            |
| `p`                   | pause / resume                                                  | `c` / `C`       | post on the selection / verdict sheet                                    |
| `S`                   | stop (opens the stop sheet)                                     | `P`             | pick what the outcome window shows                                       |
| `F`                   | follow a seat                                                   | `l`             | branch, pull-request and issue links                                     |
| `t`                   | terminal takeover, only at the launcher's own terminal (OWN-19) | `V` / `s`       | compare versions / since my verdict                                      |
| `h`                   | hand back                                                       | `(` / `)`       | previous / next post with a range link                                   |
| `Q`                   | turn an away policy on / off                                    | `<` / `>`       | down / up the stack                                                      |
| `L`                   | replay jump to live                                             | `Ctrl+T`        | add the room to or remove it from the focus set                          |
| `Shift+Space`         | replay play / pause                                             | `←` / `→`       | replay previous / next event                                             |
| `Shift+←` / `Shift+→` | replay previous / next span                                     | `,` / `.`       | replay previous / next turn                                              |
| `[` / `]`             | replay previous / next edit                                     | `{` / `}`       | replay previous / next error                                             |
| `;` / `:`             | replay next / previous held request                             | `v`             | replay context lens                                                      |

Interrupt moved from `Esc Esc` to `I`, so no key repeats into a second
act. Composer: `Enter` steer, `Ctrl+Enter` queue the steer for the next
turn, `Shift+Enter` interrupt, then steer. Clashes resolved: `x`
(dismiss, expand, mark seen) is dismiss, or acknowledge on an overlap,
with context on `+`/`-`; `e` (endorse, next error) is endorse, errors
move to `{`/`}`; `]`/`[` (next room, next edit, stack) are edits, rooms
move to `n`/`N`, the stack to `<`/`>`; `a` (allow, next held request) is
allow, replay held requests move to `;`/`:`; `r` (reply, verdict,
replay) is reply, the verdict sheet moves to `C`, replay to tab `3`; `c`
(copy, post on the selection) is post, copy moves to `y`; `p` (pause,
previous post with a range link, steer with an address) is pause,
steering with an address moves to `m`; `.` (next turn, follow) is next
turn, follow moves to `F`; `Space` (select, peek, play) is peek, play
moves to `Shift+Space`; `Shift+A` is bound to nothing, its
earlier action removed.
