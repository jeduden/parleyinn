# Phone reach and live delivery without a central service

Scope: two questions raised by the stakeholder's new requirement of
4 October 2026. The first is how a phone app reaches the owner's own
ParleyInn nodes with no central or vendor service. The second is how a
person's chat message reaches a running, possibly idle harness session
that the person started in their own terminal, not ParleyInn. The
requirement asks for the same "start ParleyInn, then you see your stuff"
on Linux, Windows, macOS, iOS and Android. The phone must show the
owner's rooms, chat and approve, and the UI must stay one more client,
never a required hop. Sources were read on 4 October 2026: vendor
documentation, specifications, repositories at named commits and
package registries. Topics already covered elsewhere are cited, not
repeated:

- transports, iroh, Tailscale `tsnet`, Headscale and hardware key
  stores: [protocol components §5 and §6](protocol-and-data-components.md);
- PWA install, web push, Periodic Background Sync and secure contexts
  on phones: [UI components](ui-and-packaging-components.md);
- the stacks of Happy, Omnara, T3 Code, Block Buzz and Remote Control:
  [prior art](prior-art-stacks.md) and the [T3 Code note](../t3code/t3code.md);
- which hook points each harness offers:
  [room protocol](../../../plan/2610012322_parinn-for-agent-fleets/room-protocol.md).

"(unverified)" marks a claim no primary source confirmed;
"(inference)" marks a conclusion drawn from cited facts.

## Key findings

1. **The new requirement goes past three SRS rows.** A phone
   credential may only read, allow once and deny (OWN-16), and a paired
   phone may only allow or deny (OWN-17, PRV-10). Chatting from the
   phone is steering, a new phone scope that needs an I2 review
   (ENG-29). Row X2 rejects a phone view served over B2. Off the LAN,
   a phone therefore sees rooms either through the owner's tunnel
   (row 11) or as a full peer that renders sealed ranges itself
   ([§6.3](../../../docs/srs/06-security.md),
   [§5c](../../../docs/srs/05c-owner-and-peer-requirements.md)).
2. **Every off-LAN method needs one node the phone can reach.** With
   no central service, that node is the owner's: a home server with a
   public address, a host they rent, or a coordination server they run.
   Hole punching saves relay traffic but still needs a rendezvous, and
   it fails when both NATs map per destination (inference).
3. **iOS has no remote wake-up except APNs.** A device token is unique
   to the device and the app, and the signing key comes from the
   publisher's developer account. A self-hosted server therefore cannot
   push to an App Store build. Local Push Connectivity works only on
   Wi-Fi networks, private LTE or Ethernet the app names, behind an
   entitlement Apple grants. Under VIEW-17 an iPhone away from home
   learns of a held request only when the app runs (inference).
4. **Android can meet VIEW-17's intent, but not its words.** UnifiedPush
   through an ntfy server the owner runs, or the app's own foreground
   connection, wakes the phone with no vendor. VIEW-17 lists no phone
   notification at all, and X4 rejects "Web Push", the protocol
   UnifiedPush servers speak. Row 22 already admits self-hosted push.
5. **Claude Code now has two local paths into an idle session it did
   not start.** Channels let an MCP server over stdio push events in;
   they are a research preview behind an allowlist and a per-session
   `--channels` flag. The cross-session inbox, shipped since v2.1.224,
   is a per-session Unix socket or named pipe for the same OS user. A
   message arriving there starts a new turn in an idle session.
   Neither path is on OWN-03's list. Channel text reaches the model in
   a `<channel>` tag. Inbox text is labelled as coming from another
   session, not from the person.
6. **Only channels and `asyncRewake` hooks fit B0 unchanged.** Both
   use stdio and files. The inbox socket, Codex's daemon socket and
   OpenCode's HTTP server each need a B1 component and a register row.
7. **Codex reaches a user-started TUI only through its local daemon.**
   When `codex app-server daemon` runs, a plain `codex` attaches to it,
   and another client can call `turn/start`, `turn/steer` or the
   experimental `thread/queue/add`. Without the daemon the TUI embeds
   its server and nothing outside the process can reach it.
8. **Gemini CLI and ACP offer no way into an idle session started
   elsewhere.** Gemini's `AfterAgent` hook turns a reason into a new
   prompt only when a turn ends. ACP's multi-client `session/attach`
   is an open proposal.

## Part 1: the phone reaching the owner's nodes

### What the phone has to do, against the SRS

- **See rooms.** Rooms live in writer logs on the owner's nodes. Two
  shapes exist (inference):
  - a thin client that renders the lane view a node serves, which the
    SRS allows only through the owner's own tunnel (row 11) and rejects
    over B2 (X2);
  - a phone that holds sealed ranges as a peer (PEER-02) and derives
    room state itself. PEER-03 then requires byte-identical state, so
    the phone runs the same projection code as the node.
- **Chat.** A reply or steer is an owner act (OWN-02). The phone's
  scope today stops at allow and deny (OWN-16, OWN-17), so chat needs
  a new scope and a recorded I2 review (PRV-10, ENG-29).
