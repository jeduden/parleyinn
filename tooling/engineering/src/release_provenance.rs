//! ENG-20: release artifacts are signed with Sigstore, keyless, and
//! carry SLSA Build Level 3 provenance and an SPDX SBOM.
//!
//! A release is not something a checkout can hold, so these checks read
//! the workflows that publish it: `release.yml`, the release-only
//! reusable `release-build.yml` it calls, which attests the build
//! provenance, the `build.yml` that one calls, and `ci.yml`, which calls
//! `build.yml` too. They hold the workflows to the shape that makes each
//! published release verifiable, and the release workflow's own `verify`
//! job runs that verification against the live release, pinned to each
//! signer's exact certificate identity at the one ref a release runs
//! from.

use crate::Checkout;
use crate::review_workflow::{Job, Workflow, parse};

/// The release workflow.
pub const RELEASE: &str = ".github/workflows/release.yml";

/// The reusable workflow only the release calls: it builds through
/// [`BUILD`] and signs the build provenance.
pub const RELEASE_BUILD: &str = ".github/workflows/release-build.yml";

/// The reusable workflow that builds, for CI and for the release alike.
pub const BUILD: &str = ".github/workflows/build.yml";

/// The CI workflow, which calls [`BUILD`] on every change.
pub const CI: &str = ".github/workflows/ci.yml";

/// The one ref a release runs from. Every signer the `verify` job
/// accepts names its workflow at this ref; only reviewed pull requests
/// move it. The tag cannot be that ref: the release creates it last.
pub const RELEASE_REF: &str = "refs/heads/main";

/// The repository the release belongs to, as `gh attestation verify`
/// names it.
const REPO: &str = "jeduden/parleyinn";

/// A workflow of the repository as a Sigstore certificate names it,
/// before its `@ref`.
const WORKFLOWS: &str = "https://github.com/jeduden/parleyinn/.github/workflows/";

/// The call of the reusable build workflow.
const CALLS_BUILD: &str = "uses: ./.github/workflows/build.yml";

/// The call of the release-only reusable workflow.
const CALLS_RELEASE_BUILD: &str = "uses: ./.github/workflows/release-build.yml";

/// The action that attests build provenance.
const PROVENANCE: &str = "actions/attest-build-provenance@";

/// The action that attests an SBOM.
const ATTEST_SBOM: &str = "actions/attest-sbom@";

/// The pinned CLI command that generates the SBOM.
const MAKE_SBOM: &str = "syft . --output ";

/// The action that checks out the repository's code.
const CHECKOUT: &str = "actions/checkout@";

/// The Sigstore bundle that carries the checksum file's signature.
const BUNDLE: &str = "checksums.txt.sigstore.json";

/// The OIDC issuer of GitHub Actions' keyless identity.
const ISSUER: &str = "https://token.actions.githubusercontent.com";

/// The predicate type of an SPDX SBOM attestation.
const SPDX_PREDICATE: &str = "https://spdx.dev/Document/v2.3";

/// The predicate type of SLSA build provenance, `gh attestation
/// verify`'s default.
const SLSA_PREDICATE: &str = "https://slsa.dev/provenance/v1";

/// The two permissions a signature or an attestation needs.
const SIGNING: [&str; 2] = ["id-token: write", "attestations: write"];

/// The `gh attestation verify` options that accept a signer at any
/// ref, and `cosign`'s regular expression for one.
const UNPINNED: [&str; 4] = [
    "--signer-workflow",
    "--signer-repo",
    "--cert-identity-regex",
    "-i",
];

/// The certificate identity of `file`, a workflow of the repository,
/// run from [`RELEASE_REF`].
#[must_use]
pub fn identity(file: &str) -> String {
    format!("{WORKFLOWS}{file}@{RELEASE_REF}")
}

/// The four workflows a release, and CI's build, run through.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Release {
    pub release: Workflow,
    pub release_build: Workflow,
    pub build: Workflow,
    pub ci: Workflow,
}

