use configuration::source::OsUser;
use serde_json::Value;

use crate::tests::{DEV, Process, audited, recorded};
use crate::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};

fn json(out: &str) -> Value {
    serde_json::from_str(out).unwrap()
}

/// A process whose node has ingested one run, which leaves every
/// residual risk open.
fn with_a_run() -> Process {
    let p = Process::new();
    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(code, EXIT_OK, "{err}");
    p
}

/// Ingests another run, which audits the keys the settings layers
/// ignore.
fn ingest_again(p: &Process) {
    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("b\n")], &DEV);
    assert_eq!(code, EXIT_OK, "{err}");
}

/// The counters each refusal audited, in order.
fn refusals(p: &Process) -> Vec<(String, String)> {
    audited(p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("principal_act_refused"))
        .map(|e| (e.operation, e.detail))
        .collect()
}

#[test]
fn each_verb_wants_its_subcommand_and_takes_only_json() {
    for (args, why) in [
        (
            &["configuration"][..],
            "parinn configuration: want accept\n",
        ),
        (
            &["configuration", "show"],
            "parinn configuration: want accept\n",
        ),
        (
            &["configuration", "accept", "--yes"],
            "parinn configuration accept: unknown flag \"--yes\"\n",
        ),
        (
            &["risk-acceptance"],
            "parinn risk-acceptance: want show, record or withdraw\n",
        ),
        (
            &["risk-acceptance", "grant"],
            "parinn risk-acceptance: want show, record or withdraw\n",
        ),
        (
            &["risk-acceptance", "record", "--all"],
            "parinn risk-acceptance record: unknown flag \"--all\"\n",
        ),
    ] {
        assert_eq!(
            Process::new().invoke(args, &DEV),
            (EXIT_USAGE, String::new(), why.into()),
            "{args:?}"
        );
    }
}

#[test]
fn every_act_refuses_without_a_terminal_before_any_other_check_and_audits_it() {
    for args in [
        &["configuration", "accept"][..],
        &["risk-acceptance", "record"],
        &["risk-acceptance", "withdraw"],
    ] {
        let mut p = Process::new();
        p.terminal = false;
        // A configuration that fails validation is not reached.
        p.configure("recal.max_k = 1\n");
        let (code, out, err) = p.invoke(args, &DEV);
        assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""), "{args:?}");
        assert!(
            err.contains("refused: a principal act needs a terminal"),
            "{err}"
        );
        assert_eq!(refusals(&p).len(), 1, "{args:?}");
        assert!(!p.parinn_home().join("store.db").exists());
    }
}

