//! One measurement: per test layer, run its tests on the instrumented
//! build, export what they ran, and clear the profiles for the next
//! layer.

use std::path::Path;

use crate::Host;
use crate::lcov::{self, Lines};
use crate::plan::{Plan, TestLayer};

pub use scenario::runner::Shard;

/// The file a shard's lcov is written to, such as `e2e-2-of-4.lcov`.
#[must_use]
pub fn shard_file_name(layer: TestLayer, shard: Shard) -> String {
    format!("{layer}-{}-of-{}.lcov", shard.index, shard.total)
}

/// The shard a file name spells for `layer`, or `None` for a name that
/// is not a shard file of the layer.
#[must_use]
pub fn shard_from_file_name(layer: TestLayer, name: &str) -> Option<Shard> {
    let (index, total) = name
        .strip_prefix(layer.name())?
        .strip_prefix('-')?
        .strip_suffix(".lcov")?
        .split_once("-of-")?;
    format!("{index}/{total}").parse().ok()
}

/// What one measurement of a layer runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// Every test of the layer.
    Whole,
    /// One shard of the end-to-end layer's scenarios.
    Shard(Shard),
    /// The end-to-end layer's targets other than the scenario runner,
    /// which its shards build and do not run.
    Others,
}

/// The file a part's lcov is written to: `e2e.lcov`, `e2e-2-of-4.lcov`
/// or `e2e-others.lcov`.
#[must_use]
pub fn part_file_name(layer: TestLayer, part: Part) -> String {
    match part {
        Part::Whole => format!("{layer}.lcov"),
        Part::Shard(shard) => shard_file_name(layer, shard),
        Part::Others => others_file_name(layer),
    }
}

/// The file the targets beside the scenario runner write.
fn others_file_name(layer: TestLayer) -> String {
    format!("{layer}-others.lcov")
}

/// The scenario runner, the one end-to-end target that shards.
const BDD: &str = "bdd";

/// The start of every cargo command that runs tests without a report.
fn base_args() -> Vec<String> {
    words(&["llvm-cov", "--no-report", "--workspace", "--locked"])
}

/// The cargo commands that run `layer`'s tests, in order; none when the
/// layer has no tests. A shard of the end-to-end layer runs `bdd` with
/// `--shard K/N`; its others run apart, in one measurement of their
/// own, so the shards and the others together run each test once. The
/// scenarios start the instrumented core, which cargo builds only beside
/// a target of its own crate, so every shard lists the others' tests,
/// which builds them and the core and runs no test.
///
/// # Errors
///
/// Fails when a shard or the others are asked of a layer but the
/// end-to-end one, or of an end-to-end layer without a `bdd` target to
/// divide, and when the others are asked of a layer with none.
pub fn test_runs(plan: &Plan, layer: TestLayer, part: Part) -> Result<Vec<Vec<String>>, String> {
    let shard = match part {
        Part::Whole => return Ok(test_args(plan, layer).into_iter().collect()),
        Part::Shard(shard) => shard,
        Part::Others => {
            let others = others(plan, layer, "set apart")?;
            if others.is_empty() {
                return Err(format!("no {layer} targets beside bdd to measure"));
            }
            return Ok(vec![with_targets(&others)]);
        }
    };
    let mut runs = Vec::new();
    let others = others(plan, layer, "shard")?;
    if !others.is_empty() {
        let mut args = with_targets(&others);
        args.extend(["--", "--list"].map(str::to_owned));
        runs.push(args);
    }
    let mut args = base_args();
    args.extend(["--test", BDD, "--", "--shard"].map(str::to_owned));
    args.push(format!("{}/{}", shard.index, shard.total));
    runs.push(args);

    Ok(runs)
}

/// The end-to-end layer's targets other than `bdd`, for a measurement
/// that would `divide` the layer around it.
///
/// # Errors
///
/// Fails when `layer` is not the end-to-end one or has no `bdd` target.
fn others(plan: &Plan, layer: TestLayer, divide: &str) -> Result<Vec<String>, String> {
    let targets = plan
        .targets
        .get(&layer)
        .filter(|_| layer == TestLayer::EndToEnd);
    let Some(targets) = targets.filter(|t| t.iter().any(|t| t == BDD)) else {
        return Err(format!(
            "no bdd target in the {layer} test layer to {divide}"
        ));
    };

    Ok(targets.iter().filter(|t| *t != BDD).cloned().collect())
}

/// The cargo arguments that run `targets` without a report.
fn with_targets(targets: &[String]) -> Vec<String> {
    let mut args = base_args();
    args.extend(
        targets
            .iter()
            .flat_map(|t| ["--test".to_owned(), t.clone()]),
    );

    args
}