/// Reads the release workflow, the reusable workflows it runs through
/// and the CI workflow from `checkout`.
///
/// # Errors
///
/// Fails when a file cannot be read.
pub fn load(checkout: &Checkout) -> Result<Release, String> {
    Ok(Release {
        release: parse(&checkout.read(RELEASE)?),
        release_build: parse(&checkout.read(RELEASE_BUILD)?),
        build: parse(&checkout.read(BUILD)?),
        ci: parse(&checkout.read(CI)?),
    })
}

impl Release {
    /// Problems with how the checksum file is signed and verified: it
    /// is signed keyless in the publish job, published with its bundle,
    /// and a later job verifies the bundle against the release
    /// workflow's exact identity at [`RELEASE_REF`].
    #[must_use]
    pub fn checksums_signed_keyless(&self) -> Vec<String> {
        let mut problems = Vec::new();
        match self.publishing() {
            Err(e) => problems.push(e),
            Ok(publish) => problems.extend(signing_problems(publish)),
        }
        match self.verifying() {
            Err(e) => problems.push(e),
            Ok(verify) => problems.extend(cosign_problems(verify)),
        }

        problems
    }

    /// Problems with the binaries' build provenance: it is attested
    /// only in the release-only `release-build.yml`, after the build it
    /// calls; `build.yml`, which CI runs on every change, attests
    /// nothing and is granted nothing to sign with; the release runs
    /// from [`RELEASE_REF`] alone; and a later job verifies every binary
    /// against `release-build.yml`'s exact identity and source ref.
    #[must_use]
    pub fn provenance_from_the_release_build_workflow(&self) -> Vec<String> {
        let mut problems: Vec<String> = self
            .release
            .jobs
            .iter()
            .filter(|j| j.text.contains(PROVENANCE))
            .map(|j| {
                format!(
                    "release.yml attests build provenance outside release-build.yml, in job {}",
                    j.name
                )
            })
            .collect();
        problems.extend(self.build_problems());
        problems.extend(self.release_build_problems());
        problems.extend(self.ci_problems());
        problems.extend(self.caller_problems());
        match self.verifying() {
            Err(e) => problems.push(e),
            Ok(verify) => problems.extend(provenance_verify_problems(verify)),
        }

        problems
    }

    /// Problems with the SBOM: an SPDX document generated, attested to
    /// every binary and attached to the release, and a later job
    /// verifies the attestation against the release workflow's exact
    /// identity and source ref.
    #[must_use]
    pub fn sbom_attached_and_attested(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let sbom = self.sbom_file(&mut problems);
        match self
            .release
            .jobs
            .iter()
            .find(|j| j.text.contains(ATTEST_SBOM))
        {
            None => problems.push("release.yml does not attest the SBOM".to_owned()),
            Some(job) => problems.extend(
                sbom.iter()
                    .filter(|file| value_of(&job.text, "sbom-path").as_ref() != Some(*file))
                    .map(|file| {
                        format!("job {} does not attest the generated SBOM {file}", job.name)
                    }),
            ),
        }
        match self.publishing() {
            Err(e) => problems.push(e),
            Ok(publish) => problems.extend(sbom.iter().filter_map(|file| {
                let name = file.rsplit('/').next().unwrap_or(file);
                (!release_assets(publish).iter().any(|a| a == name))
                    .then(|| format!("the release does not attach the SBOM {name}"))
            })),
        }
        match self.verifying() {
            Err(e) => problems.push(e),
            Ok(verify) => {
                let signer = identity("release.yml");
                let pinned = attestation_commands(verify).iter().any(|c| {
                    flag_value(c, "--predicate-type").as_deref() == Some(SPDX_PREDICATE)
                        && flag_value(c, "--cert-identity").as_ref() == Some(&signer)
                        && flag_value(c, "--source-ref").as_deref() == Some(RELEASE_REF)
                });
                if !pinned {
                    problems.push(format!(
                        "job {} does not verify the SBOM attestation (--predicate-type {SPDX_PREDICATE}) \
                         with --cert-identity {signer} --source-ref {RELEASE_REF}",
                        verify.name
                    ));
                }
            }
        }

        problems
    }

