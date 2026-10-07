//! The conformance kit's core suite (EmuSen_CoreAPI.md §12): C1-C15 on a library loaded through the core
//! ABI v1 alone, and a library's sidecar (§7.1). Each case reports its verdict and its evidence; §21 and §24 record what each
//! checks and how, and §28 the corpus form, which runs the checks that do not depend on the image once.

pub mod api;
pub mod corpus;
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
#[derive(Clone)]
pub struct Options {
    /// The system pack declares the image format self-delimiting: it records its own length or a checksum the core
    /// must verify, so a malformed image must be refused (§12.1, C4).
    pub self_delimiting: bool,
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
    /// C4's malformed images: what each was, and whether it was refused or accepted and run.
    pub images: Vec<ImageOutcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Refused at create, with its status and error text.
    Refused { status: i32, text: String },
    /// Created and run for every frame asked.
    AcceptedAndRan { frames: u64 },
    /// Created, and a frame refused with a status: a clean stop, not harm.
    AcceptedAndStopped { frame: u64, status: i32 },
    /// Created, and the frames did not finish within the deadline.
    Hung { seconds: u64 },
}

impl Outcome {
    pub fn name(&self) -> &'static str {
        match self {
            Outcome::Refused { .. } => "refused",
            Outcome::AcceptedAndRan { .. } => "accepted and ran",
            Outcome::AcceptedAndStopped { .. } => "accepted and stopped",
            Outcome::Hung { .. } => "hung",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageOutcome {
    pub image: &'static str,
    pub outcome: Outcome,
}

pub struct Report {
    pub library: String,
    /// The image's name, in a corpus run's report for one image; a single-image report carries none.
    pub image: Option<String>,
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
        if let Some(image) = &self.image {
            j.field_str("image", image);
        }
        j.field_str("abi", &self.abi).field_str("id", &self.id).field_str("version", &self.version).field_uint("frames", self.frames);
        j.field_bool("passed", self.passed()).key("cases");
        cases_json(&mut j, &self.cases);
        j.end_object();
        j.finish()
    }
}

/// The cases as a report lists them: each with its verdict and evidence, and C4's images with their outcomes.
pub(crate) fn cases_json(j: &mut Json, cases: &[Case]) {
    j.begin_array();
    for c in cases {
        j.begin_object().field_str("id", c.id).field_str("name", c.name).field_bool("passed", c.passed).field_strs("evidence", &c.evidence);
        if !c.images.is_empty() {
            j.key("images").begin_array();
            for i in &c.images {
                j.begin_object().field_str("image", i.image).field_str("outcome", i.outcome.name());
                match &i.outcome {
                    Outcome::Refused { status, text } => j.field_int("status", *status as i64).field_str("text", text),
                    Outcome::AcceptedAndRan { frames } => j.field_uint("frames", *frames),
                    Outcome::AcceptedAndStopped { frame, status } => j.field_uint("frame", *frame).field_int("status", *status as i64),
                    Outcome::Hung { seconds } => j.field_uint("seconds", *seconds),
                };
                j.end_object();
            }
            j.end_array();
        }
        j.end_object();
    }
    j.end_array();
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
        Case { id, name, passed, evidence, images: Vec::new() }
    }
}

/// What a run of frames observed: the digests, and the events and shapes C6 and C7 judge.
#[derive(Default)]
struct Trace {
    digests: Digests,
    problems: Vec<String>,
    state_size_events: Vec<u64>,
    /// At the marked frame: the digests up to it and the state saved there; `since` is the digests from it on.
    mark: Option<(Digests, Result<Vec<u8>, i64>)>,
    since: Digests,
}

/// C6's run as C8's first: both halves' digests, the state between them, the end state and machine info.
struct FirstRun {
    frames: u64,
    first: Digests,
    second: Digests,
    middle: Vec<u8>,
    end: u64,
    info: Option<String>,
}

