//! What a test ROM said about itself, by its own suite's protocol, from the memories a run kept (VenusRT_Plan.md §3.1,
//! VenusRT_Native.md §3.3), and the fetched corpus's manifest.

use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    NoDump,
    Passed,
    Failed,
    Incomplete,
    Done,
    Visual,
}

impl Outcome {
    pub fn self_graded(self) -> bool {
        matches!(self, Outcome::Passed | Outcome::Failed | Outcome::Incomplete)
    }

    pub fn word(self) -> &'static str {
        match self {
            Outcome::NoDump => "NoDump",
            Outcome::Passed => "Passed",
            Outcome::Failed => "Failed",
            Outcome::Incomplete => "Incomplete",
            Outcome::Done => "Done",
            Outcome::Visual => "Visual",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub outcome: Outcome,
    pub protocol: String,
    pub detail: String,
}

fn verdict(outcome: Outcome, protocol: &str, detail: String) -> Verdict {
    Verdict { outcome, protocol: protocol.to_string(), detail }
}

/// A picture as 15-bit BGR words, row by row (VenusRT_Native.md §3.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u16>,
}

impl Picture {
    /// From RGBA bytes read in order, each channel's top five bits.
    pub fn from_rgba(rgba: &[u8], width: usize, height: usize) -> Picture {
        let mut pixels = vec![0u16; width * height];
        for (i, p) in pixels.iter_mut().enumerate() {
            if i * 4 + 2 >= rgba.len() {
                break;
            }
            *p = (rgba[i * 4] >> 3) as u16 | ((rgba[i * 4 + 1] >> 3) as u16) << 5 | ((rgba[i * 4 + 2] >> 3) as u16) << 10;
        }
        Picture { width, height, pixels }
    }
}

/// One engine's memories by the probe's lower-case names, and its picture, after a frame.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub frame: u32,
    pub spaces: BTreeMap<String, Vec<u8>>,
    pub picture: Option<Picture>,
}

/// A run's snapshots in frame order, and its sound from power-on as interleaved stereo.
#[derive(Clone, Debug, Default)]
pub struct SnesRun {
    pub engine: String,
    pub snapshots: Vec<Snapshot>,
    pub audio_rate: u32,
    pub audio: Vec<i16>,
}

impl SnesRun {
    pub fn at(&self, frame: u32) -> Option<&Snapshot> {
        self.snapshots.iter().find(|s| s.frame == frame)
    }
}

/// The memories the grader and the differential read.
pub const SPACES: [&str; 5] = ["vram", "cgram", "oam", "wram", "apuram"];

fn text_at(vram: &[u8], word: usize, n: usize) -> String {
    (0..n).map(|i| vram.get((word + i) * 2).copied().unwrap_or(0) as char).collect()
}

/// Non-overlapping occurrences, as Regex.Matches counts a literal.
fn count(hay: &[u8], needle: &[u8]) -> usize {
    let (mut n, mut i) = (0, 0);
    while i + needle.len() <= hay.len() {
        if &hay[i..i + needle.len()] == needle {
            n += 1;
            i += needle.len();
        } else {
            i += 1;
        }
    }
    n
}

fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `\b(Passed|PASSED|Failed|FAILED|FAIL|OK|Done|Success)\b` over ASCII text, leftmost, alternatives in order.
fn verdict_words(text: &str) -> Vec<&'static str> {
    const WORDS: [&str; 8] = ["Passed", "PASSED", "Failed", "FAILED", "FAIL", "OK", "Done", "Success"];
    let t = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < t.len() {
        let before = i == 0 || !is_word(t[i - 1]);
        let hit = if before { WORDS.iter().find(|w| t[i..].starts_with(w.as_bytes()) && (i + w.len() == t.len() || !is_word(t[i + w.len()]))) } else { None };
        match hit {
            Some(w) => {
                out.push(*w);
                i += w.len();
            }
            None => i += 1,
        }
    }
    out
}

/// Maximal runs of printable ASCII of four or more, each with more than two distinct characters.
fn printable_runs(low: &[u8]) -> Vec<String> {
    let mut runs = Vec::new();
    let mut i = 0;
    while i < low.len() {
        if (0x20..0x7F).contains(&low[i]) {
            let start = i;
            while i < low.len() && (0x20..0x7F).contains(&low[i]) {
                i += 1;
            }
            if i - start >= 4 {
                let run = &low[start..i];
                let mut seen = [false; 128];
                let distinct = run.iter().filter(|&&b| !std::mem::replace(&mut seen[b as usize], true)).count();
                if distinct > 2 {
                    runs.push(String::from_utf8_lossy(run).into_owned());
                }
            }
        } else {
            i += 1;
        }
    }
    runs
}

