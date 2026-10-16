//! What the debugger records while a game runs: `counters`, `freeze`, `profile`, `cov`, `framelog`, `copflow`,
//! `cophist` and `dma`.

use crate::control::{has_call_stack, upper};
use crate::{required, scoped, session, Run};
use dianash_module::{Answer, Call};
use emusen_dotnet::file::{read_all_bytes, write_all_bytes};
use emusen_dotnet::json::{self, Value};
use emusen_dotnet::number::parse_i32;
use emusen_dotnet::suggestion;
use emusen_dotnet::text::{equals_ignore_case, pad_left, pad_right, to_lower, trim_end};
use emusen_session_service::abi::{self, status};
use emusen_session_service::scope;
use emusen_session_service::space::{find, read};
use emusen_session_service::words::{describe, hex, in_logs, parse_hex};
use emusen_session_service::{object, quoted, Record, Refusal, Session};

/// An op's answer read as JSON.
fn record(bytes: Result<Vec<u8>, Refusal>) -> Result<Value, String> {
    let bytes = bytes?;
    json::parse(&String::from_utf8_lossy(&bytes)).map_err(|e| e.to_string())
}

fn hint_and_try(word: &str, subcommands: &[&str]) -> String {
    format!("{} Try {}", suggestion::hint(word, subcommands, 2), subcommands.join("/"))
}

pub fn counters(call: &Call, parts: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    if !session.can("counters")? {
        return Ok(Answer::fail("This core does not report access counters."));
    }
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: counters on|off|clear|top|cold|<addr> ..."));
    }
    let op = |op: u32, args: &str| record(session.counters_op(op, args));
    let header = |status: &Value| format!("{}{}:", status.text("space"), if status.flag("armed") { "" } else { " (counting stopped)" });
    match to_lower(&parts[1]).as_str() {
        "on" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: counters on <space>"));
            }
            let space = find(&session, &parts[2])?;
            session.counters_op(abi::counters::ARM, &object(&[("space", quoted(&space.name)), ("size", space.size.to_string())]))?;
            Ok(Answer::ok(format!("Counting reads/writes/executes for {} ({} bytes).", space.name, space.size)))
        }
        "off" => {
            session.counters_op(abi::counters::DISARM, "{}")?;
            Ok(Answer::ok("Access counting stopped."))
        }
        "clear" => {
            session.counters_op(abi::counters::CLEAR, "{}")?;
            Ok(Answer::ok("Access counts cleared."))
        }
        "top" => {
            let status = op(abi::counters::STATUS, "{}")?;
            if !status.flag("armed") && status.int("size") == 0 {
                return Ok(Answer::ok("Not counting - run `counters on <space>` first."));
            }
            let sort = match parts.get(2).map(|s| to_lower(s)).as_deref() {
                None | Some("r" | "read" | "reads") => "reads",
                Some("w" | "write" | "writes") => "writes",
                Some("x" | "exec" | "executes") => "executes",
                Some("u" | "uninit" | "uninitialized") => "uninitialized",
                Some(_) => return Ok(Answer::fail(format!("Unknown sort '{}'. Try r/w/x/u.", parts[2]))),
            };
            let limit = if parts.len() > 3 { parse_hex(&parts[3])? } else { 20 };
            let rows = op(abi::counters::HOTTEST, &object(&[("sort", quoted(sort)), ("limit", limit.to_string())]))?;
            if rows.items().is_empty() {
                return Ok(Answer::ok("Nothing has been accessed yet."));
            }
            let lines: Vec<String> = rows
                .items()
                .iter()
                .map(|r| {
                    format!(
                        "  {}  r={} w={} x={} uninit={}",
                        describe(&session, r.address("address")),
                        pad_right(&r.int("reads").to_string(), 10),
                        pad_right(&r.int("writes").to_string(), 10),
                        pad_right(&r.int("executes").to_string(), 10),
                        r.int("uninitialized_reads")
                    )
                })
                .collect();
            Ok(Answer::ok(format!("{}\n{}", header(&status), lines.join("\n"))))
        }
        "cold" => {
            if parts.len() < 4 {
                return Ok(Answer::ok("Usage: counters cold <addr> <len>"));
            }
            let address = parse_hex(&parts[2])?;
            let length = parse_hex(&parts[3])?;
            let cold = op(abi::counters::UNTOUCHED, &object(&[("address", (address as u32).to_string()), ("length", length.to_string()), ("limit", "256".to_string())]))?;
            let space = op(abi::counters::STATUS, "{}")?.text("space");
            let end = address.wrapping_add(length).wrapping_sub(1);
            if cold.items().is_empty() {
                return Ok(Answer::ok(format!("Every byte in {space} 0x{address:X}-0x{end:X} was touched.")));
            }
            let rows: Vec<String> = cold.items().iter().filter_map(emusen_session_service::number).map(|a| format!("  0x{:X}", a as u32 as i32)).collect();
            Ok(Answer::ok(format!("{} untouched address(es) in {space} 0x{address:X}-0x{end:X}:\n{}", rows.len(), rows.join("\n"))))
        }
        _ => {
            let Ok(address) = parse_hex(&parts[1]) else {
                return Ok(Answer::ok(format!("Unknown 'counters' subcommand '{}'.{} or an address.", parts[1], hint_and_try(&parts[1], &["on", "off", "clear", "top", "cold"]))));
            };
            let length = if parts.len() > 2 { parse_hex(&parts[2])? } else { 1 };
            let totals = op(abi::counters::TOTALS, &object(&[("address", (address as u32).to_string()), ("length", length.to_string())]))?;
            let status = op(abi::counters::STATUS, "{}")?;
            Ok(Answer::ok(format!(
                "{}\n  {} 0x{address:X}-0x{:X}\n  reads {}, writes {}, executes {}\n  uninitialized reads {}, bytes touched {}/{length}",
                header(&status),
                status.text("space"),
                address.wrapping_add(length).wrapping_sub(1),
                totals.int("reads"),
                totals.int("writes"),
                totals.int("executes"),
                totals.int("uninitialized_reads"),
                totals.int("touched_bytes")
            )))
        }
    }
}

