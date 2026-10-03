//! The conformance kit's core suite (EmuSen_CoreAPI.md §12): C1-C8, C10 and C11 on a library loaded through the core
//! ABI v1 alone, and a library's sidecar (§7.1). Each case reports its verdict and its evidence; §21 records what each
//! checks and how.

pub mod api;
pub mod digest;
pub mod sidecar;

use std::collections::BTreeSet;
use std::path::Path;

use api::{Lib, Machine};
use digest::Digests;
use emusen_native::core::sys::{CAPABILITY_NAMES, CreateParams, EXPORTS, Event, FileEntry, FrameInfo, caps, event, status};
use emusen_native::core::{jsonw::Json, schema};
use emusen_native::json::{self, Value};

/// What the author supplies: the image, how many frames, settings text, files and the input.
pub struct Options {
    pub image: Vec<u8>,
    pub frames: u64,
    pub settings: String,
    pub files: Vec<(u32, Vec<u8>)>,
    pub script: Option<Script>,
}

/// The input, as Pharaoh's hold, release and tap: a bit mask held from a frame on, released from a frame on, or held for one frame.
#[derive(Clone, Debug, Default)]
pub struct Script {
    /// (frame, port, bits set, bits cleared), sorted by frame.
    pub steps: Vec<(u64, u32, u32, u32)>,
}

impl Script {
    /// Lines `hold F PORT MASK`, `release F PORT MASK`, `tap F PORT MASK`, the mask in hex; `#` begins a comment.
    pub fn parse(text: &str) -> Result<Script, String> {
        let mut steps = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let w: Vec<&str> = line.split_whitespace().collect();
            let bad = || format!("input line {}: {line:?} is not `hold|release|tap FRAME PORT MASK`", n + 1);
            if w.len() != 4 {
                return Err(bad());
            }
            let frame: u64 = w[1].parse().map_err(|_| bad())?;
            let port: u32 = w[2].parse().map_err(|_| bad())?;
            let mask = u32::from_str_radix(w[3].trim_start_matches("0x"), 16).map_err(|_| bad())?;
            match w[0] {
                "hold" => steps.push((frame, port, mask, 0)),
                "release" => steps.push((frame, port, 0, mask)),
                "tap" => {
                    steps.push((frame, port, mask, 0));
                    steps.push((frame + 1, port, 0, mask));
                }
                _ => return Err(bad()),
            }
        }
        steps.sort_by_key(|s| s.0);
        Ok(Script { steps })
    }

    /// Without a script: each of port 0's bits pressed in turn, eight frames on and eight off, so input paths are exercised the same on every run.
    pub fn default_for(bits: &[u32]) -> Script {
        let mut steps = Vec::new();
        for (i, &bit) in bits.iter().cycle().take(64).enumerate() {
            let f = 16 * i as u64 + 8;
            steps.push((f, 0, 1 << bit, 0));
            steps.push((f + 8, 0, 0, 1 << bit));
        }
        Script { steps }
    }
}

pub struct Case {
    pub id: &'static str,
    pub name: &'static str,
    pub passed: bool,
    pub evidence: Vec<String>,
}

pub struct Report {
    pub library: String,
    pub abi: String,
    pub id: String,
    pub version: String,
    pub frames: u64,
    pub cases: Vec<Case>,
}

impl Report {
    pub fn passed(&self) -> bool {
        !self.cases.is_empty() && self.cases.iter().all(|c| c.passed)
    }

    pub fn json(&self) -> String {
        let mut j = Json::new();
        j.begin_object().field_str("kit", concat!("emusen-core-conform ", env!("CARGO_PKG_VERSION"))).field_str("library", &self.library);
        j.field_str("abi", &self.abi).field_str("id", &self.id).field_str("version", &self.version).field_uint("frames", self.frames);
        j.field_bool("passed", self.passed()).key("cases").begin_array();
        for c in &self.cases {
            j.begin_object().field_str("id", c.id).field_str("name", c.name).field_bool("passed", c.passed).field_strs("evidence", &c.evidence).end_object();
        }
        j.end_array().end_object();
        j.finish()
    }
}

/// A case's findings: a failure is a line, a note is a line that does not fail it.
struct Check {
    fails: Vec<String>,
    notes: Vec<String>,
}

impl Check {
    fn new() -> Check {
        Check { fails: Vec::new(), notes: Vec::new() }
    }

    fn that(&mut self, ok: bool, what: impl FnOnce() -> String) {
        if !ok {
            self.fails.push(what());
        }
    }

