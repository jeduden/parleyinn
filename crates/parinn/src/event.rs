//! `parinn event search|expand` (RCL-08, SEC-06): the principal's search
//! and expand, through the code the MCP tools `event_search` and
//! `event_expand` run, with their envelope and limits. The principal's
//! command line is no run, so a call serves the scope `room`: the
//! personal room, or the room `--room` names.
//!
//! The output is text for a terminal: each item's closed fields on a
//! header line, then its content, every line of it indented so that no
//! content line can pass for a header, and every character a terminal would act on, such as an
//! escape sequence, a carriage return or a direction override, is shown
//! as a placeholder naming its code point, never written raw (SEC-06).
//! The verbs write no recall event: which writer records the recall of
//! a command line is open (OQ-50 (h)).

use std::io::Write;

use administration::node::Node;
use audit::entry::{Counter, Kind, Occurrence};
use serde_json::{Value, json};

use crate::{Context, EXIT_FAILURE, EXIT_USAGE, failed, start_node, written};

/// What a command line asked for.
struct Asked {
    tool: &'static str,
    arguments: Value,
}

/// Reads the command line after `event`.
fn parse(mut args: impl Iterator<Item = String>) -> Result<Asked, String> {
    let tool = match args.next().as_deref() {
        Some("search") => "event_search",
        Some("expand") => "event_expand",
        _ => return Err("want search or expand".to_owned()),
    };
    let mut arguments = json!({"scope": "room"});
    let mut words: Vec<String> = Vec::new();
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| args.next().ok_or(format!("{flag} wants a value"));
        match arg.as_str() {
            "--room" => arguments["room"] = json!(value("--room")?),
            "--cursor" if tool == "event_expand" => arguments["cursor"] = json!(value("--cursor")?),
            "--k" if tool == "event_search" => {
                let k = value("--k")?;
                arguments["k"] = json!(k.parse::<u64>().map_err(|_| "--k wants a whole number")?);
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag:?}")),
            _ => words.push(arg),
        }
    }
    let text = words.join(" ");
    match tool {
        "event_search" if text.is_empty() => return Err("search wants words to find".to_owned()),
        "event_search" => arguments["query"] = json!(text),
        _ if words.len() > 1 => return Err("expand wants one range".to_owned()),
        _ if !words.is_empty() => arguments["range"] = json!(text),
        _ if arguments.get("cursor").is_none() => {
            return Err("expand wants a range or a --cursor".to_owned());
        }
        _ => {}
    }

    Ok(Asked { tool, arguments })
}

/// `parinn event search|expand`.
pub(crate) fn event(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let asked = match parse(args) {
        Ok(asked) => asked,
        Err(e) => {
            let _ = writeln!(stderr, "parinn event: {e}");
            return EXIT_USAGE;
        }
    };
    if !ctx.terminal {
        return refuse_off_a_terminal(ctx, stderr);
    }
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("event", &e, stderr),
    };
    let served = start_node(ctx, &home, stderr).and_then(|store| {
        let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
        let server = mcp::Server::new(
            store,
            ctx.build.version,
            node.device().public(),
            node.personal_room(),
        );

        Ok(server.with_labeller(node.device_writer()))
    });
    let mut server = match served {
        Ok(server) => server,
        Err(e) => return failed("event", &e, stderr),
    };
    let request = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": asked.tool, "arguments": asked.arguments}});
    let reply = server
        .handle(&request.to_string())
        .and_then(|r| serde_json::from_str::<Value>(&r).ok())
        .map(|r| r["result"].clone())
        .unwrap_or(Value::Null);
    let text = reply["content"][0]["text"].as_str().unwrap_or_default();
    if reply["isError"] != false {
        let _ = writeln!(stderr, "parinn event: {}", safe(text));
        return EXIT_FAILURE;
    }
    let envelope: Value = serde_json::from_str(text).unwrap_or(Value::Null);

    written(stdout.write_all(render(&envelope).as_bytes()), stderr)
}

/// Why a call off a terminal is counted.
const REFUSED_OFF_A_TERMINAL: &str = "a recall verb was refused: its output is not a terminal, so it could not carry recalled content";

