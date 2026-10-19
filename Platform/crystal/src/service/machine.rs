//! The session, run control, memory, processors and disassembly groups, and the states, logs, events and games the
//! table also answers: the C# `SessionService.Machine` and its neighbours (EmuSen_DianaSh.md §9.2, §13.15).

use emusen_core_host::Machine;
use emusen_crystal_debug::breakpoints::{BreakpointRegistry, CallKind};
use emusen_crystal_debug::context::Set;
use emusen_crystal_debug::Debugger;
use emusen_dotnet::json::Value;
use emusen_session_service::abi::{run, status};

use super::args::{self, Raw};
use super::{Answer, Call, int32, not_supported, processor_of, refused, registers_json, space};
use crate::engine::Engine;
use crate::json::Json;

/// The engine's debugger and its machine.
pub(crate) fn dm(e: &mut Engine) -> (&mut Debugger, Option<&mut Machine>) {
    (&mut e.debug.d, e.machine.as_mut())
}

/// The sessions this provider serves: itself, the current one, `Sessions`.
pub fn sessions(c: &Call) -> Answer<Vec<u8>> {
    let name = c.s.engine.lock().unwrap_or_else(|p| p.into_inner()).name.clone();
    let mut j = Json::new();
    j.begin_array().begin_object().int("id", c.s.id as i64).str("engine", &name).bool("current", true).end_object().end_array();
    Ok(j.finish().into_bytes())
}

/// A session's capabilities, named for the groups that answer NOT_SUPPORTED without them, `Capabilities`.
fn capabilities(e: &mut Engine) -> Vec<&'static str> {
    let (d, m) = dm(e);
    let mut caps = vec!["breakpoints", "watches", "frame_logs", "cheats"];
    let halts = d.halts();
    let coprocessor = d.coprocessor().is_some();
    if coprocessor {
        caps.push("coprocessor_breakpoints");
    }
    if halts {
        caps.push("coverage");
    }
    if halts && coprocessor {
        caps.push("coprocessor_coverage");
    }
    if halts {
        caps.push("call_stack");
    }
    caps.push("labels");
    caps.push("freezes");
    if !d.processors().is_empty() {
        caps.push("expressions");
    }
    if m.is_some_and(|m| !d.dma_channels(m).is_empty()) {
        caps.push("dma_channels");
    }
    if !d.interrupt_vectors().is_empty() {
        caps.push("vectors");
    }
    caps.push("states");
    caps
}

/// `{"id","engine","frame","max_sprites","tilemap_entry_stride","capabilities"}`, `Info`.
pub fn info(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let caps = capabilities(e);
        let mut j = Json::new();
        j.begin_object()
            .int("id", c.s.id as i64)
            .str("engine", &e.name)
            .int("frame", e.total_frames())
            .int("max_sprites", e.debug.d.max_sprites() as i64)
            .int("tilemap_entry_stride", e.debug.d.tilemap_entry_stride() as i64)
            .strs("capabilities", caps)
            .end_object();
        Ok(j.finish().into_bytes())
    })
}

/// The core's own words for its state, `GetSummaryText`.
pub fn summary(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let version = e.info.get("version").and_then(emusen_native::json::Value::as_str).unwrap_or("").to_string();
        let display = e.display_name.clone();
        let (d, m) = dm(e);
        Ok(match m {
            Some(m) => d.summary(m, &display, &version),
            None => format!("{display} {version}, no game"),
        }
        .into_bytes())
    })
}

/// `Halting`: the registry of a processor that can halt.
fn halting<'d>(d: &'d mut Debugger, core: &str, processor: u32) -> Answer<&'d mut BreakpointRegistry> {
    let p = processor_of(d, core, processor)?;
    if let Some(i) = p {
        let cpus = d.processors();
        if !cpus[i].can_halt {
            return Err(not_supported(format!("{} cannot be halted by this core, so it cannot be stepped.", cpus[i].name)));
        }
    }
    d.registry_of(p).ok_or_else(|| not_supported("This cartridge has no coprocessor to step."))
}