fn latin1(path: &Path) -> Option<String> {
    std::fs::read(path).ok().map(|b| b.iter().map(|&c| c as char).collect())
}

/// `(?i)#\$<hex>\b` as the backdrop rule reads a source.
fn has_immediate(text: &str, hex: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let needle = format!("#${hex}");
    let b = lower.as_bytes();
    let mut from = 0;
    while let Some(at) = lower[from..].find(&needle) {
        let end = from + at + needle.len();
        if end == b.len() || !is_word(b[end]) {
            return true;
        }
        from += at + 1;
    }
    false
}

/// A folder whose sources, or blobs' disassembly, write both verdict colours to colour 0.
fn writes_backdrop_verdicts(dir: &Path) -> bool {
    let mut asm = Vec::new();
    for d in [dir.to_path_buf(), dir.join("disassembly")] {
        if let Ok(entries) = std::fs::read_dir(&d) {
            asm.extend(entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "asm")));
        }
    }
    asm.iter().filter_map(|p| latin1(p)).any(|t| has_immediate(&t, "7c") && has_immediate(&t, "1f") && t.contains("2122"))
}

/// The ROM's own source when it counts its passes with PrintText(Pass...: beside it by name, else any .asm in its folder.
fn source_of(rom: &Path) -> Option<std::path::PathBuf> {
    let own = rom.with_extension("asm");
    let mut candidates = vec![own];
    if let Some(dir) = rom.parent() {
        let mut more: Vec<_> = std::fs::read_dir(dir).map(|e| e.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "asm")).collect()).unwrap_or_default();
        more.sort_by(|a, b| a.to_string_lossy().as_bytes().cmp(b.to_string_lossy().as_bytes()));
        candidates.extend(more);
    }
    let first = candidates.into_iter().find(|p| p.exists())?;
    latin1(&first).filter(|t| t.contains("PrintText(Pass")).map(|_| first)
}

/// The ROM's verdict at the run's last snapshot; the snapshot at half of it, when kept, tells a steady screen from a running one.
pub fn grade(rom_path: &Path, snapshots: &[Snapshot]) -> Verdict {
    let Some(last) = snapshots.last() else { return verdict(Outcome::NoDump, "", String::new()) };
    let Some(vram) = last.spaces.get("vram") else { return verdict(Outcome::NoDump, "", String::new()) };
    let path = rom_path.to_string_lossy();
    let name = rom_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let low: Vec<u8> = vram.iter().step_by(2).copied().collect();

    if path.contains("gilyon") || matches!(name.as_str(), "cputest-full.sfc" | "cputest-basic.sfc" | "spctest.sfc") {
        let (text, number) = (text_at(vram, 0x32, 7), text_at(vram, 0x6E, 4));
        if text.starts_with("Success") {
            return verdict(Outcome::Passed, "gilyon", format!("last={number}"));
        }
        if text.starts_with("Failed") {
            return verdict(Outcome::Failed, "gilyon", format!("test={number}"));
        }
        return verdict(Outcome::Incomplete, "gilyon", format!("test={number}"));
    }

    if path.contains("blargg-spc-6") {
        let cg = last.spaces.get("cgram").cloned().unwrap_or_else(|| vec![0; 2]);
        let colour = cg[0] as u32 | (cg[1] as u32) << 8;
        let (r, g, b) = (colour & 31, colour >> 5 & 31, colour >> 10 & 31);
        let o = if b > r && b > g { Outcome::Passed } else if r > b && r > g { Outcome::Failed } else { Outcome::Incomplete };
        return verdict(o, "blargg-backdrop", format!("rgb={r},{g},{b}"));
    }

    if source_of(rom_path).is_some() {
        let (passes, fails) = (count(&low, b"PASS"), count(&low, b"FAIL"));
        let half = snapshots.iter().find(|s| s.frame == last.frame / 2);
        let steady = half.and_then(|h| h.spaces.get("vram")).is_some_and(|before| before == vram);
        let o = if fails > 0 { Outcome::Failed } else if passes > 0 && steady { Outcome::Passed } else { Outcome::Incomplete };
        return verdict(o, "peterlemon", format!("shown={passes} fail={fails} steady={}", u8::from(steady)));
    }

    let runs = printable_runs(&low);
    let words = verdict_words(&runs.join(" "));
    let mut detail = runs.iter().map(|s| s.trim()).collect::<Vec<_>>().join(" | ");
    if detail.chars().count() > 400 {
        detail = detail.chars().take(400).collect();
    }
    let Some(word) = words.last() else {
        return backdrop(rom_path, last).unwrap_or_else(|| verdict(Outcome::Visual, "text", detail));
    };
    let outcome = match *word {
        "Done" => Outcome::Done,
        "Passed" | "PASSED" | "OK" | "Success" => Outcome::Passed,
        _ => Outcome::Failed,
    };
    verdict(outcome, "text", detail)
}