/// Refuses a call whose output is no terminal: what it would write is
/// recalled content outside the envelope, which an agent running the
/// verb through its shell would get (RCL-04). Counts and audits it (I6).
fn refuse_off_a_terminal(ctx: &Context, stderr: &mut dyn Write) -> u8 {
    let occurrence = Occurrence::Degraded {
        kind: Kind::Rejected,
        counter: Counter::RecallRefused,
        operation: "event",
        detail: REFUSED_OFF_A_TERMINAL,
    };
    if let Err(e) = ctx.audit(&occurrence) {
        let _ = writeln!(stderr, "parinn event: audit the refusal: {e}");
    }

    failed(
        "event",
        &"refused: recalled content is written to a terminal only",
        stderr,
    )
}

/// What starts each line of an item's content, so that no content line
/// begins at the margin where a header line does: content cannot forge
/// a header, whatever line breaks it holds (SEC-06).
const MARGIN: &str = "    ";

/// The envelope as text for a terminal.
fn render(envelope: &Value) -> String {
    let mut out = format!(
        "{}\n",
        inline(envelope["warning"].as_str().unwrap_or_default())
    );
    for item in envelope["items"].as_array().into_iter().flatten() {
        let field = |name: &str| inline(item[name].as_str().unwrap_or_default());
        out.push_str(&format!(
            "\n{} {} {} trust={} origin={} run={}",
            field("address"),
            field("kind"),
            field("provenance"),
            field("trust"),
            field("origin"),
            field("run"),
        ));
        for (name, label) in [("flags", "flags"), ("continuation", "continuation")] {
            let words: Vec<String> = item[name]
                .as_array()
                .into_iter()
                .flatten()
                .map(|w| inline(w.as_str().unwrap_or_default()))
                .collect();
            if !words.is_empty() {
                out.push_str(&format!(" {label}={}", words.join(",")));
            }
        }
        if let Some(score) = item["score"].as_f64() {
            out.push_str(&format!(" score={score:.3}"));
        }
        out.push('\n');
        out.push_str(&indented(item["content"].as_str().unwrap_or_default()));
    }
    if let Some(cursor) = envelope["next_cursor"].as_str() {
        out.push_str(&format!(
            "\ncut at the cap: continue with --cursor {}\n",
            inline(cursor)
        ));
    }

    out
}

/// `text` as lines of content: each starts with the margin, so no line
/// of it can be taken for a header, and ends with a line break.
fn indented(text: &str) -> String {
    safe(text)
        .split('\n')
        .map(|line| format!("{MARGIN}{line}\n"))
        .collect()
}

/// `text` safe to put on a header line: as [`safe`], and a line break
/// shown as a placeholder too.
fn inline(text: &str) -> String {
    safe(text).replace('\n', "<U+000A>")
}

/// `text` with each character a terminal would act on shown as
/// `<U+XXXX>`, naming its code point: controls but the line break, and
/// the invisible format characters that reorder or hide text, such as
/// the direction overrides, the tag characters and the variation
/// selectors (SEC-06).
pub(crate) fn safe(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if acts(c) {
            out.push_str(&format!("<U+{:04X}>", u32::from(c)));
        } else {
            out.push(c);
        }
    }

    out
}

/// Whether a terminal may act on `c`, or `c` hides or reorders what is
/// shown.
fn acts(c: char) -> bool {
    if c == '\n' {
        return false;
    }

    c.is_control()
        || matches!(
            u32::from(c),
            0x00AD | 0x034F | 0x0600..=0x0605 | 0x061C | 0x06DD | 0x070F | 0x115F..=0x1160
                | 0x17B4..=0x17B5 | 0x180B..=0x180F | 0x200B..=0x200F | 0x2028..=0x202E
                | 0x2060..=0x206F | 0x3164 | 0xFE00..=0xFE0F | 0xFEFF | 0xFFA0
                | 0xFFF9..=0xFFFB | 0x1BCA0..=0x1BCA3 | 0x1D173..=0x1D17A
                | 0xE0000..=0xE007F | 0xE0100..=0xE01EF
        )
}

#[cfg(test)]
mod tests;
