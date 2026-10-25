use super::*;

#[test]
fn parse_reads_each_benchmark_s_samples_in_order() {
    let text = "# benchmark nanoseconds\ncore_start_up 30\n\nother 5\ncore_start_up 10\n";

    assert_eq!(
        parse(text),
        Ok(Samples::from([
            ("core_start_up".to_owned(), vec![30, 10]),
            ("other".to_owned(), vec![5]),
        ]))
    );
}

#[test]
fn parse_strata_keeps_each_stratum_s_samples_apart() {
    let text = "# stratum 0\ncore_start_up 30\n# stratum 1\ncore_start_up 10\nother 5\n";

    let strata = parse_strata(text).unwrap();

    assert_eq!(
        strata,
        Strata::from([
            (
                "0".to_owned(),
                Samples::from([("core_start_up".to_owned(), vec![30])])
            ),
            (
                "1".to_owned(),
                Samples::from([
                    ("core_start_up".to_owned(), vec![10]),
                    ("other".to_owned(), vec![5]),
                ])
            ),
        ])
    );
    assert_eq!(
        pooled(&strata),
        Samples::from([
            ("core_start_up".to_owned(), vec![30, 10]),
            ("other".to_owned(), vec![5]),
        ])
    );
}

#[test]
fn a_file_without_a_stratum_line_is_one_unnamed_stratum() {
    let strata = parse_strata("# benchmark nanoseconds\ncore_start_up 30\n").unwrap();

    assert_eq!(strata.keys().collect::<Vec<_>>(), [""]);
}

#[test]
fn a_stratum_line_without_a_name_is_refused() {
    assert_eq!(
        parse_strata("ok 1\n# stratum   \n"),
        Err("line 2: a stratum line names its stratum".to_owned())
    );
}

#[test]
fn parse_of_nothing_is_no_samples() {
    assert_eq!(parse(""), Ok(Samples::new()));
}

#[test]
fn parse_names_the_line_it_cannot_read() {
    for bad in [
        "core_start_up",
        "core_start_up ten",
        "core_start_up -1",
        "core_start_up  10",
        "Core 10",
        " 10",
    ] {
        let text = format!("ok 1\n{bad}\n");
        assert_eq!(
            parse(&text),
            Err(format!(
                "line 2: {bad:?} is not a benchmark name and a sample in nanoseconds"
            )),
            "{bad:?}"
        );
    }
}

#[test]
fn render_writes_one_line_per_sample_and_parses_back() {
    let text = render("core_start_up", &[3, 1, 2]);

    assert_eq!(text, "core_start_up 3\ncore_start_up 1\ncore_start_up 2\n");
    assert_eq!(
        parse(&format!("{HEADER}\n{text}")),
        Ok(Samples::from([("core_start_up".to_owned(), vec![3, 1, 2])]))
    );
}

#[test]
fn valid_name_takes_lowercase_letters_digits_and_underscores() {
    assert!(valid_name("core_start_up_2"));
    assert!(!valid_name(""));
    assert!(!valid_name("core start"));
    assert!(!valid_name("Core"));
    assert!(!valid_name("core-start"));
}
