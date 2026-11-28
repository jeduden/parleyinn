//! The `parinn doctor` and `parinn status` commands and the counts they show.

use super::*;

/// The counts the home's audit log gives, their rise counted after the
/// entry the counters were last acknowledged through, and its number of
/// entries; else the exit code, its failure reported: 3 for a broken
/// chain.
pub(crate) fn audit_counts(
    command: &str,
    home: &Home,
    acknowledged: Option<(u64, record::digest::Digest)>,
    stderr: &mut dyn Write,
) -> Result<(Counts, u64), u8> {
    match Log::new(home.audit()).entries() {
        Ok(entries) => Ok((counts(&entries, acknowledged), entries.len() as u64)),
        Err(e @ AuditError::Broken { .. }) => {
            let _ = writeln!(stderr, "parinn {command}: {e}");
            Err(EXIT_INTEGRITY)
        }
        Err(e) => Err(failed(command, &e, stderr)),
    }
}

/// The counts of `home`'s audit log since the counters were last
/// acknowledged, a neutral principal act the record holds (OWN-11), and
/// its number of entries; none before the home exists.
pub(crate) fn home_counts(
    command: &str,
    home: &Home,
    acts: &[act_record::Recorded],
    stderr: &mut dyn Write,
) -> Result<Option<(Counts, u64)>, u8> {
    if !home.root().exists() {
        return Ok(None);
    }

    audit_counts(command, home, act_record::acknowledged(acts), stderr).map(Some)
}

