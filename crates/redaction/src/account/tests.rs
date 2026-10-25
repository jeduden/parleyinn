use super::{ACCOUNT_EMAIL, HarnessAccountIdentifiers, ORGANIZATION_ID, emails_in, is_email};

const CONTEXT: &str = r#"{"attachment":{"type":"session_context","context":{"userEmail":"The user's email address is user@example.com. Use it only to identify the user."}},"type":"attachment"}"#;
const ORG: &str = r#"{"attachment":{"type":"credential_org","organizationUuid":"0f0e0d0c-0000-0000-0000-000000000001"},"type":"attachment"}"#;

#[test]
fn learn_reads_the_harness_fields_only() {
    let mut ids = HarnessAccountIdentifiers::default();
    assert!(ids.is_empty());
    ids.learn("not json");
    ids.learn(r#"{"message":"write to someone@example.org"}"#);
    ids.learn(r#"{"attachment":{"type":"credential_org","organizationUuid":""}}"#);
    ids.learn(r#"{"attachment":{"type":"session_context","context":{}}}"#);
    ids.learn(r#"{"attachment":{"type":"credential_org"}}"#);
    assert!(ids.is_empty());

    ids.learn(CONTEXT);
    ids.learn(ORG);
    assert!(!ids.is_empty());
    let rules = ids.rules().unwrap();
    let names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
    assert_eq!(names, [ACCOUNT_EMAIL, ORGANIZATION_ID]);
    let text = "user@example.com 0f0e0d0c-0000-0000-0000-000000000001";
    assert_eq!(rules[0].spans(text).collect::<Vec<_>>(), [(0, 16)]);
    assert_eq!(rules[1].spans(text).collect::<Vec<_>>(), [(17, 53)]);
}

#[test]
fn emails_in_finds_addresses_with_their_punctuation_trimmed() {
    let found: Vec<String> = emails_in("mail <a@b.co>, then c@d.org. not @x.y or e@f").collect();
    assert_eq!(found, ["a@b.co", "c@d.org"]);
}

#[test]
fn is_email_wants_one_at_and_a_dotted_domain() {
    assert!(is_email("a@b.c"));
    assert!(!is_email("a@b"));
    assert!(!is_email("@b.c"));
    assert!(!is_email("a@.c"));
    assert!(!is_email("a@b."));
    assert!(!is_email("a@b@c.d"));
    assert!(!is_email("plain"));
}
