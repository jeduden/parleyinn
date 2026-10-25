//! A stand-in harness process for the steps that need a live MCP
//! server: a script named `claude`, started confined to the core's
//! boundary (ENG-12), that starts `parinn mcp` as its child, its
//! standard input and output the script's, and runs each hook handler
//! a step asks for as another child, so the hook handlers and the MCP
//! server share one harness process, as Claude Code's do (REC-13).
//!
//! A step asks for a hook by writing a request file in the harness's
//! directory, its first line the hook and the rest the hook input; the
//! harness runs it and renames the file `.done`. A `mcp` file starts
//! the MCP server; a `stop` file ends the harness once its MCP server
//! exits.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use store::Store;
use testkit::boundary::{Component, Confinement, Started};

use crate::World;
use crate::record::{environment, home};

/// The stand-in harness.
const SCRIPT: &str = r#"#!/bin/sh
dir="$1"
core="$2"
exec 3<&0
mcp=""
while [ ! -e "$dir/stop" ]; do
  if [ -z "$mcp" ] && [ -e "$dir/mcp" ]; then
    "$core" mcp <&3 &
    mcp=$!
    exec 3<&-
  fi
  for req in "$dir"/req-*; do
    case "$req" in *.done|*.staged) continue ;; esac
    [ -e "$req" ] || continue
    hook=$(head -n 1 "$req")
    tail -n +2 "$req" | "$core" hook "$hook" >/dev/null 2>&1
    mv "$req" "$req.done"
  done
  sleep 0.05
done
if [ -n "$mcp" ]; then wait "$mcp"; fi
"#;

/// How long a step waits for the harness or its MCP server.
pub const PATIENCE: Duration = Duration::from_secs(60);

/// A stand-in harness process, its MCP server started or not.
pub struct Harness {
    started: Started,
    dir: PathBuf,
    replies: Receiver<Value>,
    next: u64,
    requests: u64,
}

impl std::fmt::Debug for Harness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Harness").field("dir", &self.dir).finish()
    }
}

impl Harness {
    /// Starts a harness in the scenario's home, its MCP server at once
    /// when `mcp`, with `extra` in its environment beside the
    /// scenario's.
    pub fn start(w: &mut World, mcp: bool, extra: &[(&str, &str)]) -> Result<Self, String> {
        let base = w.home.join("harnesses");
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
        let n = fs::read_dir(&base).map_err(|e| e.to_string())?.count();
        let dir = base.join(format!("h{n}"));
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let script = dir.join("claude");
        write_executable(&script, SCRIPT)?;
        if mcp {
            fs::write(dir.join("mcp"), "").map_err(|e| e.to_string())?;
        }
        let core = testkit::core_executable()?;
        let mut set = environment(w);
        set.extend(
            extra
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
        );
        let env: Vec<(&str, &str)> = set.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let confinement = Confinement::of(Component::Core).map_err(|e| e.to_string())?;
        let (dir_arg, core_arg) = (dir.display().to_string(), core.display().to_string());
        let mut started = confinement
            .spawn(&script, &[&dir_arg, &core_arg], &w.home, &env)
            .map_err(|e| e.to_string())?;
        let out = started
            .take_stdout()
            .ok_or("the harness has no standard output")?;
        let (tx, replies) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if let Ok(reply) = serde_json::from_str::<Value>(&line)
                    && tx.send(reply).is_err()
                {
                    return;
                }
            }
        });
        let mut harness = Self {
            started,
            dir,
            replies,
            next: 1,
            requests: 0,
        };
        if mcp {
            harness.initialize()?;
        }
        Ok(harness)
    }

    /// Starts its MCP server, and initializes it.
    pub fn start_mcp(&mut self) -> Result<(), String> {
        fs::write(self.dir.join("mcp"), "").map_err(|e| e.to_string())?;
        self.initialize()
    }

    fn initialize(&mut self) -> Result<(), String> {
        let reply = self.request(
            "initialize",
            json!({"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "claude-code", "version": "2.1.296"}}),
        )?;
        if reply["result"]["serverInfo"]["name"] != "parinn" {
            return Err(format!("the MCP server answered initialize with {reply}"));
        }
        self.notify("notifications/initialized")
    }

    /// Runs the hook handler `hook` on `input` as a child of the
    /// harness, and waits for it to finish.
    pub fn hook(&mut self, hook: &str, input: &Value) -> Result<(), String> {
        self.requests += 1;
        let req = self.dir.join(format!("req-{}", self.requests));
        let staged = req.with_extension("staged");
        fs::write(&staged, format!("{hook}\n{input}\n")).map_err(|e| e.to_string())?;
        fs::rename(&staged, &req).map_err(|e| e.to_string())?;
        let done = PathBuf::from(format!("{}.done", req.display()));
        wait_until(&format!("the hook {hook} to finish"), || Ok(done.exists()))
    }

    /// Sends one request and returns its reply.
    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.send(method, params)?;
        self.reply(id)
    }

    /// Sends one request without waiting for its reply; returns its id.
    pub fn send(&mut self, method: &str, params: Value) -> Result<u64, String> {
        let id = self.next;
        self.next += 1;
        self.write(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))?;
        Ok(id)
    }

    /// The reply to request `id`, replies to others passed over.
    pub fn reply(&mut self, id: u64) -> Result<Value, String> {
        let deadline = Instant::now() + PATIENCE;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let reply = self
                .replies
                .recv_timeout(left)
                .map_err(|_| format!("the MCP server did not answer request {id}"))?;
            if reply["id"] == id {
                return Ok(reply);
            }
        }
    }

    fn notify(&mut self, method: &str) -> Result<(), String> {
        self.write(&json!({"jsonrpc": "2.0", "method": method}))
    }

    fn write(&mut self, message: &Value) -> Result<(), String> {
        let stdin = self
            .started
            .stdin()
            .ok_or("the harness's input is closed")?;
        writeln!(stdin, "{message}").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())
    }

    /// Ends its harness session: the MCP server's input closes, and the
    /// harness exits once it does. Fails when either outlives its wait,
    /// or opened a socket (ENG-12).
    pub fn stop(self) -> Result<(), String> {
        fs::write(self.dir.join("stop"), "").map_err(|e| e.to_string())?;
        match self.started.finish(PATIENCE) {
            Ok(true) => Ok(()),
            Ok(false) => Err("the MCP server did not stop when its input closed".to_owned()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Kills the harness and its MCP server, as a crash would. Fails
    /// when either opened a socket (ENG-12).
    pub fn kill(self) -> Result<(), String> {
        self.started.kill().map_err(|e| e.to_string())
    }
}

fn write_executable(path: &Path, body: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, body).map_err(|e| e.to_string())?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())
}