    fn note(&mut self, what: impl Into<String>) {
        self.notes.push(what.into());
    }

    fn done(self, id: &'static str, name: &'static str) -> Case {
        let passed = self.fails.is_empty();
        let mut evidence = self.fails;
        evidence.extend(self.notes);
        Case { id, name, passed, evidence }
    }
}

/// What a run of frames observed: the digests, and the events and shapes C6 and C7 judge.
#[derive(Default)]
struct Trace {
    digests: Digests,
    problems: Vec<String>,
    state_size_events: Vec<u64>,
}

struct Session<'a> {
    lib: &'a Lib,
    opts: &'a Options,
    script: Script,
    statuses: BTreeSet<i32>,
}

impl<'a> Session<'a> {
    fn machine(&mut self, settings: &str) -> Result<Machine<'a>, String> {
        match self.lib.create(&self.opts.image, settings, &self.opts.files, 1) {
            Ok(m) => Ok(m),
            Err((code, text)) => {
                self.statuses.insert(code);
                Err(format!("create refused the image: status {code} ({text})"))
            }
        }
    }

    /// Frames `from` to `from + count` of the script on `m`; rendering skipped on odd frames when `skip_odd`.
    fn play(&mut self, m: &Machine<'_>, from: u64, count: u64, skip_odd: bool) -> Result<Trace, String> {
        let mut t = Trace::default();
        let mut masks = [0u32; 8];
        for &(f, port, set, clear) in &self.script.steps {
            if f < from && (port as usize) < masks.len() {
                masks[port as usize] = (masks[port as usize] | set) & !clear;
            }
        }
        for (port, &mask) in masks.iter().enumerate() {
            m.set_buttons(port as u32, mask, u32::MAX);
        }
        let mut shape: Option<(i32, i32)> = None;
        let mut rate: Option<i32> = None;
        let mut pending: Vec<(u64, i32)> = Vec::new();
        let mut rate_events: Vec<(u64, i32)> = Vec::new();
        let mut skipping = false;
        for f in from..from + count {
            for &(sf, port, set, clear) in self.script.steps.iter().filter(|s| s.0 == f) {
                if (port as usize) < masks.len() {
                    let before = masks[port as usize];
                    masks[port as usize] = (before | set) & !clear;
                    m.set_buttons(port, masks[port as usize], before ^ masks[port as usize]);
                }
                let _ = sf;
            }
            let skip = skip_odd && f % 2 == 1;
            if skip != skipping {
                m.set_options(skip as u32);
                skipping = skip;
            }
            if let Err(code) = m.advance() {
                self.statuses.insert(code);
                return Err(format!("advance failed at frame {f}: status {code} ({})", m.last_error()));
            }
            if !skip {
                m.present();
            }
            let events: Vec<Event> = m.events();
            for e in &events {
                match e.kind {
                    event::AUDIO_RATE => rate_events.push((f, e.a as i32)),
                    event::STATE_SIZE => t.state_size_events.push(f),
                    _ => {}
                }
            }
            if !skip {
                match m.frame_info() {
                    Err(code) => t.problems.push(format!("frame {f}: frame_info refused with {code}")),
                    Ok(fi) => {
                        let picture = m.frame();
                        t.digests.frame(&picture);
                        frame_checks(f, &fi, picture.len(), &mut t.problems);
                        let now = (fi.width, fi.height);
                        if shape.is_some_and(|s| s != now) && !events.iter().any(|e| e.kind == event::GEOMETRY && (e.a, e.b) == (now.0 as i64, now.1 as i64)) {
                            t.problems.push(format!("frame {f}: the picture became {}x{} with no GEOMETRY event before it", now.0, now.1));
                        }
                        shape = Some(now);
                    }
                }
            }
            for (samples, r) in m.drain_audio() {
                t.digests.sound(&samples);
                if rate.is_some_and(|old| old != r) {
                    pending.push((f, r));
                }
                rate = Some(r);
            }
        }
        for (f, r) in pending {
            if !rate_events.iter().any(|&(ef, er)| er == r && ef + 1 >= f && ef <= f + 1) {
                t.problems.push(format!("frame {f}: samples changed to {r} Hz with no AUDIO_RATE event of that rate in this frame or the next"));
            }
        }
        Ok(t)
    }

    fn state_digest(&mut self, m: &Machine<'_>) -> Result<u64, String> {
        m.save(0).map(|s| digest::frame_hash(&s)).map_err(|e| {
            self.statuses.insert(e as i32);
            format!("state_save failed with {e}")
        })
    }
}

