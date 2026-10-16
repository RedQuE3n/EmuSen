//! Breakpoints, watches and run control: `bp`, `watch`, `step`, `runto`, `bt`, `resume`, `trace` and `log`.

use crate::{required, scoped, session, usage, Run};
use dianash_module::{Answer, Call};
use emusen_dotnet::number::{int_parse, long_parse};
use emusen_dotnet::suggestion;
use emusen_dotnet::text::{equals_ignore_case, pad_left, pad_right, simple_upper, to_lower};
use emusen_session_service::abi::{self, status};
use emusen_session_service::scope::{self, Cpu};
use emusen_session_service::space::find;
use emusen_session_service::words::{describe, hex, parse_hex, parse_range};
use emusen_session_service::{object, quoted, Record, Session};

/// `ToUpperInvariant`.
pub(crate) fn upper(text: &str) -> String {
    text.chars().map(simple_upper).collect()
}

/// A watch kind's name as `WatchKind.ToString().ToLowerInvariant()` gives it.
pub(crate) fn watch_kind(kind: i64) -> &'static str {
    match kind {
        0 => "write",
        1 => "read",
        _ => "both",
    }
}

/// Whether the session's own call stack exists, or the named processor's.
pub(crate) fn has_call_stack(session: &Session, cpu: Option<&Cpu>) -> Result<bool, String> {
    Ok(match cpu {
        Some(cpu) => cpu.call_stack,
        None => session.can("call_stack")?,
    })
}

/// A 24-bit CPU address, or a range of them.
fn range(start: i32, end: i32) -> String {
    if start == end { format!("${}", hex(start, 6)) } else { format!("${}-${}", hex(start, 6), hex(end, 6)) }
}

/// An offset into a memory space, or a range of them.
fn data_range(start: i32, end: i32) -> String {
    if start == end { format!("0x{start:X}") } else { format!("0x{start:X}-0x{end:X}") }
}

fn is_clause(word: &str) -> bool {
    equals_ignore_case(word, "if") || equals_ignore_case(word, "log")
}

/// The trailing `if <cond>` and `log <expr>` clauses, in either order.
fn clauses(parts: &[String], from: usize) -> (Option<String>, Option<String>) {
    let (mut condition, mut log) = (None, None);
    let mut i = from;
    while i < parts.len() {
        if !is_clause(&parts[i]) {
            i += 1;
            continue;
        }
        let is_if = equals_ignore_case(&parts[i], "if");
        let start = i + 1;
        let mut end = start;
        while end < parts.len() && !is_clause(&parts[end]) {
            end += 1;
        }
        if end > start {
            let text = parts[start..end].join(" ");
            if is_if {
                condition = Some(text);
            } else {
                log = Some(text);
            }
        }
        i = end;
    }
    (condition, log)
}

fn clause_text(condition: Option<&str>, log: Option<&str>) -> String {
    condition.map_or(String::new(), |c| format!(" if {c}")) + &log.map_or(String::new(), |l| format!(" log {l}"))
}

fn optional(text: Option<&str>) -> String {
    text.map_or("null".to_string(), quoted)
}

