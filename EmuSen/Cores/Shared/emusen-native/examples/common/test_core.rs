// The test cores' machine, included by each example with its own capability set; see EmuSen_CoreAPI.md §18.1 and §19.

use emusen_native::core::*;
use emusen_native::ffi::{self, StateMachine};

/// A state of another core, or a short one.
#[derive(Debug, PartialEq, Eq)]
pub enum StateError {
    Foreign,
    Truncated,
    TooSmall,
}

impl ffi::Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Foreign => status::FOREIGN,
            StateError::Truncated => status::TRUNCATED,
            StateError::TooSmall => status::BUFFER_TOO_SMALL,
        }
    }
}

/// Seeded faults for the conformance kit's own tests: each machine differs from the last, and a truncated state loads.
pub const FAULT_MACHINES_DIFFER: u32 = 1;
pub const FAULT_TAKES_TRUNCATED_STATE: u32 = 2;
static MADE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// The core's own refusal of an image: an empty one.
pub const STATUS_EMPTY_IMAGE: i32 = -9;
/// An image is this magic, a little-endian u16 payload length, and the payload.
pub const IMAGE_MAGIC: &[u8; 4] = b"V1TC";

/// A well-formed image around `payload`.
pub fn image(payload: &[u8]) -> Vec<u8> {
    let mut v = IMAGE_MAGIC.to_vec();
    v.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    v.extend_from_slice(payload);
    v
}

fn payload(image: &[u8]) -> Result<&[u8], i32> {
    if image.is_empty() {
        return Err(STATUS_EMPTY_IMAGE);
    }
    if image.len() < 6 || &image[..4] != IMAGE_MAGIC {
        detail("not a test core image");
        return Err(status::BAD_IMAGE);
    }
    let n = u16::from_le_bytes([image[4], image[5]]) as usize;
    image.get(6..6 + n).ok_or_else(|| {
        detail("the image is truncated");
        status::BAD_IMAGE
    })
}
const MAGIC: &[u8; 4] = b"TST1";
const SNAP: &[u8; 4] = b"TSN1";

pub struct TestCore {
    frames: i64,
    ram: [u8; 256],
    sram: [u8; 16],
    sram_dirty: bool,
    pads: [u32; 2],
    axis: f64,
    wide: bool,
    rate: i32,
    pending_rate: i32,
    /// Samples at the front of the queue made at `old_rate`, before the last change.
    old: usize,
    old_rate: i32,
    samples: Vec<i16>,
    limit: usize,
    picture: Vec<u8>,
    skip: bool,
    mutes: u32,
    patches: Vec<u32>,
    pokes: Vec<u32>,
    hooks: Hooks,
}

impl TestCore {
    fn width(&self) -> i32 {
        if self.wide { 16 } else { 8 }
    }

    fn draw(&mut self) {
        let n = (self.width() * 4) as usize;
        self.picture = (0..n).flat_map(|i| [(self.frames as u8).wrapping_add(i as u8), 0, 0, 255]).collect();
    }
}

impl StateMachine for TestCore {
    type Error = StateError;

    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        if data.len() < 4 || (&data[..4] != MAGIC && &data[..4] != SNAP) {
            return Err(StateError::Foreign);
        }
        if data.len() < self.state_size() {
            if TEST_FAULTS & FAULT_TAKES_TRUNCATED_STATE == 0 || data.len() < 12 {
                return Err(StateError::Truncated);
            }
            self.frames = i64::from_le_bytes(data[4..12].try_into().unwrap());
            return Ok(());
        }
        self.frames = i64::from_le_bytes(data[4..12].try_into().unwrap());
        self.ram.copy_from_slice(&data[12..268]);
        self.draw();
        Ok(())
    }

    fn state_size(&self) -> usize {
        4 + 8 + 256
    }

    fn save_state(&self, out: &mut [u8]) -> Result<usize, StateError> {
        let n = self.state_size();
        let out = out.get_mut(..n).ok_or(StateError::TooSmall)?;
        out[..4].copy_from_slice(MAGIC);
        out[4..12].copy_from_slice(&self.frames.to_le_bytes());
        out[12..].copy_from_slice(&self.ram);
        Ok(n)
    }

    fn layout(&self) -> String {
        "0 4 magic Magic\n4 8 long Frames\n12 256 byte[] Ram\n".to_owned()
    }
}

