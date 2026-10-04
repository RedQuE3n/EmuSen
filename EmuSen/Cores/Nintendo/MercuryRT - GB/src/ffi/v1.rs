//! MercuryRT on the core ABI v1, beside its pre-stable exports until the v1 adapter is its only loader. The console's
//! facts its C# shim held are its descriptors here. See EmuSen_CoreAPI.md §13.1, §23.

use emusen_native::core::{
    self as v1, Battery, Button, Choice, Control, Controller, Create, Effect, FrameInfo, Info, MachineInfo, Port, Processor, Region, Scope, Setting,
    SettingKind, Space, StateFormat, System, Video, caps, status,
};

use super::{BATTERY_SPACE, SCREEN_HEIGHT, SCREEN_WIDTH, STATUS_ILLEGAL_OPCODE, STATUS_TOO_SHORT, STATUS_UNKNOWN_MODEL, STATUS_UNSUPPORTED_BOARD, build_rom_patches};
use crate::debug::Hooks;
use crate::machine::{CPU_CLOCK_HZ, CYCLES_PER_FRAME, Machine, Model, OLDEST_READABLE_VERSION, STATE_VERSION};
use crate::memory::cartridge::RomError;

/// `MercuryMachine.SpaceNames`, numbered as the ABI numbers them.
const SPACES: [&str; 7] = ["ROM", "VRAM", "CARTRAM", "WRAM", "OAM", "HRAM", "CPUBUS"];

/// The pad's bits (right, left, up, down, A, B, select, start) in `MercuryCore.PadButtons`' order, the order a bindings screen shows them.
const PAD: [(u32, Control, &str); 8] = [
    (2, Control::Up, "Up"),
    (3, Control::Down, "Down"),
    (1, Control::Left, "Left"),
    (0, Control::Right, "Right"),
    (4, Control::A, "A"),
    (5, Control::B, "B"),
    (6, Control::Select, "Select"),
    (7, Control::Start, "Start"),
];

/// `MercuryCore.ModelKey` and its values, as frontends already store them.
const MODEL_KEY: &str = "Model";
const MODELS: [(&str, Model); 3] = [("Auto", Model::Auto), ("Game Boy", Model::GameBoy), ("Game Boy Color", Model::GameBoyColor)];

fn pad() -> Controller {
    Controller {
        id: "gb.pad".into(),
        label: "Controller".into(),
        ports: vec![0],
        buttons: PAD.iter().map(|&(bit, control, label)| Button { bit, control: Some(control), label: label.into() }).collect(),
        axes: Vec::new(),
    }
}

impl v1::Core for Machine {
    const CAPABILITIES: u64 = caps::MUTES | caps::ROM_PATCHES | caps::DEBUG | caps::DEBUG_STACK;