/// Waits until `ready` holds, looking every 50 ms, at most [`PATIENCE`].
pub fn wait_until(
    what: &str,
    mut ready: impl FnMut() -> Result<bool, String>,
) -> Result<(), String> {
    let deadline = Instant::now() + PATIENCE;
    while Instant::now() < deadline {
        if ready()? {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(format!("waited {} s for {what}", PATIENCE.as_secs()))
}

/// The scenario's store, read as it stands.
pub fn store(w: &World) -> Result<Store, String> {
    Store::open(&home(w)?.store()).map_err(|e| e.to_string())
}

/// The key the store keeps the transcript source at `path` by: the
/// path resolved, as ingest reads it.
pub fn source_key(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

/// The line number of each event the record holds of the transcript at
/// `path`, across its generations, in the order each was placed.
pub fn placed_lines(w: &World, path: &Path) -> Result<Vec<u64>, String> {
    let store = store(w)?;
    let mut lines = Vec::new();
    for generation in store
        .generations(&source_key(path))
        .map_err(|e| e.to_string())?
    {
        for placed in store.placed(generation.id).map_err(|e| e.to_string())? {
            lines.push(placed.place.line);
        }
    }
    Ok(lines)
}

/// The number of the last line ingest read of the transcript at `path`.
pub fn lines_read(w: &World, path: &Path) -> Result<u64, String> {
    Ok(store(w)?
        .source(&source_key(path))
        .map_err(|e| e.to_string())?
        .map_or(0, |s| s.position.lines))
}

/// A transcript line in Claude Code 2.1.296's recorded shape, which
/// ingest records with nothing to audit, numbered `n`.
pub fn clean_line(n: u64) -> String {
    format!(
        "{}\n",
        json!({"type": "atis-latch", "atis": format!("line {n}"), "sessionId": "s"})
    )
}

/// Appends `count` clean lines to the transcript at `path`, numbered on
/// from what it holds.
pub fn append_lines(path: &Path, count: u64) -> Result<(), String> {
    let held = fs::read_to_string(path).unwrap_or_default().lines().count() as u64;
    let text: String = (held + 1..=held + count).map(clean_line).collect();
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())
}
