//! The processors and their code: `regs`, `setreg`, `cpus`, `eval`, `label`, `vectors`, `addr`, `disasm`, and the
//! static scans `callers`, `writers` and `readers`.

use crate::{core_name, required, session, Run};
use dianash_module::{Answer, Call};
use emusen_dotnet::file::{read_all_lines, write_all_text};
use emusen_dotnet::json::Value;
use emusen_dotnet::suggestion;
use emusen_dotnet::text::{contains_ignore_case, equals_ignore_case, pad_right, to_lower, trim_end};
use emusen_session_service::scope::{self, Cpu};
use emusen_session_service::words::{describe, hex, label_at, parse_hex};
use emusen_session_service::{abi, Record, Session};

/// A register as the service reports it.
struct Register {
    name: String,
    value: u64,
    bits: i64,
}

fn registers(value: &Value) -> Vec<Register> {
    value.items().iter().map(|r| Register { name: r.text("name"), value: r.field("value").and_then(unsigned).unwrap_or(0), bits: r.int("bits") }).collect()
}

/// A JSON number as the unsigned 64-bit register value it was written from.
fn unsigned(value: &Value) -> Option<u64> {
    match value {
        Value::Number(token) => token.parse::<u64>().ok().or_else(|| token.parse::<i64>().ok().map(|v| v as u64)),
        _ => None,
    }
}

/// `Value.ToString("X" + Math.Max(1, BitWidth / 4))`.
fn register_hex(value: u64, bits: i64) -> String {
    let digits = (bits / 4).max(1) as usize;
    format!("{value:0digits$X}")
}

fn no_such_cpu(session: &Session, cpus: &[Cpu], word: &str) -> Result<String, String> {
    Ok(format!("No debug CPU named '{word}' on this {}. Available: {}.", core_name(session)?, scope::name_list(cpus)))
}

pub fn regs(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() > 1 {
        let cpus = scope::cpus(&session)?;
        let Some(only) = scope::find(&cpus, &parts[1]) else { return Ok(Answer::ok(no_such_cpu(&session, &cpus, &parts[1])?)) };
        if !only.registers {
            return Ok(Answer::ok(format!("{} reports no registers on this core.", only.name)));
        }
        let mut out = format!("{} registers:\n", only.description);
        for r in registers(&session.registers(only.index)?) {
            out += &format!("  {} = 0x{}\n", pad_right(&r.name, 9), register_hex(r.value, r.bits));
        }
        return Ok(Answer::ok(trim_end(&out)));
    }
    let mut out = format!("{} CPU registers:\n", core_name(&session)?);
    for r in registers(&session.telemetry(abi::telemetry::CPU)?) {
        out += &format!("  {} = 0x{}\n", pad_right(&r.name, 4), register_hex(r.value, r.bits));
    }
    out += "Video registers:\n";
    for r in registers(&session.telemetry(abi::telemetry::VIDEO)?) {
        out += &format!("  {} = 0x{:02X}\n", pad_right(&r.name, 8), r.value);
    }
    for (kind, title) in [(abi::telemetry::APU, "APU registers:"), (abi::telemetry::COPROCESSOR, "Coprocessor registers:")] {
        let more = registers(&session.telemetry(kind)?);
        if !more.is_empty() {
            out += title;
            out.push('\n');
            for r in more {
                out += &format!("  {} = 0x{}\n", pad_right(&r.name, 9), register_hex(r.value, r.bits));
            }
        }
    }
    Ok(Answer::ok(trim_end(&out)))
}

