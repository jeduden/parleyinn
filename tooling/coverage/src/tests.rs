//! A fake [`Host`] the unit tests drive the measurement with.

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};

use crate::Host;

/// Records every command and file, answers `cargo metadata` and each
/// `llvm-cov report` from what it was given, and fails the first call
/// whose words contain `fail_on`.
#[derive(Debug, Default)]
pub(crate) struct FakeHost {
    pub metadata: String,
    pub reports: RefCell<VecDeque<String>>,
    pub fail_on: Option<&'static str>,
    pub calls: RefCell<Vec<String>>,
    pub files: RefCell<BTreeMap<PathBuf, String>>,
    pub sources: Vec<(PathBuf, String)>,
    pub scenarios: usize,
}

impl FakeHost {
    fn call(&self, words: String) -> Result<(), String> {
        self.calls.borrow_mut().push(words.clone());
        match self.fail_on {
            Some(needle) if words.contains(needle) => Err(format!("{words}: exit status: 1")),
            _ => Ok(()),
        }
    }
}

impl Host for FakeHost {
    fn cargo(&self, args: &[String]) -> Result<(), String> {
        self.call(format!("cargo {}", args.join(" ")))
    }

    fn cargo_output(&self, args: &[String]) -> Result<String, String> {
        self.call(format!("cargo {}", args.join(" ")))?;
        if args.first().is_some_and(|a| a == "metadata") {
            return Ok(self.metadata.clone());
        }
        Ok(self.reports.borrow_mut().pop_front().unwrap_or_default())
    }

    fn write(&self, path: &Path, text: &str) -> Result<(), String> {
        self.call(format!("write {}", path.display()))?;
        self.files
            .borrow_mut()
            .insert(path.to_path_buf(), text.to_owned());
        Ok(())
    }

    fn read(&self, path: &Path) -> Result<Option<String>, String> {
        self.call(format!("read {}", path.display()))?;
        Ok(self.files.borrow().get(path).cloned())
    }

    fn files(&self, dir: &Path) -> Result<Vec<String>, String> {
        self.call(format!("files {}", dir.display()))?;
        Ok(self
            .files
            .borrow()
            .keys()
            .filter(|p| p.parent() == Some(dir))
            .filter_map(|p| Some(p.file_name()?.to_string_lossy().into_owned()))
            .collect())
    }

    fn append(&self, path: &Path, text: &str) -> Result<(), String> {
        self.call(format!("append {}", path.display()))?;
        self.files
            .borrow_mut()
            .entry(path.to_path_buf())
            .or_default()
            .push_str(text);
        Ok(())
    }

    fn sources(&self, dir: &Path) -> Result<Vec<(PathBuf, String)>, String> {
        self.call(format!("sources {}", dir.display()))?;
        Ok(self
            .sources
            .iter()
            .filter(|(p, _)| p.starts_with(dir))
            .cloned()
            .collect())
    }

    fn bound_scenarios(&self, features: &Path) -> Result<usize, String> {
        self.call(format!("scenarios {}", features.display()))?;
        Ok(self.scenarios)
    }
}
