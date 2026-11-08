//! Recording and grading VenusRT's golden runs through the core ABI, as the runtime holds the engine, with no firmware so
//! the open replacements run (VenusRT_Native.md §68.2).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use emusen_goldens::trace::{fold, hash16, hex_upper, unhex8};
use emusen_goldens::{Driven, Trace};
use serde_json::Value;

use super::grader;

pub const RECORD_VARIABLE: &str = "EMUSEN_VENUSRT_GOLDEN_RECORD";
pub const REASON_VARIABLE: &str = "EMUSEN_VENUSRT_GOLDEN_REASON";
pub const STATES_VARIABLE: &str = "EMUSEN_VENUSRT_GOLDEN_STATES";
pub const INTERVAL_FRAMES: u32 = 300;
pub const CHAIN_ROW: u32 = 60;
pub const HEADING: &str = "A golden trace of VenusRT, recorded from VenusRT itself; re-recorded only deliberately, with the reason below - see VenusRT_Native.md §68.2.";

/// One golden run: an image, an input script, resets and frames; a corpus run ends at its suite's verdict.
#[derive(Clone)]
pub struct Run {
    pub name: String,
    pub rom: Vec<u8>,
    pub input: &'static str,
    pub skip_rendering: bool,
    pub frames: u32,
    pub corpus: Option<PathBuf>,
    pub resets: Vec<u32>,
}

impl Run {
    pub fn frames(name: &str, rom: Vec<u8>, input: &'static str, frames: u32) -> Run {
        Run { name: name.to_string(), rom, input, skip_rendering: false, frames, corpus: None, resets: Vec::new() }
    }
}

pub fn directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("goldens")
}

pub fn trace_path(name: &str) -> PathBuf {
    directory().join(format!("{name}.trace"))
}

pub fn recording() -> bool {
    std::env::var(RECORD_VARIABLE).as_deref() == Ok("1")
}

pub fn reason() -> String {
    std::env::var(REASON_VARIABLE).ok().filter(|r| !r.is_empty()).unwrap_or_else(|| panic!("{RECORD_VARIABLE}=1 needs {REASON_VARIABLE} to say why the goldens are re-recorded."))
}

/// The cdylib cargo built with the tests, in their deps folder.
pub fn library() -> PathBuf {
    let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let file = if cfg!(windows) { "venusrt.dll" } else if cfg!(target_os = "macos") { "libvenusrt.dylib" } else { "libvenusrt.so" };
    deps.join(file)
}

/// Both pads' masks, PadButton bits, on a frame numbered from 0.
pub fn script_mask(input: &str, frame: u32) -> (u32, u32) {
    const START: u32 = 1 << 3;
    const A: u32 = 1 << 8;
    match input {
        "none" => (0, 0),
        "bench" => (match frame % 90 { 0..5 => START, 45..50 => A, _ => 0 }, 0),
        "pads" => (((frame / 7 * 7) * 37) & 0xFFF, ((frame / 7 * 7) * 91) & 0xFFF),
        _ => panic!("no input script named {input}"),
    }
}

/// The script's fingerprint over the frames a run takes.
pub fn script_name(input: &str, frames: u32) -> String {
    if input == "none" {
        return input.to_string();
    }
    let mut masks = Vec::with_capacity(frames as usize * 4);
    for f in 0..frames {
        let (one, two) = script_mask(input, f);
        masks.extend_from_slice(&(one as u16).to_le_bytes());
        masks.extend_from_slice(&(two as u16).to_le_bytes());
    }
    format!("{input} {}", hash16(&masks))
}

/// A grader's detail on one line of printable ASCII, other bytes as \xNN.
pub fn printable(text: &str) -> String {
    let mut s = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => s.push_str("\\\\"),
            ' '..='~' => s.push(ch),
            _ => s.push_str(&format!("\\x{:02X}", ch as u32 & 0xFF)),
        }
    }
    s
}

fn spaces_of(d: &mut Driven) -> Vec<(String, u32)> {
    let info = d.machine.machine_info().unwrap_or_default();
    let v = serde_json::from_str::<Value>(&info).unwrap_or(Value::Null);
    v.get("spaces")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|s| Some((s.get("name")?.as_str()?.to_string(), s.get("id")?.as_i64()? as u32)))
        .collect()
}

