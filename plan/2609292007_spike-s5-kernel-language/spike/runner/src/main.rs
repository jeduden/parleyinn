//! Spike S5 runner: executes one Lua chunk or one SQL script over the
//! fixture record under the kernel's limits (CMP-04, CMP-05), and prints
//! the answer as JSON: `{"ok":true,"value":...}` or
//! `{"ok":false,"error":"..."}`.
//!
//!     s5-runner lua record.json solution.lua
//!     s5-runner sql record.json solution.sql

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use fallible_iterator::FallibleIterator;
use mlua::{HookTriggers, Lua, LuaOptions, StdLib, Value as LuaValue, VmState};
use rusqlite::functions::FunctionFlags;
use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::types::ValueRef;
use rusqlite::{Batch, Connection};
use serde_json::{Map, Number, Value as Json, json};

const WALL_CLOCK: Duration = Duration::from_secs(10);
const MEMORY: usize = 512 << 20;
const LUA_INSTRUCTIONS: u64 = 500_000_000;
const SQL_STEPS: u64 = 500_000_000;
const COLUMNS: [&str; 12] = [
    "writer",
    "seq",
    "run",
    "ts",
    "provenance",
    "kind",
    "tool",
    "call_id",
    "status",
    "tokens",
    "text",
    "args",
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = match args.as_slice() {
        [_, lang, record, solution] => run(lang, record, solution),
        _ => Err("usage: s5-runner <lua|sql> <record.json> <solution>".into()),
    };
    let doc = match out {
        Ok(value) => json!({"ok": true, "value": value}),
        Err(error) => json!({"ok": false, "error": error}),
    };
    println!("{doc}");
}

fn run(lang: &str, record: &str, solution: &str) -> Result<Json, String> {
    let record = std::fs::read_to_string(record).map_err(|e| format!("read record: {e}"))?;
    let events: Vec<Json> =
        serde_json::from_str(&record).map_err(|e| format!("parse record: {e}"))?;
    let code = std::fs::read_to_string(solution).map_err(|e| format!("read solution: {e}"))?;
    match lang {
        "lua" => run_lua(&events, &code),
        "sql" => run_sql(&events, &code),
        other => Err(format!("unknown language {other}")),
    }
}

// ---------------------------------------------------------------- Lua

fn run_lua(events: &[Json], code: &str) -> Result<Json, String> {
    let lua = sandbox().map_err(|e| e.to_string())?;
    install_record(&lua, events).map_err(|e| e.to_string())?;
    let start = Instant::now();
    let count = Rc::new(Cell::new(0u64));
    let seen = count.clone();
    let step = 10_000;
    lua.set_hook(
        HookTriggers::new().every_nth_instruction(step as u32),
        move |_, _| {
            seen.set(seen.get() + step);
            if seen.get() > LUA_INSTRUCTIONS {
                return Err(mlua::Error::runtime("step budget exceeded"));
            }
            if start.elapsed() > WALL_CLOCK {
                return Err(mlua::Error::runtime("wall-clock limit exceeded"));
            }
            Ok(VmState::Continue)
        },
    )
    .map_err(|e| e.to_string())?;
    let value: LuaValue = lua
        .load(code)
        .set_name("=kernel")
        .eval()
        .map_err(|e| e.to_string())?;
    lua_to_json(&value, 0)
}

/// A Lua state holding only the allow-listed globals: the base library
/// without loaders, string, table, math without randomness, utf8, and
/// the host's json, re and parinn tables. No io, os, package or debug.
fn sandbox() -> mlua::Result<Lua> {
    let lua = Lua::new_with(
        StdLib::STRING | StdLib::TABLE | StdLib::MATH | StdLib::UTF8,
        LuaOptions::default(),
    )?;
    lua.set_memory_limit(MEMORY)?;
    let g = lua.globals();
    for name in ["dofile", "loadfile", "load", "require", "collectgarbage", "print"] {
        g.set(name, LuaValue::Nil)?;
    }
    let math: mlua::Table = g.get("math")?;
    math.set("random", LuaValue::Nil)?;
    math.set("randomseed", LuaValue::Nil)?;
    let string: mlua::Table = g.get("string")?;
    string.set("dump", LuaValue::Nil)?;

    let json_t = lua.create_table()?;
    json_t.set(
        "decode",
        lua.create_function(|lua, s: String| {
            let v: Json = serde_json::from_str(&s).map_err(mlua::Error::external)?;
            json_to_lua(lua, &v)
        })?,
    )?;
    json_t.set(
        "encode",
        lua.create_function(|_, v: LuaValue| {
            let j = lua_to_json(&v, 0).map_err(mlua::Error::runtime)?;
            Ok(j.to_string())
        })?,
    )?;
    g.set("json", json_t)?;

    let re = lua.create_table()?;
    re.set(
        "test",
        lua.create_function(|_, (s, p): (String, String)| Ok(regex(&p)?.is_match(&s)))?,
    )?;
    re.set(
        "capture",
        lua.create_function(|_, (s, p, n): (String, String, Option<usize>)| {
            let n = n.unwrap_or(1);
            Ok(regex(&p)?
                .captures(&s)
                .and_then(|c| c.get(n))
                .map(|m| m.as_str().to_string()))
        })?,
    )?;
    re.set(
        "find_all",
        lua.create_function(|_, (s, p): (String, String)| {
            Ok(regex(&p)?
                .find_iter(&s)
                .map(|m| m.as_str().to_string())
                .collect::<Vec<_>>())
        })?,
    )?;
    g.set("re", re)?;
    Ok(lua)
}