struct Session<'a> {
    lib: &'a Lib,
    opts: &'a Options,
    script: Script,
    statuses: BTreeSet<i32>,
    /// Frames advance through `debug_run_frame` with the tables `arm` set, for C15.
    observed: bool,
    /// C6's run, kept for the C8 that follows it.
    first: Option<FirstRun>,
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
        self.play_marked(m, from, count, skip_odd, None)
    }

    /// `play`, and before frame `mark` the state saved and the buttons stated again, as a run split there would do.
    fn play_marked(&mut self, m: &Machine<'_>, from: u64, count: u64, skip_odd: bool, mark: Option<u64>) -> Result<Trace, String> {
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
            if mark == Some(f) {
                t.mark = Some((t.digests, m.save(0)));
                t.since = Digests::default();
                for (port, &mask) in masks.iter().enumerate() {
                    m.set_buttons(port as u32, mask, u32::MAX);
                }
            }
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
            if self.observed {
                observed_frame(m).map_err(|e| format!("frame {f}: {e}"))?;
            } else if let Err(code) = m.advance() {
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
                        let hash = digest::frame_hash(&picture);
                        t.digests.frame_hashed(hash);
                        t.since.frame_hashed(hash);
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
                t.since.sound(&samples);
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

/// A report's heading for the library at `path`, its cases still to come.
pub(crate) fn heading(path: &Path, lib: Option<&Lib>, frames: u64) -> Report {
    let info = lib.and_then(|l| l.info().ok()).and_then(|t| parse(&t));
    let field = |k: &str| info.as_ref().and_then(|i| i.get(k)).and_then(Value::as_str).unwrap_or("").to_owned();
    Report { library: path.display().to_string(), image: None, abi: field("abi"), id: field("id"), version: field("version"), frames, cases: Vec::new() }
}

/// A session on `lib` with the author's input, or port 0's buttons of the info's first controller in turn.
fn session<'a>(lib: &'a Lib, opts: &'a Options) -> Session<'a> {
    let bits: Vec<u32> = lib
        .info()
        .ok()
        .and_then(|t| parse(&t))
        .as_ref()
        .and_then(|i| i.get("systems")?.as_array()?.first()?.get("controllers")?.as_array()?.first()?.get("buttons")?.as_array().map(|b| b.to_vec()))
        .unwrap_or_default()
        .iter()
        .filter_map(|b| b.get("bit").and_then(Value::as_i64).map(|v| v as u32))
        .collect();
    let script = opts.script.clone().unwrap_or_else(|| Script::default_for(&bits));
    Session { lib, opts, script, statuses: BTreeSet::new(), observed: false, first: None }
}

/// Runs every case on the library at `path`.
pub fn run(path: &Path, opts: &Options) -> Report {
    let lib = match Lib::open(path) {
        Ok(lib) => lib,
        Err(why) => {
            let mut report = heading(path, None, opts.frames);
            report.cases.push(Case { id: "C1", name: "loading", passed: false, evidence: vec![why], images: Vec::new() });
            return report;
        }
    };
    let mut report = heading(path, Some(&lib), opts.frames);
    let info_text = lib.info().unwrap_or_default();
    let mut s = session(&lib, opts);
    report.cases.push(c1(&lib));
    report.cases.push(c2(&lib, &info_text));
    report.cases.push(c3(&lib));
    report.cases.push(c4(&mut s, path, C4::ALL));
    report.cases.push(c5(&mut s));
    report.cases.push(c6(&mut s));
    report.cases.push(c7(&mut s));
    report.cases.push(c8(&mut s));
    report.cases.push(c9(&mut s));
    report.cases.push(c10(&mut s));
    report.cases.push(c11(&mut s));
    report.cases.push(c12(&mut s));
    report.cases.push(c13(&mut s));
    report.cases.push(c14(&mut s));
    report.cases.push(c15(&mut s));
    report
}

/// The cases whose answer does not depend on the image, run on one image of the author's: C1-C3, C4's empty and
/// garbage images and its defaults, C5, C9, C10, C12, C13 and C15. C11 follows the images, since it needs every
/// status the run returned (`core_c11`). With the library's own report heading, and the statuses returned so far.
pub(crate) fn core_cases(path: &Path, opts: &Options) -> (Report, BTreeSet<i32>) {
    let lib = match Lib::open(path) {
        Ok(lib) => lib,
        Err(why) => {
            let mut report = heading(path, None, opts.frames);
            report.cases.push(Case { id: "C1", name: "loading", passed: false, evidence: vec![why], images: Vec::new() });
            return (report, BTreeSet::new());
        }
    };
    let mut report = heading(path, Some(&lib), opts.frames);
    let info_text = lib.info().unwrap_or_default();
    let mut s = session(&lib, opts);
    report.cases.push(c1(&lib));
    report.cases.push(c2(&lib, &info_text));
    report.cases.push(c3(&lib));
    report.cases.push(c4(&mut s, path, C4::CORE));
    report.cases.push(c5(&mut s));
    report.cases.push(c9(&mut s));
    report.cases.push(c10(&mut s));
    report.cases.push(c12(&mut s));
    report.cases.push(c13(&mut s));
    report.cases.push(c15(&mut s));
    (report, s.statuses)
}

/// C11 on the reference image, with every status the per-core and per-image cases returned.
pub(crate) fn core_c11(path: &Path, opts: &Options, statuses: BTreeSet<i32>) -> Case {
    match Lib::open(path) {
        Ok(lib) => {
            let mut s = session(&lib, opts);
            s.statuses = statuses;
            c11(&mut s)
        }
        Err(why) => Case { id: "C11", name: "error paths", passed: false, evidence: vec![why], images: Vec::new() },
    }
}

/// The cases that depend on the image: C4's half image and firmware rule, C6, C7, C8 (its first run C6's) and C14.
pub(crate) fn image_cases(lib: &Lib, path: &Path, opts: &Options) -> (Vec<Case>, BTreeSet<i32>) {
    let mut s = session(lib, opts);
    let cases = vec![c4(&mut s, path, C4::IMAGE), c6(&mut s), c7(&mut s), c8(&mut s), c14(&mut s)];
    (cases, s.statuses)
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

/// The systems an info marks in development, and why one could still be offered: an extension it shares with a system
/// the core offers, which a host routing by extension would offer it by (EmuSen_CoreAPI.md §27.4).
pub fn systems_in_development(info: &str) -> (Vec<String>, Vec<String>) {
    let systems = parse(info).as_ref().and_then(|d| d.get("systems")).and_then(Value::as_array).map(|a| a.to_vec()).unwrap_or_default();
    let developing = |s: &Value| matches!(s.get("development"), Some(Value::Bool(true)));
    let offered: BTreeSet<String> = systems.iter().filter(|s| !developing(s)).flat_map(|s| strs(s.get("extensions"))).map(|e| e.to_lowercase()).collect();
    let (mut problems, mut ids) = (Vec::new(), Vec::new());
    for s in systems.iter().filter(|s| developing(s)) {
        let id = s.get("id").and_then(Value::as_str).unwrap_or("").to_owned();
        for e in strs(s.get("extensions")).into_iter().filter(|e| offered.contains(&e.to_lowercase())) {
            problems.push(format!("system {id} is in development but {e} is also an offered system's, so a host would offer it"));
        }
        ids.push(id);
    }
    (problems, ids)
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
    let (problems, developing) = systems_in_development(info);
    for p in problems {
        c.that(false, || p.clone());
    }
    for id in developing {
        c.note(format!("system {id} in development, offered to no player"));
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

/// How long an accepted malformed image may take over the kit's frames before it counts as a hang.
pub const MALFORMED_DEADLINE_SECONDS: u64 = 60;

/// A malformed image the core accepted, run on a thread of its own against a deadline, so a hang is reported and not waited on.
fn run_accepted(path: &Path, image: Vec<u8>, files: Vec<(u32, Vec<u8>)>, frames: u64) -> Result<Outcome, String> {
    // Leaked on purpose: a hung machine's thread keeps the library, which a host never unloads anyway (§6.16).
    let lib: &'static Lib = Box::leak(Box::new(Lib::open(path)?));
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let outcome = match lib.create(&image, "", &files, 1) {
            Err((status, text)) => Outcome::Refused { status, text },
            Ok(m) => {
                let mut stopped = None;
                for f in 0..frames {
                    if let Err(status) = m.advance() {
                        stopped = Some(Outcome::AcceptedAndStopped { frame: f, status });
                        break;
                    }
                    m.present();
                    let _ = (m.events(), m.frame(), m.drain_audio());
                }
                stopped.unwrap_or(Outcome::AcceptedAndRan { frames })
            }
        };
        let _ = tx.send(outcome);
    });
    Ok(rx.recv_timeout(std::time::Duration::from_secs(MALFORMED_DEADLINE_SECONDS)).unwrap_or(Outcome::Hung { seconds: MALFORMED_DEADLINE_SECONDS }))
}

/// C4's firmware rule (EmuSen_CoreAPI.md §6.2): a replacement's effect is one of the schema's words with a cost unless
/// exact, and an image whose entries are all optional is created with no files at all.
fn firmware_optional(s: &mut Session<'_>, c: &mut Check) {
    let image = s.opts.image.clone();
    let Ok(text) = api::text(|o, l| unsafe { (s.lib.f.firmware_for)(image.as_ptr(), image.len(), o, l) }) else { return };
    let list = parse(&text).and_then(|v| v.as_array().map(|a| a.to_vec())).unwrap_or_default();
    if list.is_empty() {
        return;
    }
    for f in &list {
        let name = f.get("name").and_then(Value::as_str).unwrap_or("?").to_owned();
        if let Some(r) = f.get("replacement") {
            let effect = r.get("effect").and_then(Value::as_str).unwrap_or("");
            c.that(matches!(effect, "exact" | "accuracy" | "none"), || format!("{name}: the replacement's effect {effect:?} is not exact, accuracy or none"));
            let cost = r.get("cost").and_then(Value::as_str).unwrap_or("");
            c.that(effect == "exact" || !cost.trim().is_empty(), || format!("{name}: a replacement short of exact needs its cost"));
        }
    }
    if list.iter().all(|f| matches!(f.get("required"), Some(Value::Bool(false)))) {
        match s.lib.create(&image, "", &[], 1) {
            Ok(_) => c.note(format!("every firmware entry optional ({}): created with no files", list.len())),
            Err((status, text)) => c.that(false, || format!("every firmware entry is optional, yet create with no files refused: {status}, {text:?}")),
        }
    }
}

/// Which of C4's parts run: the empty and garbage images and the defaults do not depend on the author's image, the
/// half image and the firmware rule do.
#[derive(Clone, Copy)]
struct C4 {
    empty_and_garbage: bool,
    half: bool,
    firmware: bool,
    defaults: bool,
}

impl C4 {
    const ALL: C4 = C4 { empty_and_garbage: true, half: true, firmware: true, defaults: true };
    const CORE: C4 = C4 { empty_and_garbage: true, half: false, firmware: false, defaults: true };
    const IMAGE: C4 = C4 { empty_and_garbage: false, half: true, firmware: true, defaults: false };
}

/// C4 as decided 2026-10-03: a malformed image never harms the core; it is refused with words, or, unless the system's
/// format is self-delimiting, it loads and runs the kit's frames without hanging.
fn c4(s: &mut Session<'_>, path: &Path, parts: C4) -> Case {
    let mut c = Check::new();
    let mut images = Vec::new();
    let mut garbage = vec![0u8; 4096];
    let mut x: u32 = 0x2545F491;
    for b in &mut garbage {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *b = x as u8;
    }
    let half = s.opts.image[..s.opts.image.len() / 2].to_vec();
    let frames = s.opts.frames.min(300);
    let malformed = [(parts.empty_and_garbage, "an empty image", Vec::new()), (parts.empty_and_garbage, "a garbage image", garbage), (parts.half, "the image truncated to half", half)];
    for (_, what, image) in malformed.into_iter().filter(|m| m.0) {
        let outcome = match s.lib.create(&image, "", &s.opts.files, 1) {
            Err((status, text)) => Outcome::Refused { status, text },
            Ok(m) => {
                drop(m);
                match run_accepted(path, image, s.opts.files.clone(), frames) {
                    Ok(o) => o,
                    Err(e) => {
                        c.that(false, || format!("{what}: {e}"));
                        continue;
                    }
                }
            }
        };
        match &outcome {
            Outcome::Refused { status, text } => {
                s.statuses.insert(*status);
                c.that(*status < 0, || format!("{what}: refused with the non-negative status {status}"));
                c.that(s.lib.words(*status).is_some(), || format!("{what}: status_text has no words for {status}"));
                c.that(!text.is_empty(), || format!("{what}: the refusal left no error text"));
                c.note(format!("{what}: refused, {status}, {text:?}"));
            }
            Outcome::Hung { seconds } => c.that(false, || format!("{what}: accepted, and its {frames} frames did not finish in {seconds} s")),
            accepted => {
                if let Outcome::AcceptedAndStopped { status, .. } = accepted {
                    s.statuses.insert(*status);
                }
                c.that(!s.opts.self_delimiting, || format!("{what}: accepted, though the system's image format is self-delimiting and a malformed image must be refused"));
                c.note(format!("{what}: {}", match accepted {
                    Outcome::AcceptedAndStopped { frame, status } => format!("accepted, and stopped cleanly at frame {frame} with status {status}"),
                    _ => format!("accepted and ran {frames} frames"),
                }));
            }
        }
        images.push(ImageOutcome { image: what, outcome });
    }
    if parts.firmware {
        firmware_optional(s, &mut c);
    }
    if parts.defaults {
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
    }
    Case { images, ..c.done("C4", "create and refusal") }
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
    // The run is also C8's first, so it saves its state at C8's middle; a run of one frame has none.
    let mark = (frames >= 2).then_some(frames / 2);
    s.first = None;
    let run = s.machine(&s.opts.settings.clone()).and_then(|m| {
        let t = s.play_marked(&m, 0, frames, false, mark)?;
        if let (Some((first, Ok(middle))), Ok(end)) = (t.mark.clone(), m.save(0)) {
            s.first = Some(FirstRun { frames, first, second: t.since, middle, end: digest::frame_hash(&end), info: m.machine_info().ok() });
        }
        Ok(t)
    });
    match run {
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
        // The first machine's run is C6's where C6 has just made it over the same frames (§28).
        let (a1, mid, a2, end, info) = match s.first.take().filter(|f| f.frames == n) {
            Some(f) => (f.first, f.middle, f.second, f.end, f.info),
            None => {
                let a = s.machine(&settings)?;
                let a1 = s.play(&a, 0, half, false)?.digests;
                let mid = a.save(0).map_err(|e| format!("state_save failed with {e}"))?;
                let a2 = s.play(&a, half, n - half, false)?.digests;
                (a1, mid, a2, s.state_digest(&a)?, a.machine_info().ok())
            }
        };
        let b = s.machine(&settings)?;
        let b1 = s.play(&b, 0, half, false)?.digests;
        let b2 = s.play(&b, half, n - half, false)?.digests;
        c.that((a1, a2, end) == (b1, b2, s.state_digest(&b)?), || format!("two machines fed alike differ: {a1:?} {a2:?} against {b1:?} {b2:?}"));
        let d = s.machine(&settings)?;
        c.that(d.load(&mid) == 0, || "the state at the middle does not load".to_owned());
        let d2 = s.play(&d, half, n - half, false)?.digests;
        c.that(d2 == a2 && s.state_digest(&d)? == end, || format!("across a save and load at frame {half} the run differs: {d2:?} against {a2:?}"));
        let neutral = info.and_then(|t| parse(&t)).and_then(|v| v.get("skip_rendering_state_neutral").cloned()) == Some(Value::Bool(true));
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
                    let mut own = Session { lib, opts, script, statuses: BTreeSet::new(), observed: false, first: None };
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

// ---- C9 and C12-C15 -----------------------------------------------------------------------------------------------

/// A run of up to 300 frames from a fresh machine: its digests and its state's.
fn solo(s: &mut Session<'_>, settings: &str, frames: u64) -> Result<(Digests, u64), String> {
    let m = s.machine(settings)?;
    let t = s.play(&m, 0, frames, false)?;
    Ok((t.digests, s.state_digest(&m)?))
}

/// The values a setting's domain is sampled at: both of a switch's, each choice, a count's bounds and middle.
fn sampled_values(setting: &Value) -> Vec<String> {
    let get = |k: &str| setting.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
    match get("kind").as_str() {
        "switch" => vec!["true".into(), "false".into()],
        "choice" => setting.get("choices").and_then(Value::as_array).unwrap_or(&[]).iter().filter_map(|c| c.get("value").and_then(Value::as_str).map(str::to_owned)).collect(),
        "count" => {
            let (lo, hi) = (setting.get("min").and_then(Value::as_i64).unwrap_or(0), setting.get("max").and_then(Value::as_i64).unwrap_or(0));
            let mut v = vec![lo, (lo + hi) / 2, hi];
            v.dedup();
            v.into_iter().map(|n| n.to_string()).collect()
        }
        _ => Vec::new(),
    }
}

fn c9(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let list = s.lib.settings().ok().and_then(|t| parse(&t)).and_then(|v| v.as_array().map(|a| a.to_vec())).unwrap_or_default();
    let exact: Vec<&Value> = list.iter().filter(|x| x.get("effect").and_then(Value::as_str) == Some("exact")).collect();
    if exact.is_empty() {
        c.note("no setting declares effect exact");
        return c.done("C9", "exact settings");
    }
    let frames = s.opts.frames.min(300);
    let base = match solo(s, "", frames) {
        Ok(v) => v,
        Err(e) => {
            c.that(false, || e);
            return c.done("C9", "exact settings");
        }
    };
    let mut runs = 0;
    for setting in exact {
        let key = setting.get("key").and_then(Value::as_str).unwrap_or("").to_owned();
        let default = setting.get("default").and_then(Value::as_str).unwrap_or("").to_owned();
        for value in sampled_values(setting).into_iter().filter(|v| *v != default) {
            runs += 1;
            match solo(s, &format!("{key}={value}"), frames) {
                Ok(v) => c.that(v == base, || format!("{key}={value} is declared exact and changes the output over {frames} frames: {v:?} against the default's {base:?}")),
                Err(e) => c.that(false, || format!("{key}={value}: {e}")),
            }
        }
    }
    c.note(format!("{runs} values of exact settings against the defaults over {frames} frames"));
    c.done("C9", "exact settings")
}

fn c12(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    for (minor, pad, what) in [(0u32, 0usize, "a host of minor 0 with version 1.0's struct sizes"), (99, 32, "a host of minor 99 whose structs are 32 bytes longer")] {
        s.lib.set_host(minor, pad);
        let before = s.lib.canaries_broken.load(std::sync::atomic::Ordering::Relaxed);
        let cases = [c6(s), c7(s), c8(s)];
        let damaged = s.lib.canaries_broken.load(std::sync::atomic::Ordering::Relaxed) - before;
        for case in &cases {
            c.that(case.passed, || format!("{what}: {} fails: {}", case.id, case.evidence.first().cloned().unwrap_or_default()));
        }
        c.that(damaged == 0, || format!("{what}: the core wrote {damaged} bytes past the structs' version 1.0 size"));
        if cases.iter().all(|x| x.passed) && damaged == 0 {
            c.note(format!("{what}: C6, C7 and C8 pass, no canary touched"));
        }
    }
    s.lib.set_host(0, 0);
    c.done("C12", "skew")
}

fn c13(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let lib = s.lib;
    let image = s.opts.image.clone();
    let firmware = |lib: &Lib| api::text(|o, l| unsafe { (lib.f.firmware_for)(image.as_ptr(), image.len(), o, l) });
    let codes = [status::NULL, status::FOREIGN, status::NOT_SUPPORTED, status::BAD_STRUCT];
    let answers = |lib: &Lib| (unsafe { (lib.f.abi_version)() }, lib.capabilities(), lib.info(), lib.settings(), firmware(lib), codes.map(|x| lib.words(x)));
    let reference = answers(lib);
    let frames = s.opts.frames.min(300);
    let alone = solo(s, &s.opts.settings.clone(), frames);
    let stop = std::sync::atomic::AtomicBool::new(false);
    let (beside, mismatches, calls) = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    let (mut wrong, mut n) = (Vec::new(), 0u64);
                    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                        let got = answers(lib);
                        let mut drain = vec![0u8; 1 << 16];
                        unsafe { (lib.f.log_drain)(std::ptr::null_mut(), drain.as_mut_ptr(), drain.len()) };
                        if got != reference && wrong.len() < 3 {
                            wrong.push(format!("a library-level answer changed while a machine ran: {:?}", if got.2 != reference.2 { "info" } else if got.3 != reference.3 { "settings_schema" } else if got.4 != reference.4 { "firmware_for" } else { "status_text, abi_version or capabilities" }));
                        }
                        n += 1;
                    }
                    (wrong, n)
                })
            })
            .collect();
        let run = solo(s, &s.opts.settings.clone(), frames);
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let mut all = Vec::new();
        let mut total = 0;
        for w in workers {
            let (wrong, n) = w.join().unwrap_or_default();
            all.extend(wrong);
            total += n;
        }
        (run, all, total)
    });
    for m in mismatches {
        c.that(false, || m);
    }
    match (alone, beside) {
        (Ok(a), Ok(b)) => c.that(a == b, || format!("a machine run while four threads called the library-level exports differs from one run alone: {b:?} against {a:?}")),
        (Err(e), _) | (_, Err(e)) => c.that(false, || e),
    }
    c.note(format!("{calls} rounds of library-level calls on four threads beside {frames} frames"));
    c.done("C13", "library-level concurrency")
}

