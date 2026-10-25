# Amp orbs: agents on remote machines, agent to agent, multiplayer

Scope: Amp's orbs, agent-to-agent threads and multiplayer, read from
Amp's documentation and notes on 3 October 2026. Amp is the coding
harness of Amp Frontier Corporation, formerly Sourcegraph, for the
terminal, the web and mobile apps. The documentation leaves hosting,
network policy and the trust model unstated, so those are gaps, not
findings.

## Orbs

- An orb is a remote machine Amp creates for a thread: "Every orb
  thread gets a fresh, isolated environment with your code, plugins,
  development tools, and the context of the thread that created it" —
  [orbs manual](https://ampcode.com/manual/orbs).
- The agent keeps working with the laptop closed. When it finishes,
  the orb sleeps, costs nothing while asleep, and wakes with the
  conversation, files and services in place. Up to 32 GB of memory and
  16 CPUs; a burst of 20 metered orbs, then one more every five minutes
  — [orbs](https://ampcode.com/docs/orbs).
- The user watches from ampcode.com, the macOS and iOS apps, or a
  shared tmux session, and reviews the diff in a Changes view. Results
  are "the conversation, the code changes, the running services, and
  the proof".
- Environments are snapshotted after setup and reused for up to 24
  hours; a resume script runs on every wake — [putting an agent in an
  orb](https://ampcode.com/notes/putting-an-agent-in-an-orb).
- Hosting, network policy and the secrets mechanism are not in the
  overview pages; orbs appear to run only on Amp's service.

## Agent to agent

- Agents spawn other agents: in an orb, on the local machine, in
  another project, or on any machine running an Amp runner. They send
  each other instructions and files and bring results back into the
  parent thread — [agent to agent](https://ampcode.com/docs/orbs/agent-to-agent),
  [announcement](https://ampcode.com/news/from-agent-to-agent).
- Each thread keeps its own conversation, context window, working copy
  and orb; files move only by explicit transfer.
- Messages between agents read as task instructions. No trust model,
  permission rule or untrusted marking is documented for them.

## Multiplayer

- Any orb thread can be turned multiplayer from its menu on the web:
  anyone in the workspace can join, message the agent, and use the
  orb's portal, file changes and shared terminal until multiplayer
  mode expires — [agent to agent](https://ampcode.com/docs/orbs/agent-to-agent).
- Threads are stored on Amp's servers, and threads in workspace-owned
  projects are shared by default — [thread sharing](https://ampcode.com/docs/collaborate/thread-sharing).
- Nothing documented separates a teammate's message from the owner's:
  in multiplayer, anyone in the workspace instructs the agent directly.

## Against ParleyInn

| Question         | Amp orbs                                                       | ParleyInn (2.0-draft)                                                 |
| ---------------- | -------------------------------------------------------------- | --------------------------------------------------------------------- |
| Where agents run | Remote machines on Amp's service, or a local Amp runner        | The user's own machines and sandboxes                                 |
| Shared channel   | One thread per agent; agents message each other across threads | One lane holds many agents and people                                 |
| Multiplayer      | Workspace members join a thread and message the agent directly | Co-authors drive their own agents; posts reach yours by endorsement   |
| Trust            | Messages from teammates and agents read as instructions        | Everything not yours is untrusted; endorsement is a recorded act (I2) |
| Topology         | Central: Amp's servers hold threads and run orbs               | No central service; peers enrolled by key                             |
| Record           | Server-side threads                                            | One signed log per writer, on your nodes                              |

## Lessons for ParleyInn

- Orbs are the product shape of ParleyInn's ephemeral-sandbox persona (U3):
  agents keep working while the laptop sleeps, and wake with state in
  place. ParleyInn's answer is the sealed segments a sandbox offers to an
  enrolled peer (PEER-05), with no vendor holding the machine.
- Amp has the multiplayer ParleyInn plans, centrally and without a trust
  boundary: a teammate in the thread instructs the agent as the owner
  does. ParleyInn's one-principal rule and endorsement (OWN-01, OWN-08) are
  the difference to name in the pitch.
- Agent-to-agent messaging is a channel ParleyInn has not specified: an
  agent's message to another agent is assistant text, untrusted under
  I2, and today reaches another agent only as a post. A requirement for
  agent-to-agent delegation inside a lane is a candidate for the next
  SRS revision.