/// The cargo arguments that run `layer`'s tests without a report, or
/// `None` when the layer has no tests.
#[must_use]
pub fn test_args(plan: &Plan, layer: TestLayer) -> Option<Vec<String>> {
    match layer {
        TestLayer::Unit => {
            let mut args = base_args();
            args.extend(["--lib", "--bins"].map(str::to_owned));
            Some(args)
        }
        layer => {
            let targets = plan.targets.get(&layer).filter(|t| !t.is_empty())?;
            Some(with_targets(targets))
        }
    }
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
        let Some(lines) = run_layer(host, plan, layer, Part::Whole, out)? else {
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
/// [`merge_layers`]. A shard of the end-to-end layer measures its share
/// of the scenarios alone and writes `out/e2e-K-of-N.lcov` instead, and
/// its others write `out/e2e-others.lcov`.
///
/// # Errors
///
/// Fails as [`measure`] does, and when a shard or the others are asked
/// of a layer that does not divide.
pub fn measure_layer(
    host: &dyn Host,
    plan: &Plan,
    layer: TestLayer,
    part: Part,
    out: &Path,
) -> Result<Option<Lines>, String> {
    host.cargo(&words(&["llvm-cov", "clean", "--workspace"]))?;
    run_layer(host, plan, layer, part, out)
}

/// Reads the layers [`measure_layer`] wrote under `out`, leaving out a
/// layer with no file, and writes all of them together to
/// `out/all.lcov`. A layer measured in shards is read from all its
/// `<layer>-K-of-N.lcov` files, which must be every shard of the one
/// count, and from `<layer>-others.lcov`, the targets beside the
/// scenario runner; their sum is written to `out/<layer>.lcov`,
/// replacing the merge of an earlier run.
///
/// # Errors
///
/// Fails when a file cannot be read or is not lcov, when a layer's
/// shards or its others are not complete, or when no layer was written.
pub fn merge_layers(host: &dyn Host, out: &Path) -> Result<Vec<(TestLayer, Lines)>, String> {
    let names = host.files(out)?;
    let mut layers = Vec::new();
    for layer in TestLayer::ALL {
        if let Some(lines) = read_layer(host, layer, &names, out)? {
            layers.push((layer, lines));
        }
    }
    if layers.is_empty() {
        return Err(format!("no <layer>.lcov under {}", out.display()));
    }
    write_all(host, &layers, out)?;

    Ok(layers)
}

/// One layer's lines from the files under `out` named `names`: its
/// shard files and its others' file when it has shards, else its own
/// `<layer>.lcov`.
fn read_layer(
    host: &dyn Host,
    layer: TestLayer,
    names: &[String],
    out: &Path,
) -> Result<Option<Lines>, String> {
    let shards: Vec<Shard> = names
        .iter()
        .filter_map(|n| shard_from_file_name(layer, n))
        .collect();
    let others = out.join(others_file_name(layer));
    let Some(first) = shards.first() else {
        if names.contains(&others_file_name(layer)) {
            return Err(format!(
                "{} has no {layer} shards beside it",
                others.display()
            ));
        }
        let text = host.read(&out.join(format!("{layer}.lcov")))?;
        return text.map(|t| lcov::parse(&t)).transpose();
    };
    let total = first.total;
    if let Some(odd) = shards.iter().find(|s| s.total != total) {
        return Err(format!(
            "the {layer} shards under {} disagree on their count: {total} and {}",
            out.display(),
            odd.total
        ));
    }
    let mut parts = Vec::new();
    for index in 1..=total {
        let path = out.join(shard_file_name(layer, Shard { index, total }));
        let Some(text) = host.read(&path)? else {
            return Err(format!(
                "the {layer} shards under {} lack shard {index} of {total}",
                out.display()
            ));
        };
        parts.push(lcov::parse(&text)?);
    }
    let Some(text) = host.read(&others)? else {
        return Err(format!(
            "the {layer} shards under {} lack {}, the targets beside bdd",
            out.display(),
            others_file_name(layer)
        ));
    };
    parts.push(lcov::parse(&text)?);
    let merged = lcov::merge(&parts);
    host.write(&out.join(format!("{layer}.lcov")), &render(&merged))?;

    Ok(Some(merged))
}

/// Runs `layer`'s tests, exports and writes their lcov, and returns
/// what it parsed; `None` when the layer has no tests.
fn run_layer(
    host: &dyn Host,
    plan: &Plan,
    layer: TestLayer,
    part: Part,
    out: &Path,
) -> Result<Option<Lines>, String> {
    let runs = test_runs(plan, layer, part)?;
    if runs.is_empty() {
        return Ok(None);
    }
    for args in &runs {
        host.cargo(args)?;
    }
    let text = host.cargo_output(&report_args(plan))?;
    host.write(&out.join(part_file_name(layer, part)), &text)?;

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
