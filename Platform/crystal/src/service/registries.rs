//! The breakpoint, watch, frame-log, coverage, profile, counter, call-stack, freeze and label groups: the C#
//! `SessionService.Debugger` over the debugger's registries (EmuSen_DianaSh.md §9.2).

use emusen_crystal_debug::Debugger;
use emusen_crystal_debug::breakpoints::BreakpointRegistry;
use emusen_crystal_debug::coverage::{CallStackRegistry, CoverageRegistry};
use emusen_crystal_debug::labels::LabelRegistry;
use emusen_crystal_debug::watches::WatchKind;
use emusen_dotnet::json::Value;
use emusen_session_service::abi::{coverage as cov, profile, status};

use super::args::{self, Raw};
use super::machine::dm;
use super::{Answer, Call, READ_ONLY_WORDS, int32, no_such_id, not_supported, processor_of, refused, space};
use crate::engine::Engine;
use crate::json::Json;

/// `Breakpoints(target, processor)`: the registry a processor argument reaches.
fn breakpoints<'d>(d: &'d mut Debugger, core: &str, processor: u32) -> Answer<&'d mut BreakpointRegistry> {
    let p = processor_of(d, core, processor)?;
    d.registry_of(p).ok_or_else(|| not_supported("This core has no breakpoints on that processor."))
}

/// A registry's breakpoints answered as the C# `SessionService.BpList` writes them.
fn list(b: &BreakpointRegistry) -> Vec<u8> {
    let mut j = Json::new();
    j.begin_object().key_array("code");
    for x in b.breakpoints() {
        j.begin_object().int("id", x.id as i64).u64("address", x.address as u32 as u64).u64("end", x.end as u32 as u64).bool("enabled", x.enabled).int("hits", x.hits);
        j.str_or_null("condition", x.condition.as_deref()).str_or_null("log", x.log.as_deref()).end_object();
    }
    j.end_array().key_array("data");
    for x in b.data_breakpoints() {
        j.begin_object().int("id", x.id as i64).str("space", &x.space).u64("address", x.address as u32 as u64).u64("end", x.end as u32 as u64).int("value", x.value as i64);
        j.bool("on_read", x.on_read).bool("changed_only", x.changed_only).bool("enabled", x.enabled).int("hits", x.hits);
        j.str_or_null("condition", x.condition.as_deref()).str_or_null("log", x.log.as_deref()).end_object();
    }
    j.end_array().key_array("forbid");
    for x in b.forbid_ranges() {
        j.begin_object().int("id", x.id as i64).u64("address", x.address as u32 as u64).u64("end", x.end as u32 as u64).bool("enabled", x.enabled).end_object();
    }
    j.end_array().int("depth_guard", b.depth_guard() as i64);
    if b.uninitialized_armed() {
        j.key_object("uninitialized").str("space", b.uninitialized_space()).end_object();
    } else {
        j.null("uninitialized");
    }
    j.end_object();
    j.finish().into_bytes()
}

/// A code breakpoint, its condition and log checked first: PARSE with the evaluator's words, nothing added.
pub fn bp_add(c: &Call, processor: u32, spec: Raw) -> Answer<i64> {
    c.change(|e, _| {
        let spec = args::parse(spec.bytes()?)?;
        c.target()?;
        let core = e.name.clone();
        let (d, m) = dm(e);
        let p = processor_of(d, &core, processor)?;
        breakpoints(d, &core, processor)?;
        let address = args::address(&spec, "address")?;
        let end = args::optional_address(&spec, "end")?.unwrap_or(address);
        let (condition, log) = (args::optional_string(&spec, "condition")?, args::optional_string(&spec, "log")?);
        let Some(m) = m else { return Err(refused(status::NO_SESSION, "the session has no machine")) };
        d.add_breakpoint(m, p, address, end, condition.as_deref(), log.as_deref()).map(|id| id as i64).map_err(|why| refused(status::PARSE, why))
    })
}

