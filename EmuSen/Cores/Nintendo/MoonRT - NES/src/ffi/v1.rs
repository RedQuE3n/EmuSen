//! MoonRT on the core ABI v1, beside its pre-stable exports until the v1 adapter is its only loader. The console's
//! facts its C# shim held are its descriptors here. See EmuSen_CoreAPI.md §13.1, §22.

use emusen_native::core::{
    self as v1, Battery, Button, Control, Controller, Create, Firmware, FrameInfo, Info, MachineInfo, Port, Processor, Region, Space, StateFormat, System,
    Video, caps, status,
};

use super::{BATTERY_SPACE, STATUS_NOT_INES, STATUS_NULL, STATUS_PRG_TRUNCATED, STATUS_UNSUPPORTED_BOARD, build_rom_patches, fault_status, rom_patch};
use crate::machine::{LoadError, Machine, OLDEST_READABLE_VERSION, STATE_VERSION};
use crate::memory::cartridge::RomError;
use crate::ppu::{FRAME_BYTES, SCREEN_HEIGHT, SCREEN_WIDTH, TOTAL_SCANLINES};

/// `MoonCore.MasterClockHz` over the master clocks of a frame, 341 dots of 4 a scanline.
const MASTER_CLOCK_HZ: u64 = 21_477_272;
const MASTER_CLOCKS_PER_SCANLINE: u64 = 341 * 4;

/// `MoonMachine.SpaceNames`, numbered as the ABI numbers them.
const SPACES: [&str; 8] = ["RAM", "PRGROM", "PRGRAM", "CHR", "CIRAM", "OAM", "PALETTE", "CPUBUS"];

/// The pad's bits, `NesButton`'s numbering, in `MoonCore.PadButtons`' order, the order a bindings screen shows them.
const PAD: [(u32, Control, &str); 8] = [
    (4, Control::Up, "Up"),
    (5, Control::Down, "Down"),
    (6, Control::Left, "Left"),
    (7, Control::Right, "Right"),
    (2, Control::Select, "Select"),
    (3, Control::Start, "Start"),
    (1, Control::B, "B"),
    (0, Control::A, "A"),
];

impl v1::Core for Machine {
    const CAPABILITIES: u64 = caps::RESET | caps::MUTES | caps::ROM_PATCHES | caps::DEBUG | caps::DEBUG_STACK;