pub fn freeze(call: &Call, parts: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    if !session.can("freezes")? {
        return Ok(Answer::fail("This core does not support frozen addresses."));
    }
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: freeze add|list|remove|clear ..."));
    }
    match to_lower(&parts[1]).as_str() {
        "add" => {
            if parts.len() < 4 {
                return Ok(Answer::ok("Usage: freeze add <space> <addr> [<value>]"));
            }
            let space = find(&session, &parts[2])?;
            if !space.writable {
                return Ok(Answer::fail(format!("{} is not writable, so nothing can write to it to be undone.", space.name)));
            }
            let address = parse_hex(&parts[3])?;
            if address < 0 || address >= space.size {
                return Ok(Answer::fail(format!("0x{address:X} is outside {} (0x0-0x{:X}).", space.name, space.size.wrapping_sub(1))));
            }
            let value = if parts.len() > 4 { parse_hex(&parts[4])? as u8 } else { read(&session, &space, address, 1)?.first().copied().unwrap_or(0) };
            let id = session.freeze_add(&space.name, address as u32, u32::from(value))?;
            Ok(Answer::ok(format!("Frozen #{id}: {} 0x{address:X} = 0x{value:02X}.", space.name)))
        }
        "list" => {
            let all = session.freeze_list()?;
            if all.items().is_empty() {
                return Ok(Answer::ok("No frozen addresses."));
            }
            let rows: Vec<String> = all
                .items()
                .iter()
                .map(|f| format!("  #{}: {} 0x{:X} = 0x{:02X} ({} write(s) undone)", f.int("id"), f.text("space"), f.address("address"), f.int("value"), f.int("blocked")))
                .collect();
            Ok(Answer::ok(rows.join("\n")))
        }
        "remove" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: freeze remove <id>"));
            }
            let id = parse_hex(&parts[2])?;
            Ok(match session.freeze_remove(id as u32) {
                Ok(()) => Answer::ok(format!("Frozen address #{id} released.")),
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => Answer::fail(format!("No frozen address #{id}.")),
                Err(refusal) => return Err(refusal.words),
            })
        }
        "clear" => {
            let count = session.freeze_list()?.items().len();
            session.freeze_clear()?;
            Ok(Answer::ok(format!("{count} frozen address(es) released.")))
        }
        _ => Ok(Answer::ok(format!("Unknown 'freeze' subcommand '{}'.{}.", parts[1], hint_and_try(&parts[1], &["add", "list", "remove", "clear"])))),
    }
}

