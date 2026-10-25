//! ENG-30: every crate a release target links into a shipped executable
//! carries a license text in its published source, and each text fits the
//! canonical text of a license in the crate's declared SPDX expression.
//! The canonical texts are kept here, one file each under
//! `license_texts/canon`, with `{{}}` for the holder or name a template
//! fills in. The check reads only `cargo metadata` and the sources cargo
//! has already fetched. The checks of the repository's own records (the
//! substitute texts it keeps, `deny.toml`'s grants against the canonical
//! texts) have their drift cases (ENG-27).

pub mod cli;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::{Checkout, architecture, dependencies};

/// The canonical text of each license the allow-list in `deny.toml`
/// and its exceptions grant, by SPDX id. The text of a license a
/// template fills in leaves the holder, year and name out.
const CANON: [(&str, &str); 8] = [
    (
        "Apache-2.0",
        include_str!("license_texts/canon/Apache-2.0.txt"),
    ),
    (
        "BSD-2-Clause",
        include_str!("license_texts/canon/BSD-2-Clause.txt"),
    ),
    (
        "BSD-3-Clause",
        include_str!("license_texts/canon/BSD-3-Clause.txt"),
    ),
    (
        "BlueOak-1.0.0",
        include_str!("license_texts/canon/BlueOak-1.0.0.txt"),
    ),
    ("ISC", include_str!("license_texts/canon/ISC.txt")),
    ("MIT", include_str!("license_texts/canon/MIT.txt")),
    ("NCSA", include_str!("license_texts/canon/NCSA.txt")),
    (
        "Unicode-3.0",
        include_str!("license_texts/canon/Unicode-3.0.txt"),
    ),
];

/// Apache-2.0's appendix, which a file may add after the license text.
const APACHE_APPENDIX: &str = include_str!("license_texts/canon/Apache-2.0.appendix.txt");

/// What a template fills in, in a canonical text.
const FREE: &str = "{{}}";

/// A crate from a registry, found in `cargo metadata`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    /// The SPDX expression the crate declares, if any.
    pub license: Option<String>,
    /// The directory holding the crate's fetched source.
    pub dir: PathBuf,
}

/// The ids of the licenses the tool keeps a canonical text for.
#[must_use]
pub fn ids() -> Vec<&'static str> {
    CANON.iter().map(|(id, _)| *id).collect()
}

/// The canonical text of license `id`, if the tool keeps one.
#[must_use]
pub fn canonical(id: &str) -> Option<&'static str> {
    CANON.iter().find(|(i, _)| *i == id).map(|(_, text)| *text)
}

/// A text as the comparison sees it: lowercase, copyright lines gone,
/// whitespace collapsed to single spaces.
#[must_use]
pub fn normalize(text: &str) -> String {
    text.lines()
        .filter(|line| !copyright_line(line))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether `line` states who holds the copyright and since when,
/// rather than wrapping a sentence that mentions the word. A clause that
/// starts "(c)", such as Apache-2.0's section 4(c), is license text.
pub(crate) fn copyright_line(line: &str) -> bool {
    let line = line.trim().to_lowercase();
    let stated = line.starts_with("copyright")
        && (line.contains("(c)") || line.contains('©') || line.contains("yyyy") || has_year(&line));

    stated || line.starts_with('©')
}

/// Whether `line` holds a year: four digits starting with 1 or 2. A wrapped
/// sentence such as "copyright holder, ... prior to 30 days after" holds
/// none.
fn has_year(line: &str) -> bool {
    line.as_bytes()
        .windows(4)
        .any(|w| matches!(w[0], b'1' | b'2') && w.iter().all(u8::is_ascii_digit))
}

/// How many words a title may take before the canonical text: "MIT
/// License", "The MIT License (MIT)", "Apache License Version 2.0".
const TITLE_WORDS: usize = 8;

/// How many words a [`FREE`] gap may take: a name, not a clause.
const GAP_WORDS: usize = 12;

/// Whether `text` fits `canon`: it is the canonical text, ignoring case,
/// whitespace and copyright lines, with each [`FREE`] standing for a
/// short run of words and a title before it. Nothing may follow it, so a term
/// added after the text does not fit, except Apache-2.0's optional
/// appendix.
#[must_use]
pub fn fits(text: &str, canon_raw: &str) -> bool {
    let text = normalize(text);
    let canon = normalize(canon_raw);
    let parts: Vec<&str> = canon.split(FREE).map(str::trim).collect();
    let Some(first) = text.find(parts[0]) else {
        return false;
    };
    if text[..first].split_whitespace().count() > TITLE_WORDS {
        return false;
    }
    let mut rest = &text[first + parts[0].len()..];
    for part in &parts[1..] {
        let Some(at) = rest.find(part) else {
            return false;
        };
        if rest[..at].split_whitespace().count() > GAP_WORDS {
            return false;
        }
        rest = &rest[at + part.len()..];
    }

    rest.trim().is_empty() || (canon_is_apache(canon_raw) && appendix(rest))
}

/// Whether `canon` is Apache-2.0's text, the one license with an appendix.
fn canon_is_apache(canon: &str) -> bool {
    canonical("Apache-2.0") == Some(canon)
}

/// Whether `rest`, the words after Apache-2.0's text, are exactly its
/// appendix, whichever of `{}` or `[]` marks the fields a work fills in.
fn appendix(rest: &str) -> bool {
    let plain = |text: &str| text.replace(['{', '}', '[', ']'], "");

    plain(rest).trim() == plain(&normalize(APACHE_APPENDIX)).trim()
}

/// An SPDX license expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Id(String),
    And(Vec<Expr>),
    Or(Vec<Expr>),
}

