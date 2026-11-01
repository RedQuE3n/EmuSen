//! The libretro core as a v1 machine: one per process, its life a load and an unload, every export of
//! EmuSen_LibretroHost.md §4 mapped onto libretro's calls.

use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use emusen_libretro_sys as sys;
use emusen_native::core::{
    self as v1, Audio, Battery, BatteryKind, Button, Choice, Control, Controller, Create, Drive, Effect, Firmware, Insert, FrameInfo, Info, MachineInfo, NamedFile, Port, Scope, Setting,
    SettingKind, Space, StateFormat, System, Video, caps, event, status,
};
use emusen_native::ffi::{Status, StateMachine};

use crate::files::{self, Saves};
use crate::front::{self, Shared};
use crate::library::{self, Library};
use crate::memory;
use crate::state;
use crate::video::{self, Picture};

/// A second machine while one lives: libretro's globals allow one per process.
pub const STATUS_BUSY: i32 = -9;
/// -10 and -11 are the broker's, for a runner that ended or stopped answering (EmuSen_LibretroHost.md §3.4).
/// The core asked to stop (`SHUTDOWN`).
pub const STATUS_SHUTDOWN: i32 = -12;
/// The core refused the game; its log says why.
pub const STATUS_REFUSED: i32 = -13;
/// The libretro core could not be found, hashed or loaded.
pub const STATUS_NO_CORE: i32 = -14;
/// A state saved on another platform by a core whose states depend on it.
pub const STATUS_PLATFORM: i32 = -15;
/// The core refused a state; the machine is as it was before the load.
pub const STATUS_STATE_REFUSED: i32 = -16;
/// The private folders could not be made or written.
pub const STATUS_FOLDERS: i32 = -17;

static LIVE: AtomicBool = AtomicBool::new(false);
/// Held while the library initialises, so that a library-level call on another thread waits for its declarations.
static INIT: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The words for each of the libretro host's own statuses.
pub fn status_words(code: i32) -> Option<&'static str> {
    Some(match code {
        STATUS_BUSY => "the libretro core already runs a machine in this process",
        STATUS_SHUTDOWN => "the core asked to stop",
        STATUS_REFUSED => "the core refused the game",
        STATUS_NO_CORE => "the libretro core could not be loaded",
        STATUS_PLATFORM => "a state saved on another platform, which this core's states depend on",
        STATUS_STATE_REFUSED => "the core refused the state; the machine is as it was",
        STATUS_FOLDERS => "the core's folders could not be made or written",
        _ => return None,
    })
}

/// A state's failure, as its status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fail(pub i32);

impl Status for Fail {
    fn status(&self) -> i32 {
        self.0
    }
}

/// The battery files every machine may name: save RAM and the real-time clock.
const SRAM: u32 = 0;
const RTC: u32 = 1;

pub struct Machine {
    lib: &'static Library,
    shared: Box<Shared>,
    frames: i64,
    system: String,
    identity: u64,
    spaces: Vec<memory::Space>,
    saves: Saves,
    sram_saved: Vec<u8>,
    rtc_saved: Vec<u8>,
    /// The bytes `retro_game_info.data` points into, kept until unload.
    _image: Option<Vec<u8>>,
    _strings: Vec<CString>,
    phases: [i64; 3],
    /// The spaces are `retro_get_memory_data`'s blocks, asked again after a load or reset.
    blocks: bool,
    /// The firmware files written for this machine, removed when it is freed (EmuSen_LibretroHost.md §4.8).
    _firmware: Written,
    /// The discs given to the core's disk control, kept until it is freed: their paths and bytes.
    discs: Vec<(Option<CString>, Option<Vec<u8>>)>,
    /// The disc in the drive, as its state header records it.
    disc: [u8; 16],
}

/// Files removed when dropped: the firmware copies of a machine, or of a create that failed.
struct Written(Vec<std::path::PathBuf>);

impl Drop for Written {
    fn drop(&mut self) {
        for f in &self.0 {
            let _ = std::fs::remove_file(f);
        }
    }
}