fn c14(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    let m = match s.machine(&s.opts.settings.clone()) {
        Ok(m) => m,
        Err(e) => {
            c.that(false, || e);
            return c.done("C14", "descriptors against exports");
        }
    };
    let _ = s.play(&m, 0, 2, false);
    let Some(info) = m.machine_info().ok().and_then(|t| parse(&t)) else {
        c.that(false, || "machine info is not JSON".to_owned());
        return c.done("C14", "descriptors against exports");
    };
    let f = &s.lib.f;
    let list = |k: &str| info.get(k).and_then(Value::as_array).map(|a| a.to_vec()).unwrap_or_default();
    for space in list("spaces") {
        let id = space.get("id").and_then(Value::as_i64).unwrap_or(-1) as u32;
        let name = space.get("name").and_then(Value::as_str).unwrap_or("").to_owned();
        let declared = space.get("size").and_then(Value::as_i64).unwrap_or(-1);
        let size = unsafe { (f.space_size)(m.h, id) };
        c.that(size == declared, || format!("space {id} ({name}) is declared {declared} bytes and space_size says {size}"));
        if strs(space.get("flags")).iter().any(|x| x == "read_only") {
            let mut before = [0u8];
            unsafe { (f.space_read)(m.h, id, 0, before.as_mut_ptr(), 1) };
            let poke = [before[0] ^ 0xFF];
            let r = unsafe { (f.space_write)(m.h, id, 0, poke.as_ptr(), 1) };
            let mut after = [0u8];
            unsafe { (f.space_read)(m.h, id, 0, after.as_mut_ptr(), 1) };
            c.that(r < 0, || format!("space {id} ({name}) is read-only and a write to it answered {r}"));
            c.that(after == before, || format!("space {id} ({name}) is read-only and a write changed it"));
        }
    }
    for file in list("battery") {
        let which = file.get("which").and_then(Value::as_i64).unwrap_or(-1) as u32;
        let declared = file.get("length").and_then(Value::as_i64).unwrap_or(-1);
        let mut flags = 0u32;
        let length = unsafe { (f.battery)(m.h, which, std::ptr::null_mut(), 0, &mut flags) };
        c.that(length == declared, || format!("battery file {which} is declared {declared} bytes and battery says {length}"));
    }
    if s.lib.claims(caps::DEBUG) {
        for p in list("processors") {
            let id = p.get("id").and_then(Value::as_i64).unwrap_or(-1) as u32;
            let mut pc = 0u64;
            let r = f.debug_pc.map_or(status::NOT_SUPPORTED, |g| unsafe { g(m.h, id, &mut pc) });
            c.that(r == 0, || format!("processor {id} is named and debug_pc answers {r}"));
        }
    }
    c.note(format!("{} spaces, {} battery files, {} processors", list("spaces").len(), list("battery").len(), list("processors").len()));
    c.done("C14", "descriptors against exports")
}

