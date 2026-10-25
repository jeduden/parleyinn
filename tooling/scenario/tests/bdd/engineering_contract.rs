//! The steps of ENG-17 that phases 7 and 9 of plan 2609292012 carry:
//! the contract suite replays every recorded transcript of every
//! supported Claude Code version through the core's own parser, and
//! every recorded hook input through the core's hook input decoder, and
//! the residual-identifier check refuses a fixture that carries an
//! account's identifiers, with its drift case. `@ENG-17` stays pending:
//! its nightly live run and recording are M2's.

use cucumber::{given, then, when};
use harness_contract::{recording, replay, residual, shapes};
use redaction::Redactor;
use transcript::shape::Shapes;

use crate::World;

/// What the contract steps track: each recorded version, and the
/// problems the replay found.
#[derive(Debug, Default)]
pub struct ContractSteps {
    versions: Vec<std::path::PathBuf>,
    problems: Option<Vec<String>>,
}

fn steps(w: &mut World) -> &mut ContractSteps {
    w.section::<ContractSteps>()
}

#[given(regex = r"^recorded hook inputs and transcripts for each supported Claude Code version$")]
fn recordings(w: &mut World) -> Result<(), String> {
    let versions = recording::versions(&testkit::repo_root().join("testdata/claude-code"))?;
    for version in &versions {
        let name = version
            .file_name()
            .map(|n| n.to_string_lossy().into_owned());
        let supported = name
            .as_deref()
            .is_some_and(|n| Shapes::supported().version(n).is_some());
        if !supported || shapes::transcripts(version)?.is_empty() || !version.join("hooks").is_dir()
        {
            return Err(format!(
                "{} holds no supported version's transcripts and hook inputs",
                version.display()
            ));
        }
    }
    steps(w).versions = versions;
    Ok(())
}

#[when(regex = r"^the contract suite replays them$")]
fn replays(w: &mut World) -> Result<(), String> {
    let supported = Shapes::supported();
    let redactor = Redactor::new(&[]).map_err(|e| e.to_string())?.0;
    let mut problems = Vec::new();
    for version in steps(w).versions.clone() {
        for path in shapes::transcripts(&version)? {
            problems.extend(replay::replay_file(&path, &supported, &redactor)?);
        }
        for path in recording::files(&version.join("hooks"), ".json")? {
            problems.extend(replay::replay_hook_input(&path)?);
        }
    }
    steps(w).problems = Some(problems);
    Ok(())
}

#[then(regex = r"^every one parses without an unparsed event or format drift$")]
fn every_one_parses(w: &mut World) -> Result<(), String> {
    match steps(w).problems.take() {
        Some(problems) if problems.is_empty() => Ok(()),
        Some(problems) => Err(problems.join("\n")),
        None => Err("the contract suite did not replay".to_owned()),
    }
}

#[then(
    regex = r"^it fails on a recorded fixture that carries an account's email address, organization id, a home path or the name of one of its skills or connectors, with its drift case$"
)]
fn residual_identifiers_refused(_w: &mut World) -> Result<(), String> {
    let skill = serde_json::json!({"type": "attachment", "attachment": {"type": "skill_listing", "names": ["acme-billing"]}});
    let samples = [
        ("an email address", "jane.doe@gmail.com".to_owned()),
        (
            "an organization id",
            "\"organizationUuid\":\"4f7d2c1e-9a3b-4c8d-8e2f-1a2b3c4d5e6f\"".to_owned(),
        ),
        ("a home path", "/home/jane/src".to_owned()),
        ("a skill", skill.to_string()),
        (
            "a claude.ai connector",
            "mcp__claude_ai_Gmail__send".to_owned(),
        ),
    ];
    for (kind, body) in &samples {
        if !residual::residual(body).iter().any(|p| p.starts_with(kind)) {
            return Err(format!(
                "the check passes a fixture carrying {kind}: {body}"
            ));
        }
    }
    let guarded = drift::cases().iter().any(|c| {
        c.guards == "ENG-17"
            && c.check
                .args
                .iter()
                .any(|a| a == "recorded_fixtures_hold_no_residual_identifier")
    });
    if guarded {
        return Ok(());
    }
    Err("no drift case puts a residual identifier in a fixture".to_owned())
}