fn frame_checks(f: u64, fi: &FrameInfo, copied: usize, problems: &mut Vec<String>) {
    if fi.bytes != copied as i64 {
        problems.push(format!("frame {f}: frame_info says {} bytes, frame_copy gives {copied}", fi.bytes));
    }
    if fi.format != 0 {
        problems.push(format!("frame {f}: format {} was not offered (only RGBA8888)", fi.format));
    }
    if fi.width <= 0 || fi.height <= 0 || (fi.stride as i64) * (fi.height as i64) > fi.bytes || fi.stride < fi.width * 4 {
        problems.push(format!("frame {f}: {}x{} with stride {} does not fit {} bytes", fi.width, fi.height, fi.stride, fi.bytes));
    }
}

fn parse(text: &str) -> Option<Value> {
    json::parse(text.as_bytes()).ok()
}

fn strs(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array).unwrap_or(&[]).iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()
}

/// Runs every case on the library at `path`.
pub fn run(path: &Path, opts: &Options) -> Report {
    let mut report = Report { library: path.display().to_string(), abi: String::new(), id: String::new(), version: String::new(), frames: opts.frames, cases: Vec::new() };
    let lib = match Lib::open(path) {
        Ok(lib) => lib,
        Err(why) => {
            report.cases.push(Case { id: "C1", name: "loading", passed: false, evidence: vec![why] });
            return report;
        }
    };
    let info_text = lib.info().unwrap_or_default();
    let info = parse(&info_text);
    report.abi = info.as_ref().and_then(|i| i.get("abi")).and_then(Value::as_str).unwrap_or("").to_owned();
    report.id = info.as_ref().and_then(|i| i.get("id")).and_then(Value::as_str).unwrap_or("").to_owned();
    report.version = info.as_ref().and_then(|i| i.get("version")).and_then(Value::as_str).unwrap_or("").to_owned();
    let bits: Vec<u32> = info
        .as_ref()
        .and_then(|i| i.get("systems")?.as_array()?.first()?.get("controllers")?.as_array()?.first()?.get("buttons")?.as_array().map(|b| b.to_vec()))
        .unwrap_or_default()
        .iter()
        .filter_map(|b| b.get("bit").and_then(Value::as_i64).map(|v| v as u32))
        .collect();
    let script = opts.script.clone().unwrap_or_else(|| Script::default_for(&bits));
    let mut s = Session { lib: &lib, opts, script, statuses: BTreeSet::new() };
    report.cases.push(c1(&lib));
    report.cases.push(c2(&lib, &info_text));
    report.cases.push(c3(&lib));
    report.cases.push(c4(&mut s));
    report.cases.push(c5(&mut s));
    report.cases.push(c6(&mut s));
    report.cases.push(c7(&mut s));
    report.cases.push(c8(&mut s));
    report.cases.push(c10(&mut s));
    report.cases.push(c11(&mut s));
    report
}

/// The `emusen_core_` symbols the library defines, where `nm` can list them.
fn listed_symbols(path: &Path) -> Option<BTreeSet<String>> {
    let args: &[&str] = if cfg!(target_os = "macos") { &["-gU"] } else if cfg!(windows) { return None } else { &["-D", "--defined-only"] };
    let out = std::process::Command::new("nm").args(args).arg(path).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.split_whitespace().last())
            .map(|n| n.strip_prefix('_').filter(|_| cfg!(target_os = "macos")).unwrap_or(n).to_owned())
            .filter(|n| n.starts_with("emusen_core_"))
            .collect(),
    )
}

fn c1(lib: &Lib) -> Case {
    let mut c = Check::new();
    let v = unsafe { (lib.f.abi_version)() };
    c.that(v >> 16 == 1, || format!("abi_version is {v:#x}: major {}, not 1", v >> 16));
    c.note(format!("core ABI {}.{}; every required export resolves", v >> 16, v & 0xFFFF));
    match listed_symbols(&lib.path) {
        Some(names) => {
            let known: BTreeSet<&str> = EXPORTS.iter().map(|e| e.0).collect();
            for n in &names {
                c.that(known.contains(n.as_str()), || format!("{n} is exported but is no export of version 1"));
            }
            c.note(format!("{} emusen_core_ symbols listed by nm", names.len()));
        }
        None => c.note("nm is not available here: the symbol listing was not checked"),
    }
    c.done("C1", "loading")
}