use emusen_native::debug::{flag, run as runflag, stop};

/// Every debug log drained, so that a full one does not stop the frame.
fn drain_debug(m: &Machine<'_>) {
    let f = &m.lib.f;
    unsafe {
        if let Some(g) = f.debug_writes {
            let n = g(m.h, std::ptr::null_mut(), 0).max(0) as usize;
            let mut b = vec![0u32; 4 * n + 4];
            g(m.h, b.as_mut_ptr(), b.len());
        }
        if let Some(g) = f.debug_calls {
            let n = g(m.h, std::ptr::null_mut(), 0).max(0) as usize;
            let mut b = vec![0u32; 3 * n + 3];
            g(m.h, b.as_mut_ptr(), b.len());
        }
        if let Some(g) = f.debug_profile {
            let n = g(m.h, std::ptr::null_mut(), 0).max(0) as usize;
            let mut b = vec![0i64; 2 * n + 2];
            g(m.h, b.as_mut_ptr(), b.len());
        }
        if let Some(g) = f.debug_coverage {
            let mut recorded = 0i64;
            let n = g(m.h, 0, std::ptr::null_mut(), 0, &mut recorded).max(0) as usize;
            let mut b = vec![0u8; n];
            g(m.h, 0, b.as_mut_ptr(), b.len(), &mut recorded);
        }
    }
}