/// A `profile_op` status: whether it profiles, and how many instructions it charged.
fn profile_status(session: &Session, processor: u32) -> Result<Value, String> {
    record(session.profile_op(processor, abi::profile::STATUS, "{}"))
}

pub fn profile(call: &Call, parts: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    let cpus = scope::cpus(&session)?;
    let (cpu, at) = scoped(&cpus, parts, 1);
    if cpu.is_none() && parts.len() > 1 && scope::is_scope_word(&cpus, &parts[1]) {
        return Ok(Answer::fail("This cartridge has no coprocessor to profile."));
    }
    if !has_call_stack(&session, cpu)? {
        return Ok(Answer::fail(match cpu {
            Some(cpu) if !scope::is_main(&cpus, Some(cpu)) => format!("{} does not report a call stack, so it cannot be profiled - see `man cpus`.", cpu.name),
            _ => "This core does not report a call stack, so it cannot be profiled.".to_string(),
        }));
    }
    if parts.len() <= at {
        return Ok(Answer::ok("Usage: profile on|off|clear|top ..."));
    }
    let processor = scope::index(&session, cpu);
    match to_lower(&parts[at]).as_str() {
        "on" => {
            session.profile_op(processor, abi::profile::ARM, "{}")?;
            Ok(Answer::ok("Profiling instructions per routine."))
        }
        "off" => {
            session.profile_op(processor, abi::profile::DISARM, "{}")?;
            Ok(Answer::ok(format!("Profiling stopped after {} instruction(s).", profile_status(&session, processor)?.int("instructions"))))
        }
        "clear" => {
            session.profile_op(processor, abi::profile::CLEAR, "{}")?;
            Ok(Answer::ok("Profile cleared."))
        }
        "top" => {
            let limit = if parts.len() > at + 1 { parse_hex(&parts[at + 1])? } else { 20 };
            let rows = record(session.profile_op(processor, abi::profile::HOTTEST, &object(&[("limit", limit.to_string())])))?;
            let status = profile_status(&session, processor)?;
            let profiling = status.flag("profiling");
            if rows.items().is_empty() {
                return Ok(Answer::ok(if profiling { "Nothing profiled yet." } else { "Not profiling - run `profile on` first." }));
            }
            let lines: Vec<String> = rows
                .items()
                .iter()
                .map(|r| {
                    let address = r.address("address");
                    let place = if address == 0 { "(outside any recorded call)".to_string() } else { describe(&session, address) };
                    format!("  {:6.2}%  {} {} {place}", r.float("percent"), pad_right(&r.int("instructions").to_string(), 12), pad_right(&r.int("calls").to_string(), 8))
                })
                .collect();
            Ok(Answer::ok(format!(
                "{} instruction(s) profiled{}:\n{}",
                status.int("instructions"),
                if profiling { "" } else { ", profiling stopped" },
                lines.join("\n")
            )))
        }
        _ => Ok(Answer::ok(format!("Unknown 'profile' subcommand '{}'.{}.", parts[at], hint_and_try(&parts[at], &["on", "off", "clear", "top"])))),
    }
}

/// A call-frame kind's name as `CallFrameKind.ToString().ToLowerInvariant()` gives it.
fn frame_kind(kind: i64) -> &'static str {
    match kind {
        0 => "call",
        1 => "nmi",
        2 => "irq",
        3 => "brk",
        _ => "cop",
    }
}

