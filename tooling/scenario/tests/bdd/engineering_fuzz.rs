//! The ENG-07 steps: each parser the requirement names has a cargo-fuzz
//! target in the fuzz workspace, its seed corpus is committed under
//! `fuzz/corpus`, and the nightly workflow fuzzes it from that corpus.
//! The fuzzing itself runs in the nightly job; these steps hold the
//! checkout to it.

use std::fs;

use cucumber::{given, then};
use engineering::{Checkout, fuzzing, outcome};

use crate::World;

/// The target the scenario's input names.
#[derive(Debug, Default)]
struct Fuzzing {
    target: Option<&'static str>,
}

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

fn target(w: &mut World) -> Result<&'static str, String> {
    w.section::<Fuzzing>()
        .target
        .ok_or_else(|| "no step named a fuzz target".to_owned())
}

#[given(regex = r"^the cargo-fuzz target for the (.+)$")]
fn the_target_for(w: &mut World, input: String) -> Result<(), String> {
    let target = fuzzing::target_for(&input)?;
    let code = checkout();
    let manifest = code.read(fuzzing::MANIFEST)?;
    outcome(fuzzing::target_problems(&manifest, target, |f| {
        code.read(f)
    }))?;
    w.section::<Fuzzing>().target = Some(target);
    Ok(())
}

#[then(regex = r"^its seed corpus is committed under fuzz/corpus$")]
fn its_corpus_is_committed(w: &mut World) -> Result<(), String> {
    let target = target(w)?;
    let dir = checkout().root.join(fuzzing::CORPUS).join(target);
    let seeds: Vec<String> = fs::read_dir(&dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();

    outcome(fuzzing::corpus_problems(target, &seeds))
}

#[then(regex = r"^the nightly fuzz job fuzzes it$")]
fn the_nightly_job_fuzzes_it(w: &mut World) -> Result<(), String> {
    let target = target(w)?;
    outcome(fuzzing::nightly_problems(
        &checkout().read(fuzzing::NIGHTLY)?,
        target,
    ))
}
