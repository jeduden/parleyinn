use std::error::Error as _;

use audit::entry::{Counter, Kind, Occurrence};

use super::*;

#[test]
fn the_installed_hooks_parse_and_name_their_operation() {
    for (name, hook, operation) in [
        ("SessionStart", Hook::SessionStart, "hook SessionStart"),
        (
            "UserPromptSubmit",
            Hook::UserPromptSubmit,
            "hook UserPromptSubmit",
        ),
        (
            "PermissionRequest",
            Hook::PermissionRequest,
            "hook PermissionRequest",
        ),
    ] {
        assert_eq!(Hook::parse(name), Some(hook));
        assert_eq!(hook.operation(), operation);
    }
    for name in ["Stop", "sessionstart", ""] {
        assert_eq!(Hook::parse(name), None, "{name}");
    }
}

#[test]
fn a_panic_in_the_handler_is_caught_as_its_own_failure() {
    let caught = guarded(|| -> Result<String, Failure> { panic!("boom") });

    assert_eq!(caught, Err(Failure::Panicked));
    assert_eq!(guarded(|| Ok("out".to_owned())), Ok("out".to_owned()));
    assert_eq!(
        guarded(|| Err::<String, _>(Failure::Malformed)),
        Err(Failure::Malformed)
    );
}

#[test]
fn each_failure_is_audited_with_its_own_kind_and_counter() {
    let cases = [
        (
            Failure::UnknownHook,
            Kind::Rejected,
            Counter::HookRejected,
            "no such hook",
        ),
        (
            Failure::Malformed,
            Kind::Rejected,
            Counter::HookRejected,
            "not a JSON object",
        ),
        (
            Failure::Invalid,
            Kind::Rejected,
            Counter::HookInputRejected,
            "hook input failed schema validation",
        ),
        (
            Failure::Panicked,
            Kind::Failed,
            Counter::HookPanicked,
            "panic recovered",
        ),
        (
            Failure::TooLarge,
            Kind::Rejected,
            Counter::HookInputTooLarge,
            "hook input exceeds 1 MiB",
        ),
        (
            Failure::TooDeep,
            Kind::Rejected,
            Counter::HookInputTooDeep,
            "hook input nesting exceeds depth 64",
        ),
    ];
    for (failure, kind, counter, detail) in cases {
        assert_eq!(
            failure.occurrence("hook SessionStart"),
            Occurrence::Degraded {
                kind,
                counter,
                operation: "hook SessionStart",
                detail,
            }
        );
        assert_eq!(failure.to_string(), detail);
        assert!(failure.source().is_none());
    }
}

#[test]
fn each_hook_names_itself_with_a_budget_below_its_harness_timeout() {
    for (hook, name, budget) in [
        (Hook::SessionStart, "SessionStart", 150),
        (Hook::UserPromptSubmit, "UserPromptSubmit", 50),
        (Hook::PermissionRequest, "PermissionRequest", 50),
    ] {
        assert_eq!(hook.name(), name);
        assert_eq!(Hook::parse(name), Some(hook));
        assert_eq!(hook.budget(), std::time::Duration::from_millis(budget));
        assert!(hook.budget() < hook.harness_timeout());
    }
}
