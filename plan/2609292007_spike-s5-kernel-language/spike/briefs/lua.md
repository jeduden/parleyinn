# Brief: answer questions about a record in Lua

You are writing code for a compute kernel. Each solution you write is
executed once, without you seeing the output, and its answer is
compared with a reference. Write each solution so that it is right the
first time.

## The record

The record is a list of events from six agent runs. Each run has its
own writer (`A1` to `A6`); a seventh writer, `D1`, holds the
operator's pin events. Every event has the same twelve fields. A field
that does not apply to an event is absent.

| Field        | Type    | Meaning                                                                                                        |
| ------------ | ------- | -------------------------------------------------------------------------------------------------------------- |
| `writer`     | string  | The writer the event belongs to: `A1` to `A6`, `D1`                                                            |
| `seq`        | integer | The event's position in its writer: 1, 2, 3, … with no gaps                                                    |
| `run`        | string  | The run: `r1` to `r6` (writer `A1` holds run `r1`, and so on); absent on `D1`'s events                         |
| `ts`         | integer | When the event happened, in Unix seconds, UTC; never decreases within a writer                                 |
| `provenance` | string  | What produced the content: `user`, `assistant`, `tool_result:<tool>`, `harness_meta`, `structural`, `operator` |
| `kind`       | string  | `message`, `tool_call`, `tool_result`, `compaction`, `recall` or `pin`                                         |
| `tool`       | string  | The tool's name, on tool calls and tool results: `Bash`, `Read`, `Edit`, `Write`, `Grep`, `Glob`, `WebFetch`   |
| `call_id`    | string  | On a tool call and on its tool result, the same id; links the two                                              |
| `status`     | string  | On tool results only: `ok` or `error`                                                                          |
| `tokens`     | integer | The event's size in model tokens; present on every event                                                       |
| `text`       | string  | The event's text; present on every event                                                                       |
| `args`       | string  | JSON text, on four kinds of event, described below                                                             |

The `args` field holds:

- on a tool call, the tool's input: `{"command": …}` for `Bash`,
  `{"file_path": …}` for `Read`, `Edit` and `Write`, `{"pattern": …,
  "path": …}` for `Grep`, `{"pattern": …}` for `Glob`, `{"url": …}`
  for `WebFetch`
- on a `compaction` event: `{"pre_tokens": <integer>, "trigger":
  "auto" | "manual"}`
- on a `recall` event: `{"query": <string>, "tool": "event_search"}`
- on a `pin` event: `{"action": "add" | "remove", "pin_id": <string>}`

Terms the questions use:

- A **tool call** is an event of kind `tool_call`; a **tool result**
  is an event of kind `tool_result`. A tool result answers the tool
  call with the same `call_id`. A tool call may have no result.
- A **user message** is an event of kind `message` with provenance
  `user`; an **assistant message** has kind `message` and provenance
  `assistant`.
- A run's events are the events whose `run` field names it.

## The language

Lua 5.4, in a sandbox. These globals exist:

- the base functions `assert`, `error`, `ipairs`, `next`, `pairs`,
  `pcall`, `xpcall`, `select`, `tonumber`, `tostring`, `type`,
  `rawequal`, `rawget`, `rawlen`, `rawset`, `setmetatable`,
  `getmetatable`
- the libraries `string`, `table`, `utf8`, and `math` without
  `math.random` and `math.randomseed`
- `parinn.events()`: returns a new array of every event, each a table
  with the fields above, in writer then seq order
- `json.decode(text)` and `json.encode(value)`; a JSON null decodes to
  nil
- `re.test(s, pattern)`: true if the regular expression matches
  anywhere in `s`
- `re.capture(s, pattern, n)`: the text of capture group `n` (default
  1) of the first match, or nil
- `re.find_all(s, pattern)`: an array of the text of every match

`re` patterns are regular expressions in Rust `regex` syntax (as in
Python's `re`: `\d`, `(…)`, `[^.]`, `$`), not Lua patterns. Lua's own
`string.find`, `string.match` and `string.gmatch` with Lua patterns
work too.

Not available: `io`, `os`, `require`, `load`, `loadfile`, `dofile`,
`print`, `debug`.

## The answer

The chunk's `return` value is the answer. Each question names a shape:

- **scalar**: return one number or one string
- **list**: return an array of values, in the order the question asks
- **rows**: return an array of rows, each row an array of its column
  values in the order the question names the columns

Integers may come back as integers or as floats with the same value;
floats are compared with a small tolerance. Do not round unless the
question says so.

## Your task

For each task you are given, write one file holding only Lua code, at
the path you are given. Do not run any code, do not open any other file
in the repository, and do not look at the record's data: work from
this brief alone.