pub fn bp(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: bp [<cpu>] add|list|remove ..."));
    }
    let cpus = scope::cpus(&session)?;
    let (cpu, at) = scoped(&cpus, parts, 1);
    if cpu.is_none() && scope::is_scope_word(&cpus, &parts[1]) {
        return Ok(Answer::ok("This cartridge has no coprocessor CPU to break on."));
    }
    if parts.len() <= at {
        return Ok(Answer::ok("Usage: bp <cpu> add|list|remove ..."));
    }
    if let Some(cpu) = cpu.filter(|c| !c.can_halt) {
        return Ok(Answer::ok(format!("{} cannot be halted by this core, so it takes no breakpoints.", cpu.name)));
    }
    let processor = scope::index(&session, cpu);
    let sub = to_lower(&parts[at]);
    let arg = parts.get(at + 1).map(String::as_str).unwrap_or("");
    let coprocessor = cpu.is_some() && !scope::is_main(&cpus, cpu);
    let scope_name = if coprocessor { format!("{} breakpoint", upper(&cpu.unwrap().name)) } else { "Breakpoint".to_string() };
    let other = if coprocessor { format!("{} ", cpu.unwrap().name) } else { String::new() };
    match sub.as_str() {
        "add" => {
            if arg.is_empty() {
                return Ok(Answer::ok("Usage: bp add <addr>[-<end>] [if <expr>]"));
            }
            let (start, end) = parse_range(arg)?;
            let (condition, log) = clauses(parts, at + 2);
            let spec = object(&[("address", (start as u32).to_string()), ("end", (end as u32).to_string()), ("condition", optional(condition.as_deref())), ("log", optional(log.as_deref()))]);
            let id = session.bp_add(processor, &spec)?;
            Ok(Answer::ok(format!("{scope_name} #{id} {} {}{}.", if log.is_none() { "added at" } else { "logging at" }, range(start, end), clause_text(condition.as_deref(), log.as_deref()))))
        }
        "read" | "write" => {
            if parts.len() < at + 3 {
                return Ok(Answer::ok(format!("Usage: bp {sub} <space> <addr>[-<end>] [<value>] [changed] [if <expr>]")));
            }
            let space = find(&session, &parts[at + 1])?;
            let (start, end) = parse_range(&parts[at + 2])?;
            let mut value = -1;
            let mut changed = false;
            let mut i = at + 3;
            while i < parts.len() && !is_clause(&parts[i]) {
                if equals_ignore_case(&parts[i], "changed") {
                    changed = true;
                } else {
                    value = parse_hex(&parts[i])?;
                }
                i += 1;
            }
            let (condition, log) = clauses(parts, i);
            if changed && sub == "read" {
                return Ok(Answer::ok("'changed' only applies to writes - a read never changes the value."));
            }
            let spec = object(&[
                ("space", quoted(&space.name)),
                ("address", (start as u32).to_string()),
                ("end", (end as u32).to_string()),
                ("value", value.to_string()),
                ("on_read", (sub == "read").to_string()),
                ("changed_only", changed.to_string()),
                ("condition", optional(condition.as_deref())),
                ("log", optional(log.as_deref())),
            ]);
            let id = session.bp_add_data(processor, &spec)?;
            let value_text = if value < 0 { String::new() } else { format!(" = 0x{value:02X}") };
            let changed_text = if changed { " (only when it changes)" } else { "" };
            let verb = if log.is_none() { "added on" } else { "logging on" };
            Ok(Answer::ok(format!(
                "{scope_name} #{id} {verb} {sub}s of {} {}{value_text}{changed_text}{}.",
                space.name,
                data_range(start, end),
                clause_text(condition.as_deref(), log.as_deref())
            )))
        }
        "uninit" => {
            if arg.is_empty() {
                return Ok(Answer::ok("Usage: bp uninit <space>"));
            }
            let space = find(&session, arg)?;
            match session.bp_uninitialized(processor, Some(&space.name), space.size.max(0) as u32) {
                Ok(()) => Ok(Answer::ok(format!("Halting on the first read of any never-written byte of {}.", space.name))),
                Err(refusal) if refusal.is(status::BAD_ARGUMENT) => Ok(Answer::fail(format!("{} reports no size, so its writes cannot be tracked.", space.name))),
                Err(refusal) => Err(refusal.words),
            }
        }
        "when" => {
            let conditions = session.bp_conditions(processor)?;
            let available = conditions.list("available");
            if available.is_empty() {
                return Ok(Answer::fail("This core reports no hardware conditions to break on."));
            }
            let armed = conditions.list("armed");
            if arg.is_empty() {
                let rows: Vec<String> = available
                    .iter()
                    .map(|c| {
                        let name = c.text("name");
                        let state = match armed.iter().find(|a| equals_ignore_case(&a.text("name"), &name)) {
                            None => "off".to_string(),
                            Some(a) => format!("{}, {}x", if a.flag("log_only") { "logging" } else { "halting" }, a.int("hits")),
                        };
                        format!("  {} {}  [{state}]", pad_right(&name, 12), c.text("description"))
                    })
                    .collect();
                return Ok(Answer::ok(format!("Conditions this core detects:\n{}", rows.join("\n"))));
            }
            let Some(chosen) = available.iter().find(|c| equals_ignore_case(&c.text("name"), arg)) else {
                let names: Vec<String> = available.iter().map(|c| c.text("name")).collect();
                let refs: Vec<&str> = names.iter().map(String::as_str).collect();
                return Ok(Answer::fail(format!("'{arg}' is not a condition this core detects.{} Try {}.", suggestion::hint(arg, &refs, 2), names.join("/"))));
            };
            let (name, description) = (chosen.text("name"), chosen.text("description"));
            let mode = parts.get(at + 2).map(|m| to_lower(m)).unwrap_or_default();
            if mode == "off" {
                return Ok(Answer::ok(match session.bp_arm_condition(processor, &name, abi::condition::DISARM) {
                    Ok(()) => format!("No longer watching for {name}."),
                    Err(refusal) if refusal.is(status::NO_SUCH_ID) => format!("{name} was not armed."),
                    Err(refusal) => return Err(refusal.words),
                }));
            }
            if !mode.is_empty() && mode != "log" {
                return Ok(Answer::ok(format!("Usage: bp when {name} [log|off]")));
            }
            let log_only = mode == "log";
            session.bp_arm_condition(processor, &name, if log_only { abi::condition::LOG_ONLY } else { 0 })?;
            Ok(Answer::ok(if log_only {
                format!("Recording every {name} ({description}) without halting - read it back with `bp log`.")
            } else {
                format!("Halting on {name} ({description}).")
            }))
        }
        "depth" => {
            if arg.is_empty() {
                return Ok(Answer::ok("Usage: bp depth <n>"));
            }
            if equals_ignore_case(arg, "off") {
                session.bp_depth_guard(processor, -1)?;
                return Ok(Answer::ok("Call-depth guard removed."));
            }
            let depth = parse_hex(arg)?;
            match session.bp_depth_guard(processor, depth) {
                Ok(()) => Ok(Answer::ok(format!("Halting if the call stack ever gets deeper than {depth}."))),
                Err(refusal) if refusal.is(status::BAD_ARGUMENT) => Ok(Answer::fail("This core does not report a call stack, so there is no depth to guard.")),
                Err(refusal) => Err(refusal.words),
            }
        }
        "forbid" => {
            if arg.is_empty() {
                return Ok(Answer::ok("Usage: bp forbid <addr>-<end>"));
            }
            let (start, end) = parse_range(arg)?;
            let id = session.bp_forbid_range(processor, start as u32, end as u32)?;
            Ok(Answer::ok(format!("Forbid range #{id} added: nothing halts while the PC is in {}.", range(start, end))))
        }
        "log" => {
            if equals_ignore_case(arg, "clear") {
                session.bp_log_clear(processor)?;
                return Ok(Answer::ok("Logpoint log cleared."));
            }
            let count = if arg.is_empty() { 32 } else { parse_hex(arg)? };
            let log = session.bp_log(processor, count.max(0) as u32)?;
            let entries = log.list("entries");
            if entries.is_empty() {
                let list = session.bp_list(processor)?;
                let conditions = session.bp_conditions(processor)?;
                let any = list.list("code").iter().chain(list.list("data")).any(|b| b.field("log").is_some()) || conditions.list("armed").iter().any(|c| c.flag("log_only"));
                return Ok(Answer::ok(if any { "Nothing logged yet - the logpoints have not been hit." } else { "No logpoints set - add one with `bp add <addr> log <expr>`." }));
            }
            let lines: Vec<String> = entries
                .iter()
                .map(|e| {
                    let id = e.int("id");
                    format!("  {} {}  {}", if id == 0 { "when".to_string() } else { format!("#{id}") }, describe(&session, e.address("address")), e.text("text"))
                })
                .collect();
            let recorded = log.int("recorded");
            let more = if recorded > entries.len() as i64 { format!("\n  ({recorded} recorded in total)") } else { String::new() };
            Ok(Answer::ok(lines.join("\n") + &more))
        }
        "list" => {
            let list = session.bp_list(processor)?;
            let armed = session.bp_conditions(processor)?;
            let (code, data, forbid, conditions) = (list.list("code"), list.list("data"), list.list("forbid"), armed.list("armed"));
            let uninitialized = list.field("uninitialized").map(|u| u.text("space"));
            if code.is_empty() && data.is_empty() && forbid.is_empty() && conditions.is_empty() {
                let none = if coprocessor { format!("No active {} breakpoints.", cpu.unwrap().name) } else { "No active breakpoints.".to_string() };
                return Ok(Answer::ok(match uninitialized {
                    Some(space) => format!("{none}\n  (halting on uninitialized reads of {space})"),
                    None => none,
                }));
            }
            let state = |b: &emusen_dotnet::json::Value| format!("({}, {} {}x)", if b.flag("enabled") { "enabled" } else { "disabled" }, if b.field("log").is_none() { "hit" } else { "logged" }, b.int("hits"));
            let mut lines = Vec::new();
            for b in code {
                let (start, end) = (b.address("address"), b.address("end"));
                let place = if start == end { describe(&session, start) } else { range(start, end) };
                lines.push(format!("  #{}: {place}{} {}", b.int("id"), clause_text(b.optional_text("condition").as_deref(), b.optional_text("log").as_deref()), state(b)));
            }
            for b in data {
                let value = b.int("value");
                lines.push(format!(
                    "  #{}: {} {} {}{}{}{} {}",
                    b.int("id"),
                    if b.flag("on_read") { "read" } else { "write" },
                    b.text("space"),
                    data_range(b.address("address"), b.address("end")),
                    if value < 0 { String::new() } else { format!(" = 0x{value:02X}") },
                    if b.flag("changed_only") { " changed" } else { "" },
                    clause_text(b.optional_text("condition").as_deref(), b.optional_text("log").as_deref()),
                    state(b)
                ));
            }
            for f in forbid {
                lines.push(format!("  #{}: forbid {} ({})", f.int("id"), range(f.address("address"), f.address("end")), if f.flag("enabled") { "enabled" } else { "disabled" }));
            }
            for c in conditions {
                lines.push(format!(
                    "  when {}: {} ({}, {}x)",
                    c.text("name"),
                    if c.flag("log_only") { "logging" } else { "halting" },
                    if c.flag("enabled") { "enabled" } else { "disabled" },
                    c.int("hits")
                ));
            }
            let guard = list.int("depth_guard");
            if guard >= 0 {
                lines.push(format!("  depth guard: halt above depth {guard}"));
            }
            if let Some(space) = uninitialized {
                lines.push(format!("  uninitialized reads: {space}"));
            }
            Ok(Answer::ok(lines.join("\n")))
        }
        "on" | "off" => {
            if arg.is_empty() {
                return Ok(Answer::ok(format!("Usage: bp {sub} <id>")));
            }
            let id = parse_hex(arg)?;
            match session.bp_set_enabled(processor, id as u32, sub == "on") {
                Ok(()) => Ok(Answer::ok(format!("{scope_name} #{id} {}.", if sub == "on" { "enabled" } else { "disabled" }))),
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => Ok(Answer::fail(format!("No {other}breakpoint #{id} found."))),
                Err(refusal) => Err(refusal.words),
            }
        }
        "remove" => {
            if arg.is_empty() {
                return Ok(Answer::ok("Usage: bp remove <id>"));
            }
            let id = parse_hex(arg)?;
            match session.bp_remove(processor, id as u32) {
                Ok(()) => Ok(Answer::ok(format!("{scope_name} #{arg} removed."))),
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => Ok(Answer::ok(format!("No {other}breakpoint #{arg} found."))),
                Err(refusal) => Err(refusal.words),
            }
        }
        _ => {
            let subcommands = ["add", "read", "write", "uninit", "when", "depth", "forbid", "log", "list", "on", "off", "remove"];
            Ok(Answer::ok(format!("Unknown 'bp' subcommand '{sub}'.{} Try {}.", suggestion::hint(&sub, &subcommands, 2), subcommands.join("/"))))
        }
    }
}

