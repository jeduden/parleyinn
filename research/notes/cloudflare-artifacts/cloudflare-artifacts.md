# Cloudflare Artifacts: a git platform for agents, and its call for the next GitHub

Scope: Cloudflare's post "Building the next Git platform for AI
agents" of 1 October 2026 by Dina Kozlov and Zebulon Piasecki
([post](https://blog.cloudflare.com/next-git-platform-on-cloudflare/)),
and the Artifacts documentation
([docs](https://developers.cloudflare.com/artifacts/),
[how it works](https://developers.cloudflare.com/artifacts/concepts/how-artifacts-works/)),
read on 4 October 2026. The storage backend behind the git interface is
not stated in what was read, so it is a gap, not a finding.

## What it is

- **A versioned filesystem behind a git interface.** Artifacts lets
  developers "store and version files, code, and projects behind a
  Git-compatible interface", reached from Workers, a REST API or any
  git client, and is meant to "scale to millions of repositories".
- **Repositories as cheap, programmable units.**
  - Repos are created and forked from code.
  - A fork "creates a new repo that starts from an existing repo's
    history, then diverges independently with its own tokens, routing,
    and lifecycle".
  - Namespaces are the top-level container. A namespace's data stays
    in the United States or the European Union, chosen when it is
    created.
- **Scoped tokens.** Each repo has its own tokens, each limited to read
  (clone, fetch, pull, indexing, review) or write (push and other
  changes).
- **Events.** Artifacts publishes an event when a repo is created,
  imported, forked, deleted, pushed to, cloned or fetched. A push can
  start CI, a review workflow or a Workers deployment, with preview
  deployments for non-main branches.
- **Durability.** Repo data is replicated synchronously across data
  centres and copied asynchronously to object storage and snapshots.
- **Availability.** Open beta on the Workers Paid plan. Billing starts
  15 October 2026, priced by repository operations and data stored.

## What it says about agents

- It pictures "hundreds, or even thousands, of agents working on the
  same codebase at the same time". Each agent forks a project for a
  task, reads its instructions from files in the repository, pushes
  its changes, and a review runs on every push.
- Developers use it to "persist the code and context from agent
  sessions". The documentation names "session tracking with parallel
  exploration and merging" as a use case.
- The post asks "What does the next GitHub look like?" and calls for
  rethinking "repositories, branches, pull requests, worktrees, code
  review, and merge conflicts". A developer competition closes on
  14 October 2026.
- **Not described in what was read:**
  - signing or provenance of agent work;
  - any trust boundary between instructions in repository files and
    the agent reading them;
  - roles beyond read and write tokens;
  - how a person judges an outcome.

## What it means for ParleyInn

| Artifacts                                             | ParleyInn                                                                                   |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| A fork per agent task, diverging independently        | A workspace per agent, several serving one room: parallel attempts at one intent            |
| Code and agent context persisted together in repos    | The record keeps the context beside git, signed per writer, never as repository files       |
| Instructions read from files in the repository        | Repository files an agent reads are untrusted data; only its own person's rules instruct it |
| Events on push, fork, clone                           | Structural events in the record, and Needs you items for people                             |
| Read and write tokens per repo                        | Roles and capabilities per room, configured by the owner                                    |
| Central, Cloudflare-hosted, by namespace jurisdiction | No central service (CON-06); self-hosted peers; a person may name a host to publish to (B3) |
| Review triggered on every push                        | Agents present outcomes against the intent; people judge                                    |

- **It confirms the direction.** A large platform now treats many
  agents on one codebase, a fork per attempt, and persisted session
  context as the problem to solve, and asks openly what replaces the
  pull request. ParleyInn's answer is the room: intent, live work,
  judgement and correction in one place.
- **It sharpens the difference.**
  - Artifacts is a hosted store with git semantics and no stated trust
    model.
  - ParleyInn is a local record with a trust boundary (I2), signed writers
    and no central service.
  - Artifacts reads agent instructions from repository files, which is
    exactly the path by which a poisoned file becomes a command.
- **It could be a publish target.** Since Artifacts speaks git, a
  person could name an Artifacts repository as a B3 host for ParleyInn's
  git carrier or a published bundle. ParleyInn would depend on it for
  nothing.
- **It joins the comparison.** It sits beside Pierre Code Storage in
  the survey of agent version control
  ([agent VCS](../git-alternatives-for-agent-sessions/agent-vcs.md)):
  managed, git-compatible, built for many agents.