/// The library, initialised: callbacks set, `retro_init` called, the folders made.
pub fn ready() -> Result<&'static Library, String> {
    let lib = library::get().map_err(str::to_owned)?;
    let _once = INIT.lock().unwrap_or_else(|e| e.into_inner());
    if front::with_declared(|d| std::mem::replace(&mut d.initialised, true)) {
        return Ok(lib);
    }
    let folders = files::Folders::for_entry(&lib.folder, &lib.stem).map_err(|e| {
        front::with_declared(|d| d.initialised = false);
        format!("the core's folders: {e}")
    })?;
    let c = |p: &std::path::Path| CString::new(p.to_string_lossy().as_bytes()).unwrap_or_default();
    front::set_library_folders(c(&folders.system), c(&folders.save));
    unsafe {
        (lib.api.set_environment)(Some(front::environment));
        (lib.api.set_video_refresh)(Some(front::video_refresh));
        (lib.api.set_audio_sample)(Some(front::audio_sample));
        (lib.api.set_audio_sample_batch)(Some(front::audio_sample_batch));
        (lib.api.set_input_poll)(Some(front::input_poll));
        (lib.api.set_input_state)(Some(front::input_state));
        (lib.api.init)();
    }
    front::flush_logs();
    Ok(lib)
}

fn info_value(lib: &Library, key: &str) -> String {
    lib.info_file.get(key).cloned().unwrap_or_default()
}

/// The systems the core runs, by the systems table (EmuSen_LibretroHost.md §4.1).
fn systems(lib: &Library) -> Vec<(String, String, Vec<String>)> {
    let databases: Vec<String> = info_value(lib, "database").split('|').map(str::to_owned).collect();
    let refs: Vec<&str> = databases.iter().map(String::as_str).collect();
    let found = emusen_systems::for_libretro(&info_value(lib, "systemid"), &refs);
    if found.is_empty() {
        let id = format!("libretro-{}", lib.stem.to_ascii_lowercase().replace(|c: char| !c.is_ascii_alphanumeric(), "-"));
        let name = Some(info_value(lib, "systemname")).filter(|n| !n.is_empty()).unwrap_or_else(|| lib.name.clone());
        return vec![(id, name, lib.extensions.clone())];
    }
    found.iter().map(|s| (s.id.clone(), s.name.clone(), s.extensions.iter().filter(|e| lib.extensions.contains(e)).cloned().collect())).collect()
}

/// The canonical pad: the RetroPad's sixteen controls in its order, and its two sticks and two triggers.
fn pad(ports: usize, labels: &[(u32, String)]) -> Controller {
    const CONTROLS: [Control; 16] = [
        Control::B, Control::Y, Control::Select, Control::Start, Control::Up, Control::Down, Control::Left, Control::Right,
        Control::A, Control::X, Control::L, Control::R, Control::L2, Control::R2, Control::L3, Control::R3,
    ];
    let label = |bit: u32, c: Control| labels.iter().find(|l| l.0 == bit).map_or_else(|| c.name().to_owned(), |l| l.1.clone());
    Controller {
        id: "retropad".into(),
        label: "Controller".into(),
        ports: (0..ports as u32).collect(),
        buttons: CONTROLS.iter().enumerate().map(|(bit, &c)| Button { bit: bit as u32, control: Some(c), label: label(bit as u32, c) }).collect(),
        axes: [(0, v1::Axis::LeftX, "Left stick X"), (1, v1::Axis::LeftY, "Left stick Y"), (2, v1::Axis::RightX, "Right stick X"), (3, v1::Axis::RightY, "Right stick Y"), (4, v1::Axis::LeftTrigger, "L2"), (5, v1::Axis::RightTrigger, "R2")]
            .iter()
            .map(|&(axis, control, label)| v1::AxisControl { axis, control: Some(control), trigger: control.is_trigger(), label: label.into() })
            .collect(),
    }
}

/// The `.info` file's firmware entries, numbered from 1.
fn firmware(lib: &Library) -> Vec<Firmware> {
    let count: usize = info_value(lib, "firmware_count").parse().unwrap_or(0);
    (0..count.min(64))
        .map(|i| {
            let path = info_value(lib, &format!("firmware{i}_path"));
            let optional = info_value(lib, &format!("firmware{i}_opt")) == "true";
            Firmware { which: 1 + i as u32, name: path.clone(), label: Some(info_value(lib, &format!("firmware{i}_desc"))).filter(|d| !d.is_empty()).unwrap_or(path), size: 0, required: !optional, ..Firmware::default() }
        })
        .collect()
}

impl Machine {
    fn shared(&mut self) -> &mut Shared {
        &mut self.shared
    }

    /// The events the callbacks queued, raised for the host.
    fn raise(&mut self) {
        for (kind, a, b) in std::mem::take(&mut self.shared.events) {
            v1::emit(kind, a, b);
        }
    }

    fn scan_saves(&mut self) {
        let c = self.saves.scan();
        for which in c.changed {
            v1::emit(event::BATTERY, which as i64, 0);
        }
        if c.listed {
            v1::emit(event::MACHINE_INFO, 0, 0);
        }
    }

