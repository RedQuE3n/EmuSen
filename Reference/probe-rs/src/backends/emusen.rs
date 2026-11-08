// The emusen backend: EmuSen itself, driven through the frontend API of the
// platform library it loads, as a front end drives it. It replaces Pharaoh's
// --probe and writes what Pharaoh wrote. See EmuSen_Backend.md §40.11.4.
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::backend::{
    InputSchedule, MemorySpace, ProbeBackend, ProbeButton, ProbeIdentity, ProbeOptions, ScreenFormat, ScreenView,
};

pub mod sys {
    #![allow(unsafe_op_in_unsafe_fn)]
    include!(concat!(env!("OUT_DIR"), "/emusen_sys.rs"));
}

use sys::emusen_control as control;

// The probe's names for the NES's spaces, not EmuSen's, so a consumer cannot
// tell which emulator wrote a dump.
const NES_SPACES: [(&str, &str); 6] = [("ram", "RAM"), ("work", "PRGRAM"), ("nametable", "CIRAM"), ("oam", "OAM"), ("palette", "PALETTE"), ("chr", "CHR")];

#[cfg(target_os = "windows")]
const LIBRARY: &str = "emusen_platform.dll";
#[cfg(target_os = "macos")]
const LIBRARY: &str = "libemusen_platform.dylib";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const LIBRARY: &str = "libemusen_platform.so";

pub struct EmuSenBackend {
    folder: PathBuf,
    api: Option<sys::Platform>,
    runtime: *mut sys::emusen_crystal_runtime,
    session: u32,
    system: String,
    identity: ProbeIdentity,
    frame: u32,
    schedule: InputSchedule,
    live: [bool; ProbeButton::ALL.len()],
    // The spaces and the picture as the last frame left them, read once a stop.
    spaces: Vec<(String, Vec<u8>)>,
    screen: Vec<u8>,
    width: u32,
    height: u32,
}