pub fn watch(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: watch add|list|log|clear|remove ..."));
    }
    let sub = to_lower(&parts[1]);
    match sub.as_str() {
        "add" => {
            if parts.len() < 5 {
                return Ok(Answer::ok("Usage: watch add <space> <addr> <len> [write|read|both]"));
            }
            find(&session, &parts[2])?;
            let addr = parse_hex(&parts[3])?;
            let len = parse_hex(&parts[4])?;
            let kind = match parts.get(5).map(|k| to_lower(k)).as_deref() {
                None | Some("write") => abi::watch::WRITE,
                Some("read") => abi::watch::READ,
                Some("both") => abi::watch::BOTH,
                Some(_) => return Err(format!("Unknown watch kind '{}'. Use write, read, or both.", parts[5])),
            };
            let id = session.watch_add(&parts[2], addr as u32, len as u32, kind)?;
            Ok(Answer::ok(format!("Watch #{id} added: {} 0x{addr:X}-0x{:X} ({})", parts[2], addr.wrapping_add(len).wrapping_sub(1), watch_kind(i64::from(kind)))))
        }
        "list" => {
            let list = session.watch_list()?;
            if list.items().is_empty() {
                return Ok(Answer::ok("No active watches."));
            }
            let rows: Vec<String> = list
                .items()
                .iter()
                .map(|w| {
                    let start = w.address("address");
                    format!("  #{}: {} 0x{start:X}-0x{:X} ({})", w.int("id"), w.text("space"), start.wrapping_add(w.int("length") as i32).wrapping_sub(1), watch_kind(w.int("kind")))
                })
                .collect();
            Ok(Answer::ok(rows.join("\n")))
        }
        "log" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: watch log <id> [<count>]"));
            }
            let id = parse_hex(&parts[2])?;
            let count = if parts.len() >= 4 { parse_hex(&parts[3])? } else { 20 };
            let events = match session.watch_events(id as u32, count.max(0) as u32) {
                Ok(events) => events.items().to_vec(),
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => Vec::new(),
                Err(refusal) => return Err(refusal.words),
            };
            if events.is_empty() {
                return Ok(Answer::ok(format!("No events recorded for watch #{id} (or it doesn't exist).")));
            }
            let rows: Vec<String> = events
                .iter()
                .map(|e| format!("  [{}] {} 0x{:X} = 0x{:02X} ({})", e.int("sequence"), if e.int("kind") == 0 { "W" } else { "R" }, e.address("address"), e.int("value"), e.text("context")))
                .collect();
            Ok(Answer::ok(rows.join("\n")))
        }
        "summary" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: watch summary <id>"));
            }
            let id = parse_hex(&parts[2])?;
            let summary = match session.watch_summary(id as u32) {
                Ok(summary) => Some(summary),
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => None,
                Err(refusal) => return Err(refusal.words),
            };
            let Some(summary) = summary.filter(|s| !s.list("sites").is_empty()) else {
                return Ok(Answer::ok(format!("No events recorded for watch #{id} (or it doesn't exist).")));
            };
            let sites = summary.list("sites");
            let mut lines: Vec<String> = sites.iter().map(|s| format!("  {}x  {}", pad_left(&s.int("count").to_string(), 6), s.text("context"))).collect();
            let (total, retained) = (summary.int("total"), summary.int("retained"));
            let buffer = if total > retained { format!(" ({retained} still in the event buffer - `watch log` shows only those)") } else { String::new() };
            lines.push(format!("  {} site(s), {total} access(es) total{buffer}", sites.len()));
            Ok(Answer::ok(lines.join("\n")))
        }
        "clear" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: watch clear <id>"));
            }
            match session.watch_clear(parse_hex(&parts[2])? as u32) {
                Ok(()) => {}
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => {}
                Err(refusal) => return Err(refusal.words),
            }
            Ok(Answer::ok(format!("Cleared events for watch #{}.", parts[2])))
        }
        "remove" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: watch remove <id>"));
            }
            Ok(Answer::ok(match session.watch_remove(parse_hex(&parts[2])? as u32) {
                Ok(()) => format!("Watch #{} removed.", parts[2]),
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => format!("No watch #{} found.", parts[2]),
                Err(refusal) => return Err(refusal.words),
            }))
        }
        _ => {
            let subcommands = ["add", "list", "log", "summary", "clear", "remove"];
            Ok(Answer::ok(format!("Unknown 'watch' subcommand '{sub}'.{} Try {}.", suggestion::hint(&sub, &subcommands, 2), subcommands.join("/"))))
        }
    }
}