pub fn cov(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: cov [<cpu>] on|off|clear|mark|new|funcs|save|load|<addr> [<len>]"));
    }
    let cpus = scope::cpus(&session)?;
    let (cpu, mut at) = scoped(&cpus, parts, 1);
    if cpu.is_none() && scope::is_scope_word(&cpus, &parts[1]) {
        return Ok(Answer::ok("This cartridge has no coprocessor to record coverage for."));
    }
    if parts.len() <= at {
        return Ok(Answer::ok("Usage: cov <cpu> on|off|clear|<addr> [<len>]"));
    }
    let coprocessor = cpu.is_some() && !scope::is_main(&cpus, cpu);
    let has = match cpu {
        Some(cpu) => cpu.coverage,
        None => session.can("coverage")?,
    };
    if !has {
        return Ok(Answer::ok(if coprocessor {
            format!("{} records no execution coverage on this core.", cpu.unwrap().name)
        } else {
            format!("{} target records no execution coverage.", session.core_name()?)
        }));
    }
    let processor = scope::index(&session, cpu);
    let scope_name = if coprocessor { format!("{} coverage", upper(&cpu.unwrap().name)) } else { "Coverage".to_string() };
    let op = |op: u32, args: &str| session.coverage_op(processor, op, args.as_bytes());
    let status = || record(op(abi::coverage::STATUS, "{}"));
    let sub = to_lower(&parts[at]);
    match sub.as_str() {
        "on" => {
            op(abi::coverage::ARM, "{}")?;
            return Ok(Answer::ok(format!("{scope_name} recording on.")));
        }
        "off" => {
            op(abi::coverage::DISARM, "{}")?;
            return Ok(Answer::ok(format!("{scope_name} recording off, {} instructions recorded.", status()?.int("instructions"))));
        }
        "clear" => {
            op(abi::coverage::CLEAR, "{}")?;
            return Ok(Answer::ok(format!("{scope_name} cleared.")));
        }
        "mark" => {
            op(abi::coverage::MARK, "{}")?;
            return Ok(Answer::ok(format!(
                "{scope_name} marked at {} instructions - `cov new <addr>` now reports only what runs from here.",
                status()?.int("instructions")
            )));
        }
        "funcs" => {
            let limit = if parts.len() > at + 1 { parse_hex(&parts[at + 1])? } else { 32 };
            let funcs = record(op(abi::coverage::ENTRY_POINTS, &object(&[("limit", limit.to_string())])))?;
            let status = status()?;
            if funcs.items().is_empty() {
                return Ok(Answer::ok(if status.flag("armed") {
                    "No routines discovered yet - nothing has been called since recording started."
                } else {
                    "No routines discovered (recording is OFF - `cov on` first)."
                }));
            }
            let rows: Vec<String> = funcs
                .items()
                .iter()
                .map(|f| format!("  {}  {}  entered {}x", describe(&session, f.address("address")), pad_right(frame_kind(f.int("kind")), 4), f.int("entries")))
                .collect();
            return Ok(Answer::ok(format!("{} routine(s) discovered:\n{}", status.int("entry_points"), rows.join("\n"))));
        }
        "save" | "load" => {
            if parts.len() <= at + 1 {
                return Ok(Answer::ok(format!("Usage: cov {sub} <file>")));
            }
            let path = match in_logs(|p| call.resolve(p), &session.core_name()?, &parts[at + 1], "cov") {
                Ok(path) => path,
                Err(refused) => return Ok(Answer::fail(refused)),
            };
            if let Some(parent) = std::path::Path::new(&path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if sub == "save" {
                let map = op(abi::coverage::EXPORT, "{}")?;
                write_all_bytes(&path, &map)?;
                return Ok(Answer::ok(format!("{scope_name} map saved to {path} ({} routine(s)).", status()?.int("entry_points"))));
            }
            if !std::path::Path::new(&path).is_file() {
                return Ok(Answer::fail(format!("No coverage map at {path}.")));
            }
            let map = read_all_bytes(&path)?;
            return Ok(match session.coverage_op(processor, abi::coverage::IMPORT, &map) {
                Ok(_) => Answer::ok(format!("{scope_name} map merged from {path} ({} routine(s) known now).", status()?.int("entry_points"))),
                Err(refusal) if refusal.is(status::BAD_ARGUMENT) => Answer::fail(format!("{path} is not a coverage map this build can read.")),
                Err(refusal) => return Err(refusal.words),
            });
        }
        _ => {}
    }
    let new_only = sub == "new";
    if new_only {
        if !status()?.flag("has_mark") {
            return Ok(Answer::fail("Nothing marked yet - run `cov mark`, do the thing, then ask again."));
        }
        at += 1;
        if parts.len() <= at {
            return Ok(Answer::ok("Usage: cov new <addr> [<len>]"));
        }
    }
    let address = parse_hex(&parts[at])?;
    let length = if parts.len() > at + 1 { parse_hex(&parts[at + 1])? } else { 16 };
    if length <= 0 {
        return Ok(Answer::ok("Length must be positive."));
    }
    let range = object(&[("address", (address as u32).to_string()), ("length", length.to_string()), ("limit", "32".to_string())]);
    let query = record(op(abi::coverage::QUERY, &range))?;
    let hits = if new_only { query.int("new_since_mark") } else { query.int("executed") };
    let what = if new_only { "bytes executed since the mark" } else { "bytes executed" };
    let armed = status()?.flag("armed");
    let mut lines = vec![format!(
        "{scope_name} ${}-${}: {hits}/{length} {what}{}",
        hex(address, 6),
        hex(address.wrapping_add(length).wrapping_sub(1), 6),
        if armed { "" } else { " (recording is OFF)" }
    )];
    if hits == 0 {
        lines.push(if new_only { "  Nothing new here since the mark." } else { "  Never reached." }.to_string());
    } else {
        let executed: Vec<i64> = if new_only {
            record(op(abi::coverage::NEW_SINCE_MARK, &range))?.items().iter().filter_map(emusen_session_service::number).collect()
        } else {
            query.list("addresses").iter().filter_map(emusen_session_service::number).collect()
        };
        let shown: Vec<String> = executed.iter().map(|&a| format!("${}", hex(a as u32 as i32, 6))).collect();
        lines.push(format!("  {}{}", shown.join(" "), if hits > executed.len() as i64 { " ..." } else { "" }));
    }
    Ok(Answer::ok(lines.join("\n")))
}