/// Parses an SPDX expression: `OR` binds looser than `AND`, parentheses
/// group, an exception (`WITH ...`) is dropped, and the legacy `/` counts
/// as `OR`.
///
/// # Errors
///
/// Fails on an empty expression or unbalanced parentheses.
pub fn parse(expression: &str) -> Result<Expr, String> {
    let spaced = expression
        .replace('(', " ( ")
        .replace(')', " ) ")
        .replace('/', " OR ");
    let mut tokens = spaced.split_whitespace().peekable();
    let expr = parse_or(&mut tokens)?;
    match tokens.next() {
        None => Ok(expr),
        Some(extra) => Err(format!("{expression:?}: unexpected {extra:?}")),
    }
}

type Tokens<'a> = std::iter::Peekable<std::str::SplitWhitespace<'a>>;

fn parse_or(tokens: &mut Tokens) -> Result<Expr, String> {
    let mut parts = vec![parse_and(tokens)?];
    while tokens.next_if_eq(&"OR").is_some() {
        parts.push(parse_and(tokens)?);
    }

    Ok(if parts.len() == 1 {
        parts.remove(0)
    } else {
        Expr::Or(parts)
    })
}

fn parse_and(tokens: &mut Tokens) -> Result<Expr, String> {
    let mut parts = vec![parse_term(tokens)?];
    while tokens.next_if_eq(&"AND").is_some() {
        parts.push(parse_term(tokens)?);
    }

    Ok(if parts.len() == 1 {
        parts.remove(0)
    } else {
        Expr::And(parts)
    })
}

fn parse_term(tokens: &mut Tokens) -> Result<Expr, String> {
    match tokens.next() {
        Some("(") => {
            let inner = parse_or(tokens)?;
            (tokens.next() == Some(")"))
                .then_some(inner)
                .ok_or_else(|| "missing closing parenthesis".to_owned())
        }
        Some(id @ ("AND" | "OR" | ")")) => Err(format!("unexpected {id:?}")),
        Some(id) => {
            if tokens.next_if_eq(&"WITH").is_some() {
                tokens.next().ok_or("WITH names no exception")?;
            }
            Ok(Expr::Id(id.to_owned()))
        }
        None => Err("expression ends early".to_owned()),
    }
}

/// What keeps `expr` from being satisfied when `met` says which licenses
/// are: the ids still unmet, `None` when it is satisfied.
fn unmet(expr: &Expr, met: &dyn Fn(&str) -> bool) -> Option<Vec<String>> {
    match expr {
        Expr::Id(id) => (!met(id)).then(|| vec![id.clone()]),
        Expr::And(parts) => {
            let all: Vec<String> = parts
                .iter()
                .filter_map(|p| unmet(p, met))
                .flatten()
                .collect();
            (!all.is_empty()).then_some(all)
        }
        Expr::Or(parts) => {
            let all: Option<Vec<Vec<String>>> = parts.iter().map(|p| unmet(p, met)).collect();
            all.map(|lists| lists.into_iter().flatten().collect())
        }
    }
}