- **Approve.** Already in scope: allow once or deny, with the phone's
  own presence check bound to the answer (OWN-16).
- **Notify.** VIEW-17 forbids a vendor push service; X4 rejects Web
  Push and vendor relays; X7 rejects a ParleyInn relay host, but "any
  self-hosted node can be the reachable peer"
  ([§6.3](../../../docs/srs/06-security.md)).

### Same-LAN discovery

- **Rules.** Local discovery may announce only "a random per-boot
  instance id and a port" (SEC-24) and never enrolls (PEER-06).
- **iOS.** `NWBrowser` browses Bonjour services from iOS 13
  ([NWBrowser](https://developer.apple.com/documentation/network/nwbrowser)).
  The first local-network operation shows a permission alert. An app
  that browses Bonjour services lists their types in `NSBonjourServices`,
  and raw multicast needs the `com.apple.developer.networking.multicast`
  entitlement
  ([TN3179](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy)).
  - "A local network is an IP network associated with a
    broadcast-capable network interface. Such interfaces include Wi-Fi
    and Ethernet, but not cellular (WWAN) or VPN" (TN3179). Traffic
    over the owner's VPN needs no local-network permission.
  - "Traffic originating from WKWebView, SFSafariViewController, and
    Safari doesn't require local network access" (TN3179), so a page
    in Safari can reach a LAN address without the alert.
- **Android.** Android 17 enforces local network protection for apps
  targeting SDK 37, behind the `ACCESS_LOCAL_NETWORK` runtime
  permission; Android 16 offered it as an opt-in. `NsdManager` is
  covered, except when the app lets the system picker choose
  (`DiscoveryRequest.FLAG_SHOW_PICKER`)
  ([local network permission](https://developer.android.com/privacy-and-security/local-network-permission)).
- **Libraries for the node side.**
  - Go: `hashicorp/mdns` v1.0.7, 11 June 2026, MIT
    ([proxy](https://proxy.golang.org/github.com/hashicorp/mdns/@latest),
    [LICENSE](https://github.com/hashicorp/mdns/blob/main/LICENSE));
    `brutella/dnssd` v1.2.14, 22 October 2024, MIT
    ([proxy](https://proxy.golang.org/github.com/brutella/dnssd/@latest));
    `grandcat/zeroconf` v1.0.0 dates from January 2020
    ([proxy](https://proxy.golang.org/github.com/grandcat/zeroconf/@latest)).
  - Rust: `mdns-sd` 0.21.4, 21 September 2026, Apache-2.0 OR MIT
    ([crates.io](https://crates.io/crates/mdns-sd)).
- **Fit.** Discovery finds a node at home; it does not reach one away
  from home. It belongs to the peer component (row 13).

### The owner's own tunnel or VPN

- **WireGuard.** The official apps are MIT on Apple platforms and
  Apache-2.0 on Android
  ([wireguard-apple](https://github.com/WireGuard/wireguard-apple/blob/master/COPYING),
  [wireguard-android](https://github.com/WireGuard/wireguard-android/blob/master/COPYING)).
  WireGuard gives keys and a tunnel but no NAT traversal, so the owner
  needs an endpoint with a public address or a forwarded port
  ([protocol components](protocol-and-data-components.md)).
- **One tunnel at a time on iOS.** "Only one Personal VPN configuration
  can be enabled simultaneously on the system"
  ([NEVPNManager.isEnabled](https://developer.apple.com/documentation/networkextension/nevpnmanager/isenabled)).
  Packet-tunnel apps such as WireGuard and Tailscale subclass the same
  manager, so the phone cannot run the owner's tunnel beside a work VPN
  (inference).
- **Tailscale with Headscale.** The official iOS app has a "Use custom
  coordination server" option
  ([Headscale: Apple](https://headscale.net/stable/usage/connect/apple/)).
  - Headscale's embedded DERP relay "is disabled by default", and by
    default Headscale adds its relay to Tailscale's public DERP map.
    Using only the owner's relay means setting `urls: []`, which the
    docs warn "is a single point of failure"
    ([Headscale: DERP](https://headscale.net/stable/ref/derp/)).
  - Licenses: the Android app is BSD-3-Clause
    ([tailscale-android](https://github.com/tailscale/tailscale-android/blob/main/LICENSE)).
    "where the operating system is closed, the daemon is open source
    and the GUI is closed source"
    ([Tailscale open source](https://tailscale.com/opensource)), so
    the iOS app is closed (inference).
  - Cost to the owner: a coordination server and a relay to keep up,
    beside the ParleyInn node (inference).
- **Tailscale's own service.** Its coordination server and relays are
  Tailscale's. Row 11 leaves the tunnel to the owner, so an owner may
  choose it; CON-06 forbids ParleyInn to depend on it (inference).
- **Fit.** This is stage one as written (row 11): the phone opens the
  loopback lane view with a phone-scoped credential (OWN-16). It shows
  rooms only while the tunnel is up and the page is open.

### iroh with a self-hosted relay and discovery

- **Mobile bindings.** iroh-ffi publishes Swift through SwiftPM and
  CocoaPods (`IrohLib`) and builds an xcframework for iOS and macOS.
  Its Maven artifact is listed as "Kotlin / JVM"
  ([iroh-ffi](https://github.com/n0-computer/iroh-ffi)). The Kotlin
  readme tells Android developers to build with the NDK
  ([README.kotlin](https://github.com/n0-computer/iroh-ffi/blob/main/README.kotlin.md)),
  so no prebuilt Android library was found (inference). The UniFFI
  runtime is MPL-2.0 ([protocol components](protocol-and-data-components.md)).
- **Self-hosted relay.** `iroh-relay` runs from a TOML file, binds
  ports 80 and 443, and needs a public hostname for an ACME
  certificate. "Documentation of all possible configuration options
  for the relay is not published currently"
  ([self-hosted relays](https://docs.iroh.computer/iroh-services/relays/self-hosted)).
- **NAT traversal.** n0 describes its QUIC hole punching in an
  individual draft of July 2026 that is "not yet a specification"
  ([draft-bruynooghe-n0-quic-nat-traversal](https://datatracker.ietf.org/doc/draft-bruynooghe-n0-quic-nat-traversal/)).
- **Fit.** A B2 transport for a phone that is a peer, with the
  owner's always-on node as relay: one hop through an enrolled peer
  (PEER-04). It brings Rust onto the phone (inference).

### Direct QUIC with hole punching

- **Platform support.** Apple's Network framework has had QUIC since
  iOS 15
  ([NWProtocolQUIC](https://developer.apple.com/documentation/network/nwprotocolquic)).
  Go and Rust QUIC stacks are in
  [protocol components](protocol-and-data-components.md).
- **Limits.** Tailscale's account of NAT traversal falls back to "a
  relay that both sides can talk to unimpeded" when both sides sit
  behind NATs whose mapping depends on the destination
  ([how NAT traversal works](https://tailscale.com/blog/how-nat-traversal-works)).
  Carrier-grade NAT on phones makes that case common (unverified).
- **Fit.** ParleyInn would write its own rendezvous on an enrolled peer
  with a public address; the direct path only saves relay bandwidth
  (inference).

### An owner-run relay on the home server

- The always-on node is the reachable peer that X7 allows, relaying
  sealed ranges one hop (PEER-04) or storing them blind (PEER-12).
- It needs a public address: IPv6, a forwarded port, or a small host
  the owner rents. The phone dials out to it, which every platform
  allows while the app runs (inference).
- Happy's self-hosted server is the friendliest prior form, one process
  with PGlite ([prior art](prior-art-stacks.md)). Its push sender
  still posts to Expo's service at `exp.host`
  ([pushSend.ts](https://github.com/slopus/happy/blob/dafe9a5/packages/happy-server/sources/app/push/pushSend.ts)),
  so self-hosting moves the relay but not the push path (inference).

### Owner-run Nostr and Matrix relays

- **Block Buzz** promises rooms "on a relay you own"
  ([prior art](prior-art-stacks.md)). Its iOS notifications go through
  `buzz-push-gateway`, "a standalone APNs last hop" holding the app's
  APNs certificate. The current build serves exactly one compiled-in
  profile, `buzz-ios-dogfood`
  ([push gateway deployment](https://github.com/block/buzz/blob/f0eb557/docs/push-gateway-deployment.md)).
  The relay can be the owner's; the iOS push hop is the publisher's.
- **Matrix.** Sygnal, "a reference Push Gateway for Matrix", supports
  APNs and GCM/FCM, under AGPL-3.0 or a commercial license
  ([Sygnal](https://github.com/element-hq/sygnal)).
- **Fit.** Either adds a second server protocol beside sealed segments
  and still meets the same iOS push wall (inference).

### iOS limits

- **Background execution.** `BGAppRefreshTask` runs when "the system
  decides the best time" and gives "up to 30 seconds of background
  runtime"; work continued from the foreground gets "a limited amount
  of time"
  ([background strategies](https://developer.apple.com/documentation/backgroundtasks/choosing-background-strategies-for-your-app)).
  An app cannot keep its own connection open in the background
  (inference).
- **Remote push is APNs.** A device token is "unique to both the device
  and your app"
  ([registering with APNs](https://developer.apple.com/documentation/usernotifications/registering-your-app-with-apns)).
  The provider signs with a key requested "from your developer account"
  ([token-based connection](https://developer.apple.com/documentation/usernotifications/establishing-a-token-based-connection-to-apns)).
  Only the app's publisher, or an owner who builds and signs their own
  copy, can push to it (inference).
- **Projects that tried.** UnifiedPush: "iOS doesn't support running
  services in the background, so running a UnifiedPush distributor
  won't be possible" ([FAQ](https://unifiedpush.org/users/faq/)).
  ntfy calls instant push on iOS "impossible ... without a central
  server": a self-hosted ntfy forwards `poll_request` messages to
  `ntfy.sh`, which forwards them to APNs
  ([ntfy config](https://github.com/binwiederhier/ntfy/blob/main/docs/config.md)).
- **Local Push Connectivity.** Since iOS 14, an `NEAppPushProvider`
  extension keeps "a persistent network connection to a server" where
  "an iOS device operates on a local, restricted network that doesn't
  have access to APNs". It shows alerts as local notifications. It
  needs the `app-push-provider` entitlement, which the developer
  requests from Apple
  ([Local Push Connectivity](https://developer.apple.com/documentation/networkextension/local-push-connectivity)).
  It activates on `matchSSIDs`, `matchPrivateLTENetworks`,
  `matchEthernet` or `matchMissionCriticalService`, never on public
  cellular
  ([NEAppPushManager](https://developer.apple.com/documentation/networkextension/neapppushmanager)).
- **Local notifications.** The app may raise one whenever it runs:
  in the foreground, in a refresh window, or from the push provider
  above. Away from home, timing is the system's (inference).

### Android limits

- **UnifiedPush.** "a decentralized push notification system that lets
  you choose the service you want to use", "Self-hostable", with ntfy,
  NextPush and Sunup as distributors
  ([UnifiedPush](https://unifiedpush.org/)). The app server must
  "support Web Push" ([FAQ](https://unifiedpush.org/users/faq/)).
- **ntfy without Google.** Instant delivery uses "a foreground service",
  which works "even when your phone is in doze mode". The app "won't
  use Firebase for any self-hosted servers"
  ([ntfy phone docs](https://github.com/binwiederhier/ntfy/blob/main/docs/subscribe/phone.md)).
- **Foreground service caps.** From Android 15, `dataSync` and
  `mediaProcessing` services may run "6 hours total in any 24-hour
  period"
  ([timeouts](https://developer.android.com/develop/background-work/services/fgs/timeout)).
  A ParleyInn app syncing under `dataSync` would be stopped; which service
  type ntfy declares was not checked (unverified).
- **Local network.** `ACCESS_LOCAL_NETWORK` from Android 17, as above.

### What this means for VIEW-17

- **Android.** An owner-run ntfy reached through UnifiedPush is a
  self-hosted push service, which row 22 already admits, carrying
  "alias, class and count only" (SEC-28). VIEW-17's list of allowed
  signals and X4's "Web Push" would need words that tell an owner-run
  push server from a vendor's (inference).
- **iOS.** Without APNs an iPhone gets: signals while the app is open;
  Local Push Connectivity on the home Wi-Fi, if Apple grants the
  entitlement; and local notifications in refresh windows the system
  picks (inference). Timely notifications away from home need one of
  these:
  - APNs through a gateway ParleyInn's publisher runs, a central service
    that CON-06 forbids;
  - APNs through the owner's own Apple developer account and own build
    of the app;
  - an upstream relay such as `ntfy.sh`, a vendor relay that X4
    rejects.
- **Same UX.** "The same UX on every platform" cannot hold for
  notifications on iOS under VIEW-17. OWN-15 already requires showing
  an unavailable control with its reason rather than simulating it
  (inference).

### What the prior art does

[Prior art](prior-art-stacks.md) found that "Every phone path is a
relay": Claude Remote Control and Codex through their vendors, Happy
through an encrypted relay, T3 Code and Superset through Cloudflare.
T3 Code also pairs on the LAN and supports Tailscale Serve and SSH
([T3 Code note](../t3code/t3code.md)). Added here:

- Happy pushes through Expo, Buzz through its own APNs gateway, and
  Remote Control through the Claude app after `/config` "Push when
  Claude decides" or "Push when actions required"
  ([Remote Control](https://code.claude.com/docs/en/remote-control)).
- Codex Remote pairs the ChatGPT desktop app with the ChatGPT mobile
  app under one ChatGPT account
  ([Codex Remote](https://learn.chatgpt.com/docs/remote)).
- No surveyed product notifies an iPhone without APNs.

## Part 2: live delivery into a session ParleyInn did not start

### What the SRS asks of a delivery path

- **OWN-03.** Owner text reaches an agent "only through the harness's
  own input interface (the Agent SDK, ACP, the Codex app-server, or the
  terminal the run component hosts), never through ParleyInn hook output".
- **I2.** With an owner act, ParleyInn writes "through the harness's own
  input, only owner-typed text, a fixed template that references ids,
  or a post a principal endorsed". Recall is pull-only and enveloped.
- **I4.** B0 opens no socket. A stdio MCP server and hooks are B0
  (rows 1 and 2); the run component (B1) "reaches the lane view only
  over loopback or an endpoint only the same local user can reach; no
  outbound" (row 10, SEC-29).
- **OWN-15.** A control an adapter cannot honour is shown unavailable
  with its reason, never simulated.

### Claude Code channels

- **What.** "A channel is an MCP server that pushes events into your
  running Claude Code session". "Claude Code spawns it as a subprocess
  and communicates over stdio"
  ([channels reference](https://code.claude.com/docs/en/channels-reference)).
- **How.** The server declares `capabilities.experimental['claude/channel']`
  and emits `notifications/claude/channel` with `content` and string
  `meta`. The model sees `<channel source="..." ...>content</channel>`.
  The server's `instructions` string reaches Claude "as context when
  the server connects". In the webhook example an event arriving at an
  idle session makes Claude "start responding". Events that arrive
  while Claude is busy are "delivered together on the next turn"
  (channels reference).
- **Acknowledgement.** "Claude Code doesn't acknowledge notifications";
  if the session has not loaded the server as a channel, "Claude Code
  drops the events silently" (channels reference). ParleyInn would have
  to record delivery itself (I6, inference).
- **Permission relay.** Declaring `claude/channel/permission` makes
  Claude Code send `notifications/claude/channel/permission_request`
  (a five-letter `request_id`, tool name, description, input preview).
  The server answers `notifications/claude/channel/permission` with
  `allow` or `deny`; the first answer, terminal or remote, wins
  (channels reference). This matches the phone's allow-or-deny scope
  (inference).
- **Status and gates.**
  - "Channels are in research preview"; "the `--channels` flag syntax
    and protocol contract may change"
    ([channels](https://code.claude.com/docs/en/channels)).
  - "no channel runs until a user opts it in for the session with
    `--channels`"; "Being in `.mcp.json` isn't enough" (channels).
  - Only plugins on Anthropic's allowlist, or an organisation's
    `allowedChannelPlugins`, register. Others need
    `--dangerously-load-development-channels`, which first shows "a
    full-screen warning dialog" (channels reference). Whether the
    dialog returns at every launch is unverified.
  - Channels need claude.ai or Console authentication and are absent
    on Bedrock, Google Cloud and Foundry; Team and Enterprise owners
    must enable them (channels).
  - "a channel server that negotiates MCP protocol revision 2026-07-28
    can't deliver channel messages"
    ([MCP](https://code.claude.com/docs/en/mcp)). A channel stays on
    the older handshake.
- **Fit.** ParleyInn's B0 MCP server could watch its store files and emit
  an owner message as a channel event: no socket, idle sessions woken.
  The text arrives as an MCP event, not as the harness's own user
  input, so OWN-03 and I2's closed list need amending (inference). The
  user's own `claude` command must carry the flag, which `parinn
  install` could add as a shown change under I7 (inference).

### The Claude Code cross-session inbox

- **What.** "Claude Code binds an inbox socket for each session with
  cross-session messaging enabled": a Unix domain socket on macOS and
  Linux, a named pipe on Windows. It needs v2.1.224, or v2.1.234 on
  native Windows, and is "on with nothing to enable"
  ([cross-session messaging](https://code.claude.com/docs/en/cross-session-messaging)).
- **Idle sessions.** "When the receiving session is idle, Claude Code
  starts a new turn with the message." Same-machine messages travel
  "never through Anthropic servers" and work on every provider.
- **Who may post.** On macOS and Linux the socket is limited to the
  same OS user; Windows requires an auth line with
  `CLAUDE_CODE_MESSAGING_TOKEN`. The path and token are exported to
  hooks and Bash commands
  ([env vars](https://code.claude.com/docs/en/env-vars)). The agent's
  own Bash commands can therefore post too, which is T21's same-user
  case (inference).
- **Trust.** "Claude Code tells B's Claude that the message came from
  another session, not from you". A message "can't approve anything",
  "can't change configuration", and slash commands in it "arrive as
  plain text". A session that bypasses permission prompts holds an
  unverified sender's message for approval, as does the setting
  `crossSessionInbound: hold` (cross-session messaging).
- **Wire format.** Only the auth line,
  `{"type":"auth","token":"<token>"}`, is documented; the message line
  that follows is not (unverified).
- **Fit.** This is the harness's own input and it wakes idle sessions.
  It labels the person's words as another session's, which lowers
  their authority rather than raising it. It is a socket, so the
  sender must be B1 with a new register row. A ParleyInn `SessionStart`
  hook could record the exported path in the store for that sender
  (inference).

### Claude Code hooks that wake or continue

- **`asyncRewake`.** "If `true`, runs in the background and wakes
  Claude on exit code 2. The hook's stderr, or stdout if stderr is
  empty, is shown to Claude as a system reminder." It "wakes Claude
  immediately even when the session is idle", and Claude Code "still
  enforces `timeout`" on it, by default 600 seconds
  ([hooks](https://code.claude.com/docs/en/hooks)). A system reminder
  is text the harness adds, "wrapped in `<system-reminder>` tags inside
  a user message"
  ([glossary](https://code.claude.com/docs/en/glossary)).
  - Fit: B0, but the text is hook output, which OWN-03 forbids for
    owner text. It could carry only a fixed template that names ids,
    asking Claude to fetch the message. ParleyInn would have to rearm the
    hook after each timeout, and the maximum `timeout` is not
    documented (unverified).
- **`Stop` with `decision: "block"`.** `reason` "Tells Claude why it
  should continue"; after eight consecutive continuations Claude Code
  "overrides the next block and ends the turn" (hooks). It acts only
  as a turn ends, never on an idle session.
- **Async hooks.** "If the session is idle, the response waits until
  the next user interaction", except `asyncRewake` (hooks).
- **Scheduled wake-ups.** `/loop` tasks fire "only ... while Claude
  Code is running and idle", at least a minute apart, and recurring
  tasks expire after seven days
  ([scheduled tasks](https://code.claude.com/docs/en/scheduled-tasks)).
  An agent could poll a ParleyInn tool, but the result is recall, which I2
  envelopes as untrusted, and each poll is a model turn (inference).

### Claude Code Remote Control, the Agent SDK and background sessions

- **Remote Control** is the harness's own input from the Claude apps.
  The local session "registers with the Anthropic API and polls for
  work"; API keys are not supported
  ([Remote Control](https://code.claude.com/docs/en/remote-control)).
  It fails CON-06 as a ParleyInn path; the person may still use it.
- **Agent SDK and stream-json.** Streaming input gives "Queued
  messages: send multiple messages that process sequentially"
  ([streaming input](https://code.claude.com/docs/en/agent-sdk/streaming-vs-single-mode)).
  It drives the process the host started; `resume` and `fork` start a
  new process on a stored session
  ([sessions](https://code.claude.com/docs/en/agent-sdk/sessions)). It
  cannot reach a terminal session the person started (inference).
- **Background sessions.** Agent view is a research preview. A
  supervisor runs each background session as its own process; a peek
  reply to a working session "joins the session's message queue".
  From the shell, `claude agents --json` reads state, and the listed
  commands attach, read logs, stop, respawn and remove; none sends a
  message
  ([agent view](https://code.claude.com/docs/en/agent-view)).

### Codex

- **The shared daemon.** `codex app-server daemon start` runs an
  app-server that "backs the machine-readable `codex app-server`
  lifecycle commands used by remote clients such as the desktop and
  mobile apps"; it is "experimental". If an implicitly found daemon
  fails, "the TUI starts an embedded server instead"
  ([daemon README](https://github.com/openai/codex/blob/4ad985e/codex-rs/app-server-daemon/README.md)).
- **Implicit attach.** At commit 4ad985e (4 October 2026), a plain
  `codex` probes the default control socket and becomes a
  `LocalDaemon` client when the socket answers; otherwise it embeds
  the server
  ([tui/src/lib.rs](https://github.com/openai/codex/blob/4ad985e/codex-rs/tui/src/lib.rs)).
- **Input methods.**
  - `turn/start` begins a turn on a thread; `turn/steer` appends user
    input to the active turn and "fails if there is no active turn"
    ([app-server](https://learn.chatgpt.com/docs/app-server)).
  - `thread/queue/add` is marked `#[experimental]`
    ([protocol](https://github.com/openai/codex/blob/4ad985e/codex-rs/app-server-protocol/src/protocol/common.rs)).
    `codex queue` ("Queue a message for an existing session") uses it
    and refuses to run through an embedded server while a daemon runs
    ([session_queue_commands.rs](https://github.com/openai/codex/blob/4ad985e/codex-rs/tui/src/session_queue_commands.rs)).
  - `thread/inject_items` appends model-visible items of any role
    without a turn; the
    [OpenAI note](../openai-agent-ui/openai-agent-ui.md) explains why
    ParleyInn should not use it.
- **Transport.** Unix-socket listeners carry WebSocket framing; the
  WebSocket transport is "experimental and unsupported" (app-server).
- **Hooks.** A finished background hook does not start a turn: "If no
  turn is active, Codex waits until the next user turn". A `Stop`
  block "creates a new continuation prompt that acts as a new user
  prompt, using your `reason`"
  ([Codex hooks](https://learn.chatgpt.com/docs/hooks)). Both act only
  within a turn.
- **Fit.** The app-server is on OWN-03's list, so the daemon path
  needs no I2 change. It needs a B1 sender and the person's opt-in to
  run the daemon (inference). With an embedded server the control is
  unavailable (OWN-15).

### Gemini CLI, ACP and OpenCode

- **Gemini CLI.** `AfterAgent` "Fires once per turn after the model
  generates its final response"; on `deny`, `reason` "is sent to the
  agent as a new prompt"
  ([hooks reference](https://geminicli.com/docs/hooks/reference/)).
  No path into an idle interactive session was found (unverified
  absence). `gemini --acp` reaches only a session a client launched.
- **ACP.** "The client launches the agent as a subprocess"
  ([protocol components](protocol-and-data-components.md)).
  "RFD: Multi-Client Session Attach" proposes `session/attach` through
  a multiplexing proxy, first answer wins on permissions; it has been
  open since 18 February 2026
  ([PR 533](https://github.com/agentclientprotocol/agent-client-protocol/pull/533)).
- **OpenCode.** "When you run opencode it starts a TUI and a server";
  the TUI "randomly assigns a port and hostname" unless given
  `--port`. `POST /session/:id/prompt_async` sends a message without
  waiting, and `/tui/append-prompt` drives the TUI
  ([server](https://opencode.ai/docs/server/)). This is loopback HTTP,
  so B1, and not on OWN-03's list (inference).

### Prior art: hcom

hcom (MIT, mostly Rust) lets agents "message, watch, and spawn each
other across terminals". Hooks record activity to a local SQLite
database and deliver messages from it, and an optional MQTT relay
links devices. "Hooks activate only when an agent is launched with
`hcom` in front" ([hcom](https://github.com/aannoo/hcom)). The closest
prior art also needs a wrapper launch.

### Which paths stay on the machine

- **No socket (B0):** channels over stdio, `asyncRewake` and `Stop`
  hooks, `/loop` polling of a ParleyInn tool.
- **A local socket or loopback (B1):** the Claude Code inbox, the Codex
  daemon, OpenCode's server, and stream-json or ACP when ParleyInn's run
  component hosts the harness.
- **Off the machine:** Remote Control and Codex Remote, both vendor
  relays.

## Tables

### Phone reach methods

| Method                                    | Works off-LAN                                   | Central service needed                              | Runs on iOS/Android                                                      | Language/library                                      | License                                   | Fit                                                           |
| ----------------------------------------- | ----------------------------------------------- | --------------------------------------------------- | ------------------------------------------------------------------------ | ----------------------------------------------------- | ----------------------------------------- | ------------------------------------------------------------- |
| mDNS/DNS-SD                               | no                                              | none                                                | yes: `NWBrowser` with a permission alert; `NsdManager`, permission on 17 | Go `hashicorp/mdns`, `brutella/dnssd`; Rust `mdns-sd` | MIT; MIT; Apache-2.0 OR MIT               | discovery at home only (row 13)                               |
| WireGuard to the owner's endpoint         | yes, with a public address or forwarded port    | none; the owner's endpoint                          | yes: official apps; one VPN at a time on iOS                             | `wireguard-go`; WireGuard apps                        | MIT; Apache-2.0 (Android app)             | stage one (row 11); the owner keeps keys and a port           |
| Tailscale with Headscale and own DERP     | yes                                             | the owner's Headscale and relay                     | yes: custom coordination server in the iOS app                           | Tailscale clients; Headscale                          | BSD-3-Clause; iOS GUI closed (inference)  | stage one; two services for the owner                         |
| Tailscale's own service                   | yes                                             | yes: Tailscale's control and relays                 | yes                                                                      | Tailscale clients                                     | as above                                  | the owner's choice under row 11, never ParleyInn's dependency |
| iroh with self-hosted relay               | yes, through the owner's relay                  | none; the owner's `iroh-relay` with a public name   | Swift package; Kotlin JVM artifact, Android built with the NDK           | Rust `iroh` 1.3; iroh-ffi                             | MIT OR Apache-2.0; UniFFI runtime MPL-2.0 | B2 phone-as-peer; relay on an enrolled peer (PEER-04)         |
| Direct QUIC with hole punching            | often; fails when both NATs map per destination | a rendezvous both sides reach                       | iOS 15 QUIC; Android through a library                                   | `quic-go`, `quinn`                                    | MIT; MIT OR Apache-2.0                    | own protocol work; still needs a reachable peer               |
| Owner-run relay on the home server        | yes, with a public address                      | none; the owner's node                              | yes while the app runs                                                   | ParleyInn's peer component                            | ParleyInn's                               | fits X7, PEER-02, PEER-04, PEER-12                            |
| Owner-run Nostr or Matrix relay           | yes                                             | the owner's relay; iOS push via a publisher gateway | yes                                                                      | Buzz relay (Rust); Sygnal                             | Apache-2.0; AGPL-3.0 or commercial        | a second protocol beside segments; lesson only                |
| UnifiedPush to an owner-run ntfy (wake)   | yes                                             | none; the owner's ntfy                              | Android only                                                             | ntfy, UnifiedPush connectors                          | not checked                               | Android notifications under row 22                            |
| Local Push Connectivity (wake)            | no: named Wi-Fi, private LTE, Ethernet          | none                                                | iOS 14 and later, entitlement from Apple                                 | `NEAppPushProvider`                                   | Apple SDK                                 | home-network notifications on iPhone                          |
| APNs (wake)                               | yes                                             | yes: Apple, through the publisher's key             | iOS                                                                      | any APNs client                                       | Apple terms                               | breaks VIEW-17 and CON-06 unless the owner signs the app      |
| Vendor relays (Remote Control, T3, Happy) | yes                                             | yes                                                 | yes                                                                      | their apps                                            | mixed                                     | rejected (X4, X7, CON-06)                                     |

### Live delivery paths

| Path                                              | Idle session reachable              | Is harness input (OWN-03)                      | Network needed                   | Status                                        | Fit                                            |
| ------------------------------------------------- | ----------------------------------- | ---------------------------------------------- | -------------------------------- | --------------------------------------------- | ---------------------------------------------- |
| Claude Code channels                              | yes: an event starts a turn         | no: MCP event in a `<channel>` tag             | none: stdio (B0)                 | research preview; allowlist; flag per session | best B0 fit; needs an OWN-03 and I2 change     |
| Claude Code cross-session inbox                   | yes: starts a new turn              | harness input, labelled as another session     | same-user Unix socket, pipe (B1) | shipped v2.1.224; message format undocumented | new B1 row and OWN-03 change; spike the format |
| Claude Code `asyncRewake` hook                    | yes: exit 2 wakes                   | no: hook output as a system reminder           | none (B0)                        | documented; timeout enforced                  | a fixed id-only template at most               |
| Claude Code `Stop` block                          | no: turn end only                   | no: hook output                                | none (B0)                        | documented; eight-continuation cap            | not a delivery path                            |
| Claude Code `/loop` polling a ParleyInn tool      | yes, a minute or more later         | no: tool result is enveloped recall            | none (B0)                        | documented; seven-day expiry                  | pull only; loses owner authority               |
| Claude Code Remote Control                        | yes                                 | yes                                            | Anthropic relay                  | shipped; claude.ai login                      | rejected for ParleyInn (CON-06)                |
| Claude Agent SDK or stream-json                   | yes, for sessions it hosts          | yes                                            | none; run component (B1)         | SDK under Commercial Terms                    | only when launched through ParleyInn (OWN-15)  |
| Codex daemon: `turn/start`, `thread/queue/add`    | yes, when the TUI uses the daemon   | yes: app-server                                | Unix socket (B1)                 | daemon and queue experimental                 | good where the daemon runs; else unavailable   |
| Codex `turn/steer`                                | no: active turn only                | yes: app-server                                | Unix socket (B1)                 | documented                                    | mid-turn steer                                 |
| Codex `thread/inject_items`                       | no turn starts                      | no: raw items of any role                      | Unix socket (B1)                 | documented                                    | reject                                         |
| Codex `Stop` block or background hook             | no: turn end, or waits for the user | no: hook output                                | none                             | documented                                    | not a delivery path                            |
| Codex Remote                                      | yes                                 | yes                                            | OpenAI relay                     | rolling out                                   | rejected                                       |
| Gemini CLI `AfterAgent` deny                      | no: turn end only                   | no: hook output becomes a prompt               | none                             | documented                                    | not a delivery path                            |
| ACP (`gemini --acp`, adapters) and `session/load` | only sessions a client launched     | yes: ACP                                       | stdio, run component (B1)        | ACP v1 stable                                 | only when launched through ParleyInn           |
| ACP `session/attach`                              | would be                            | yes: ACP                                       | through a proxy                  | RFD open since February 2026                  | watch                                          |
| OpenCode `prompt_async`                           | yes                                 | the harness's own server, not on OWN-03's list | loopback HTTP (B1)               | documented                                    | workable once the TUI port is fixed            |

## What this means for the options

- **The phone does not escape a reachable node.** Every option that
  works away from home needs a node the owner keeps reachable. Making
  the owner's always-on node the reachable peer (X7, PEER-04) keeps
  ParleyInn free of central services. A tunnel the owner already runs
  (WireGuard, Headscale) serves stage one without new ParleyInn code.
- **"See your stuff" on the phone favours the phone as a peer.** X2
  rules out a remote view over B2, and a tunnel shows rooms only while
  it is up. A phone that holds sealed ranges must derive byte-identical
  state (PEER-03). That favours a core language that also builds for
  phones: Rust through iroh-ffi or UniFFI, or Kotlin Multiplatform.
  Go reaches phones only through the experimental gomobile
  ([protocol components](protocol-and-data-components.md), inference).
  This raises the weight the earlier notes gave the phone, which they
  called the smallest slot.
- **Chat from the phone is an SRS change before it is code.** OWN-16,
  OWN-17 and PRV-10 limit the phone to allow and deny. A chat scope
  needs the stakeholder's decision and an ENG-29 review.
- **Notifications split by platform.** Android can use an owner-run
  ntfy through UnifiedPush under row 22, once VIEW-17 and X4 are
  worded to admit it. iOS cannot get timely notifications away from
  home without APNs. The stakeholder chooses between showing that gap
  (OWN-15), Local Push Connectivity at home, or admitting APNs with
  alias, class and count through a key the owner holds.
- **Live delivery to Claude Code has a B0 path, but it is a
  preview.** Channels wake idle sessions over stdio from ParleyInn's own
  MCP server and carry permission verdicts, which suits the phone's
  allow and deny. The inbox socket has shipped but needs B1 and labels
  the person as another session. Both need OWN-03 and I2 amended. A
  spike should check how the model treats a `<channel>` event and the
  inbox's undocumented message format.
- **Codex fits OWN-03 today through its daemon.** ParleyInn would offer
  the daemon as a shown opt-in (I7) and mark delivery unavailable when
  the TUI embeds its server.
- **Gemini CLI and ACP show the control as unavailable** until ACP's
  attach proposal lands or ParleyInn's run component launches them
  (OWN-15).
- **The language choice is not decided here.** A channel is
  JSON-RPC over stdio with an experimental capability, which a
  hand-written Go stdio server can emit as well as an SDK can
  (inference). The phone, not live delivery, is where this research
  moves the language choice.