/// One frame through the observed loop; a stop other than a full log is a table hit, which an armed run must not have.
fn observed_frame(m: &Machine<'_>) -> Result<(), String> {
    let run = m.lib.f.debug_run_frame.ok_or("DEBUG is claimed and debug_run_frame is missing")?;
    let mut flags = 0;
    for _ in 0..1_000_000 {
        let (mut p, mut pc, mut detail) = (0u32, 0u64, 0u64);
        let r = unsafe { run(m.h, flags, &mut p, &mut pc, &mut detail) };
        drain_debug(m);
        if r < 0 {
            return Err(format!("debug_run_frame failed with {r}"));
        }
        if r == 0 {
            return Ok(());
        }
        if r as u32 & !stop::RING != 0 {
            return Err(format!("the armed frame stopped with reasons {r:#x} on processor {p} at {pc:#x}, though nothing was set to hit"));
        }
        flags = runflag::CONTINUE;
    }
    Err("the armed frame did not end".to_owned())
}

/// Every table armed with nothing to hit: calls, stores, the profile and processor 0's coverage tracked, a breakpoint
/// at an address no 32-bit counter reaches, and watch and data-breakpoint ranges over a space no core has.
fn arm(m: &Machine<'_>) {
    let f = &m.lib.f;
    unsafe {
        if let Some(g) = f.debug_set {
            g(m.h, flag::CALLS | flag::WRITES | flag::PROFILING | (1 << flag::COVERAGE), i32::MIN, -1);
        }
        if let Some(g) = f.debug_set_breakpoints {
            let pairs = [-2i32, -2];
            g(m.h, 0, pairs.as_ptr(), 1);
        }
        if let Some(g) = f.debug_set_ranges {
            let none = [u32::MAX - 1, 0, u32::MAX];
            g(m.h, 0, none.as_ptr(), 1);
            g(m.h, 1, none.as_ptr(), 1);
        }
    }
}