/// The memories the grader reads, by the probe's lower-case names.
fn snapshot(d: &mut Driven, frame: u32) -> grader::Snapshot {
    let spaces = spaces_of(d);
    let mut kept = BTreeMap::new();
    for name in ["vram", "cgram", "oam", "wram", "apuram"] {
        let Some((_, id)) = spaces.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)) else { continue };
        let size = d.machine.space_size(*id).unwrap_or(0) as usize;
        let mut bytes = vec![0u8; size];
        let _ = d.machine.space_read(*id, 0, &mut bytes);
        kept.insert(name.to_string(), bytes);
    }
    grader::Snapshot { frame, spaces: kept, picture: None }
}

/// The run as a trace, from power-on or from a state at a frame; `interval` sees the state every INTERVAL_FRAMES.
pub fn run(r: &Run, resume: Option<(&[u8], u32)>, mut interval: impl FnMut(u32, &[u8])) -> Trace {
    let mut trace = Trace::new(if r.corpus.is_some() { "chain" } else { "frame" });
    trace.heading = HEADING.to_string();
    trace.name = r.name.clone();
    trace.rom = emusen_core_host::sha256::sha256_hex(&r.rom).to_ascii_uppercase();
    trace.input = script_name(r.input, r.frames);
    trace.rendering = if r.skip_rendering { "skipped" } else { "on" }.to_string();
    trace.recorded_by = "VenusRT".to_string();
    let mut d = Driven::open(&library(), &r.rom, "sfc", r.skip_rendering).unwrap_or_else(|e| panic!("{}: VenusRT refused it: {e}", r.name));
    let start = match resume {
        Some((state, at)) => {
            d.load_state(state).expect("the recorded state loads");
            at
        }
        None => 0,
    };
    let graded: &[u32] = r.corpus.as_ref().map_or(&[], |p| grader::frames(&p.to_string_lossy()));
    let mut shots = Vec::new();
    let mut chain = [0u8; 8];
    let mut held = (u32::MAX, u32::MAX);
    let mut done = start;
    for f in start..r.frames {
        if r.resets.contains(&f) {
            d.reset().expect("a reset");
        }
        let mask = script_mask(r.input, f);
        if mask != held {
            d.hold(0, mask.0);
            d.hold(1, mask.1);
            held = mask;
        }
        if let Err(words) = d.frame() {
            trace.transcript.insert("verdict".into(), format!("Error {} {words}", f + 1));
            break;
        }
        done = f + 1;
        let samples = d.drain_audio();
        let state = d.state();
        let (picture, _, _) = d.picture();
        let sound: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        let (s, p, a) = (hash16(&state), hash16(&picture), hash16(&sound));
        if done % INTERVAL_FRAMES == 0 {
            interval(done, &state);
        }
        if r.corpus.is_none() {
            trace.rows.push(vec![done.to_string(), s, p, a]);
            continue;
        }
        let mut link = [0u8; 24];
        link[..8].copy_from_slice(&unhex8(&s));
        link[8..16].copy_from_slice(&unhex8(&p));
        link[16..].copy_from_slice(&unhex8(&a));
        let h = emusen_core_host::sha256::sha256_hex(&link);
        fold(&mut chain, &unhex8(&h[..16]));
        if done % CHAIN_ROW == 0 {
            trace.rows.push(vec![done.to_string(), hex_upper(&chain)]);
        }
        if graded.contains(&done) {
            shots.push(snapshot(&mut d, done));
        }
    }
    let Some(path) = &r.corpus else { return trace };
    if done > 0 && trace.rows.last().is_none_or(|row| row[0] != done.to_string()) {
        trace.rows.push(vec![done.to_string(), hex_upper(&chain)]);
    }
    if !trace.transcript.contains_key("verdict") {
        let v = grader::grade(path, &shots);
        trace.transcript.insert("verdict".into(), format!("{} {done}", v.outcome.word()));
        if !v.protocol.is_empty() {
            trace.transcript.insert("protocol".into(), v.protocol);
        }
        if !v.detail.is_empty() {
            trace.transcript.insert("detail".into(), printable(&v.detail));
        }
    }
    trace
}

/// Where the full states every INTERVAL_FRAMES go, outside the tree.
pub fn state_cache() -> PathBuf {
    std::env::var_os(STATES_VARIABLE)
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache/emusen/venusrt-goldens"))
}

