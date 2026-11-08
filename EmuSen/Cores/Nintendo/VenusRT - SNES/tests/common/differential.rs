//! Frame-by-frame differential of two SNES engines' runs, VenusRT's bare machine on the core ABI and Mesen run as a
//! black box through the reference probe (VenusRT_Native.md §3, §3.4 for what each comparison can and cannot say).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use emusen_core_host::{Create, Library, Machine};
use serde_json::Value;

use super::golden::library;
use super::grader::{self, Picture, SPACES, Snapshot, SnesRun};

/// A 224-line engine's row r is Mesen's row r + 7, measured on steady pictures (VenusRT_Native.md §3.2).
pub const MESEN_ROW_OFFSET: usize = 7;

/// One frame's comparison of two engines: bytes differing per space, pixels differing over the shared picture.
#[derive(Clone, Debug)]
pub struct FrameDiff {
    pub frame: u32,
    pub space_bytes: BTreeMap<String, usize>,
    pub pixels_compared: usize,
    pub pixels_differing: usize,
}

impl FrameDiff {
    pub fn identical(&self) -> bool {
        self.space_bytes.values().all(|&n| n == 0) && self.pixels_differing == 0
    }
}

/// The sound's agreement: the loudness envelope's best correlation within a lag, and whether each side is silent.
#[derive(Clone, Debug)]
pub struct AudioDiff {
    pub correlation: f64,
    pub lag_windows: i32,
    pub first_silent: bool,
    pub second_silent: bool,
}

/// The waveforms' agreement at one rate, second by second.
#[derive(Clone, Debug)]
pub struct SampleDiff {
    pub correlation: f64,
    pub least: f64,
    pub lag_from: i32,
    pub lag_to: i32,
    pub gain: f64,
    pub relative_error: f64,
    pub seconds: usize,
}

pub fn compare(a: &SnesRun, b: &SnesRun, second_row_offset: usize) -> Vec<FrameDiff> {
    let mut diffs = Vec::new();
    for x in &a.snapshots {
        let Some(y) = b.at(x.frame) else { continue };
        let mut bytes = BTreeMap::new();
        for (name, left) in &x.spaces {
            let Some(right) = y.spaces.get(name) else { continue };
            let n = left.len().abs_diff(right.len()) + left.iter().zip(right).filter(|(l, r)| l != r).count();
            bytes.insert(name.clone(), n);
        }
        let (compared, differing) = pictures(x.picture.as_ref(), y.picture.as_ref(), second_row_offset);
        diffs.push(FrameDiff { frame: x.frame, space_bytes: bytes, pixels_compared: compared, pixels_differing: differing });
    }
    diffs
}

/// The rectangle both pictures cover, the second shifted down by its offset; the top bit of a colour word is not compared.
pub fn pictures(p: Option<&Picture>, q: Option<&Picture>, row_offset: usize) -> (usize, usize) {
    let (Some(p), Some(q)) = (p, q) else { return (0, 0) };
    let (mut py, mut qy, mut offset) = (if p.height > 300 { 2 } else { 1 }, if q.height > 300 { 2 } else { 1 }, row_offset as i64);
    if py == 2 && qy == 2 {
        (py, qy, offset) = (1, 1, offset * 2);
    }
    let w = p.width.max(q.width) as i64;
    let h = (p.height as i64 / py).min(q.height as i64 / qy - offset);
    let mut differing = 0;
    for y in 0..h.max(0) {
        for x in 0..w {
            let px = if p.width as i64 == w { x } else { x * p.width as i64 / w };
            let qx = if q.width as i64 == w { x } else { x * q.width as i64 / w };
            let a = p.pixels[(y * py * p.width as i64 + px) as usize];
            let b = q.pixels[((y + offset) * qy * q.width as i64 + qx) as usize];
            if (a ^ b) & 0x7FFF != 0 {
                differing += 1;
            }
        }
    }
    ((w * h).max(0) as usize, differing)
}