    /// Problems with `build.yml`: CI runs it on every change, so it
    /// takes no `attest` input, attests nothing and asks for neither
    /// signing permission.
    fn build_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.build.on.contains("attest") {
            problems.push("build.yml takes an attest input".to_owned());
        }
        for job in &self.build.jobs {
            if job.text.contains(PROVENANCE) {
                problems.push(format!(
                    "build.yml attests build provenance in job {}; only release-build.yml may",
                    job.name
                ));
            }
            for p in granted(job) {
                problems.push(format!("build.yml job {} asks for {p}", job.name));
            }
        }

        problems
    }

    /// Problems with `release-build.yml`: it builds through `build.yml`,
    /// granting it nothing to sign with, and its attest job holds both
    /// signing permissions, waits for the build and checks out no code.
    fn release_build_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let build = self
            .release_build
            .jobs
            .iter()
            .find(|j| j.text.contains(CALLS_BUILD));
        match build {
            None => problems.push("release-build.yml does not call build.yml".to_owned()),
            Some(job) => problems.extend(
                granted(job)
                    .into_iter()
                    .map(|p| format!("release-build.yml job {} grants {p} to build.yml", job.name)),
            ),
        }
        let Some(attest) = self
            .release_build
            .jobs
            .iter()
            .find(|j| j.text.contains(PROVENANCE))
        else {
            problems.push("release-build.yml does not attest build provenance".to_owned());
            return problems;
        };
        problems.extend(permission_problems(attest));
        if let Some(build) = build
            && !needs(attest).contains(&build.name)
        {
            problems.push(format!(
                "release-build.yml job {} does not wait for job {}",
                attest.name, build.name
            ));
        }
        if attest.text.contains(CHECKOUT) {
            problems.push(format!(
                "release-build.yml job {} checks out the repository's code",
                attest.name
            ));
        }

        problems
    }

    /// Problems with `ci.yml`: its call of `build.yml` is granted
    /// nothing to sign with, and it never calls `release-build.yml`.
    fn ci_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        for job in &self.ci.jobs {
            if job.text.contains(CALLS_BUILD) {
                problems.extend(
                    granted(job)
                        .into_iter()
                        .map(|p| format!("ci.yml job {} grants {p} to build.yml", job.name)),
                );
            }
            if job.text.contains(CALLS_RELEASE_BUILD) {
                problems.push(format!(
                    "ci.yml job {} calls release-build.yml, which only release.yml may call",
                    job.name
                ));
            }
        }

        problems
    }

    /// Problems with the release job that calls `release-build.yml`: it
    /// grants both signing permissions, and runs only after a job that
    /// refuses any ref but [`RELEASE_REF`].
    fn caller_problems(&self) -> Vec<String> {
        let Some(job) = self
            .release
            .jobs
            .iter()
            .find(|j| j.text.contains(CALLS_RELEASE_BUILD))
        else {
            return vec!["release.yml does not call release-build.yml".to_owned()];
        };
        let mut problems = permission_problems(job);
        let guarded = needs(job).iter().any(|name| {
            self.release
                .jobs
                .iter()
                .any(|j| j.name == *name && j.text.contains(RELEASE_REF))
        });
        if !guarded {
            problems.push(format!(
                "release.yml builds without refusing a ref other than {RELEASE_REF}"
            ));
        }

        problems
    }

    /// The SPDX file the release workflow generates, or why there is
    /// none.
    fn sbom_file(&self, problems: &mut Vec<String>) -> Option<String> {
        let Some(job) = self
            .release
            .jobs
            .iter()
            .find(|j| j.text.contains(MAKE_SBOM))
        else {
            problems.push("release.yml does not generate an SBOM".to_owned());
            return None;
        };
        let output = job.text.lines().find_map(|line| {
            line.trim()
                .strip_prefix("- ")
                .unwrap_or(line.trim())
                .strip_prefix("run: ")?
                .strip_prefix(MAKE_SBOM)
        });
        let (format, file) = output
            .and_then(|spec| spec.split_once('='))
            .unwrap_or(("", ""));
        if format != "spdx-json" {
            problems.push(format!("job {} does not generate an SPDX SBOM", job.name));
        }
        let file = (!file.trim().is_empty()).then(|| file.trim().to_owned());
        if file.is_none() {
            problems.push(format!("job {} names no SBOM file", job.name));
        }

        file
    }

    /// The job that creates the release.
    fn publishing(&self) -> Result<&Job, String> {
        self.release
            .jobs
            .iter()
            .find(|j| logical(&j.text).contains("gh release create"))
            .ok_or_else(|| "no job in release.yml creates the release".to_owned())
    }

    /// The job that verifies the published release: one that runs only
    /// after the publishing job.
    fn verifying(&self) -> Result<&Job, String> {
        let publish = self.publishing()?;
        self.release
            .jobs
            .iter()
            .find(|j| needs(j).contains(&publish.name))
            .ok_or_else(|| {
                format!(
                    "no job verifies the published release after job {}",
                    publish.name
                )
            })
    }
}

