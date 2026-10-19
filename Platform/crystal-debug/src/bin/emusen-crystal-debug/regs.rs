//! The registries' verbs over plain data: breakpoints with their conditions, watches, the frame log, freezes, coverage
//! and the call stack. `CrystalDebugScript.Registries` runs the same verbs over the C# registries.

use crate::fake::Script;
use emusen_cheats::registry::{CheatRegistry, parse_write_type};
use emusen_core_host::sha256;
use emusen_systems::cheats::{Kind, Write, WriteType};
use emusen_crystal_debug::breakpoints::{BreakpointRegistry, CallKind, Evaluate, Unprepared};
use emusen_crystal_debug::conditions;
use emusen_crystal_debug::coverage::{CallStackRegistry, CoverageRegistry};
use emusen_crystal_debug::watches::{self, FrameLogRegistry, FreezeRegistry, WatchKind, WatchRegistry};

pub struct Registries {
    pub bp: BreakpointRegistry,
    /// The call stack the breakpoints are told, its depth; none for a registry told no stack.
    pub depth: Option<i32>,
    pub watches: WatchRegistry,
    pub frame_log: FrameLogRegistry,
    pub freezes: FreezeRegistry,
    pub coverage: CoverageRegistry,
    pub stack: CallStackRegistry,
    pub cheats: CheatRegistry,
}

impl Registries {
    pub fn new() -> Registries {
        Registries {
            bp: BreakpointRegistry::new(),
            depth: None,
            watches: WatchRegistry::new(),
            frame_log: FrameLogRegistry::new(),
            freezes: FreezeRegistry::new(),
            coverage: CoverageRegistry::new(),
            stack: CallStackRegistry::new(),
            cheats: CheatRegistry::new(),
        }
    }
}

fn number(word: &str) -> i64 {
    let w = word.trim();
    if let Some(hex) = w.strip_prefix("0x").or_else(|| w.strip_prefix('$')) {
        return u64::from_str_radix(hex, 16).map(|v| v as i64).unwrap_or(0);
    }
    w.parse().unwrap_or(0)
}

fn kind_named(word: &str) -> CallKind {
    match word {
        "nmi" => CallKind::Nmi,
        "irq" => CallKind::Irq,
        "brk" => CallKind::Brk,
        "cop" => CallKind::Cop,
        _ => CallKind::Call,
    }
}

fn opt(text: &str) -> Option<&str> {
    let t = text.trim();
    (t != "-").then_some(t)
}

fn or_dash(text: &Option<String>) -> &str {
    text.as_deref().unwrap_or("-")
}

fn hex_bytes(text: &str) -> Vec<u8> {
    (0..text.len() / 2).map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap_or(0)).collect()
}

fn coverage_status(c: &CoverageRegistry) -> String {
    format!("{} {} {} {}", c.is_armed(), c.instructions(), c.has_mark(), c.entry_point_count())
}

fn join<T>(items: impl IntoIterator<Item = T>, f: impl Fn(T) -> String) -> String {
    items.into_iter().map(f).collect::<Vec<_>>().join(";")
}