    fn memory(&self, id: u32) -> &'static [u8] {
        let (p, n) = unsafe { ((self.lib.api.get_memory_data)(id), (self.lib.api.get_memory_size)(id)) };
        if p.is_null() || n == 0 { &[] } else { unsafe { std::slice::from_raw_parts(p as *const u8, n) } }
    }

    fn refresh_blocks(&mut self) {
        if self.blocks {
            self.spaces = blocks(self.lib);
        }
    }

    fn platform(&self) -> Option<u32> {
        (self.shared.quirks & (sys::QUIRK_ENDIAN_DEPENDENT | sys::QUIRK_PLATFORM_DEPENDENT) != 0).then(state::platform)
    }

    fn serialize_into(&self, out: &mut [u8]) -> Result<usize, i32> {
        let n = unsafe { (self.lib.api.serialize_size)() };
        if out.len() < state::HEADER + n {
            return Err(status::BUFFER_TOO_SMALL);
        }
        if !unsafe { (self.lib.api.serialize)(out[state::HEADER..].as_mut_ptr() as *mut c_void, n) } {
            v1::detail("the core did not write its state");
            return Err(STATUS_STATE_REFUSED);
        }
        state::seal(out, n, self.identity, self.platform(), self.disc);
        Ok(state::HEADER + n)
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        unsafe {
            (self.lib.api.unload_game)();
            (self.lib.api.deinit)();
        }
        front::with_declared(|d| d.initialised = false);
        front::bind(std::ptr::null_mut());
        front::flush_logs();
        LIVE.store(false, Ordering::Release);
    }
}

impl StateMachine for Machine {
    type Error = Fail;

    fn load_state(&mut self, data: &[u8]) -> Result<(), Fail> {
        let payload = state::open(data, self.identity, self.platform(), self.disc).map_err(|r| {
            v1::detail(match r {
                state::Refusal::Foreign => "not a libretro host's state",
                state::Refusal::OtherCore => "a state of another libretro core or of another version of it",
                state::Refusal::Version => "a state of another version of the libretro host",
                state::Refusal::Platform => "a state saved on another platform",
                state::Refusal::Damaged => "the state's payload does not match its digest",
                state::Refusal::WrongMedia => "the state was saved over another disc than the one in the drive",
            });
            Fail(match r {
                state::Refusal::Foreign | state::Refusal::OtherCore => status::FOREIGN,
                state::Refusal::Version => status::STATE_VERSION,
                state::Refusal::Platform => STATUS_PLATFORM,
                state::Refusal::Damaged => status::TRUNCATED,
                state::Refusal::WrongMedia => status::WRONG_MEDIA,
            })
        })?;
        let mut before = vec![0u8; state::HEADER + unsafe { (self.lib.api.serialize_size)() }];
        let kept = self.serialize_into(&mut before).map_err(Fail)?;
        if unsafe { (self.lib.api.unserialize)(payload.as_ptr() as *const c_void, payload.len()) } {
            self.refresh_blocks();
            front::flush_logs();
            return Ok(());
        }
        let restored = unsafe { (self.lib.api.unserialize)(before[state::HEADER..kept].as_ptr() as *const c_void, kept - state::HEADER) };
        self.refresh_blocks();
        v1::detail(if restored { "the core refused the state, and the machine was restored as it was" } else { "the core refused the state and then its own copy of the machine" });
        front::flush_logs();
        Err(Fail(STATUS_STATE_REFUSED))
    }

    fn state_size(&self) -> usize {
        state::HEADER + unsafe { (self.lib.api.serialize_size)() }
    }

    fn save_state(&self, out: &mut [u8]) -> Result<usize, Fail> {
        self.serialize_into(out).map_err(Fail)
    }

    fn layout(&self) -> String {
        state::layout(unsafe { (self.lib.api.serialize_size)() })
    }
}

impl v1::Core for Machine {
    const CAPABILITIES: u64 = caps::RESET
        | caps::PHASES
        | caps::AXES
        | caps::SETTINGS
        | caps::SETTING_NOTES
        | caps::PRESENCE
        | caps::SNAPSHOT
        | caps::FRAME_SERIAL
        | caps::BATTERY_DIRTY
        | caps::IMAGE_PATH
        | caps::MEDIA;

