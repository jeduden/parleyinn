# Annotation brief: feature spans over the domain model

Larger task: a genetic / simulated-annealing search for the largest subset of
ParleyInn's domain model that is consistent, secure and useful. Candidates are
assembled by removing the text of excluded features. Your job: mark that text.

Folder: this one, research/notes/domain-model-subset-search/
(called GA below). Read GA/features.json first: the core feature and 45
optional features, each with its entries ("stem: First bold term"),
requires, requires_any and soft edges.

For each file assigned to you, copy the repository original
(docs/domain-model/<file>, or docs/srs/invariants.md)
to GA/annotated/<file> and insert markers. Edit nothing in the repository.

## Markers

`⟦f:ID⟧` opens and `⟦/f⟧` closes a span of optional feature ID. Spans nest.
When a candidate leaves ID out, the span and everything inside it disappear.
Never mark core (it is always present).

1. **Whole items.** Wrap every list item (top-level or nested) that
   features.json assigns to an optional feature, from the `- ` of its first
   line to the end of its last line, including nested items under it. Put the
   opening marker right before `- ` (after the indentation) and the closing
   marker at the end of its last line.
2. **Clauses.** Inside every other item and paragraph (core items included),
   wrap each word, list element, clause or sentence that names a concept of an
   optional feature, or only makes sense with it, in that feature's span. A
   list element "a, b or c" where b belongs to F becomes "a⟦f:F⟧, b⟧/f⟧ or c" —
   that is: include the separator so removal leaves grammatical text. When
   the removed element is last, take the preceding separator ("a or b⟦f:F⟧ or
   c⟦/f⟧" is fine only if "a or b" remains correct; otherwise restructure the
   span boundaries, never the words). A sentence needing two features nests
   both spans. A parenthetical citing a requirement id only of F goes with F.
3. **Hub (index.md).** Relations items as features.json assigns them; inside
   core relations and every other section (verbs, Names follow the model, Not
   ParleyInn concepts table rows), mark clauses and table rows that mention
   optional concepts. A table row goes in one span from `|` to end of line.
   Keep the catalog block as is (no markers inside `<?catalog ... ?>`).
4. **Invariants.** Mark the clauses of I1–I10 that exist only for an optional
   feature (paired phones, peers and B2, B3 publishing and bridges, trust
   grants, delegation and acceptance grants, device-seat posts and pins of
   other nodes, stamps, notices, compaction guidance only if optional, …),
   following features.json. Keep each invariant grammatical when spans go.
5. **Never change any character** outside the markers. Removing every marker
   must reproduce the original byte for byte, line breaks included. Markers
   may sit mid-line and span lines.

## Self-check (required)

Run `python3 GA/check_annotation.py <your files...>` until it prints OK and no
`dangling` line for your files. A dangling line means: with feature F removed,
a term F defines is still mentioned in your file outside an F span. Fix it by
spanning that mention, unless the mention is legitimately another sense
(then say so in your report). Also assemble two spot genomes to read the
result: create GA/genomes/_test-min.json `{"name":"min","features":[]}` and
GA/genomes/_test-all.json with every feature id, run
`python3 GA/assemble.py GA/genomes/_test-min.json GA/out/_test-min-<yourname>`
and read your files there: the core-only text must read as a coherent model.

Report: files annotated, span counts per feature, every dangling mention you
left on purpose with the reason, and any item whose feature assignment in
features.json looks wrong (do not edit features.json).