/// Whether a file name is one a crate keeps a license text in.
#[must_use]
pub fn license_file_name(name: &str) -> bool {
    let name = name.to_lowercase();

    ["license", "licence", "copying", "unlicense"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// The registry crates `metadata` lists: every package with a source,
/// which leaves out the workspace's own crates.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON, or lists a crate without
/// a name, version or manifest path, which would go unchecked.
pub fn packages(metadata: &str) -> Result<Vec<Package>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let mut found = Vec::new();
    for p in value["packages"].as_array().into_iter().flatten() {
        if p["source"].is_string() {
            found.push(package_of(p)?);
        }
    }
    found.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

    Ok(found)
}

/// One package of `cargo metadata`.
fn package_of(p: &serde_json::Value) -> Result<Package, String> {
    (|| {
        Some(Package {
            name: p["name"].as_str()?.to_owned(),
            version: p["version"].as_str()?.to_owned(),
            license: p["license"].as_str().map(str::to_owned),
            dir: Path::new(p["manifest_path"].as_str()?)
                .parent()?
                .to_path_buf(),
        })
    })()
    .ok_or_else(|| {
        format!("cargo metadata lists a crate without a name, version or manifest path: {p}")
    })
}

/// The license files in `dir`, as (name, text), in name order: those
/// named for a license at the top, and any file in a REUSE-style
/// `LICENSES` directory, listed as `LICENSES/<name>`.
///
/// # Errors
///
/// Fails when the directory cannot be read.
pub fn license_files(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    for (sub, prefix) in [("", ""), ("LICENSES", "LICENSES/")] {
        files.extend(files_in(&dir.join(sub), prefix)?);
    }
    files.sort();

    Ok(files)
}

/// The files in `dir` listed under `prefix` and their names: with no
/// prefix, those named for a license; under `LICENSES/`, every file. A
/// `LICENSES` directory that does not exist holds nothing.
fn files_in(dir: &Path, prefix: &str) -> Result<Vec<(String, String)>, String> {
    let entries = match fs::read_dir(dir) {
        Err(e) if !prefix.is_empty() && e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        other => other.map_err(|e| format!("read {}: {e}", dir.display()))?,
    };
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("read {}: {e}", dir.display()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if (!prefix.is_empty() || license_file_name(&name)) && entry.path().is_file() {
            // Not UTF-8 means no canonical text fits it; lossy keeps it listed.
            let bytes = fs::read(entry.path()).map_err(|e| format!("read {name}: {e}"))?;
            files.push((
                format!("{prefix}{name}"),
                String::from_utf8_lossy(&bytes).into_owned(),
            ));
        }
    }

    Ok(files)
}

/// How a problem with a crate that keeps no license file ends.
const NO_FILE: &str = "ships no license file";

/// Splits the problems of [`check_packages`] into those about a missing
/// license file or an unreadable crate directory, and those about a
/// text that does not fit.
#[must_use]
pub fn split_problems(problems: Vec<String>) -> (Vec<String>, Vec<String>) {
    problems
        .into_iter()
        .partition(|p| p.ends_with(NO_FILE) || p.starts_with("read "))
}

/// Problems with one crate: no license file, no license declared, an
/// expression that does not parse, or a part of the expression no file
/// fits, with a note for each license the tool keeps no text for.
#[must_use]
pub fn check_crate(package: &Package, files: &[(String, String)]) -> Vec<String> {
    let who = format!("{} {}", package.name, package.version);
    if files.is_empty() {
        return vec![format!("{who} {NO_FILE}")];
    }
    let Some(expression) = &package.license else {
        return vec![format!("{who} declares no license")];
    };
    let expr = match parse(expression) {
        Ok(expr) => expr,
        Err(e) => return vec![format!("{who} declares a license that does not parse: {e}")],
    };
    let met = |id: &str| {
        canonical(id).is_some_and(|canon| files.iter().any(|(_, text)| fits(text, canon)))
    };

    unmet(&expr, &met)
        .map(|ids| {
            let no_text: Vec<&str> = ids
                .iter()
                .map(String::as_str)
                .filter(|id| canonical(id).is_none())
                .collect();
            let note = if no_text.is_empty() {
                String::new()
            } else {
                format!(" (the tool keeps no text for {})", no_text.join(", "))
            };
            format!(
                "{who} declares {expression}, but no license file fits {}{note}",
                ids.join(" or ")
            )
        })
        .into_iter()
        .collect()
}

/// Problems with every crate in `metadata`, reading each crate's
/// license files through `read`.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON.
pub fn check_tree(
    metadata: &str,
    read: impl Fn(&Path) -> Result<Vec<(String, String)>, String>,
) -> Result<Vec<String>, String> {
    let mut problems = Vec::new();
    for package in packages(metadata)? {
        match read(&package.dir) {
            Ok(files) => problems.extend(check_crate(&package, &files)),
            Err(e) => problems.push(e),
        }
    }

    Ok(problems)
}

/// Problems with the canonical texts themselves: a license `deny.toml`
/// allows, or grants by exception, that the tool keeps no text for.
#[must_use]
pub fn canon_covers(deny_toml: &str) -> Vec<String> {
    let mut wanted: BTreeSet<String> = deny_allow(deny_toml).into_iter().collect();
    wanted.extend(
        dependencies::license_exceptions(deny_toml)
            .into_values()
            .flatten(),
    );

    wanted
        .into_iter()
        .filter(|id| canonical(id).is_none())
        .map(|id| format!("deny.toml grants {id}, which the license tool keeps no text for"))
        .collect()
}

/// The licenses `deny.toml`'s top-level `allow = [...]` lists, on one
/// line or several.
fn deny_allow(deny_toml: &str) -> Vec<String> {
    let Some(at) = deny_toml
        .lines()
        .scan(0, |offset, line| {
            let start = *offset;
            *offset += line.len() + 1;
            Some((start, line))
        })
        .find_map(|(start, line)| line.trim().starts_with("allow = [").then_some(start))
    else {
        return Vec::new();
    };
    let body = &deny_toml[at..];
    let list = body
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map_or("", |(list, _)| list);

    list.lines()
        .map(|l| l.split('#').next().unwrap_or_default())
        .flat_map(|l| l.split(','))
        .map(|l| l.trim().trim_matches('"').to_owned())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Problems with license texts the repository keeps for a crate: a
/// license file under `licenses/`, `third_party/` or `vendor/` in a
/// directory named for a crate, or for it and its version, other than a
/// target's committed `THIRD_PARTY_LICENSES` (ENG-31). A directory the
/// check cannot read is a problem too. A crate that ships no text is
/// removed or replaced, never patched here.
#[must_use]
pub fn kept_texts(checkout: &Checkout, packages: &[Package]) -> Vec<String> {
    let names: BTreeSet<String> = packages
        .iter()
        .flat_map(|p| [p.name.clone(), format!("{}-{}", p.name, p.version)])
        .collect();
    let (mut files, mut found) = (Vec::new(), Vec::new());
    for top in ["licenses", "third_party", "vendor"] {
        walk(&checkout.root.join(top), &mut files, &mut found);
    }
    for path in files {
        let in_crate_dir = path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|d| names.contains(d.to_string_lossy().as_ref()));
        let named = path
            .file_name()
            .is_some_and(|n| license_file_name(&n.to_string_lossy()));
        if in_crate_dir && named {
            found.push(format!(
                "the repository keeps a license text at {}",
                path.strip_prefix(&checkout.root).unwrap_or(&path).display()
            ));
        }
    }
    found.sort();

    found
}

/// Lists in `files` each file under `dir`, and notes in `problems` a
/// directory it cannot read. A directory that does not exist holds
/// nothing.
fn walk(dir: &Path, files: &mut Vec<PathBuf>, problems: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
        Err(e) => {
            problems.push(format!("read {}: {e}", dir.display()));
            return;
        }
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            walk(&path, files, problems);
        } else {
            files.push(path);
        }
    }
}

