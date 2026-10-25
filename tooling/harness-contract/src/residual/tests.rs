use std::fs;

use serde_json::json;
use testkit::TempDir;

use super::*;

#[test]
fn placeholders_pass() {
    let body = concat!(
        "{\"email\":\"user@example.com\",\"commit\":\"noreply@anthropic.com\"}\n",
        "{\"organizationUuid\":\"00000000-0000-0000-0000-000000000000\"}\n",
        "cd /home/u/w && ls /home/.claude /home# /Users/\n",
        "mcp__claude_ai_Connector_D__search, {\"addedNames\":[\"claude.ai Connector A\"]}\n",
        "## claude.ai Connector B\\nclaude.ai Connector C\n",
        "a claude.ai connector setting, x@y, a@b.c, [0-9]*@gmail\\.com\n",
    );
    assert_eq!(residual(body), Vec::<String>::new());
}

#[test]
fn an_email_address_other_than_the_placeholders_is_residual() {
    assert_eq!(
        residual("mail jane.doe+parinn@gmail.com. or ops@corp.example.org-"),
        [
            "an email address jane.doe+parinn@gmail.com",
            "an email address ops@corp.example.org"
        ]
    );
}

#[test]
fn an_organization_id_other_than_the_nil_one_is_residual() {
    let id = "4f7d2c1e-9a3b-4c8d-8e2f-1a2b3c4d5e6f";
    assert_eq!(
        residual(&format!(
            "{{\"organizationUuid\":\"{id}\"}} \\\"organizationUuid\\\": \\\"{id}\\\""
        )),
        [
            format!("an organization id {id}"),
            format!("an organization id {id}")
        ]
    );
    assert!(residual("\"organizationUuid\":\"[0-9a-f-]*\"").is_empty());
}

#[test]
fn a_home_path_other_than_the_placeholders_is_residual() {
    assert_eq!(
        residual("/home/jeduden/src and /Users/Jane.Doe/x"),
        ["a home path /home/jeduden", "a home path /Users/Jane.Doe"]
    );
}

#[test]
fn a_connector_named_other_than_its_placeholder_is_residual() {
    let body = concat!(
        "mcp__claude_ai_Gmail__send ",
        "{\"addedNames\":[\"claude.ai Google Drive\"]} ",
        "## claude.ai Acme CRM\n",
        "\\nclaude.ai Connector AB",
    );
    assert_eq!(
        residual(body),
        [
            "a claude.ai connector claude_ai_Gmail",
            "a claude.ai connector \"Google Drive\"",
            "a claude.ai connector \"Acme CRM\"",
            "a claude.ai connector \"Connector AB\"",
        ]
    );
}

#[test]
fn a_skill_no_listed_one_names_is_residual() {
    let listing = |names: &[&str], content: &str| {
        json!({"type": "attachment", "attachment": {"type": "skill_listing", "names": names, "content": content}})
            .to_string()
    };
    let body = [
        listing(
            &["dataviz", "my-secret-skill"],
            "- dataviz: charts\n- billing-tool: pay\nnot an entry",
        ),
        listing(&["greet"], "- greet: hi"),
        json!({"type": "attachment", "attachment": {"type": "date", "names": ["x"]}}).to_string(),
        "not json".to_owned(),
    ]
    .join("\n");

    assert_eq!(
        residual(&body),
        ["a skill billing-tool", "a skill my-secret-skill"]
    );
}

#[test]
fn checking_a_tree_names_each_file_with_a_residual_identifier() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("v/transcripts")).unwrap();
    fs::write(dir.path().join("v/clean.json"), "user@example.com").unwrap();
    fs::write(dir.path().join("v/transcripts/a.jsonl"), b"/home/alice\xff").unwrap();

    let problems = check(dir.path()).unwrap();
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].ends_with("a.jsonl: a home path /home/alice"),
        "{problems:?}"
    );
    assert!(check(&dir.path().join("missing")).is_err());
}