/// byuu's protocol in Jonas Quinn's collection: colour 0 blue for a pass and red for a failure.
fn backdrop(rom_path: &Path, last: &Snapshot) -> Option<Verdict> {
    if !rom_path.to_string_lossy().contains("jonasquinn-test-roms") || !rom_path.parent().is_some_and(writes_backdrop_verdicts) {
        return None;
    }
    let cg = last.spaces.get("cgram").cloned().unwrap_or_else(|| vec![0; 2]);
    match (cg[0] as u32 | (cg[1] as u32) << 8) & 0x7FFF {
        0x7C00 => Some(verdict(Outcome::Passed, "byuu-backdrop", "colour0=7C00".into())),
        0x001F => Some(verdict(Outcome::Failed, "byuu-backdrop", "colour0=001F".into())),
        _ => None,
    }
}

/// The fetched corpus, never committed: its folder from EMUSEN_VENUSRT_CORPUS (VenusRT_Plan.md §3.1).
pub const CORPUS_VARIABLE: &str = "EMUSEN_VENUSRT_CORPUS";

pub fn corpus_root() -> Option<std::path::PathBuf> {
    std::env::var_os(CORPUS_VARIABLE).map(std::path::PathBuf::from).filter(|p| p.is_dir())
}

/// The corpus's own manifest: one ROM per MD5, as unique-roms.txt names them, by relative path.
pub fn unique(root: &Path) -> Vec<String> {
    std::fs::read_to_string(root.join("unique-roms.txt"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.splitn(3, ' ').nth(2).map(str::to_string))
        .collect()
}

/// The corpus's own manifest with each ROM's MD5.
pub fn unique_with_md5(root: &Path) -> Vec<(String, String)> {
    std::fs::read_to_string(root.join("unique-roms.txt"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let p: Vec<&str> = l.splitn(3, ' ').collect();
            (p.len() == 3).then(|| (p[0].to_string(), p[2].to_string()))
        })
        .collect()
}

/// The row of VenusRT_Plan.md §3.5's table a ROM belongs to.
pub fn suite(path: &str) -> &'static str {
    let n = path.rsplit('/').next().unwrap_or(path);
    let lower = path.to_ascii_lowercase();
    if path.contains("gilyon") && n.starts_with("cputest") {
        return "gilyon cputest";
    }
    if n == "spctest.sfc" {
        return "gilyon spctest";
    }
    if path.contains("CPUTest/CPU/") || path.contains("SNES-CPUTest-CPU/") {
        return "PeterLemon CPU";
    }
    if path.contains("CPUTest/SPC700/") || path.contains("SNES-CPUTest-SPC700/") {
        return "PeterLemon SPC700";
    }
    if path.contains("GSUTest") {
        return "PeterLemon GSU";
    }
    if path.contains("blargg-spc-6") {
        return if n.starts_with("spc_dsp6") { "blargg spc_dsp6" } else { "blargg SPC" };
    }
    if path.contains("blargg_2010") {
        return "blargg 2010";
    }
    if lower.contains("adc") || lower.contains("sbc") {
        return "ADC/SBC";
    }
    if lower.contains("mul") || lower.contains("div") {
        return "multiply/divide";
    }
    if path.contains("absindx") {
        return "absindx SA-1";
    }
    if lower.contains("cx4") {
        return "Cx4";
    }
    if path.contains("undisbeliever") {
        return "undisbeliever";
    }
    if path.contains("240pSuite") {
        return "240p suite";
    }
    if path.contains("PeterLemon") {
        return "PeterLemon other";
    }
    "higan collection, other"
}

/// Frames each ROM is run to: 1800 and 3600, and 18,000 for blargg's DSP test.
pub fn frames(path: &str) -> &'static [u32] {
    if path.ends_with("spc_dsp6.sfc") { &[9000, 18000] } else { &[1800, 3600] }
}