/// What run control arms of the debugger, `OwnRunControl`, the request kept for `events_drain`.
fn own_run_control(c: &Call, e: &mut Engine, op: u32, processor: u32, argument: i64) -> Answer<i64> {
    c.target()?;
    let core = e.name.clone();
    let d = &mut e.debug.d;
    let mut request = "step";
    let mut result = 0;
    match op {
        run::PAUSE => return Err(not_supported("This provider has no frame loop to pause; pausing is the frontend's (emusen.frontend@1).")),
        run::RESUME => request = "resume",
        run::STEP => {
            if argument < 1 {
                return Err(refused(status::BAD_ARGUMENT, "Step count must be at least 1."));
            }
            let registry = halting(d, &core, processor)?;
            registry.arm_step(int32(argument, "count")?);
        }
        run::STEP_OVER | run::STEP_OUT => {
            let p = processor_of(d, &core, processor)?;
            halting(d, &core, processor)?;
            let has_stack = d.halts() && p.is_none_or(|i| i == 0);
            let Some(depth) = (if has_stack { d.call_stack().map(|s| s.depth()) } else { None }) else {
                return Err(not_supported("This core does not report a call stack, so stepping over or out has no depth to measure against."));
            };
            let depth = if op == run::STEP_OVER { depth } else { depth - 1 };
            if depth < 0 {
                return Err(refused(status::BAD_ARGUMENT, "Already at the outermost frame - nothing to step out of."));
            }
            if !halting(d, &core, processor)?.arm_step_to_depth(depth) {
                return Err(not_supported("This core's breakpoints are not told its call stack, so they cannot step to a depth."));
            }
        }
        run::TO_ADDRESS => {
            let registry = halting(d, &core, processor)?;
            if !(i32::MIN as i64..=u32::MAX as i64).contains(&argument) {
                return Err(refused(status::BAD_ARGUMENT, format!("{argument} is not a 32-bit address")));
            }
            let at = argument as u32 as i32;
            result = registry.add_breakpoint(at, at, None, None) as i64;
        }
        run::TO_INTERRUPT => {
            let Some(kind) = CallKind::from_number(argument).filter(|_| (1..=4).contains(&argument)) else {
                return Err(not_supported(format!("no interrupt kind {argument}")));
            };
            if !halting(d, &core, processor)?.arm_run_to_interrupt(kind) {
                return Err(refused(status::BAD_ARGUMENT, "This core cannot run to that interrupt."));
            }
        }
        run::TO_SCANLINE => {
            let registry = halting(d, &core, processor)?;
            registry.arm_run_to_scanline(int32(argument, "scanline")?);
        }
        run::TO_FRAME => halting(d, &core, processor)?.arm_run_to_frame(argument),
        _ => return Err(not_supported(format!("no run-control op {op}"))),
    }
    c.s.provider.raise(request);
    Ok(result)
}

/// Run control as the runtime answers it, `ManagedSession.RunControl`: pause and resume hold the session, the rest are
/// armed by the provider and let the next frame run; a `run` event says which.
pub fn run_control(c: &Call, op: u32, processor: u32, argument: i64) -> Answer<i64> {
    c.change(|e, run| {
        let (said, result) = match op {
            run::PAUSE => {
                run.paused = true;
                ("paused", 0)
            }
            run::RESUME => {
                let result = own_run_control(c, e, op, processor, argument)?;
                run.paused = false;
                run.halted = false;
                ("resumed", result)
            }
            run::STEP | run::STEP_OVER | run::STEP_OUT => {
                let result = own_run_control(c, e, op, processor, argument)?;
                run.run_requested = true;
                ("stepped", result)
            }
            _ => {
                let result = own_run_control(c, e, op, processor, argument)?;
                run.paused = false;
                run.run_requested = true;
                ("resumed", result)
            }
        };
        c.s.events.raise(c.s.id, crate::abi::EVENT_RUN, "run", |j| {
            j.str("op", said);
        });
        Ok(result)
    })
}