    fn info() -> Info {
        let lib = match ready() {
            Ok(lib) => lib,
            Err(why) => {
                return Info {
                    id: "libretro-host".into(),
                    name: "libretro host".into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                    license: "GPL-3.0-or-later".into(),
                    description: Some(format!("No libretro core runs here: {why}.")),
                    ..Info::default()
                };
            }
        };
        let ports = front::ports();
        let id = format!("libretro-{}", lib.stem.to_ascii_lowercase().replace(|c: char| !c.is_ascii_alphanumeric(), "-"));
        let mut host_requires = vec!["named_files".to_owned()];
        if lib.need_fullpath {
            host_requires.push("image_path".into());
        }
        Info {
            id,
            name: Some(info_value(lib, "corename")).filter(|n| !n.is_empty()).unwrap_or_else(|| lib.name.clone()),
            display_name: Some(info_value(lib, "display_name")).filter(|n| !n.is_empty()),
            version: lib.version.clone(),
            license: Some(info_value(lib, "license")).filter(|l| !l.is_empty()).unwrap_or_else(|| "NOASSERTION".into()),
            authors: info_value(lib, "authors").split('|').filter(|a| !a.is_empty()).map(str::to_owned).collect(),
            url: None,
            description: Some(format!("{} {}, a libretro core, through EmuSen's libretro host {}; SHA-256 {}.", lib.name, lib.version, env!("CARGO_PKG_VERSION"), lib.sha256)),
            systems: systems(lib)
                .into_iter()
                .map(|(id, name, extensions)| System { id, name, extensions, controllers: vec![pad(ports, &[])], firmware: firmware(lib), ..System::default() })
                .collect(),
            host_requires,
            deterministic: false,
            accuracy: None,
        }
    }

    fn settings_schema() -> Vec<Setting> {
        if ready().is_err() {
            return Vec::new();
        }
        front::with_declared(|d| {
            d.options
                .declared
                .iter()
                .map(|o| Setting {
                    key: o.key.clone(),
                    label: if o.label.is_empty() { o.key.clone() } else { o.label.clone() },
                    help: o.info.clone(),
                    kind: SettingKind::Choice(o.values.iter().map(|(v, l)| Choice::new(v, l)).collect()),
                    default: o.default.clone(),
                    scope: Scope::Run,
                    category: o.category.as_ref().and_then(|c| d.options.categories.get(c)).cloned(),
                    effect: Effect::Unrated,
                    advanced: false,
                    hidden: false,
                    restart: false,
                })
                .filter(|s| s.check().is_ok())
                .collect()
        })
    }

    fn firmware_for(_image: &[u8]) -> Vec<Firmware> {
        ready().map(firmware).unwrap_or_default()
    }

    fn status_text(code: i32) -> Option<String> {
        status_words(code).map(str::to_owned)
    }

    fn battery_files(_system: &str) -> Option<Vec<Battery>> {
        Some(vec![Battery::game(SRAM, ".srm"), Battery::game(RTC, ".rtc")])
    }

    fn battery_families(_system: &str) -> Vec<BatteryKind> {
        vec![BatteryKind::Console]
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        let systems = ready().map(systems).unwrap_or_default();
        let first = systems.first().map(|s| s.0.clone()).unwrap_or_default();
        Self::create_as(&first, request)
    }

    fn create_as(system: &str, r: &Create<'_>) -> Result<Self, i32> {
        let lib = ready().map_err(|why| {
            v1::detail(&why);
            STATUS_NO_CORE
        })?;
        if LIVE.swap(true, Ordering::AcqRel) {
            return Err(STATUS_BUSY);
        }
        let made = Self::make(lib, system, r);
        if made.is_err() {
            front::bind(std::ptr::null_mut());
            LIVE.store(false, Ordering::Release);
        }
        front::flush_logs();
        made
    }

    fn machine_info(&self) -> MachineInfo {
        let g = self.shared.av.geometry;
        let aspect = if g.aspect_ratio > 0.0 { video::ratio(g.aspect_ratio as f64, 100_000) } else { (g.base_width as u64, g.base_height.max(1) as u64) };
        let mut battery = Vec::new();
        if !self.memory(sys::MEMORY_SAVE_RAM).is_empty() {
            battery.push(Battery::game(SRAM, ".srm"));
        }
        if !self.memory(sys::MEMORY_RTC).is_empty() {
            battery.push(Battery::game(RTC, ".rtc"));
        }
        MachineInfo {
            system: self.system.clone(),
            region: None,
            frame_rate: video::ratio(self.shared.av.timing.fps, 1_000_000),
            video: Video { base_width: g.base_width, base_height: g.base_height, max_width: g.max_width, max_height: g.max_height, aspect: (aspect.0 as u32, aspect.1 as u32), formats: vec![self.shared.picture.format] },
            audio: Audio { rate: crate::audio::rounded(self.shared.sound.rate()), channels: Vec::new() },
            ports: (0..front::ports() as u32).map(|port| Port { port, controller: Some("retropad".into()) }).collect(),
            spaces: self
                .spaces
                .iter()
                .enumerate()
                .map(|(i, s)| Space { cheats: crate::cheat_space(&self.system).is_some_and(|c| c == s.name), byte_order: s.big_endian.then_some(v1::BYTE_ORDER_BIG), ..Space::new(i as u32, &s.name) })
                .collect(),
            processors: Vec::new(),
            battery,
            state: StateFormat { format: "ESLR".into(), version: state::STATE_VERSION as i64, loads_from: vec![state::STATE_VERSION as i64] },
            phases: vec!["retro_run".into(), "picture".into(), "saves".into()],
            patches: None,
            skip_rendering_state_neutral: false,
            firmware: Vec::new(),
        }
    }