pub fn step(call: &Call, args: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    let cpus = scope::cpus(&session)?;
    let (cpu, at) = scoped(&cpus, args, 1);
    if cpu.is_none() && args.len() > 1 && scope::is_scope_word(&cpus, &args[1]) {
        return Ok(Answer::fail("This cartridge has no coprocessor to step."));
    }
    if let Some(cpu) = cpu.filter(|c| !c.can_halt) {
        return Ok(Answer::fail(format!("{} cannot be halted by this core, so it cannot be stepped.", cpu.name)));
    }
    let processor = scope::index(&session, cpu);
    let mode = args.get(at).map(|m| to_lower(m)).unwrap_or_default();
    if mode == "over" || mode == "out" {
        if !has_call_stack(&session, cpu)? {
            return Ok(Answer::fail(format!("This core does not report a call stack, so `step {mode}` has no depth to measure against.")));
        }
        let op = if mode == "over" { abi::run::STEP_OVER } else { abi::run::STEP_OUT };
        return Ok(match session.run_control(op, processor, 0) {
            Ok(_) => Answer::ok(if mode == "over" { "Stepping over..." } else { "Stepping out..." }),
            Err(refusal) if refusal.is(status::BAD_ARGUMENT) => Answer::fail("Already at the outermost frame - nothing to step out of."),
            Err(refusal) if refusal.is(status::NOT_SUPPORTED) => Answer::fail(format!("This core does not report a call stack to its breakpoints, so `step {mode}` cannot be armed.")),
            Err(refusal) => return Err(refusal.words),
        });
    }
    let mut count = 1;
    if !mode.is_empty() {
        let Ok(parsed) = parse_hex(&args[at]) else { return Ok(Answer::fail(format!("'{}' is not 'over', 'out', or a count.", args[at]))) };
        count = parsed;
        if count < 1 {
            return Ok(Answer::fail("Step count must be at least 1."));
        }
    }
    session.run_control(abi::run::STEP, processor, i64::from(count))?;
    Ok(Answer::ok(if count == 1 { "Stepping one instruction...".to_string() } else { format!("Stepping {count} instructions...") }))
}