fn pearson(x: &[f64], y: &[f64], lag: i64) -> f64 {
    let from = (-lag).max(0);
    let to = (x.len() as i64).min(y.len() as i64 - lag);
    let n = to - from;
    if n < 2 {
        return f64::NAN;
    }
    let (mut mx, mut my) = (0.0, 0.0);
    for i in from..to {
        mx += x[i as usize];
        my += y[(i + lag) as usize];
    }
    mx /= n as f64;
    my /= n as f64;
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for i in from..to {
        let (dx, dy) = (x[i as usize] - mx, y[(i + lag) as usize] - my);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx == 0.0 || syy == 0.0 { f64::NAN } else { sxy / (sxx * syy).sqrt() }
}

/// A run's loudness in windows of a sixtieth of a second.
pub fn envelope(run: &SnesRun) -> Vec<f64> {
    if run.audio_rate == 0 {
        return Vec::new();
    }
    let window = (run.audio_rate as usize / 60).max(1);
    let frames = run.audio.len() / 2;
    (0..frames / window)
        .map(|w| {
            let sum: f64 = (w * window..(w + 1) * window).map(|i| (run.audio[i * 2] as f64 + run.audio[i * 2 + 1] as f64) / 2.0).map(|m| m * m).sum();
            (sum / window as f64).sqrt()
        })
        .collect()
}

/// Each run's loudness correlated at the best lag within `max_lag` windows.
pub fn audio(a: &SnesRun, b: &SnesRun, max_lag: i64) -> AudioDiff {
    let (ea, eb) = (envelope(a), envelope(b));
    let (sa, sb) = (ea.iter().all(|&v| v < 1.0), eb.iter().all(|&v| v < 1.0));
    let (mut best, mut best_lag) = (f64::NAN, 0);
    for lag in -max_lag..=max_lag {
        let r = pearson(&ea, &eb, lag);
        if best.is_nan() || r > best {
            best = r;
            best_lag = lag;
        }
    }
    AudioDiff { correlation: best, lag_windows: best_lag as i32, first_silent: sa, second_silent: sb }
}

/// A run's sound as mono at `rate`, by linear interpolation between its own samples.
pub fn resample(run: &SnesRun, rate: u32) -> Vec<f64> {
    if run.audio_rate == 0 {
        return Vec::new();
    }
    let frames = run.audio.len() / 2;
    let mono: Vec<f64> = (0..frames).map(|i| (run.audio[i * 2] as f64 + run.audio[i * 2 + 1] as f64) / 2.0).collect();
    if run.audio_rate == rate {
        return mono;
    }
    let length = (frames as u64 * rate as u64 / run.audio_rate as u64) as usize;
    (0..length)
        .map(|j| {
            let t = j as f64 * run.audio_rate as f64 / rate as f64;
            let k = t as usize;
            let f = t - k as f64;
            if k + 1 < frames { mono[k] * (1.0 - f) + mono[k + 1] * f } else { mono[k.min(frames - 1)] }
        })
        .collect()
}

/// Both runs as mono at `rate` from frame `skip_frames` on, each second compared at its own best lag within `max_lag` samples (VenusRT_Native.md §23).
pub fn samples(a: &SnesRun, b: &SnesRun, rate: u32, max_lag: i64, skip_frames: u32) -> SampleDiff {
    let (x, y) = (resample(a, rate), resample(b, rate));
    let (mut rs, mut lags, mut gains, mut errors) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let rate = rate as i64;
    let mut from = (skip_frames as i64 * rate / 60).max(max_lag);
    while from + rate + max_lag <= x.len().min(y.len()) as i64 {
        let (mut best, mut best_lag) = (f64::NAN, 0i64);
        for lag in -max_lag..=max_lag {
            let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
            for i in from..from + rate {
                let (u, v) = (x[i as usize], y[(i + lag) as usize]);
                sxy += u * v;
                sxx += u * u;
                syy += v * v;
            }
            let c = if sxx == 0.0 || syy == 0.0 { f64::NAN } else { sxy / (sxx * syy).sqrt() };
            if !c.is_nan() && (best.is_nan() || c.abs() > best.abs()) {
                best = c;
                best_lag = lag;
            }
        }
        if !best.is_nan() {
            let (mut xy, mut yy, mut xx) = (0.0, 0.0, 0.0);
            for i in from..from + rate {
                let v = y[(i + best_lag) as usize];
                xy += x[i as usize] * v;
                yy += v * v;
                xx += x[i as usize] * x[i as usize];
            }
            let gain = xy / yy;
            let err: f64 = (from..from + rate).map(|i| x[i as usize] - gain * y[(i + best_lag) as usize]).map(|e| e * e).sum();
            rs.push(best);
            lags.push(best_lag);
            gains.push(gain);
            errors.push((err / xx).sqrt());
        }
        from += rate;
    }
    let median = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(f64::total_cmp);
        s[s.len() / 2]
    };
    if rs.is_empty() {
        return SampleDiff { correlation: f64::NAN, least: f64::NAN, lag_from: 0, lag_to: 0, gain: f64::NAN, relative_error: f64::NAN, seconds: 0 };
    }
    SampleDiff {
        correlation: median(&rs),
        least: rs.iter().copied().fold(f64::INFINITY, f64::min),
        lag_from: *lags.iter().min().unwrap() as i32,
        lag_to: *lags.iter().max().unwrap() as i32,
        gain: median(&gains),
        relative_error: median(&errors),
        seconds: rs.len(),
    }
}