impl EmuSenBackend {
    pub fn new(folder: &str) -> EmuSenBackend {
        EmuSenBackend {
            folder: PathBuf::from(folder),
            api: None,
            runtime: std::ptr::null_mut(),
            session: 0,
            system: String::new(),
            identity: ProbeIdentity::default(),
            frame: 0,
            schedule: InputSchedule::default(),
            live: [false; ProbeButton::ALL.len()],
            spaces: Vec::new(),
            screen: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    fn api(&self) -> &sys::Platform {
        self.api.as_ref().expect("the platform library is loaded")
    }

    // The runtime's last error, in its words.
    fn last_error(&self) -> String {
        let mut out = vec![0u8; 1024];
        let n = unsafe { self.api().emusen_crystal_last_error(self.runtime, out.as_mut_ptr(), out.len()) };
        out.truncate(n.clamp(0, 1024) as usize);
        String::from_utf8_lossy(&out).into_owned()
    }

    // A call that answers text: asked once, and once more in the length it said it needs.
    fn text(&self, ask: impl Fn(*mut u8, usize) -> i64) -> Result<String, String> {
        let mut out = vec![0u8; 4096];
        let mut n = ask(out.as_mut_ptr(), out.len());
        if n > out.len() as i64 {
            out.resize(n as usize, 0);
            n = ask(out.as_mut_ptr(), out.len());
        }
        if n < 0 {
            return Err(format!("status {n}: {}", self.last_error()));
        }
        out.truncate(n as usize);
        String::from_utf8(out).map_err(|e| e.to_string())
    }

    fn json(&self, ask: impl Fn(*mut u8, usize) -> i64) -> Result<Value, String> {
        serde_json::from_str(&self.text(ask)?).map_err(|e| e.to_string())
    }

    // The probe's home for both of the runtime's redirects, so a run reads and writes nothing of a player's.
    fn redirect(&self, home: &str) {
        let config = Path::new(home).join("etc/EmuSen").to_string_lossy().into_owned();
        unsafe {
            self.api().emusen_galaxia_set_override(sys::EMUSEN_GALAXIA_OVERRIDE_DATA, home.as_ptr(), home.len());
            self.api().emusen_galaxia_set_override(sys::EMUSEN_GALAXIA_OVERRIDE_CONFIG, config.as_ptr(), config.len());
        }
    }

    // What the route says of the file: its system, and an NES cartridge's header as the pack reads it.
    fn identify(&mut self, rom_path: &str) -> Result<(), String> {
        let (rt, api) = (self.runtime, self.api());
        let route = self.json(|out, len| unsafe { api.emusen_crystal_route(rt, rom_path.as_ptr(), rom_path.len(), out, len) })?;
        self.system = route["system"].as_str().unwrap_or("").to_string();
        let cart = &route["cartridge"];
        if cart.is_object() {
            self.identity = ProbeIdentity {
                board: cart["board"].as_i64().map_or(String::new(), |b| b.to_string()),
                region: "ntsc".to_string(),
                header_trust: cart["header_trust"].as_str().unwrap_or("").to_string(),
                prg_bytes: cart["prg_bytes"].as_u64().unwrap_or(0),
                chr_bytes: cart["chr_bytes"].as_u64().unwrap_or(0),
                save_loaded: cart["battery"].as_bool().unwrap_or(false),
            };
        } else {
            self.identity = ProbeIdentity { region: "ntsc".to_string(), ..ProbeIdentity::default() };
        }
        Ok(())
    }

    // The spaces and the picture read again, where the run has stopped.
    fn read_back(&mut self) {
        let (rt, session, api) = (self.runtime, self.session, self.api());
        let mut spaces = Vec::new();
        // The debugger's reads go through the session's read-only table, the session service's one surface.
        let table = unsafe { api.emusen_crystal_session_service(rt, session, sys::EMUSEN_CRYSTAL_SERVICE_READ_ONLY) };
        if self.system == "nes" && table.is_null() {
            println!("[WARN] the session's table could not be had: {}", self.last_error());
        } else if self.system == "nes" {
            let t = unsafe { &*table };
            let (spaces_of, read, words) = (t.spaces.expect("spaces"), t.space_read.expect("space_read"), t.last_error.expect("last_error"));
            let sizes = self.json(|out, len| unsafe { spaces_of(t.ctx, session, out, len) }).unwrap_or(Value::Null);
            for (name, space) in NES_SPACES {
                let size = sizes.as_array().and_then(|a| a.iter().find(|s| s["name"] == space)).and_then(|s| s["size"].as_u64()).unwrap_or(0) as usize;
                if size == 0 {
                    continue;
                }
                let mut data = vec![0u8; size];
                let status = unsafe { read(t.ctx, session, space.as_ptr(), space.len(), 0, data.as_mut_ptr(), data.len()) };
                if status < 0 {
                    let mut said = vec![0u8; 1024];
                    let n = unsafe { words(t.ctx, said.as_mut_ptr(), said.len()) };
                    said.truncate(n.clamp(0, 1024) as usize);
                    println!("[WARN] {space} could not be read: {}", String::from_utf8_lossy(&said));
                }
                spaces.push((name.to_string(), data));
            }
        }
        let bytes = unsafe { api.emusen_crystal_picture_copy(rt, session, std::ptr::null_mut(), 0) }.max(0) as usize;
        let mut screen = vec![0u8; bytes];
        unsafe { api.emusen_crystal_picture_copy(rt, session, screen.as_mut_ptr(), screen.len()) };
        self.spaces = spaces;
        self.screen = screen;
    }

    // A pad's button at a frame: the schedule's, and on the first pad the live override's too.
    fn button_held(&self, port: u32, button: ProbeButton, frame: u32) -> bool {
        (port == 0 && self.live[button.index()]) || self.schedule.held_on(port, frame, button)
    }

    // The pads the schedule names, the first always among them.
    fn ports(&self) -> u32 {
        self.schedule.entries.iter().map(|e| e.port + 1).max().unwrap_or(1).max(1)
    }
}

// The probe's buttons as the core ABI's canonical controls.
fn control_of(button: ProbeButton) -> u32 {
    match button {
        ProbeButton::A => control::EMUSEN_CONTROL_A,
        ProbeButton::B => control::EMUSEN_CONTROL_B,
        ProbeButton::X => control::EMUSEN_CONTROL_X,
        ProbeButton::Y => control::EMUSEN_CONTROL_Y,
        ProbeButton::L => control::EMUSEN_CONTROL_L,
        ProbeButton::R => control::EMUSEN_CONTROL_R,
        ProbeButton::Up => control::EMUSEN_CONTROL_UP,
        ProbeButton::Down => control::EMUSEN_CONTROL_DOWN,
        ProbeButton::Left => control::EMUSEN_CONTROL_LEFT,
        ProbeButton::Right => control::EMUSEN_CONTROL_RIGHT,
        ProbeButton::Start => control::EMUSEN_CONTROL_START,
        ProbeButton::Select => control::EMUSEN_CONTROL_SELECT,
    }
}

impl ProbeBackend for EmuSenBackend {
    fn name(&self) -> &str {
        "emusen"
    }

    fn system(&self) -> &str {
        &self.system
    }

    fn identity(&self) -> ProbeIdentity {
        self.identity.clone()
    }

    fn load(&mut self, rom_path: &str, options: &ProbeOptions) -> bool {
        let library = self.folder.join(LIBRARY);
        match unsafe { sys::Platform::new(&library) } {
            Ok(api) => self.api = Some(api),
            Err(error) => {
                println!("[ERROR] {} could not be loaded: {error}", library.display());
                return false;
            }
        }
        let major = unsafe { self.api().emusen_crystal_abi_version() } >> 16;
        if major != sys::EMUSEN_CRYSTAL_MAJOR {
            println!("[ERROR] {} speaks the frontend API's major {major}, not {}", library.display(), sys::EMUSEN_CRYSTAL_MAJOR);
            return false;
        }
        let base = self.folder.to_string_lossy().into_owned();
        unsafe { self.api().emusen_galaxia_init(base.as_ptr(), base.len(), std::ptr::null(), 0) };
        self.redirect(&options.home_folder);

        let cores = serde_json::to_string(&[base.clone(), self.folder.join("cores").to_string_lossy().into_owned()]).unwrap();
        let config = sys::emusen_crystal_config {
            size: std::mem::size_of::<sys::emusen_crystal_config>() as u32,
            cores: cores.as_ptr(),
            cores_len: cores.len(),
            ..Default::default()
        };
        self.runtime = unsafe { self.api().emusen_crystal_new(&config) };
        if self.runtime.is_null() {
            println!("[ERROR] the runtime was not made: {}", self.last_error());
            return false;
        }
        if let Err(why) = self.identify(rom_path) {
            println!("[ERROR] {rom_path} was not routed: {why}");
            return false;
        }
        let params = sys::emusen_crystal_open_params {
            size: std::mem::size_of::<sys::emusen_crystal_open_params>() as u32,
            flags: sys::EMUSEN_CRYSTAL_OPEN_NO_BATTERY,
            path: rom_path.as_ptr(),
            path_len: rom_path.len(),
            ..Default::default()
        };
        let opened = unsafe { self.api().emusen_crystal_open(self.runtime, &params) };
        if opened <= 0 {
            println!("[ERROR] {rom_path} was not opened: {}", self.last_error());
            return false;
        }
        self.session = opened as u32;
        let (rt, session, api) = (self.runtime, self.session, self.api());
        if let Ok(info) = self.json(|out, len| unsafe { api.emusen_crystal_session_info(rt, session, out, len) }) {
            if let Some(notice) = info["notices"]["engine"].as_str() {
                println!("[core] {notice}");
            }
            self.width = info["picture"]["width"].as_u64().unwrap_or(0) as u32;
            self.height = info["picture"]["height"].as_u64().unwrap_or(0) as u32;
        }
        self.read_back();
        true
    }

    fn shutdown(&mut self) {
        if let Some(api) = self.api.as_ref()
            && !self.runtime.is_null()
        {
            unsafe {
                if self.session != 0 {
                    api.emusen_crystal_close(self.runtime, self.session);
                }
                api.emusen_crystal_free(self.runtime);
            }
            self.runtime = std::ptr::null_mut();
            self.session = 0;
        }
    }

    // One frame of the API at a time, each with the buttons the schedule and the live override hold for it; the API's ports count from 0, as the schedule's do.
    fn run_until(&mut self, frame: u32) {
        if self.frame >= frame {
            return;
        }
        while self.frame < frame {
            for port in 0..self.ports() {
                for button in ProbeButton::ALL {
                    let held = self.button_held(port, button, self.frame);
                    unsafe { self.api().emusen_crystal_input_button(self.runtime, self.session, port, control_of(button), u32::from(held)) };
                }
            }
            let mut info = sys::emusen_crystal_frame_info { size: std::mem::size_of::<sys::emusen_crystal_frame_info>() as u32, ..Default::default() };
            let status = unsafe { self.api().emusen_crystal_frame(self.runtime, self.session, &mut info) };
            if status < 0 {
                println!("[WARN] frame {} failed: {}", self.frame + 1, self.last_error());
            }
            if info.width > 0 && info.height > 0 {
                (self.width, self.height) = (info.width as u32, info.height as u32);
            }
            self.frame += 1;
        }
        self.read_back();
    }

    fn frame_count(&self) -> u32 {
        self.frame
    }

    fn spaces(&self) -> Vec<MemorySpace<'_>> {
        self.spaces.iter().map(|(name, data)| MemorySpace { name: name.clone(), data }).collect()
    }

    fn anchor_space(&self) -> &str {
        if self.system == "nes" { "ram" } else { "" }
    }

    fn set_schedule(&mut self, schedule: InputSchedule) {
        self.schedule = schedule;
    }

    fn set_button(&mut self, button: ProbeButton, held: bool) {
        self.live[button.index()] = held;
    }

    fn screen(&self) -> Option<ScreenView<'_>> {
        (!self.screen.is_empty()).then(|| ScreenView { data: &self.screen, width: self.width, height: self.height, format: ScreenFormat::Rgba8888 })
    }
}