    fn reset(&mut self) -> Result<(), i32> {
        unsafe { (self.lib.api.reset)() };
        self.refresh_blocks();
        front::flush_logs();
        self.raise();
        Ok(())
    }

    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        if self.shared.shutdown {
            v1::detail("the core asked to stop");
            return Err(STATUS_SHUTDOWN);
        }
        let start = std::time::Instant::now();
        if let Some((cb, reference)) = self.shared.frame_time {
            unsafe { cb(reference) };
        }
        unsafe { (self.lib.api.run)() };
        let ran = start.elapsed().as_nanos() as i64;
        self.frames += 1;
        let saves_start = std::time::Instant::now();
        if self.frames % 15 == 0 {
            self.scan_saves();
        }
        self.phases = [ran, 0, saves_start.elapsed().as_nanos() as i64];
        front::flush_logs();
        self.raise();
        Ok(())
    }

    fn set_options(&mut self, flags: u32) {
        self.shared().skip_video = flags & 1 != 0;
    }

    fn frame_count(&self) -> i64 {
        self.frames
    }

    fn phases(&self) -> Vec<i64> {
        self.phases.to_vec()
    }

    fn frame_info(&self) -> FrameInfo {
        let p = &self.shared.picture;
        let g = self.shared.av.geometry;
        let (num, den) = if g.aspect_ratio > 0.0 { video::ratio(g.aspect_ratio as f64, 100_000) } else { (0, 0) };
        FrameInfo { format: p.format, stride: p.stride as i32, aspect_num: num as u32, aspect_den: den as u32, serial: p.serial, bytes: p.bytes().len() as i64, ..FrameInfo::rgba(p.width as i32, p.height as i32) }
    }

    fn frame(&self) -> &[u8] {
        self.shared.picture.bytes()
    }

    fn audio_rate(&self) -> i32 {
        crate::audio::rounded(self.shared.sound.next_rate())
    }

    fn audio_rate_exact(&self) -> Option<(u64, u64)> {
        Some(self.shared.sound.next_rate())
    }

    fn audio_buffered(&self) -> usize {
        self.shared.sound.buffered()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        self.shared().sound.drain(out, max_frames).0
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.shared().sound.set_limit(samples);
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        if port as usize >= front::ports() {
            return Err(status::NO_SUCH_PORT);
        }
        self.shared().input.set_buttons(port as usize, mask, changed);
        Ok(())
    }

    fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), i32> {
        if port as usize >= front::ports() {
            return Err(status::NO_SUCH_PORT);
        }
        if self.shared().input.set_axis(port as usize, axis, value) { Ok(()) } else { Err(status::NOT_SUPPORTED) }
    }

    fn set_present(&mut self, port: u32, present: bool) -> Result<(), i32> {
        if port as usize >= front::ports() {
            return Err(status::NO_SUCH_PORT);
        }
        self.shared().input.pads[port as usize].present = present;
        let device = if present { device_for(port as usize) } else { sys::DEVICE_NONE };
        unsafe { (self.lib.api.set_controller_port_device)(port, device) };
        front::flush_logs();
        Ok(())
    }

    fn snapshot_offered(&self) -> bool {
        self.shared.quirks & (sys::QUIRK_INCOMPLETE | sys::QUIRK_SINGLE_SESSION) == 0
    }

    fn snapshot_size(&self) -> usize {
        self.state_size()
    }

    fn save_snapshot(&mut self, out: &mut [u8]) -> Result<usize, i32> {
        self.shared().savestate_context = sys::SAVESTATE_CONTEXT_RUNAHEAD_SAME_INSTANCE;
        let r = self.serialize_into(out);
        self.shared().savestate_context = sys::SAVESTATE_CONTEXT_NORMAL;
        r
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        self.spaces.get(space as usize).map(|s| s.size as i64).ok_or(status::NO_SUCH_SPACE)
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        self.spaces.get(space as usize).ok_or(status::NO_SUCH_SPACE)?.read(address, out);
        Ok(())
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        if self.spaces.get(space as usize).ok_or(status::NO_SUCH_SPACE)?.write(address, data) { Ok(()) } else { Err(status::READ_ONLY) }
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        let tracked = v1::flags::BATTERY_TRACKED;
        let changed = |now: &[u8], saved: &[u8]| if now != saved { v1::flags::BATTERY_CHANGED } else { 0 };
        match which {
            SRAM => {
                let m = self.memory(sys::MEMORY_SAVE_RAM);
                Ok((m, tracked | changed(m, &self.sram_saved)))
            }
            RTC => {
                let m = self.memory(sys::MEMORY_RTC);
                Ok((m, tracked | changed(m, &self.rtc_saved)))
            }
            w => self.saves.bytes(w).map(|(b, dirty)| (b, tracked | if dirty { v1::flags::BATTERY_CHANGED } else { 0 })).ok_or(status::BAD_FILE),
        }
    }

    fn battery_saved(&mut self, which: u32) -> Result<(), i32> {
        match which {
            SRAM => self.sram_saved = self.memory(sys::MEMORY_SAVE_RAM).to_vec(),
            RTC => self.rtc_saved = self.memory(sys::MEMORY_RTC).to_vec(),
            w if self.saves.saved(w) => {}
            _ => return Err(status::BAD_FILE),
        }
        Ok(())
    }

    fn drives(&self) -> Vec<Drive> {
        let Some(d) = self.shared.disk.as_ref() else { return Vec::new() };
        let lid_open = d.get_eject_state.is_some_and(|f| unsafe { f() });
        let images = d.get_num_images.map_or(0, |f| unsafe { f() });
        let index = d.get_image_index.map_or(0, |f| unsafe { f() });
        vec![Drive { drive: 0, label: "Disc drive".into(), kind: "disc".into(), lid_open, loaded: index < images }]
    }

    fn media_lid(&mut self, drive: u32, open: bool) -> Result<(), i32> {
        let d = self.shared.disk.as_ref().filter(|_| drive == 0).ok_or(status::NO_SUCH_DRIVE)?;
        let set = d.set_eject_state.ok_or(status::NOT_SUPPORTED)?;
        let done = unsafe { set(open) };
        front::flush_logs();
        if done { Ok(()) } else { Err(STATUS_REFUSED) }
    }

    fn media_insert(&mut self, drive: u32, insert: &Insert<'_>) -> Result<(), i32> {
        let d = *self.shared.disk.as_ref().filter(|_| drive == 0).ok_or(status::NO_SUCH_DRIVE)?;
        if !insert.restore && !d.get_eject_state.is_some_and(|f| unsafe { f() }) {
            return Err(status::LID_CLOSED);
        }
        let (Some(images), Some(index), Some(replace), Some(set_index)) = (d.get_num_images, d.get_image_index, d.replace_image_index, d.set_image_index) else {
            return Err(status::NOT_SUPPORTED);
        };
        let closed = !d.get_eject_state.is_some_and(|f| unsafe { f() });
        if closed {
            d.set_eject_state.map(|f| unsafe { f(true) });
        }
        let r = match insert.image {
            None => unsafe { replace(index(), std::ptr::null()) },
            Some(image) => {
                let path = v1::image_path().map(|p| CString::new(p).unwrap_or_default());
                if self.lib.need_fullpath && path.is_none() {
                    v1::detail("this core opens the disc's file itself, and the host gave no path to it");
                    return Err(status::BAD_IMAGE);
                }
                let data = (!self.lib.need_fullpath).then(|| image.to_vec());
                let game = sys::retro_game_info {
                    path: path.as_ref().map_or(std::ptr::null(), |p| p.as_ptr()),
                    data: data.as_ref().map_or(std::ptr::null(), |k| k.as_ptr() as *const c_void),
                    size: data.as_ref().map_or(0, |k| k.len()),
                    meta: std::ptr::null(),
                };
                let added = d.add_image_index.is_some_and(|f| unsafe { f() });
                let at = unsafe { images() }.saturating_sub(1);
                let ok = added && unsafe { replace(at, &game) } && unsafe { set_index(at) };
                self.discs.push((path, data));
                ok
            }
        };
        if closed {
            d.set_eject_state.map(|f| unsafe { f(false) });
        }
        front::flush_logs();
        if !r {
            return Err(STATUS_REFUSED);
        }
        self.disc = state::disc(insert.image);
        Ok(())
    }

    fn named_files(&self) -> Vec<NamedFile> {
        self.saves.listed()
    }

    fn set_settings(&mut self, settings: &v1::Settings) -> Result<(), i32> {
        front::with_declared(|d| {
            for (k, v) in settings.pairs() {
                d.options.set(k, v);
            }
        });
        Ok(())
    }

    fn setting_notes(&self) -> Vec<(String, String)> {
        let update = front::with_declared(|d| d.options.update_display);
        if let Some(cb) = update {
            unsafe { cb() };
        }
        front::with_declared(|d| {
            let current = d.options.current();
            let mut notes: Vec<(String, String)> = d.options.hidden.iter().filter(|(_, h)| **h).map(|(k, _)| (k.clone(), "Not used with the current settings.".to_owned())).collect();
            for k in &d.options.changed_by_core {
                let value = current.iter().find(|c| c.0 == *k).map_or("", |c| c.1.as_str());
                notes.push((k.clone(), format!("The core set this to {value} itself.")));
            }
            if d.declared_late {
                notes.push(("libretro".into(), "The core declared options while loading the game; they take their defaults until the core is loaded again.".into()));
            }
            notes.retain(|n| d.options.declared.iter().any(|o| o.key == n.0) || n.0 == "libretro");
            notes
        })
    }
}