/// `parinn doctor [--json]`: read-only (ADM-10); exits 1 while a counter
/// rose since the counters were last acknowledged, naming each
/// (OPS-03).
pub(crate) fn doctor(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("doctor", args, stderr) else {
        return EXIT_USAGE;
    };
    let read = ctx
        .home()
        .map_err(|e| e.to_string())
        .and_then(|home| Ok((settings::recorded_acts(&home)?, home)));
    let (acts, home) = match read {
        Ok(read) => read,
        Err(e) => return failed("doctor", &e, stderr),
    };
    let counts = match home_counts("doctor", &home, &acts, stderr) {
        Ok(c) => c.map(|(c, _)| c),
        Err(code) => return code,
    };
    let code = if counts.as_ref().is_some_and(Counts::risen) {
        EXIT_FAILURE
    } else {
        EXIT_OK
    };
    let risen = counts.map(|c| c.since_ack);
    let text = if as_json {
        json!({"risen": risen.unwrap_or_default()}).to_string()
    } else {
        match risen {
            None => format!(
                "no home at {} yet: nothing to diagnose",
                home.root().display()
            ),
            Some(r) if r.is_empty() => {
                "no counter rose since the counters were last acknowledged".to_owned()
            }
            Some(r) => r
                .iter()
                .map(|(name, n)| {
                    format!("counter {name} rose by {n} since the counters were last acknowledged")
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    };

    match written(writeln!(stdout, "{text}"), stderr) {
        EXIT_OK => code,
        other => other,
    }
}

/// What `parinn status` shows.
pub(crate) struct Status {
    home: String,
    /// The store's path and size, once the home exists.
    store: Option<(String, Option<u64>)>,
    /// The schema version of the store, once it is made.
    schema_version: Option<u32>,
    /// The audit log's path and number of entries, once the home exists.
    audit: Option<(String, u64)>,
    /// Each counter's total and rise since the last acknowledgement.
    counters: Vec<(String, u64, u64)>,
    deployment_mode: String,
    awaiting_acceptance: Vec<String>,
    /// Why managed policy is refused, when it is.
    managed_policy: Option<String>,
    /// The key store the home's keys are kept in (SEC-10).
    keys: KeyStoreKind,
    /// Where the node identity is read from (REC-24).
    identity: IdentitySource,
    /// The store's open and newer segments, once there is a store.
    segments: Option<segments::SegmentsStatus>,
    /// The transcript sources held or lost, once there is a store.
    sources: Option<sources::SourcesStatus>,
    /// How many session bindings an MCP server took and did not take
    /// (REC-13, REC-19), once any `SessionStart` was observed.
    bindings: Option<(u64, u64)>,
}

impl Status {
    fn json(&self) -> String {
        let counters: BTreeMap<&String, Value> = self
            .counters
            .iter()
            .map(|(name, total, since)| (name, json!({"total": total, "since_ack": since})))
            .collect();
        let mut status = json!({
            "home": self.home,
            "store": self.store.as_ref().map(|(p, b)| {
                json!({"path": p, "bytes": b, "schema_version": self.schema_version})
            }),
            "audit": self.audit.as_ref().map(|(p, n)| json!({"path": p, "entries": n})),
            "counters": counters,
            "deployment_mode": self.deployment_mode,
            "awaiting_acceptance": self.awaiting_acceptance,
            "managed_policy": {"refused": self.managed_policy},
            "keys": {
                "store": self.keys.as_str(),
                "readable_by_agents": self.keys.readable_by_agents(),
                "where": self.keys.describe(),
            },
            "node_identity": identity_status::json(&self.identity),
            "fork_detection": identity_status::fork_detection_json(),
            "segments": self.segments.as_ref().map(segments::SegmentsStatus::json),
            "transcript_sources": self.sources.as_ref().map(sources::SourcesStatus::json),
        });
        // Named once any session binding was observed.
        if let Some((taken, untaken)) = self.bindings {
            status["mcp_server"] = json!({"bindings_taken": taken, "bindings_untaken": untaken});
        }

        status.to_string()
    }

    fn text(&self) -> String {
        let mut lines = Vec::new();
        match (&self.store, &self.audit) {
            (Some((store, bytes)), Some((audit, entries))) => {
                let size = bytes.map_or("none yet".to_owned(), |b| {
                    let schema = self
                        .schema_version
                        .map_or(String::new(), |v| format!(", schema version {v}"));
                    format!("{b} bytes{schema}")
                });
                lines.push(format!("home: {}", self.home));
                lines.push(format!("store: {store} ({size})"));
                lines.push(format!("audit log: {audit} ({entries} entries)"));
            }
            _ => lines.push(format!("home: {} (none yet)", self.home)),
        }
        let counters: Vec<String> = self
            .counters
            .iter()
            .map(|(name, total, since)| format!("{name} {total} ({since} since acknowledged)"))
            .collect();
        if self.store.is_some() {
            let counters = if counters.is_empty() {
                "none".to_owned()
            } else {
                counters.join(", ")
            };
            lines.push(format!("counters: {counters}"));
        }
        lines.push(format!("deployment mode: {}", self.deployment_mode));
        if !self.awaiting_acceptance.is_empty() {
            lines.push(format!(
                "awaiting acceptance (parinn configuration accept): {}",
                self.awaiting_acceptance.join(", ")
            ));
        }
        if let Some(why) = &self.managed_policy {
            lines.push(format!("managed policy refused: {why}"));
        }
        lines.push(format!("keys: {}", self.keys.describe()));
        lines.push(identity_status::node_identity_line(&self.identity));
        lines.push(identity_status::FORK_DETECTION_LINE.to_owned());
        lines.extend(
            self.segments
                .iter()
                .flat_map(segments::SegmentsStatus::lines),
        );
        lines.extend(self.sources.iter().flat_map(sources::SourcesStatus::lines));
        lines.extend(
            self.bindings
                .map(|(taken, untaken)| binding_line(taken, untaken)),
        );

        lines.join("\n")
    }
}

/// What `parinn status` says of the MCP server: whether session bindings
/// are taken (REC-13, REC-19).
pub(crate) fn binding_line(taken: u64, untaken: u64) -> String {
    match (taken, untaken) {
        (0, untaken) => {
            format!("MCP server: none has taken a session binding on this node ({untaken} untaken)")
        }
        (taken, 0) => format!("MCP server: took {taken} session bindings"),
        (taken, untaken) => {
            format!("MCP server: took {taken} session bindings, {untaken} untaken")
        }
    }
}

/// `parinn status [--json]`: the home, the store, the audit log, the
/// non-zero counters and the settings in force (ADM-11, which phase 15
/// completes). It exits 2 while the configuration or the repository
/// configuration fails validation (ADM-04).
pub(crate) fn status(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("status", args, stderr) else {
        return EXIT_USAGE;
    };
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("status", &e, stderr),
    };
    // The store's version is read first, and nothing in a newer store is
    // opened for writing (ADM-05).
    let segments = match segments::read(&home) {
        Ok(s) => s,
        Err(e) => return store_failed("status", &e, stderr),
    };
    let (mut store, acts) = match settings::opened_acts(&home) {
        Ok(opened) => opened,
        Err(e) => return failed("status", &e, stderr),
    };
    let in_force = settings::in_force(ctx, &home, &acts);
    if in_force.loaded.principal_layer_failed() {
        return invalid_layers("status", &in_force.loaded, stderr);
    }
    // Each start marker left behind and each session binding left
    // untaken is counted before the counters are read (NFR-02, REC-19).
    if home.root().exists() {
        hook_handlers::count_unfinished(&home, (ctx.clock)(), None);
    }
    if let Some(store) = store.as_mut() {
        count_untaken_bindings(ctx, store);
    }
    let counted = match home_counts("status", &home, &acts, stderr) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let refused = segments.as_ref().is_some_and(|s| !s.newer.is_empty());
    let mut status = status_of(&home, counted, &in_force);
    status.schema_version = segments.as_ref().map(|s| s.schema_version);
    status.segments = segments;
    status.bindings = store
        .as_ref()
        .and_then(|store| store.binding_counts().ok())
        .filter(|(taken, untaken)| taken + untaken > 0);
    status.sources = match sources::read(&home) {
        Ok(s) => s,
        Err(e) => return store_failed("status", &e, stderr),
    };
    let text = if as_json {
        status.json()
    } else {
        status.text()
    };

    // A refused segment of a newer format version is named and exits 3
    // (ADM-05).
    match written(writeln!(stdout, "{text}"), stderr) {
        EXIT_OK if refused => EXIT_INTEGRITY,
        other => other,
    }
}

/// The status of `home`, given its audit log's counts.
pub(crate) fn status_of(
    home: &Home,
    counted: Option<(Counts, u64)>,
    in_force: &settings::InForce,
) -> Status {
    let (store, audit) = (home.store(), home.audit());
    let mut status = Status {
        home: home.root().display().to_string(),
        store: None,
        schema_version: None,
        audit: None,
        counters: Vec::new(),
        deployment_mode: in_force.resolved.settings.deployment_mode().to_owned(),
        awaiting_acceptance: in_force
            .resolved
            .awaiting_acceptance
            .iter()
            .map(|k| k.name().to_owned())
            .collect(),
        managed_policy: in_force
            .loaded
            .failures
            .iter()
            .find(|f| f.layer == SettingsLayer::ManagedPolicy)
            .map(ToString::to_string),
        keys: home.key_store().kind(),
        identity: home.identity().clone(),
        segments: None,
        sources: None,
        bindings: None,
    };
    if let Some((counts, entries)) = counted {
        let bytes = fs::metadata(&store).ok().map(|m| m.len());
        status.store = Some((store.display().to_string(), bytes));
        status.audit = Some((audit.display().to_string(), entries));
        status.counters = counts
            .total
            .iter()
            .map(|(name, total)| {
                let since = counts.since_ack.get(name).copied().unwrap_or(0);
                (name.clone(), *total, since)
            })
            .collect();
    }

    status
}

/// `s` as a JSON string literal.
pub(crate) fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');

    out
}