pub fn runto(call: &Call, parts: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: runto nmi|irq|brk|cop|scanline <n>|frame [<n>]|<addr>"));
    }
    let processor = session.no_processor();
    let interrupt = |kind: u32, description: &str| -> Run {
        session.run_control(abi::run::TO_INTERRUPT, processor, i64::from(kind))?;
        Ok(Answer::ok(format!("Running to {description}...")))
    };
    match to_lower(&parts[1]).as_str() {
        "nmi" => interrupt(abi::frame::NMI, "the next NMI"),
        "irq" => interrupt(abi::frame::IRQ, "the next IRQ"),
        "brk" => interrupt(abi::frame::BRK, "the next BRK"),
        "cop" => interrupt(abi::frame::COP, "the next COP"),
        "scanline" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: runto scanline <n>"));
            }
            let scanline = int_parse(&parts[2])?;
            session.run_control(abi::run::TO_SCANLINE, processor, i64::from(scanline))?;
            Ok(Answer::ok(format!("Running to scanline {scanline}...")))
        }
        "frame" => {
            let frame = if parts.len() > 2 { long_parse(&parts[2])? } else { session.info()?.int("frame") + 1 };
            session.run_control(abi::run::TO_FRAME, processor, frame)?;
            Ok(Answer::ok(format!("Running to frame {frame}...")))
        }
        _ => {
            let Ok(address) = parse_hex(&parts[1]) else {
                let subcommands = ["nmi", "irq", "brk", "cop", "scanline", "frame"];
                return Ok(Answer::ok(format!("Unknown 'runto' target '{}'.{} Try {} or an address.", parts[1], suggestion::hint(&parts[1], &subcommands, 2), subcommands.join("/"))));
            };
            let id = session.run_control(abi::run::TO_ADDRESS, processor, i64::from(address as u32))?;
            Ok(Answer::ok(format!("Running to ${} (breakpoint #{id} - `bp remove {id}` when done)...", hex(address, 6))))
        }
    }
}