fn setting(key: &str, kind: SettingKind, default: &str, scope: Scope, effect: Effect) -> Setting {
    Setting { key: key.into(), label: key.into(), help: format!("The test core's {key}."), kind, default: default.into(), scope, category: None, effect, advanced: false, hidden: false, restart: false }
}

impl Core for TestCore {
    const CAPABILITIES: u64 = TEST_CAPABILITIES;

    fn info() -> Info {
        Info {
            id: TEST_ID.into(),
            name: TEST_NAME.into(),
            display_name: Some(format!("{TEST_NAME} (test)")),
            version: "1.0.0".into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("A counter that exercises every export of the core ABI.".into()),
            systems: vec![System {
                id: "test".into(),
                name: "Test system".into(),
                extensions: vec![".tst".into()],
                regions: vec![Region::Ntsc],
                controllers: vec![Controller {
                    id: "test.pad".into(),
                    label: "Controller".into(),
                    ports: vec![0, 1],
                    buttons: vec![Button { bit: 0, control: Some(Control::A), label: "A".into() }, Button { bit: 1, control: None, label: "Turbo".into() }],
                    axes: vec![AxisControl { axis: Axis::LeftX as u32, control: Some(Axis::LeftX), trigger: false, label: "Stick".into() }],
                }],
                firmware: vec![Firmware { which: 16, name: "boot.rom".into(), label: "Boot ROM".into(), size: 64, required: false, parts: vec![] }],
            }],
            deterministic: true,
            accuracy: Some(Accuracy { measured_with: "defaults".into(), suite: "the crate's tests".into(), notes: String::new() }),
            ..Info::default()
        }
    }