/// The answer to a registry's verb; none for a verb that is not one.
pub fn run(script: &mut Script, verb: &str, rest: &str, words: &[&str]) -> Option<String> {
    if verb.starts_with("cheat.") {
        return cheat(script, verb, rest, words);
    }
    let (mut eval, r) = script.split();
    let depth = r.depth.unwrap_or(0);
    let mut unprepared = Unprepared;
    let e: &mut dyn Evaluate = if r.bp.prepared { &mut eval } else { &mut unprepared };
    let w = |i: usize| words.get(i).copied().unwrap_or("");
    Some(match verb {
        "bp.add" | "bp.data" => {
            // bp.add ADDR END ; COND ; LOG and bp.data SPACE ADDR END VALUE READ CHANGED ; COND ; LOG
            let fields: Vec<&str> = rest.split(" ; ").collect();
            let (condition, log) = (opt(fields.get(1).copied().unwrap_or("-")), opt(fields.get(2).copied().unwrap_or("-")));
            let head: Vec<&str> = fields[0].split(' ').filter(|x| !x.is_empty()).collect();
            match eval.scope(|c| conditions::prepare(c, condition, log)) {
                Err(words) => format!("! {words}"),
                Ok(prepared) => {
                    if prepared {
                        r.bp.prepared = true;
                    }
                    let id = if verb == "bp.add" {
                        r.bp.add_breakpoint(number(head[0]) as i32, number(head[1]) as i32, condition, log)
                    } else {
                        r.bp.add_data_breakpoint(head[0], number(head[1]) as i32, number(head[2]) as i32, number(head[3]) as i32, head[4] == "1", head[5] == "1", condition, log)
                    };
                    id.to_string()
                }
            }
        }
        "bp.forbid" => r.bp.add_forbid_range(number(w(0)) as i32, number(w(1)) as i32).to_string(),
        "bp.remove" => r.bp.remove(number(w(0)) as i32).to_string(),
        "bp.enable" => r.bp.set_enabled(number(w(0)) as i32, w(1) == "1").to_string(),
        "bp.stack" => {
            r.depth = if w(0) == "none" { None } else { Some(number(w(0)) as i32) };
            r.bp.knows_stack = r.depth.is_some();
            "ok".into()
        }
        "bp.should" => {
            let halt = r.bp.should_break(number(w(0)) as i32, depth, e);
            format!("{halt} reason={} error={}", r.bp.last_break_reason, r.bp.last_condition_error)
        }
        "bp.write" => {
            r.bp.note_write(w(0), number(w(1)) as i32, number(w(2)) as u8, e);
            "ok".into()
        }
        "bp.read" => {
            r.bp.note_read(w(0), number(w(1)) as i32, number(w(2)) as u8, e);
            "ok".into()
        }
        "bp.list" => {
            let b = &r.bp;
            format!(
                "code[{}] data[{}] forbid[{}] guard={} uninit={}",
                join(b.breakpoints(), |x| format!("{}:{}:{}:{}:{}:{}:{}", x.id, x.address, x.end, x.enabled, x.hits, or_dash(&x.condition), or_dash(&x.log))),
                join(b.data_breakpoints(), |x| format!(
                    "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
                    x.id,
                    x.space,
                    x.address,
                    x.end,
                    x.value,
                    x.on_read,
                    x.changed_only,
                    x.enabled,
                    x.hits,
                    or_dash(&x.condition),
                    or_dash(&x.log)
                )),
                join(b.forbid_ranges(), |x| format!("{}:{}:{}:{}", x.id, x.address, x.end, x.enabled)),
                b.depth_guard(),
                if b.uninitialized_armed() { b.uninitialized_space() } else { "-" }
            )
        }
        "bp.log" => format!("{} {}", r.bp.log_recorded(), join(r.bp.log_tail(number(w(0)) as i32), |x| format!("{}:{}:{}", x.id, x.address, x.text))),
        "bp.clearlog" => {
            r.bp.clear_log();
            "ok".into()
        }
        "bp.state" => {
            let b = &r.bp;
            format!(
                "step={} depth={} pending={} data={} event={} could={} quiet={} writes={} conds={} uninit={}:{}",
                b.single_step_armed(),
                b.step_depth_target().map_or("-".to_string(), |d| d.to_string()),
                b.has_pending_break(),
                b.last_data_break(),
                b.last_event_break(),
                b.could_break(),
                b.is_quiet(),
                b.watches_writes(),
                b.any_condition_armed(),
                b.uninitialized_armed(),
                b.uninitialized_space()
            )
        }
        "bp.step" => {
            r.bp.arm_step(number(w(0)) as i32);
            "ok".into()
        }
        "bp.stepdepth" => r.bp.arm_step_to_depth(number(w(0)) as i32).to_string(),
        "bp.guard" => r.bp.arm_depth_guard(number(w(0)) as i32).to_string(),
        "bp.unguard" => {
            r.bp.disarm_depth_guard();
            "ok".into()
        }
        "bp.runto" => r.bp.arm_run_to_interrupt(kind_named(w(0))).to_string(),
        "bp.scanline" => {
            r.bp.arm_run_to_scanline(number(w(0)) as i32);
            "ok".into()
        }
        "bp.runframe" => {
            r.bp.arm_run_to_frame(number(w(0)));
            "ok".into()
        }
        "bp.disarm" => {
            r.bp.disarm_steps();
            "ok".into()
        }
        "bp.interrupt" => {
            r.bp.note_interrupt(kind_named(w(0)));
            "ok".into()
        }
        "bp.notescanline" => {
            r.bp.note_scanline(number(w(0)) as i32);
            "ok".into()
        }
        "bp.frame" => {
            r.bp.note_frame(number(w(0)));
            "ok".into()
        }
        "bp.uninit" => r.bp.arm_uninitialized(w(0), number(w(1)) as i32).to_string(),
        "bp.uninitoff" => {
            r.bp.disarm_uninitialized();
            "ok".into()
        }
        "bp.cond" => {
            r.bp.arm_condition(w(0), w(1) == "1");
            "ok".into()
        }
        "bp.uncond" => r.bp.disarm_condition(w(0)).to_string(),
        "bp.condnote" => {
            r.bp.note_condition(w(0), number(w(1)) as i32, &words[2..].join(" "));
            "ok".into()
        }
        "bp.armed" => r.bp.is_condition_armed(w(0)).to_string(),
        "bp.conds" => join(r.bp.conditions(), |c| format!("{}:{}:{}:{}", c.name, c.log_only, c.enabled, c.hits)),
        "logging" => {
            watches::set_logging(w(0) == "1");
            "ok".into()
        }
        "watch.add" => r.watches.add(w(0), number(w(1)) as i32, number(w(2)) as i32, WatchKind::from_number(number(w(3)) as u32).unwrap_or(WatchKind::Write)).to_string(),
        "watch.remove" => r.watches.remove(number(w(0)) as i32).to_string(),
        "watch.list" => join(r.watches.watches(), |x| format!("{}:{}:{}:{}:{}", x.id, x.space, x.address, x.length, x.kind as i32)),
        "watch.rec" => {
            let pc = number(w(4)) as u32;
            let kind = if w(0) == "r" { WatchKind::Read } else { WatchKind::Write };
            r.watches.record(kind, w(1), number(w(2)) as i32, number(w(3)) as u8, || format!("PC=${pc:04X}"));
            "ok".into()
        }
        "watch.events" => join(r.watches.events(number(w(0)) as i32, number(w(1)) as i32), |x| format!("{}:{}:{}:{}:{}", x.sequence, x.kind as i32, x.address, x.value, x.context)),
        "watch.sites" => join(r.watches.sites(number(w(0)) as i32), |(c, n)| format!("{c}:{n}")),
        "watch.counts" => {
            let (total, retained) = r.watches.counts(number(w(0)) as i32);
            format!("{total} {retained}")
        }
        "watch.clear" => {
            r.watches.clear_events(number(w(0)) as i32);
            "ok".into()
        }
        "said" => std::mem::take(&mut r.watches.said).join("|"),
        "fl.add" => r.frame_log.add(w(0), number(w(1)) as i32, number(w(2)) as i32).to_string(),
        "fl.remove" => r.frame_log.remove(number(w(0)) as i32).to_string(),
        "fl.list" => join(r.frame_log.entries(), |x| format!("{}:{}:{}:{}", x.id, x.space, x.address, x.width)),
        "fl.record" => {
            let data = &*eval.data;
            r.frame_log.record_frame(number(w(0)), |space, address, width| {
                (0..width).fold(0i64, |v, i| v | (data.byte(space, address.wrapping_add(i)) as i64) << (8 * i))
            });
            "ok".into()
        }
        "fl.samples" => join(r.frame_log.samples(number(w(0)) as i32, number(w(1)) as i32), |s| format!("{}:{}", s.frame, s.value)),
        "fl.clear" => {
            r.frame_log.clear_samples(number(w(0)) as i32);
            "ok".into()
        }
        "fz.add" => r.freezes.add(w(0), number(w(1)) as i32, number(w(2)) as u8).to_string(),
        "fz.remove" => r.freezes.remove(number(w(0)) as i32).to_string(),
        "fz.clear" => {
            r.freezes.clear();
            "ok".into()
        }
        "fz.list" => join(r.freezes.all(), |x| format!("{}:{}:{}:{}:{}", x.id, x.space, x.address, x.value, x.blocked)),
        "fz.enforce" => {
            let data = std::cell::RefCell::new(&mut *eval.data);
            r.freezes.enforce(|s, a| data.borrow().byte(s, a), |s, a, v| data.borrow_mut().put(s, a, v));
            "ok".into()
        }
        "poke" => {
            eval.data.put(w(0), number(w(1)) as i32, number(w(2)) as u8);
            "ok".into()
        }
        "peek" => eval.data.byte(w(0), number(w(1)) as i32).to_string(),
        "cov.arm" => {
            r.coverage.arm();
            "ok".into()
        }
        "cov.disarm" => {
            r.coverage.disarm();
            "ok".into()
        }
        "cov.clear" => {
            r.coverage.clear();
            "ok".into()
        }
        "cov.merge" => {
            r.coverage.merge(&hex_bytes(w(1)), number(w(0)));
            "ok".into()
        }
        "cov.entry" => {
            r.coverage.record_entry_point(number(w(0)) as i32, kind_named(w(1)));
            "ok".into()
        }
        "cov.status" => coverage_status(&r.coverage),
        "cov.query" => {
            let (a, l, n) = (number(w(0)) as i32, number(w(1)) as i32, number(w(2)) as i32);
            let (executed, total, fresh) = r.coverage.statistics(a, l);
            format!("{executed} {total} {fresh} [{}]", join(r.coverage.executed_addresses(a, l, n), |x| x.to_string()))
        }
        "cov.mark" => {
            r.coverage.mark();
            "ok".into()
        }
        "cov.clearmark" => {
            r.coverage.clear_mark();
            "ok".into()
        }
        "cov.new" => join(r.coverage.new_since_mark(number(w(0)) as i32, number(w(1)) as i32, number(w(2)) as i32), |x| x.to_string()),
        "cov.entries" => join(r.coverage.entry_points(number(w(0)) as i32), |(a, k, n)| format!("{a}:{k}:{n}")),
        "cov.in" => join(r.coverage.entry_points_in(number(w(0)) as i32, number(w(1)) as i32), |x| x.to_string()),
        "cov.export" => {
            let bytes = r.coverage.export();
            format!("{} {}", bytes.len(), sha256::sha256_hex(&bytes))
        }
        "cov.roundtrip" => {
            let bytes = r.coverage.export();
            let mut other = CoverageRegistry::new();
            other.arm();
            let ok = other.import(&bytes);
            let again = other.export();
            format!("{ok} {} {} {}", coverage_status(&other), again.len(), sha256::sha256_hex(&again))
        }
        "cov.reimport" => {
            let bytes = r.coverage.export();
            r.coverage.import(&bytes).to_string()
        }
        "cov.import" => r.coverage.import(&hex_bytes(rest.trim())).to_string(),
        "cs.push" => {
            let frame = eval.data.frame_now();
            r.stack.note_push(number(w(0)) as i32, number(w(1)) as i32, kind_named(w(2)), frame, Some(&mut r.coverage));
            "ok".into()
        }
        "cs.pop" => {
            r.stack.note_pop();
            "ok".into()
        }
        "cs.bt" => join(r.stack.backtrace().take(number(w(0)).max(0) as usize), |f| format!("{}:{}:{}:{}", f.source, f.target, f.kind as i32, f.frame)),
        "cs.profile" => {
            if w(0) == "on" { r.stack.arm_profiler() } else { r.stack.disarm_profiler() }
            "ok".into()
        }
        "cs.clearprofile" => {
            r.stack.clear_profile();
            "ok".into()
        }
        "cs.instr" => {
            r.stack.note_instructions(number(w(0)) as i32, number(w(1)));
            "ok".into()
        }
        "cs.hot" => join(r.stack.hottest(number(w(0)) as i32), |(a, n, c, p)| format!("{a}:{n}:{c}:{}", emusen_dotnet::json::format_f64(p).unwrap_or_default())),
        "cs.status" => format!("{} {} {} {}", r.stack.is_profiling(), r.stack.profiled_instructions(), r.stack.unmatched_returns(), r.stack.depth()),
        "cs.reset" => {
            r.stack.reset();
            "ok".into()
        }
        _ => return None,
    })
}

