# Hacker News discussions: agent session history alongside code, and git versus alternatives for agent workflows

Method note: threads were found with the hn.algolia.com search API and read in full through the Algolia items API (`/api/v1/items/<id>`), between 2025-01-01 and 2026-10-01. Points and comment counts are as returned on 2026-10-01. Comment counts from the item tree can be lower than the story's `num_comments` because flagged or deleted comments are dropped. Unless noted, each quoted or paraphrased view is one commenter's opinion, cited by its HN item link.

## Q1. Which HN threads matter, with title, URL, date and engagement

### Takeaway

The topic has drawn a dozen large threads since early 2026. The central ones are: "If AI writes code, should the session be part of the commit?" (497 pts, March 2026); Entire's launch (611 pts, Feb 2026); the Claude Code session-URL-in-commits issue (209 pts, Aug 2026); Zed DeltaDB/Delta (three threads, 154–529 pts); Cursor Origin (597 pts); Block's Buzz (378 pts); and the "git for agents" Show HNs (Oak 216 pts, re_gent 129 pts). Attribution tools like git-ai and Cursor's Agent Trace barely registered as stories (5–6 pts). They come up mainly inside other threads.

### Cited Findings

**Sessions or prompts committed with code**

- "If AI writes code, should the session be part of the commit?" was posted 2026-03-02, with 497 points and about 391 comments. It links git-memento, which "attaches AI session transcripts to commits using Git notes" — [HN 47212355](https://news.ycombinator.com/item?id=47212355); [repo](https://github.com/mandel-macaque/memento); [author comment](https://news.ycombinator.com/item?id=47212356)
- "Claude Session URL appended to commit messages and PR descriptions by default" was posted 2026-08-30, with 209 points and about 240 comments. It links GitHub issue anthropics/claude-code#66504 — [HN 49498201](https://news.ycombinator.com/item?id=49498201); [issue](https://github.com/anthropics/claude-code/issues/66504)
- "Cloudflare builds OAuth with Claude and publishes all the prompts" was posted 2025-06-02, with 889 points and 529 comments. The prompts sit in the commit history of workers-oauth-provider — [HN 44159166](https://news.ycombinator.com/item?id=44159166); [repo](https://github.com/cloudflare/workers-oauth-provider/)
- "AI tooling must be disclosed for contributions" (Ghostty PR 8289) was posted 2025-08-21, with 729 points and about 436 comments — [HN 44976568](https://news.ycombinator.com/item?id=44976568); [PR](https://github.com/ghostty-org/ghostty/pull/8289)
- dang proposed on 2026-02-21 that Show HNs of generated projects share their prompts. He wrote: "the prompts are the real source, while the GH repo or generated artifact is actually the object code" — [dang comment 47096202](https://news.ycombinator.com/item?id=47096202)
- "Prompts are (not) the new source code" (Quesma blog) was posted 2026-01-09 with 6 points and 1 comment, so it got little discussion — [HN 46552632](https://news.ycombinator.com/item?id=46552632); [post](https://quesma.com/blog/prompts-source-code/)
- "Tell HN: Do not include co-authored-by Claude in your commits" was posted 2026-04-20, with 11 points and 7 comments — [HN 47840791](https://news.ycombinator.com/item?id=47840791)
- "Why Git is no 'good' for AI-generated code" (SpecStory's GOOD, a git companion) was posted 2025-04-02, with 35 points and 11 comments — [HN 43557698](https://news.ycombinator.com/item?id=43557698); [GOOD.md](https://github.com/specstoryai/getspecstory/blob/main/GOOD.md)

**Session provenance and attribution platforms**

- "Ex-GitHub CEO launches a new developer platform for AI agents" (Entire) was posted 2026-02-10, with 611 points and 577 comments — [HN 46961345](https://news.ycombinator.com/item?id=46961345); [blog](https://entire.io/blog/hello-entire-world/)
- "How version control will evolve for the agent boom" (Entire blog) was posted 2026-07-09, with 57 points and 66 comments — [HN 48844709](https://news.ycombinator.com/item?id=48844709); [blog](https://entire.io/blog/how-version-control-will-evolve-for-the-agent-boom)
- "An Entirely New Git Hosting Network" (Entire) was posted 2026-07-08, with 20 points and 4 comments — [blog](https://entire.io/blog/an-entirely-new-git-hosting-network) (found via Algolia search; thread not read)
- "Show HN: Tracking AI Code with Git AI" was posted 2025-11-10 with 6 points and 4 comments. A later Show HN of the git-ai repo (2026-01-23) got 4 points — [HN 45878276](https://news.ycombinator.com/item?id=45878276); [HN 46736044](https://news.ycombinator.com/item?id=46736044)
- "Show HN: GitHub Browser Plugin for AI Contribution Blame in Pull Requests" builds on git-ai. It was posted 2026-02-03, with 62 points and about 34 comments — [HN 46871473](https://news.ycombinator.com/item?id=46871473)
- "Cursor Agent Trace RFC" was posted 2026-01-29 with 5 points and 2 comments. Three more submissions of agent-trace.dev got 1 point each — [HN 46813938](https://news.ycombinator.com/item?id=46813938); [spec](https://agent-trace.dev/)

**New version control systems and forges built for agents**

- "Zed DeltaDB" was posted 2026-08-05, with 529 points and 314 comments — [HN 49187256](https://news.ycombinator.com/item?id=49187256); [page](https://zed.dev/deltadb)
- "Software is made between commits" (introducing DeltaDB) was posted 2026-06-11, with 319 points and 214 comments — [HN 48492533](https://news.ycombinator.com/item?id=48492533); [blog](https://zed.dev/blog/introducing-deltadb)
- "Replacing Pull Requests with Delta" (public beta) was posted 2026-09-16, with 154 points and 102 comments — [HN 49727245](https://news.ycombinator.com/item?id=49727245); [blog](https://zed.dev/blog/delta-public-beta)
- "Graphite is joining Cursor" was posted 2025-12-19, with 276 points and 253 comments — [HN 46327206](https://news.ycombinator.com/item?id=46327206)
- "Cursor launches Origin, GitHub alternative" was posted 2026-08-17, with 597 points and 454 comments — [HN 49334209](https://news.ycombinator.com/item?id=49334209); [changelog](https://cursor.com/changelog/origin-code-hosting)
- Cursor's ownership comes up in the Origin thread. Related stories: "SpaceX to buy Cursor for $60B" (2026-06-16, 1151 points, 1703 comments) and "Our decision on Cursor following its acquisition by SpaceX" (OpenAI, 2026-08-29, 852 points) — [HN 48553224](https://news.ycombinator.com/item?id=48553224); [HN 49486172](https://news.ycombinator.com/item?id=49486172) (neither thread read)
- "Jack Dorsey launches Buzz to combine team chat, AI agents and Git hosting" was posted 2026-07-21, with 378 points and 339 comments — [HN 48995213](https://news.ycombinator.com/item?id=48995213); [repo](https://github.com/block/buzz)
- "Show HN: Oak – Git alternative designed for agents" was posted 2026-06-22, with 216 points and 189 comments. A follow-up, "Oak: Git for Agents" (2026-07-03), got 21 points — [HN 48631726](https://news.ycombinator.com/item?id=48631726); [HN 48779321](https://news.ycombinator.com/item?id=48779321)
- "Show HN: Git for AI Agents" (re_gent) was posted 2026-05-08, with 129 points and 67 comments — [HN 48063548](https://news.ycombinator.com/item?id=48063548); [repo](https://github.com/regent-vcs/re_gent)
- "Code Storage by the Pierre Computer Company" was posted 2026-02-10, with 66 points and 42 comments — [HN 46957629](https://news.ycombinator.com/item?id=46957629); [site](https://code.storage/)
- "Grit: Rewriting Git in Rust with agents" (GitButler) was posted 2026-06-09, with 178 points and 306 comments — [HN 48466812](https://news.ycombinator.com/item?id=48466812)
- "DoltLite: A SQLite fork with Git-style version control, built with 2k agent PRs" was posted 2026-09-01, with 62 points and 60 comments — [HN 49516848](https://news.ycombinator.com/item?id=49516848)

**Beads, Gas Town and agent memory**

- "Beads – A memory upgrade for your coding agent" was posted 2025-11-28, with 111 points and 68 comments. An earlier Medium post (2025-10-13) got 19 points — [HN 46075616](https://news.ycombinator.com/item?id=46075616); [HN 45566864](https://news.ycombinator.com/item?id=45566864)
- "Show HN: I replaced Beads with a faster, simpler Markdown-based task tracker" (ticket) was posted 2026-01-04, with 84 points and 51 comments — [HN 46487580](https://news.ycombinator.com/item?id=46487580)
- Gas Town threads:
  - "Welcome to Gas Town" (2026-01-01, 354 points, 224 comments) — [HN 46458936](https://news.ycombinator.com/item?id=46458936)
  - Maggie Appleton's "Gas Town's agent patterns, design bottlenecks, and vibecoding at scale" (2026-01-23, 403 points, 433 comments) — [HN 46734302](https://news.ycombinator.com/item?id=46734302)
  - "Gas Town Decoded" (2026-01-14, 219 points) — [HN 46624883](https://news.ycombinator.com/item?id=46624883)
  - "Gas Town: From Clown Show to v1.0" (2026-04-14, 113 points) — [HN 47770124](https://news.ycombinator.com/item?id=47770124)
  - "Does Gas Town 'steal' usage from users' LLM credits to improve itself?" (2026-04-15, 253 points) — [HN 47785053](https://news.ycombinator.com/item?id=47785053)
  - Only the first two were read in depth.
- "Agent memory as a file format" was posted 2026-08-31, with 191 points and 96 comments — [HN 49508317](https://news.ycombinator.com/item?id=49508317)
- "OKF Agent Memory – Git-native persistent memory for AI coding agents" was posted 2026-09-05, with 81 points and 32 comments — [HN 49581240](https://news.ycombinator.com/item?id=49581240)
- "Show HN: I replaced vector databases with Git for AI memory (PoC)" (DiffMem) was posted 2025-08-21, with 198 points and 45 comments — [HN 44969622](https://news.ycombinator.com/item?id=44969622)
- "Show HN: ctx – Search the coding agent history already on your machine" was posted 2026-07-02, with 65 points and 43 comments — [HN 48763462](https://news.ycombinator.com/item?id=48763462)

**Jujutsu, worktrees and multi-agent coordination**

- "jj – the CLI for Jujutsu" (2026-04-14, 549 points, 498 comments) has a sub-discussion on agents using jj — [HN 47763759](https://news.ycombinator.com/item?id=47763759)
- "The creator of Jujutsu has joined ERSC" was posted 2026-09-01, with 265 points and 252 comments — [HN 49525297](https://news.ycombinator.com/item?id=49525297)
- "Show HN: jj-benchmark – Evaluating AI agents on Jujutsu version control" (2026-03-12) got 5 points — [HN 47352189](https://news.ycombinator.com/item?id=47352189)
- "Jujutsu worktrees are convenient" (2025-12-03, 128 points, 113 comments) exists; it was not read — [HN 46141731](https://news.ycombinator.com/item?id=46141731)
- "LLM codegen go brrr – Parallelization with Git worktrees and tmux" was posted 2025-05-28, with 156 points and 73 comments — [HN 44116872](https://news.ycombinator.com/item?id=44116872)
- "Show HN: Ccgs – Collaborative Claude Code sessions, stored in Git branches" (claude-git-sessions) was posted 2026-06-06, with 6 points and 2 comments — [HN 48426297](https://news.ycombinator.com/item?id=48426297)
- "Git-bug: Distributed, offline-first bug tracker embedded in Git" was posted 2026-09-25, with 363 points and 109 comments — [HN 49843174](https://news.ycombinator.com/item?id=49843174)

### Inferences

- Attention clusters around funded launches (Entire, Zed, Cursor, Block) and around Claude Code's own default behaviour. The small open tools that do exactly what ParleyInn contemplates got little attention as stories: git-memento, git-ai, ccgs and Agent Trace. Their ideas mostly spread through comments in the bigger threads.
- The Claude Code session-URL thread is the one most directly about ParleyInn's host agent. It shows how users react when session linkage appears in their git history by default.

### Gaps

- I found no HN story specifically titled "git is not built for AI agents". The theme is argued inside the Oak, re_gent, DeltaDB and Entire threads instead.
- I found no dedicated HN thread on Agent Trace beyond 5-point submissions, so community views on it are thin.
- The Gas Town credits thread, "Gas Town Decoded", the Jujutsu worktrees thread and the SpaceX/Cursor threads were catalogued but not read.

## Q2. Main arguments for and against storing sessions with code (and git versus alternatives)

### Takeaway

Most commenters oppose committing raw transcripts into the main history. The reasons given are noise, context-window bloat, non-reproducibility, privacy and PII leakage, and "the code is the artifact". There is broad support for keeping a distilled record: plan/spec files, ADRs, good commit messages. There is also notable support for keeping the full session out-of-band but linked: git notes, an orphan branch, a separate repo or a session ID. Privacy and surveillance concerns are strongest against hosted systems (Zed Delta, Cursor Origin, Claude session URLs). Proposals for new version control systems "for agents" face a "why not git?" burden, and commenters often answer that burden with jj.

### Cited Findings

**Usefulness for review, debugging and provenance (for)**

- jlawrence6809 wrote a script early on to append the Claude session ID to every commit. He calls it "a lifesaver when you are trying to debug an old commit to pull up the chat session that actually wrote it" — [HN 49498363](https://news.ycombinator.com/item?id=49498363)
- The top comment in the session-URL thread (88 replies) calls the link attribution, and says attribution is professional. "I still get to control whether other people can see the session, but I don't lose it" — [HN 49498447](https://news.ycombinator.com/item?id=49498447)
- TeMPOraL says the session UID is useful even when others cannot open the transcript. It links "several commits across multiple unrelated repositories", records the causal link and joins up with agent telemetry — [HN 49506400](https://news.ycombinator.com/item?id=49506400)
- D-Machine argues that code-generation prompts should be saved verbatim, since vibe-coded work is "highly inscrutable and hard to review". Research prompts can be distilled — [HN 47214019](https://news.ycombinator.com/item?id=47214019); [HN 47213895](https://news.ycombinator.com/item?id=47213895)
- bear3r: "reproducibility isn't really the goal imo. more like a decision audit trail" — [HN 47213417](https://news.ycombinator.com/item?id=47213417)
- fragmede argues that intent matters years later. "I need a login screen" and "…magic link login and nothing else" lead to different refactoring decisions — [HN 47213316](https://news.ycombinator.com/item?id=47213316)
- In the Cloudflare thread, commenters used the published prompts to audit the work. One prompt asked Claude to remove a "backup" encryption key that defeated end-to-end encryption, and another commit records a human fixing a bug Claude kept getting wrong — [HN 44167006](https://news.ycombinator.com/item?id=44167006); [HN 44159659](https://news.ycombinator.com/item?id=44159659)
- Kenton Varda (author) states Cloudflare's rule: "the human engineer directing the AI must fully understand and take responsibility for any code which the AI has written" — [HN 44159712](https://news.ycombinator.com/item?id=44159712)
- arppacket says sessions are "primarily for future AI to read". Future agents could treat the current session as an extension of the long history — [HN 47214600](https://news.ycombinator.com/item?id=47214600)
- The git-ai author says reviewing colleagues' prompts and seeing "where they stepped in to override" changed how he reviews AI code — [HN 45878276](https://news.ycombinator.com/item?id=45878276)
- An early Delta beta user calls it "a huge improvement over reviewing directly on the PR". Being able to "jump into a team mates thread and see the context" is the draw — [HN 49753499](https://news.ycombinator.com/item?id=49753499)
- In the Entire thread, willmarquis frames the real need as observability for agent code: "what did the agent do, why, and how do I audit/reproduce it?" He argues markdown-in-git works at small scale and breaks with many agents across many sessions — [HN 46969371](https://news.ycombinator.com/item?id=46969371)

**Noise, bloat and value (against)**

- The top comment in the session-in-commit thread (53 replies) says sessions "contain a significant amount of noise, incorrect implementations, and red herrings. The product of the session is what matters". It would store the initial spec or first prompt plus a summary — [HN 47214007](https://news.ycombinator.com/item?id=47214007)
- rfw300: "The agent session is a messy intermediate output… have your agent write a commit message or a documentation file that is polished" — [HN 47213208](https://news.ycombinator.com/item?id=47213208)
- gck1 says at least half of session JSONL is noise. Committing it would spoil "study git commits to ground yourself" for agents: "Context window is precious" — [HN 47230488](https://news.ycombinator.com/item?id=47230488)
- causal: "It is just context-bloat for whatever agent ends up reading it" — [HN 47219726](https://news.ycombinator.com/item?id=47219726)
- Many commenters compare transcripts to search history, keystroke logs or car exhaust:
  - "Should my google search history be part of the commit?" — [HN 47213296](https://news.ycombinator.com/item?id=47213296)
  - "keystroke logs" — [HN 47214164](https://news.ycombinator.com/item?id=47214164)
  - "car exhaust", answered with "We use flight data recorders on airplanes, though" — [HN 47213336](https://news.ycombinator.com/item?id=47213336); [HN 47213441](https://news.ycombinator.com/item?id=47213441)
- onion2k frames it as the squash-versus-keep-history debate in a new form — [HN 47214096](https://news.ycombinator.com/item?id=47214096)
- dang summarised the objections from his own earlier proposal: (1) no single input generates a project; (2) human–AI back-and-forth is a conversation, not a compiler loop; (3) the artifact would be "so noisy and complicated" it adds little — [HN 47213630](https://news.ycombinator.com/item?id=47213630)
- Commenters question size and scale. One says "Isnt this overloading git commits too much? Like 50kb per commit message". williamstein notes per-turn context "could in theory be nearly 1MB", and asks whether checkouts get heavy after a thousand turns. A reply notes Entire's data sits on another branch — [HN 46967015](https://news.ycombinator.com/item?id=46967015); [HN 46969098](https://news.ycombinator.com/item?id=46969098); [HN 46971764](https://news.ycombinator.com/item?id=46971764)
- sdevonoes, replying to Entire's claim that "session logs are now the most important artifact", says committing specs and prompts "doesn't scale". After a while you have "hundreds if not thousands" of spec files — [HN 48847525](https://news.ycombinator.com/item?id=48847525)
- In the Delta threads, readers worry about having to read teammates' agent conversations. "Why is this better than a good pr description" — [HN 49754282](https://news.ycombinator.com/item?id=49754282); [HN 49756716](https://news.ycombinator.com/item?id=49756716)
- daemonk recorded sessions at first "and realized I never went back to it" — [HN 47213312](https://news.ycombinator.com/item?id=47213312). Jason Hall built cnotes (a Claude hook writing conversations to git notes, with a Chrome extension to display them) "but ended up not getting much out of it" — [HN 46962431](https://news.ycombinator.com/item?id=46962431)

**Reproducibility**

- Many commenters dismiss replay because LLM output is non-deterministic:
  - "This is only true if a llm session would produce a deterministic output" — [HN 47213287](https://news.ycombinator.com/item?id=47213287)
  - "Reproducibility in ML generation is a total myth… changes to continuous batching… or just a different CUDA driver" — [HN 47216915](https://news.ycombinator.com/item?id=47216915)
  - "In a year, they'll deprecate this model's API" — [HN 47216752](https://news.ycombinator.com/item?id=47216752)
- nz argues a commit would also need the exact model, and that only open weights could make that work — [HN 47231510](https://news.ycombinator.com/item?id=47231510)
- dang's counterpoint: the input is "the tuple of (prompt, model, X)", and it is still the source in the sense that output is generated from it — [HN 47103417](https://news.ycombinator.com/item?id=47103417)

**Privacy, secrets and PII**

- raggi: "Someones going to leak important private data using something like this". A bug report with user PII goes into the session, the code reviewer never sees it, and "the notes thing they forgot about goes and makes this all public" — [HN 47213738](https://news.ycombinator.com/item?id=47213738)
- kzahel wants to share sessions automatically, but only "a carefully PII/secrets redacted session". He says "Sharing raw JSONL is probably a waste" — [HN 47214945](https://news.ycombinator.com/item?id=47214945)
- dogas built a tool that pulls Claude sessions into a separate repo and "sanitizes any sensitive data like API keys" — [HN 47218987](https://news.ycombinator.com/item?id=47218987); [repo](https://github.com/gammons/ai-session)
- In the ccgs thread, AG342 says people paste env secrets and customer data into Claude anyway, making shared sessions a systemic risk. jazzen asks for "a middle layer that preserves enough state to resume and debug while aggressively quarantining obvious secret classes and high-risk blobs". He describes the tradeoff as "perfect replayability versus shareability" — [HN 48426872](https://news.ycombinator.com/item?id=48426872); [HN 48427556](https://news.ycombinator.com/item?id=48427556)
- On the Claude session URL appearing in a public repo: "I was left wondering 'did I just leak my private session?'… why it isn't opt-in" — [HN 49498601](https://news.ycombinator.com/item?id=49498601)
- glub runs three repos per project: code, docs, and "Sensitive data (holds the transcript snapshots…)". Only a "special" agent can read the latter two — [HN 49505920](https://news.ycombinator.com/item?id=49505920)
- aghilmort keeps a full private repo with transcripts and agent attribution, plus a public repo showing a subset without session IDs — [HN 49499562](https://news.ycombinator.com/item?id=49499562)
- Some treat prompts as personal or commercial property. "I consider my prompts and prompting style to be trade secrets" drew mockery in reply — [HN 49501291](https://news.ycombinator.com/item?id=49501291); [HN 49501379](https://news.ycombinator.com/item?id=49501379). Another: "basically would be a major privacy breach" — [HN 47217311](https://news.ycombinator.com/item?id=47217311). Another: "The way I prompt is my asset" — [HN 47214447](https://news.ycombinator.com/item?id=47214447)
- PeterStuer: sessions "might contain many artifacts that are not suited for open sourcing… preserved private session records might be of great personal benefit" — [HN 47216022](https://news.ycombinator.com/item?id=47216022)

**Surveillance and hosted lock-in**

- The top DeltaDB-launch comment (87 replies): "The code I write between commits is my thinking… I don't want my thoughts to be serialized, version controlled and publicly accessible" — [HN 48494076](https://news.ycombinator.com/item?id=48494076)
- drdexebtjl: DeltaDB "gives micro-managers the data they need to micro-manage you", and he imagines "layoffs being justified with 'bad prompt quality'". mplanchard also notes "a level of developer surveillance with which I am deeply uncomfortable" — [HN 49188128](https://news.ycombinator.com/item?id=49188128); [HN 48494124](https://news.ycombinator.com/item?id=48494124)
- On Delta: "You also upload your code and conversations into their server-hosted DeltaDB service and the only way to remove it is to email their privacy contact." Others complain it "forces you to use their editor/agent harness" — [HN 49758135](https://news.ycombinator.com/item?id=49758135); [HN 49756716](https://news.ycombinator.com/item?id=49756716); [HN 49758825](https://news.ycombinator.com/item?id=49758825)
- Linkrot: "Do any of us truly believe these links will work 30 years from now?… Git is supposed to be the durable storage medium, self containing." Others want "an open format stored in/with the repository rather than their website" — [HN 49499194](https://news.ycombinator.com/item?id=49499194); [HN 49498632](https://news.ycombinator.com/item?id=49498632)
- nijave: "If you want session context, commit the session or prompt or add a git note. I think opaque URIs are the wrong approach… Proprietary URIs" — [HN 49499112](https://news.ycombinator.com/item?id=49499112)
- In the Origin thread, commenters refuse to host code with a Musk-owned company and expect x.ai to train on hosted code. Others push decentralised forges instead: Radicle, federated Forgejo, Tangled — [HN 49347692](https://news.ycombinator.com/item?id=49347692); [HN 49348936](https://news.ycombinator.com/item?id=49348936); [HN 49348841](https://news.ycombinator.com/item?id=49348841); [HN 49339647](https://news.ycombinator.com/item?id=49339647)
- In the Buzz thread, a Slack employee warns that "multiplayer agents" can leak data across permission boundaries. Teams then "end up having to write and maintain complex rulesets about what specific resources agents have access to" — [HN 48996051](https://news.ycombinator.com/item?id=48996051)

**Where to store it: git notes, branches, trailers, separate repos**

- git notes are the most-suggested container. Supporters say they keep noise out of history: "No one has to read them if they're not interested", and "If you do not sync them you have no knowledge of their existence" — [HN 47213853](https://news.ycombinator.com/item?id=47213853); [HN 47213999](https://news.ycombinator.com/item?id=47213999); [HN 47214115](https://news.ycombinator.com/item?id=47214115); [HN 47214750](https://news.ycombinator.com/item?id=47214750)
- Known drawbacks of git notes: GitHub does not show them, and "You also risk losing those notes if you rebase the commit they are attached to" — [HN 47213187](https://news.ycombinator.com/item?id=47213187). ramoz says git notes carry "complexities around it", and calls putting all sessions in separate branches the "less elegant" option. He mentions size and squash-merge workflows as issues — [HN 47213163](https://news.ycombinator.com/item?id=47213163)
- shayief argues the opposite: attribution belongs in the commit object or a trailer, not notes, so it is "part of repository history (and commit SHA)". dec0dedab0de says aider does this — [HN 46872346](https://news.ycombinator.com/item?id=46872346); [HN 46872595](https://news.ycombinator.com/item?id=46872595)
- mikepurvis proposes a commit trailer reference to external storage: `Chat-Session-Ref: claude://…` — [HN 47214148](https://news.ycombinator.com/item?id=47214148)
- Entire Checkpoints are quoted in the thread from Entire's blog. On each agent commit the CLI "writes a structured checkpoint object and associates it with the commit SHA", then pushes the metadata "to a separate branch (entire/checkpoints/v1), giving you a complete, append-only audit log inside your repository" — [HN 46969098](https://news.ycombinator.com/item?id=46969098); [blog](https://entire.io/blog/hello-entire-world/)
- An Entire CLI maintainer: "We take the raw session logs, put it in the repo with a stable link to the commit". An Entire employee: "we store the whole session but with tools to query the summarized choices… The session acts as metadata" — [HN 48846265](https://news.ycombinator.com/item?id=48846265); [HN 48867025](https://news.ycombinator.com/item?id=48867025)
- vidarh notes that for Claude Code the session is "literally a JSONL file in .claude/projects/…" and a commit hook can archive it. He dumps JSONL snapshots into docs/plans — [HN 46973388](https://news.ycombinator.com/item?id=46973388); [HN 46973614](https://news.ycombinator.com/item?id=46973614)
- ccgs shares Claude Code sessions through an orphan branch (`@ccgs/<name>`) in the existing remote. On pull it rewrites the author's absolute `cwd` paths so `claude --resume` works — [HN 48426297](https://news.ycombinator.com/item?id=48426297)
- hakanderyal started by committing a "devlog" plus the first prompt and plan. "Later due to noise & volume" he moved them into a database and commits only the devlog ID — [HN 47214109](https://news.ycombinator.com/item?id=47214109)
- Local retention is a practical problem. Claude archives or deletes local conversations after 30 days unless `cleanupPeriodDays` is raised. One user moves Claude memory into the repo via a symlink — [HN 49498411](https://news.ycombinator.com/item?id=49498411); [HN 49501742](https://news.ycombinator.com/item?id=49501742); [HN 49509075](https://news.ycombinator.com/item?id=49509075)

**Distillation instead of transcripts (the most common middle ground)**

- jedberg's workflow (50 replies): project.md → plan.md → todo list, then commit project.md and plan.md with the code. Replies describe design/plan/debug doc trees and spec-kit — [HN 47214629](https://news.ycombinator.com/item?id=47214629); [HN 47215088](https://news.ycombinator.com/item?id=47215088)
- hatmanstack: "versioning the plan files is the better artifact, it preserves agentic decisions and my own reasoning without the noise" — [HN 47213988](https://news.ycombinator.com/item?id=47213988)
- claud_ia says sessions hold one signal that rarely reaches commits, "the counterfactual space", and that this belongs in ADRs — [HN 47215896](https://news.ycombinator.com/item?id=47215896)
- reflectt describes structured post-task reflections instead: what was tried, what failed, and the decision reasoning — [HN 47214274](https://news.ycombinator.com/item?id=47214274)
- In the Entire VC thread: "We need to store CHOICES with the commit or branch. Not the whole session" — [HN 48846415](https://news.ycombinator.com/item?id=48846415)
- In re_gent: plans checked in "with the same commit so that you can see the why" is "Much simpler" than a new tool — [HN 48067135](https://news.ycombinator.com/item?id=48067135)

**Merge conflicts and multi-agent coordination over git**

- On Beads: "There kept being merge conflicts and the agent just kept one or the other changes instead of merging it intelligently, killing any work I did" — [HN 46469974](https://news.ycombinator.com/item?id=46469974)
- Simon Willison's workaround for Beads conflicts: "Add the .beads directory to .gitignore and always make edits on that same machine" — [HN 46080641](https://news.ycombinator.com/item?id=46080641)
- The ticket author dropped Beads because "every release made it slower", and its "background daemon took to syncing the wrong things at the wrong times" — [HN 46487580](https://news.ycombinator.com/item?id=46487580)
- A Gas Town issue cited in the thread: every `gt` command runs `bd version`, and "Under high concurrency (17+ agent sessions), this check times out and blocks gt commands" — [HN 46734587](https://news.ycombinator.com/item?id=46734587)
- Beads stores tasks in Dolt (a versioned database), not in plain git files, per a commenter — [HN 48846446](https://news.ycombinator.com/item?id=48846446)
- On DoltLite: "The problem DoltLite solves is merging, not forking" — [HN 49519289](https://news.ycombinator.com/item?id=49519289)
- Agents mishandle git: "my agent rebased and forcepushed with conflicts" — [HN 48065840](https://news.ycombinator.com/item?id=48065840). See also "Tell HN: Cursor agent force-pushed despite explicit 'ask for permission' rules" — [HN 46728766](https://news.ycombinator.com/item?id=46728766)
- Worktrees are the standard answer for parallel agents. The usual escalation is "multiple VMs" — [HN 46510368](https://news.ycombinator.com/item?id=46510368). A noted downside is per-worktree dependency installs, such as node_modules in JS projects — [HN 44123806](https://news.ycombinator.com/item?id=44123806)
- Oak pitches "virtual mounts" so agents locally and in the cloud need no full repo copy and need not "fight worktrees" — [HN 48631726](https://news.ycombinator.com/item?id=48631726)
- pjm331 says builders of git alternatives miss "a multi-repo story". He keeps context on issues referenced from commits — [HN 48493235](https://news.ycombinator.com/item?id=48493235)
- gritzko (a CRDT researcher) says implementing CRDT merges in git itself lets tickets live as plain Markdown, "having CRDT merges for both code and metadata" — [HN 49845533](https://news.ycombinator.com/item?id=49845533); [blog](https://replicated.live/blog/meta)

**Git versus alternatives**

- SwellJoe (21 replies): any tool "for agents" must beat what is in the training data. "Models know git because there's a monstrous amount of git in their training data", so new tools carry a context cost. The Oak author agrees "on the burden of proof" — [HN 48635235](https://news.ycombinator.com/item?id=48635235); [HN 48635799](https://news.ycombinator.com/item?id=48635799)
- hnlmorg: "the performance of git isn't a bottleneck for agents". Others note inference dominates latency — [HN 48633789](https://news.ycombinator.com/item?id=48633789); [HN 48779992](https://news.ycombinator.com/item?id=48779992)
- In re_gent: "Just use git", and "None of these X-for-agents seem to motivate why they don't use X". _ink_ defends it: "a separate VCS for the agent… keep git clean and easy to understand for humans and still keep all the verbosity the agent needs" — [HN 48064990](https://news.ycombinator.com/item?id=48064990); [HN 48064856](https://news.ycombinator.com/item?id=48064856); [HN 48064954](https://news.ycombinator.com/item?id=48064954)
- Codex reportedly uses git as a backend for file-state snapshots that stay out of the user's history — [HN 48066489](https://news.ycombinator.com/item?id=48066489); [PR](https://github.com/openai/codex/pull/6041)
- Jujutsu comes up often as the existing answer:
  - Zambyte uses jj per prompt (`jj evolog`, diff between prompts) — [HN 48065446](https://news.ycombinator.com/item?id=48065446); [HN 48075157](https://news.ycombinator.com/item?id=48075157)
  - "`jj undo` and the jj architecture generally make it difficult for agents to screw something up in a way that cannot be recovered" — [HN 47764604](https://news.ycombinator.com/item?id=47764604)
  - One user has agents summarise prompt instructions via `jj describe` — [HN 47778307](https://news.ycombinator.com/item?id=47778307)
  - "Jujutsu might be what you're looking for" — [HN 48632141](https://news.ycombinator.com/item?id=48632141)
- Entire users asked "Is JJ compatibility in the cards?" and "Why not build on top of jj?". The reply: "jj has compatibility gaps" — [HN 46968126](https://news.ycombinator.com/item?id=46968126); [HN 46963792](https://news.ycombinator.com/item?id=46963792); [HN 46965760](https://news.ycombinator.com/item?id=46965760)
- A GitButler co-founder built a benchmark comparing git, jj and GitButler in agent use — [HN 48847192](https://news.ycombinator.com/item?id=48847192); [vcbench](https://vcbench.dev/)
- Fossil is cited for merges that fold commits — [HN 48632226](https://news.ycombinator.com/item?id=48632226)
- Buzz describes itself as "an open-source, self-hosted workspace that combines team chat, AI agents and Git hosting using signed Nostr events, so teams can keep control of their data" (quoted from its site). sulam questions what Nostr adds. baron3dl answers: "nostr solves for who owns the shared service. nobody" — [HN 48996363](https://news.ycombinator.com/item?id=48996363); [HN 48997102](https://news.ycombinator.com/item?id=48997102); [HN 48997310](https://news.ycombinator.com/item?id=48997310)
- Cursor Origin is currently a GitHub-style host. Its developer (a Graphite co-founder) promised "a handful of features starting to change source control to better understand and work with agents". Commenters found it "a bit of a let down" as "a GitHub clone" — [HN 49334347](https://news.ycombinator.com/item?id=49334347); [HN 49338232](https://news.ycombinator.com/item?id=49338232); [HN 49338010](https://news.ycombinator.com/item?id=49338010)
- On the name: "push to origin main" becomes ambiguous to LLMs, "walking a thin line between genius-growth-move and domain typosquatting" — [HN 49335466](https://news.ycombinator.com/item?id=49335466)
- Pierre pitches "code storage for machines – think GitHub's infrastructure layer, but API-first and tuned for LLMs". Commenters mock it as "'git init --bare' as a service" and criticise its storage price — [HN 47016562](https://news.ycombinator.com/item?id=47016562); [HN 47015960](https://news.ycombinator.com/item?id=47015960); [HN 47016272](https://news.ycombinator.com/item?id=47016272)
- On the DeltaDB launch: "Commercially speaking, going against Git is a fool's errand". The reply: "This isn't trying to be a replacement for Git" — [HN 49193353](https://news.ycombinator.com/item?id=49193353); [HN 49193677](https://news.ycombinator.com/item?id=49193677). skybrian likens Delta to jj: a database of changes kept in sync with the working directory, which the agent edits directly — [HN 49752736](https://news.ycombinator.com/item?id=49752736)
- Steve Klabnik says ERSC (where jj's creator now works) is "building infrastructure for enterprises, not a social coding site" — [HN 49526190](https://news.ycombinator.com/item?id=49526190)

**Prompt injection and malicious repo content**

- The theme is rarely raised in these threads. Examples found:
  - "The security implications of scanning and merging external prompts at scale are going to be interesting" — [HN 48845627](https://news.ycombinator.com/item?id=48845627)
  - A `.git` folder with a malicious `fsmonitor` that runs when tools open a repo — [HN 47770185](https://news.ycombinator.com/item?id=47770185)
  - "Remote Prompt Injection in GitLab Duo Leads to Source Code Theft" (2025-05-23, 214 points) — [HN 44070626](https://news.ycombinator.com/item?id=44070626) (not read)
- Memory poisoning gets more attention. "once there is one poisoned line of text it negatively affects everything else downstream". Another commenter cites Yegge's "heresies" — "untrue things that stick around and permanently influence its behavior" — [HN 49509345](https://news.ycombinator.com/item?id=49509345); [HN 49510087](https://news.ycombinator.com/item?id=49510087)

**Legal and licensing of AI output**

- panny argues that unlabelled AI-authored commits are "copyfraud", because "AI does not enjoy copyright protection". Others push back that the US Copyright Office guidance is not law, and that human-contributed parts remain protected — [HN 49498379](https://news.ycombinator.com/item?id=49498379); [HN 49498550](https://news.ycombinator.com/item?id=49498550); [HN 49498592](https://news.ycombinator.com/item?id=49498592); [CRS LSB10922](https://www.congress.gov/crs-product/LSB10922)
- otterley predicts lawyers will counsel "line-by-line provenance of their source code" for future copyright disputes — [HN 49498807](https://news.ycombinator.com/item?id=49498807)
- In the Ghostty thread: "There is also IP taint when using 'AI'". Others ask whether open-source licences stay enforceable if most code is AI-generated. One commenter cites the US Copyright Office Part 2 report — [HN 44976959](https://news.ycombinator.com/item?id=44976959); [HN 44976709](https://news.ycombinator.com/item?id=44976709); [HN 44983033](https://news.ycombinator.com/item?id=44983033)
- Trust and provenance for reviewers: "reviewers are fallible and finite, so trust enters the equation inevitably" — [HN 44976945](https://news.ycombinator.com/item?id=44976945)
- GitButler's Grit stated that the agent-written Rust code "is not a derivative work that would require carrying forward the GPL". The top replies call this "plagiarism of GPL-licensed code, and license-washing" — [HN 48468904](https://news.ycombinator.com/item?id=48468904); [HN 48472056](https://news.ycombinator.com/item?id=48472056)
- A commenter links Debian's position on AI contributions (LWN). He argues responsibility lies with the contributor, which is why he dislikes session links — [HN 49498543](https://news.ycombinator.com/item?id=49498543); [LWN](https://lwn.net/Articles/1091231/)
- On attribution as data: "Do not include co-authored-by Claude", so that AI companies cannot filter AI commits out of training sets. A reply says vendors can detect their outputs anyway — [HN 47840791](https://news.ycombinator.com/item?id=47840791); [HN 47842414](https://news.ycombinator.com/item?id=47842414)

**Claude Code's session-URL default (direct relevance to ParleyInn's host)**

- One commenter says the maintainer limited the feature to web and Remote Control sessions; another reports it appeared from plain desktop CLI use, so the reports conflict. Enabling Remote Control once makes it global for all sessions — [HN 49498386](https://news.ycombinator.com/item?id=49498386); [HN 49506329](https://news.ycombinator.com/item?id=49506329); [HN 49498657](https://news.ycombinator.com/item?id=49498657)
- Opt-out is documented as: "set commit and pr to empty strings and sessionUrl to false". A commenter also reports an env var, `CLAUDE_CODE_SUPPRESS_SESSION_ATTRIBUTION`, added in 2.1.202 (July 8, 2026). That claim rests on a gist and is not verified — [HN 49501534](https://news.ycombinator.com/item?id=49501534); [settings docs](https://code.claude.com/docs/en/settings-reference#attribution); [HN 49508415](https://news.ycombinator.com/item?id=49508415)
- One user reports Claude ignoring the attribution block in settings.json in workflows on 2.1.246 — [HN 49523060](https://news.ycombinator.com/item?id=49523060)
- Objections in the thread:
  - The link is advertising ("Sent from my iPhone" vibes) — [HN 49498502](https://news.ycombinator.com/item?id=49498502)
  - It is misleading when humans edited the code — [HN 49498349](https://news.ycombinator.com/item?id=49498349)
  - It shifts burden onto reviewers — [HN 49500291](https://news.ycombinator.com/item?id=49500291)
  - It replaces the reasoning that should sit in the commit message — [HN 49498642](https://news.ycombinator.com/item?id=49498642)
  - It shipped default-on without release notes — [HN 49499030](https://news.ycombinator.com/item?id=49499030)

### Inferences

- The thread record supports a design that keeps the full, lossless session out of the main branch and the working tree. It still wants the session addressable from a commit, by ID or trailer, and queryable rather than read raw. That is close to what Entire, git-memento and ccgs do, and what ParleyInn's append-only record plus pull-only recall implies.
- Commenters want privacy controls before sharing: redaction of secrets and PII, private-by-default, separate access tiers. That favours ParleyInn's security-first stance. A git transport that pushes session data to a shared remote by default would meet the same backlash as the Claude session-URL default.
- Commenters distrust durability that depends on vendor URLs and want an open format in or beside the repository. ParleyInn's local, self-contained record answers that, provided the transport does not depend on a hosted service.
- The Beads experience shows that agent-written metadata in git meets real merge-conflict and concurrency problems at multi-agent scale. Append-only, per-session or per-agent files (or CRDT/union merges) avoid the case where "the agent kept one side".
- Prompt injection through stored history is almost absent from HN discussion. ParleyInn's I2 concern is ahead of the community, so it is a point to explain rather than one that will resonate unprompted.

### Gaps

- I found no HN thread with measured data on repository growth from storing transcripts in git (notes or branches). The size claims are back-of-envelope comments.
- No thread compared git notes, orphan branches and separate repos head-to-head with operational experience: fetch refspecs, rebase loss, GC. Only scattered anecdotes exist.
- No HN discussion was found specifically about cloud sandboxes or multiple machines syncing agent sessions through git, beyond ccgs's path rewriting and Oak's mounts.

## Q3. Additional projects surfaced in the threads (not in the given list)

### Takeaway

The threads surface many small tools, most of them one-person projects with little traction. They fall into a few groups: git-notes or branch-based session capture (git-memento, cnotes, claudit, y, ai-session), attribution and blame (agentblame, Selvedge), local session search (ctx, obliscence, clancey), git-native memory (DiffMem, OKF Agent Memory), new forges or version control systems (Cursor Origin, Oak, Tangled, Zed Delta) and versioned databases (Dolt/DoltLite).

### Cited Findings

- git-memento: attaches cleaned markdown AI session transcripts to commits as git notes — [repo](https://github.com/mandel-macaque/memento); [HN 47212355](https://news.ycombinator.com/item?id=47212355)
- cnotes: a Claude hook that writes conversations to git notes, with a Chrome extension to show them on GitHub. The author stopped using it — [repo](https://github.com/imjasonh/cnotes); [HN 46962431](https://news.ycombinator.com/item?id=46962431)
- claudit: a weekend project similar to Entire Checkpoints, with Gemini/OpenCode support in progress — [repo](https://github.com/re-cinq/claudit); [HN 46975536](https://news.ycombinator.com/item?id=46975536)
- y (eqtylab): a hackathon prototype for intent provenance. A commit hook has a background agent summarise the work into a git note — [repo](https://github.com/eqtylab/y); [HN 46962485](https://news.ycombinator.com/item?id=46962485)
- ai-session: copies Claude sessions into a separate repo and sanitises API keys — [repo](https://github.com/gammons/ai-session); [HN 47218987](https://news.ycombinator.com/item?id=47218987)
- agentblame (Mesa): an open-source tool that tracks agent sessions with commits, offered as an alternative to Entire — [repo](https://github.com/mesa-dot-dev/agentblame); [HN 46961911](https://news.ycombinator.com/item?id=46961911)
- Selvedge: an MCP server that captures why AI agents change code. Commenters contrasted it with git-ai's git-notes approach — [HN 48057104](https://news.ycombinator.com/item?id=48057104); [HN 48057127](https://news.ycombinator.com/item?id=48057127)
- ctx: a Rust CLI that ingests local agent transcripts into SQLite with ranked text search. Its author calls for a standard transcript format — [repo](https://github.com/ctxrs/ctx); [HN 48763577](https://news.ycombinator.com/item?id=48763577)
- obliscence: another local agent-history search tool, refreshing on a SessionStart hook, with semantic search — [repo](https://github.com/beaugunderson/obliscence); [HN 48770886](https://news.ycombinator.com/item?id=48770886)
- clancey: conversation lookup across past agent sessions ("find this conversation we talked about yesterday") — [repo](https://github.com/divmgl/clancey/); [HN 48066733](https://news.ycombinator.com/item?id=48066733)
- pi-brains: a Pi agent extension adding "brain checkpoints" to debug AI reasoning after a task — [repo](https://github.com/gitsense/pi-brains); [HN 48845733](https://news.ycombinator.com/item?id=48845733)
- recursive-mode: workflow run docs for traceability, which can double as fine-tuning data — [site](https://recursive-mode.dev/introduction); [HN 48069046](https://news.ycombinator.com/item?id=48069046)
- ticket (tk): a single-file bash and markdown task tracker that replaced Beads for one long-running-agent user — [repo](https://github.com/wedow/ticket); [HN 46487580](https://news.ycombinator.com/item?id=46487580)
- git-issue: a git-based issue tracker suggested as an alternative to Beads or ticket — [repo](https://github.com/dspinellis/git-issue); [HN 46508533](https://news.ycombinator.com/item?id=46508533)
- DiffMem: AI memory as markdown files in a git repo. Each conversation is a commit, with BM25 search and git diff to track evolving facts — [repo](https://github.com/Growth-Kinetics/DiffMem); [HN 44969622](https://news.ycombinator.com/item?id=44969622)
- OKF Agent Memory: a zero-dependency Go binary that turns a git repo into a knowledge corpus with in-memory BM25. A commenter criticised it for benchmarking latency, not recall — [repo](https://github.com/okf-memory/okf-agent-memory); [HN 49581240](https://news.ycombinator.com/item?id=49581240); [HN 49583839](https://news.ycombinator.com/item?id=49583839)
- Cursor Origin: Cursor's GitHub-style code host, in paid beta, built by ex-Graphite staff, with agent features promised — [changelog](https://cursor.com/changelog/origin-code-hosting); [HN 49334209](https://news.ycombinator.com/item?id=49334209)
- Oak: a new version control system for agents, with lazy virtual mounts, messageless intermediate commits and no separate LFS — [site](https://oak.space/oak/oak); [HN 48631726](https://news.ycombinator.com/item?id=48631726)
- Zed Delta: replaces PRs with shared agent "threads" over DeltaDB, a hosted service. It supports ACP agents including Claude Code and Codex — [site](https://delta.dev/); [HN 49727245](https://news.ycombinator.com/item?id=49727245); [HN 49187529](https://news.ycombinator.com/item?id=49187529)
- Tangled: an ATProto-based forge where git, issues, PRs and CI can be self-hosted — [site](https://tangled.org/); [HN 49339647](https://news.ycombinator.com/item?id=49339647)
- Forgejo federation: federated forge roadmap, suggested as the alternative to centralised agent forges — [roadmap](https://codeberg.org/forgejo-contrib/federation/src/branch/main/FederationRoadmap.md); [HN 49348841](https://news.ycombinator.com/item?id=49348841)
- Dolt / DoltLite: versioned SQL databases. Per a commenter, Beads' storage is Dolt. DoltLite is a SQLite fork with git-style branching and merging — [blog](https://www.dolthub.com/blog/2026-08-31-doltlite-beta/); [HN 49516848](https://news.ycombinator.com/item?id=49516848); [HN 48846446](https://news.ycombinator.com/item?id=48846446)
- vcbench: a benchmark of git, jj and GitButler used by agents (from GitButler) — [site](https://vcbench.dev/); [HN 48847192](https://news.ycombinator.com/item?id=48847192)
- jj-benchmark (TabbyML): evaluates AI agents on Jujutsu — [site](https://tabbyml.github.io/jj-benchmark/); [HN 47352189](https://news.ycombinator.com/item?id=47352189)
- jujutsu-skill: an agent skill (command reference) so agents can drive jj — [repo](https://github.com/danverbraganza/jujutsu-skill); [HN 47771815](https://news.ycombinator.com/item?id=47771815)
- replicated.live: CRDT merges inside git for code plus markdown metadata — [blog](https://replicated.live/blog/meta); [HN 49845533](https://news.ycombinator.com/item?id=49845533)
- Codex ghost snapshots (PR 6041): git as a hidden backend for agent file states, kept out of user history — [PR](https://github.com/openai/codex/pull/6041); [HN 48066489](https://news.ycombinator.com/item?id=48066489)
- Rudel: Claude Code session analytics — [repo](https://github.com/obsessiondb/rudel); story found via Algolia: "Show HN: Rudel" (2026-03-12, 144 points) [HN 47350416](https://news.ycombinator.com/item?id=47350416)
- claude-replay: a video-like player for Claude Code sessions — [repo](https://github.com/es617/claude-replay); [HN 47276604](https://news.ycombinator.com/item?id=47276604) (105 points)
- Claudebin: share and resume Claude Code sessions with a link — [site](https://claudebin.com/); [HN 47073488](https://news.ycombinator.com/item?id=47073488)
- Grov: persistent memory for Claude Code sessions — [repo](https://github.com/TonyStef/Grov); [HN 46126066](https://news.ycombinator.com/item?id=46126066)
- claude-devtools: a local log viewer for Claude Code session logs — [repo](https://github.com/matt1398/claude-devtools); [HN 47004712](https://news.ycombinator.com/item?id=47004712)
- cli-continues: resumes the same session across Claude, Gemini and Codex — [repo](https://github.com/yigitkonur/cli-continues); [HN 47075089](https://news.ycombinator.com/item?id=47075089)
- CodeAlmanac: a codebase wiki built from your agent conversations — [repo](https://github.com/AlmanacCode/codealmanac/); [HN 48995181](https://news.ycombinator.com/item?id=48995181)
- wuphf: an LLM wiki agents maintain, in Markdown and git — [repo](https://github.com/nex-crm/wuphf); [HN 47899844](https://news.ycombinator.com/item?id=47899844) (260 points)
- Traces: share and discover agent traces — [site](https://www.traces.com); [HN 47277436](https://news.ycombinator.com/item?id=47277436)
- PearSync sessions: an example of a project publishing all its agent sessions in-repo — [file](https://github.com/kzahel/PearSync/blob/main/sessions/sessions.md); [HN 47214945](https://news.ycombinator.com/item?id=47214945)
- spec-kit (GitHub): spec-driven development; often named as the "commit the spec, not the session" pattern — [repo](https://github.com/github/spec-kit); [HN 47214827](https://news.ycombinator.com/item?id=47214827)
- Packmind OSS: versioning and governance of AI coding context — [repo](https://github.com/PackmindHub/packmind); [HN 45836219](https://news.ycombinator.com/item?id=45836219)
- PDD (Prompt-Driven Development): the prompt as the source of truth for code — [repo](https://github.com/promptdriven/pdd); [HN 44935941](https://news.ycombinator.com/item?id=44935941)
- Universal Memory Protocol: a proposed shared format for agent memory — [site](https://universalmemoryprotocol.io/); [HN 48428796](https://news.ycombinator.com/item?id=48428796)
- funes (Hugging Face) and mempalace: suggested as cross-project agent memory — [funes](https://github.com/huggingface/funes); [mempalace](https://github.com/mempalace/mempalace); [HN 49582092](https://news.ycombinator.com/item?id=49582092)

### Inferences

- Many people have independently built "Claude session JSONL → git notes or branch" tools, and several abandoned theirs. The capture mechanism is easy; the hard and unsolved parts are retrieval, redaction and an agreed format.
- The repeated calls for a standard transcript format (ctx author, Agent Trace RFC, Universal Memory Protocol) suggest an opening for ParleyInn to adopt or emit an open, documented record format rather than a proprietary one.

### Gaps

- Most of these projects were not independently checked beyond the HN description. Maturity, licences and activity are unknown.
- The Selvedge repository URL was not captured from the thread. Only the HN link is given.

## Q4. Where community consensus sits, and the strongest dissent

### Takeaway

Consensus has four parts:

- Do not put raw agent transcripts into the main git history or PRs, and do not make humans read them.
- Do commit the distilled intent: plans, specs, ADRs and good commit messages.
- If full sessions are kept, keep them out-of-band (git notes, a side branch, a separate private repo or a local database), linked by ID, private by default and redacted.
- Be sceptical of new version control systems "for agents": git (or jj on top of it) is good enough, and models already know it.

Three dissents are strong. One says sessions are the new source or the "most important artifact" and should be stored losslessly (Entire, dang's Show HN proposal, provenance advocates). One says session linkage is professional attribution and an audit trail. The third, from a different angle, is privacy and surveillance objection to hosted session capture.

### Cited Findings

- The top-voted comments in the main thread oppose raw sessions. The two most-replied top-level comments argue for spec/plan files over transcripts (53 and 50 replies) — [HN 47214007](https://news.ycombinator.com/item?id=47214007); [HN 47214629](https://news.ycombinator.com/item?id=47214629)
- In the Claude session-URL thread, by contrast, the most-replied top-level comment supports the default (88 replies). A reply calls the positivity so strong "I'd think it's astroturfing", so the thread is split, not one-sided — [HN 49498447](https://news.ycombinator.com/item?id=49498447); [HN 49499267](https://news.ycombinator.com/item?id=49499267)
- The Entire thread was "extremely negative", in its own top commenter's words. Much of the negativity was about the $60M seed and the marketing rather than the idea, with replies asking what moat a git-plus-JSONL tool has — [HN 46968612](https://news.ycombinator.com/item?id=46968612); [HN 46970697](https://news.ycombinator.com/item?id=46970697); [HN 46966990](https://news.ycombinator.com/item?id=46966990)
- A TechCrunch article linked in the thread reports a $60M seed at a $300M valuation (not independently verified) — [HN 47213218](https://news.ycombinator.com/item?id=47213218); [TechCrunch URL as linked](https://techcrunch.com/2026/02/10/former-github-ceo-raises-record-60m-dev-tool-seed-round-at-300m-valuation/)
- Entire's strongest form of the dissent: "session logs are now the most important artifact in software development, and should be stored alongside the code itself in the repository" (quoted from Entire's blog) — [HN 48847525](https://news.ycombinator.com/item?id=48847525)
- dang planned to require prompts with Show HNs of generated projects: "we're going to implement this unless we hear strong reasons not to". Objections centred on feasibility ("thousands and thousands of prompts to hundreds of agents") — [HN 47096202](https://news.ycombinator.com/item?id=47096202); [HN 47096685](https://news.ycombinator.com/item?id=47096685)
- "YES! The session becomes the source code" (burntoutgray) was rebutted on determinism grounds — [HN 47213028](https://news.ycombinator.com/item?id=47213028); [HN 47213262](https://news.ycombinator.com/item?id=47213262)
- Some argue the reader is a future agent, not a human, so noise matters less — [HN 47214600](https://news.ycombinator.com/item?id=47214600). The rebuttal: "It's just noise for AI too… ask the AI to write the summary of the session" — [HN 47219884](https://news.ycombinator.com/item?id=47219884)
- vpribish suggests a middle path: keep the commit message human-dense, "and also have a development process flight-recorder log stored alongside. Storage is basically free so why not?" — [HN 47220090](https://news.ycombinator.com/item?id=47220090)
- Privacy and surveillance dissent is strongest against Zed: "I don't want my thoughts to be serialized, version controlled and publicly accessible" (87 replies) — [HN 48494076](https://news.ycombinator.com/item?id=48494076). It is countered by pro-history voices: "This is why I use rebase before PRs, and despise squash" — [HN 48494563](https://news.ycombinator.com/item?id=48494563)
- On new version control systems, the "show me why not git" view dominates the Oak and re_gent threads. jj gets the most sympathy as an evolution that stays git-compatible — [HN 48635235](https://news.ycombinator.com/item?id=48635235); [HN 48064856](https://news.ycombinator.com/item?id=48064856); [HN 48632284](https://news.ycombinator.com/item?id=48632284)
- weinzierl dissents the other way: "I also think git is an ill-fit for the majority of modern commercial software projects and there will be a breaking point". Replies ask for specifics — [HN 48632198](https://news.ycombinator.com/item?id=48632198); [HN 48632223](https://news.ycombinator.com/item?id=48632223)
- The git-ai author describes it as "decidedly not trying to replace the SCMs". Teams are building review tools "on top of the standard" — [HN 46963025](https://news.ycombinator.com/item?id=46963025)
- Commenters doubt agent-vendor features last. "Either the models are good and this sort of platform gets swept away, or they aren't, and this sort of platform gets swept away" — [HN 46966676](https://news.ycombinator.com/item?id=46966676). Others say OpenAI or Anthropic "could easily spin up a competitor internally" — [HN 46966541](https://news.ycombinator.com/item?id=46966541)

### Inferences

- For ParleyInn, the community view favours keeping the lossless record (dissenters and provenance advocates value it). It also favours keeping that record out of the human-facing history, private by default, redacted before any sharing, and reachable by a stable ID from commits. Using git as a transport for a separate ref namespace (orphan branch or notes) fits that consensus. Writing into the working tree or the main branch does not.
- Vendor-neutral, self-hosted and open-format attributes count heavily with HN readers. Distrust of Cursor/SpaceX, Zed's hosted service and Claude session URLs suggests ParleyInn's "never talks to the network" invariant (I4) could be a selling point. That holds as long as any git push is something the user explicitly sets up, not something ParleyInn does itself.
- Few commenters raise the risk of stored history becoming a prompt-injection channel, but memory-poisoning complaints are common. ParleyInn can frame I2 (pull-only, enveloped recall) as the fix for "one poisoned line affects everything downstream".

### Gaps

- HN commenters are self-selected and skew sceptical of AI tooling. Several threads include accusations of LLM-written comments and astroturfing (for example [HN 46969715](https://news.ycombinator.com/item?id=46969715) and [HN 49499267](https://news.ycombinator.com/item?id=49499267)), so vote-weighted "consensus" should be read as a sentiment signal, not a measurement.
- I found no HN thread reporting long-term, team-scale results (for example a year of sessions in git notes or branches across many agents and machines). The evidence on durability and value is anecdotal.