pub fn setreg(call: &Call, parts: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    if parts.len() < 3 {
        return Ok(Answer::ok("Usage: setreg [<cpu>] <register> <value>"));
    }
    let cpus = scope::cpus(&session)?;
    let mut at = 1;
    let mut cpu = cpus.first();
    if parts.len() > 3 {
        if let Some(named) = scope::find(&cpus, &parts[1]) {
            cpu = Some(named);
            at = 2;
        }
    }
    let Some(cpu) = cpu else {
        return Ok(Answer::fail(format!("{} publishes no debug CPUs, so it has no writable registers.", core_name(&session)?)));
    };
    if parts.len() < at + 2 {
        return Ok(Answer::ok("Usage: setreg [<cpu>] <register> <value>"));
    }
    let name = &parts[at];
    let Ok(value) = parse_hex(&parts[at + 1]) else {
        return Ok(Answer::fail(format!("'{}' is not a value.", parts[at + 1])));
    };
    let value = value as i64 as u64;
    let known = if cpu.registers { registers(&session.registers(cpu.index)?) } else { Vec::new() };
    let Some(register) = known.iter().find(|r| equals_ignore_case(&r.name, name)) else {
        let available = if known.is_empty() { "(none)".to_string() } else { known.iter().map(|r| r.name.as_str()).collect::<Vec<_>>().join(", ") };
        return Ok(Answer::fail(format!("{} has no register named '{name}'. Available: {available}.", cpu.name)));
    };
    if !cpu.writable_registers {
        return Ok(Answer::fail(format!("{} reports its registers but this core cannot write them - see `man cpus`.", cpu.name)));
    }
    let max = if register.bits >= 64 { u64::MAX } else { (1u64 << register.bits) - 1 };
    if value > max {
        return Ok(Answer::fail(format!("{} is {}-bit, so 0x{value:X} does not fit (max 0x{max:X}).", register.name, register.bits)));
    }
    match session.set_register(cpu.index, &register.name, value) {
        Ok(()) => {}
        Err(refusal) if refusal.is(abi::status::READ_ONLY) => return Ok(Answer::fail(format!("{}'s {} is read-only on this core.", cpu.name, register.name))),
        Err(refusal) => return Err(refusal.words),
    }
    Ok(Answer::ok(format!("{} {} = 0x{} (was 0x{}).", cpu.name, register.name, register_hex(value, register.bits), register_hex(register.value, register.bits))))
}

pub fn cpus(call: &Call) -> Run {
    let session = session(call)?;
    let cpus = scope::cpus(&session)?;
    let core = core_name(&session)?;
    if cpus.is_empty() {
        return Ok(Answer::ok(format!("{core} publishes no per-CPU debug list; every command targets its main CPU.")));
    }
    let mut out = format!("{core} debug processors:\n");
    for cpu in &cpus {
        let pc = cpu.pc.map_or("-".to_string(), |pc| format!("${}", hex(pc, 6)));
        let features: Vec<&str> = [
            cpu.can_halt.then_some("bp"),
            cpu.coverage.then_some("cov"),
            cpu.call_stack.then_some("bt"),
            cpu.registers.then_some("regs"),
            cpu.writable_registers.then_some("setreg"),
            cpu.code_space.is_some().then_some("disasm"),
        ]
        .into_iter()
        .flatten()
        .collect();
        out += &format!("  {} PC={} {}\n", pad_right(&cpu.name, 5), pad_right(&pc, 8), cpu.description);
        out += &format!("        supports: {}{}\n", features.join(", "), cpu.code_space.as_ref().map_or(String::new(), |c| format!("   code space: {c}")));
    }
    out += "Scope a command by putting the name first, e.g. `bp sa1 add 8000` or `cov gsu on`.";
    Ok(Answer::ok(out))
}

/// `ExpressionEvaluator.Format`: decimal, hexadecimal and binary, the binary in 8, 16 or 24 bits by size.
fn format_value(value: i64) -> String {
    let magnitude = value.unsigned_abs();
    let bits = if magnitude > 0xFFFF { 24 } else if magnitude > 0xFF { 16 } else { 8 };
    let mut binary = String::new();
    for bit in (0..bits).rev() {
        binary.push(if (magnitude >> bit) & 1 != 0 { '1' } else { '0' });
        if bit % 4 == 0 && bit != 0 {
            binary.push(' ');
        }
    }
    let hex = if value < 0 { format!("-${magnitude:X}") } else { format!("${value:X}") };
    format!("{value}  {hex}  %{binary}")
}

pub fn eval(call: &Call, parts: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: eval <expr> | eval symbols [<filter>]"));
    }
    let cpus = scope::cpus(&session)?;
    let mut at = 1;
    let mut cpu = None;
    if parts.len() > 2 {
        if let Some(named) = scope::find(&cpus, &parts[1]) {
            cpu = Some(named);
            at = 2;
        }
    }
    let processor = scope::index(&session, cpu);
    if equals_ignore_case(&parts[at], "symbols") {
        let filter = parts.get(at + 1).map(String::as_str).unwrap_or("");
        let names = session.symbols(processor, (!filter.is_empty()).then_some(filter))?;
        let names: Vec<String> = names.items().iter().filter_map(|n| n.as_str().map(|s| format!("  {s}"))).collect();
        if names.is_empty() {
            return Ok(Answer::ok(if filter.is_empty() { "No symbols available.".to_string() } else { format!("No symbol matches '{filter}'.") }));
        }
        return Ok(Answer::ok(names.join("\n")));
    }
    let expression = parts[at..].join(" ");
    match session.evaluate(processor, &expression) {
        Ok(value) => Ok(Answer::ok(format_value(value.int("value")))),
        Err(refusal) if refusal.is(abi::status::PARSE) => Ok(Answer::fail(refusal.words)),
        Err(refusal) => Ok(Answer::fail(format!("Cannot evaluate '{expression}': {}", refusal.words))),
    }
}