/// A data breakpoint on a space: a value of -1 is any.
pub fn bp_add_data(c: &Call, processor: u32, spec: Raw) -> Answer<i64> {
    c.change(|e, _| {
        let spec = args::parse(spec.bytes()?)?;
        c.target()?;
        let core = e.name.clone();
        let (d, m) = dm(e);
        let p = processor_of(d, &core, processor)?;
        breakpoints(d, &core, processor)?;
        let name = args::string(&spec, "space")?;
        let named = space(d, m.as_deref(), Some(&name))?;
        let address = args::address(&spec, "address")?;
        let end = args::optional_address(&spec, "end")?.unwrap_or(address);
        let value = int32(args::optional_int(&spec, "value")?.unwrap_or(-1), "value")?;
        let on_read = args::optional_bool(&spec, "on_read")?.unwrap_or(false);
        let changed_only = args::optional_bool(&spec, "changed_only")?.unwrap_or(false);
        let (condition, log) = (args::optional_string(&spec, "condition")?, args::optional_string(&spec, "log")?);
        let Some(m) = m else { return Err(refused(status::NO_SESSION, "the session has no machine")) };
        d.add_data_breakpoint(m, p, &named, address, end, value, on_read, changed_only, condition.as_deref(), log.as_deref()).map(|id| id as i64).map_err(|why| refused(status::PARSE, why))
    })
}

pub fn bp_set_enabled(c: &Call, processor: u32, id: u32, enabled: bool) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        let core = e.name.clone();
        if !breakpoints(&mut e.debug.d, &core, processor)?.set_enabled(id as i32, enabled) {
            return Err(no_such_id("breakpoint", id as i64));
        }
        Ok(0)
    })
}

pub fn bp_remove(c: &Call, processor: u32, id: u32) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        let core = e.name.clone();
        if !breakpoints(&mut e.debug.d, &core, processor)?.remove(id as i32) {
            return Err(no_such_id("breakpoint", id as i64));
        }
        Ok(0)
    })
}

pub fn bp_list(c: &Call, processor: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let core = e.name.clone();
        Ok(list(breakpoints(&mut e.debug.d, &core, processor)?))
    })
}

/// `{"recorded","entries"}`, the last `count` entries.
pub fn bp_log(c: &Call, processor: u32, count: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let core = e.name.clone();
        let b = breakpoints(&mut e.debug.d, &core, processor)?;
        let entries = b.log_tail(int32(count as i64, "count")?);
        let mut j = Json::new();
        j.begin_object().int("recorded", b.log_recorded()).key_array("entries");
        for x in entries {
            j.begin_object().int("id", x.id as i64).u64("address", x.address as u32 as u64).str("text", &x.text).end_object();
        }
        j.end_array().end_object();
        Ok(j.finish().into_bytes())
    })
}

pub fn bp_log_clear(c: &Call, processor: u32) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        let core = e.name.clone();
        breakpoints(&mut e.debug.d, &core, processor)?.clear_log();
        Ok(0)
    })
}

/// A hardware condition armed or disarmed; the generic engine detects none.
pub fn bp_arm_condition(c: &Call, processor: u32, name: Raw, flags: u32) -> Answer<i64> {
    use emusen_session_service::abi::condition::{DISARM, LOG_ONLY};
    c.change(|e, _| {
        let name = name.text()?;
        c.target()?;
        let core = e.name.clone();
        let b = breakpoints(&mut e.debug.d, &core, processor)?;
        if flags & !(LOG_ONLY | DISARM) != 0 {
            return Err(not_supported(format!("unknown condition flags 0x{flags:X}")));
        }
        if flags & DISARM != 0 {
            if !name.is_some_and(|n| b.disarm_condition(n)) {
                return Err(refused(status::NO_SUCH_ID, format!("'{}' is not armed.", name.unwrap_or(""))));
            }
            return Ok(0);
        }
        Err(refused(status::NO_SUCH_ID, format!("This core does not detect '{}'.", name.unwrap_or(""))))
    })
}

/// `{"available","armed"}`: none available on the generic engine.
pub fn bp_conditions(c: &Call, processor: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let core = e.name.clone();
        let b = breakpoints(&mut e.debug.d, &core, processor)?;
        let mut j = Json::new();
        j.begin_object().key_array("available").end_array().key_array("armed");
        for x in b.conditions() {
            j.begin_object().str("name", &x.name).bool("log_only", x.log_only).bool("enabled", x.enabled).int("hits", x.hits).end_object();
        }
        j.end_array().end_object();
        Ok(j.finish().into_bytes())
    })
}

pub fn bp_depth_guard(c: &Call, processor: u32, depth: i32) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        let core = e.name.clone();
        let b = breakpoints(&mut e.debug.d, &core, processor)?;
        if depth < 0 {
            b.disarm_depth_guard();
        } else if !b.arm_depth_guard(depth) {
            return Err(refused(status::BAD_ARGUMENT, format!("a depth guard of {depth} is refused")));
        }
        Ok(0)
    })
}