pub fn bt(call: &Call, parts: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    let cpus = scope::cpus(&session)?;
    let (cpu, at) = scoped(&cpus, parts, 1);
    if cpu.is_none() && parts.len() > 1 && scope::is_scope_word(&cpus, &parts[1]) {
        return Ok(Answer::fail("This cartridge has no coprocessor to report a call stack for."));
    }
    if !has_call_stack(&session, cpu)? {
        return Ok(Answer::fail(match cpu {
            Some(cpu) if !scope::is_main(&cpus, Some(cpu)) => format!("{} does not report a call stack - see `man cpus`.", cpu.name),
            _ => "This core does not report a call stack.".to_string(),
        }));
    }
    let processor = scope::index(&session, cpu);
    if parts.get(at).is_some_and(|p| equals_ignore_case(p, "reset")) {
        session.profile_op(processor, abi::profile::RESET_STACK, "{}")?;
        return Ok(Answer::ok("Call stack cleared."));
    }
    let limit = if parts.len() > at { parse_hex(&parts[at])? } else { 32 };
    let frames = session.call_stack(processor, i32::MAX as u32)?;
    let frames = frames.items();
    let status = emusen_dotnet::json::parse(&String::from_utf8_lossy(&session.profile_op(processor, abi::profile::STATUS, "{}")?)).map_err(|e| e.to_string())?;
    let unmatched = status.int("unmatched_returns");
    if frames.is_empty() {
        return Ok(Answer::ok(if unmatched > 0 {
            format!("Call stack empty ({unmatched} unmatched return(s) seen).")
        } else {
            "Call stack empty - nothing has been called since the last reset.".to_string()
        }));
    }
    let mut lines = Vec::new();
    let mut depth = frames.len() as i64;
    for frame in frames.iter().take(limit.max(0) as usize) {
        let kind = match frame.int("kind") {
            0 => "call",
            1 => "NMI",
            2 => "IRQ",
            3 => "BRK",
            _ => "COP",
        };
        lines.push(format!(
            "  #{} {}  <- {kind} from {}  (frame {})",
            pad_right(&(depth - 1).to_string(), 3),
            describe(&session, frame.address("target")),
            describe(&session, frame.address("source")),
            frame.int("frame")
        ));
        depth -= 1;
    }
    if frames.len() as i64 > i64::from(limit) {
        lines.push(format!("  ... {} more frame(s)", frames.len() as i64 - i64::from(limit)));
    }
    if unmatched > 0 {
        lines.push(format!("  ({unmatched} unmatched return(s) seen - see `man bt`)"));
    }
    Ok(Answer::ok(lines.join("\n")))
}