fn c2(lib: &Lib, info: &str) -> Case {
    let mut c = Check::new();
    for e in schema::validate(schema::INFO, info) {
        c.that(false, || format!("info: {e}"));
    }
    let v = unsafe { (lib.f.abi_version)() };
    let doc = parse(info);
    let abi = doc.as_ref().and_then(|d| d.get("abi")).and_then(Value::as_str).unwrap_or("");
    c.that(abi == format!("{}.{}", v >> 16, v & 0xFFFF), || format!("info's abi {abi:?} is not the export's {}.{}", v >> 16, v & 0xFFFF));
    let claimed: BTreeSet<String> = CAPABILITY_NAMES.iter().filter(|(b, _)| lib.claims(*b)).map(|(_, n)| n.to_string()).collect();
    let listed: BTreeSet<String> = strs(doc.as_ref().and_then(|d| d.get("capabilities"))).into_iter().filter(|n| CAPABILITY_NAMES.iter().any(|(_, k)| k == n)).collect();
    c.that(claimed == listed, || format!("info lists {listed:?}, the bits claim {claimed:?}"));
    for &(name, bit) in EXPORTS.iter().filter(|e| e.1 != 0) {
        let present = lib.has(name);
        c.that(present == lib.claims(bit), || format!("{name} is {} but its bit is {}", if present { "exported" } else { "missing" }, if lib.claims(bit) { "claimed" } else { "not claimed" }));
    }
    c.note(format!("capabilities {:#x}", lib.capabilities()));
    c.done("C2", "info")
}

fn c3(lib: &Lib) -> Case {
    let mut c = Check::new();
    let text = match lib.settings() {
        Ok(t) => t,
        Err(e) => {
            c.that(false, || format!("settings_schema failed with {e}"));
            return c.done("C3", "the schema");
        }
    };
    for e in schema::validate(schema::SETTINGS, &text) {
        c.that(false, || format!("schema: {e}"));
    }
    let list = parse(&text).and_then(|v| v.as_array().map(|a| a.to_vec())).unwrap_or_default();
    let mut keys = BTreeSet::new();
    for s in &list {
        let get = |k: &str| s.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
        let (key, kind, default, effect) = (get("key"), get("kind"), get("default"), get("effect"));
        c.that(keys.insert(key.clone()), || format!("{key} appears twice"));
        let in_domain = match kind.as_str() {
            "switch" => default == "true" || default == "false",
            "count" => default.parse::<i64>().is_ok_and(|d| d >= s.get("min").and_then(Value::as_i64).unwrap_or(i64::MIN) && d <= s.get("max").and_then(Value::as_i64).unwrap_or(i64::MAX)),
            "choice" => s.get("choices").and_then(Value::as_array).unwrap_or(&[]).iter().any(|ch| ch.get("value").and_then(Value::as_str) == Some(default.as_str())),
            _ => !default.contains(['\r', '\n']),
        };
        c.that(in_domain, || format!("{key}: the default {default:?} is outside its own domain"));
        if effect == "accuracy" {
            c.that(get("accurate") == default, || format!("{key}: an accuracy setting's default must be its accurate value"));
        }
        if effect == "enhancement" {
            c.that(get("hardware") == default, || format!("{key}: an enhancement's default must be the hardware's value"));
        }
        if matches!(effect.as_str(), "accuracy" | "latency" | "enhancement") {
            c.that(!get("cost").trim().is_empty(), || format!("{key}: a trade-off needs its cost"));
        }
    }
    c.note(format!("{} settings", list.len()));
    c.done("C3", "the schema")
}

fn defaults(lib: &Lib) -> String {
    let list = lib.settings().ok().and_then(|t| parse(&t)).and_then(|v| v.as_array().map(|a| a.to_vec())).unwrap_or_default();
    list.iter()
        .filter_map(|s| Some(format!("{}={}\n", s.get("key")?.as_str()?, s.get("default")?.as_str()?)))
        .collect()
}