pub fn bp_forbid_range(c: &Call, processor: u32, address: u32, end: u32) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        let core = e.name.clone();
        Ok(breakpoints(&mut e.debug.d, &core, processor)?.add_forbid_range(address as i32, end as i32) as i64)
    })
}

/// A halt on a read of a byte not yet written; an absent space disarms it.
pub fn bp_uninitialized(c: &Call, processor: u32, name: Raw, size: u32) -> Answer<i64> {
    c.change(|e, _| {
        let name = name.text()?;
        c.target()?;
        let core = e.name.clone();
        let (d, m) = dm(e);
        breakpoints(d, &core, processor)?;
        let Some(name) = name else {
            breakpoints(d, &core, processor)?.disarm_uninitialized();
            return Ok(0);
        };
        let named = space(d, m.as_deref(), Some(name))?;
        let size = int32(size as i64, "size")?;
        if !breakpoints(d, &core, processor)?.arm_uninitialized(&named, size) {
            return Err(refused(status::BAD_ARGUMENT, "the space's size must be above 0"));
        }
        Ok(0)
    })
}

/// A watch of a space's bytes, of a kind (write, read or both).
pub fn watch_add(c: &Call, name: Raw, address: u32, length: u32, kind: u32) -> Answer<i64> {
    c.change(|e, _| {
        let name = name.text()?;
        c.target()?;
        let Some(kind) = WatchKind::from_number(kind) else { return Err(not_supported(format!("no watch kind {kind}"))) };
        if length < 1 {
            return Err(refused(status::BAD_ARGUMENT, "a watch covers at least one byte"));
        }
        let (d, m) = dm(e);
        let named = space(d, m.as_deref(), name)?;
        let length = int32(length as i64, "length")?;
        Ok(d.watches.add(&named, address as i32, length, kind) as i64)
    })
}

/// `Watch(target, id)`: a watch the registry holds.
fn watched(e: &Engine, id: u32) -> Answer<()> {
    if e.debug.d.watches.watches().iter().any(|w| w.id as i64 == id as i64) { Ok(()) } else { Err(no_such_id("watch", id as i64)) }
}

pub fn watch_remove(c: &Call, id: u32) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        if !e.debug.d.watches.remove(id as i32) {
            return Err(no_such_id("watch", id as i64));
        }
        Ok(0)
    })
}

pub fn watch_list(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let mut j = Json::new();
        j.begin_array();
        for w in e.debug.d.watches.watches() {
            j.begin_object().int("id", w.id as i64).str("space", &w.space).u64("address", w.address as u32 as u64).int("length", w.length as i64).int("kind", w.kind as i64).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

pub fn watch_events(c: &Call, id: u32, count: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        watched(e, id)?;
        let events = e.debug.d.watches.events(id as i32, int32(count as i64, "count")?);
        let mut j = Json::new();
        j.begin_array();
        for x in events {
            j.begin_object().int("sequence", x.sequence).int("kind", x.kind as i64).u64("address", x.address as u32 as u64).int("value", x.value as i64).str("context", &x.context).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

pub fn watch_summary(c: &Call, id: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        watched(e, id)?;
        let w = &e.debug.d.watches;
        let (total, retained) = w.counts(id as i32);
        let mut j = Json::new();
        j.begin_object().int("total", total).int("retained", retained as i64).key_array("sites");
        for (context, count) in w.sites(id as i32) {
            j.begin_object().str("context", &context).int("count", count).end_object();
        }
        j.end_array().end_object();
        Ok(j.finish().into_bytes())
    })
}

pub fn watch_clear(c: &Call, id: u32) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        watched(e, id)?;
        e.debug.d.watches.clear_events(id as i32);
        Ok(0)
    })
}

/// A value `width` bytes wide sampled once a frame.
pub fn framelog_add(c: &Call, name: Raw, address: u32, width: u32) -> Answer<i64> {
    c.change(|e, _| {
        let name = name.text()?;
        c.target()?;
        if width < 1 {
            return Err(refused(status::BAD_ARGUMENT, "a frame log's width is at least one byte"));
        }
        let (d, m) = dm(e);
        let named = space(d, m.as_deref(), name)?;
        Ok(d.frame_log.add(&named, address as i32, width as i32) as i64)
    })
}

