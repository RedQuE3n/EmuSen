//! A core driven as a front end's frame loop drives it through the runtime: its settings' defaults at create, the
//! input latch sent before each frame, the frame advanced and presented, the sound drained, the picture copied packed
//! with its rows repeated, and the state saved whole; no firmware, no battery (EmuSen_Backend.md §40.10).

use std::path::Path;

use emusen_core_host::{Create, InputLatch, Library, Machine};
use emusen_native::core::sys::caps;
use serde_json::Value;

/// The pad's controls in their canonical order, emusen-native's: bit n of a script's mask is control n.
pub const PAD_BUTTONS: [&str; 16] = emusen_native::core::Control::NAMES;

/// The audio queue's bound the runtime sets, `SampleQueue.DefaultLimit`.
pub const AUDIO_LIMIT: u64 = 128_000;

pub struct Driven {
    pub machine: Machine,
    latch: InputLatch,
    /// Per port, the core's bit for each PadButton, none where its pad has no such control.
    bits: Vec<[Option<u32>; 16]>,
    skip: bool,
    frame: Vec<u8>,
    audio: Vec<i16>,
    present: bool,
}

fn text<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

impl Driven {
    /// The library's machine for the image, as the system its info gives the extension.
    pub fn open(library: &Path, image: &[u8], extension: &str, skip_rendering: bool) -> Result<Driven, String> {
        let lib = Library::load(library).map_err(|e| format!("{}: {e:?}", library.display()))?;
        let info = serde_json::from_str::<Value>(&lib.info_text().map_err(|c| lib.words(c))?).map_err(|e| format!("info: {e:?}"))?;
        let settings = serde_json::from_str::<Value>(&lib.settings_text().map_err(|c| lib.words(c))?).map_err(|e| format!("settings: {e:?}"))?;
        let rows: Vec<(String, String, bool)> = settings
            .as_array()
            .into_iter()
            .flatten()
            .map(|s| (text(s, "key").unwrap_or("").to_string(), text(s, "default").unwrap_or("").to_string(), text(s, "scope") == Some("create")))
            .collect();
        let all: Vec<String> = rows.iter().map(|(k, v, _)| format!("{k}={v}")).collect();
        let run: Vec<String> = rows.iter().filter(|r| !r.2).map(|(k, v, _)| format!("{k}={v}")).collect();
        let system = info
            .get("systems")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|s| s.get("extensions").and_then(Value::as_array).into_iter().flatten().any(|e| e.as_str().is_some_and(|e| e.trim_start_matches('.').eq_ignore_ascii_case(extension.trim_start_matches('.')))))
            .cloned();
        let id = system.as_ref().and_then(|s| text(s, "id")).map(|s| std::ffi::CString::new(s).unwrap());
        let all = all.join("\n");
        let mut create = Create::new(image);
        create.settings = &all;
        create.system = id.as_deref();
        let mut machine = Machine::create(lib, &create).map_err(|r| r.words)?;
        machine.set_options(u32::from(skip_rendering)).map_err(|c| lib.words(c as i64))?;
        let run = run.join("\n");
        if !run.is_empty() && lib.claims(caps::SETTINGS) {
            machine.set_settings(&run).map_err(|c| lib.words(c as i64))?;
        }
        let pads: Vec<&Value> = system.as_ref().and_then(|s| s.get("controllers")).and_then(Value::as_array).map(|c| c.iter().collect()).unwrap_or_default();
        let ports = pads.iter().flat_map(|c| c.get("ports").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_i64)).max().map_or(0, |p| p as usize + 1).max(2);
        // Each port's pad as the machine's info names it, as `CoreEngine.ControllerFor` chose it; the system's first where the info names no port.
        let machine_info = serde_json::from_str::<Value>(&machine.machine_info().unwrap_or_default()).unwrap_or(Value::Null);
        let named: Vec<(i64, Option<String>)> = machine_info
            .get("ports")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|p| (p.get("port").and_then(Value::as_i64).unwrap_or(-1), text(p, "controller").map(str::to_string)))
            .collect();
        let ports = ports.max(named.len());
        let mut bits = vec![[None; 16]; ports];
        for (port, row) in bits.iter_mut().enumerate() {
            let pad = if named.is_empty() {
                pads.first()
            } else {
                let id = named.iter().find(|(p, _)| *p == port as i64).and_then(|(_, c)| c.clone());
                pads.iter().find(|c| id.is_some() && text(c, "id") == id.as_deref())
            };
            for b in pad.and_then(|c| c.get("buttons")).and_then(Value::as_array).into_iter().flatten() {
                if let (Some(control), Some(bit)) = (text(b, "control"), b.get("bit").and_then(Value::as_i64)) {
                    if let Some(i) = PAD_BUTTONS.iter().position(|n| *n == control) {
                        if bit < 32 {
                            row[i] = Some(bit as u32);
                        }
                    }
                }
            }
        }
        machine.events().map_err(|c| lib.words(c as i64))?;
        machine.set_audio_limit(AUDIO_LIMIT).map_err(|c| lib.words(c as i64))?;
        let present = lib.claims(caps::PRESENT);
        Ok(Driven { machine, latch: InputLatch::new(ports, 0), bits, skip: skip_rendering, frame: Vec::new(), audio: vec![0; 1 << 16], present })
    }

    /// A port's held PadButtons, as a mask of PadButton bits.
    pub fn hold(&mut self, port: usize, pad_mask: u32) {
        if port >= self.bits.len() {
            return;
        }
        let (mut mask, mut changed) = (0u32, 0u32);
        for (i, bit) in self.bits[port].iter().enumerate() {
            if let Some(bit) = bit {
                changed |= 1 << bit;
                if pad_mask & (1 << i) != 0 {
                    mask |= 1 << bit;
                }
            }
        }
        self.latch.hold(port, mask, changed);
    }

    /// One frame; the error is the core's words.
    pub fn frame(&mut self) -> Result<(), String> {
        let lib = self.machine.library();
        self.latch.send(&mut self.machine);
        self.machine.advance().map_err(|(c, _)| lib.words(c as i64))?;
        if self.present && !self.skip {
            self.machine.present().map_err(|c| lib.words(c as i64))?;
        }
        if !self.skip {
            let _ = self.machine.frame_device();
        }
        self.machine.events().map_err(|c| lib.words(c as i64))?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), String> {
        self.machine.reset().map_err(|c| self.machine.library().words(c as i64))
    }

    /// Every sample queued since the last drain, interleaved stereo.
    pub fn drain_audio(&mut self) -> Vec<i16> {
        let mut out = Vec::new();
        loop {
            match self.machine.audio_drain(&mut self.audio) {
                Ok((0, _)) | Err(_) => break,
                Ok((samples, _)) => out.extend_from_slice(&self.audio[..samples]),
            }
        }
        out
    }

    /// The picture packed, each row repeated as the core asks: width x 4 x height x repeat bytes.
    pub fn picture(&mut self) -> (Vec<u8>, i32, i32) {
        let Ok(info) = self.machine.frame_info() else { return (Vec::new(), 0, 0) };
        let bytes = info.bytes.max(0) as usize;
        if self.frame.len() < bytes {
            self.frame.resize(bytes, 0);
        }
        let _ = self.machine.frame_copy(&mut self.frame);
        let (w, h) = (info.width.max(0) as usize, info.height.max(0) as usize);
        let stride = if info.stride == 0 { w * 4 } else { info.stride as usize };
        let repeat = info.row_repeat.max(1) as usize;
        let mut out = Vec::with_capacity(w * 4 * h * repeat);
        for y in 0..h {
            for _ in 0..repeat {
                out.extend_from_slice(&self.frame[y * stride..y * stride + w * 4]);
            }
        }
        (out, info.width, info.height * repeat as i32)
    }

    /// The whole state, kind 0.
    pub fn state(&mut self) -> Vec<u8> {
        let size = self.machine.state_size(0).unwrap_or(0);
        let mut s = vec![0u8; size];
        let n = self.machine.state_save(0, &mut s).unwrap_or(0);
        s.truncate(n);
        s
    }

    pub fn load_state(&mut self, state: &[u8]) -> Result<(), String> {
        self.machine.state_load(state).map_err(|c| self.machine.library().words(c as i64))
    }
}