/// The interval state saved during recording at or after a frame, if this machine has it.
pub fn cached_state(name: &str, at_or_after: u32) -> Option<(u32, Vec<u8>)> {
    let dir = state_cache().join(name);
    let mut found: Vec<(u32, PathBuf)> = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            (p.extension()? == "state").then_some(())?;
            Some((p.file_stem()?.to_str()?.parse().ok()?, p))
        })
        .filter(|(f, _)| *f >= at_or_after)
        .collect();
    found.sort();
    let (f, p) = found.into_iter().next()?;
    Some((f, std::fs::read(p).ok()?))
}

pub fn save_state(name: &str, frame: u32, state: &[u8]) {
    let dir = state_cache().join(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{frame}.state")), state).unwrap();
}

/// Records when asked, writing only a trace that changed; otherwise grades from power-on, then from a recorded interval state where this machine keeps one.
pub fn check(r: &Run) -> String {
    let path = trace_path(&r.name);
    if recording() {
        let why = reason();
        let name = r.name.clone();
        let corpus = r.corpus.is_some();
        let mut t = run(r, None, |f, s| if !corpus { save_state(&name, f, s) });
        if let Ok(old) = std::fs::read_to_string(&path) {
            if Trace::parse(&old).first_difference(&t).is_none() {
                return format!("{}: unchanged", r.name);
            }
        }
        t.reason = why;
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, t.text()).unwrap();
        return format!("{}: recorded, {} rows", r.name, t.rows.len());
    }
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} has no golden; record one with {RECORD_VARIABLE}=1 and a reason in {REASON_VARIABLE}.", r.name));
    let golden = Trace::parse(&text);
    let now = run(r, None, |_, _| {});
    if let Some(d) = golden.first_difference(&now) {
        panic!("{}: {d}{}", r.name, bisect(r, &d));
    }
    if r.corpus.is_some() {
        return format!("{}: {} rows as recorded", r.name, now.rows.len());
    }
    let resume_at = INTERVAL_FRAMES * (r.frames / 4 / INTERVAL_FRAMES).max(1);
    let Some((frame, state)) = (resume_at < r.frames).then(|| cached_state(&r.name, resume_at)).flatten().filter(|c| c.0 == resume_at) else {
        return format!("{}: {} frames as recorded", r.name, now.rows.len());
    };
    let resumed = run(r, Some((&state, frame)), |_, _| {});
    let mut tail = golden.clone();
    tail.rows.drain(..resume_at as usize);
    if let Some(after) = tail.first_difference(&resumed) {
        panic!("{}, resumed from the recorded state of frame {resume_at}: {after}", r.name);
    }
    format!("{}: {} frames as recorded, and again from the recorded state of frame {resume_at}", r.name, now.rows.len())
}

/// The first recorded interval state at or after a failing frame, and the field this run's state first differs from it in.
fn bisect(r: &Run, difference: &str) -> String {
    let Some(frame) = difference.strip_prefix("frame ").and_then(|t| t.split(':').next()).and_then(|n| n.parse::<u32>().ok()) else { return String::new() };
    if r.corpus.is_some() {
        return String::new();
    }
    let Some((at, cached)) = cached_state(&r.name, frame) else { return String::new() };
    let mut mine = None;
    let mut short = r.clone();
    short.frames = at;
    run(&short, None, |f, s| if f == at { mine = Some(s.to_vec()) });
    let Some(mine) = mine.filter(|m| m.len() == cached.len()) else { return format!("; at frame {at} the states differ in length") };
    let Some(first) = mine.iter().zip(&cached).position(|(a, b)| a != b) else {
        return format!("; at frame {at} the state equals the recorded one byte for byte, so the trace moved and not the engine");
    };
    let d = Driven::open(&library(), &r.rom, "sfc", false).unwrap();
    let layout = d.machine.layout(0).unwrap_or_default();
    let field = layout
        .lines()
        .filter_map(|l| {
            let p: Vec<&str> = l.split(' ').collect();
            let at: usize = p.first()?.parse().ok()?;
            (at <= first).then(|| p.last().unwrap().to_string())
        })
        .last()
        .unwrap_or_else(|| "?".into());
    format!("; at frame {at} the state first differs from the recorded one at byte {first}, in {field}")
}
