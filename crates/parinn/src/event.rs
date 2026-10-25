//! `parinn event search|expand` (RCL-08, SEC-06): the principal's search
//! and expand, through the code the MCP tools `event_search` and
//! `event_expand` run, with their envelope and limits. The principal's
//! command line is no run, so a call serves the scope `room`: the
//! personal room, or the room `--room` names.
//!
//! The output is text for a terminal: each item's closed fields, then
//! its content, and every character a terminal would act on, such as an
//! escape sequence, a carriage return or a direction override, is shown
//! as a placeholder naming its code point, never written raw (SEC-06).
//! The verbs write no recall event: which writer records the recall of
//! a command line is open (OQ-50 (h)).

use std::io::Write;

use administration::node::Node;
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

/// The envelope as text for a terminal.
fn render(envelope: &Value) -> String {
    let mut out = format!(
        "{}\n",
        safe(envelope["warning"].as_str().unwrap_or_default())
    );
    for item in envelope["items"].as_array().into_iter().flatten() {
        let field = |name: &str| safe(item[name].as_str().unwrap_or_default());
        out.push_str(&format!(
            "\n{} {} {} trust={} origin={} run={}",
            field("address"),
            field("kind"),
            field("provenance"),
            field("trust"),
            field("origin"),
            field("run"),
        ));
        let flags: Vec<String> = item["flags"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|f| safe(f.as_str().unwrap_or_default()))
            .collect();
        if !flags.is_empty() {
            out.push_str(&format!(" flags={}", flags.join(",")));
        }
        if let Some(score) = item["score"].as_f64() {
            out.push_str(&format!(" score={score:.3}"));
        }
        out.push_str(&format!(
            "\n{}\n",
            safe(item["content"].as_str().unwrap_or_default())
        ));
    }
    if let Some(cursor) = envelope["next_cursor"].as_str() {
        out.push_str(&format!(
            "\ncut at the cap: continue with --cursor {}\n",
            safe(cursor)
        ));
    }

    out
}

/// `text` with each character a terminal would act on shown as
/// `<U+XXXX>`, naming its code point: controls but the line break and
/// the tab, and the invisible format characters that reorder or hide
/// text, such as the direction overrides (SEC-06).
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
    if c == '\n' || c == '\t' {
        return false;
    }

    c.is_control()
        || matches!(
            u32::from(c),
            0x00AD | 0x0600..=0x0605 | 0x061C | 0x06DD | 0x070F | 0x180E
                | 0x200B..=0x200F | 0x2028..=0x202E | 0x2060..=0x206F | 0xFEFF | 0xFFF9..=0xFFFB
        )
}

#[cfg(test)]
mod tests;