/// A button held on port 0 from `frame` for `frames` frames, as the probe's --press counts them; button is a PadButton index.
#[derive(Clone, Copy)]
pub struct Press {
    pub frame: u32,
    pub button: usize,
    pub frames: u32,
}

/// The probe's button names, PadButton's own spelling.
pub const BUTTONS: [&str; 12] = ["B", "Y", "Select", "Start", "Up", "Down", "Left", "Right", "A", "X", "L", "R"];

fn held(presses: &[Press], button: usize, frame: u32) -> bool {
    presses.iter().any(|p| p.button == button && frame >= p.frame && frame < p.frame + p.frames)
}

/// The NEC DSP firmware VenusRT's own firmware_for names for the image as file 2, by name and size.
pub fn dsp_request(image: &[u8]) -> Option<(String, u64)> {
    let lib = Library::load(&library()).ok()?;
    let text = lib.firmware_for(image).ok()?;
    let v = serde_json::from_str::<Value>(&text).ok()?;
    let list = v.as_array().cloned().or_else(|| v.get("firmware").and_then(Value::as_array).cloned()).unwrap_or_default();
    list.iter().find(|f| f.get("which").and_then(Value::as_i64) == Some(2)).and_then(|f| Some((f.get("name")?.as_str()?.to_string(), f.get("size")?.as_i64()? as u64)))
}

/// That firmware's bytes, whole or as its program and data pair, from the first folder holding it.
pub fn dsp_firmware(image: &[u8], folders: &[PathBuf]) -> Option<Vec<u8>> {
    let (name, size) = dsp_request(image)?;
    let stem = Path::new(&name).file_stem()?.to_string_lossy().into_owned();
    for dir in folders {
        let whole = dir.join(&name);
        if whole.metadata().is_ok_and(|m| m.len() == size) {
            return std::fs::read(whole).ok();
        }
        let (program, data) = (dir.join(format!("{stem}.program.rom")), dir.join(format!("{stem}.data.rom")));
        if program.exists() && data.exists() {
            let mut b = std::fs::read(program).ok()?;
            b.extend(std::fs::read(data).ok()?);
            return Some(b);
        }
    }
    None
}

/// The folders a firmware is looked for in: the corpus's own firmware folder.
pub fn firmware_folders() -> Vec<PathBuf> {
    grader::corpus_root().map(|r| vec![r.join("firmware")]).unwrap_or_default()
}

/// VenusRT's bare machine on the core ABI v1, with file 1 the boot image and file 2 the DSP's firmware where given.
pub fn venusrt_machine(image: &[u8], dsp: Option<&[u8]>, boot: Option<&[u8]>) -> Result<Machine, emusen_core_host::Refusal> {
    let lib = Library::load(&library()).expect("VenusRT's library");
    let mut files: Vec<(u32, &[u8])> = Vec::new();
    if let Some(b) = boot {
        files.push((1, b));
    }
    if let Some(d) = dsp {
        files.push((2, d));
    }
    let mut c = Create::new(image);
    c.files = &files;
    Machine::create(lib, &c)
}

fn spaces_by_name(m: &Machine) -> Vec<(String, u32)> {
    let v = serde_json::from_str::<Value>(&m.machine_info().unwrap_or_default()).unwrap_or(Value::Null);
    v.get("spaces").and_then(Value::as_array).into_iter().flatten().filter_map(|s| Some((s.get("name")?.as_str()?.to_string(), s.get("id")?.as_i64()? as u32))).collect()
}

/// A space read whole, by its name in any case; empty for one the machine does not list.
pub fn read_space(m: &mut Machine, name: &str) -> Vec<u8> {
    let Some((_, id)) = spaces_by_name(m).into_iter().find(|(n, _)| n.eq_ignore_ascii_case(name)) else { return Vec::new() };
    let mut bytes = vec![0u8; m.space_size(id).unwrap_or(0) as usize];
    if !bytes.is_empty() {
        let _ = m.space_read(id, 0, &mut bytes);
    }
    bytes
}

