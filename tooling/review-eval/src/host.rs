//! The one seam to the machine: a command the harness starts, with its
//! working directory, extra environment, where its output goes and how
//! long it may run. The tests hand in a fake; the executable's `main`
//! starts real processes.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// A command to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    /// Variables set for the command, over the harness's own.
    pub env: Vec<(String, String)>,
    /// A file the command's standard output goes to; captured when none.
    pub stdout_to: Option<PathBuf>,
    /// How long the command may run before it is killed.
    pub deadline: Option<Duration>,
}

impl Command {
    /// `program` with `args`, run in `cwd`.
    #[must_use]
    pub fn new(program: &str, args: &[&str], cwd: &Path) -> Self {
        Self {
            program: program.to_owned(),
            args: args.iter().map(|a| (*a).to_owned()).collect(),
            cwd: cwd.to_path_buf(),
            env: Vec::new(),
            stdout_to: None,
            deadline: None,
        }
    }

    /// The same command with its standard output written to `path`.
    #[must_use]
    pub fn logging_to(mut self, path: &Path) -> Self {
        self.stdout_to = Some(path.to_path_buf());
        self
    }

    /// The same command, killed after `deadline`.
    #[must_use]
    pub fn within(mut self, deadline: Duration) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// The command line, for messages.
    #[must_use]
    pub fn line(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// How a command ended.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output {
    /// The exit code; none when a signal ended it.
    pub code: Option<i32>,
    /// What it printed, when no file took it.
    pub stdout: String,
    pub stderr: String,
    /// Whether the deadline killed it.
    pub timed_out: bool,
}

impl Output {
    /// Whether the command exited 0.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.code == Some(0) && !self.timed_out
    }
}

/// What starts commands.
pub trait Host: Sync {
    /// Runs `command` to its end, or its deadline.
    ///
    /// # Errors
    ///
    /// Fails when the command cannot start.
    fn run(&self, command: &Command) -> Result<Output, String>;
}

#[cfg(test)]
mod tests;

/// A host for the tests: `git worktree add` copies a fixture tree,
/// `git worktree remove` deletes it, `mdsmith` answers as told, and
/// `claude` runs a closure the test hands in.
#[cfg(test)]
pub(crate) mod fake {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use super::{Command, Host, Output};

    pub(crate) type Claude = Box<dyn Fn(&Command) -> Result<Output, String> + Sync>;

    pub(crate) struct FakeHost {
        /// What a worktree holds.
        pub tree: PathBuf,
        /// Whether `mdsmith` is on the fake's path.
        pub mdsmith: bool,
        /// Whether the `mdsmith` on the path fails.
        pub mdsmith_broken: bool,
        pub claude: Claude,
        pub calls: Mutex<Vec<String>>,
    }

    impl FakeHost {
        pub(crate) fn new(tree: &Path) -> Self {
            Self {
                tree: tree.to_path_buf(),
                mdsmith: true,
                mdsmith_broken: false,
                claude: Box::new(|_| Err("no claude in this test".to_owned())),
                calls: Mutex::new(Vec::new()),
            }
        }

        pub(crate) fn calls(&self) -> Vec<String> {
            self.calls.lock().map(|c| c.clone()).unwrap_or_default()
        }
    }

    /// Copies `from` to `to`, `.git` left out.
    pub(crate) fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
        fs::create_dir_all(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let target = to.join(entry.file_name());
            if entry.file_name() == ".git" {
                continue;
            }
            if entry.file_type()?.is_dir() {
                copy_tree(&entry.path(), &target)?;
            } else {
                fs::copy(entry.path(), target)?;
            }
        }
        Ok(())
    }

    fn exit(code: i32, stdout: &str, stderr: &str) -> Result<Output, String> {
        Ok(Output {
            code: Some(code),
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
            timed_out: false,
        })
    }

    impl Host for FakeHost {
        fn run(&self, command: &Command) -> Result<Output, String> {
            if let Ok(mut calls) = self.calls.lock() {
                calls.push(command.line());
            }
            let args: Vec<&str> = command.args.iter().map(String::as_str).collect();
            match (command.program.as_str(), args.as_slice()) {
                ("git", ["worktree", "add", "--detach", path, "HEAD"]) => {
                    copy_tree(&self.tree, Path::new(path)).map_err(|e| e.to_string())?;
                    exit(0, "", "")
                }
                ("git", ["worktree", "remove", "--force", path]) => {
                    fs::remove_dir_all(path).map_err(|e| e.to_string())?;
                    exit(0, "", "")
                }
                ("git", ["worktree", "prune"]) => exit(0, "", ""),
                ("mdsmith", _) if self.mdsmith && self.mdsmith_broken => {
                    exit(1, "", "mdsmith: no such command\n")
                }
                ("mdsmith", _) if self.mdsmith => exit(0, "[{\"name\":\"include\"}]\n", ""),
                ("mdsmith", _) => Err("mdsmith: not found".to_owned()),
                ("claude", _) => (self.claude)(command),
                (program, _) => Err(format!("{program}: not found")),
            }
        }
    }
}
