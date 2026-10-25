# Crypto-shredding for append-only, hash-chained, replicated logs (design reference for ParleyInn)

Scope note: technical claims and legal analysis are kept in separate sections (Q1–Q3 technical, Q4 legal, Q5 Go libraries, Q6 scheme sketch). Sources marked "(not fetched this session)" are canonical URLs cited from background knowledge; the report writer should treat them as lower-confidence than fetched sources. Research date: 2026-10-02.

## Q1. Patterns and precedents (event sourcing, envelope encryption, key hierarchies, ratchets, Matrix/MLS, Keybase, git-crypt/SOPS, transparency logs, deletable hash chains)

### Takeaway

Every precedent uses the same shape: per-subject (or per-folder/per-epoch) data keys wrapped by a higher key, ciphertext kept forever, deletion = destroy the data key; tamper evidence is preserved by having the hash chain/Merkle tree commit to ciphertext (KBFS, Matrix-style redacted forms) or to a keyed hash/commitment rather than plaintext. No precedent re-encrypts history on rotation; they all rotate forward and leave old ciphertext under old keys, which means "revoking" a device never removes what it already had.

### Cited Findings

**Event sourcing**

- Mathias Verraes' "Crypto-Shredding" pattern (13 May 2019): encrypt sensitive attributes "with a different encryption key for each resource (such as a customer)", give the key only to consumers that need it, and delete the key to erase; "This effectively makes all copies and backups of the sensitive data unusable." He warns "Today's unbreakable encryption could be tomorrow's infosec disaster" and that neither pattern solves consumers that store decrypted data or derive new values from it. — [Verraes, Crypto-Shredding](https://verraes.net/2019/05/eventsourcing-patterns-throw-away-the-key/)
- Companion pattern "Forgettable Payloads": store the sensitive payload in a separate, deletable store and keep only a reference in the immutable event. — [Verraes, Forgettable Payloads](https://verraes.net/2019/05/eventsourcing-patterns-forgettable-payloads/)
- Kurrent (EventStoreDB) blog, Diego Martin, 12 Aug 2021: crypto-shredding with a user-specific symmetric key generated at user creation, encrypting PII fields (name, email) in events; deleting the key makes them illegible. This is a blog/sample pattern, not a built-in server feature. — [Kurrent blog](https://kurrent.io/blog/protecting-sensitive-data-in-event-sourced-systems-with-crypto-shredding-1)
- AxonIQ offers a commercial "Axon Data Protection" module for field-level crypto-shredding in Axon Framework. — [AxonIQ blog](https://www.axoniq.io/blog/protect-sensitive-data-in-an-event-sourced-application); [AxonIQ forum](https://discuss.axoniq.io/t/crypto-shredding-and-gdpr-with-axon-framework/5987)
- Confluent Client-Side Field Level Encryption (CSFLE): envelope encryption where a KEK in an external KMS (AWS KMS, Azure Key Vault, GCP KMS) wraps DEKs; Schema Registry's "DEK Registry" stores the wrapped DEKs; Confluent "never directly accesses or persists the KEKs". The Confluent docs found do not describe a crypto-shredding/erasure workflow explicitly. — [Confluent CSFLE (Platform)](https://docs.confluent.io/platform/current/security/protect-data/csfle/quick-start.html); [Confluent Cloud CSFLE](https://docs.confluent.io/cloud/current/clusters/csfle/client-side.html)
- Other event-sourcing implementations ship per-subject key lifecycle docs, e.g. Cratis Chronicle key lifecycle and a Go `eskit/gdpr` package. — [Cratis key lifecycle](https://www.cratis.io/chronicle/compliance/key-lifecycle.md); [pkg.go.dev eskit/gdpr](https://pkg.go.dev/git.nullsoft.is/ash/eskit/gdpr)

**Keybase KBFS (closest precedent to "encrypted, Merkle-verified, multi-device, git on top")**

- Per top-level-folder (TLF) random 32-byte secret, versioned in key generations; per-block random key s_j, block key = first 32 bytes of HMAC-SHA512(TLF secret, s_j); blocks sealed with SecretBox. — [Keybase KBFS crypto spec](https://book.keybase.io/docs/crypto/kbfs)
- "Block IDs are computed as the SHA-256 hashes of encrypted blocks and their nonces" — i.e. content addressing commits to ciphertext, not plaintext. — [KBFS crypto spec](https://book.keybase.io/docs/crypto/kbfs)
- Device revocation creates a new key generation for future data; "Don't bother to reencrypt old blocks, so leave the old decryption materials around (except for the compromised key)." — [KBFS crypto spec](https://book.keybase.io/docs/crypto/kbfs)
- Deletion: users "ask the server to delete s_i" (the per-block key halves); server-side key-half split means deleting the server half shreds the block. — [KBFS crypto spec](https://book.keybase.io/docs/crypto/kbfs)
- Tamper evidence: signed root blocks plus Merkle trees; "Clients check their views of the file system with those published in the Merkle trees to make sure the server isn't rolling back state." — [KBFS crypto spec](https://book.keybase.io/docs/crypto/kbfs)
- Teams use "Cascading Lazy Key Rotations": per-team keys rotate on device revoke, member leave/removal or account reset, lazily, with new keys boxed for remaining members and recorded on the team sigchain. — [Keybase CLKR](https://book.keybase.io/docs/teams/clkr); [Team crypto](https://book.keybase.io/docs/teams/crypto)
- Keybase encrypted git repos are built on KBFS. — [Understanding KBFS](https://keybase.io/docs/kbfs/understanding_kbfs)

**MLS / ratchets used for deletion**

- RFC 9420 §9.2 "Deletion Schedule": a secret is "consumed" once used to encrypt/decrypt or once a derived value is consumed; "As soon as a group member consumes a value, they MUST immediately delete (all representations of) that value. This is crucial to ensuring forward secrecy for past messages." Members MAY keep unconsumed values briefly for out-of-order delivery. Covers init_secret, commit_secret, epoch_secret, encryption_secret, secret-tree nodes and ratchets. — [RFC 9420](https://www.rfc-editor.org/rfc/rfc9420.txt)
- RFC 9420: after processing an UpdatePath, recipients "MUST delete outdated key material" (path secrets, node secrets, replaced node key pairs). — [RFC 9420](https://www.rfc-editor.org/rfc/rfc9420.txt)
- RFC 9420 notes sender-key schemes get forward secrecy via a hash ratchet but post-compromise security is hard; MLS tree KEM gives both. — [RFC 9420](https://www.rfc-editor.org/rfc/rfc9420.txt)
- Note: MLS forward secrecy is a transport property — it deletes keys members hold, and it is incompatible with "Claude recalls exact history on demand" unless messages are re-encrypted at rest under a separate storage key. The DMLS draft (draft-kohbrok-mls-dmls) works on decentralized/forked MLS commits relevant to multi-device without a central delivery service. — [DMLS draft-03](https://ftp.sjtu.edu.cn/pub/internet-drafts/draft-kohbrok-mls-dmls-03.html)

**git-crypt / SOPS / git-remote-gcrypt limits**

- git-crypt has no "remove user" command; correct revocation requires rotating the internal symmetric key and re-encrypting files, while keeping old key versions to read old commits — so a revoked user can still decrypt all history they had. — [git-crypt issue #47](https://github.com/AGWA/git-crypt/issues/47); [Remove users from git-crypt repo](https://giorgos.sealabs.net/remove-users-from-git-crypt-enabled-repository.html); [UK MoJ runbook: rotate git-crypt key](https://runbooks.cloud-platform.service.justice.gov.uk/rotate-git-crypt-key.html)
- SOPS: revoke by removing the recipient and running `sops updatekeys`; "the old encrypted blobs remain decryptable in git history." — [Protea docs: sops + age](https://protea.readthedocs.io/en/stable/appendix/secrets.html)
- Inference from the above: in any git-native scheme, git object IDs hash content, so shredding inside git requires either encrypting before git sees content (gcrypt/git-crypt style, where git hashes ciphertext) or history rewriting. git-remote-gcrypt encrypts the whole repository to a remote; not verified this session.

**Transparency logs**

- C2SP tlog-tiles: Merkle operations "as specified by RFC 6962" with SHA-256; entries are "arbitrary" data; signed checkpoints; tiles of 256 hashes. The log commits to whatever bytes are entries — it is agnostic to whether those bytes are plaintext, ciphertext, or a hash. — [C2SP tlog-tiles](https://github.com/C2SP/C2SP/blob/main/tlog-tiles.md)
- Certificate Transparency (RFC 6962/9162) logs commit to the full public certificate, so it is not a shredding precedent; Sigstore Rekor-style logs commonly store hashes/signatures of artifacts rather than the artifacts. — [RFC 9162](https://www.rfc-editor.org/rfc/rfc9162) (not fetched this session)

**Deletable-but-verifiable hash chains**

- EDPB (2026) lists three on-chain protection measures: encrypt payloads; store only a keyed hash/HMAC/KDF output with data and key off-chain; or store a cryptographic commitment ("perfectly hiding" → once data and witness are deleted the commitment "is useless... neither possible to recover nor to recognise the original personal data"). — [EDPB Guidelines 02/2025 v2.0, paras 32–34, 51–54](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB para 64: an "anonymised transaction" can lose all semantics yet "still exists to allow the verification of integrity for other, remaining transactions" — this is the tombstone model. — [EDPB Guidelines 02/2025 v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- Redactable blockchains use chameleon hashes (trapdoor allows a collision so a block can be rewritten without breaking links); Ateniese et al. 2017 was first; later work adds verifiability/accountability (double-trapdoor, accumulators, vector commitments). The trapdoor holder can rewrite history, which weakens tamper evidence. — [Redactable Blockchains: An Overview (arXiv 2508.08898)](https://arxiv.org/html/2508.08898v1); [Towards Data Redaction in Bitcoin](https://arxiv.org/pdf/2305.10075); [VRBC](https://ro.uow.edu.au/test2021/7232)
- Matrix redaction precedent (not fetched this session): Matrix signs and reference-hashes the *redacted* form of an event while a separate content hash covers the full content, so servers can strip content on redaction without breaking the event graph's signatures. — [Matrix server-server spec, reference hashes](https://spec.matrix.org/latest/server-server-api/#calculating-the-reference-hash-for-an-event)

### Inferences

- The pattern that best fits ParleyInn's I10 (rebuildable from record) and tamper evidence: the chain/Merkle leaf hashes a header that contains (a) non-sensitive metadata, (b) a hash of the ciphertext (KBFS-style), and (c) optionally a keyed commitment to plaintext. Shredding a key leaves the chain verifiable; ciphertext becomes noise; plaintext projections must be rebuilt (and will show a tombstone).
- Committing to a plain (unsalted, unkeyed) hash of plaintext is a leakage risk: short or guessable content (e.g. "yes", file paths, known code) can be confirmed by brute force; EDPB explicitly says unsalted/unkeyed hashes are generally insufficient (para 52).
- Chameleon-hash redaction is a poor fit for a security-first tool because it introduces a rewrite trapdoor; crypto-shredding with ciphertext commitment keeps the log strictly append-only.

### Gaps

- Marten (.NET) crypto-shredding / masking features were not verified.
- Tahoe-LAFS (capability URIs, convergent encryption) and Peergos (cryptree, per-file keys, revocation via re-keying) were not researched this session; both are relevant precedents for capability-based sharing and should be checked.
- Matrix MLS-based deletion (MSC proposals) and Signal disappearing messages were not fetched.
- git-remote-gcrypt internals not verified.

## Q2. Envelope encryption and key hierarchies (tenant → project → session → event)

### Takeaway

Industry practice is two or three wrap levels: a root/KEK (KMS, hardware, or passphrase/identity), per-subject DEKs wrapped by it, and per-object keys derived (HKDF/HMAC) or randomly generated per block. Shredding granularity equals the lowest level whose key is stored independently; derived keys cannot be shredded independently of their parent.

### Cited Findings

- Confluent: KEK in KMS wraps DEKs; wrapped DEKs stored beside data (DEK Registry). — [Confluent CSFLE](https://docs.confluent.io/platform/current/security/protect-data/csfle/client-side.html)
- KBFS: per-TLF secret (with generations) → per-block key derived via HMAC from a random per-block nonce; deletion operates on per-block server-held key halves. — [KBFS crypto spec](https://book.keybase.io/docs/crypto/kbfs)
- EDPB: management of "cryptographic information (keys, seeds, salts, etc.)" must itself be in the risk assessment and data-protection-by-design analysis. — [EDPB Guidelines 02/2025 v2.0, para ~93](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- age: files can be encrypted to multiple recipients (each recipient wraps the same file key); post-quantum hybrid recipients `age1pq1...` exist (~2000 chars). — [age README](https://github.com/FiloSottile/age/blob/main/README.md)
- age format uses a per-file random file key, wrapped per recipient in the header, with payload encrypted in ChaCha20-Poly1305 STREAM chunks (64 KiB). — [age spec, C2SP](https://c2sp.org/age) (not fetched this session)
- libsodium `crypto_secretstream_xchacha20poly1305`: chunked authenticated stream with rekeying and a final tag, suited to append-style streams. — [libsodium secretstream docs](https://doc.libsodium.org/secret-key_cryptography/secretstream) (not fetched this session)
- Tink: keysets with key rotation and envelope AEAD over a KMS KEK. — [Tink docs](https://developers.google.com/tink) (not fetched this session)

### Inferences

- For ParleyInn: a session DEK (random, stored wrapped) is the natural shred unit; per-event keys should be *derived* from it (HKDF with event sequence number) for nonce safety, not stored, unless per-event shredding is required. If per-event redaction is needed, store per-event random keys wrapped by the session key (the KBFS approach) so deleting one wrapped key shreds one event.
- A key hierarchy where project or tenant keys merely wrap session keys allows two shred granularities: delete one wrapped session key, or delete the project KEK (shreds all sessions it wraps, across all replicas, as long as no replica cached unwrapped session keys).
- Wrapped keys must live outside the immutable record (a mutable keystore), otherwise the hash chain would preserve the key that is supposed to be destroyed.

### Gaps

- No primary source found that standardizes a "tenant→project→session→event" hierarchy for logs; it is a synthesis.

## Q3. Threats and pitfalls

### Takeaway

Crypto-shredding only erases what the key protected and only if every copy of the key is gone; the dominant failure modes are key copies (backups, escrow, replicas, process memory/swap), retained plaintext derivatives (projections, indexes, caches, model context), long-lived ciphertext facing future cryptanalysis (including quantum), and metadata that stays in the clear.

### Cited Findings

- EDPB: after key deletion data is unintelligible "at least until the algorithm is broken, the decryption techniques advance sufficiently to allow the decryption of the cipher text, or if the key had already been compromised or leaked"; "even state-of-the-art encryption perfectly implemented will be overtaken by time if the blockchain is retained indefinitely." — [EDPB Guidelines 02/2025 v2.0, para 51](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB para 96: "all encryption systems have an undetermined, but limited, lifespan"; controllers must manage obsolescence including "cryptanalytically-relevant quantum computers", with periodic reassessment and planned migration. — [EDPB v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB para 50: modifications "may not even impact all copies of the original block, meaning that the original data might still be available" — the replica problem. — [EDPB v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB para 52: keyed/salted hashes become unlinkable after key/salt deletion only if "the keys have not been compromised or leaked, and the salt was not leaked or poorly chosen"; unsalted/unkeyed hashes are generally insufficient. — [EDPB v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB para 65: metadata confidentiality depends on who receives broadcast data; content confidentiality alone does not protect metadata. — [EDPB v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- Verraes: crypto-shredding does not help when consumers store data decrypted or derive new values from it. — [Verraes](https://verraes.net/2019/05/eventsourcing-patterns-throw-away-the-key/)
- RFC 9420 requires deleting "all representations" of consumed secrets — recognizing in-memory copies as a deletion target. — [RFC 9420 §9.2](https://www.rfc-editor.org/rfc/rfc9420.txt)
- Rotation cost: KBFS, git-crypt and SOPS all avoid re-encrypting history; revoked parties retain access to anything already obtained. — [KBFS](https://book.keybase.io/docs/crypto/kbfs); [git-crypt #47](https://github.com/AGWA/git-crypt/issues/47); [Protea sops](https://protea.readthedocs.io/en/stable/appendix/secrets.html)

### Inferences

- Deterministic encryption (e.g. SIV or convergent encryption for dedup) leaks equality of plaintexts and enables confirmation attacks; for chat/code logs with repeated content, use randomized AEAD with unique nonces (XChaCha20-Poly1305 or derived per-event keys).
- Go cannot reliably zeroize memory (GC copies, no mlock by default); key lifetime in process memory should be minimized and swap/core dumps considered. ParleyInn hooks are short-lived processes, which helps.
- For an AI agent, the model's context and any provider-side logs are plaintext derivatives outside ParleyInn's control; shredding cannot recall what was already sent to the model.
- Backups of the keystore defeat shredding unless the keystore backup itself is encrypted under a key that is rotated/destroyed on a schedule (the "backup retention window" becomes the effective erasure delay and should be documented).
- Rotation in an immutable store = re-wrap DEKs (cheap) rather than re-encrypt payloads (expensive and breaks ciphertext-hash commitments unless the chain commits to plaintext commitments instead).

### Gaps

- No sourced quantitative data on re-encryption costs found.
- No primary source on Go memory zeroization behavior fetched (Go's proposed `runtime/secret` / `crypto/subtle` work not verified).

## Q4. Legal status (GDPR Art. 17, regulator guidance, courts, legal holds) — LEGAL ANALYSIS, separate from technical claims

### Takeaway

As of October 2026 no EU regulator has stated that key destruction alone *is* erasure under Art. 17. The EDPB's final blockchain guidelines (v2.0, adopted 7 July 2026) treat encrypted data as still personal data, describe key deletion as making data "unintelligible" only until crypto breaks, and steer controllers to keep personal data (even encrypted or hashed) off the immutable ledger, with erasure of the off-ledger data rendering on-ledger residue anonymous. The CJEU's SRB judgment (Sept 2025) supports a relative view (data may be non-personal for a party without the key), which strengthens the argument that ciphertext whose key is destroyed everywhere is no longer personal data — but that is an argument, not a ruling on shredding.

### Cited Findings

- EDPB Guidelines 02/2025 on blockchain, Version 2.0, "Adopted on 07 July 2026" (v1.1 adopted 8 April 2025 for consultation). — [EDPB v2.0 PDF](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB para 51: "The EDPB recalls that encrypted personal data is still personal data and encryption does not remove the need for GDPR compliance." — [EDPB v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB para 50: "technical impossibility cannot be invoked to justify non-compliance with GDPR requirements." — [EDPB v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB para 103: controllers should ensure on-chain personal data "can be effectively rendered anonymous if an erasure request or objection is received", which presupposes on-chain data does not directly identify and that off-chain data allowing indirect identification "is erased"; EDPB "recommends looking at other tools if the strong integrity property of blockchains is not needed." — [EDPB v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB para 104: it is "not advisable to register personal data in those forms [clear text, encrypted or hashed] on a blockchain. Instead, personal data in those forms should be stored off-chain." — [EDPB v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB paras 119–120: retention must be set under Art. 17 with Art. 25(1); if retention is shorter than ledger life, a technical solution must allow deletion or anonymisation, "If such solution does not exist, then no personal data should be stored on the chain." Para 116: consent-based processing requires that erasure of off-chain data renders on-chain data anonymous. — [EDPB v2.0](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- EDPB cites the Spanish AEPD technical note "Proof of concept: Blockchain and the right of erasure" and notes "Deletion requires governance and traceability measures" (Recital 66). — [EDPB v2.0 fn 31](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf); [AEPD tech note](https://www.aepd.es/guias/Tech-note-blockchain.pdf) (not fetched)
- Consultation responses (e.g. industry bodies) asked the EDPB to state that erasure can be fulfilled by key destruction; the adopted text does not adopt that wording explicitly. — [Bitkom response](https://www.edpb.europa.eu/sites/default/files/webform/public_consultation_reply/en_ff_bitkom-zu-den-edsa-leitlinien_blockchain_datenschutz_juni-2025.pdf); [GBBC response](https://www.edpb.europa.eu/sites/default/files/webform/public_consultation_reply/gbbc-edpb-draft-guidelines-02_2025.pdf)
- CJEU, EDPS v SRB (C-413/23 P), 4 September 2025: pseudonymised data may be personal for the controller holding re-identification means but not for a recipient that cannot reasonably re-identify; first explicit CJEU confirmation of the relative approach. — [Bird & Bird](https://cm.twobirds.com/en/insights/2025/eu-the-srb-decision-a-new-era-for-personal-data-and-data-processing-agreements); [Clifford Chance](https://www.cliffordchance.com/insights/resources/blogs/talking-tech/en/articles/2025/09/pseudonymized-data-after-edps-v-srb.html); [Jones Day](https://www.jonesday.com/de/insights/2025/09/cjeu-clarifies-scope-of-personal-data-in-edps-v-srb-decision)
- ICO (UK) "beyond use" position (from its older deletion guidance, quoted by secondary sources): satisfied if data is "put beyond use", i.e. the controller will not use it to inform decisions, gives no one else access, secures it, and "commits to permanent deletion of the information if, or when, this becomes possible"; applied to backups. — [The Register analysis](https://www.theregister.com/2018/05/31/backup_gdpr_analysis/); [ComplyDog](https://complydog.com/blog/gdpr-delete-personal-data-from-backups). Primary ico.org.uk page not fetched.
- Counter-view: a lawyer quoted on Verraes' blog said deleting the key is not equal to deleting the data because "Encrypted personal data is still personal data, regardless of whether anyone has the key" (anecdotal, 2019). — [Verraes](https://verraes.net/2019/05/eventsourcing-patterns-throw-away-the-key/)
- European Commission guidance (quoted by Verraes): a breach of state-of-the-art encrypted data with an intact key is still notifiable to the authority but likely not to data subjects. — [Verraes](https://verraes.net/2019/05/eventsourcing-patterns-throw-away-the-key/)
- Unsupported claim to flag: several vendor blogs assert that "EDPB Guidelines 5/2019, the ICO and CNIL all recognize" crypto-shredding as valid erasure. EDPB Guidelines 5/2019 concern the right to be forgotten in search engines, and I found no regulator text supporting the claim; treat as unreliable. — [Granit blog](https://granit-fx.dev/blog/crypto-shredding-gdpr-erasure-without-deleting-rows/); [Chameleon Data](https://www.chameleon-data.com/learn/crypto-shredding)

### Inferences

- Defensible legal position for ParleyInn: crypto-shredding is a strong technical measure that makes residual ciphertext unintelligible and (post-SRB) arguably non-personal for any party without the key; pair it with deletion of plaintext derivatives, destruction of all key copies with an audit record (Recital 66 "traceability"), documented backup windows (ICO "beyond use"), and a crypto-agility/PQ plan (EDPB para 96). Do not market it as "GDPR-compliant erasure" outright.
- EDPB's steer "keep personal data off the immutable ledger" maps to: the hash chain should hold only ciphertext hashes/commitments and non-identifying metadata, with ciphertext in a deletable blob store, so key shredding plus optional blob deletion together approach "actual" deletion where storage allows.
- ParleyInn users are mostly controllers of their own sessions; erasure obligations arise mainly when sessions contain third-party personal data (customer data in code, chat logs).

### Gaps

- No CNIL or ICO page specifically addressing crypto-shredding was fetched; no court decision on crypto-shredding as erasure was found.
- Legal holds vs. shredding: not researched. GDPR Art. 17(3)(b) and (e) exempt erasure where retention is legally required or for legal claims ([GDPR Art. 17](https://gdpr-info.eu/art-17-gdpr/), not fetched); the design implication (a hold must block key destruction and be audited) is an inference.
- EDPB Guidelines 01/2025 on pseudonymisation (final status) not checked.

## Q5. Go libraries and network-freedom

### Takeaway

A network-free Go stack is available entirely from the standard library plus golang.org/x/crypto and filippo.io/age; Tink Go core is local but its KMS integrations are networked; OS keyring libraries reach outside the process (D-Bus on Linux, and on macOS zalando/go-keyring shells out to `/usr/bin/security`, which would violate ParleyInn's I4 ban on `os/exec`). MLS in Go is immature.

### Cited Findings

- filippo.io/age: Go reference implementation; latest release seen v1.3.2; multiple recipients; passphrase and identity-file recipients; hardware tokens and post-quantum support via plugins/`age1pq1` recipients. Plugins run as external binaries found in `$PATH`. — [age README](https://github.com/FiloSottile/age/blob/main/README.md); [pkg.go.dev filippo.io/age](https://pkg.go.dev/filippo.io/age)
- Tink Go: v2 module `github.com/tink-crypto/tink-go/v2`, latest 2.8.0; maintained by Google cryptographers; versions before v2 live in the old monorepo. — [tink-go README](https://github.com/tink-crypto/tink-go/blob/main/README.md)
- zalando/go-keyring: supports macOS, Linux/BSD (D-Bus Secret Service) and Windows; "The OS X implementation depends on the `/usr/bin/security` binary". — [go-keyring README](https://github.com/zalando/go-keyring/blob/master/README.md)
- Go MLS: `github.com/BitravenS/go-mls` (claims RFC 9420), `github.com/thomas-vilte/mls-go` (underlies dave-go, Discord DAVE). Maturity and audits unknown. — [go-mls](https://pkg.go.dev/github.com/BitravenS/go-mls); [dave-go](https://pkg.go.dev/github.com/thomas-vilte/dave-go)
- golang.org/x/crypto provides chacha20poly1305 (incl. XChaCha20 `NewX`) and hkdf; Go 1.24 added `crypto/hkdf` to the standard library. — [x/crypto/chacha20poly1305](https://pkg.go.dev/golang.org/x/crypto/chacha20poly1305); [Go 1.24 release notes](https://go.dev/doc/go1.24) (not fetched this session)
- TPM: `github.com/google/go-tpm` talks to /dev/tpmrm0 locally (no network). — [go-tpm](https://github.com/google/go-tpm) (not fetched)

### Inferences

- Network-free and I4-compatible: stdlib `crypto/*` (AES-GCM, `crypto/hkdf`, `crypto/mlkem`, `crypto/ed25519`), `golang.org/x/crypto/chacha20poly1305`, `filippo.io/age` core (without plugins; plugins use `os/exec`), Tink Go core without KMS extensions.
- Not I4-compatible as-is: KMS clients (network), age plugins and zalando/go-keyring on macOS (`os/exec`). Linux D-Bus is local IPC but uses unix sockets via `net`-family packages in godbus — would need a depguard exception and security review.
- Simplest dependency-minimal choice for ParleyInn's ≤10-dependency budget: stdlib + x/crypto (already common) for AEAD/HKDF, age only if a file format and multi-recipient wrapping are wanted; keystore as files under `$PARINN_HOME` protected by a passphrase/age identity, with optional TPM later.

### Gaps

- Whether godbus imports `net` was not verified; whether age core imports `os/exec` outside the plugin package was not verified.
- No audit status found for Go MLS libraries.

## Q6. Recommended scheme sketch for "append-only, replicated, shreddable"

### Takeaway

Combine (1) a hash-chained/Merkle log whose leaves commit to non-sensitive headers plus SHA-256 of ciphertext (KBFS/tlog pattern), (2) per-session random DEKs (optionally per-event wrapped keys) stored only in a mutable, replicated keystore outside the log, (3) recipient wrapping per device/host (age-style multi-recipient) with lazy forward rotation on revoke (Keybase CLKR), and (4) shredding as an audited, replicated keystore event (a signed tombstone leaf in the log naming the destroyed key ID), so verifiability survives and the destruction is itself tamper-evident.

### Cited Findings

- Ciphertext-addressed blocks + signed Merkle roots + per-block deletable keys: [KBFS crypto spec](https://book.keybase.io/docs/crypto/kbfs)
- Tombstone/anonymised entries kept for integrity of others: [EDPB v2.0 para 64](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- Keyed hash / perfectly hiding commitment on-ledger, data off-ledger: [EDPB v2.0 paras 33–34, 52–54](https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf)
- Signed checkpoints and tiles over arbitrary entries (RFC 6962 hashing): [C2SP tlog-tiles](https://github.com/C2SP/C2SP/blob/main/tlog-tiles.md)
- Lazy rotation on device revoke, new keys boxed to remaining devices: [Keybase CLKR](https://book.keybase.io/docs/teams/clkr)
- Immediate deletion of consumed secrets for forward secrecy: [RFC 9420 §9.2](https://www.rfc-editor.org/rfc/rfc9420.txt)

### Inferences

Sketch (synthesis, not from a single source):

1. Event record = `header || ciphertext`. Header: sequence number, previous-leaf hash, session ID, key ID, timestamp, kind, `SHA-256(ciphertext)`. Chain/Merkle leaf = hash(header). No plaintext hash in the header (or, if needed for dedupe/verification, an HMAC under the session key, which is shredded with it).
2. Ciphertext = XChaCha20-Poly1305 (or AES-256-GCM) under `HKDF(session_DEK, "parinn/event", seq)`, header as associated data, so headers are authenticated and ciphertext cannot be replayed across positions.
3. Keystore (mutable, outside the log): `session_DEK` wrapped to each authorized device/host recipient (X25519, ideally hybrid ML-KEM for PQ) and/or to a project KEK. Shred = delete every wrapped copy of the DEK on every replica; append a signed `KeyDestroyed{key_id, reason, actor}` leaf to the log (I6 audit, Recital 66 traceability).
4. Replication: logs replicate freely (ciphertext + headers are safe to copy to sandboxes); keystore replicates only to authorized devices; cloud sandboxes receive short-lived, session-scoped unwrapped keys or none (git interface over ciphertext-only clones).
5. Revocation: new session DEKs for future data wrapped only to remaining devices (forward rotation); accept that revoked devices keep what they had — the only remedy is shredding the affected session keys everywhere.
6. Projections (search indexes, plaintext caches, git working trees) are derived and rebuildable (I10); a shred event must trigger deterministic rebuild that drops affected plaintext, and verification still passes because the chain never committed to plaintext.
7. Legal hold: a hold flag in the keystore blocks `KeyDestroyed` for covered keys, itself audited.
8. Crypto-agility: key IDs carry an algorithm identifier; periodic reassessment per EDPB para 96; for very long-lived records, also delete ciphertext blobs where replicas allow, rather than relying on ciphertext staying unbreakable forever.

### Gaps

- No peer-reviewed paper was found that specifies exactly this combination for agent session logs; the closest documented systems are KBFS and EDPB's on/off-chain model.
- Academic work on "proofs of deletion" (e.g. verifiable key destruction via TEEs/HSMs) was not researched; key destruction is generally not provable to a third party without trusted hardware.
