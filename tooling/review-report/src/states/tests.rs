use super::*;

const BASE: &str = "cbf144c6e74833535424c54cbfd04a23d90b47b0";
const HEAD: &str = "86ec49030e8e8fe2af2bc2fd9441a64167e12ab6";

fn state(name: &str, base: &str, head: &str) -> String {
    format!("[[state]]\nname = \"{name}\"\npull = 99\nbase = \"{base}\"\nhead = \"{head}\"\n")
}

#[test]
fn a_states_reads_each_state() {
    let text = format!(
        "{}{}",
        state("pr-99", BASE, HEAD),
        state("pr-94", HEAD, BASE)
    );

    let states = parse(&text).unwrap();

    assert_eq!(states.len(), 2);
    assert_eq!(
        states[0],
        State {
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
        ("pull = 1".to_owned(), "malformed states: "),
        ("state = []".to_owned(), "the states holds no state"),
        (state("PR 99", BASE, HEAD), "a name is lowercase"),
        (state("", BASE, HEAD), "a name is lowercase"),
        (
            format!("{}{}", state("a", BASE, HEAD), state("a", BASE, HEAD)),
            "is named twice",
        ),
        (state("a", "main", HEAD), "40-digit commit ids"),
        (
            state("a", BASE, &HEAD.to_uppercase()),
            "40-digit commit ids",
        ),
        (state("a", BASE, BASE), "its base is its head"),
    ];
    for (text, want) in cases {
        let err = parse(&text).unwrap_err();
        assert!(err.contains(want), "{text}: {err}");
    }
}

#[test]
fn the_matrices_run_each_state_once_and_each_sample_of_it() {
    let states = parse(&state("pr-99", BASE, HEAD)).unwrap();

    let out = matrices(&states);

    let mut lines = out.lines();
    let states: serde_json::Value =
        serde_json::from_str(lines.next().unwrap().strip_prefix("states=").unwrap()).unwrap();
    let samples: serde_json::Value =
        serde_json::from_str(lines.next().unwrap().strip_prefix("samples=").unwrap()).unwrap();
    assert_eq!(states["include"][0]["head"], HEAD);
    let include = samples["include"].as_array().unwrap();
    assert_eq!(include.len(), SAMPLES);
    assert_eq!(include[2]["sample"], 3);
    assert_eq!(include[2]["name"], "pr-99");
    assert_eq!(include[2]["base"], BASE);
}