pub fn framelog(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: framelog add|list|show|clear|remove ..."));
    }
    let sub = to_lower(&parts[1]);
    match sub.as_str() {
        "add" => {
            if parts.len() < 4 {
                return Ok(Answer::ok("Usage: framelog add <space> <addr> [<width>]"));
            }
            find(&session, &parts[2])?;
            let addr = parse_hex(&parts[3])?;
            let width = if parts.len() >= 5 { parse_hex(&parts[4])? } else { 1 };
            if !matches!(width, 1 | 2 | 4) {
                return Ok(Answer::ok("width must be 1, 2, or 4"));
            }
            let id = session.framelog_add(&parts[2], addr as u32, width as u32)?;
            Ok(Answer::ok(format!("Frame log #{id} added: {} 0x{addr:X} (width {width})", parts[2])))
        }
        "list" => {
            let list = session.framelog_list()?;
            if list.items().is_empty() {
                return Ok(Answer::ok("No active frame logs."));
            }
            let rows: Vec<String> = list.items().iter().map(|e| format!("  #{}: {} 0x{:X} (width {})", e.int("id"), e.text("space"), e.address("address"), e.int("width"))).collect();
            Ok(Answer::ok(rows.join("\n")))
        }
        "show" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: framelog show <id> [<count>]"));
            }
            let id = parse_hex(&parts[2])?;
            let count = if parts.len() >= 4 { parse_hex(&parts[3])? } else { 20 };
            let samples = match session.framelog_samples(id as u32, count.max(0) as u32) {
                Ok(samples) => samples.items().to_vec(),
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => Vec::new(),
                Err(refusal) => return Err(refusal.words),
            };
            if samples.is_empty() {
                return Ok(Answer::ok(format!("No samples recorded for frame log #{id} (or it doesn't exist).")));
            }
            let rows: Vec<String> = samples.iter().map(|s| format!("  frame {}: 0x{:X}", s.int("frame"), s.int("value"))).collect();
            Ok(Answer::ok(rows.join("\n")))
        }
        "clear" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: framelog clear <id>"));
            }
            match session.framelog_clear(parse_hex(&parts[2])? as u32) {
                Ok(()) => {}
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => {}
                Err(refusal) => return Err(refusal.words),
            }
            Ok(Answer::ok(format!("Cleared samples for frame log #{}.", parts[2])))
        }
        "remove" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: framelog remove <id>"));
            }
            Ok(Answer::ok(match session.framelog_remove(parse_hex(&parts[2])? as u32) {
                Ok(()) => format!("Frame log #{} removed.", parts[2]),
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => format!("No frame log #{} found.", parts[2]),
                Err(refusal) => return Err(refusal.words),
            }))
        }
        _ => {
            let subcommands = ["add", "list", "show", "clear", "remove"];
            Ok(Answer::ok(format!("Unknown 'framelog' subcommand '{sub}'.{}.", hint_and_try(&sub, &subcommands))))
        }
    }
}

fn flow_nothing(status: &Value) -> Answer {
    Answer::ok(if status.flag("armed") {
        "Nothing logged yet - the cartridge has not touched its coprocessor registers since `copflow on`."
    } else {
        "Coprocessor register flow logging is off. Turn it on with `copflow on`."
    })
}

