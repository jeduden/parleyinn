---
n: 1
title: "Every crate has a license text, checked on each pull request"
status: "✅"
result: false
---
# Phase 1: every crate has a license text, checked on each pull request

Requirements. Closes ENG-30. Takes the `@ENG-30` scenario off
`@pending`. ENG-31 stays pending.

BDD coverage: ENG-30 inspects the repository checkout. Its scenario is
bound in [the BDD steps](../../tooling/scenario/tests/bdd/),
beside ENG-20's. No cross-host or lease behavior is involved.

RED spec, in `tooling/engineering` (unit tests beside the module, over
fixture `cargo metadata` output):

- a crate whose source holds a license file passes;
- a crate with none fails, naming the crate and version, and the
  repository keeps no substitute text;
- a license file whose text does not match the declared license fails;
- the check reads only `Cargo.lock`, `cargo metadata` and fetched
  sources, and runs offline.

GREEN sites:

- a license-text check in `tooling/engineering`, sharing the
  linked-set reader ENG-18 uses;
- a command line entry;
- a CI step that runs it with `--locked` on every pull request
  (stakeholder path `.github/workflows/ci.yml`);
- a removal or replacement for any crate in the tree today that ships
  no license file.

Drift case (ENG-27). Add one to `tooling/drift/src/cases.rs`: an edit
that adds a lockfile package with no license text must make the ENG-30
scenario fail. ENG-27 requires it because the check inspects the
repository's records.

Gate. Run the built tool against the real tree in an isolated
`PARINN_HOME`, then against a scratch copy with one crate's license file
removed, and confirm the second run exits non-zero naming that crate.

## Result

Closes ENG-30. The `@ENG-30` scenario is off `@pending` and passes,
bound in
[the ENG-30 steps](../../tooling/scenario/tests/bdd/engineering_licenses.rs).
`@ENG-31` stays pending for phases 2 and 3.

On the stakeholder's ruling of this session, the check covers the
crates a release links, not the whole dependency tree. A crate is linked
when a normal-dependency path from a binary under `crates/` reaches it
with `cargo metadata --filter-platform <target-triple>` for any release
target the build workflow names. That leaves out the cucumber, peg and
synthez test stack, the fuzz workspace, `r-efi` and the ICU4X crates,
which ship no license text or add trailing attribution but link into no
shipped executable. ENG-30's wording changed to match.

- The check is in `tooling/engineering/src/license_texts.rs`, with the
  canonical text of every license `deny.toml` allows or grants. It
  refuses a term added after the text, a clause dropped, and a template
  gap longer than a name.
- `license-check` runs it, and CI's supply-chain job runs that with
  `--locked`.
- Three drift cases catch a license `deny.toml` grants with no
  canonical text, a canonical text that no longer fits, and CI that
  stops running the check.
- Run over the real tree, the check passes.