/// The labels in address order, as `All()` gives them.
fn labels(session: &Session) -> Result<Vec<(i32, String, Option<String>)>, String> {
    Ok(session.label_list()?.items().iter().map(|l| (l.address("address"), l.text("name"), l.optional_text("comment"))).collect())
}

pub fn label(call: &Call, parts: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    if !session.info()?.list("capabilities").iter().any(|c| c.as_str() == Some("labels")) {
        return Ok(Answer::fail("This core does not support labels."));
    }
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: label add|list|at|remove|clear|load|save ..."));
    }
    let sub = to_lower(&parts[1]);
    match sub.as_str() {
        "add" => {
            if parts.len() < 4 {
                return Ok(Answer::ok("Usage: label add <addr> <name> [<comment...>]"));
            }
            let address = parse_hex(&parts[2])?;
            let comment = if parts.len() > 4 { parts[4..].join(" ") } else { String::new() };
            session.label_add(address as u32, &parts[3], Some(&comment))?;
            Ok(Answer::ok(format!("${} is now '{}'.", hex(address, 6), parts[3])))
        }
        "list" => {
            let filter = parts.get(2).map(String::as_str).unwrap_or("");
            let rows: Vec<String> = labels(&session)?
                .into_iter()
                .filter(|(_, name, _)| filter.is_empty() || contains_ignore_case(name, filter))
                .map(|(address, name, comment)| format!("  ${}  {name}{}", hex(address, 6), comment.map_or(String::new(), |c| format!("  ; {c}"))))
                .collect();
            if rows.is_empty() {
                return Ok(Answer::ok(if filter.is_empty() { "No labels defined.".to_string() } else { format!("No label matches '{filter}'.") }));
            }
            Ok(Answer::ok(rows.join("\n")))
        }
        "at" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: label at <addr>"));
            }
            let address = parse_hex(&parts[2])?;
            if let Some((exact, _)) = label_at(&session, address) {
                return Ok(Answer::ok(format!("  ${}  {exact}", hex(address, 6))));
            }
            let all = labels(&session)?;
            let Some((below, name, _)) = all.into_iter().filter(|(a, _, _)| *a <= address).last() else {
                return Ok(Answer::ok(format!("Nothing labelled at or below ${}.", hex(address, 6))));
            };
            Ok(Answer::ok(format!("  ${}  {name}+{}", hex(below, 6), address.wrapping_sub(below))))
        }
        "remove" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: label remove <name|addr>"));
            }
            let key = &parts[2];
            if session.label_remove(Some(key), 0).is_ok() {
                return Ok(Answer::ok(format!("Label '{key}' removed.")));
            }
            if let Ok(address) = parse_hex(key) {
                if session.label_remove(None, address as u32).is_ok() {
                    return Ok(Answer::ok(format!("Label at ${} removed.", hex(address, 6))));
                }
            }
            Ok(Answer::fail(format!("No label named '{key}'.")))
        }
        "clear" => {
            let count = labels(&session)?.len();
            session.label_clear()?;
            Ok(Answer::ok(format!("{count} label(s) cleared.")))
        }
        "load" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: label load <path>"));
            }
            let Ok(path) = call.resolve(&parts[2]) else { return Ok(Answer::fail(format!("Path outside this sandbox: {}", parts[2]))) };
            if !std::path::Path::new(&path).is_file() {
                return Ok(Answer::fail(format!("No such file: {}", parts[2])));
            }
            let lines = read_all_lines(&path)?;
            let loaded = session.label_load(&lines.join("\n"))?;
            let report = format!("{} label(s) loaded from {}.", loaded.int("added"), parts[2]);
            let errors: Vec<String> = loaded.list("errors").iter().filter_map(|e| e.as_str().map(|e| format!("  {e}"))).collect();
            Ok(Answer::ok(if errors.is_empty() { report } else { format!("{report}\n{}", errors.join("\n")) }))
        }
        "save" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: label save <path>"));
            }
            let Ok(path) = call.resolve(&parts[2]) else { return Ok(Answer::fail(format!("Path outside this sandbox: {}", parts[2]))) };
            let text = session.label_save()?;
            write_all_text(&path, &text, false)?;
            Ok(Answer::ok(format!("{} label(s) written to {}.", labels(&session)?.len(), parts[2])))
        }
        _ => {
            let subcommands = ["add", "list", "at", "remove", "clear", "load", "save"];
            Ok(Answer::ok(format!("Unknown 'label' subcommand '{sub}'.{} Try {}.", suggestion::hint(&sub, &subcommands, 2), subcommands.join("/"))))
        }
    }
}