fn logged(e: &Engine, id: u32) -> Answer<()> {
    if e.debug.d.frame_log.entries().iter().any(|x| x.id as i64 == id as i64) { Ok(()) } else { Err(no_such_id("frame-log entry", id as i64)) }
}

pub fn framelog_remove(c: &Call, id: u32) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        if !e.debug.d.frame_log.remove(id as i32) {
            return Err(no_such_id("frame-log entry", id as i64));
        }
        Ok(0)
    })
}

pub fn framelog_list(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let mut j = Json::new();
        j.begin_array();
        for x in e.debug.d.frame_log.entries() {
            j.begin_object().int("id", x.id as i64).str("space", &x.space).u64("address", x.address as u32 as u64).int("width", x.width as i64).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

pub fn framelog_samples(c: &Call, id: u32, count: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        logged(e, id)?;
        let samples = e.debug.d.frame_log.samples(id as i32, int32(count as i64, "count")?);
        let mut j = Json::new();
        j.begin_array();
        for s in samples {
            j.begin_object().int("frame", s.frame).int("value", s.value).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

pub fn framelog_clear(c: &Call, id: u32) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        logged(e, id)?;
        e.debug.d.frame_log.clear_samples(id as i32);
        Ok(0)
    })
}

/// The coverage a processor argument reaches: the main processor's for none named.
fn coverage<'d>(d: &'d mut Debugger, core: &str, processor: u32) -> Answer<&'d mut CoverageRegistry> {
    let p = processor_of(d, core, processor)?;
    d.coverage(p.unwrap_or(0)).ok_or_else(|| not_supported("Coverage is not available on this core."))
}

/// `Range(args)`: an address, a length and a limit, `int.MaxValue` where none is given.
fn range(a: &Value) -> Answer<(i32, i32, i32)> {
    let address = args::address(a, "address")?;
    let length = int32(args::int(a, "length")?, "length")?;
    let limit = int32(args::optional_int(a, "limit")?.unwrap_or(i32::MAX as i64), "limit")?;
    Ok((address, length, limit))
}

fn addresses(list: &[i32]) -> Vec<u8> {
    let mut j = Json::new();
    j.begin_array();
    for a in list {
        j.u64_value(*a as u32 as u64);
    }
    j.end_array();
    j.finish().into_bytes()
}

/// `EMUSEN_SESSION_COVERAGE_*`, under the gate whether it reads or changes, as the C#'s export holds it.
pub fn coverage_op(c: &Call, processor: u32, op: u32, raw: Raw) -> Answer<Vec<u8>> {
    c.gated(|e, _| {
        let bytes = raw.bytes()?;
        c.target()?;
        let core = e.name.clone();
        let r = coverage(&mut e.debug.d, &core, processor)?;
        if op > cov::MERGE {
            return Err(not_supported(format!("no coverage op {op}")));
        }
        let changes = matches!(op, cov::ARM | cov::DISARM | cov::CLEAR | cov::MARK | cov::CLEAR_MARK | cov::IMPORT | cov::MERGE);
        if c.read_only && changes {
            return Err(refused(status::READ_ONLY, READ_ONLY_WORDS));
        }
        let a = if matches!(op, cov::IMPORT | cov::MERGE | cov::EXPORT) { Value::Null } else { args::parse(bytes)? };
        Ok(match op {
            cov::ARM => {
                r.arm();
                Vec::new()
            }
            cov::DISARM => {
                r.disarm();
                Vec::new()
            }
            cov::CLEAR => {
                r.clear();
                Vec::new()
            }
            cov::MARK => {
                r.mark();
                Vec::new()
            }
            cov::CLEAR_MARK => {
                r.clear_mark();
                Vec::new()
            }
            cov::STATUS => {
                let mut j = Json::new();
                j.begin_object().bool("armed", r.is_armed()).int("instructions", r.instructions()).bool("has_mark", r.has_mark()).int("entry_points", r.entry_point_count() as i64).end_object();
                j.finish().into_bytes()
            }
            cov::QUERY => {
                let (address, length, limit) = range(&a)?;
                let (executed, total, fresh) = r.statistics(address, length);
                let listed = r.executed_addresses(address, length, limit);
                let mut j = Json::new();
                j.begin_object().int("executed", executed as i64).int("total", total as i64).int("new_since_mark", fresh as i64).key_array("addresses");
                for x in listed {
                    j.u64_value(x as u32 as u64);
                }
                j.end_array().end_object();
                j.finish().into_bytes()
            }
            cov::NEW_SINCE_MARK => {
                let (address, length, limit) = range(&a)?;
                addresses(&r.new_since_mark(address, length, limit))
            }
            cov::ENTRY_POINTS => {
                let limit = int32(args::optional_int(&a, "limit")?.unwrap_or(i32::MAX as i64), "limit")?;
                let mut j = Json::new();
                j.begin_array();
                for (address, kind, entries) in r.entry_points(limit) {
                    j.begin_object().u64("address", address as u32 as u64).int("kind", kind as i64).int("entries", entries).end_object();
                }
                j.end_array();
                j.finish().into_bytes()
            }
            cov::EXPORT => r.export(),
            cov::IMPORT => {
                if !r.import(bytes) {
                    return Err(refused(status::BAD_ARGUMENT, "That is not a coverage map this core can read."));
                }
                Vec::new()
            }
            _ => {
                if bytes.len() < 8 {
                    return Err(refused(status::BAD_ARGUMENT, "a merge is 8 bytes of instructions, then a bitmap"));
                }
                r.merge(&bytes[8..], i64::from_le_bytes(bytes[..8].try_into().unwrap_or([0; 8])));
                Vec::new()
            }
        })
    })
}