pub fn copflow(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: copflow on|off|clear|tail|stats|poll"));
    }
    if !session.can("register_flow")? {
        return Ok(Answer::ok(format!("{} exposes no coprocessor register window to watch.", session.core_name()?)));
    }
    let op = |op: u32, args: &str| record(session.register_flow_op(op, args));
    match to_lower(&parts[1]).as_str() {
        "on" => {
            let size = if parts.len() > 2 { parse_hex(&parts[2])? } else { 0x1000 };
            let window = op(abi::flow::STATUS, "{}")?.text("window");
            let window = if window.is_empty() { "CoprocessorRegisters".to_string() } else { window };
            session.register_flow_op(abi::flow::ARM, &object(&[("window", quoted(&window)), ("size", size.to_string())]))?;
            Ok(Answer::ok(format!("Coprocessor register flow logging on ({size} registers, {} retained).", op(abi::flow::STATUS, "{}")?.int("capacity"))))
        }
        "off" => {
            session.register_flow_op(abi::flow::DISARM, "{}")?;
            Ok(Answer::ok("Coprocessor register flow logging off."))
        }
        "clear" => {
            session.register_flow_op(abi::flow::CLEAR, "{}")?;
            Ok(Answer::ok("Coprocessor register flow log cleared."))
        }
        "tail" => {
            let count = if parts.len() > 2 { parse_hex(&parts[2])? } else { 20 };
            let entries = op(abi::flow::TAIL, &object(&[("count", count.max(1).to_string())]))?;
            let status = op(abi::flow::STATUS, "{}")?;
            if entries.items().is_empty() {
                return Ok(flow_nothing(&status));
            }
            let mut out = format!("Last {} of {} access(es):\n       seq  frame  dir  register  value  by\n", entries.items().len(), status.int("accesses"));
            for e in entries.items() {
                out += &format!(
                    "  {}  {}  {}  ${:04X}     0x{:02X}   {}\n",
                    pad_left(&e.int("sequence").to_string(), 8),
                    pad_left(&e.int("frame").to_string(), 5),
                    pad_left(if e.flag("write") { "W " } else { " R" }, 3),
                    e.int("register") as i32,
                    e.int("value"),
                    e.text("actor")
                );
            }
            Ok(Answer::ok(trim_end(&out)))
        }
        "stats" => {
            let stats = op(abi::flow::STATS, "{}")?;
            let status = op(abi::flow::STATUS, "{}")?;
            if stats.items().is_empty() {
                return Ok(flow_nothing(&status));
            }
            let mut out = format!("{} register(s) touched, {} access(es) total:\n  register    reads   writes  distinct  last\n", stats.items().len(), status.int("accesses"));
            for s in stats.items() {
                out += &format!(
                    "  ${:04X}  {}  {}  {}  0x{:02X}\n",
                    s.int("register") as i32,
                    pad_left(&s.int("reads").to_string(), 9),
                    pad_left(&s.int("writes").to_string(), 7),
                    pad_left(&s.int("distinct_values").to_string(), 8),
                    s.int("last_value")
                );
            }
            Ok(Answer::ok(trim_end(&out)))
        }
        "poll" => {
            let status = op(abi::flow::STATUS, "{}")?;
            if status.int("accesses") == 0 {
                return Ok(flow_nothing(&status));
            }
            let longest = status.field("longest_poll").cloned().unwrap_or(Value::Null);
            let current = status.field("current_poll").cloned().unwrap_or(Value::Null);
            let mut out = String::new();
            if longest.int("register") < 0 {
                out += "No unchanging read run recorded - every read saw a value change or was followed by a write.\n";
            } else {
                out += &format!(
                    "Longest unchanging read run: ${:04X} read {} time(s) in a row, always 0x{:02X}.\n",
                    longest.int("register") as i32,
                    longest.int("run"),
                    longest.int("value")
                );
            }
            if current.int("register") >= 0 && current.int("run") > 1 {
                out += &format!("Right now: ${:04X} has read the same value {} time(s) with no intervening write.\n", current.int("register") as i32, current.int("run"));
            }
            out += "A long run means the CPU is spinning on a status bit the chip never changed.";
            Ok(Answer::ok(out))
        }
        _ => Ok(Answer::ok(format!("Unknown copflow subcommand '{}'. Try on|off|clear|tail|stats|poll.", parts[1]))),
    }
}