impl Drop for EmuSenBackend {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_button_is_the_canonical_control_of_its_own_name() {
        let pairs = [
            (ProbeButton::A, control::EMUSEN_CONTROL_A),
            (ProbeButton::B, control::EMUSEN_CONTROL_B),
            (ProbeButton::Start, control::EMUSEN_CONTROL_START),
            (ProbeButton::Select, control::EMUSEN_CONTROL_SELECT),
            (ProbeButton::Up, control::EMUSEN_CONTROL_UP),
            (ProbeButton::Left, control::EMUSEN_CONTROL_LEFT),
        ];
        for (button, expected) in pairs {
            assert_eq!(control_of(button), expected, "{}", button.name());
        }
        let mut seen: Vec<u32> = ProbeButton::ALL.iter().map(|&b| control_of(b)).collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), ProbeButton::ALL.len());
    }

    // A press belongs to its pad, counted from 0 as the API counts ports; the live override is the first pad's.
    #[test]
    fn a_press_reaches_the_pad_the_schedule_names() {
        let mut backend = EmuSenBackend::new("/nowhere");
        let mut schedule = InputSchedule::default();
        schedule.add_on(2, 10, 4, ProbeButton::Up);
        schedule.add(10, 4, ProbeButton::Start);
        backend.set_schedule(schedule);
        assert_eq!(backend.ports(), 3);
        assert!(backend.button_held(2, ProbeButton::Up, 11) && !backend.button_held(0, ProbeButton::Up, 11));
        assert!(backend.button_held(0, ProbeButton::Start, 13) && !backend.button_held(0, ProbeButton::Start, 14));
        backend.set_button(ProbeButton::A, true);
        assert!(backend.button_held(0, ProbeButton::A, 99) && !backend.button_held(1, ProbeButton::A, 99));
        assert_eq!(EmuSenBackend::new("/nowhere").ports(), 1);
    }
}