fn c4(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let mut garbage = vec![0u8; 4096];
    let mut x: u32 = 0x2545F491;
    for b in &mut garbage {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *b = x as u8;
    }
    let half = s.opts.image[..s.opts.image.len() / 2].to_vec();
    for (what, image) in [("an empty image", Vec::new()), ("a garbage image", garbage), ("the image truncated to half", half)] {
        match s.lib.create(&image, "", &s.opts.files, 1) {
            Ok(_) => c.that(false, || format!("{what} was accepted")),
            Err((code, text)) => {
                s.statuses.insert(code);
                c.that(code < 0, || format!("{what}: refused with the non-negative status {code}"));
                c.that(s.lib.words(code).is_some(), || format!("{what}: status_text has no words for {code}"));
                c.that(!text.is_empty(), || format!("{what}: the refusal left no error text"));
                c.note(format!("{what}: {code}, {text:?}"));
            }
        }
    }
    let stated = defaults(s.lib);
    let frames = s.opts.frames.min(300);
    let run = |s: &mut Session<'_>, settings: &str| -> Result<(Digests, u64), String> {
        let m = s.machine(settings)?;
        let t = s.play(&m, 0, frames, false)?;
        Ok((t.digests, s.state_digest(&m)?))
    };
    match (run(s, ""), run(s, &stated)) {
        (Ok(a), Ok(b)) => c.that(a == b, || format!("no settings and every default stated differ over {frames} frames: {a:?} against {b:?}")),
        (Err(e), _) | (_, Err(e)) => c.that(false, || e),
    }
    c.done("C4", "create and refusal")
}

fn c5(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let m = match s.machine(&s.opts.settings.clone()) {
        Ok(m) => m,
        Err(e) => {
            c.that(false, || e);
            return c.done("C5", "capability honesty");
        }
    };
    let _ = s.play(&m, 0, 2, false);
    let f = &s.lib.f;
    let h = m.h;
    let mi = m.machine_info().ok().and_then(|t| parse(&t));
    let spaces: Vec<i64> = mi.as_ref().and_then(|v| v.get("spaces")?.as_array().map(|a| a.to_vec())).unwrap_or_default().iter().filter_map(|x| x.get("id").and_then(Value::as_i64)).collect();
    let processors = mi.as_ref().and_then(|v| v.get("processors")?.as_array().map(|a| a.len())).unwrap_or(0);
    let mut claim = |bit: u64, name: &str, result: Option<i64>| {
        if s.lib.claims(bit) {
            match result {
                Some(r) => c.that(r >= 0 && r != status::NOT_SUPPORTED as i64, || format!("{name} is claimed and answered {r}")),
                None => c.that(false, || format!("{name} is claimed and not exported")),
            }
        }
    };
    unsafe {
        claim(caps::RESET, "RESET", f.reset.map(|g| g(h) as i64));
        claim(caps::PRESENT, "PRESENT", f.present.map(|g| g(h) as i64));
        claim(caps::SNAPSHOT, "SNAPSHOT", Some((f.state_size)(h, 1)));
        claim(caps::AUDIO_PEEK, "AUDIO_PEEK", f.audio_peek.map(|g| g(h, std::ptr::null_mut(), 0)));
        claim(caps::MUTES, "MUTES", f.set_mutes.map(|g| g(h, 0) as i64));
        claim(caps::AXES, "AXES", f.set_axis.map(|g| g(h, 0, 0, 0.0) as i64));
        claim(caps::SETTINGS, "SETTINGS", f.set_settings.map(|g| g(h, std::ptr::null(), 0) as i64));
        claim(caps::PHASES, "PHASES", f.phases.map(|g| g(h, std::ptr::null_mut(), 0)));
        claim(caps::ROM_PATCHES, "ROM_PATCHES", f.set_rom_patches.map(|g| g(h, std::ptr::null(), 0)));
        claim(caps::CHEAT_POKES, "CHEAT_POKES", f.set_cheat_pokes.map(|g| g(h, std::ptr::null(), 0)));
        claim(caps::SETTING_NOTES, "SETTING_NOTES", f.setting_notes.map(|g| g(h, std::ptr::null_mut(), 0)));
        claim(caps::DEBUG, "DEBUG", f.debug_set.map(|g| g(h, 0, i32::MIN, -1) as i64));
        claim(caps::DEBUG_STACK, "DEBUG_STACK", f.debug_set_stack.map(|g| g(h, std::ptr::null(), 0) as i64));
        if processors > 0 {
            claim(caps::DEBUG_REGISTERS, "DEBUG_REGISTERS", f.debug_registers.map(|g| g(h, 0, std::ptr::null_mut(), 0)));
            if let Some(&space) = spaces.first() {
                claim(caps::DEBUG_DISASSEMBLE, "DEBUG_DISASSEMBLE", f.debug_disassemble.map(|g| g(h, 0, space as u32, 0, 1, std::ptr::null_mut(), 0)));
            }
        }
        let unknown_kind = (f.state_size)(h, 77);
        c.that(unknown_kind == status::NOT_SUPPORTED as i64, || format!("state kind 77 answered {unknown_kind}, not NOT_SUPPORTED"));
        if !s.lib.claims(caps::SNAPSHOT) {
            let k1 = (f.state_size)(h, 1);
            c.that(k1 == status::NOT_SUPPORTED as i64, || format!("the snapshot is not claimed and state kind 1 answered {k1}"));
        }
        let unknown_space = spaces.iter().max().map_or(1000, |m| m + 1000) as u32;
        let sz = (f.space_size)(h, unknown_space);
        c.that(sz < 0, || format!("space {unknown_space}, which machine info does not list, answered size {sz}"));
        let opt = (f.set_options)(h, !1);
        c.that(opt == 0, || format!("set_options with every reserved bit answered {opt}; reserved bits are ignored"));
        let port = (f.set_buttons)(h, 99, 1, 1);
        c.that(port == 0 || port == status::NO_SUCH_PORT, || format!("port 99 answered {port}, neither ignored nor NO_SUCH_PORT"));
    }
    drop(m);
    match s.lib.create(&s.opts.image, &s.opts.settings, &s.opts.files, 1 | 1 << 40) {
        Ok(m) => c.that(m.frame_info().map(|fi| fi.format == 0).unwrap_or(false), || "with an unknown pixel format offered, the picture is not RGBA8888".to_owned()),
        Err((code, _)) => c.that(false, || format!("create refused an unknown pixel-format bit with {code}")),
    }
    c.done("C5", "capability honesty")
}