/// The registry crates linked into a shipped executable: those reached
/// through normal dependencies from each binary under `product`, which
/// `cargo metadata --filter-platform <target>` resolved. A crate that
/// only builds, tests or fuzzes, and a proc-macro with what only it
/// reaches, is not linked.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON or holds no resolved graph.
pub fn linked(metadata: &str, product: &Path) -> Result<Vec<Package>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let nodes: BTreeMap<&str, &serde_json::Value> = value["resolve"]["nodes"]
        .as_array()
        .ok_or("cargo metadata holds no resolved graph")?
        .iter()
        .filter_map(|n| Some((n["id"].as_str()?, n)))
        .collect();
    let root = value["workspace_root"]
        .as_str()
        .ok_or("cargo metadata names no workspace_root")?;
    let product = Path::new(root).join(product);
    let mut todo: Vec<&str> = value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| dependencies::ships(p, &product))
        .filter_map(|p| p["id"].as_str())
        .collect();
    let proc_macros: BTreeSet<&str> = value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| {
            p["targets"].as_array().into_iter().flatten().any(|t| {
                t["kind"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|k| k == "proc-macro")
            })
        })
        .filter_map(|p| p["id"].as_str())
        .collect();
    let mut seen = BTreeSet::new();
    while let Some(id) = todo.pop() {
        // A proc-macro runs in the compiler and links into nothing.
        if proc_macros.contains(id) || !seen.insert(id) {
            continue;
        }
        let deps = nodes.get(id).map_or(&[][..], |n| {
            n["deps"].as_array().map_or(&[][..], Vec::as_slice)
        });
        todo.extend(
            deps.iter()
                .filter(|d| {
                    d["dep_kinds"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .any(|k| k["kind"].is_null())
                })
                .filter_map(|d| d["pkg"].as_str()),
        );
    }

    let mut found = Vec::new();
    for p in value["packages"].as_array().into_iter().flatten() {
        if p["source"].is_string() && p["id"].as_str().is_some_and(|id| seen.contains(id)) {
            found.push(package_of(p)?);
        }
    }
    found.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

    Ok(found)
}