/// `resume`: the host's loop asked to run, whether or not a game is loaded, as the C# returned `HostAction.Resume`.
pub fn resume(call: &Call) -> Run {
    if let Ok(session) = session(call) {
        let _ = session.run_control(abi::run::RESUME, 0, 0);
    }
    Ok(Answer::ok("Resuming."))
}

pub fn trace(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: trace <count> | trace off"));
    }
    let traces = session(call).ok().and_then(|s| s.can("trace").ok().filter(|&can| can).map(|_| s));
    let Some(session) = traces else { return Ok(Answer::ok("No CPU trace switch is registered for this target.")) };
    if equals_ignore_case(&parts[1], "off") {
        session.trace_off(0)?;
        return Ok(Answer::ok("Trace disabled."));
    }
    let count = parse_hex(&parts[1])?;
    if count < 1 {
        return Ok(Answer::fail("Trace count must be at least 1."));
    }
    session.trace(0, count as u32)?;
    Ok(Answer::ok(format!("Trace armed for the next {count} instructions.")))
}

pub fn log(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 2 {
        return Ok(Answer::ok(usage("log")));
    }
    let session = session(call)?;
    let word = &parts[1];
    if equals_ignore_case(word, "off") {
        session.log_switch(false)?;
        return Ok(Answer::ok("Logging disabled (individual flags unchanged)."));
    }
    if equals_ignore_case(word, "on") {
        session.log_switch(true)?;
        return Ok(Answer::ok("Logging enabled."));
    }
    if equals_ignore_case(word, "status") {
        return Ok(Answer::ok(format!("Logging is {}.", if session.log_status()? { "on" } else { "off" })));
    }
    Ok(Answer::ok(usage("log")))
}