fn c6(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let frames = s.opts.frames;
    match s.machine(&s.opts.settings.clone()).and_then(|m| s.play(&m, 0, frames, false)) {
        Ok(t) => {
            for p in t.problems {
                c.that(false, || p);
            }
            c.note(format!("{frames} frames, frames digest {:016X}, audio digest {:016X} over {} samples", t.digests.frames, t.digests.audio, t.digests.samples));
        }
        Err(e) => c.that(false, || e),
    }
    c.done("C6", "the frame")
}

fn c7(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let half = (s.opts.frames / 2).max(1);
    let body = |s: &mut Session<'_>, c: &mut Check| -> Result<(), String> {
        let settings = s.opts.settings.clone();
        let a = s.machine(&settings)?;
        let size0 = a.state_size(0);
        let mut sizes_moved = 0u64;
        for f in 0..half {
            s.play(&a, f, 1, false).map(|t| sizes_moved += t.state_size_events.len() as u64)?;
            let now = a.state_size(0);
            c.that(now == size0 || sizes_moved > 0, || format!("frame {f}: the state's size went from {size0} to {now} with no STATE_SIZE event"));
        }
        let saved = a.save(0).map_err(|e| format!("state_save failed with {e}"))?;
        c.that(a.load(&saved) == 0, || "the state just saved does not load".to_owned());
        let again = a.save(0).map_err(|e| format!("state_save failed with {e}"))?;
        c.that(saved == again, || "save, load, save is not byte-identical".to_owned());
        let mut x: u32 = 0x9E3779B9;
        let random: Vec<u8> = (0..saved.len().max(16)).map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x as u8
        }).collect();
        for (what, bad) in [("a truncated state", saved[..saved.len() / 2].to_vec()), ("a foreign state", random), ("an empty state", Vec::new())] {
            let r = a.load(&bad);
            s.statuses.insert(r);
            c.that(r != 0, || format!("{what} was accepted"));
            c.that(a.save(0).ok().as_ref() == Some(&saved), || format!("{what} was refused but changed the machine"));
        }
        let b = s.machine(&settings)?;
        c.that(b.load(&saved) == 0, || "machine A's state does not load on machine B".to_owned());
        let ta = s.play(&a, half, half, false)?;
        let tb = s.play(&b, half, half, false)?;
        c.that(ta.digests == tb.digests && s.state_digest(&a)? == s.state_digest(&b)?, || format!("B, loaded with A's state, did not continue as A: {:?} against {:?}", ta.digests, tb.digests));
        if a.has_snapshot() {
            let snap = a.save(1).map_err(|e| format!("the snapshot could not be saved: {e}"))?;
            let full = a.save(0).map_err(|e| format!("state_save failed with {e}"))?;
            s.play(&a, 2 * half, 2, false)?;
            c.that(a.load(&snap) == 0, || "the snapshot does not load".to_owned());
            c.that(a.save(0).ok().as_ref() == Some(&full), || "loading the snapshot does not restore the state it was taken from".to_owned());
        }
        c.note(format!("state {} bytes", saved.len()));
        Ok(())
    };
    if let Err(e) = body(s, &mut c) {
        c.that(false, || e);
    }
    c.done("C7", "state")
}

