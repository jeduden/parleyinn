# ParleyInn pitch under review

ParleyInn is the live pull request for the agent era. Each lane of work is
a branch with its worktree, its agents and its humans, held as one
record. That record holds the conversation, every edit, every tool run
and every result, in the order they happened, so code and talk can
never drift apart.

A PR today is a frozen diff with comments bolted on. A ParleyInn lane is
live:

- Chat: people talk with the agents working on the branch, in the branch.
- Real time: everyone sees edits, test runs and messages as they happen.
- Multiplayer: several agents and people work one lane together, or
  watch a whole fleet of lanes at once.
- Interactive results: test results, benchmarks, diffs and rendered
  output arrive as live views you can explore, not pasted logs.

When the lane is done, the code lands in git as an ordinary commit, and
the whole story of how it was made stays one click away, verifiable and
searchable.

It is built so that collaboration can't become an attack on the agents.
A message from anyone except the lane's owner reaches an agent as
untrusted, and nothing stored is injected without being asked for. The
core that feeds the model never touches the network. The lane's record
is tamper-evident, and sensitive content can be erased by key.

One line: Git keeps the code; ParleyInn keeps the lane: a live, multiplayer
PR where agents and humans work in real time and the record never
drifts from the code.
