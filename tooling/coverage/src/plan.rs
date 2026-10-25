//! What a measurement runs, read from `cargo metadata --no-deps`: the
//! test layers and the test targets in each, the crates to report on,
//! and the policy in `[workspace.metadata.coverage]`.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use serde_json::Value;

/// A layer of the test pyramid, told apart by where a test lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
pub enum TestLayer {
    /// The tests beside the code, in each crate's `src/`.
    Unit,
    /// A crate's `tests/` targets that are not end-to-end ones.
    Integration,
    /// The scenario runner `bdd` and every `tests/e2e*` target: they
    /// drive the system through its entry points, a built executable
    /// run as a process among them.
    EndToEnd,
}

impl TestLayer {
    /// Every test layer, from the pyramid's base up.
    pub const ALL: [Self; 3] = [Self::Unit, Self::Integration, Self::EndToEnd];

    /// The layer's name in reports, file names and Codecov flags.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Integration => "integration",
            Self::EndToEnd => "e2e",
        }
    }

    /// The layer `name` names, as [`Self::name`] spells it.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.name() == name)
    }

    /// The layer a `tests/` target named `target` belongs to.
    #[must_use]
    pub fn of_test_target(target: &str) -> Self {
        if target == "bdd" || target.starts_with("e2e") {
            Self::EndToEnd
        } else {
            Self::Integration
        }
    }
}

impl fmt::Display for TestLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Line-coverage floors in percent. A floor left out is not enforced.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Floors {
    /// Every test layer together.
    pub all: Option<f64>,
    pub unit: Option<f64>,
    pub integration: Option<f64>,
    pub e2e: Option<f64>,
}

impl Floors {
    /// The floor for one test layer.
    #[must_use]
    pub fn of(&self, layer: TestLayer) -> Option<f64> {
        match layer {
            TestLayer::Unit => self.unit,
            TestLayer::Integration => self.integration,
            TestLayer::EndToEnd => self.e2e,
        }
    }

    /// These floors, each one `over` sets replacing it.
    #[must_use]
    pub fn overridden_by(self, over: Self) -> Self {
        Self {
            all: over.all.or(self.all),
            unit: over.unit.or(self.unit),
            integration: over.integration.or(self.integration),
            e2e: over.e2e.or(self.e2e),
        }
    }
}

/// One crate of the workspace, by name and directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crate {
    pub name: String,
    pub dir: PathBuf,
}

/// Everything a measurement needs to know before it runs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Plan {
    pub crates: Vec<Crate>,
    /// The `tests/` targets of each test layer above the unit one.
    pub targets: BTreeMap<TestLayer, Vec<String>>,
    /// The source files left out of the measure: a regular expression
    /// cargo-llvm-cov applies to their paths.
    pub exclude: String,
    /// The floors every crate meets.
    pub floors: Floors,
    /// Floors for one crate, over the default ones.
    pub crate_floors: BTreeMap<String, Floors>,
    /// Path endings of the files that are an executable's entry point,
    /// such as `src/main.rs`. The end-to-end tests prove them, so the
    /// unit and integration measures leave them out.
    pub entry_points: Vec<String>,
    /// Whether the test pyramid's shape is enforced: more unit tests
    /// than integration tests, and more of those than end-to-end ones.
    pub pyramid: bool,
    /// The workspace's root directory.
    pub root: PathBuf,
}