/// `{"requested","last_break",…}`, `HaltState`.
pub fn halt_state(c: &Call, processor: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let core = e.name.clone();
        let p = processor_of(&e.debug.d, &core, processor)?;
        let requested = c.s.provider.last_request();
        let b = e.debug.d.registry_of(p).ok_or_else(|| not_supported("This core has no breakpoints on that processor."))?;
        let mut j = Json::new();
        j.begin_object()
            .str("requested", &requested)
            .str("last_break", &b.last_break_reason)
            .str("last_data_break", b.last_data_break())
            .str("last_event_break", b.last_event_break())
            .bool("single_step", b.single_step_armed());
        match b.step_depth_target() {
            Some(depth) => j.int("step_depth", depth as i64),
            None => j.null("step_depth"),
        };
        j.bool("pending", b.has_pending_break()).end_object();
        Ok(j.finish().into_bytes())
    })
}

/// `[{"name","size","writable","side_effects"}]`.
pub fn spaces(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        let mut j = Json::new();
        j.begin_array();
        for s in m.map(|m| d.spaces(m)).unwrap_or_default() {
            j.begin_object().str("name", &s.name).int("size", s.size as i64).bool("writable", s.writable).bool("side_effects", s.side_effects).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

/// `len` bytes from `address` of a space into `into`, each read as the space reads any address.
pub fn space_read(c: &Call, name: Raw, address: u32, into: &mut [u8]) -> Answer<()> {
    let name = name.text()?;
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        let canonical = space(d, m.as_deref(), name)?;
        let Some(m) = m else { return Ok(()) };
        let at = address as i32;
        for (i, b) in into.iter_mut().enumerate() {
            *b = d.space_read(m, &canonical, at.wrapping_add(i as i32));
        }
        Ok(())
    })
}

/// `data` written at `address`, refused for a space that cannot be written.
pub fn space_write(c: &Call, name: Raw, address: u32, data: Raw) -> Answer<i64> {
    c.change(|e, _| {
        let (name, data) = (name.text()?, data.bytes()?);
        c.target()?;
        let (d, m) = dm(e);
        let canonical = space(d, m.as_deref(), name)?;
        let Some(m) = m else { return Ok(0) };
        if !d.spaces(m).iter().any(|s| s.name == canonical && s.writable) {
            return Err(refused(status::READ_ONLY, format!("{canonical} is read-only on this core.")));
        }
        let at = address as i32;
        for (i, b) in data.iter().enumerate() {
            d.space_write(m, &canonical, at.wrapping_add(i as i32), *b);
        }
        Ok(0)
    })
}

/// `{"space","offset","addressable"}`; ABSENT where the core does not say.
pub fn resolve_physical(c: &Call, address: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        let absent = || refused(status::ABSENT, "This core does not report what that address decodes to.");
        let (space, offset, addressable) = m.and_then(|m| d.resolve_physical(m, address as i32)).ok_or_else(absent)?;
        let mut j = Json::new();
        j.begin_object().str("space", &space).int("offset", offset as i64).bool("addressable", addressable).end_object();
        Ok(j.finish().into_bytes())
    })
}

/// `[{"index","name","description","can_halt","code_space","coverage","call_stack","registers","writable_registers","pc"}]`.
pub fn processors(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (d, m) = dm(e);
        let mut j = Json::new();
        j.begin_array();
        for (i, p) in d.processors().iter().enumerate() {
            j.begin_object().int("index", i as i64).str("name", &p.name).str("description", &p.description).bool("can_halt", p.can_halt);
            j.str_or_null("code_space", p.code_space.as_deref()).bool("coverage", p.coverage).bool("call_stack", p.call_stack).bool("registers", true).bool("writable_registers", false);
            match (&m, p.has_pc) {
                (Some(m), true) => j.int("pc", d.program_counter(m, i) as u32 as i64),
                _ => j.null("pc"),
            };
            j.end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

/// A processor's registers as its snapshot shows them, `[{"name","value","bits"}]`.
pub fn registers(c: &Call, processor: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let core = e.name.clone();
        let d = &e.debug.d;
        let set = Set::Processor(processor_of(d, &core, processor)?.unwrap_or(0));
        let mut j = Json::new();
        registers_json(&mut j, &d.registers(set));
        Ok(j.finish().into_bytes())
    })
}