#[test]
fn configuration_accept_needs_a_configuration() {
    let p = Process::new();
    let (code, _, err) = p.invoke(&["configuration", "accept"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("parinn configuration accept: no configuration at "),
        "{err}"
    );
    assert!(err.ends_with("config.toml to accept\n"), "{err}");
}

#[test]
fn configuration_accept_exits_2_on_an_invalid_layer_naming_key_and_problem() {
    let p = Process::new();
    p.configure("recall.max_k = 500\n");
    let (code, out, err) = p.invoke(&["configuration", "accept"], &DEV);
    assert_eq!((code, out.as_str()), (EXIT_USAGE, ""));
    assert!(
        err.ends_with("config.toml: key \"recall.max_k\": out of range: want 1 to 200\n"),
        "{err}"
    );
}

#[test]
fn configuration_accept_records_the_digest_and_settings_and_they_apply() {
    let p = Process::new();
    p.configure("node.deployment_mode = 'interactive'\nrecall.max_k = 10\n");
    let (_, before, _) = p.invoke(&["status"], &DEV);
    assert!(before.contains("deployment mode: automation\n"), "{before}");
    assert!(
        before.contains("awaiting acceptance (parinn configuration accept): node.deployment_mode"),
        "{before}"
    );

    let (code, out, err) = p.invoke(&["configuration", "accept"], &DEV);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(
        out.starts_with("accepted the configuration with digest "),
        "{out}"
    );
    assert!(
        out.ends_with("; now applying: node.deployment_mode\n"),
        "{out}"
    );
    let acts = recorded(&p);
    let act = acts.last().unwrap();
    assert_eq!(act["act"], "configuration_accept");
    assert_eq!(act["class"], "widening");
    assert_eq!(
        act["settings"],
        serde_json::json!({"node.deployment_mode": "interactive", "recall.max_k": 10})
    );
    let (_, after, _) = p.invoke(&["status", "--json"], &DEV);
    let after = json(&after);
    assert_eq!(after["deployment_mode"], "interactive");
    assert_eq!(after["awaiting_acceptance"], serde_json::json!([]));

    let (code, out, _) = p.invoke(&["configuration", "accept", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    assert_eq!(json(&out)["applying"], serde_json::json!([]));
    let (_, text, _) = p.invoke(&["configuration", "accept"], &DEV);
    assert!(!text.contains("now applying"), "{text}");
}

#[test]
fn configuration_accept_leaves_what_managed_policy_sets_to_managed_policy() {
    let p = Process::new();
    p.configure("node.deployment_mode = 'interactive'\nrecall.max_k = 10\n");
    p.manage("node.deployment_mode = 'automation'\n");

    let (code, _, err) = p.invoke(&["configuration", "accept"], &DEV);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(refusals(&p).is_empty());
    let (_, after, _) = p.invoke(&["status", "--json"], &DEV);
    assert_eq!(json(&after)["deployment_mode"], "automation");
}

#[test]
fn a_widening_act_is_refused_while_managed_policy_requires_an_authenticator() {
    let p = Process::new();
    p.configure("recall.max_k = 10\n");
    p.manage("authenticator.required = true\n");
    for args in [
        &["configuration", "accept"][..],
        &["risk-acceptance", "record"],
    ] {
        let (code, _, err) = p.invoke(args, &DEV);
        if args[0] == "risk-acceptance" {
            // No run leaves a risk open, so nothing is to be accepted.
            assert_eq!(code, EXIT_OK, "{err}");
            continue;
        }
        assert_eq!(code, EXIT_FAILURE, "{args:?}");
        assert!(err.contains("an authenticator is required"), "{err}");
    }
    let q = with_a_run();
    q.manage("authenticator.required = true\n");
    let (code, _, err) = q.invoke(&["risk-acceptance", "record"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("an authenticator is required"), "{err}");
    assert_eq!(
        refusals(&p),
        [(
            "configuration accept".to_owned(),
            "refused: an authenticator is required".to_owned()
        )]
    );
}

#[test]
fn a_widening_act_is_refused_while_the_principal_s_configuration_requires_an_authenticator() {
    let p = with_a_run();
    p.configure("authenticator.required = true\n");

    let (code, _, err) = p.invoke(&["risk-acceptance", "record"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("an authenticator is required"), "{err}");
    assert_eq!(
        refusals(&p),
        [(
            "risk-acceptance record".to_owned(),
            "refused: an authenticator is required".to_owned()
        )]
    );
}

#[test]
fn a_repository_can_neither_require_an_authenticator_nor_forbid_risk_acceptance() {
    let p = with_a_run();
    p.repository("authenticator.required = true\nrisk_acceptance.forbidden = true\n");
    let (code, _, err) = p.invoke(&["risk-acceptance", "record"], &DEV);
    assert_eq!(code, EXIT_OK, "{err}");
    p.configure("recall.max_k = 10\n");
    let (code, _, err) = p.invoke(&["configuration", "accept"], &DEV);
    assert_eq!(code, EXIT_OK, "{err}");
    assert!(refusals(&p).is_empty());
    ingest_again(&p);
    let ignored: Vec<String> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("repository_configuration_key_ignored"))
        .map(|e| e.detail)
        .collect();
    assert!(
        ignored.contains(&"rejected repository setting authenticator.required".to_owned()),
        "{ignored:?}"
    );
}

#[test]
fn the_principal_s_configuration_cannot_forbid_risk_acceptance() {
    let p = with_a_run();
    p.configure("risk_acceptance.forbidden = true\n");

    let (code, _, err) = p.invoke(&["risk-acceptance", "record"], &DEV);

    assert_eq!(code, EXIT_OK, "{err}");
    ingest_again(&p);
    let ignored: Vec<String> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("configuration_key_ignored"))
        .map(|e| e.detail)
        .collect();
    assert!(
        ignored.contains(
            &"ignored configuration key only managed policy sets risk_acceptance.forbidden"
                .to_owned()
        ),
        "{ignored:?}"
    );
}

#[test]
fn a_widening_act_is_refused_while_managed_policy_is_invalid() {
    let mut p = Process::new();
    p.configure("recall.max_k = 10\n");
    p.manage("kernel.memory_mib = 64\n");
    // As another OS user's file, it passes; the running OS user owns
    // it, so it could have written it.
    p.os_user = OsUser::Known(u32::MAX);
    assert_eq!(p.invoke(&["configuration", "accept"], &DEV).0, EXIT_OK);
    let mine = std::os::unix::fs::MetadataExt::uid(&std::fs::metadata(p.dir.path()).unwrap());
    p.os_user = OsUser::Known(mine);
    let (code, _, err) = p.invoke(&["configuration", "accept"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("refused: managed policy is invalid"), "{err}");
    let (_, status, _) = p.invoke(&["status"], &DEV);
    assert!(
        status.contains("managed policy refused: managed policy "),
        "{status}"
    );
}

#[test]
fn managed_policy_is_refused_where_the_os_has_no_modes_to_check() {
    let mut p = Process::new();
    p.configure("recall.max_k = 10\n");
    p.manage("kernel.memory_mib = 64\n");
    p.os_user = OsUser::Unchecked;

    let (code, _, err) = p.invoke(&["configuration", "accept"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("refused: managed policy is invalid"), "{err}");
    let (_, status, _) = p.invoke(&["status"], &DEV);
    assert!(
        status.contains("cannot be checked on this OS yet"),
        "{status}"
    );
}

#[test]
fn managed_policy_is_refused_while_the_running_os_user_is_unknown() {
    let mut p = Process::new();
    p.configure("recall.max_k = 10\n");
    p.manage("kernel.memory_mib = 64\n");
    p.os_user = OsUser::Unknown;

    let (code, _, err) = p.invoke(&["configuration", "accept"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("refused: managed policy is invalid"), "{err}");
    let (_, status, _) = p.invoke(&["status"], &DEV);
    assert!(
        status.contains("the running OS user cannot be learnt"),
        "{status}"
    );
}

#[test]
fn a_widening_act_is_refused_while_a_run_leaves_risks_open_until_they_are_accepted() {
    let p = with_a_run();
    p.configure("recall.max_k = 10\n");

    let (code, _, err) = p.invoke(&["configuration", "accept"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("these residual risks are open"), "{err}");
    for id in ["R1", "R2", "R3", "R4", "R5", "R6", "R7", "R8"] {
        assert!(err.contains(&format!("  {id}: the agent ")), "{id}: {err}");
    }
    assert!(
        err.contains("runs leaving them open, as no run records a sandbox state: "),
        "{err}"
    );
    assert_eq!(refusals(&p).len(), 1);

    let (code, out, err) = p.invoke(&["risk-acceptance", "record"], &DEV);
    assert_eq!(code, EXIT_OK, "{err}");
    assert!(
        out.starts_with("accepted these residual risks:\n  R1: "),
        "{out}"
    );
    assert!(out.ends_with("and that an unsandboxed agent can forge this risk acceptance itself\n"));
    assert_eq!(p.invoke(&["configuration", "accept"], &DEV).0, EXIT_OK);

    let (code, out, _) = p.invoke(&["risk-acceptance", "withdraw"], &DEV);
    assert_eq!(
        (code, out.as_str()),
        (EXIT_OK, "withdrew the risk acceptance\n")
    );
    assert_eq!(p.invoke(&["configuration", "accept"], &DEV).0, EXIT_FAILURE);
    let classes: Vec<String> = recorded(&p)
        .iter()
        .map(|a| {
            format!(
                "{} {}",
                a["act"].as_str().unwrap(),
                a["class"].as_str().unwrap()
            )
        })
        .collect();
    assert_eq!(
        classes,
        [
            "risk_acceptance_record widening",
            "configuration_accept widening",
            "risk_acceptance_withdraw cut",
        ]
    );
}

#[test]
fn risk_acceptance_record_is_refused_where_managed_policy_forbids_it() {
    let p = with_a_run();
    p.manage("risk_acceptance.forbidden = true\n");
    let (code, _, err) = p.invoke(&["risk-acceptance", "record"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("managed policy forbids risk acceptance"),
        "{err}"
    );
    assert_eq!(
        refusals(&p),
        [(
            "risk-acceptance record".to_owned(),
            "refused: managed policy forbids risk acceptance".to_owned()
        )]
    );
}

#[test]
fn risk_acceptance_record_with_no_open_risk_records_nothing() {
    let p = Process::new();
    assert_eq!(
        p.invoke(&["risk-acceptance", "record"], &DEV),
        (
            EXIT_OK,
            "no residual risk is open on this node: nothing to accept\n".into(),
            String::new()
        )
    );
    let (_, out, _) = p.invoke(&["risk-acceptance", "record", "--json"], &DEV);
    assert_eq!(json(&out), serde_json::json!({"accepted": [], "act": null}));
    assert!(recorded(&p).is_empty());
}

#[test]
fn risk_acceptance_withdraw_with_none_standing_records_nothing() {
    let p = Process::new();
    assert_eq!(
        p.invoke(&["risk-acceptance", "withdraw"], &DEV).1,
        "no risk acceptance stands: nothing to withdraw\n"
    );
    let (_, out, _) = p.invoke(&["risk-acceptance", "withdraw", "--json"], &DEV);
    assert_eq!(
        json(&out),
        serde_json::json!({"withdrawn": false, "act": null})
    );
}

#[test]
fn risk_acceptance_show_names_the_open_risks_the_runs_and_the_acceptance() {
    let fresh = Process::new();
    assert_eq!(
        fresh.invoke(&["risk-acceptance", "show"], &DEV),
        (
            EXIT_OK,
            "no residual risk is open on this node\nno risk acceptance stands\n".into(),
            String::new()
        )
    );
    let p = with_a_run();
    let (_, out, _) = p.invoke(&["risk-acceptance", "show"], &DEV);
    assert!(
        out.starts_with("residual risks open on this node:\n  R1: "),
        "{out}"
    );
    assert!(out.ends_with("no risk acceptance stands\n"), "{out}");
    p.invoke(&["risk-acceptance", "record"], &DEV);
    let (_, out, _) = p.invoke(&["risk-acceptance", "show", "--json"], &DEV);
    let v = json(&out);
    assert_eq!(v["open"].as_array().unwrap().len(), 8);
    assert_eq!(v["runs"].as_array().unwrap().len(), 1);
    assert_eq!(v["accepted"].as_array().unwrap().len(), 8);
    let (_, out, _) = p.invoke(&["risk-acceptance", "show"], &DEV);
    assert!(
        out.ends_with("the standing risk acceptance names R1, R2, R3, R4, R5, R6, R7, R8\n"),
        "{out}"
    );
    let (_, out, _) = p.invoke(&["risk-acceptance", "record", "--json"], &DEV);
    assert_eq!(json(&out)["accepted"].as_array().unwrap().len(), 8);
    let (_, out, _) = p.invoke(&["risk-acceptance", "withdraw", "--json"], &DEV);
    assert_eq!(json(&out)["withdrawn"], true);
}

#[test]
fn risk_acceptance_show_runs_without_a_terminal() {
    let mut p = with_a_run();
    p.terminal = false;
    assert_eq!(p.invoke(&["risk-acceptance", "show"], &DEV).0, EXIT_OK);
}

#[test]
fn a_verb_reports_a_home_it_cannot_open() {
    let p = Process::new();
    std::fs::create_dir_all(p.parinn_home()).unwrap();
    crate::tests::set_mode(&p.parinn_home(), 0o750);
    for args in [
        &["configuration", "accept"][..],
        &["risk-acceptance", "record"],
        &["risk-acceptance", "withdraw"],
        &["risk-acceptance", "show"],
        &["counter", "ack"],
    ] {
        let (code, _, err) = p.invoke(args, &DEV);
        assert_eq!(code, EXIT_FAILURE, "{args:?}");
        assert!(err.contains("mode 0750 lets other OS users in"), "{err}");
    }
}

/// Makes the store refuse every principal act whose content holds
/// `holding`, as a store that fails mid-act would.
fn refuse_acts(p: &Process, holding: &str) {
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute_batch(&format!(
        "CREATE TRIGGER refuse BEFORE INSERT ON events \
         WHEN NEW.kind = 'principal_act' AND NEW.content LIKE '%{holding}%' \
         BEGIN SELECT RAISE(ABORT, 'refused by the test'); END;"
    ))
    .unwrap();
}

#[test]
fn a_store_that_takes_no_act_fails_the_verb() {
    let p = with_a_run();
    refuse_acts(&p, "counter_ack");
    let (code, _, err) = p.invoke(&["counter", "ack"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("refused by the test"), "{err}");

    let q = with_a_run();
    assert_eq!(q.invoke(&["risk-acceptance", "record"], &DEV).0, EXIT_OK);
    refuse_acts(&q, "withdraw");
    let (code, _, err) = q.invoke(&["risk-acceptance", "withdraw"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("refused by the test"), "{err}");
}

#[test]
fn a_run_seat_whose_certificate_does_not_read_fails_risk_acceptance() {
    let p = with_a_run();
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute(
        "UPDATE seats SET certificate = '{}' WHERE certificate NOT LIKE '%\"device\"%'",
        [],
    )
    .unwrap();
    drop(conn);
    for args in [
        &["risk-acceptance", "record"][..],
        &["risk-acceptance", "show"],
    ] {
        let (code, _, err) = p.invoke(args, &DEV);
        assert_eq!(code, EXIT_FAILURE, "{args:?}");
        assert!(
            err.starts_with(&format!("parinn {} {}: ", args[0], args[1])),
            "{err}"
        );
    }
    p.configure("recall.max_k = 10\n");
    assert_eq!(p.invoke(&["configuration", "accept"], &DEV).0, EXIT_FAILURE);
}

#[test]
fn cut_and_neutral_acts_need_no_open_residual_risks() {
    let p = with_a_run();
    assert_eq!(p.invoke(&["risk-acceptance", "record"], &DEV).0, EXIT_OK);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute(
        "UPDATE seats SET certificate = '{}' WHERE certificate NOT LIKE '%\"device\"%'",
        [],
    )
    .unwrap();
    drop(conn);
    for args in [&["counter", "ack"][..], &["risk-acceptance", "withdraw"]] {
        let (code, _, err) = p.invoke(args, &DEV);
        assert_eq!(code, EXIT_OK, "{args:?}: {err}");
    }
}

#[test]
fn a_refusal_the_audit_log_cannot_take_is_still_reported() {
    let mut p = Process::new();
    p.terminal = false;
    let blocked = p.parinn_home().join("audit/audit-000001.jsonl");
    std::fs::create_dir_all(&blocked).unwrap();
    for dir in [p.parinn_home(), p.parinn_home().join("audit"), blocked] {
        crate::tests::set_mode(&dir, 0o700);
    }
    let (code, _, err) = p.invoke(&["counter", "ack"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("parinn counter ack: audit the refusal: "),
        "{err}"
    );
    assert!(err.contains("needs a terminal"), "{err}");
}