pub fn cophist(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    session.info()?;
    if !session.can("coprocessor_history")? {
        return Ok(Answer::ok("This core does not keep coprocessor register history."));
    }
    let history = session.coprocessor_history(i32::MAX as u32)?;
    let frames = history.list("frames");
    if frames.last().is_none_or(|f| f.items().is_empty()) {
        return Ok(Answer::ok("No coprocessor present on this cartridge."));
    }
    let wanted = parts.get(1).filter(|p| parse_i32(p).is_none());
    let count = parts.iter().skip(1).find_map(|p| parse_i32(p)).map_or(16, |c| c.max(1));
    let shown = &frames[frames.len().saturating_sub(count as usize)..];
    let first_index = history.int("refreshes") - shown.len() as i64 + 1;
    let mut out = format!(
        "Coprocessor history: {} of {} retained (capacity {}),\nunchanged for {} refresh(es).\n",
        shown.len(),
        frames.len(),
        history.int("capacity"),
        history.int("unchanged_for")
    );
    let last = shown[shown.len() - 1].items();
    let names: Vec<String> = last.iter().map(|r| r.text("name")).filter(|n| wanted.is_none_or(|w| equals_ignore_case(n, w))).collect();
    if names.is_empty() {
        return Ok(Answer::ok(format!("No coprocessor register named '{}'.", wanted.map(String::as_str).unwrap_or(""))));
    }
    out += &format!("  refresh  {}\n", names.iter().map(|n| pad_left(n, 6)).collect::<Vec<_>>().join("  "));
    for (i, frame) in shown.iter().enumerate() {
        let cells: Vec<String> = names
            .iter()
            .map(|n| match frame.items().iter().find(|r| r.text("name") == *n) {
                Some(r) => {
                    let digits = (r.int("bits") / 4).max(1) as usize;
                    let value = r.field("value").and_then(|v| if let Value::Number(t) = v { t.parse::<u64>().ok() } else { None }).unwrap_or(0);
                    pad_left(&format!("{value:0digits$X}"), 6)
                }
                None => pad_left("", 6),
            })
            .collect();
        out += &format!("  {}  {}\n", pad_left(&(first_index + i as i64).to_string(), 7), cells.join("  "));
    }
    Ok(Answer::ok(trim_end(&out)))
}

/// The core's own name for a B-bus register, or its number.
fn destination(name: Option<String>, register: i64) -> String {
    name.unwrap_or_else(|| format!("reg 0x{:02X}", register as u8))
}

fn dma_nothing(status: &Value) -> Answer {
    Answer::ok(if status.flag("armed") {
        "Nothing logged yet - no transfer has run since `dma log on`."
    } else {
        "DMA transfer logging is off. Turn it on with `dma log on`."
    })
}

