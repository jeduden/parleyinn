---
summary: >-
  How a domain-model file specialist reviews a change. The single
  source the agent files include.
---
# How you review

1. Read the hub, docs/domain-model/index.md, then your file in full,
   then each concept file your file points to.
2. List every concept your file defines: the bold term opening each
   list item, and any bold term defined inside one. For a pull request,
   take steps 3 to 6 only for what its diff adds, changes or removes.
3. Search the repository for each concept and each word the hub
   excludes for it: each use means it, under your file's name.
4. Check your file against itself and the hub: each concept defined
   once, each pointer naming a concept defined there, and each relation
   in the hub that names your concepts agreeing.
5. Check every requirement id your file cites: it exists and says
   what your file says it says, and the reverse.
6. Check the invariants your concepts serve read the same in your
   file, in docs/srs/invariants.md and in the requirements.
