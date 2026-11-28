//! The `parinn verify` command and its report.

use super::*;

/// `parinn verify [--json]`: checks the record against this node's
/// device key, exiting 3 at the first failure, which it names. No log
/// is signed yet, so a record that passes is `unsigned` (§9.7.5). It
/// changes nothing it checks: a purge a kill cut short is reported,
/// with the search index its residue leaves unchecked, for the next
/// purge, ingest or rebuild to finish (ADM-09, ENG-06).
pub(crate) fn verify_command(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("verify", args, stderr) else {
        return EXIT_USAGE;
    };
    let (home, store) = match verified_store(ctx, stderr) {
        Ok(opened) => opened,
        Err(code) => return code,
    };
    let incomplete = match sources::verified(ctx, &home, &store) {
        Ok(found) => found,
        Err(e) => return failed("verify", &e, stderr),
    };
    let read = store
        .refused_forks()
        .and_then(|forks| Ok((forks, unfinished_purges(ctx, &store)?)));
    let (forks, unfinished) = match read {
        Ok(read) => read,
        Err(e) => return failed("verify", &e, stderr),
    };
    let mut shape = Shape::Checked;
    let verified = match verify_record(&home, &store) {
        Ok(verified) => verified.and_then(|(r, staged)| {
            shape = checked_shape(ctx, &store)?;
            Ok((r, staged))
        }),
        Err(e) => return failed("verify", &e, stderr),
    };
    let checked = match verified {
        Ok((r, staged)) => Log::new(home.audit())
            .verify()
            .map(|a| (r, staged, a))
            .map_err(Err),
        Err(e) => Err(Ok(e)),
    };
    let (code, line) = match checked {
        Ok((r, staged, _)) if !forks.is_empty() => {
            refused_report(as_json, &r, staged, &forks, stderr)
        }
        Ok(_) if incomplete.failure().is_some() => broken_json(
            as_json,
            "transcript_source_completeness",
            "transcript source completeness",
            None,
            &incomplete.failure().unwrap_or_default(),
            stderr,
        ),
        Ok((r, staged, a)) => (
            EXIT_OK,
            verified_line(
                as_json,
                &r,
                staged,
                &a,
                &incomplete,
                Checks {
                    shape,
                    unfinished: &unfinished,
                },
            ),
        ),
        Err(Ok(VerifyError::Store(e))) => return failed("verify", &e, stderr),
        Err(Err(e @ AuditError::Broken { .. })) => {
            broken_json(as_json, "audit", "hash chain", None, &e.to_string(), stderr)
        }
        Err(Err(e)) => return failed("verify", &e, stderr),
        Err(Ok(broken)) => broken_json(
            as_json,
            broken.class(),
            broken.check(),
            broken.address(),
            &broken.to_string(),
            stderr,
        ),
    };
    if line.is_empty() {
        return code;
    }

    match written(writeln!(stdout, "{line}"), stderr) {
        EXIT_OK => code,
        other => other,
    }
}

/// The home and the store `parinn verify` checks, or the exit code of
/// the failure it reported on `stderr`.
pub(crate) fn verified_store(ctx: &Context, stderr: &mut dyn Write) -> Result<(Home, Store), u8> {
    let home = ctx.home().map_err(|e| failed("verify", &e, stderr))?;
    if !home.store().exists() {
        let missing = format!("no store at {}", home.store().display());
        return Err(failed("verify", &missing, stderr));
    }
    let store = Store::open(&home.store()).map_err(|e| store_failed("verify", &e, stderr))?;

    Ok((home, store))
}

/// The ranges a purge a kill cut short has still to clear from the
/// search index, audited and counted when there are any (I6): the next
/// purge, ingest or rebuild finishes them.
pub(crate) fn unfinished_purges(
    ctx: &Context,
    store: &Store,
) -> Result<Vec<String>, store::StoreError> {
    let unfinished = store.unfinished_purges()?;
    if !unfinished.is_empty() {
        purge::audit_residue(ctx);
    }

    Ok(unfinished)
}

/// What `parinn verify` checked beside the record: the store's shape,
/// and the ranges an unfinished purge leaves in the search index.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Checks<'a> {
    shape: Shape,
    unfinished: &'a [String],
}

/// How long `parinn verify` derives the search index again before it
/// leaves the index unchecked (ADM-09, ENG-03), in seconds on its clock.
pub(crate) const INDEX_CHECK_SECS: u64 = 60;

/// The shape of `store`, its search index derived again while
/// [`INDEX_CHECK_SECS`] last on `ctx`'s clock and compared outside the
/// ranges a pending purge has still to clear, whose residue it holds; a
/// derivation that ran out of time is audited and counted (I6).
pub(crate) fn checked_shape(ctx: &Context, store: &Store) -> Result<Shape, VerifyError> {
    let ends = (ctx.clock)().saturating_add(INDEX_CHECK_SECS);
    let shape = administration::verify::verify_shape(store, &|| (ctx.clock)() < ends)?;
    if shape == Shape::IndexUnfinished {
        let _ = ctx.audit(&Occurrence::Degraded {
            kind: Kind::TimedOut,
            counter: Counter::VerifyTimedOut,
            operation: "verify",
            detail: "the search index was not derived again in time and is unchecked",
        });
    }

    Ok(shape)
}

