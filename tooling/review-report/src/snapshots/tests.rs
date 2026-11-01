use super::*;

const BASE: &str = "cbf144c6e74833535424c54cbfd04a23d90b47b0";
const HEAD: &str = "86ec49030e8e8fe2af2bc2fd9441a64167e12ab6";

fn snapshot(name: &str, base: &str, head: &str) -> String {
    format!("[[snapshot]]\nname = \"{name}\"\npull = 99\nbase = \"{base}\"\nhead = \"{head}\"\n")
}

#[test]
fn a_states_reads_each_state() {
    let text = format!(
        "{}{}",
        snapshot("pr-99", BASE, HEAD),
        snapshot("pr-94", HEAD, BASE)
    );

    let snapshots = parse(&text).unwrap();

    assert_eq!(snapshots.len(), 2);
    assert_eq!(
        snapshots[0],
        Snapshot {
            name: "pr-99".to_owned(),
            pull: 99,
            base: BASE.to_owned(),
            head: HEAD.to_owned(),
        }
    );
}

#[test]
fn a_states_out_of_shape_is_refused_naming_why() {
    let cases = [
        ("pull = 1".to_owned(), "malformed snapshots: "),
        (
            "snapshot = []".to_owned(),
            "the snapshots file holds no snapshot",
        ),
        (snapshot("PR 99", BASE, HEAD), "a name is lowercase"),
        (snapshot("", BASE, HEAD), "a name is lowercase"),
        (
            format!("{}{}", snapshot("a", BASE, HEAD), snapshot("a", BASE, HEAD)),
            "is named twice",
        ),
        (snapshot("a", "main", HEAD), "40-digit commit ids"),
        (
            snapshot("a", BASE, &HEAD.to_uppercase()),
            "40-digit commit ids",
        ),
        (snapshot("a", BASE, BASE), "its base is its head"),
    ];
    for (text, want) in cases {
        let err = parse(&text).unwrap_err();
        assert!(err.contains(want), "{text}: {err}");
    }
}

#[test]
fn the_matrices_run_each_state_once_and_each_sample_of_it() {
    let snapshots = parse(&snapshot("pr-99", BASE, HEAD)).unwrap();

    let out = matrices(&snapshots);

    let mut lines = out.lines();
    let snapshots: serde_json::Value =
        serde_json::from_str(lines.next().unwrap().strip_prefix("snapshots=").unwrap()).unwrap();
    let samples: serde_json::Value =
        serde_json::from_str(lines.next().unwrap().strip_prefix("samples=").unwrap()).unwrap();
    assert_eq!(snapshots["include"][0]["head"], HEAD);
    let include = samples["include"].as_array().unwrap();
    assert_eq!(include.len(), SAMPLES);
    assert_eq!(include[2]["sample"], 3);
    assert_eq!(include[2]["name"], "pr-99");
    assert_eq!(include[2]["base"], BASE);
}