fn c15(s: &mut Session<'_>) -> Case {
    let mut c = Check::new();
    if !s.lib.claims(caps::DEBUG) {
        c.note("DEBUG is not claimed");
        return c.done("C15", "debug");
    }
    let frames = s.opts.frames.min(300);
    let settings = s.opts.settings.clone();
    let body = |s: &mut Session<'_>, c: &mut Check| -> Result<(), String> {
        let plain = solo(s, &settings, frames)?;
        let armed = s.machine(&settings)?;
        arm(&armed);
        s.observed = true;
        let t = s.play(&armed, 0, frames, false);
        s.observed = false;
        let got = (t?.digests, s.state_digest(&armed)?);
        c.that(got == plain, || format!("every table armed with nothing to hit gives {got:?}, the plain run {plain:?}"));

        let f = &s.lib.f;
        let ids: Vec<i64> = armed.machine_info().ok().and_then(|t| parse(&t)).and_then(|v| v.get("processors")?.as_array().map(|a| a.iter().filter_map(|p| p.get("id").and_then(Value::as_i64)).collect())).unwrap_or_default();
        let m = s.machine(&settings)?;
        s.play(&m, 0, 2, false)?;
        let (run, set, set_bp, pc_of) = (f.debug_run_frame.unwrap(), f.debug_set.unwrap(), f.debug_set_breakpoints.unwrap(), f.debug_pc.unwrap());
        unsafe {
            let mut first = 0u64;
            pc_of(m.h, 0, &mut first);
            let pair = [first as u32 as i32, first as u32 as i32];
            set_bp(m.h, 0, pair.as_ptr(), 1);
            let (mut p, mut pc, mut d) = (u32::MAX, 0u64, 0u64);
            let r = run(m.h, 0, &mut p, &mut pc, &mut d);
            c.that(r > 0 && r as u32 & stop::BREAKPOINT != 0 && pc == first, || format!("a breakpoint at the next instruction, {first:#x}, gave reasons {r:#x} at {pc:#x}"));
            c.that(ids.contains(&(p as i64)), || format!("the halt reported processor {p}, which machine info does not name"));
            set_bp(m.h, 0, std::ptr::null(), 0);

            set(m.h, flag::EACH, i32::MIN, -1);
            let at = |m: &Machine<'_>| {
                let mut pc = 0u64;
                pc_of(m.h, 0, &mut pc);
                ((m.lib.f.frame_count)(m.h), pc)
            };
            let before = at(&m);
            let r = run(m.h, runflag::CONTINUE, &mut p, &mut pc, &mut d);
            c.that(r > 0 && r as u32 & stop::EACH != 0 && at(&m) == before, || format!("with EACH armed, the call stopped with {r:#x} and moved the machine from {before:?} to {:?}", at(&m)));
            for i in 0..8 {
                let before = at(&m);
                let r = run(m.h, runflag::UNCHECKED | runflag::CONTINUE, &mut p, &mut pc, &mut d);
                c.that(r == 0 || (r > 0 && r as u32 & stop::EACH != 0), || format!("step {i} under EACH answered {r:#x}"));
                c.that(at(&m) != before, || format!("step {i} under EACH did not move the machine from {before:?}"));
            }
            set(m.h, 0, i32::MIN, -1);
        }
        c.note(format!("armed against plain over {frames} frames; a breakpoint at the next instruction; eight steps under EACH"));
        Ok(())
    };
    if let Err(e) = body(s, &mut c) {
        c.that(false, || e);
    }
    s.observed = false;
    c.done("C15", "debug")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The examples cargo built beside this test.
    fn library(name: &str) -> std::path::PathBuf {
        let profile = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
        let file = if cfg!(windows) { format!("{name}.dll") } else if cfg!(target_os = "macos") { format!("lib{name}.dylib") } else { format!("lib{name}.so") };
        profile.join("examples").join(file)
    }

    fn options(frames: u64) -> Options {
        let mut image = b"V1TC".to_vec();
        image.extend_from_slice(&5u16.to_le_bytes());
        image.extend_from_slice(&[1, 2, 3, 0x20, 0x41]);
        Options { self_delimiting: true, image, frames, settings: String::new(), files: Vec::new(), script: None }
    }

    // C8 with C6's run for its first machine says what C8 says with a machine of its own, to the digest on a core that passes and by its findings on one that fails; EmuSen_CoreAPI.md §28.3.
    #[test]
    fn c8_says_the_same_with_c6s_run_as_with_its_own() {
        for (name, frames) in [("kit_test_core", 120), ("kit_test_core", 121), ("kit_test_core", 2), ("kit_plain_core", 120), ("kit_faulty_core", 120)] {
            let lib = Lib::open(&library(name)).unwrap();
            let opts = options(frames);
            let mut s = session(&lib, &opts);
            let six = c6(&mut s);
            assert!(s.first.is_some(), "{name}: C6 left no run for C8");
            let reused = c8(&mut s);
            assert!(s.first.is_none());
            let own = c8(&mut s);
            // The faulty core's machines each differ from the last, so its state's digest is no two runs' alike; its verdict and findings are.
            match name {
                "kit_faulty_core" => assert_eq!((reused.passed, reused.evidence.len()), (own.passed, own.evidence.len()), "{name}: {:?} against {:?}", reused.evidence, own.evidence),
                _ => assert_eq!((reused.passed, &reused.evidence), (own.passed, &own.evidence), "{name} over {frames} frames"),
            }
            // C6's own words are those of a run with no state saved in the middle.
            let whole = s.machine("").and_then(|m| s.play(&m, 0, frames, false)).unwrap().digests;
            assert!(six.evidence.iter().any(|e| e.contains(&format!("frames digest {:016X}, audio digest {:016X}", whole.frames, whole.audio))), "{name}: {:?}", six.evidence);
        }
    }

    // A run of one frame has no middle for C8, so C6 leaves nothing and C8 runs its own two frames as before.
    #[test]
    fn c6_of_one_frame_leaves_c8_its_own_run() {
        let lib = Lib::open(&library("kit_test_core")).unwrap();
        let opts = options(1);
        let mut s = session(&lib, &opts);
        assert!(c6(&mut s).passed);
        assert!(s.first.is_none());
        assert!(c8(&mut s).passed);
    }
}