    fn info() -> Info {
        Info {
            id: "moonrt".into(),
            name: "MoonRT".into(),
            display_name: Some("MoonRT (Rust)".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("A Rust port of Moon, the project's NES core, exact against it in state, picture and sound.".into()),
            systems: vec![System {
                id: "nes".into(),
                name: "Nintendo Entertainment System".into(),
                extensions: vec![".nes".into()],
                regions: vec![Region::Ntsc],
                controllers: vec![Controller {
                    id: "nes.pad".into(),
                    label: "Controller".into(),
                    ports: vec![0, 1],
                    buttons: PAD.iter().map(|&(bit, control, label)| Button { bit, control: Some(control), label: label.into() }).collect(),
                    axes: Vec::new(),
                }],
                firmware: Vec::<Firmware>::new(),
                development: false,
            }],
            deterministic: true,
            accuracy: Some(v1::Accuracy { measured_with: "defaults".into(), suite: "Moon (C#), the reference it is graded against".into(), notes: String::new() }),
            ..Info::default()
        }
    }

    fn status_text(code: i32) -> Option<String> {
        let words = match code {
            STATUS_NOT_INES => "not an iNES image",
            STATUS_UNSUPPORTED_BOARD => "a mapper no board implements",
            STATUS_PRG_TRUNCATED => "a header claiming more PRG than the file holds",
            c if c == fault_status(crate::Fault::IndexOutOfRange) => "an index outside an array",
            c if c == fault_status(crate::Fault::DivideByZero) => "a division by zero",
            c if c == fault_status(crate::Fault::ArgumentOutOfRange) => "a clock running backwards",
            _ => return None,
        };
        Some(words.to_owned())
    }

    /// `MoonCore.LoadRom` from an image, then the battery save written into PRG RAM, clipped to it, before any frame.
    fn create(r: &Create<'_>) -> Result<Self, i32> {
        let mut m = Machine::load_rom(r.image).map_err(|e| match e {
            LoadError::Rom(RomError::NotInes) => STATUS_NOT_INES,
            LoadError::Rom(RomError::PrgTruncated) => STATUS_PRG_TRUNCATED,
            LoadError::Rom(RomError::UnsupportedMapper(n)) => {
                v1::detail(&format!("mapper {n} is not implemented"));
                STATUS_UNSUPPORTED_BOARD
            }
            LoadError::Fault(f) => fault_status(f),
        })?;
        for file in &r.files {
            if file.which != 0 {
                v1::detail(&format!("file {} is not one MoonRT takes", file.which));
                return Err(status::BAD_FILE);
            }
            let length = m.space_size(BATTERY_SPACE).min(file.data.len());
            for (i, &b) in file.data[..length].iter().enumerate() {
                m.write_space(BATTERY_SPACE, i as i32, b);
            }
        }
        crate::take_fault();
        Ok(m)
    }

    fn machine_info(&self) -> MachineInfo {
        let battery = *self.bus.board.cart.has_battery && !self.bus.board.cart.prg_ram.is_empty();
        MachineInfo {
            system: "nes".into(),
            region: Some(Region::Ntsc),
            frame_rate: (MASTER_CLOCK_HZ, TOTAL_SCANLINES as u64 * MASTER_CLOCKS_PER_SCANLINE),
            video: Video { base_width: SCREEN_WIDTH as u32, base_height: SCREEN_HEIGHT as u32, max_width: SCREEN_WIDTH as u32, max_height: SCREEN_HEIGHT as u32, aspect: (4, 3), formats: Vec::new() },
            audio: v1::Audio { rate: 44100, channels: ["Pulse 1", "Pulse 2", "Triangle", "Noise", "DMC"].map(String::from).to_vec() },
            ports: vec![Port { port: 0, controller: Some("nes.pad".into()) }, Port { port: 1, controller: Some("nes.pad".into()) }],
            spaces: SPACES
                .iter()
                .enumerate()
                .map(|(id, &name)| Space { cheats: id == 0, side_effects: id == 7, reports_stores: matches!(id, 0 | 2 | 7), ..Space::new(id as u32, name) })
                .collect(),
            processors: vec![Processor { id: 0, name: "CPU".into(), pc_bits: 16, registers: Vec::new(), code_space: None }],
            battery: if battery { vec![Battery { which: 0, suffix: ".srm".into() }] } else { Vec::new() },
            state: StateFormat { format: "MOON".into(), version: STATE_VERSION as i64, loads_from: (OLDEST_READABLE_VERSION..=STATE_VERSION).map(i64::from).collect() },
            phases: Vec::new(),
            patches: Some((0x4020, 0xFFFF)),
            skip_rendering_state_neutral: false,
            firmware: Vec::new(),
        }
    }

    fn reset(&mut self) -> Result<(), i32> {
        Machine::reset(self).map_err(fault_status)
    }

    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        self.run_frame().map_err(fault_status)
    }

    fn set_options(&mut self, flags: u32) {
        *self.bus.ppu.skip_rendering = flags & 1 != 0;
    }

    fn frame_count(&self) -> i64 {
        self.total_frames
    }

    fn frame_info(&self) -> FrameInfo {
        FrameInfo { aspect_num: 4, aspect_den: 3, ..FrameInfo::rgba(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32) }
    }

    fn frame(&self) -> &[u8] {
        debug_assert_eq!(self.bus.ppu.frame_rgba.len(), FRAME_BYTES);
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

    /// Port 0 is pad 1 and every other port pad 2, as `MoonCore.SetButton` has it.
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        let pad = if port == 0 { &mut *self.bus.controller1 } else { &mut *self.bus.controller2 };
        pad.state = (pad.state & !(changed as u8)) | (mask as u8 & changed as u8);
        Ok(())
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
        let cart = &self.bus.board.cart;
        Ok((if *cart.has_battery { &cart.prg_ram[..] } else { &[] }, 0))
    }

    fn set_rom_patches(&mut self, triples: &[u32]) -> Result<(), i32> {
        *self.bus.rom_patches = build_rom_patches(triples);
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut emusen_native::debug::Hooks> {
        Some(&mut self.bus.hooks)
    }

    fn debug_run_frame(&mut self, flags: u32, _detail: &mut u64) -> Result<u32, i32> {
        self.run_frame_debug(flags).map_err(fault_status)
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        (processor == 0).then_some(self.cpu.pc as u64)
    }

    fn end_call() {
        crate::take_fault();
    }
}

emusen_native::core_exports!(Machine; reset, mutes, rom_patches, debug, debug_stack);

/// `moonrt_rom_patch` for a machine of the core ABI v1: the byte a CPU read of `address` returns for `original` under
/// the current patches, or -1 for none, the exhaustive patch test's view through the v1 loader (EmuSen_CoreAPI.md §26).
///
/// # Safety
/// `machine` must be null or a live handle `emusen_core_create` returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moonrt_core_rom_patch(machine: *const v1::sys::Machine, address: u32, original: u32) -> i32 {
    unsafe { v1::exports::core_of::<Machine>(machine) }.map_or(STATUS_NULL, |m| rom_patch(m, address, original))
}