fn regex(p: &str) -> mlua::Result<regex::Regex> {
    regex::Regex::new(p).map_err(|e| mlua::Error::runtime(format!("bad pattern: {e}")))
}

/// `parinn.events()` returns a fresh array of the record's events in
/// (writer, seq) order; null fields are absent (nil).
fn install_record(lua: &Lua, events: &[Json]) -> mlua::Result<()> {
    let source: Vec<Json> = events.to_vec();
    let parinn = lua.create_table()?;
    parinn.set(
        "events",
        lua.create_function(move |lua, ()| {
            let arr = lua.create_table_with_capacity(source.len(), 0)?;
            for (i, e) in source.iter().enumerate() {
                arr.raw_set(i + 1, json_to_lua(lua, e)?)?;
            }
            Ok(arr)
        })?,
    )?;
    lua.globals().set("parinn", parinn)
}

fn json_to_lua(lua: &Lua, v: &Json) -> mlua::Result<LuaValue> {
    Ok(match v {
        Json::Null => LuaValue::Nil,
        Json::Bool(b) => LuaValue::Boolean(*b),
        Json::Number(n) => match n.as_i64() {
            Some(i) => LuaValue::Integer(i),
            None => LuaValue::Number(n.as_f64().unwrap_or(f64::NAN)),
        },
        Json::String(s) => LuaValue::String(lua.create_string(s)?),
        Json::Array(a) => {
            let t = lua.create_table_with_capacity(a.len(), 0)?;
            for (i, x) in a.iter().enumerate() {
                t.raw_set(i + 1, json_to_lua(lua, x)?)?;
            }
            LuaValue::Table(t)
        }
        Json::Object(o) => {
            let t = lua.create_table_with_capacity(0, o.len())?;
            for (k, x) in o {
                t.raw_set(k.as_str(), json_to_lua(lua, x)?)?;
            }
            LuaValue::Table(t)
        }
    })
}

/// A table whose keys are exactly 1..n is an array (an empty table
/// too); any other table is an object with its keys as strings.
fn lua_to_json(v: &LuaValue, depth: usize) -> Result<Json, String> {
    if depth > 64 {
        return Err("value nested too deeply".into());
    }
    Ok(match v {
        LuaValue::Nil => Json::Null,
        LuaValue::Boolean(b) => Json::Bool(*b),
        LuaValue::Integer(i) => Json::Number((*i).into()),
        LuaValue::Number(f) => Number::from_f64(*f).map(Json::Number).unwrap_or(Json::Null),
        LuaValue::String(s) => Json::String(s.to_string_lossy().to_string()),
        LuaValue::Table(t) => {
            let n = t.raw_len();
            let pairs: Vec<(LuaValue, LuaValue)> = t
                .pairs::<LuaValue, LuaValue>()
                .collect::<mlua::Result<_>>()
                .map_err(|e| e.to_string())?;
            if pairs.len() == n {
                let mut a = Vec::with_capacity(n);
                for i in 1..=n {
                    let x: LuaValue = t.raw_get(i).map_err(|e| e.to_string())?;
                    a.push(lua_to_json(&x, depth + 1)?);
                }
                Json::Array(a)
            } else {
                let mut m = Map::new();
                for (k, x) in pairs {
                    let key = match k {
                        LuaValue::String(s) => s.to_string_lossy().to_string(),
                        LuaValue::Integer(i) => i.to_string(),
                        LuaValue::Number(f) => f.to_string(),
                        other => return Err(format!("unsupported key type {}", other.type_name())),
                    };
                    m.insert(key, lua_to_json(&x, depth + 1)?);
                }
                Json::Object(m)
            }
        }
        other => return Err(format!("unsupported value type {}", other.type_name())),
    })
}

// ---------------------------------------------------------------- SQL