/// The call stack a processor argument reaches: the main processor's, where the machine halts.
fn call_stack_of<'d>(d: &'d mut Debugger, core: &str, processor: u32) -> Answer<Option<&'d mut CallStackRegistry>> {
    let p = processor_of(d, core, processor)?;
    Ok(if p.is_none_or(|i| i == 0) { d.call_stack() } else { None })
}

/// `EMUSEN_SESSION_PROFILE_*`, under the gate.
pub fn profile_op(c: &Call, processor: u32, op: u32, raw: Raw) -> Answer<Vec<u8>> {
    c.gated(|e, _| {
        let bytes = raw.bytes()?;
        c.target()?;
        let core = e.name.clone();
        let stack = call_stack_of(&mut e.debug.d, &core, processor)?.ok_or_else(|| not_supported("This core does not report a call stack, so it cannot be profiled."))?;
        if op > profile::RESET_STACK {
            return Err(not_supported(format!("no profile op {op}")));
        }
        if c.read_only && (op <= profile::CLEAR || op == profile::RESET_STACK) {
            return Err(refused(status::READ_ONLY, READ_ONLY_WORDS));
        }
        Ok(match op {
            profile::RESET_STACK => {
                stack.reset();
                Vec::new()
            }
            profile::ARM => {
                stack.arm_profiler();
                Vec::new()
            }
            profile::DISARM => {
                stack.disarm_profiler();
                Vec::new()
            }
            profile::CLEAR => {
                stack.clear_profile();
                Vec::new()
            }
            profile::STATUS => {
                let mut j = Json::new();
                j.begin_object()
                    .bool("profiling", stack.is_profiling())
                    .int("instructions", stack.profiled_instructions())
                    .int("unmatched_returns", stack.unmatched_returns())
                    .int("depth", stack.depth() as i64)
                    .end_object();
                j.finish().into_bytes()
            }
            _ => {
                let a = args::parse(bytes)?;
                let limit = int32(args::optional_int(&a, "limit")?.unwrap_or(i32::MAX as i64), "limit")?;
                let mut j = Json::new();
                j.begin_array();
                for (address, instructions, calls, percent) in stack.hottest(limit) {
                    j.begin_object().u64("address", address as u32 as u64).int("instructions", instructions).int("calls", calls).f64("percent", percent).end_object();
                }
                j.end_array();
                j.finish().into_bytes()
            }
        })
    })
}

/// `EMUSEN_SESSION_COUNTERS_*`: the generic engine counts no accesses.
pub fn counters_op(c: &Call, raw: Raw) -> Answer<Vec<u8>> {
    c.gated(|_, _| {
        raw.bytes()?;
        c.target()?;
        Err(not_supported("This core does not count accesses."))
    })
}