/// VenusRT run from power-on to each frame asked for, as the probe runs Mesen: the DSP image found and the core's own boot program unless given one.
pub fn run_venusrt(rom: &Path, frames: &[u32], presses: &[Press], with_audio: bool, dsp: bool, boot: Option<&[u8]>) -> SnesRun {
    let image = std::fs::read(rom).expect("the ROM reads");
    let firmware = if dsp { dsp_firmware(&image, &firmware_folders()) } else { None };
    let mut m = venusrt_machine(&image, firmware.as_deref(), boot).unwrap_or_else(|r| panic!("VenusRT refused {}: {}", rom.display(), r.words));
    let mut wanted = frames.to_vec();
    wanted.sort();
    let (mut shots, mut sound, mut scratch) = (Vec::new(), Vec::new(), vec![0i16; 1 << 16]);
    for target in wanted {
        // The machine's own count: a DMA longer than a frame is one step, and one advance then ends two frames on.
        while (m.frame_count() as u32) < target {
            let frame = m.frame_count() as u32;
            let mask = (0..12).filter(|&b| held(presses, b, frame)).fold(0u32, |m, b| m | 1 << b);
            m.set_buttons(0, mask, 0xFFF).unwrap();
            m.advance().unwrap();
            while let Ok((n, _)) = m.audio_drain(&mut scratch) {
                if n == 0 {
                    break;
                }
                if with_audio {
                    sound.extend_from_slice(&scratch[..n]);
                }
            }
        }
        let spaces = SPACES.iter().map(|n| (n.to_string(), read_space(&mut m, n))).collect();
        let info = m.frame_info().unwrap();
        let mut rgba = vec![0u8; info.bytes.max(0) as usize];
        let _ = m.frame_copy(&mut rgba);
        shots.push(Snapshot { frame: target, spaces, picture: Some(Picture::from_rgba(&rgba, info.width as usize, info.height as usize)) });
    }
    SnesRun { engine: "VenusRT".into(), snapshots: shots, audio_rate: m.audio_rate() as u32, audio: sound }
}

/// The runner's choices from the environment: EMUSEN_VENUSRT_NO_FIRMWARE=1 runs without the DSP image, EMUSEN_VENUSRT_BOOT names a boot image.
pub fn run_default(rom: &Path, frames: &[u32], presses: &[Press], with_audio: bool) -> SnesRun {
    let dsp = std::env::var("EMUSEN_VENUSRT_NO_FIRMWARE").as_deref() != Ok("1");
    let boot = std::env::var_os("EMUSEN_VENUSRT_BOOT").filter(|b| !b.is_empty()).map(|b| std::fs::read(b).expect("the boot image reads"));
    run_venusrt(rom, frames, presses, with_audio, dsp, boot.as_deref())
}

/// Mesen's probe: EMUSEN_MESEN_PROBE, run in EMUSEN_MESEN_CHECKOUT, a black box whose dumps are read back.
pub fn mesen_probe() -> Option<(PathBuf, PathBuf)> {
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let probe = std::env::var_os("EMUSEN_MESEN_PROBE").map(PathBuf::from).unwrap_or_else(|| home.join(".cache/emusen/probe/mesen/probe"));
    let checkout = std::env::var_os("EMUSEN_MESEN_CHECKOUT").map(PathBuf::from).unwrap_or_else(|| home.join("Projects/mesen-reference"));
    (probe.exists() && checkout.join("bin/pgohelperlib.so").exists()).then_some((probe, checkout))
}

/// A 16-bit PCM WAV's rate and samples; a mono file is widened to stereo.
pub fn read_wav(path: &Path) -> (u32, Vec<i16>) {
    let Ok(b) = std::fs::read(path) else { return (0, Vec::new()) };
    let (mut rate, mut channels, mut at) = (0u32, 2u16, 12usize);
    while at + 8 <= b.len() {
        let id = &b[at..at + 4];
        let size = i32::from_le_bytes(b[at + 4..at + 8].try_into().unwrap()) as usize;
        if id == b"fmt " {
            channels = u16::from_le_bytes([b[at + 10], b[at + 11]]);
            rate = u32::from_le_bytes(b[at + 12..at + 16].try_into().unwrap());
        }
        if id == b"data" {
            let n = size.min(b.len() - at - 8) / 2;
            let s: Vec<i16> = (0..n).map(|i| i16::from_le_bytes([b[at + 8 + 2 * i], b[at + 9 + 2 * i]])).collect();
            return (rate, if channels == 1 { s.iter().flat_map(|&x| [x, x]).collect() } else { s });
        }
        at += 8 + size + (size & 1);
    }
    (0, Vec::new())
}