fn kind_name(kind: Kind) -> &'static str {
    if kind == Kind::RomPatch { "RomPatch" } else { "RamPoke" }
}

fn type_number(kind: WriteType) -> i32 {
    kind as i32
}

/// A write as a script spells it: space:address:value:width:type:bit:big:repeat:repeat-address:repeat-value.
fn write_of(text: &str) -> Option<Write> {
    let f: Vec<&str> = text.trim().split(':').collect();
    let g = |i: usize| f.get(i).copied().unwrap_or("");
    Some(Write {
        space: if g(0) == "-" { String::new() } else { g(0).to_string() },
        address: number(g(1)) as i32,
        value: number(g(2)) as u32,
        width: number(g(3)) as i32,
        kind: parse_write_type(Some(g(4)))?,
        bit_position: (g(5) != "-" && !g(5).is_empty()).then(|| number(g(5)) as i32),
        big_endian: g(6) == "1",
        repeat_count: number(g(7)) as i32,
        repeat_add_address: number(g(8)) as i32,
        repeat_add_value: number(g(9)) as u32,
    })
}

fn cheat(script: &mut Script, verb: &str, rest: &str, words: &[&str]) -> Option<String> {
    let (eval, r) = script.split();
    let c = &mut r.cheats;
    let w = |i: usize| words.get(i).copied().unwrap_or("");
    let desc = |s: &str| s.replace('~', " ");
    Some(match verb {
        "cheat.add" => {
            let fields: Vec<&str> = rest.split(" ; ").collect();
            let head: Vec<&str> = fields[0].split(' ').filter(|x| !x.is_empty()).collect();
            let kind = if head.first() == Some(&"romPatch") { Kind::RomPatch } else { Kind::RamPoke };
            let compare = head.get(1).filter(|x| **x != "-").map(|x| number(x) as u8);
            let mut writes = Vec::new();
            for f in &fields[1..] {
                match write_of(f) {
                    Some(x) => writes.push(x),
                    None => return Some("bad write".into()),
                }
            }
            match c.add(kind, writes, compare, &desc(head.get(2).copied().unwrap_or("")), head.get(3) != Some(&"0")) {
                Ok(id) => id.to_string(),
                Err(e) => format!("! {e}"),
            }
        }
        "cheat.poke" => c.add_ram_poke(w(0), number(w(1)) as i32, number(w(2)) as u8, &desc(w(3)), true).map_or_else(|e| format!("! {e}"), |id| id.to_string()),
        "cheat.patch" => c
            .add_rom_patch(number(w(0)) as i32, number(w(1)) as u8, (w(2) != "-").then(|| number(w(2)) as u8), &desc(w(3)), true)
            .map_or_else(|e| format!("! {e}"), |id| id.to_string()),
        "cheat.remove" => c.remove(number(w(0)) as i32).to_string(),
        "cheat.enable" => c.set_enabled(number(w(0)) as i32, w(1) == "1").to_string(),
        "cheat.master" => {
            c.set_master_enabled(w(0) == "1");
            "ok".into()
        }
        "cheat.clear" => {
            c.clear();
            "ok".into()
        }
        "cheat.version" => format!("{} {} {}", c.version(), c.enabled_rom_patches(), c.master_enabled()),
        "cheat.list" => join(c.cheats(), |x| {
            format!(
                "{}:{}:{}:{}:{}:[{}]",
                x.id,
                kind_name(x.kind),
                x.compare.map_or("-".to_string(), |v| v.to_string()),
                x.description,
                x.enabled,
                x.writes
                    .iter()
                    .map(|v| format!(
                        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
                        v.space,
                        v.address,
                        v.value,
                        v.width,
                        type_number(v.kind),
                        v.bit_position.map_or("-".to_string(), |b| b.to_string()),
                        v.big_endian,
                        v.repeat_count,
                        v.repeat_add_address,
                        v.repeat_add_value
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }),
        "cheat.apply" => {
            let data = std::cell::RefCell::new(&mut *eval.data);
            c.apply_all(&mut |s, a| data.borrow().byte(s, a), &mut |s, a, v| data.borrow_mut().put(s, a, v));
            "ok".into()
        }
        "cheat.rom" => c.try_patch_rom(number(w(0)) as u32, number(w(1)) as u8).map_or("none".to_string(), |v| v.to_string()),
        "cheat.resolve" => join(c.resolve_rom_patches(number(w(0))), |p| format!("{}:{}:{}", p.address, p.value, p.compare.map_or("-".to_string(), |v| v.to_string()))),
        "cheat.triples" => {
            let mut applied = emusen_cheats::registry::Applied::default();
            match applied.refresh(c, Some((number(w(0)), number(w(1)))), false, &|_| None) {
                emusen_cheats::registry::Send::Tables { triples: Some(t), .. } => join(t, |x| x.to_string()),
                _ => "none".into(),
            }
        }
        "cheat.file" => {
            let mut out = String::new();
            emusen_dotnet::json::write_compact(&mut out, &c.to_file());
            out
        }
        "cheat.load" => match emusen_dotnet::json::parse(rest) {
            Ok(v) => {
                let (loaded, skipped) = c.load(&v);
                format!("{loaded} {skipped}")
            }
            Err(_) => "bad json".into(),
        },
        _ => return None,
    })
}
