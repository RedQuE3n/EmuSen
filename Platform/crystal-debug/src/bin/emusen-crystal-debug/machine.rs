//! The script over a machine: a core library opened with a game as `CoreEngine.LoadRom` opens one with no battery, its
//! frames run in `CoreEngine.RunFrame`'s order with the debugger's points, and the debugger asked as a session would.
//! `CrystalDebugScript.Machine` runs the same verbs over the C# engine and target.

use emusen_core_host::{Create, Library, Machine};
use emusen_crystal_debug::breakpoints::{BreakpointRegistry, CallKind};
use emusen_crystal_debug::context::Set;
use emusen_crystal_debug::expr;
use emusen_crystal_debug::watches::WatchKind;
use emusen_crystal_debug::{Debugger, Ran};
use emusen_native::core::caps;
use emusen_native::core::sys::event;
use emusen_native::json::{self, Value};

pub struct MachineScript {
    machine: Option<Machine>,
    pub d: Debugger,
    halted: bool,
    halted_processor: u32,
    halted_address: u32,
    audio_limit_sent: bool,
    raw_selected: bool,
    display: (String, String),
    /// The core and image last opened, which the bench opens again for each of its runs.
    opened: (String, String),
}

fn number(word: &str) -> i64 {
    let w = word.trim();
    if let Some(hex) = w.strip_prefix("0x").or_else(|| w.strip_prefix('$')) {
        return u64::from_str_radix(hex, 16).map(|v| v as i64).unwrap_or(0);
    }
    w.parse().unwrap_or(0)
}

fn opt(text: &str) -> Option<&str> {
    let t = text.trim();
    (t != "-").then_some(t)
}

fn join<T>(items: impl IntoIterator<Item = T>, f: impl Fn(T) -> String) -> String {
    items.into_iter().map(f).collect::<Vec<_>>().join(";")
}

fn or_dash(text: &Option<String>) -> &str {
    text.as_deref().unwrap_or("-")
}

/// The settings a library's schema gives, each `key=default`: all of them, or those set while running.
fn settings_text(lib: &Library, create: bool) -> String {
    let doc = json::parse(lib.settings_text().unwrap_or_default().as_bytes()).unwrap_or(Value::Null);
    doc.as_array()
        .unwrap_or(&[])
        .iter()
        .filter(|s| create || s.get("scope").and_then(Value::as_str) != Some("create"))
        .map(|s| format!("{}={}", s.get("key").and_then(Value::as_str).unwrap_or(""), s.get("default").and_then(Value::as_str).unwrap_or("")))
        .collect::<Vec<_>>()
        .join("\n")
}

fn registry_line(b: &BreakpointRegistry) -> String {
    format!(
        "code[{}] data[{}] forbid[{}] guard={} hits-log={}",
        join(b.breakpoints(), |x| format!("{}:{}:{}:{}:{}:{}:{}", x.id, x.address, x.end, x.enabled, x.hits, or_dash(&x.condition), or_dash(&x.log))),
        join(b.data_breakpoints(), |x| format!("{}:{}:{}:{}:{}:{}:{}:{}", x.id, x.space, x.address, x.end, x.value, x.on_read, x.enabled, x.hits)),
        join(b.forbid_ranges(), |x| format!("{}:{}:{}", x.id, x.address, x.end)),
        b.depth_guard(),
        b.log_recorded()
    )
}

fn state_line(b: &BreakpointRegistry) -> String {
    format!(
        "reason={} data={} event={} step={} depth={} pending={} error={}",
        b.last_break_reason,
        b.last_data_break(),
        b.last_event_break(),
        b.single_step_armed(),
        b.step_depth_target().map_or("-".to_string(), |d| d.to_string()),
        b.has_pending_break(),
        b.last_condition_error
    )
}