/// Mesen run as a black box through the probe; each dump set cached by ROM, frames, presses, probe build and firmware.
pub fn run_mesen(cache: &Path, rom: &Path, frames: &[u32], presses: &[Press], with_audio: bool) -> SnesRun {
    let (probe, checkout) = mesen_probe().expect("the Mesen probe");
    let mut sorted = frames.to_vec();
    sorted.sort();
    let (start, end) = (sorted[0], *sorted.last().unwrap());
    let stride = if sorted.len() > 1 { sorted[1] - sorted[0] } else { 1 };
    assert!(sorted.windows(2).all(|w| w[1] - w[0] == stride), "the probe reports at a fixed stride");
    let mut args = vec![rom.to_string_lossy().into_owned(), String::new(), start.to_string(), end.to_string(), stride.to_string()];
    for p in presses {
        args.push("--press".into());
        args.push(format!("{}:{}:{}", p.frame, BUTTONS[p.button], p.frames));
    }
    let image = std::fs::read(rom).expect("the ROM reads");
    let firmware = dsp_firmware(&image, &firmware_folders());
    let built = probe.metadata().ok().and_then(|m| m.modified().ok()).map(|t| format!("{t:?}")).unwrap_or_default();
    let key_text = format!(
        "{}\n{with_audio}\n{}:{built}\n{}{}",
        args[2..].join("\n"),
        probe.metadata().map_or(0, |m| m.len()),
        emusen_core_host::sha256::sha256_hex(&image),
        firmware.as_ref().map_or(String::new(), |f| format!("\nfirmware {}", emusen_core_host::sha256::sha256_hex(f)))
    );
    let dir = cache.join(&emusen_core_host::sha256::sha256_hex(key_text.as_bytes())[..20]);
    let log = dir.join("probe.log");
    if !dir.join("done").exists() {
        std::fs::create_dir_all(&dir).unwrap();
        args[1] = dir.to_string_lossy().into_owned();
        // Mesen reads a DSP's firmware as home/Firmware/<name>.rom, program and data in one file (measured 2026-10-02).
        if let (Some(f), Some((name, _))) = (&firmware, dsp_request(&image)) {
            std::fs::create_dir_all(dir.join("mesenhome/Firmware")).unwrap();
            std::fs::write(dir.join("mesenhome/Firmware").join(name), f).unwrap();
        }
        if with_audio {
            args.push("--wav".into());
            args.push(dir.join("audio.wav").to_string_lossy().into_owned());
        }
        let out = Command::new(&probe).args(&args).current_dir(&checkout).output().expect("the probe starts");
        let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&out.stderr));
        text.push_str(&format!("\nexit={}\n", out.status.code().unwrap_or(-1)));
        std::fs::write(&log, &text).unwrap();
        let _ = std::fs::remove_dir_all(dir.join("mesenhome"));
        assert!(out.status.success(), "the probe exited {:?} on {}: {}", out.status.code(), rom.display(), String::from_utf8_lossy(&out.stderr));
        std::fs::write(dir.join("done"), "").unwrap();
    }
    // The probe prints the PPU's output size once per report, in report order.
    let log_text = std::fs::read_to_string(&log).unwrap_or_default();
    let sizes: Vec<(usize, usize)> = log_text
        .split("PPU ")
        .skip(1)
        .filter_map(|s| {
            let dims = s.split(':').next()?;
            let (w, h) = dims.split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        })
        .collect();
    let mut shots = Vec::new();
    for (i, &f) in sorted.iter().enumerate() {
        let mut spaces = BTreeMap::new();
        for name in SPACES {
            if let Ok(b) = std::fs::read(dir.join(format!("mesen_{name}_f{f:05}.bin"))) {
                spaces.insert(name.to_string(), b);
            }
        }
        let mut picture = None;
        if let (Ok(raw), Some(&(mut w, mut h))) = (std::fs::read(dir.join(format!("mesen_screen_f{f:05}.bin"))), sizes.get(i)) {
            // A hi-res or interlaced frame fills the probe's whole 512x478 buffer, every line twice when not interlaced; its log still says 256x239.
            let wide = (w * h * 2..raw.len().saturating_sub(1)).step_by(2).any(|j| (raw[j] as u16 | (raw[j + 1] as u16) << 8) & 0x7FFF != 0);
            if wide {
                (w, h) = (512, 478);
            }
            let pixels = (0..w * h).map(|j| if j * 2 + 1 < raw.len() { raw[j * 2] as u16 | (raw[j * 2 + 1] as u16) << 8 } else { 0 }).collect();
            picture = Some(Picture { width: w, height: h, pixels });
        }
        shots.push(Snapshot { frame: f, spaces, picture });
    }
    let (rate, sound) = if with_audio { read_wav(&dir.join("audio.wav")) } else { (0, Vec::new()) };
    SnesRun { engine: "Mesen".into(), snapshots: shots, audio_rate: rate, audio: sound }
}
