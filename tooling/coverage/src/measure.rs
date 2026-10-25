//! One measurement: per test layer, run its tests on the instrumented
//! build, export what they ran, and clear the profiles for the next
//! layer.

use std::path::Path;

use crate::Host;
use crate::lcov::{self, Lines};
use crate::plan::{Plan, TestLayer};

/// The cargo arguments that run `layer`'s tests without a report, or
/// `None` when the layer has no tests.
#[must_use]
pub fn test_args(plan: &Plan, layer: TestLayer) -> Option<Vec<String>> {
    let mut args: Vec<String> = ["llvm-cov", "--no-report", "--workspace", "--locked"]
        .map(str::to_owned)
        .into();
    match layer {
        TestLayer::Unit => args.extend(["--lib", "--bins"].map(str::to_owned)),
        layer => {
            let targets = plan.targets.get(&layer).filter(|t| !t.is_empty())?;
            args.extend(
                targets
                    .iter()
                    .flat_map(|t| ["--test".to_owned(), t.clone()]),
            );
        }
    }

    Some(args)
}

/// The cargo arguments that export the profiles gathered so far as
/// lcov, leaving out the files the plan excludes.
#[must_use]
pub fn report_args(plan: &Plan) -> Vec<String> {
    let mut args: Vec<String> = ["llvm-cov", "report", "--lcov"].map(str::to_owned).into();
    if !plan.exclude.is_empty() {
        args.extend(["--ignore-filename-regex".to_owned(), plan.exclude.clone()]);
    }

    args
}

/// Measures every test layer in turn, writing each one's lcov to
/// `out/<layer>.lcov` and all of them together to `out/all.lcov`.
///
/// # Errors
///
/// Fails when a cargo command fails, a test fails among them, or a
/// report cannot be read or written.
pub fn measure(
    host: &dyn Host,
    plan: &Plan,
    out: &Path,
) -> Result<Vec<(TestLayer, Lines)>, String> {
    host.cargo(&words(&["llvm-cov", "clean", "--workspace"]))?;
    let mut layers = Vec::new();
    for layer in TestLayer::ALL {
        let Some(lines) = run_layer(host, plan, layer, out)? else {
            continue;
        };
        layers.push((layer, lines));
        host.cargo(&words(&["llvm-cov", "clean", "--profraw-only"]))?;
    }
    write_all(host, &layers, out)?;

    Ok(layers)
}

/// Measures one test layer on a fresh build, writing its lcov to
/// `out/<layer>.lcov`, or `None` when the layer has no tests. CI runs
/// each layer this way in a job of its own, then merges them with
/// [`merge_layers`].
///
/// # Errors
///
/// Fails as [`measure`] does.
pub fn measure_layer(
    host: &dyn Host,
    plan: &Plan,
    layer: TestLayer,
    out: &Path,
) -> Result<Option<Lines>, String> {
    host.cargo(&words(&["llvm-cov", "clean", "--workspace"]))?;
    run_layer(host, plan, layer, out)
}

/// Reads the layers [`measure_layer`] wrote under `out`, leaving out a
/// layer with no file, and writes all of them together to
/// `out/all.lcov`.
///
/// # Errors
///
/// Fails when a file cannot be read or is not lcov, or when no layer
/// was written.
pub fn merge_layers(host: &dyn Host, out: &Path) -> Result<Vec<(TestLayer, Lines)>, String> {
    let mut layers = Vec::new();
    for layer in TestLayer::ALL {
        if let Some(text) = host.read(&out.join(format!("{layer}.lcov")))? {
            layers.push((layer, lcov::parse(&text)?));
        }
    }
    if layers.is_empty() {
        return Err(format!("no <layer>.lcov under {}", out.display()));
    }
    write_all(host, &layers, out)?;

    Ok(layers)
}

/// Runs `layer`'s tests, exports and writes their lcov, and returns
/// what it parsed; `None` when the layer has no tests.
fn run_layer(
    host: &dyn Host,
    plan: &Plan,
    layer: TestLayer,
    out: &Path,
) -> Result<Option<Lines>, String> {
    let Some(args) = test_args(plan, layer) else {
        return Ok(None);
    };
    host.cargo(&args)?;
    let text = host.cargo_output(&report_args(plan))?;
    host.write(&out.join(format!("{layer}.lcov")), &text)?;

    Ok(Some(lcov::parse(&text)?))
}

/// Writes every layer together to `out/all.lcov`.
fn write_all(host: &dyn Host, layers: &[(TestLayer, Lines)], out: &Path) -> Result<(), String> {
    let all = lcov::merge(layers.iter().map(|(_, lines)| lines));
    host.write(&out.join("all.lcov"), &render(&all))
}

/// `w` as owned words.
fn words(w: &[&str]) -> Vec<String> {
    w.iter().map(|s| (*s).to_owned()).collect()
}

/// Writes `lines` back as lcov.
#[must_use]
pub fn render(lines: &Lines) -> String {
    let mut out = String::new();
    for (file, hits) in lines {
        out.push_str(&format!("SF:{}\n", file.display()));
        for (line, n) in hits {
            out.push_str(&format!("DA:{line},{n}\n"));
        }
        out.push_str("end_of_record\n");
    }

    out
}

#[cfg(test)]
mod tests;