/// The spaces of a core with no memory maps, from `retro_get_memory_data`, asked again after every load and reset since
/// a core may move them.
fn blocks(lib: &Library) -> Vec<memory::Space> {
    [("RAM", sys::MEMORY_SYSTEM_RAM), ("VRAM", sys::MEMORY_VIDEO_RAM), ("SRAM", sys::MEMORY_SAVE_RAM)]
        .iter()
        .filter_map(|&(name, id)| memory::block(name, unsafe { (lib.api.get_memory_data)(id) } as *mut u8, unsafe { (lib.api.get_memory_size)(id) }))
        .collect()
}

/// The device a present pad is set to: the core's pad with sticks, else the analog pad, else the RetroPad.
fn device_for(port: usize) -> std::ffi::c_uint {
    front::with_declared(|d| {
        let types = d.controllers.get(port).cloned().unwrap_or_default();
        types.iter().map(|t| t.1).find(|&id| id & sys::DEVICE_MASK == sys::DEVICE_ANALOG && id != sys::DEVICE_ANALOG).or_else(|| types.iter().map(|t| t.1).find(|&id| id == sys::DEVICE_ANALOG)).unwrap_or(if types.is_empty() { sys::DEVICE_ANALOG } else { sys::DEVICE_JOYPAD })
    })
}