/// The release targets the build workflow names: the `rust-target` of
/// each `rust-target:artifact-name` pair in its `*_TARGETS` lists.
///
/// # Errors
///
/// Fails when the workflow names no target.
pub fn release_targets(build_workflow: &str) -> Result<Vec<String>, String> {
    let mut targets = Vec::new();
    let mut listing = false;
    for line in build_workflow.lines().map(str::trim) {
        if line.ends_with("_TARGETS: >-") {
            listing = true;
        } else if listing {
            match line.split_once(':') {
                Some((target, artifact)) if !line.contains(' ') && !artifact.is_empty() => {
                    targets.push(target.to_owned());
                }
                _ => listing = false,
            }
        }
    }
    if targets.is_empty() {
        return Err("the build workflow names no release target".to_owned());
    }
    targets.sort();
    targets.dedup();

    Ok(targets)
}

/// How long `cargo metadata` may take for one target (ENG-03).
const METADATA_DEADLINE: Duration = Duration::from_secs(600);

/// Waits for `child`, whose output is piped, until `deadline`, then kills
/// it and says so.
///
/// # Errors
///
/// Fails when the child is still running at the deadline.
pub fn finish_within(mut child: Child, deadline: Duration) -> Result<Output, String> {
    let drain = |pipe: Option<Box<dyn Read + Send>>| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut pipe) = pipe {
                let _ = pipe.read_to_end(&mut bytes);
            }
            bytes
        })
    };
    let out = drain(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let err = drain(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let started = Instant::now();
    let status: ExitStatus = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("wait for cargo: {e}"))?
        {
            break status;
        }
        if started.elapsed() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("cargo did not finish within {deadline:?}"));
        }
        thread::sleep(Duration::from_millis(20));
    };

    Ok(Output {
        status,
        stdout: out.join().unwrap_or_default(),
        stderr: err.join().unwrap_or_default(),
    })
}