pub fn vectors(call: &Call) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    let vectors = session.vectors()?;
    if vectors.items().is_empty() {
        return Ok(Answer::ok(format!("{} does not report an interrupt vector table.", core_name(&session)?)));
    }
    let spaces = crate::memory::all(&session)?;
    let Some(bus) = spaces.iter().find(|s| equals_ignore_case(&s.name, "CpuBus")) else {
        return Ok(Answer::fail("This core publishes no CpuBus space to read the vectors from."));
    };
    let coverage_armed = session.coverage_op(session.no_processor(), abi::coverage::STATUS, b"{}").ok().and_then(|b| emusen_dotnet::json::parse(&String::from_utf8_lossy(&b)).ok()).is_some_and(|s| s.flag("armed"));
    let mut lines = Vec::new();
    let mut mode = String::new();
    for vector in vectors.items() {
        let this_mode = vector.text("mode");
        if this_mode != mode {
            mode = this_mode;
            if !mode.is_empty() {
                lines.push(format!("{mode}:"));
            }
        }
        let address = vector.address("address");
        let width = vector.int("width") as i32;
        let bytes = crate::memory::read(&session, bus, address, width)?;
        let destination = bytes.iter().enumerate().fold(0i32, |d, (i, &b)| d | (i32::from(b).wrapping_shl(8 * i as u32)));
        let ran = if coverage_armed && executed(&session, destination) { "  (ran)" } else { "" };
        lines.push(format!("  {} ${} -> {}{ran}", pad_right(&vector.text("name"), 12), hex(address, 6), describe(&session, destination)));
    }
    Ok(Answer::ok(lines.join("\n")))
}

/// `CoverageRegistry.WasExecuted` on the main processor's coverage.
fn executed(session: &Session, address: i32) -> bool {
    let args = format!("{{\"address\":{},\"length\":1,\"limit\":0}}", address as u32);
    session.coverage_op(session.no_processor(), abi::coverage::QUERY, args.as_bytes()).ok().and_then(|b| emusen_dotnet::json::parse(&String::from_utf8_lossy(&b)).ok()).is_some_and(|q| q.int("executed") > 0)
}

pub fn addr(call: &Call, parts: &[String]) -> Run {
    let session = match required(call) {
        Ok(session) => session,
        Err(refused) => return Ok(refused),
    };
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: addr <addr>"));
    }
    let Ok(address) = parse_hex(&parts[1]) else { return Ok(Answer::fail(format!("'{}' is not an address.", parts[1]))) };
    let physical = match session.resolve_physical(address as u32) {
        Ok(physical) => physical,
        Err(refusal) if refusal.is(abi::status::ABSENT) => {
            return Ok(Answer::ok(format!("{} does not report an address decode, or ${} is unmapped.", core_name(&session)?, hex(address, 6))));
        }
        Err(refusal) => return Err(refusal.words),
    };
    let label = describe(&session, address);
    let space = physical.text("space");
    if !physical.flag("addressable") {
        return Ok(Answer::ok(format!("{label} -> {space}")));
    }
    let offset = physical.int("offset") as i32;
    Ok(Answer::ok(format!("{label} -> {space} 0x{offset:X}\n  mem {space} {offset:X} 10")))
}

fn is_width_hint(word: &str) -> bool {
    matches!(word, "m8" | "m16" | "x8" | "x16" | "M8" | "M16" | "X8" | "X16")
}

/// One disassembled instruction as the service gives it.
struct Instruction {
    address: i32,
    bytes: Vec<u8>,
    mnemonic: String,
    operands: String,
    reference: Option<(String, i32)>,
}