/// A register written; no generic engine writes one, so a name it lists is refused as one it cannot write.
pub fn set_register(c: &Call, processor: u32, name: Raw, value: u64) -> Answer<i64> {
    c.change(|e, _| {
        let name = name.text()?;
        c.target()?;
        let core = e.name.clone();
        let d = &e.debug.d;
        let cpus = d.processors();
        if cpus.is_empty() {
            return Err(not_supported(format!("{core} publishes no debug CPUs, so it has no writable registers.")));
        }
        let Some(i) = processor_of(d, &core, processor)? else {
            return Err(refused(status::FAULT, "Object reference not set to an instance of an object."));
        };
        let cpu = &cpus[i];
        let known = d.registers(Set::Processor(i));
        if !known.iter().any(|r| name.is_some_and(|n| emusen_dotnet::text::equals_ignore_case(&r.name, n))) {
            return Err(refused(status::NO_SUCH_ID, format!("{} has no register named '{}'.", cpu.name, name.unwrap_or(""))));
        }
        let _ = value;
        Err(not_supported(format!("{} reports its registers but this core cannot write them.", cpu.name)))
    })
}

/// `[{"address","bytes","mnemonic","operands","reference"}]` over a space, the processor's code space when none is named.
pub fn disassemble(c: &Call, processor: u32, space_name: Raw, address: u32, count: u32, hints: Raw) -> Answer<Vec<u8>> {
    let (space_name, hints) = (space_name.text()?, hints.text()?);
    c.target()?;
    c.read(|e, _| {
        let core = e.name.clone();
        let name = match space_name {
            Some(s) => s.to_string(),
            None => match processor_of(&e.debug.d, &core, processor)? {
                Some(i) => e.debug.d.processors()[i].code_space.clone().unwrap_or_else(|| "CpuBus".to_string()),
                None => "CpuBus".to_string(),
            },
        };
        if let Some(h) = hints {
            let parsed = args::parse(h.as_bytes())?;
            let Value::Array(items) = &parsed else { return Err(refused(status::BAD_ARGUMENT, "hints must be a JSON array of strings")) };
            for item in items {
                args::element_string(item)?;
            }
        }
        let count = int32(count as i64, "count")?;
        let (d, m) = dm(e);
        let Some(m) = m else { return Ok(b"[]".to_vec()) };
        let listed = d.disassemble(m, &name, address as i32, count);
        let mut j = Json::new();
        j.begin_array();
        for i in &listed {
            j.begin_object().u64("address", i.address as u32 as u64).key_array("bytes");
            for b in &i.bytes {
                j.int_value(*b as i64);
            }
            j.end_array().str("mnemonic", &i.mnemonic).str("operands", &i.operands);
            match d.reference(i.address) {
                Some((kind, target)) => {
                    let word = match kind {
                        emusen_systems::debug::Reference::Call => "call",
                        emusen_systems::debug::Reference::Write => "write",
                        _ => "read",
                    };
                    j.key_object("reference").str("kind", word).u64("target", target as u32 as u64).end_object();
                }
                None => {
                    j.null("reference");
                }
            }
            j.end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

/// The emulator's log switch, the process's, `DebugLogging.MasterEnabled`.
pub fn set_log(on: bool) {
    emusen_crystal_debug::watches::set_logging(on);
}

/// The switch set through a table, a change of its session.
pub fn log_switch(c: &Call, on: bool) -> Answer<i64> {
    c.change(|_, _| {
        set_log(on);
        Ok(0)
    })
}

pub fn log_status() -> i64 {
    emusen_crystal_debug::watches::logging() as i64
}

/// Run control's requests not yet drained, of the kinds asked, and how many the answer holds, `EventsPeek`.
pub fn events_peek(c: &Call, kinds: u32) -> Answer<(Vec<u8>, usize)> {
    c.target()?;
    if kinds & !emusen_session_service::abi::event::RUN != 0 {
        return Err(not_supported(format!("unknown event kinds 0x{kinds:X}")));
    }
    let requests = c.s.provider.requests.lock().unwrap_or_else(|p| p.into_inner());
    let mut j = Json::new();
    j.begin_array();
    for op in &requests.0 {
        j.begin_object().str("kind", "run").str("op", op).end_object();
    }
    j.end_array();
    Ok((j.finish().into_bytes(), requests.0.len()))
}

/// The first `count` requests taken, once a buffer has held them all, `EventsTake`.
pub fn events_take(c: &Call, count: usize) {
    let mut requests = c.s.provider.requests.lock().unwrap_or_else(|p| p.into_inner());
    let n = count.min(requests.0.len());
    requests.0.drain(..n);
}

/// What the runtime can open: `systems`, `Games`.
pub fn games(c: &Call) -> Answer<Vec<u8>> {
    Ok(c.rt.systems_json().into_bytes())
}

/// A session opened as this one was, its id: a game by path or a console by name, `Open`.
pub fn open(c: &Call, path: Raw, console: Raw, engine: Raw, game: bool) -> Answer<i64> {
    if c.read_only {
        return Err(refused(status::READ_ONLY, super::READ_ONLY_WORDS));
    }
    let (path, console) = if game {
        (Some(path.text()?.ok_or_else(|| refused(status::BAD_ARGUMENT, "open_game needs the game's path"))?), None)
    } else {
        (None, Some(console.text()?.ok_or_else(|| refused(status::BAD_ARGUMENT, "open_console needs the console's name"))?))
    };
    let engine = engine.text()?;
    c.target()?;
    let asked = crate::open::Asked { path, console, engine, opened: c.s.opened };
    c.rt.open(&asked).map(|id| id as i64)
}

/// A refusal of a state as the twins give it: the state's own kept, anything else a fault in its words.
fn state_refusal((status, words): (i32, String)) -> (i32, String) {
    if status == emusen_session_service::abi::status::ABSENT || status == crate::abi::status::STATE_REFUSED {
        return (status, words);
    }
    let bare = match words.split_once(": ") {
        Some((kind, rest)) if kind.ends_with("Exception") && !kind.contains(' ') => rest.to_string(),
        _ => words,
    };
    (emusen_session_service::abi::status::FAULT, bare)
}

/// The state's path: the one given, or the session's default, slot 1, `StatePath`.
fn state_path(c: &Call, given: Option<&str>) -> String {
    given.map(str::to_string).unwrap_or_else(|| c.s.state_path(&emusen_galaxia::tree::tree(), crate::saving::DEFAULT_SLOT))
}

/// A state written at a path, with its record and no picture, `StateSave`.
pub fn state_save(c: &Call, path: Raw) -> Answer<i64> {
    c.rt.with_records(|records| {
        c.change(|e, _| {
            let path = path.text()?;
            c.target()?;
            let at = state_path(c, path);
            c.s.save_in(e, &at, records, false).map_err(state_refusal)?;
            Ok(0)
        })
    })
}

/// A state loaded from a path, `StateLoad`.
pub fn state_load(c: &Call, path: Raw) -> Answer<i64> {
    c.rt.with_records(|records| {
        c.rt.with_ctx(|ctx| {
            c.change(|e, run| {
                let path = path.text()?;
                c.target()?;
                let at = state_path(c, path);
                c.s.load_in(e, run, &at, records, ctx).map_err(state_refusal)?;
                Ok(0)
            })
        })
    })
}

/// Where a state is saved when no path is given.
pub fn state_default_path(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    Ok(state_path(c, None).into_bytes())
}