/// The record in `store`, verified against the device key `home` keeps,
/// or against the key an interrupted first start staged, which its next
/// start places, with whether it was staged; an empty record while that
/// start has recorded nothing yet, as a crash can leave a home (ENG-06).
pub(crate) fn verify_record(
    home: &Home,
    store: &Store,
) -> Result<Result<(Report, bool), VerifyError>, administration::AdminError> {
    Ok(match administration::node::verifying_key(home, store)? {
        Some((key, staged)) => {
            let verified = verify(store, &key);
            // The worktrees' names derive from the device key, which a
            // staged first start has not placed yet.
            if verified.is_ok() && !staged {
                let node = Node::find(home, store)?;
                if let Err(e) = administration::verify::kept_worktrees_agree(store, &node) {
                    return Ok(Err(e));
                }
            }
            verified.map(|r| (r, staged))
        }
        None => Ok((
            Report {
                writers: 0,
                events: 0,
                order: Vec::new(),
                rooms: BTreeMap::new(),
                seats: Vec::new(),
                open_segments: Vec::new(),
            },
            true,
        )),
    })
}

/// What `parinn verify` says of a record that verified, `staged` while
/// the node's first start is still to finish, as JSON when asked.
pub(crate) fn verified_line(
    as_json: bool,
    r: &Report,
    staged: bool,
    a: &audit::log::Checked,
    incomplete: &sources::Completeness,
    checks: Checks<'_>,
) -> String {
    let index_checked = checks.shape == Shape::Checked;
    if as_json {
        return json!({
            "integrity": "unsigned",
            "writers": r.writers,
            "events": r.events,
            "order": r.full_order(),
            "seats": seats_json(&r.seats),
            "open_segments": segments::open_json(&r.open_segments),
            "short_sources": incomplete.short,
            "pending_purges": checks.unfinished,
            "index_checked": index_checked,
            "index_checked_outside_pending_purges": index_checked && !checks.unfinished.is_empty(),
            "staged": staged,
            "audit": {"files": a.files, "entries": a.entries},
            "refused": [],
            "broken": null,
        })
        .to_string();
    }
    let staged = if staged {
        "; the node's first start was interrupted, and its next start finishes it"
    } else {
        ""
    };
    let pending = if checks.unfinished.is_empty() {
        String::new()
    } else if index_checked {
        format!(
            "; a purge was cut short before it finished, so its residue stays until the next \
             purge, ingest or rebuild clears it, and the search index was checked outside the \
             address ranges of pending purges: {}",
            checks.unfinished.join(", ")
        )
    } else {
        format!(
            "; a purge was cut short before it finished, so its residue stays until the next \
             purge, ingest or rebuild clears it, in the address ranges of pending purges: {}",
            checks.unfinished.join(", ")
        )
    };
    let unchecked = if index_checked {
        ""
    } else {
        "; the search index was not derived again in time, so it is unchecked: \
         run parinn verify again"
    };

    format!(
        "checked {} writers and {} events: every chain, commitment and seat certificate \
         holds; integrity unsigned, as no log is signed yet, so a log cut at its tail or \
         rewritten whole goes unseen until seals; completeness covers only the transcript \
         sources the record names; the audit log's chain holds over {} entries{}{}{pending}{unchecked}\
         {staged}",
        r.writers,
        r.events,
        a.entries,
        open_text(&r.open_segments),
        incomplete.short_text()
    )
}

/// What `parinn verify` says of a record whose chains hold but which
/// refused a second event at one (writer, seq): each fork on `stderr`,
/// exiting 3, with the integrity status `refused` and the forks as JSON
/// when asked (PEER-10, VIEW-10). The writer is not marked, and no
/// choice between forks is offered: only the two digests are shown.
pub(crate) fn refused_report(
    as_json: bool,
    r: &Report,
    staged: bool,
    forks: &[store::Fork],
    stderr: &mut dyn Write,
) -> (u8, String) {
    for line in forks::lines(forks) {
        let _ = writeln!(stderr, "parinn verify: {line}");
    }
    if !as_json {
        return (EXIT_INTEGRITY, String::new());
    }
    let line = json!({
        "integrity": "refused",
        "writers": r.writers,
        "events": r.events,
        "staged": staged,
        "refused": forks::json(forks),
        "broken": null,
    });

    (EXIT_INTEGRITY, line.to_string())
}

/// What `parinn verify` adds for writer chains that ended without a
/// closed segment: nothing when none did (REC-19).
pub(crate) fn open_text(open: &[store::Segment]) -> String {
    if open.is_empty() {
        return String::new();
    }

    format!(
        "; writer chains ended without a closed segment: {}",
        segments::open_list(open)
    )
}

/// Each seat as `parinn verify --json` reports it: its writer, and its
/// seat id, room, seat kind, ingest flag and seat certificate, all read
/// from that certificate (PRV-11).
pub(crate) fn seats_json(seats: &[administration::verify::SeatReport]) -> Value {
    seats
        .iter()
        .map(|s| {
            json!({
                "writer": s.writer.to_string(),
                "seat": s.seat.to_string(),
                "room": s.room.to_string(),
                "seat_kind": s.seat_kind.as_str(),
                "ingest_seat": s.ingest_seat,
                "certificate": s.certificate,
            })
        })
        .collect()
}

/// Reports a broken record or audit log on `stderr`, exiting 3, with
/// the failure's class, address and message as JSON when asked; the
/// line is empty when no JSON was asked for.
pub(crate) fn broken_json(
    as_json: bool,
    class: &str,
    check: &str,
    address: Option<&str>,
    message: &str,
    stderr: &mut dyn Write,
) -> (u8, String) {
    let _ = writeln!(stderr, "parinn verify: {check}: {message}");
    if !as_json {
        return (EXIT_INTEGRITY, String::new());
    }
    let line = json!({
        "integrity": "broken",
        "writers": null,
        "events": null,
        "order": null,
        "seats": null,
        "audit": null,
        "broken": {"class": class, "check": check, "address": address, "message": message},
    });

    (EXIT_INTEGRITY, line.to_string())
}
