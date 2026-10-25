---
id: ADR-2610152132
title: "The settings layers' TOML: the toml crate, parsing only"
status: accepted
scope: dependencies
summary: >-
  Every settings layer, managed policy, the principal's configuration and
  a repository's .parinn.toml, is read through the toml crate's parser
  into a table the configuration crate validates strictly itself. No
  serde derive, no writer, and the parser's nesting limit kept on.
---
# ADR-2610152132: The settings layers' TOML

## Context

ADM-04 requires every settings layer to be TOML, validated strictly:
an unknown key, a type error or a value out of range fails naming the
key and the problem. A repository's `.parinn.toml` comes from whoever
can commit to the repository (T5), so the parser reads hostile input.
Phase 3 of plan
[2609292012](../../plan/2609292012_m1-record-and-recall/plan.md) builds
the layers, and the stakeholder's ruling 1 of 14 October 2026 counts
`toml` among the core's dependencies.

## Decision

| Module | Purpose                                                                                         | License           | Maintenance                                                    |
| ------ | ----------------------------------------------------------------------------------------------- | ----------------- | -------------------------------------------------------------- |
| `toml` | Parses a settings layer's text into a table of values, TOML 1.1, with its default nesting limit | MIT OR Apache-2.0 | Active; toml-rs organisation, the de-facto TOML crate for Rust |

The core takes it without default features, with `std`, `parse` and
`serde`: `serde` holds the `Table` and `Value` types the parser
returns, and the core derives nothing through it. The `display`
feature, a TOML writer, stays off, as does `unbounded`, which would
lift the parser's nesting limit. The `configuration` crate walks the
table itself against its own list of keys, so validation never rests
on the parser's leniency.

## Alternatives

- **A TOML reader of ParleyInn's own.** Parser code the core must fuzz and
  keep in step with TOML 1.1, for one dependency saved.
- **`toml_edit`.** It keeps formatting and comments for writing back;
  ParleyInn never writes a settings layer.
- **`basic-toml`.** A smaller fork of an older parser, kept for
  compiler builds rather than for wide use.
- **Another format, such as JSON.** ADM-04 names TOML.

## Consequences

- The core's reach test allows nine crates: `toml`, `toml_parser`,
  `toml_datetime`, `serde_spanned` and `winnow`, which parse text in
  memory, and `toml_writer`, `indexmap`, `hashbrown` and `equivalent`,
  which `toml` links for its tables. None opens a socket or starts a
  program.
- The direct set is eight, against ENG-18's target of ten.
- The parser's fuzz target joins the first `fuzz/` crate, in phase 7.
