use super::*;
use serde_json::Value;

/// The committed sample: 646 texts with Claude's count of each.
const SAMPLE: &str = include_str!("../../../../testdata/model-tokens/sample.jsonl");

#[test]
fn class_sorts_ascii() {
    assert_eq!(class('q'), Class::Letter);
    assert_eq!(class('Z'), Class::Letter);
    assert_eq!(class('7'), Class::Digit);
    assert_eq!(class(' '), Class::Space);
    assert_eq!(class('\n'), Class::Break);
    assert_eq!(class('\x7f'), Class::Break);
    assert_eq!(class('{'), Class::Punct);
    assert_eq!(class('~'), Class::Punct);
}

#[test]
fn class_sorts_by_script() {
    assert_eq!(class('é'), Class::Latin);
    assert_eq!(class('Ж'), Class::Two);
    assert_eq!(class('ह'), Class::Three);
    assert_eq!(class('—'), Class::Symbol);
    assert_eq!(class('東'), Class::Cjk);
    assert_eq!(class('한'), Class::Cjk);
    assert_eq!(class('豈'), Class::Cjk);
    assert_eq!(class('Ａ'), Class::Cjk);
    assert_eq!(class('🚀'), Class::Four);
}

#[test]
fn counts_counts_classes_and_switches() {
    let (n, switches) = counts("ab 1\n{é東🚀");
    assert_eq!(n, [2, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1]);
    assert_eq!(switches, 0);
    assert_eq!(counts("aB3c").1, 3);
    assert_eq!(counts("ab cd").1, 0);
    assert_eq!(counts(""), ([0; CLASSES], 0));
}

#[test]
fn from_counts_rounds_up_and_adds_the_base() {
    assert_eq!(from_counts(&[0; CLASSES], 0), BASE);
    let mut n = [0; CLASSES];
    n[0] = 1;
    assert_eq!(from_counts(&n, 0), 1 + BASE);
    assert_eq!(from_counts(&n, 1), 2 + BASE);
}

#[test]
fn an_estimate_is_deterministic_and_never_falls_as_text_is_added() {
    let a = estimate("Never push to main.");
    assert_eq!(a, estimate("Never push to main."));
    assert!(estimate("Never push to main. Never.") >= a);
    assert!(estimate("x") > estimate(""));
}

/// §8.4's bound: the estimate is at least Claude's count for at least
/// 99% of a representative sample.
#[test]
fn the_estimate_holds_the_99_percent_bound_on_the_committed_sample() {
    let (mut held, mut total) = (0u64, 0u64);
    for line in SAMPLE.lines() {
        let sample: Value = serde_json::from_str(line).unwrap();
        let text = sample["text"].as_str().unwrap();
        let counted = sample["tokens"]["claude-haiku-5-5"].as_u64().unwrap();
        total += 1;
        held += u64::from(estimate(text) >= counted);
    }

    assert_eq!(total, 646);
    assert!(held * 100 >= total * 99, "{held} of {total} samples held");
}