    fn info() -> Info {
        let system = |id: &str, name: &str, ext: &str| System {
            id: id.into(),
            name: name.into(),
            extensions: vec![ext.into()],
            regions: vec![Region::Ntsc],
            controllers: vec![pad()],
            firmware: Vec::new(),
        };
        Info {
            id: "mercuryrt".into(),
            name: "MercuryRT".into(),
            display_name: Some("MercuryRT (Rust)".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("A Rust port of Mercury, the project's Game Boy and Game Boy Color core, exact against it in state, picture and sound.".into()),
            systems: vec![system("gb", "Game Boy", ".gb"), system("gbc", "Game Boy Color", ".gbc")],
            deterministic: true,
            accuracy: Some(v1::Accuracy { measured_with: "defaults".into(), suite: "Mercury (C#), the reference it is graded against".into(), notes: String::new() }),
            ..Info::default()
        }
    }

    fn settings_schema() -> Vec<Setting> {
        vec![Setting {
            key: MODEL_KEY.into(),
            label: "Model".into(),
            help: "Which console a game runs on. Auto follows the cartridge: a Game Boy Color game on a Game Boy Color, the rest on a Game Boy. Game Boy runs every game on a Game Boy; Game Boy Color runs every game on a Game Boy Color. A save state resumes on the console it was made on.".into(),
            kind: SettingKind::Choice(MODELS.iter().map(|&(v, _)| Choice { value: v.into(), label: v.into(), help: None }).collect()),
            default: "Auto".into(),
            scope: Scope::Create,
            category: Some("System".into()),
            effect: Effect::None,
            advanced: false,
            hidden: false,
            restart: true,
        }]
    }

    fn status_text(code: i32) -> Option<String> {
        let words = match code {
            STATUS_TOO_SHORT => "an image shorter than the 336-byte header",
            STATUS_UNSUPPORTED_BOARD => "a cartridge type no board implements",
            STATUS_UNKNOWN_MODEL => "a model that is not Auto, Game Boy or Game Boy Color",
            STATUS_ILLEGAL_OPCODE => "an opcode no SM83 has",
            _ => return None,
        };
        Some(words.to_owned())
    }

    /// `MercuryCore.LoadRom` on the console the Model setting names, then the battery save written into cartridge RAM, clipped to it.
    fn create(r: &Create<'_>) -> Result<Self, i32> {
        let chosen = r.settings.get(MODEL_KEY).unwrap_or("Auto");
        let model = MODELS.iter().find(|m| m.0 == chosen).map(|m| m.1).ok_or(STATUS_UNKNOWN_MODEL)?;
        let mut m = Machine::load_rom(r.image.to_vec(), model).map_err(|e| match e {
            RomError::TooShort(_) => STATUS_TOO_SHORT,
            RomError::UnsupportedType(kind) => {
                v1::detail(&format!("cartridge type ${kind:02X} is not implemented"));
                STATUS_UNSUPPORTED_BOARD
            }
        })?;
        for file in &r.files {
            if file.which != 0 {
                v1::detail(&format!("file {} is not one MercuryRT takes", file.which));
                return Err(status::BAD_FILE);
            }
            let length = m.space_size(BATTERY_SPACE).min(file.data.len());
            for (i, &b) in file.data[..length].iter().enumerate() {
                m.write_space(BATTERY_SPACE, i as i32, b);
            }
        }
        Ok(m)
    }

    fn machine_info(&self) -> MachineInfo {
        let battery = *self.bus.cart.has_battery && !self.bus.cart.ram.is_empty();
        MachineInfo {
            system: if self.cgb_hardware() { "gbc" } else { "gb" }.into(),
            region: None,
            frame_rate: (CPU_CLOCK_HZ as u64, CYCLES_PER_FRAME as u64),
            video: Video { base_width: SCREEN_WIDTH as u32, base_height: SCREEN_HEIGHT as u32, max_width: SCREEN_WIDTH as u32, max_height: SCREEN_HEIGHT as u32, aspect: (10, 9), formats: Vec::new() },
            audio: v1::Audio { rate: 44100, channels: ["Pulse 1", "Pulse 2", "Wave", "Noise"].map(String::from).to_vec() },
            ports: vec![Port { port: 0, controller: Some("gb.pad".into()) }],
            spaces: SPACES
                .iter()
                .enumerate()
                .map(|(id, &name)| Space { cheats: id == 3, side_effects: id == 6, reports_stores: (1..=5).contains(&id), ..Space::new(id as u32, name) })
                .collect(),
            processors: vec![Processor { id: 0, name: "CPU".into(), pc_bits: 16, registers: Vec::new(), code_space: None }],
            battery: if battery { vec![Battery { which: 0, suffix: ".srm".into() }] } else { Vec::new() },
            state: StateFormat { format: "MERC".into(), version: STATE_VERSION as i64, loads_from: (OLDEST_READABLE_VERSION..=STATE_VERSION).map(i64::from).collect() },
            phases: Vec::new(),
            patches: Some((0, 0x7FFF)),
            skip_rendering_state_neutral: false,
        }
    }

    fn advance(&mut self, detail: &mut u64) -> Result<(), i32> {
        self.run_frame().map_err(|e| {
            *detail = ((e.opcode as u64) << 16) | e.pc as u64;
            v1::detail(&format!("opcode ${:02X} at ${:04X} is no SM83's", e.opcode, e.pc));
            STATUS_ILLEGAL_OPCODE
        })
    }

    fn set_options(&mut self, flags: u32) {
        *self.bus.ppu.skip_rendering = flags & 1 != 0;
    }

    fn frame_count(&self) -> i64 {
        self.total_frames
    }

    fn frame_info(&self) -> FrameInfo {
        FrameInfo { aspect_num: 10, aspect_den: 9, ..FrameInfo::rgba(SCREEN_WIDTH, SCREEN_HEIGHT) }
    }

    fn frame(&self) -> &[u8] {
        &self.bus.ppu.frame_rgba
    }

    fn audio_rate(&self) -> i32 {
        44100
    }

    fn audio_buffered(&self) -> usize {
        self.bus.apu.mixer.samples.len()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        self.bus.apu.drain(out, max_frames)
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.bus.apu.mixer.samples.set_limit(samples);
    }

    fn set_mutes(&mut self, mask: u32) -> Result<(), i32> {
        for (i, muted) in self.bus.apu.mixer.channel_muted.iter_mut().enumerate() {
            *muted = mask & (1 << i) != 0;
        }
        Ok(())
    }

    /// Port 0 alone; another port is ignored, as `MercuryCore.SetButton` ignores it.
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        <Machine as emusen_native::abi::NativeCore>::set_buttons(self, port, mask, changed)
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        if space as usize >= SPACES.len() {
            return Err(status::NO_SUCH_SPACE);
        }
        Ok(Machine::space_size(self, space) as i64)
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        for (i, b) in out.iter_mut().enumerate() {
            *b = self.read_space(space, (address as i32).wrapping_add(i as i32));
        }
        Ok(())
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        if space as usize >= SPACES.len() {
            return Err(status::NO_SUCH_SPACE);
        }
        for (i, &b) in data.iter().enumerate() {
            self.write_space(space, (address as i32).wrapping_add(i as i32), b);
        }
        Ok(())
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        if which != 0 {
            return Err(status::BAD_FILE);
        }
        let cart = &self.bus.cart;
        Ok((if *cart.has_battery { &cart.ram[..] } else { &[] }, 0))
    }

    fn set_rom_patches(&mut self, triples: &[u32]) -> Result<(), i32> {
        *self.bus.rom_patches = build_rom_patches(triples);
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut Hooks> {
        Some(&mut self.hooks)
    }

    fn debug_run_frame(&mut self, flags: u32, detail: &mut u64) -> Result<u32, i32> {
        self.run_frame_debug(flags).map_err(|e| {
            *detail = ((e.opcode as u64) << 16) | e.pc as u64;
            STATUS_ILLEGAL_OPCODE
        })
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        (processor == 0).then_some(self.cpu.pc as u64)
    }
}

emusen_native::core_exports!(Machine; mutes, rom_patches, debug, debug_stack);