fn c8(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let n = s.opts.frames.max(2);
    let half = n / 2;
    let body = |s: &mut Session<'_>, c: &mut Check| -> Result<(), String> {
        let settings = s.opts.settings.clone();
        let a = s.machine(&settings)?;
        let a1 = s.play(&a, 0, half, false)?.digests;
        let mid = a.save(0).map_err(|e| format!("state_save failed with {e}"))?;
        let a2 = s.play(&a, half, n - half, false)?.digests;
        let end = s.state_digest(&a)?;
        let b = s.machine(&settings)?;
        let b1 = s.play(&b, 0, half, false)?.digests;
        let b2 = s.play(&b, half, n - half, false)?.digests;
        c.that((a1, a2, end) == (b1, b2, s.state_digest(&b)?), || format!("two machines fed alike differ: {a1:?} {a2:?} against {b1:?} {b2:?}"));
        let d = s.machine(&settings)?;
        c.that(d.load(&mid) == 0, || "the state at the middle does not load".to_owned());
        let d2 = s.play(&d, half, n - half, false)?.digests;
        c.that(d2 == a2 && s.state_digest(&d)? == end, || format!("across a save and load at frame {half} the run differs: {d2:?} against {a2:?}"));
        let neutral = a.machine_info().ok().and_then(|t| parse(&t)).and_then(|v| v.get("skip_rendering_state_neutral").cloned()) == Some(Value::Bool(true));
        if neutral {
            let e = s.machine(&settings)?;
            s.play(&e, 0, n, true)?;
            c.that(s.state_digest(&e)? == end, || "with rendering skipped on alternate frames the state differs, though skipping is declared state-neutral".to_owned());
        }
        c.note(format!("{n} frames; frames {:016X} {:016X}, audio {:016X} {:016X}, state {end:016X}{}", a1.frames, a2.frames, a1.audio, a2.audio, if neutral { "; skipped rendering checked" } else { "" }));
        Ok(())
    };
    if let Err(e) = body(s, &mut c) {
        c.that(false, || e);
    }
    c.done("C8", "determinism")
}

fn c10(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let n = s.opts.frames.min(300).max(2);
    let settings = s.opts.settings.clone();
    let solo = (|| -> Result<(Digests, u64), String> {
        let m = s.machine(&settings)?;
        let t = s.play(&m, 0, n, false)?;
        Ok((t.digests, s.state_digest(&m)?))
    })();
    let solo = match solo {
        Ok(v) => v,
        Err(e) => {
            c.that(false, || e);
            return c.done("C10", "isolation");
        }
    };
    let (lib, opts, script) = (s.lib, s.opts, s.script.clone());
    let results: Vec<Result<(Digests, u64), String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..2)
            .map(|i| {
                let (settings, script) = (settings.clone(), script.clone());
                scope.spawn(move || -> Result<(Digests, u64), String> {
                    let mut own = Session { lib, opts, script, statuses: BTreeSet::new() };
                    let m = own.machine(&settings)?;
                    let t = own.play(&m, 0, n, false)?;
                    let d = own.state_digest(&m)?;
                    if i == 1 {
                        drop(m);
                    }
                    Ok((t.digests, d))
                })
            })
            .collect();
        let early = s.machine(&settings).ok();
        drop(early);
        handles.into_iter().map(|h| h.join().unwrap_or_else(|_| Err("a thread panicked".to_owned()))).collect()
    });
    for (i, r) in results.into_iter().enumerate() {
        match r {
            Ok(v) => c.that(v == solo, || format!("machine {i}, run beside another on its own thread, differs from the solo run: {v:?} against {solo:?}")),
            Err(e) => c.that(false, || e),
        }
    }
    c.note(format!("two machines of {n} frames on two threads, a third created and freed while they ran"));
    c.done("C10", "isolation")
}