/// Problems with the publishing job's keyless signature and assets.
fn signing_problems(publish: &Job) -> Vec<String> {
    let text = logical(&publish.text);
    let mut problems = Vec::new();
    let sign = format!("cosign sign-blob --bundle {BUNDLE} checksums.txt");
    if !text.contains(&sign) {
        problems.push(format!("job {} does not run {sign:?}", publish.name));
    }
    if text.contains("--key") {
        problems.push(format!(
            "job {} signs with a key, not keyless",
            publish.name
        ));
    }
    if !text.contains("id-token: write") {
        problems.push(format!(
            "job {} lacks the id-token write permission for a keyless signature",
            publish.name
        ));
    }
    let assets = release_assets(publish);
    for want in ["checksums.txt", BUNDLE] {
        if !assets.iter().any(|a| a == want) {
            problems.push(format!("the release does not attach {want}"));
        }
    }

    problems
}

/// Problems with the verify job's check of the checksum bundle.
fn cosign_problems(verify: &Job) -> Vec<String> {
    let text = logical(&verify.text);
    let Some(line) = text.lines().find(|l| l.contains("cosign verify-blob")) else {
        return vec![format!(
            "job {} does not run cosign verify-blob",
            verify.name
        )];
    };
    let mut problems = Vec::new();
    if !line.contains(&format!("--bundle {BUNDLE}")) {
        problems.push(format!("job {} verifies no {BUNDLE}", verify.name));
    }
    let signer = identity("release.yml");
    if flag_value(line, "--certificate-identity").as_ref() != Some(&signer) {
        problems.push(format!(
            "job {} does not pin the certificate identity to {signer}",
            verify.name
        ));
    }
    if !line.contains(&format!("--certificate-oidc-issuer {ISSUER}")) {
        problems.push(format!(
            "job {} does not pin the OIDC issuer {ISSUER}",
            verify.name
        ));
    }

    problems
}

/// The two permissions an attestation needs, as `job` must hold them.
fn permission_problems(job: &Job) -> Vec<String> {
    SIGNING
        .into_iter()
        .filter(|p| !job.text.contains(p))
        .map(|p| format!("job {} lacks {p}", job.name))
        .collect()
}

/// The signing permissions `job` grants or asks for.
fn granted(job: &Job) -> Vec<&'static str> {
    SIGNING
        .into_iter()
        .filter(|p| job.text.contains(p))
        .collect()
}