fn instructions(value: &Value) -> Vec<Instruction> {
    value
        .items()
        .iter()
        .map(|i| Instruction {
            address: i.address("address"),
            bytes: i.list("bytes").iter().filter_map(emusen_session_service::number).map(|b| b as u8).collect(),
            mnemonic: i.text("mnemonic"),
            operands: i.text("operands"),
            reference: i.field("reference").map(|r| (r.text("kind"), r.address("target"))),
        })
        .collect()
}

pub fn disasm(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: disasm <space>|<cpu> <addr> [<count>]"));
    }
    let hints: Vec<String> = parts.iter().filter(|p| is_width_hint(p)).cloned().collect();
    let parts: Vec<String> = if hints.is_empty() { parts.to_vec() } else { parts.iter().filter(|p| !is_width_hint(p)).cloned().collect() };
    let cpus = scope::cpus(&session)?;
    let mut named = scope::find(&cpus, &parts[1]);
    let space = match named.and_then(|c| c.code_space.clone()) {
        Some(code_space) => code_space,
        None => {
            named = None;
            parts[1].clone()
        }
    };
    let at = 2;
    if named.is_none() && parts.len() < 3 {
        return Ok(Answer::ok("Usage: disasm <space> <addr> [<count>]"));
    }
    crate::memory::find(&session, &space)?;
    let address = if parts.len() > at {
        parse_hex(&parts[at])?
    } else if let Some(pc) = named.and_then(|c| c.pc) {
        pc
    } else {
        return Ok(Answer::ok(format!("Usage: disasm {} <addr> [<count>]", parts[1])));
    };
    let count = if parts.len() >= at + 2 { parse_hex(&parts[at + 1])? } else { 10 };
    let hints_json = format!("[{}]", hints.iter().map(|h| emusen_session_service::quoted(h)).collect::<Vec<_>>().join(","));
    let listing = instructions(&session.disassemble(0, Some(&space), address as u32, count as u32, Some(&hints_json))?);
    if listing.is_empty() {
        return Ok(Answer::ok(format!("{} target has no disassembler, or nothing was returned.", core_name(&session)?)));
    }
    let mut out = String::new();
    if !hints.is_empty() {
        out += &format!("  ; decoding with {}\n", to_lower(&hints.join(" ")));
    }
    for instruction in &listing {
        if let Some((name, comment)) = label_at(&session, instruction.address) {
            out += &format!("{name}:{}\n", comment.map_or(String::new(), |c| format!("  ; {c}")));
        }
        let bytes = instruction.bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
        let target = instruction.reference.as_ref().and_then(|(_, t)| label_at(&session, *t)).map_or(String::new(), |(name, _)| format!("  ; <{name}>"));
        let line = format!("  {}: {} {} {}{target}", hex(instruction.address, 6), pad_right(&bytes, 9), instruction.mnemonic, instruction.operands);
        out += trim_end(&line);
        out.push('\n');
    }
    Ok(Answer::ok(trim_end(&out)))
}

/// `callers`, `writers` and `readers`: the instructions of a scan whose static reference is of `kind` and lands on the address.
pub fn scan(call: &Call, parts: &[String], command: &str, kind: &str) -> Run {
    let session = session(call)?;
    if parts.len() < 2 {
        return Ok(Answer::ok(format!("Usage: {command} <addr> [<scanstart> <scanlen>]")));
    }
    let target = parse_hex(&parts[1])?;
    let (start, length) = if parts.len() >= 4 { (parse_hex(&parts[2])?, parse_hex(&parts[3])?) } else { ((target & 0xFF0000) | 0x8000, 0x8000) };
    let listing = instructions(&session.disassemble(0, Some("CpuBus"), start as u32, length as u32, None)?);
    let end = start.wrapping_add(length);
    let mut matches = Vec::new();
    for instruction in &listing {
        if instruction.address >= end {
            break;
        }
        if instruction.reference.as_ref().is_some_and(|(k, t)| k == kind && *t == target) {
            matches.push(format!("  ${}: {} {}", hex(instruction.address, 6), instruction.mnemonic, instruction.operands));
        }
    }
    if matches.is_empty() {
        let verb = match kind {
            "call" => "calls of",
            "write" => "writes to",
            _ => "reads of",
        };
        return Ok(Answer::ok(format!("No statically-resolvable {verb} ${} found in ${}-${}.", hex(target, 6), hex(start, 6), hex(end.wrapping_sub(1), 6))));
    }
    Ok(Answer::ok(matches.join("\n")))
}