/// `[{"source","target","kind","frame"}]`, innermost first, at most `count`.
pub fn call_stack(c: &Call, processor: u32, count: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let core = e.name.clone();
        let stack = call_stack_of(&mut e.debug.d, &core, processor)?.ok_or_else(|| not_supported("This core does not report a call stack."))?;
        let count = int32(count as i64, "count")?;
        let mut j = Json::new();
        j.begin_array();
        for f in stack.backtrace().take(count.max(0) as usize) {
            j.begin_object().u64("source", f.source as u32 as u64).u64("target", f.target as u32 as u64).int("kind", f.kind as i64).int("frame", f.frame).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

/// A byte held at a value at each frame's end.
pub fn freeze_add(c: &Call, name: Raw, address: u32, value: u32) -> Answer<i64> {
    c.change(|e, _| {
        let name = name.text()?;
        c.target()?;
        if value > 0xFF {
            return Err(refused(status::BAD_ARGUMENT, "a frozen value is one byte"));
        }
        let (d, m) = dm(e);
        let named = space(d, m.as_deref(), name)?;
        Ok(d.freezes.add(&named, address as i32, value as u8) as i64)
    })
}

pub fn freeze_remove(c: &Call, id: u32) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        if !e.debug.d.freezes.remove(id as i32) {
            return Err(no_such_id("freeze", id as i64));
        }
        Ok(0)
    })
}

pub fn freeze_list(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let mut j = Json::new();
        j.begin_array();
        for f in e.debug.d.freezes.all() {
            j.begin_object().int("id", f.id as i64).str("space", &f.space).u64("address", f.address as u32 as u64).int("value", f.value as i64).int("blocked", f.blocked).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

pub fn freeze_clear(c: &Call) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        e.debug.d.freezes.clear();
        Ok(0)
    })
}

pub fn label_add(c: &Call, address: u32, name: Raw, comment: Raw) -> Answer<i64> {
    c.change(|e, _| {
        let (name, comment) = (name.text()?, comment.text()?);
        if emusen_dotnet::text::is_blank(name) {
            return Err(refused(status::BAD_ARGUMENT, "a label must be named"));
        }
        c.target()?;
        e.debug.d.labels.add(address as i32, name.unwrap_or(""), comment);
        Ok(0)
    })
}

/// A label removed by name, or by address where no name is given.
pub fn label_remove(c: &Call, name: Raw, address: u32) -> Answer<i64> {
    c.change(|e, _| {
        let name = name.text()?;
        c.target()?;
        let labels = &mut e.debug.d.labels;
        let removed = match name {
            None => labels.remove_by_address(address as i32),
            Some(n) => labels.remove_by_name(n),
        };
        if !removed {
            return Err(refused(status::NO_SUCH_ID, match name {
                None => format!("No label at ${address:06X}."),
                Some(n) => format!("No label named '{n}'."),
            }));
        }
        Ok(0)
    })
}

pub fn label_list(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let mut j = Json::new();
        j.begin_array();
        for (address, name, comment) in e.debug.d.labels.all() {
            j.begin_object().u64("address", address as u32 as u64).str("name", name).str_or_null("comment", comment).end_object();
        }
        j.end_array();
        Ok(j.finish().into_bytes())
    })
}

pub fn label_at(c: &Call, address: u32) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let labels = &e.debug.d.labels;
        let Some(name) = labels.name_at(address as i32) else { return Err(refused(status::ABSENT, format!("No label at ${address:06X}."))) };
        let mut j = Json::new();
        j.begin_object().str("name", name).str_or_null("comment", labels.comment_at(address as i32)).end_object();
        Ok(j.finish().into_bytes())
    })
}

/// `{"added","errors"}` of a label file's lines.
fn label_record((added, errors): (usize, Vec<String>)) -> Vec<u8> {
    let mut j = Json::new();
    j.begin_object().int("added", added as i64).strs("errors", errors).end_object();
    j.finish().into_bytes()
}

/// A label file loaded only when its record fits `room`; a length query parses on a scratch registry and loads nothing.
pub fn label_load(c: &Call, text: Raw, room: i64) -> Answer<Vec<u8>> {
    c.change(|e, _| {
        let text = text.text()?.unwrap_or("");
        c.target()?;
        let lines: Vec<&str> = text.split('\n').map(|l| l.trim_end_matches('\r')).collect();
        let predicted = label_record(LabelRegistry::default().load_from_lines(lines.iter().copied()));
        if predicted.len() as i64 > room {
            return Ok(predicted);
        }
        Ok(label_record(e.debug.d.labels.load_from_lines(lines.iter().copied())))
    })
}

/// The labels as a label file's text.
pub fn label_save(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| Ok(e.debug.d.labels.to_lines().iter().map(|l| format!("{l}\n")).collect::<String>().into_bytes()))
}

pub fn label_clear(c: &Call) -> Answer<i64> {
    c.change(|e, _| {
        c.target()?;
        e.debug.d.labels.clear();
        Ok(0)
    })
}