/// Problems with the verify job's check of every binary's build
/// provenance: pinned to `release-build.yml`'s exact certificate
/// identity, its source ref and a GitHub-hosted runner, and no
/// attestation accepted from a signer at any ref.
fn provenance_verify_problems(verify: &Job) -> Vec<String> {
    let mut problems = Vec::new();
    let signer = identity("release-build.yml");
    let commands = attestation_commands(verify);
    let verifies = commands.iter().any(|c| {
        flag_value(c, "--repo").as_deref() == Some(REPO)
            && flag_value(c, "--cert-identity").as_ref() == Some(&signer)
            && flag_value(c, "--source-ref").as_deref() == Some(RELEASE_REF)
            && c.split_whitespace()
                .any(|w| w == "--deny-self-hosted-runners")
            && flag_value(c, "--predicate-type").is_none_or(|t| t == SLSA_PREDICATE)
    });
    if !verifies {
        problems.push(format!(
            "job {} does not verify build provenance with --repo {REPO} \
             --cert-identity {signer} --source-ref {RELEASE_REF} --deny-self-hosted-runners",
            verify.name
        ));
    }
    for flag in UNPINNED {
        if commands.iter().any(|c| names_flag(c, flag)) {
            problems.push(format!(
                "job {} verifies an attestation with {flag}, which pins no ref",
                verify.name
            ));
        }
    }
    if !verify.text.contains("parinn-*") {
        problems.push(format!(
            "job {} does not verify every binary (parinn-*)",
            verify.name
        ));
    }

    problems
}

/// Whether `command` passes `flag`, alone or as `flag=value`.
fn names_flag(command: &str, flag: &str) -> bool {
    command
        .split_whitespace()
        .any(|w| w == flag || w.strip_prefix(flag).is_some_and(|r| r.starts_with('=')))
}

/// The value `command` gives `flag`, as the next word or after `=`,
/// without quotes.
fn flag_value(command: &str, flag: &str) -> Option<String> {
    let mut words = command.split_whitespace();
    while let Some(w) = words.next() {
        let value = if w == flag {
            words.next()
        } else {
            w.strip_prefix(flag).and_then(|r| r.strip_prefix('='))
        };
        if let Some(v) = value {
            return Some(v.trim_matches(['"', '\'']).to_owned());
        }
    }

    None
}

/// The `gh attestation verify` commands of `job`, each as one line.
fn attestation_commands(job: &Job) -> Vec<String> {
    logical(&job.text)
        .lines()
        .filter(|l| l.contains("gh attestation verify"))
        .map(str::to_owned)
        .collect()
}

/// The names `gh release create` attaches in `job`: the words after
/// `--generate-notes`, the last option the repository's workflow gives.
fn release_assets(job: &Job) -> Vec<String> {
    let text = logical(&job.text);
    text.lines()
        .find(|l| l.contains("gh release create"))
        .map(|line| {
            line.split_whitespace()
                .skip_while(|w| *w != "--generate-notes")
                .skip(1)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The jobs `job` needs, from a one-line `needs:`.
fn needs(job: &Job) -> Vec<String> {
    job.text
        .lines()
        .find_map(|l| l.strip_prefix("    needs:"))
        .map(|rest| {
            rest.trim()
                .trim_matches(['[', ']'])
                .split(',')
                .map(|n| n.trim().to_owned())
                .filter(|n| !n.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// The value a `key:` line gives in `text`, without quotes.
fn value_of(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        l.trim_start()
            .strip_prefix(key)?
            .strip_prefix(':')
            .map(|v| v.trim().trim_matches(['"', '\'']).to_owned())
    })
}

/// `text` with each shell line continuation joined, so a command is one
/// line however the workflow wraps it.
fn logical(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines().map(str::trim) {
        match line.strip_suffix('\\') {
            Some(head) => {
                out.push_str(head.trim_end());
                out.push(' ');
            }
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }

    out
}

#[cfg(test)]
mod tests;