impl Machine {
    fn make(lib: &'static Library, system: &str, r: &Create<'_>) -> Result<Machine, i32> {
        let folders = files::Folders::for_entry(&lib.folder, &lib.stem).map_err(|e| {
            v1::detail(&format!("the core's folders: {e}"));
            STATUS_FOLDERS
        })?;
        let mut shared = Box::new(Shared::new(r.pixel_formats));
        let c = |p: &std::path::Path| CString::new(p.to_string_lossy().as_bytes()).unwrap_or_default();
        let image_path = v1::image_path();
        let content = image_path.as_deref().and_then(|p| std::path::Path::new(p).parent()).map(c);
        shared.folders = Some(front::Folders { system: c(&folders.system), save: c(&folders.save), content });
        front::bind(&mut *shared);
        front::with_declared(|d| {
            for (k, v) in r.settings.pairs() {
                d.options.set(k, v);
            }
            d.options.updated = false;
            d.loaded_once = true;
        });

        let mut saves = Saves::new(folders.save.clone());
        let names = v1::file_names();
        let passed: Vec<(u32, String, &[u8])> = r.files.iter().filter_map(|f| names.iter().find(|n| n.0 == f.which).map(|n| (f.which, n.1.clone(), f.data))).collect();
        let fail_folders = |e: std::io::Error| {
            v1::detail(&format!("the core's save folder: {e}"));
            STATUS_FOLDERS
        };
        saves.place(&passed).map_err(fail_folders)?;
        let mut firmware_written = Written(Vec::new());
        for fw in firmware(lib) {
            if let Some(file) = r.files.iter().find(|f| f.which == fw.which) {
                let target = folders.system.join(&fw.name);
                let target = if target.extension().is_none() { target.join(format!("firmware-{}.bin", fw.which)) } else { target };
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).map_err(fail_folders)?;
                }
                std::fs::write(&target, file.data).map_err(fail_folders)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600));
                }
                firmware_written.0.push(target);
            }
        }

        let mut strings = Vec::new();
        let path_c = image_path.as_deref().map(|p| CString::new(p).unwrap_or_default());
        if lib.need_fullpath && path_c.is_none() {
            v1::detail("this core opens the game file itself, and the host gave no path to it (an altered or compressed image cannot be given)");
            return Err(status::BAD_IMAGE);
        }
        // Copied, never borrowed: a libretro core may keep every image it was given until it is unloaded.
        let image = (!lib.need_fullpath).then(|| r.image.to_vec());
        let (data, size) = image.as_ref().map_or((std::ptr::null(), 0), |k| (k.as_ptr() as *const c_void, k.len()));
        let path_ptr = path_c.as_ref().map_or(std::ptr::null(), |p| p.as_ptr());
        {
            let pp = image_path.as_deref().map(std::path::Path::new);
            let part = |s: Option<&std::ffi::OsStr>| s.map(|s| CString::new(s.to_string_lossy().into_owned()).unwrap_or_default());
            let ext_strings: Vec<Option<CString>> = vec![image_path.as_deref().map(|p| CString::new(p).unwrap_or_default()), part(pp.and_then(|p| p.parent()).map(|d| d.as_os_str())), part(pp.and_then(|p| p.file_stem())), part(pp.and_then(|p| p.extension()))];
            let at = |i: usize| ext_strings[i].as_ref().map_or(std::ptr::null(), |c| c.as_ptr());
            let ext = sys::retro_game_info_ext {
                full_path: at(0),
                archive_path: std::ptr::null(),
                archive_file: std::ptr::null(),
                dir: at(1),
                name: at(2),
                ext: at(3),
                meta: std::ptr::null(),
                data,
                size,
                file_in_archive: false,
                persistent_data: image.is_some(),
            };
            shared.game_info_ext = Some(front::GameInfoExt { strings: ext_strings.into_iter().flatten().collect(), ext });
        }
        if let Some(p) = path_c {
            strings.push(p);
        }
        let game = sys::retro_game_info { path: path_ptr, data, size, meta: std::ptr::null() };
        let loaded = unsafe { (lib.api.load_game)(&game) };
        front::flush_logs();
        if !loaded {
            let asked = front::with_declared(|d| d.contexts_asked.clone());
            let why = if asked.is_empty() { "the core refused the game; its log says why".to_owned() } else { format!("the core refused the game after asking for a hardware context this host does not yet give ({asked:?})") };
            v1::detail(&why);
            return Err(STATUS_REFUSED);
        }
        let mut av = sys::retro_system_av_info::default();
        unsafe { (lib.api.get_system_av_info)(&mut av) };
        let handed = video::handed_format(shared.retro_format, r.pixel_formats);
        shared.picture = Picture::new(av.geometry.max_width, av.geometry.max_height, handed);
        shared.av = av;
        shared.sound.set_rate(video::ratio(av.timing.sample_rate, 1_000_000));
        shared.events.clear();

        let mut machine = Machine {
            lib,
            shared,
            frames: 0,
            system: system.to_owned(),
            identity: state::identity(&lib.name, &lib.version),
            spaces: Vec::new(),
            saves,
            sram_saved: Vec::new(),
            rtc_saved: Vec::new(),
            _image: image,
            _strings: strings,
            phases: [0; 3],
            blocks: false,
            _firmware: firmware_written,
            discs: Vec::new(),
            disc: state::disc(Some(r.image)),
        };
        front::bind(&mut *machine.shared);
        for (which, id) in [(SRAM, sys::MEMORY_SAVE_RAM), (RTC, sys::MEMORY_RTC)] {
            let (p, n) = unsafe { ((lib.api.get_memory_data)(id), (lib.api.get_memory_size)(id)) };
            if let Some(file) = r.files.iter().find(|f| f.which == which)
                && !p.is_null()
            {
                let k = n.min(file.data.len());
                unsafe { std::ptr::copy_nonoverlapping(file.data.as_ptr(), p as *mut u8, k) };
            }
        }
        machine.sram_saved = machine.memory(sys::MEMORY_SAVE_RAM).to_vec();
        machine.rtc_saved = machine.memory(sys::MEMORY_RTC).to_vec();
        for port in 0..front::ports() {
            unsafe { (lib.api.set_controller_port_device)(port as u32, device_for(port)) };
        }
        machine.spaces = match machine.shared.maps.take() {
            Some(maps) if !maps.is_empty() => {
                let mut s = maps;
                if let Some(name) = crate::cheat_space(system)
                    && !s.iter().any(|x| x.name == name)
                    && let Some(b) = memory::block(&name, unsafe { (lib.api.get_memory_data)(sys::MEMORY_SYSTEM_RAM) } as *mut u8, unsafe { (lib.api.get_memory_size)(sys::MEMORY_SYSTEM_RAM) })
                {
                    s.push(b);
                }
                s
            }
            _ => {
                machine.blocks = true;
                blocks(lib)
            }
        };
        machine.scan_saves();
        front::flush_logs();
        Ok(machine)
    }
}