/// Runs `cargo metadata`, offline, on the workspace `manifest` resolved for
/// `target`, with `PARINN_HOME` pointed at `parinn_home` (ENG-14).
///
/// # Errors
///
/// Fails when cargo cannot run, exits unsuccessfully, or prints no UTF-8.
pub fn cargo_metadata_for(
    cargo: &OsStr,
    manifest: &Path,
    parinn_home: &Path,
    target: &str,
) -> Result<String, String> {
    let out = Command::new(cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--offline",
            "--filter-platform",
        ])
        .arg(target)
        .arg("--manifest-path")
        .arg(manifest)
        .env("PARINN_HOME", parinn_home)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("run cargo metadata: {e}"))
        .and_then(|child| finish_within(child, METADATA_DEADLINE))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata for {target} failed (cargo fetch --locked fetches the sources): {}",
            String::from_utf8_lossy(&out.stderr).trim_end()
        ));
    }

    String::from_utf8(out.stdout).map_err(|e| format!("read cargo metadata: {e}"))
}

/// The crates linked into a shipped executable for any of `targets`,
/// from each target's `metadata`, once each.
///
/// # Errors
///
/// Fails when a target's metadata cannot be read.
pub fn linked_for_release(
    targets: &[String],
    metadata: impl Fn(&str) -> Result<String, String>,
) -> Result<Vec<Package>, String> {
    let mut all: BTreeMap<(String, String), Package> = BTreeMap::new();
    for target in targets {
        for package in linked(&metadata(target)?, Path::new(architecture::PRODUCT))? {
            all.insert((package.name.clone(), package.version.clone()), package);
        }
    }

    Ok(all.into_values().collect())
}

/// Problems with the crates a release links: those of
/// [`check_crate`] for each, reading its files through `read`.
pub fn check_packages(
    packages: &[Package],
    read: impl Fn(&Path) -> Result<Vec<(String, String)>, String>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for package in packages {
        match read(&package.dir) {
            Ok(files) => problems.extend(check_crate(package, &files)),
            Err(e) => problems.push(e),
        }
    }

    problems
}

/// Problems with ENG-30's CI step: a step, not a comment, runs
/// `license-check` with the locked dependency tree, and the workflow
/// runs on every pull request.
#[must_use]
pub fn ci_runs_check(ci_workflow: &str) -> Vec<String> {
    let runs = ci_workflow.lines().map(str::trim).any(|l| {
        l.starts_with("- run:") && l.contains("--bin license-check") && l.contains("--locked")
    });
    let on_pull_requests = ci_workflow
        .lines()
        .any(|l| l.trim_end() == "  pull_request:");
    let mut problems = Vec::new();
    if !runs {
        problems.push("the CI workflow does not run license-check with --locked".to_owned());
    }
    if !on_pull_requests {
        problems.push("the CI workflow does not run on every pull request".to_owned());
    }

    problems
}

/// Problems with the repository for ENG-30: the crates a release links
/// (`linked`) with no fitting license file, the canonical texts
/// `deny.toml` grants that the tool lacks, and texts the repository
/// keeps for a crate.
#[must_use]
pub fn check_repository(checkout: &Checkout, linked: &[Package]) -> Vec<String> {
    let mut problems = check_packages(linked, license_files);
    problems.extend(
        checkout
            .read("deny.toml")
            .map_or_else(|e| vec![e], |deny| canon_covers(&deny)),
    );
    problems.extend(kept_texts(checkout, linked));

    problems
}

/// Problems with what the product links of the check: a package under
/// `crates/` that depends on the `engineering` crate, which holds it,
/// would ship it.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON.
pub fn product_links_check(metadata: &str) -> Result<Vec<String>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let root = value["workspace_root"].as_str().unwrap_or_default();
    let product = Path::new(root).join(architecture::PRODUCT);

    Ok(value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| {
            p["manifest_path"]
                .as_str()
                .is_some_and(|m| Path::new(m).starts_with(&product))
        })
        .filter(|p| {
            p["dependencies"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|d| d["name"] == "engineering" && d["kind"].is_null())
        })
        .map(|p| {
            format!(
                "{} depends on the engineering crate, which would ship the license check",
                p["name"].as_str().unwrap_or_default()
            )
        })
        .collect())
}

/// The workflow that names the release targets.
pub const BUILD_WORKFLOW: &str = ".github/workflows/build.yml";

#[cfg(test)]
pub(crate) mod tests;