pub fn dma(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    session.info()?;
    if parts.len() < 2 {
        let channels = session.dma_channels()?;
        if channels.items().is_empty() {
            return Ok(Answer::fail(format!("{} publishes no DMA channels.", session.core_name()?)));
        }
        let mut out = String::from("  ch  hdma  dir   mode  destination       source     length  hdma table\n");
        for c in channels.items() {
            let control = c.int("control");
            let hdma = if c.flag("hdma") { if c.flag("hdma_active") { "on " } else { "arm" } } else { "-  " };
            let table = if c.flag("hdma") {
                format!("${} line {:02X}", hex(c.int("table") as i32, 6), c.int("line_counter") as u8)
                    + &if control & 0x40 != 0 { format!(" -> ${}", hex(c.int("indirect") as i32, 6)) } else { String::new() }
            } else {
                "-".to_string()
            };
            out += &format!(
                "  {}   {hdma}   {}  {}  {}  ${}  {}  {table}\n",
                c.int("index"),
                if control & 0x80 != 0 { "B->A" } else { "A->B" },
                pad_left(&(control & 0x07).to_string(), 4),
                pad_right(&destination(c.optional_text("destination_name"), c.int("destination")), 16),
                hex(c.address("source"), 6),
                pad_left(&c.int("size").to_string(), 7)
            );
        }
        out += "  (mode is the B-bus write pattern; dir B->A reads the register instead)";
        return Ok(Answer::ok(out));
    }
    match to_lower(&parts[1]).as_str() {
        "log" => {
            if !session.can("dma_log")? {
                return Ok(Answer::fail(format!("{} records no DMA history.", session.core_name()?)));
            }
            let op = |op: u32, args: &str| record(session.dma_log_op(op, args));
            let arg = parts.get(2).map(|a| to_lower(a)).unwrap_or_default();
            match arg.as_str() {
                "on" => {
                    let capacity = if parts.len() > 3 { parse_hex(&parts[3])? } else { 4096 };
                    session.dma_log_op(abi::dma_log::ARM, &object(&[("capacity", capacity.to_string())]))?;
                    return Ok(Answer::ok(format!("DMA transfer logging on ({} retained).", op(abi::dma_log::STATUS, "{}")?.int("capacity"))));
                }
                "off" => {
                    session.dma_log_op(abi::dma_log::DISARM, "{}")?;
                    return Ok(Answer::ok("DMA transfer logging off."));
                }
                "clear" => {
                    session.dma_log_op(abi::dma_log::CLEAR, "{}")?;
                    return Ok(Answer::ok("DMA transfer log cleared."));
                }
                _ => {}
            }
            let count = if arg.is_empty() { 20 } else { parse_hex(&arg)? };
            let entries = op(abi::dma_log::TAIL, &object(&[("count", count.max(1).to_string())]))?;
            let status = op(abi::dma_log::STATUS, "{}")?;
            if entries.items().is_empty() {
                return Ok(dma_nothing(&status));
            }
            let mut out = format!("Last {} of {} transfer(s):\n       seq  frame  line  ch  kind  destination       source     bytes\n", entries.items().len(), status.int("transfers"));
            for e in entries.items() {
                out += &format!(
                    "  {}  {}  {}  {}   {}  {}  ${}  {}\n",
                    pad_left(&e.int("sequence").to_string(), 8),
                    pad_left(&e.int("frame").to_string(), 5),
                    pad_left(&e.int("scanline").to_string(), 4),
                    e.int("channel"),
                    if e.text("kind") == "hdma" { "hdma" } else { "gen " },
                    pad_right(&destination(e.optional_text("destination_name"), e.int("destination")), 16),
                    hex(e.address("source"), 6),
                    pad_left(&e.int("length").to_string(), 6)
                );
            }
            Ok(Answer::ok(trim_end(&out)))
        }
        "stats" => {
            if !session.can("dma_log")? {
                return Ok(Answer::fail(format!("{} records no DMA history.", session.core_name()?)));
            }
            let op = |op: u32| record(session.dma_log_op(op, "{}"));
            let channels = op(abi::dma_log::CHANNELS)?;
            let status = op(abi::dma_log::STATUS)?;
            if channels.items().is_empty() {
                return Ok(dma_nothing(&status));
            }
            let frames = status.int("last_frame") - status.int("first_frame") + 1;
            let bytes = status.int("bytes");
            let mut out = format!(
                "{} transfer(s), {bytes} byte(s) over {frames} frame(s){}\n",
                status.int("transfers"),
                if frames > 0 { format!(" - {} bytes/frame.", bytes / frames) } else { ".".to_string() }
            );
            out += "  ch   general    bytes       hdma    bytes\n";
            for c in channels.items() {
                out += &format!(
                    "  {}  {}  {}  {}  {}\n",
                    c.int("channel"),
                    pad_left(&c.int("general_transfers").to_string(), 8),
                    pad_left(&c.int("general_bytes").to_string(), 7),
                    pad_left(&c.int("hdma_transfers").to_string(), 9),
                    pad_left(&c.int("hdma_bytes").to_string(), 7)
                );
            }
            out += "  destination       transfers    bytes  channels\n";
            for d in op(abi::dma_log::DESTINATIONS)?.items() {
                let mask = d.int("channel_mask");
                let list: Vec<String> = (0..16).filter(|i| mask & (1 << i) != 0).map(|i| i.to_string()).collect();
                out += &format!(
                    "  {}  {}  {}  {}\n",
                    pad_right(&destination(d.optional_text("destination_name"), d.int("destination")), 16),
                    pad_left(&d.int("transfers").to_string(), 9),
                    pad_left(&d.int("bytes").to_string(), 7),
                    if list.is_empty() { "-".to_string() } else { list.join(",") }
                );
            }
            Ok(Answer::ok(trim_end(&out)))
        }
        _ => Ok(Answer::ok(format!("Unknown dma subcommand '{}'. Try log|stats, or `dma` alone for the channel table.", parts[1]))),
    }
}