impl Plan {
    /// Reads the plan from `cargo metadata --no-deps --format-version 1`.
    ///
    /// # Errors
    ///
    /// Fails when `metadata` is not cargo's JSON, or its coverage policy
    /// is malformed.
    pub fn from_metadata(metadata: &str) -> Result<Self, String> {
        let value: Value =
            serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
        let mut plan = Self::default();
        for package in value["packages"].as_array().into_iter().flatten() {
            let (Some(name), Some(dir)) = (
                package["name"].as_str(),
                package["manifest_path"]
                    .as_str()
                    .and_then(|m| PathBuf::from(m).parent().map(PathBuf::from)),
            ) else {
                return Err(format!(
                    "cargo metadata: a package without a name or manifest: {package}"
                ));
            };
            let name = name.to_owned();
            plan.crates.push(Crate { name, dir });
            for target in package["targets"].as_array().into_iter().flatten() {
                if target["kind"]
                    .as_array()
                    .is_some_and(|k| k.iter().any(|k| k == "test"))
                {
                    let name = target["name"].as_str().unwrap_or_default().to_owned();
                    plan.targets
                        .entry(TestLayer::of_test_target(&name))
                        .or_default()
                        .push(name);
                }
            }
        }
        for names in plan.targets.values_mut() {
            names.sort();
            names.dedup();
        }

        let policy = &value["metadata"]["coverage"];
        known_keys(
            policy,
            &["exclude", "entry-points", "floors", "pyramid", "crates"],
            "coverage",
        )?;
        plan.root = PathBuf::from(value["workspace_root"].as_str().unwrap_or_default());
        plan.exclude = typed(
            &policy["exclude"],
            Value::as_str,
            "exclude",
            "regular expression",
        )?
        .unwrap_or_default()
        .to_owned();
        plan.pyramid =
            typed(&policy["pyramid"], Value::as_bool, "pyramid", "boolean")?.unwrap_or_default();
        for entry in policy["entry-points"].as_array().into_iter().flatten() {
            let entry = entry
                .as_str()
                .ok_or_else(|| format!("coverage entry-points: {entry} is no path"))?;
            plan.entry_points.push(entry.to_owned());
        }
        plan.floors = floors(&policy["floors"]).map_err(|e| format!("coverage floors: {e}"))?;
        for (name, over) in policy["crates"].as_object().into_iter().flatten() {
            if !plan.crates.iter().any(|c| &c.name == name) {
                return Err(format!("coverage crates: {name:?} is no workspace crate"));
            }
            known_keys(over, &["floors"], &format!("coverage crates.{name}"))?;
            let over =
                floors(&over["floors"]).map_err(|e| format!("coverage floors of {name}: {e}"))?;
            plan.crate_floors.insert(name.clone(), over);
        }

        Ok(plan)
    }

    /// The floors `name` meets: the default ones, with the crate's own
    /// over them.
    #[must_use]
    pub fn floors_of(&self, name: &str) -> Floors {
        self.crate_floors
            .get(name)
            .map_or(self.floors, |over| self.floors.overridden_by(*over))
    }

    /// Whether `file` is an executable's entry point.
    #[must_use]
    pub fn is_entry_point(&self, file: &std::path::Path) -> bool {
        self.entry_points.iter().any(|e| file.ends_with(e))
    }

    /// The crate whose directory holds `file`: the deepest one, so a
    /// crate nested in another's directory keeps its own files.
    #[must_use]
    pub fn crate_of(&self, file: &std::path::Path) -> Option<&Crate> {
        self.crates
            .iter()
            .filter(|c| file.starts_with(&c.dir))
            .max_by_key(|c| c.dir.components().count())
    }
}

/// Refuses a key of the `table` at `place` that `known` does not name,
/// so a misspelled key cannot switch a floor off.
fn known_keys(table: &Value, known: &[&str], place: &str) -> Result<(), String> {
    match table
        .as_object()
        .into_iter()
        .flatten()
        .find(|(k, _)| !known.contains(&k.as_str()))
    {
        Some((key, _)) => Err(format!("{place}: unknown key {key:?}")),
        None => Ok(()),
    }
}

/// Reads the policy value `key` through `read`: `None` when it is left
/// out, and an error when it is set to a value of another type, so a
/// wrongly typed key cannot switch its check off.
fn typed<'a, T>(
    value: &'a Value,
    read: impl Fn(&'a Value) -> Option<T>,
    key: &str,
    kind: &str,
) -> Result<Option<T>, String> {
    if value.is_null() {
        return Ok(None);
    }
    read(value)
        .map(Some)
        .ok_or_else(|| format!("coverage {key}: {value} is no {kind}"))
}

/// Reads a `{ all = .., unit = .., integration = .., e2e = .. }` table
/// of percentages; `null` reads as no floors.
fn floors(value: &Value) -> Result<Floors, String> {
    if value.is_null() {
        return Ok(Floors::default());
    }
    let table = value.as_object().ok_or("want a table of percentages")?;
    let mut out = Floors::default();
    for (key, percent) in table {
        let percent = percent
            .as_f64()
            .filter(|p| (0.0..=100.0).contains(p))
            .ok_or_else(|| format!("{key} = {percent} is no percentage"))?;
        let slot = match key.as_str() {
            "all" => &mut out.all,
            "unit" => &mut out.unit,
            "integration" => &mut out.integration,
            "e2e" => &mut out.e2e,
            other => return Err(format!("{other:?} is no test layer")),
        };
        *slot = Some(percent);
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