fn c11(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let f = &s.lib.f;
    let n = std::ptr::null_mut();
    let null = status::NULL as i64;
    unsafe {
        let calls: Vec<(&str, i64)> = vec![
            ("free", (f.free)(n) as i64),
            ("machine_info", (f.machine_info)(n, std::ptr::null_mut(), 0)),
            ("last_error", (f.last_error)(n, std::ptr::null_mut(), 0)),
            ("advance", (f.advance)(n, std::ptr::null_mut()) as i64),
            ("set_options", (f.set_options)(n, 0) as i64),
            ("frame_count", (f.frame_count)(n)),
            ("events", (f.events)(n, std::ptr::null_mut(), 0, 24)),
            ("frame_info", (f.frame_info)(n, std::ptr::null_mut()) as i64),
            ("frame_copy", (f.frame_copy)(n, std::ptr::null_mut(), 0)),
            ("audio_rate", (f.audio_rate)(n) as i64),
            ("audio_buffered", (f.audio_buffered)(n)),
            ("audio_drain", (f.audio_drain)(n, std::ptr::null_mut(), 0, 0, std::ptr::null_mut())),
            ("set_audio_limit", (f.set_audio_limit)(n, 0) as i64),
            ("set_buttons", (f.set_buttons)(n, 0, 0, 0) as i64),
            ("state_size", (f.state_size)(n, 0)),
            ("state_save", (f.state_save)(n, 0, std::ptr::null_mut(), 0)),
            ("state_load", (f.state_load)(n, std::ptr::null(), 0) as i64),
            ("state_layout", (f.state_layout)(n, 0, std::ptr::null_mut(), 0)),
            ("space_size", (f.space_size)(n, 0)),
            ("space_read", (f.space_read)(n, 0, 0, std::ptr::null_mut(), 0)),
            ("space_write", (f.space_write)(n, 0, 0, std::ptr::null(), 0)),
            ("battery", (f.battery)(n, 0, std::ptr::null_mut(), 0, std::ptr::null_mut())),
            ("battery_saved", (f.battery_saved)(n, 0) as i64),
        ];
        for (name, r) in calls {
            c.that(r == null, || format!("{name} with a null machine answered {r}, not EMUSEN_NULL"));
        }
        let mut code = 0;
        c.that((f.create)(std::ptr::null(), &mut code).is_null() && code == status::NULL, || format!("create with null params answered status {code}"));
        let whole = (f.info)(std::ptr::null_mut(), 0);
        let mut one = [0u8; 1];
        c.that((f.info)(one.as_mut_ptr(), 1) == whole && whole > 1, || "info with a one-byte buffer does not answer the whole length".to_owned());
    }
    let small_params = |size: u32, file_size: usize| -> i32 {
        let file = FileEntry { size: 24, which: 0, data: std::ptr::null(), len: 0 };
        let p = CreateParams {
            size,
            host_abi_version: emusen_native::core::sys::ABI_VERSION,
            image: s.opts.image.as_ptr(),
            image_len: s.opts.image.len(),
            settings: std::ptr::null(),
            settings_len: 0,
            files: &file,
            file_count: 1,
            file_size,
            pixel_formats: 1,
            error: std::ptr::null_mut(),
            error_len: 0,
        };
        let mut code = 0;
        let h = unsafe { (s.lib.f.create)(&p, &mut code) };
        if !h.is_null() {
            unsafe { (s.lib.f.free)(h) };
        }
        code
    };
    let short = small_params(80, 24);
    c.that(short == status::BAD_STRUCT, || format!("create_params of 80 bytes answered {short}, not BAD_STRUCT"));
    let short_file = small_params(88, 16);
    c.that(short_file == status::BAD_STRUCT, || format!("a file element of 16 bytes answered {short_file}, not BAD_STRUCT"));
    match s.machine(&s.opts.settings.clone()) {
        Ok(m) => {
            let mut fi = FrameInfo { size: 48, ..FrameInfo::default() };
            let r = unsafe { (s.lib.f.frame_info)(m.h, &mut fi) };
            c.that(r == status::BAD_STRUCT, || format!("frame_info of 48 bytes answered {r}, not BAD_STRUCT"));
            let mut ev = [Event::default(); 2];
            let r = unsafe { (s.lib.f.events)(m.h, ev.as_mut_ptr(), 2, 16) };
            c.that(r == status::BAD_STRUCT as i64, || format!("events of 16 bytes answered {r}, not BAD_STRUCT"));
        }
        Err(e) => c.that(false, || e),
    }
    let returned: Vec<i32> = s.statuses.iter().copied().filter(|&x| x < 0).collect();
    for code in &returned {
        c.that(s.lib.words(*code).is_some(), || format!("status_text has no words for {code}, which the core returned"));
    }
    c.note(format!("status_text answered for every code returned: {returned:?}"));
    c.done("C11", "error paths")
}