fn run_sql(events: &[Json], code: &str) -> Result<Json, String> {
    let conn = load_sql(events).map_err(|e| format!("load record: {e}"))?;
    let start = Instant::now();
    let steps = Arc::new(AtomicU64::new(0));
    let seen = steps.clone();
    let every = 1_000;
    conn.progress_handler(
        every as i32,
        Some(move || {
            let n = seen.fetch_add(every, Ordering::Relaxed) + every;
            n > SQL_STEPS || start.elapsed() > WALL_CLOCK
        }),
    )
    .map_err(|e| e.to_string())?;
    conn.authorizer(Some(authorize)).map_err(|e| e.to_string())?;
    let mut last: Option<Vec<Vec<Json>>> = None;
    let mut batch = Batch::new(&conn, code);
    while let Some(mut stmt) = batch.next().map_err(|e| sql_error(e, &steps))? {
        let cols = stmt.column_count();
        if cols == 0 {
            stmt.raw_execute().map_err(|e| sql_error(e, &steps))?;
            continue;
        }
        let mut rows = stmt.raw_query();
        let mut out = Vec::new();
        while let Some(row) = rows.next().map_err(|e| sql_error(e, &steps))? {
            let mut r = Vec::with_capacity(cols);
            for i in 0..cols {
                r.push(sql_value(row.get_ref(i).map_err(|e| e.to_string())?));
            }
            out.push(r);
        }
        last = Some(out);
    }
    match last {
        Some(rows) => Ok(Json::Array(rows.into_iter().map(Json::Array).collect())),
        None => Err("no statement returned rows".into()),
    }
}

fn sql_error(e: rusqlite::Error, steps: &AtomicU64) -> String {
    if steps.load(Ordering::Relaxed) > SQL_STEPS {
        return "step budget exceeded".into();
    }
    match &e {
        rusqlite::Error::SqliteFailure(f, _) if f.code == rusqlite::ErrorCode::OperationInterrupted => {
            "step budget or wall-clock limit exceeded".into()
        }
        _ => e.to_string(),
    }
}

fn load_sql(events: &[Json]) -> rusqlite::Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(&format!(
        "PRAGMA hard_heap_limit = {MEMORY};
         CREATE TABLE record (writer TEXT, seq INTEGER, run TEXT, ts INTEGER,
           provenance TEXT, kind TEXT, tool TEXT, call_id TEXT, status TEXT,
           tokens INTEGER, text TEXT, args TEXT);
         CREATE VIEW events AS SELECT * FROM record;"
    ))?;
    {
        let mut ins = conn.prepare("INSERT INTO record VALUES (?,?,?,?,?,?,?,?,?,?,?,?)")?;
        for e in events {
            let vals: Vec<rusqlite::types::Value> = COLUMNS
                .iter()
                .map(|c| match &e[*c] {
                    Json::Null => rusqlite::types::Value::Null,
                    Json::Number(n) => rusqlite::types::Value::Integer(n.as_i64().unwrap_or(0)),
                    Json::String(s) => rusqlite::types::Value::Text(s.clone()),
                    other => rusqlite::types::Value::Text(other.to_string()),
                })
                .collect();
            ins.execute(rusqlite::params_from_iter(vals))?;
        }
    }
    let flags = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC;
    conn.create_scalar_function("regexp", 2, flags, |ctx| {
        let p: String = ctx.get(0)?;
        let s: Option<String> = ctx.get(1)?;
        let re = regex::Regex::new(&p).map_err(|e| rusqlite::Error::UserFunctionError(e.into()))?;
        Ok(s.map(|s| re.is_match(&s)))
    })?;
    conn.create_scalar_function("regexp_capture", 3, flags, |ctx| {
        let s: Option<String> = ctx.get(0)?;
        let p: String = ctx.get(1)?;
        let n: usize = ctx.get::<i64>(2)?.max(0) as usize;
        let re = regex::Regex::new(&p).map_err(|e| rusqlite::Error::UserFunctionError(e.into()))?;
        Ok(s.and_then(|s| re.captures(&s).and_then(|c| c.get(n)).map(|m| m.as_str().to_string())))
    })?;
    Ok(conn)
}

/// The kernel's SQL may read the record and build temporary tables and
/// views; it may not write the record, attach, run pragmas or call the
/// nondeterministic functions.
fn authorize(ctx: AuthContext<'_>) -> Authorization {
    let temp = ctx.database_name == Some("temp");
    match ctx.action {
        AuthAction::Select | AuthAction::Read { .. } | AuthAction::Recursive => Authorization::Allow,
        AuthAction::Function { function_name } => {
            let f = function_name.to_ascii_lowercase();
            if f == "random" || f == "randomblob" || f == "load_extension" {
                Authorization::Deny
            } else {
                Authorization::Allow
            }
        }
        AuthAction::CreateTempTable { .. }
        | AuthAction::CreateTempView { .. }
        | AuthAction::CreateTempIndex { .. }
        | AuthAction::DropTempTable { .. }
        | AuthAction::DropTempView { .. }
        | AuthAction::DropTempIndex { .. } => Authorization::Allow,
        AuthAction::Insert { .. } | AuthAction::Update { .. } | AuthAction::Delete { .. }
            if temp =>
        {
            Authorization::Allow
        }
        _ => Authorization::Deny,
    }
}

fn sql_value(v: ValueRef<'_>) -> Json {
    match v {
        ValueRef::Null => Json::Null,
        ValueRef::Integer(i) => Json::Number(i.into()),
        ValueRef::Real(f) => Number::from_f64(f).map(Json::Number).unwrap_or(Json::Null),
        ValueRef::Text(t) => Json::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Json::String(format!("<blob {} bytes>", b.len())),
    }
}
