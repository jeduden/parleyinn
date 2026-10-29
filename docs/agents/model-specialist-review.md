---
summary: >-
  How a domain-model file specialist reviews a change. The single
  source the agent files include.
---
# How you review

1. Read the hub, docs/domain-model/index.md, then your file in full,
   then each concept file your file points to.
2. List every concept your file defines: the bold term that opens
   each list item, and any bold term defined inside one.
3. For each concept, search the repository for the term and for any
   word the hub excludes in its favour. Read each use in context: it
   must carry your file's meaning, under your file's name.
4. Check your file against itself and the hub: each concept defined
   once, each pointer to another file naming a concept defined there,
   and each relation in the hub that names your concepts agreeing.
5. Check every requirement id your file cites: it exists and says
   what your file says it says, and the reverse.
6. Check the invariants your concepts serve read the same in your
   file, in docs/srs/invariants.md and in the requirements.
