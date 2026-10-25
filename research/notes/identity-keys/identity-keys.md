# Identity and key certification: prior art for ParleyInn's key chain

Scope: how standard systems certify delegated keys, read on 4 October
2026 from their specifications, source and documentation: Nostr,
decentralised identifiers and capabilities, blockchain-style identity,
OIDC and workload identity, classic PKI and messaging, Radicle, and
git's signing. The question is how a person's owner key reaches every
key acting for them, so that a bar on the person covers all of them,
a join needs no approval each time, a stolen key can be cut off, keys
never enter a model's context, and everything works offline and peer
to peer.

## What each system offers

- **Nostr:**
  - **NIP-26 delegation** failed: no revocation, and optional for
    verifiers. It is now marked "unrecommended", because a delegation
    some verifiers skip breaks every client that skips it
    ([NIP-26](https://github.com/nostr-protocol/nips/blob/master/26.md),
    [fiatjaf](https://fiatjaf.com/4c79fd7b.html)).
  - **NIP-46 remote signers** keep the user key inside a signer and
    have apps ask for signatures; this is the clearest model for keeping
    keys out of a model's context
    ([NIP-46](https://github.com/nostr-protocol/nips/blob/master/46.md)).
  - **Key rotation** has no merged proposal.
- **Decentralised identifiers and capabilities:**
  - did:key is a self-describing public key with no update or
    deactivation, suited to short-lived keys
    ([did:key](https://w3c-ccg.github.io/did-key-spec/)).
  - **KERI** keeps a hash-chained key event log per identifier. Each
    establishment event commits to the digest of the next key
    (pre-rotation), so a stolen current key cannot seize the identity.
    Delegated identifiers are anchored in the delegator's log, and
    conflicting signed versions are detected as duplicity. Ordering is
    first-seen, not clock
    ([KERI](https://trustoverip.github.io/kswg-keri-specification/)).
  - **UCAN** delegations narrow a parent's authority by command path
    and policy, with append-only revocation by content id that works
    eventually consistent. An unrevoked second chain keeps authority
    alive, and its expiry depends on clocks
    ([UCAN](https://github.com/ucan-wg/delegation),
    [revocation](https://github.com/ucan-wg/revocation)).
  - **Biscuit** attenuates tokens offline by appending signed blocks
    ([Biscuit](https://doc.biscuitsec.org/reference/specifications)).
  - **Macaroons** need a shared root secret, so peers cannot verify
    them.
- **Blockchain-style identity:**
  - Farcaster's custody address authorises and revokes app keys
    through an on-chain registry
    ([Farcaster](https://github.com/farcasterxyz/protocol/blob/main/docs/SPECIFICATION.md)).
  - Bluesky's did:plc separates priority-ordered rotation keys from
    signing keys, through a central directory
    ([did:plc](https://web.plc.directory/spec/v0.1/did-plc)).
  - Keybase's sigchain adds each device by an existing device's link
    plus the new key's reverse signature, keeps a paper key for
    recovery, and anchors in Bitcoin
    ([Keybase](https://book.keybase.io/docs/server)).
  - All three depend on a chain or a server for ordering.
- **OIDC and workload identity:**
  - Token exchange records delegation chains through a token service
    ([RFC 8693](https://datatracker.ietf.org/doc/html/rfc8693)).
  - DPoP binds a token to a key with a proof on every use
    ([RFC 9449](https://datatracker.ietf.org/doc/html/rfc9449)).
  - Sigstore turns an OIDC login into a ten-minute certificate logged
    publicly
    ([Fulcio](https://docs.sigstore.dev/certificate_authority/overview/)).
  - SPIFFE attests a workload process before giving it a short-lived
    identity
    ([SPIRE](https://spiffe.io/docs/latest/spire-about/spire-concepts/)).
  - Each needs an online authority.
- **Classic PKI and messaging:**
  - X.509 name constraints limit what lies below a CA.
  - SSH certificates carry principals, a validity window and critical
    options, revoked by key revocation lists
    ([ssh-keygen](https://man.openbsd.org/ssh-keygen#CERTIFICATES)).
  - OpenPGP subkeys carry a back-signature from the subkey
    ([RFC 9580](https://datatracker.ietf.org/doc/html/rfc9580)).
  - Matrix cross-signing separates a master key, a key that signs your
    own devices and a key that signs other people
    ([Matrix](https://spec.matrix.org/latest/client-server-api/#cross-signing)).
- **Radicle**
  ([protocol](https://radicle.dev/guides/protocol),
  [user guide](https://radicle.dev/guides/user),
  [heartwood](https://github.com/radicle-dev/heartwood)):
  - **Keys.** One Ed25519 key per device, published as a did:key. No
    person-level identity, and no rotation or revocation: a lost key
    means a new identity.
  - **Repository identity.** A canonical document of delegates and a
    threshold, whose first version's hash is the repository id; the
    man page and the code disagree on whether a revision needs the
    threshold or a majority.
  - **Signed refs and collaborative objects.** Each node signs all its
    refs, binding the repository id and the previous signature against
    replay. Every collaborative-object operation names the identity
    document it was made under, so authorisation is checked offline.
  - **Blocking** is local policy only.
  - **Its predecessor, radicle-link,** had person identities over
    device keys, quorum verification against the previous revision, a
    fork rule that stops counting a forked person, and one vote per
    person; key recovery was left open
    ([identities](https://github.com/radicle-dev/radicle-link/blob/master/docs/spec/sections/identities.adoc)).
- **git:**
  - **Signing formats.** git signs with OpenPGP, X.509 or SSH. SSH
    allowed-signers lines carry principals, namespaces and validity
    windows, and verify offline
    ([git-config](https://git-scm.com/docs/git-config),
    [allowed signers](https://man.openbsd.org/ssh-keygen.1#ALLOWED_SIGNERS)).
  - **Time.** The validity window is checked against the timestamp
    inside the signed object, which the signer chose, so a stolen key
    can backdate.
  - **gitsign** depends on Fulcio, Rekor and an identity provider to
    sign.
  - **Author fields** are free text a signature does not cover.
  - **Forges differ on revocation.** GitHub keeps a commit verified
    after its key is revoked; GitLab flips it to unverified.

## What ParleyInn takes

| Idea                                   | From                          | In ParleyInn                                                                    |
| -------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------- |
| Pre-rotation in a key event log        | KERI                          | The owner key's log commits to its next key, held offline for recovery          |
| Reverse signature on certification     | OpenPGP, Keybase              | A device countersigns its certificate, so nobody claims a key they lack         |
| Attenuated delegation                  | UCAN, Biscuit                 | Each key's authority is a subset of its parent's: commands, rooms, count        |
| Signer outside the app                 | NIP-46, ssh-agent in Radicle  | A local signer process holds keys; hooks ask it to sign; no key reaches a model |
| did:key for short-lived keys           | did:key, Radicle              | Participant keys named as did:key                                               |
| Principals and namespaces              | SSH allowed signers           | A participant certificate names its room; namespaces keep key roles apart       |
| Proof of possession                    | DPoP                          | Joining signs a room nonce, so a copied certificate is useless                  |
| Operations cite their governance       | Radicle collaborative objects | Every room act names the room document it was made under (I10)                  |
| Signed log heads binding id and parent | Radicle signed refs           | Each participant's log head is signed with the room id and previous head        |
| Quorum of the previous revision        | radicle-link                  | Room ownership and handover take effect only when the previous owners sign      |
| One vote per person                    | radicle-link                  | Several devices of one person count once                                        |
| Identity as evidence, never authority  | Keybase proofs, Sigstore      | An optional statement binding an owner key to an account such as GitHub         |

## The proposed chain

Developers already have an SSH identity: the key git pushes with and,
with `gpg.format=ssh`, signs commits with, held in ssh-agent (often on
a hardware security key) and frequently published on their forge. The
chain starts from it instead of a new key.

1. **Owner key: the person's existing SSH signing key.** It is used
   through ssh-agent, so the private key never leaves the agent or the
   hardware token, and ParleyInn never holds it. ParleyInn signs with it the
   way git does, `ssh-keygen -Y sign` under its own namespaces
   (`parinn-device`, `parinn-participant@<room>`), so a ParleyInn signature
   can never pass as a git signature or the reverse. The same key in
   the person's git allowed-signers file, or listed by their forge,
   is evidence of who they are, never authority.
2. **Rotation and recovery.** SSH has no pre-rotation, so the owner's
   key event log in the record does it: the log names the current
   SSH key and commits to the next one, a second SSH key held offline
   (a spare hardware key) for recovery. A rotation entry is signed by
   the committed next key.
3. **Device key.** Certified by an owner-log entry signed with the
   SSH key, naming its scope and rule ceiling, and countersigned by
   the device.
4. **Harness key.** Issued once by the device when a harness is first
   used, with a standing grant: which commands and rooms, and how many
   joins. Held by a local signer process, as ssh-agent holds SSH keys,
   that the harness asks to sign.
5. **Participant key.** Minted per room on joining, as a did:key,
   signed by the harness key within its standing grant, so the person
   approves once per harness, not per join. Joining proves possession
   by signing a room nonce.

OpenSSH certificates cannot chain beyond one level, so the chain is
kept as SSH-signed entries in the record rather than as OpenSSH
certificates; the principal-and-namespace idea carries over.

**Checking a bar offline.**

- Every participant act carries, or references, its chain up to the
  owner key, all stored in the record.
- A bar names an owner key; verification walks the chain to its root
  and refuses, and audits, an act whose root is barred.
- Every key under that owner is covered without being listed.

**Revocation without clocks.**

- Revocations are append-only entries in the issuer's log and sync
  first.
- Each act cites the heads of the owner and device logs its writer had
  seen, so an act citing a head after a revocation is invalid.
- An act concurrent with a revocation is held and audited, never
  silently accepted or dropped.
- A short validity window may be checked at admission by the local
  hook, never inside merges or projections.

## Pitfalls

- **Optional delegation** that verifiers may skip (NIP-26): chain
  verification must be mandatory from the start.
- **A second unrevoked chain** keeps revoked authority alive (UCAN):
  one parent chain per key.
- **Bearer credentials:** bind every credential to a key and prove
  possession on use.
- **Keys in a model's context:** signing stays in a separate process,
  and key-shaped strings in the record are redacted.
- **Clock-dependent validity:** UCAN's drift allowance and git's
  backdating show clocks cannot order trust; merges use causal order
  only.
- **Hidden servers:** did:plc, Fulcio, SPIRE, token services and status
  lists all need one; borrow their ideas, not their wire formats.
- **No recovery:** without pre-rotation a stolen root is lost for good,
  as in did:key, Nostr and Radicle today.