    fn settings_schema() -> Vec<Setting> {
        vec![
            setting("Wide", SettingKind::Switch, "false", Scope::Run, Effect::Enhancement { cost: "Twice the pixels.".into(), hardware: "false".into() }),
            setting("Rate", SettingKind::Choice(vec![Choice { value: "32000".into(), label: "32 kHz".into(), help: None }, Choice { value: "48000".into(), label: "48 kHz".into(), help: None }]), "32000", Scope::Run, Effect::None),
            setting("Ram", SettingKind::Count { min: 1, max: 255, step: 1 }, "7", Scope::Create, Effect::None),
            setting("Threads", SettingKind::Count { min: 1, max: 4, step: 1 }, "1", Scope::Run, Effect::Exact),
        ]
    }

    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        if payload(image).ok().and_then(|p| p.first()) == Some(&0xB0) { Self::info().systems[0].firmware.clone() } else { Vec::new() }
    }

    fn status_text(code: i32) -> Option<String> {
        (code == STATUS_EMPTY_IMAGE).then(|| "the image is empty".to_owned())
    }

    fn create(r: &Create<'_>) -> Result<Self, i32> {
        let body = payload(r.image)?;
        let mut ram = [0u8; 256];
        ram[0] = r.settings.get("Ram").and_then(|v| v.parse().ok()).unwrap_or(0);
        ram[1..].iter_mut().zip(body).for_each(|(d, s)| *d = *s);
        if TEST_FAULTS & FAULT_MACHINES_DIFFER != 0 {
            ram[255] = MADE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        let mut sram = [0u8; 16];
        for f in &r.files {
            match f.which {
                0 => sram.iter_mut().zip(f.data).for_each(|(d, s)| *d = *s),
                16 => {}
                _ => {
                    detail(&format!("file {} is not this core's", f.which));
                    return Err(status::BAD_FILE);
                }
            }
        }
        log(Level::Info, "test", "created");
        let mut m = TestCore {
            frames: 0,
            ram,
            sram,
            sram_dirty: false,
            pads: [0; 2],
            axis: 0.0,
            wide: r.settings.get("Wide") == Some("true"),
            rate: 32000,
            pending_rate: 32000,
            old: 0,
            old_rate: 32000,
            samples: Vec::new(),
            limit: 1 << 16,
            picture: Vec::new(),
            skip: false,
            mutes: 0,
            patches: Vec::new(),
            pokes: Vec::new(),
            hooks: Hooks::new(&[8]),
        };
        m.draw();
        Ok(m)
    }

    fn machine_info(&self) -> MachineInfo {
        MachineInfo {
            system: "test".into(),
            region: Some(Region::Ntsc),
            frame_rate: (60, 1),
            video: Video { base_width: 8, base_height: 4, max_width: 16, max_height: 4, aspect: (4, 3), formats: vec![pixel::RGBA8888] },
            audio: Audio { rate: self.rate, channels: vec!["Tone".into()] },
            ports: vec![Port { port: 0, controller: Some("test.pad".into()) }, Port { port: 1, controller: None }],
            spaces: vec![Space { cheats: true, ..Space::new(0, "RAM") }, Space { read_only: true, ..Space::new(1, "ROM") }],
            processors: vec![Processor { id: 0, name: "Counter".into(), pc_bits: 8, registers: vec![("F".into(), 64)] }],
            battery: vec![Battery { which: 0, suffix: ".srm".into() }],
            state: StateFormat { format: "TST1".into(), version: 1, loads_from: vec![1] },
            phases: vec!["count".into()],
            patches: Some((0, 0xFF)),
            skip_rendering_state_neutral: true,
        }
    }

    fn reset(&mut self) -> Result<(), i32> {
        self.frames = 0;
        self.draw();
        Ok(())
    }

    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        self.frames += 1;
        self.ram[2] = self.ram[2].wrapping_add(1);
        if self.pads[0] & 1 != 0 {
            self.ram[3] = self.ram[3].wrapping_add(1);
            self.sram[0] = self.sram[0].wrapping_add(1);
            self.sram_dirty = true;
            emit(event::BATTERY, 0, 0);
        }
        for q in self.pokes.chunks_exact(4) {
            if q[0] == 0 && (q[3] == flags::NO_COMPARE || self.ram[q[1] as usize & 0xFF] as u32 == q[3]) {
                self.ram[q[1] as usize & 0xFF] = q[2] as u8;
            }
        }
        if self.pending_rate != self.rate {
            (self.old, self.old_rate) = (self.samples.len(), self.rate);
            self.rate = self.pending_rate;
        }
        let tone = if self.mutes & 1 != 0 { 0 } else { 1000 };
        for _ in 0..(self.rate / 60) {
            if self.samples.len() + 2 > self.limit {
                self.samples.drain(..2);
                self.old = self.old.saturating_sub(2);
            }
            self.samples.extend_from_slice(&[tone, -tone]);
        }
        if !self.skip {
            self.draw();
        }
        Ok(())
    }

    fn present(&mut self) -> Result<(), i32> {
        Ok(())
    }

    fn set_options(&mut self, flags: u32) {
        self.skip = flags & 1 != 0;
    }

    fn frame_count(&self) -> i64 {
        self.frames
    }

    fn phases(&self) -> Vec<i64> {
        vec![1000]
    }

    fn frame_info(&self) -> FrameInfo {
        FrameInfo { serial: self.frames, aspect_num: 4, aspect_den: 3, ..FrameInfo::rgba(self.width(), 4) }
    }

    fn frame(&self) -> &[u8] {
        &self.picture
    }

    fn audio_rate(&self) -> i32 {
        if self.old > 0 { self.old_rate } else { self.rate }
    }

    fn audio_buffered(&self) -> usize {
        self.samples.len()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        let available = if self.old > 0 { self.old } else { self.samples.len() };
        let n = (out.len() / 2).min(max_frames).min(available / 2) * 2;
        out[..n].copy_from_slice(&self.samples[..n]);
        self.samples.drain(..n);
        self.old = self.old.saturating_sub(n);
        n
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.limit = samples.max(2);
    }

    fn peek_audio(&self, out: &mut [i16]) -> usize {
        let n = out.len().min(self.samples.len());
        out[..n].copy_from_slice(&self.samples[..n]);
        self.samples.len()
    }

    fn set_mutes(&mut self, mask: u32) -> Result<(), i32> {
        self.mutes = mask;
        Ok(())
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        let pad = self.pads.get_mut(port as usize).ok_or(status::NO_SUCH_PORT)?;
        *pad = (*pad & !changed) | (mask & changed);
        Ok(())
    }

    fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), i32> {
        if port != 0 || axis != Axis::LeftX as u32 {
            return Err(status::NOT_SUPPORTED);
        }
        self.axis = value.clamp(-1.0, 1.0);
        Ok(())
    }

    fn snapshot_size(&self) -> usize {
        self.state_size()
    }

    fn save_snapshot(&mut self, out: &mut [u8]) -> Result<usize, i32> {
        let n = self.save_state(out).map_err(|e| ffi::Status::status(&e))?;
        out[..4].copy_from_slice(SNAP);
        Ok(n)
    }

    fn snapshot_layout(&self) -> String {
        self.layout().replace("magic Magic", "magic Snapshot")
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        match space {
            0 => Ok(256),
            1 => Ok(0),
            _ => Err(status::NO_SUCH_SPACE),
        }
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        self.space_size(space)?;
        for (i, b) in out.iter_mut().enumerate() {
            *b = *self.ram.get(address as usize + i).unwrap_or(&0);
        }
        Ok(())
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        match space {
            0 => {
                for (i, &b) in data.iter().enumerate() {
                    if let Some(d) = self.ram.get_mut(address as usize + i) {
                        *d = b;
                    }
                }
                Ok(())
            }
            1 => Err(status::READ_ONLY),
            _ => Err(status::NO_SUCH_SPACE),
        }
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        if which != 0 {
            return Err(status::BAD_FILE);
        }
        Ok((&self.sram, flags::BATTERY_TRACKED | self.sram_dirty as u32))
    }

    fn battery_saved(&mut self, _which: u32) -> Result<(), i32> {
        self.sram_dirty = false;
        Ok(())
    }

    fn set_rom_patches(&mut self, triples: &[u32]) -> Result<(), i32> {
        self.patches = triples.to_vec();
        Ok(())
    }

    fn set_cheat_pokes(&mut self, quads: &[u32]) -> Result<(), i32> {
        self.pokes = quads.to_vec();
        Ok(())
    }

    fn set_settings(&mut self, s: &Settings) -> Result<(), i32> {
        if let Some(v) = s.get("Wide") {
            self.wide = v == "true";
            self.draw();
        }
        if let Some(v) = s.get("Rate") {
            self.pending_rate = v.parse().unwrap_or(32000);
        }
        Ok(())
    }

    fn setting_notes(&self) -> Vec<(String, String)> {
        if self.wide { vec![("Wide".into(), "The picture is drawn twice as wide.".into())] } else { Vec::new() }
    }

    fn debug_hooks(&mut self) -> Option<&mut Hooks> {
        Some(&mut self.hooks)
    }

    fn debug_run_frame(&mut self, _flags: u32, detail: &mut u64) -> Result<u32, i32> {
        let pc = self.frames as u32 & 0xFF;
        self.hooks.record(pc);
        let why = self.hooks.stop_before(pc);
        if why != 0 {
            return Ok(why);
        }
        self.advance(detail).map(|()| 0)
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        (processor == 0).then_some(self.frames as u64 & 0xFF)
    }

    fn debug_registers(&self, processor: u32) -> Result<Vec<i64>, i32> {
        if processor == 0 { Ok(vec![self.frames]) } else { Err(status::NOT_SUPPORTED) }
    }

    fn debug_disassemble(&mut self, processor: u32, space: u32, address: u32, count: u32) -> Result<Vec<Instruction>, i32> {
        if processor != 0 || space != 0 {
            return Err(status::NOT_SUPPORTED);
        }
        Ok((0..count)
            .map(|i| {
                let a = address.wrapping_add(i);
                let b = *self.ram.get(a as usize).unwrap_or(&0);
                Instruction { address: a, bytes: vec![b], mnemonic: "DB".into(), operands: format!("${b:02X}"), reference: (b == 0x20).then_some((Reference::Call, b as u32)) }
            })
            .collect())
    }
}

