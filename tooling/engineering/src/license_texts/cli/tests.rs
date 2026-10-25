use super::*;

fn run_with(args: &[&str], check: Check) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(
        args.iter().map(|a| (*a).to_owned()),
        &mut out,
        &mut err,
        check,
    );

    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn a_clean_checkout_passes() {
    let (code, out, err) = run_with(&["--root", "repo"], &|root, _| {
        assert_eq!(root, Path::new("repo"));
        Ok(vec![])
    });

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.contains("has a license text"));
}

#[test]
fn the_root_defaults_to_the_current_directory() {
    let (code, _, _) = run_with(&[], &|root, _| {
        assert_eq!(root, Path::new("."));
        Ok(vec![])
    });

    assert_eq!(code, EXIT_OK);
}

#[test]
fn every_problem_is_printed_and_fails() {
    let (code, _, err) = run_with(&[], &|_, _| {
        Ok(vec![
            "a 1.0.0 ships no license file".into(),
            "b 2.0.0 ships no license file".into(),
        ])
    });

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        err,
        "license-check: a 1.0.0 ships no license file\nlicense-check: b 2.0.0 ships no license file\n"
    );
}

#[test]
fn a_check_that_cannot_run_fails() {
    let (code, _, err) = run_with(&[], &|_, _| Err("run cargo metadata: gone".into()));

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(err, "license-check: run cargo metadata: gone\n");
}

#[test]
fn a_wrong_command_line_is_a_usage_error() {
    for args in [&["--bogus"][..], &["--root"], &["--root", ""], &["--out"]] {
        let (code, _, err) = run_with(args, &|_, _| unreachable!("usage is checked first"));

        assert_eq!(code, EXIT_USAGE, "{args:?}");
        assert!(
            err.contains("usage: license-check [--root DIR] [--write] [--out DIR]"),
            "{err}"
        );
    }
}

#[test]
fn write_and_out_reach_the_check_as_options() {
    let (code, _, _) = run_with(&["--write", "--out", "regen"], &|_, options| {
        assert_eq!(
            options,
            &Options {
                write: true,
                out: Some(PathBuf::from("regen"))
            }
        );
        Ok(vec![])
    });

    assert_eq!(code, EXIT_OK);
}