fn set_named(word: &str) -> Option<Set> {
    Some(match word {
        "cpu" => Set::Processor(0),
        "video" => Set::Video,
        "apu" => Set::Apu,
        "cop" => Set::Coprocessor,
        w => Set::Processor(w.parse().ok()?),
    })
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

/// A scope word: "-" the session's own, else a processor's index.
fn scope(word: &str) -> Option<usize> {
    (word != "-").then(|| word.parse().unwrap_or(0))
}

impl MachineScript {
    pub fn new() -> MachineScript {
        MachineScript { machine: None, d: Debugger::new(), halted: false, halted_processor: 0, halted_address: 0, audio_limit_sent: false, raw_selected: false, display: (String::new(), String::new()), opened: (String::new(), String::new()) }
    }

    /// The events drained as `CoreEngine.Drain` handles those a debugger sees: a new machine info read, a log drained.
    fn drain(&mut self) {
        let m = self.machine.as_mut().unwrap();
        let kinds: Vec<u32> = m.events().map(|e| e.iter().map(|e| e.kind).collect()).unwrap_or_default();
        for kind in kinds {
            match kind {
                event::MACHINE_INFO => {
                    self.raw_selected = false;
                    self.d.info_changed(m);
                }
                event::LOG => {
                    let mut s = String::new();
                    let _ = m.log_drain(&mut s);
                }
                event::GEOMETRY => {
                    let _ = m.frame_info();
                }
                _ => {}
            }
        }
    }

    fn open(&mut self, core: &str, image: &str) -> Result<(), String> {
        self.opened = (core.to_string(), image.to_string());
        let lib = Library::load(std::path::Path::new(core)).map_err(|e| format!("{e:?}"))?;
        lib.check().map_err(str::to_string)?;
        let bytes = std::fs::read(image).map_err(|e| e.to_string())?;
        let settings = settings_text(lib, true);
        let mut c = Create::new(&bytes);
        c.settings = &settings;
        c.pixel_formats = 1;
        let mut m = Machine::create(lib, &c).map_err(|r| r.words)?;
        let _ = m.set_options(0);
        let run = settings_text(lib, false);
        if !run.is_empty() && lib.claims(caps::SETTINGS) {
            let _ = m.set_settings(&run);
        }
        let _ = m.setting_notes();
        let info = json::parse(lib.info_text().unwrap_or_default().as_bytes()).unwrap_or(Value::Null);
        let name = info.get("name").and_then(Value::as_str).unwrap_or("").to_string();
        let display = info.get("display_name").and_then(Value::as_str).map_or(name, str::to_string);
        self.display = (display, info.get("version").and_then(Value::as_str).unwrap_or("").to_string());
        self.machine = Some(m);
        self.drain();
        let m = self.machine.as_mut().unwrap();
        // A game opened is a session of its own, as the C# makes an engine for it.
        self.d = Debugger::new();
        self.d.attach(m);
        (self.halted, self.audio_limit_sent, self.raw_selected) = (false, false, false);
        Ok(())
    }

    /// `CoreEngine.RunFrame` with the debugger's points, then the session's refresh.
    fn frame(&mut self) -> Result<(), String> {
        self.engine_frame()?;
        let m = self.machine.as_mut().unwrap();
        self.d.refresh(m, 0.0);
        Ok(())
    }

    /// `CoreEngine.RunFrame` alone, as the bench times it.
    fn engine_frame(&mut self) -> Result<(), String> {
        let m = self.machine.as_mut().ok_or("no machine")?;
        self.d.refresh_cheats(m);
        if !self.audio_limit_sent {
            let _ = m.set_audio_limit(128_000);
            self.audio_limit_sent = true;
        }
        if !self.raw_selected {
            if m.library().claims(caps::RAW_PICTURE) {
                let _ = m.raw_select(0);
            }
            self.raw_selected = true;
        }
        let resuming = self.halted;
        self.halted = false;
        let ran = if !resuming && !self.d.armed() {
            m.advance().map_err(|(s, _)| format!("status {s}"))?;
            Ran::Completed
        } else {
            self.d.run_frame(m, resuming).map_err(|(s, _)| format!("status {s}"))?
        };
        if let Ran::Halted { processor, address } = ran {
            (self.halted, self.halted_processor, self.halted_address) = (true, processor, address);
            self.drain();
        } else {
            self.d.frame_end(m);
            if m.library().claims(caps::PRESENT) {
                let _ = m.present();
            }
            let _ = m.frame_device();
            self.drain();
            let m = self.machine.as_mut().unwrap();
            if m.frame_count() % 60 == 0 {
                let mut s = String::new();
                let _ = m.log_drain(&mut s);
            }
        }
        Ok(())
    }

    /// `BreakpointConditionCostBench`'s method: a fresh machine for each of five runs (nothing armed, a breakpoint never
    /// reached, a logpoint, `if 0` and the condition at a hot address), 30 frames to warm, the median of 300; each
    /// hot row's cost a hit over the cold breakpoint, the hits a frame counted by the logpoint.
    fn bench(&mut self, address: i32, condition: &str, log: &str) -> Result<String, String> {
        const FRAMES: usize = 300;
        let (core, image) = self.opened.clone();
        let runs: [(Option<i32>, Option<&str>, Option<&str>); 5] =
            [(None, None, None), (Some(0x7FFE), None, None), (Some(address), None, Some(log)), (Some(address), Some("0"), None), (Some(address), Some(condition), None)];
        let mut medians = [0f64; 5];
        let mut hits = 0f64;
        for (i, &(at, c, l)) in runs.iter().enumerate() {
            self.open(&core, &image)?;
            let m = self.machine.as_mut().unwrap();
            if let Some(at) = at {
                self.d.add_breakpoint(m, None, at, at, c, l)?;
            }
            for _ in 0..30 {
                self.engine_frame()?;
            }
            if self.halted {
                return Err(format!("run {i} halts at every hit"));
            }
            let before: i64 = self.d.breakpoints().breakpoints().iter().map(|b| b.hits).sum();
            let mut times = Vec::with_capacity(FRAMES);
            for _ in 0..FRAMES {
                let start = std::time::Instant::now();
                self.engine_frame()?;
                times.push(start.elapsed().as_secs_f64() * 1e6);
            }
            times.sort_by(f64::total_cmp);
            medians[i] = times[FRAMES / 2];
            if l.is_some() {
                let after: i64 = self.d.breakpoints().breakpoints().iter().map(|b| b.hits).sum();
                hits = (after - before) as f64 / FRAMES as f64;
            }
        }
        let per = |median: f64| if hits > 0.0 { (median - medians[1]) / hits } else { 0.0 };
        Ok(format!(
            "frame {:.1} us, cold {:.1} us, {:.0} hits a frame; a hit: log {:.2} us, if 0 {:.2} us, if {condition} {:.2} us",
            medians[0],
            medians[1],
            hits,
            per(medians[2]),
            per(medians[3]),
            per(medians[4])
        ))
    }

    pub fn run(&mut self, verb: &str, rest: &str) -> Option<String> {
        let words: Vec<&str> = rest.split(' ').filter(|w| !w.is_empty()).collect();
        let w = |i: usize| words.get(i).copied().unwrap_or("");
        if verb == "m.open" {
            return Some(match self.open(w(0), w(1)) {
                Ok(()) => "ok".into(),
                Err(e) => format!("! {e}"),
            });
        }
        if verb == "m.bench" {
            return Some(match self.bench(number(w(0)) as i32, w(1), w(2)) {
                Ok(row) => row,
                Err(e) => format!("! {e}"),
            });
        }
        if verb == "m.run" {
            let mut ran = 0;
            for _ in 0..number(w(0)) {
                if let Err(e) = self.frame() {
                    return Some(format!("! {e}"));
                }
                ran += 1;
                if self.halted {
                    break;
                }
            }
            let Some(m) = self.machine.as_ref() else { return Some("no machine".into()) };
            return Some(if self.halted {
                format!("ran={ran} frame={} halted {} at ${:X}", m.frame_count(), self.halted_processor, self.halted_address)
            } else {
                format!("ran={ran} frame={}", m.frame_count())
            });
        }
        let Some(m) = self.machine.as_mut() else { return Some("no machine".into()) };
        let d = &mut self.d;
        Some(match verb {
            "m.frame" => m.frame_count().to_string(),
            "m.refresh" => {
                d.refresh(m, 0.0);
                "ok".into()
            }
            "m.spaces" => join(d.spaces(m), |s| format!("{}:{}:{}:{}", s.name, s.size, s.writable, s.side_effects)),
            "m.peek" => {
                let n = if words.len() > 2 { number(w(2)) } else { 1 };
                (0..n).map(|i| format!("{:02X}", d.space_read(m, w(0), (number(w(1)) + i) as i32))).collect::<Vec<_>>().join(" ")
            }
            "m.poke" => {
                d.space_write(m, w(0), number(w(1)) as i32, number(w(2)) as u8);
                "ok".into()
            }
            "m.procs" => {
                let cpus = d.processors();
                join(cpus.iter().enumerate(), |(i, c)| {
                    format!(
                        "{}:{}:{}:{}:{}:{}:{}",
                        c.name,
                        c.description,
                        c.can_halt,
                        c.coverage,
                        c.call_stack,
                        c.code_space.as_deref().unwrap_or("-"),
                        if c.has_pc { format!("{:X}", d.program_counter(m, i)) } else { "-".into() }
                    )
                })
            }
            "m.regs" => match set_named(w(0)) {
                Some(set) => join(d.registers(set), |r| format!("{}={}/{}", r.name, r.value, r.bits)),
                None => "bad set".into(),
            },
            "m.counters" => join(d.counters(), |c| format!("{}:{}:{}:{}:{}", c.name, c.label, c.unit, c.kind, c.value)),
            // A core whose counters time a thread of its own, named but not compared.
            "m.counternames" => join(d.counters(), |c| format!("{}:{}:{}:{}", c.name, c.label, c.unit, c.kind)),
            "m.load" => join(d.load(), |(n, _)| n),
            "m.eval" => {
                let (p, e) = rest.split_once(' ').unwrap_or((rest, ""));
                match d.evaluate(m, scope(p), e) {
                    Ok(v) => expr::format(v),
                    Err(e) => format!("! {}", e.message()),
                }
            }
            "m.symbols" => d.symbol_names(m, scope(w(0))).join(","),
            "m.bp.add" | "m.bp.data" => {
                let fields: Vec<&str> = rest.split(" ; ").collect();
                let (c, l) = (opt(fields.get(1).copied().unwrap_or("-")), opt(fields.get(2).copied().unwrap_or("-")));
                let h: Vec<&str> = fields[0].split(' ').filter(|x| !x.is_empty()).collect();
                let r = if verb == "m.bp.add" {
                    d.add_breakpoint(m, scope(h[0]), number(h[1]) as i32, number(h[2]) as i32, c, l)
                } else {
                    d.add_data_breakpoint(m, scope(h[0]), h[1], number(h[2]) as i32, number(h[3]) as i32, number(h[4]) as i32, h[5] == "1", h[6] == "1", c, l)
                };
                match r {
                    Ok(id) => id.to_string(),
                    Err(e) => format!("! {e}"),
                }
            }
            "m.bp.list" => d.registry_of(scope(w(0))).map_or("none".into(), |b| registry_line(b)),
            "m.bp.state" => d.registry_of(scope(w(0))).map_or("none".into(), |b| state_line(b)),
            "m.bp.log" => d.registry_of(scope(w(0))).map_or("none".into(), |b| join(b.log_tail(number(w(1)) as i32), |x| format!("{}:{}:{}", x.id, x.address, x.text))),
            "m.bp.remove" => d.registry_of(scope(w(0))).map_or("none".into(), |b| b.remove(number(w(1)) as i32).to_string()),
            "m.bp.step" => d.registry_of(scope(w(0))).map_or("none".into(), |b| {
                b.arm_step(number(w(1)) as i32);
                "ok".into()
            }),
            "m.bp.stepdepth" => d.registry_of(scope(w(0))).map_or("none".into(), |b| b.arm_step_to_depth(number(w(1)) as i32).to_string()),
            "m.bp.runto" => d.registry_of(scope(w(0))).map_or("none".into(), |b| b.arm_run_to_interrupt(kind_named(w(1))).to_string()),
            "m.bp.runframe" => d.registry_of(scope(w(0))).map_or("none".into(), |b| {
                b.arm_run_to_frame(number(w(1)));
                "ok".into()
            }),
            "m.bp.uninit" => d.registry_of(scope(w(0))).map_or("none".into(), |b| b.arm_uninitialized(w(1), number(w(2)) as i32).to_string()),
            "m.armed" => d.armed().to_string(),
            "m.watch.add" => d.watches.add(w(0), number(w(1)) as i32, number(w(2)) as i32, WatchKind::from_number(number(w(3)) as u32).unwrap_or(WatchKind::Write)).to_string(),
            "m.watch.events" => join(d.watches.events(number(w(0)) as i32, number(w(1)) as i32), |x| format!("{}:{}:{}:{}:{}", x.sequence, x.kind as i32, x.address, x.value, x.context)),
            "m.watch.remove" => d.watches.remove(number(w(0)) as i32).to_string(),
            "m.fl.add" => d.frame_log.add(w(0), number(w(1)) as i32, number(w(2)) as i32).to_string(),
            "m.fl.samples" => join(d.frame_log.samples(number(w(0)) as i32, number(w(1)) as i32), |s| format!("{}:{}", s.frame, s.value)),
            "m.fz.add" => d.freezes.add(w(0), number(w(1)) as i32, number(w(2)) as u8).to_string(),
            "m.fz.list" => join(d.freezes.all(), |x| format!("{}:{}:{}:{}:{}", x.id, x.space, x.address, x.value, x.blocked)),
            "m.fz.clear" => {
                d.freezes.clear();
                "ok".into()
            }
            "m.label" => {
                d.labels.add(number(w(0)) as i32, w(1), None);
                "ok".into()
            }
            "m.cov.arm" => match d.coverage(number(w(0)) as usize) {
                Some(c) => {
                    c.arm();
                    "ok".into()
                }
                None => "none".into(),
            },
            "m.cov.status" => d.coverage(number(w(0)) as usize).map_or("none".into(), |c| format!("{} {} {}", c.is_armed(), c.instructions(), c.entry_point_count())),
            "m.cov.query" => d.coverage(number(w(0)) as usize).map_or("none".into(), |c| {
                let (a, l, n) = (number(w(1)) as i32, number(w(2)) as i32, number(w(3)) as i32);
                let (x, t, f) = c.statistics(a, l);
                format!("{x} {t} {f} [{}]", join(c.executed_addresses(a, l, n), |v| v.to_string()))
            }),
            "m.cov.entries" => d.coverage(number(w(0)) as usize).map_or("none".into(), |c| join(c.entry_points(number(w(1)) as i32), |(a, k, n)| format!("{a}:{k}:{n}"))),
            "m.cs.bt" => d.call_stack().map_or("none".into(), |s| join(s.backtrace().take(number(w(0)) as usize), |f| format!("{}:{}:{}:{}", f.source, f.target, f.kind as i32, f.frame))),
            "m.cs.status" => d.call_stack().map_or("none".into(), |s| format!("{} {} {} {}", s.is_profiling(), s.profiled_instructions(), s.unmatched_returns(), s.depth())),
            "m.cs.profile" => d.call_stack().map_or("none".into(), |s| {
                if w(0) == "on" { s.arm_profiler() } else { s.disarm_profiler() }
                "ok".into()
            }),
            "m.cs.hot" => d.call_stack().map_or("none".into(), |s| join(s.hottest(number(w(0)) as i32), |(a, n, c, _)| format!("{a}:{n}:{c}"))),
            "m.cheat.poke" => d.cheats.add_ram_poke(w(0), number(w(1)) as i32, number(w(2)) as u8, "poke", true).map_or_else(|e| format!("! {e}"), |id| id.to_string()),
            "m.cheat.wide" => d
                .cheats
                .add(emusen_systems::cheats::Kind::RamPoke, vec![emusen_systems::cheats::Write { width: 2, ..emusen_systems::cheats::Write::poke(w(0), number(w(1)) as i32, number(w(2)) as u32, 2) }], None, "wide", true)
                .map_or_else(|e| format!("! {e}"), |id| id.to_string()),
            "m.cheat.patch" => d
                .cheats
                .add_rom_patch(number(w(0)) as i32, number(w(1)) as u8, (w(2) != "-").then(|| number(w(2)) as u8), "patch", true)
                .map_or_else(|e| format!("! {e}"), |id| id.to_string()),
            "m.cheat.master" => {
                d.cheats.set_master_enabled(w(0) == "1");
                "ok".into()
            }
            "m.cheat.clear" => {
                d.cheats.clear();
                "ok".into()
            }
            "m.cheat.apply" => {
                d.apply_cheats(m);
                "ok".into()
            }
            "m.palettes" => join(d.palettes(m), |(i, c)| format!("{i}:{}", c.iter().map(|(r, g, b)| format!("{r},{g},{b}")).collect::<Vec<_>>().join(","))),
            "m.sprites" => join(d.sprites(m), |s| format!("{}:{}:{}:{}:{}:{}:{}:{}:{}:{}", s.0, s.1, s.2, s.3, s.4, s.5, s.6, s.7, s.8, s.9)),
            "m.channels" => join(d.audio_channels(m), |c| format!("{}:{}:{}:{}:{}:{}", c.0, c.1, c.2, c.3, c.4, c.5)),
            "m.tiles" => {
                let (rgba, w, h) = d.tile_sheet(m);
                format!("{w} {h} {}", emusen_core_host::sha256::sha256_hex(&rgba))
            }
            "m.swatch" => {
                let (rgba, w, h) = d.palette_swatch(m);
                format!("{w} {h} {}", emusen_core_host::sha256::sha256_hex(&rgba))
            }
            "m.tilemap" => d.decode_tilemap_entry(m, w(0), number(w(1)) as i32),
            "m.tilepixels" => match d.decode_tile_pixels(m, w(0), number(w(1)) as i32, number(w(2)) as i32) {
                Ok(p) => p.iter().map(|b| format!("{b:X}")).collect(),
                Err(e) => format!("! {e}"),
            },
            "m.dma" => join(d.dma_channels(m), |c| format!("{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}", c.0, c.1, c.2, c.3, c.4, c.5, c.6, c.7, c.8, c.9, c.10)),
            "m.vectors" => join(d.interrupt_vectors().iter(), |v| format!("{}:{}:{}:{}", v.0, v.1, v.2, v.3)),
            "m.dmaname" => d.name_dma_destination(number(w(0)) as u8).unwrap_or_else(|| "-".into()),
            "m.maxsprites" => d.max_sprites().to_string(),
            "m.stride" => d.tilemap_entry_stride().to_string(),
            "m.summary" => {
                let (name, version) = self.display.clone();
                self.d.summary(self.machine.as_mut().unwrap(), &name, &version).replace('\n', "|")
            }
            "m.disasm" => {
                let listed = d.disassemble(m, w(0), number(w(1)) as i32, number(w(2)) as i32);
                join(listed.iter(), |i| {
                    let reference = d.reference(i.address).map_or("-".to_string(), |(k, t)| format!("{}/{t}", k.name()));
                    format!("{}:{}:{}:{}:{reference}", i.address, i.bytes.iter().map(|b| format!("{b:02X}")).collect::<String>(), i.mnemonic, i.operands)
                })
            }
            "m.physical" => d.resolve_physical(m, number(w(0)) as i32).map_or("-".into(), |(s, o, a)| format!("{s}:{o}:{a}")),
            "m.audio" => {
                let (samples, rate) = d.audio_samples(m);
                let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
                format!("{} {rate} {}", samples.len(), emusen_core_host::sha256::sha256_hex(&bytes))
            }
            "m.mute" => {
                d.set_channel_muted(m, number(w(0)) as i32, w(1) == "1");
                "ok".into()
            }
            _ => return None,
        })
    }
}
