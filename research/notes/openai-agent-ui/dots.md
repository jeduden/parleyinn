# OpenAI dots: always-on agents, and what ParleyInn takes from them

OpenAI launched dots at DevDay on 29 September 2026: "remarkably
capable, always-on agents built to handle everything", powered by
GPT-6 Astra — [OpenAI on X](https://x.com/OpenAI/status/2104984504133918973),
[Introducing dots](https://openai.com/index/introducing-dots/) (the page
refused automated fetches; facts below come from OpenAI's docs and the
launch coverage).

## What a dot is

- "Your dot is an always-on agent that keeps work moving across your
  tools and projects." It has "its own computer and browser in the
  cloud", and cloud work continues while the user's devices are off —
  [Meet dots](https://learn.chatgpt.com/docs/dots).
- One identity across channels: "You reach the same dot in ChatGPT,
  Slack, Teams, or a call." Changing channel does not start a new dot
  or reset its memory; messages stay in their channels while the dot
  uses context across them — [Meet dots](https://learn.chatgpt.com/docs/dots).
- It keeps "its own notes about preferences, decisions, and ongoing
  work", separate from ChatGPT memory, updated automatically —
  [Tasks and memory](https://learn.chatgpt.com/docs/dots/tasks-and-memory).
- It can work on a connected local computer "while OpenAI's cloud
  coordinates the task", and create Work or Codex tasks, including
  cloud coding environments — [Meet dots](https://learn.chatgpt.com/docs/dots),
  [Local computer access](https://learn.chatgpt.com/docs/enterprise/cloud-local-access).
- Specialist dots get "specific identities, credentials, and tools
  through existing systems"; OpenAI works with Microsoft on Agent 365
  security controls; "Over time, we envision teams of Dots working
  together on your behalf" — [TechCrunch](https://techcrunch.com/2026/09/29/openai-launches-dots-its-bubbly-agentic-avatar/).
- Availability: Pro and Business Premium and Enterprise, rolling out;
  not in the EEA, UK or Switzerland for consumer plans —
  [Meet dots](https://learn.chatgpt.com/docs/dots),
  [NBC News](https://www.nbcnews.com/tech/tech-news/openai-launches-dots-ai-agents-safety-questions-rcna600338).

## How work is shown

- Activity view: inspect "progress, files, results, or requests for
  input", and instruct the dot while work continues —
  [Tasks and memory](https://learn.chatgpt.com/docs/dots/tasks-and-memory).
- "Your dot can divide work among background agents that run in
  parallel and report back to it", each in "separate, visible threads"
  the user can review and steer — [Tasks and memory](https://learn.chatgpt.com/docs/dots/tasks-and-memory).
- "A completed run doesn't by itself confirm that the requested result
  was achieved or delivered." — [Tasks and memory](https://learn.chatgpt.com/docs/dots/tasks-and-memory).

## Control

- Four rule levels per action: "Take action without asking", "Take
  action when you say so", "Ask before taking action", "Hand off to
  you" — [Control your dot](https://learn.chatgpt.com/docs/dots/controls).
- An "automatic review checks [actions] against your instructions,
  permissions, custom rules, and built-in safety requirements";
  password changes stay with the user; "Stopping work doesn't undo
  completed actions." — [Control your dot](https://learn.chatgpt.com/docs/dots/controls).
- Not documented: who else may instruct a dot in a shared Slack or
  Teams channel, how instructions inside emails or web pages are
  treated, and where dot data is stored or how it is deleted —
  [Control your dot](https://learn.chatgpt.com/docs/dots/controls),
  [Meet dots](https://learn.chatgpt.com/docs/dots).

## The safety climate at launch

- The day before launch OpenAI apologised after internal agents
  "accessed nonpublic information" on an Australian government site in
  June: "We are sorry and working to do better in the future." —
  [NBC News](https://www.nbcnews.com/tech/tech-news/openai-launches-dots-ai-agents-safety-questions-rcna600338).
- OpenAI withheld GPT-6.1 Astra because it "didn't quite meet the bar
  in terms of staying within scope and authorization" —
  [NBC News](https://www.nbcnews.com/tech/tech-news/openai-launches-dots-ai-agents-safety-questions-rcna600338).
- Axios framed the shift as "What did my AI assistant do now?" —
  [Axios](https://www.axios.com/2026/09/30/openai-dots-ai-agent-safety)
  (headline only; the article refused automated fetches).

## What ParleyInn takes from this

1. The question of the moment is "what did my agent do?". A lossless,
   tamper-evident record of every agent action, kept by the user, is
   the answer dots do not give: their memory is the dot's own notes,
   stored with OpenAI, and their activity view is not an audit trail.
2. One identity across surfaces. A dot is the same agent in ChatGPT,
   Slack and a call. A lane's harness should be the same participant
   whether reached from its terminal, the UI or another harness.
3. Background agents as visible, steerable child threads matches the
   fleet view's parent and child tiles.
4. Completed is not verified. Results in the lane should show what
   checked them: a claimed result, a local run, or the canonical CI.
5. Rule levels per action. The four levels (act, act when told, ask
   first, hand off) give the owner's trust model a familiar shape for
   what an agent may do on its own, next to I2's rule on what it reads.
6. Context mixing across channels is an injection path. A dot uses
   context from every channel; in ParleyInn, content from another channel
   or participant stays untrusted and pull-only (I2).
7. Dots need OpenAI's cloud, coordinate through it, and are not offered
   in the EEA or UK. A self-hosted ParleyInn has neither limit.
