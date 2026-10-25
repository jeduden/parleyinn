# IRC: channel roles, services, bots and netsplits as prior art for rooms

Scope: IRC's patterns for channel roles, services, bots, merge after a
netsplit and trust, read on 4 October 2026 from modern.ircdocs.horse,
RFC 2811, the TS6 specification, the Atheme help files, the Libera
guides, the IRCv3 specifications and the Eggdrop and Limnoria
documentation. Claims resting on secondary sources are marked so.

## Roles and modes

- **The ladder.** Founder (`~`, +q), protected (`&`, +a), op (`@`, +o),
  halfop (`%`, +h) and voice (`+`, +v); only op and voice are near
  universal ([modern IRC](https://modern.ircdocs.horse/)).
- **Speaking and topic.** +m lets only ops and voiced users speak; +t
  limits the topic to ops; +n refuses messages from outside.
- **Entry.** +i invite only, +k a shared key, +s and +p hidden, and on
  Libera +r or +R for identified users only
  ([channel modes](https://libera.chat/guides/channelmodes)).
- **Lists.**
  - +b bans masks.
  - +e exceptions override bans.
  - +I invite exceptions.
  - Solanum's +q quiet keeps a user present but silent.
  - +z routes blocked messages to the ops instead of dropping them.
  - `$a:account` bans by services account rather than by host mask.
- **Kick and ban differ.** A kick removes a user who may rejoin at once
  ([quick ops](https://libera.chat/guides/quickops)); only a ban
  persists.
- **On join** a client receives the topic, who set it and when
  (replies 332 and 333), then the member list with role prefixes
  (353, 366).

## Services

- **Why they exist.** Without services, ops is whatever a user holds at
  the moment, lost when ops leave or a netsplit empties the channel.
  Registration keeps an access list as lasting authority
  ([Atheme REGISTER](https://raw.githubusercontent.com/atheme/atheme/master/help/default/cservice/register)).
- **Access flags** ([Atheme FLAGS](https://github.com/atheme/atheme/blob/master/help/default/cservice/flags)):
  - +o and +v may take op or voice; +O and +V give them on join.
  - +t sets the topic, +r kicks and bans, +f edits the access list.
  - +F is founder, +S successor, +b auto-kickban.
  - Libera warns against granting +F lightly and against automatic
    status.
- **Settings.**
  - SECURE: only the access list can gain op.
  - RESTRICTED: everyone else is kickbanned on join.
  - MLOCK: fixes modes.
  - TOPICLOCK: reverts topic changes by users without +t.
  - FANTASY: allows `!op`-style commands in the channel.
- **AKICK** entries carry a reason, a private note after `|`, and a
  permanent or timed expiry.
- **Founder transfer** needs the new founder to confirm. A named
  successor takes over when a founder's account expires (secondary
  source).

## Bots

- **Eggdrop** identifies users by `nick!user@host`, keeps owner, op,
  auto-op and auto-kick flags, and offers flood protection, ban
  enforcement and retaliation settings
  ([users](https://docs.eggheads.org/using/users.html)).
- **Limnoria** checks capabilities on every command. Its owner
  capability can only be granted locally, never over IRC
  ([capabilities](https://docs.limnoria.net/use/capabilities.html)).
- **IRCv3 bot mode** marks bots with a server-set tag that clients
  cannot forge ([bot mode](https://ircv3.net/specs/extensions/bot-mode)).
- **Moderation practice.** Libera advises:
  - ops who stay unopped until they act;
  - talking in private first;
  - temporary bans;
  - an appeal path ([catalyst](https://libera.chat/guides/catalyst)).

  The usual ladder is warn, quiet, kick, ban.

## Netsplits and merge

- **Takeovers.** During a split, the first user to rejoin an emptied
  channel gains ops and kicks the real ops when the network merges
  (secondary source).
- **TS6** resolves a merge by channel timestamp
  ([TS6](https://raw.githubusercontent.com/grawity/irc-docs/master/server/ts6.txt)):
  - The older side wins and the newer side loses its modes and
    statuses.
  - Equal timestamps are unioned.
  - Users who lost op that way have their mode changes ignored.
- **RFC 2811 safe channels** carry a server-generated id, so a split
  cannot recreate them under the same short name
  ([RFC 2811](https://www.rfc-editor.org/rfc/rfc2811.html)).
- **Why it works for IRC and not for ParleyInn.** It relies on trusted
  servers with roughly agreeing clocks. Among peers, a timestamp is
  whatever the writer claims.

## IRCv3

- **message-ids, server-time:** stable ids and times on messages.
- **message-tags:** client-only `+` tags are untrusted; the server
  strips unprefixed tags that clients send.
- **account-tag, account-notify:** the sender's identity comes from the
  server, not from the message.
- **chathistory:** servers must not serve history a user may not see.
- **echo-message, labeled-response:** the sender sees the stored
  message, correlated by label.
- **standard-replies:** coded refusals.
- **channel-rename:** names change, identity stays.
- **read-marker:** private read positions.

## Trust

- **Injection.** Bots that parse channel text as commands were
  injectable. POE::Component::IRC and matrix-appservice-irc
  ([CVE-2023-38690](https://nvd.nist.gov/vuln/detail/CVE-2023-38690))
  ran newline-split text as protocol commands.
- **Impersonation.** The fixes moved from nicks to host masks, then to
  services accounts, then to capability checks on every command.

## What ParleyInn takes

| IRC                                   | ParleyInn                                                               |
| ------------------------------------- | ----------------------------------------------------------------------- |
| Topic, setter and time on join        | Room pins, intent first, each with author and time, shown on join       |
| Member list with prefixes             | Roster with roles on join                                               |
| +t                                    | Pin edits refused by the authorisation layer, not reverted afterwards   |
| ChanServ registration and access list | The owner's role configuration as lasting authority                     |
| SECURE, RESTRICTED                    | Powers only from configured roles; presence only from a person's add    |
| AKICK with reason, note, expiry       | A bar record: target key, reason, private note, expiry, setter, audited |
| KICK, rejoin allowed                  | Kick revokes the current add; the player's person may add it again      |
| +q quiet, +m                          | An optional quiet restriction: present, cannot post                     |
| `$a:` account bans                    | Bars by key, never by name or mask                                      |
| Bot mode, server-set tag              | Player kind attested by the layer                                       |
| ChanServ GUARD, Eggdrop               | The etiquette bot, holding the operator role                            |
| +z                                    | A classifier finding routed to operators                                |
| Successor, two-sided transfer         | An owner-named successor; handover confirmed by both sides              |
| TS6 merge, "deopped"                  | Restriction wins; powers only from the owner's grant chain, never time  |
| Safe channels                         | Room id from a key or creation record                                   |
| `+` tags against server tags          | Player-supplied fields against layer-attested fields                    |
| chathistory authorisation             | Recall checked against membership intervals                             |

**Avoid:**

- timestamp-wins merge among peers;
- exceptions that override bars;
- commands parsed from room text;
- automatic operator status;
- mask bans;
- bots that fight human operators or retaliate;
- heuristic successors.
