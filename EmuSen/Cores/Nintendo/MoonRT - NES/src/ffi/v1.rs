//! MoonRT on the core ABI v1, beside its pre-stable exports until the v1 adapter is its only loader. The console's
//! facts its C# shim held are its descriptors here. See EmuSen_CoreAPI.md §13.1, §22.

use emusen_native::core::{
    self as v1, Battery, Button, Choice, Control, Controller, Create, Effect, Firmware, FrameInfo, Info, Instruction, MachineInfo, Port, Processor, RawEncoding, RawFrame,
    Region, Scope, Setting, SettingKind, Space, StateFormat, System, Video, caps, status,
};

use super::{BATTERY_SPACE, STATUS_NOT_INES, STATUS_NULL, STATUS_PRG_TRUNCATED, STATUS_UNSUPPORTED_BOARD, build_rom_patches, fault_status, rom_patch};
use crate::machine::{LoadError, Machine, OLDEST_READABLE_VERSION, SAMPLE_RATE, STATE_VERSION};
use crate::memory::cartridge::{Cartridge, RomError};
use crate::model::Model;
use crate::ppu::render::NES_PALETTE;
use crate::ppu::{FRAME_BYTES, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::state::STATUS_OTHER_MODEL;

/// The setting that chooses the console, read at create: the header's, or one console for every game (Moon_PAL.md §3).
pub const REGION_KEY: &str = "region";
pub const REGION_AUTO: &str = "auto";
pub const REGIONS: [(&str, &str, Model); 3] = [("nes.ntsc", "NTSC NES", Model::Ntsc), ("nes.pal", "PAL NES", Model::Pal), ("nes.dendy", "Dendy", Model::Dendy)];

fn region_setting() -> Setting {
    let mut choices = vec![Choice { value: REGION_AUTO.into(), label: "From the cartridge".into(), help: None }];
    choices.extend(REGIONS.iter().map(|&(value, label, _)| Choice { value: value.into(), label: label.into(), help: None }));
    Setting {
        key: REGION_KEY.into(),
        label: "Region".into(),
        help: "The console a game runs on, which sets its speed and the picture's rate: the NTSC NES at 60 frames a second, the PAL NES at 50, and the Dendy, a famiclone, at 50 with the NTSC NES's sound. From the cartridge, a game runs as the console its NES 2.0 header names, and an older iNES header, which has no reliable mark, as the NTSC NES. A game made for one console runs too fast, too slow or wrongly on another. Takes effect when a game is next loaded.".into(),
        kind: SettingKind::Choice(choices),
        default: REGION_AUTO.into(),
        scope: Scope::Create,
        category: Some("Console".into()),
        effect: Effect::None,
        advanced: false,
        hidden: false,
        restart: true,
    }
}

/// The console the setting asks for, the header's where it says `auto` or nothing.
pub fn chosen_model(setting: Option<&str>, image: &[u8]) -> Model {
    REGIONS.iter().find(|r| Some(r.0) == setting).map_or_else(|| Cartridge::header_model(image), |r| r.2)
}

/// `MoonMachine.SpaceNames`, numbered as the ABI numbers them.
const SPACES: [&str; 8] = ["RAM", "PRGROM", "PRGRAM", "CHR", "CIRAM", "OAM", "PALETTE", "CPUBUS"];

/// The CPU's bus, and the two register windows the core logs stores under: `$2000`-`$2007` and `$4000`-`$401F` read and
/// written through it, so with the bus's side effects (EmuSen_Backend.md §19.3.3, D15).
const CPU_BUS: u32 = 7;
const PPU_REGISTERS: (u32, &str, i32, i32) = (crate::memory::bus::SPACE_PPU_REGISTERS, "PPUREG", 0x2000, 8);
const APU_REGISTERS: (u32, &str, i32, i32) = (crate::memory::bus::SPACE_APU_REGISTERS, "APUREG", 0x4000, 0x20);

/// The 2A03's registers as `debug_registers` gives them: the six, P's six flags broken out, and the cycle count.
const REGISTERS: [(&str, u32); 13] =
    [("A", 8), ("X", 8), ("Y", 8), ("S", 8), ("PC", 16), ("P", 8), ("N", 1), ("V", 1), ("D", 1), ("I", 1), ("Z", 1), ("C", 1), ("Cycles", 64)];

/// A register window's space, its first CPU address and its length; `None` for any other space.
fn window(space: u32) -> Option<(i32, i32)> {
    [PPU_REGISTERS, APU_REGISTERS].iter().find(|w| w.0 == space).map(|w| (w.2, w.3))
}

/// A byte of a space as a host reads it: a register window's through the CPU's bus, past its end 0.
fn space_byte(m: &mut Machine, space: u32, address: i32) -> u8 {
    match window(space) {
        Some((base, length)) => {
            if (0..length).contains(&address) { m.read_space(CPU_BUS, base + address) } else { 0 }
        }
        None => m.read_space(space, address),
    }
}

/// The registered name of the PPU's picture before the palette (EmuSen_CoreAPI.md §6.30).
pub const RAW_NES_PPU: &str = "nes.ppu";

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
    const CAPABILITIES: u64 =
        caps::RESET | caps::MUTES | caps::ROM_PATCHES | caps::DEBUG | caps::DEBUG_STACK | caps::RAW_PICTURE | caps::DEBUG_REGISTERS | caps::DEBUG_DISASSEMBLE;

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
                regions: vec![Region::Ntsc, Region::Pal],
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

    fn settings_schema() -> Vec<Setting> {
        vec![region_setting()]
    }

    fn status_text(code: i32) -> Option<String> {
        let words = match code {
            STATUS_OTHER_MODEL => "a state saved on another console, the NTSC NES, the PAL NES or the Dendy, than the one this game runs on",
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

    /// `MoonCore.LoadRom` from an image as the console chosen, then the battery save written into PRG RAM, clipped to it, for a
    /// cartridge whose header claims a battery; another's file 0 is left unread (EmuSen_Backend.md §19.6, C1).
    fn create(r: &Create<'_>) -> Result<Self, i32> {
        let model = chosen_model(r.settings.get(REGION_KEY), r.image);
        let mut m = Machine::load_rom_as(r.image, model).map_err(|e| match e {
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
            if !*m.bus.board.cart.has_battery {
                continue;
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
            region: Some(if *self.model == Model::Ntsc { Region::Ntsc } else { Region::Pal }),
            frame_rate: self.model.frame_rate(),
            video: Video { base_width: SCREEN_WIDTH as u32, base_height: SCREEN_HEIGHT as u32, max_width: SCREEN_WIDTH as u32, max_height: SCREEN_HEIGHT as u32, aspect: (4, 3), formats: Vec::new() },
            audio: v1::Audio { rate: SAMPLE_RATE, channels: ["Pulse 1", "Pulse 2", "Triangle", "Noise", "DMC"].map(String::from).to_vec() },
            ports: vec![Port { port: 0, controller: Some("nes.pad".into()) }, Port { port: 1, controller: Some("nes.pad".into()) }],
            spaces: SPACES
                .iter()
                .enumerate()
                .map(|(id, &name)| Space { cheats: id == 0, read_only: id == 1, side_effects: id == 7, reports_stores: matches!(id, 0 | 2 | 7), ..Space::new(id as u32, name) })
                .chain([PPU_REGISTERS, APU_REGISTERS].iter().map(|w| Space { side_effects: true, reports_stores: true, ..Space::new(w.0, w.1) }))
                .collect(),
            processors: vec![Processor {
                id: 0,
                name: "CPU".into(),
                pc_bits: 16,
                registers: REGISTERS.iter().map(|&(name, bits)| (name.to_owned(), bits)).collect(),
                code_space: None,
            }],
            battery: if battery { vec![Battery::game(0, ".srm")] } else { Vec::new() },
            state: StateFormat { format: "MOON".into(), version: self.state_version() as i64, loads_from: (OLDEST_READABLE_VERSION..=STATE_VERSION).map(i64::from).collect() },
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
        self.bus.ppu.begin_raw(!*self.frame_open);
        let ran = self.run_frame().map_err(fault_status);
        self.bus.ppu.end_raw();
        ran
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

    fn raw_encodings(&self) -> Vec<RawEncoding> {
        let palette = (0..512).map(|v| (v & 0x3F) * 3).map(|i| (NES_PALETTE[i] as u32) << 16 | (NES_PALETTE[i + 1] as u32) << 8 | NES_PALETTE[i + 2] as u32).collect();
        vec![RawEncoding { encoding: RAW_NES_PPU.into(), element: 2, max_width: SCREEN_WIDTH as u32, max_height: SCREEN_HEIGHT as u32, palette }]
    }

    fn raw_select(&mut self, mask: u64) -> Result<(), i32> {
        self.bus.ppu.select_raw(mask & 1 != 0);
        Ok(())
    }

    fn raw_frame(&self, raw: u32) -> Result<Option<(RawFrame, &[u8])>, i32> {
        if raw != 0 {
            return Err(status::NOT_SUPPORTED);
        }
        Ok(self.bus.ppu.raw_picture().map(|values| (RawFrame::packed(2, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32), values)))
    }

    fn audio_rate(&self) -> i32 {
        SAMPLE_RATE
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
        match window(space) {
            Some((_, length)) => Ok(length as i64),
            None if (space as usize) < SPACES.len() => Ok(Machine::space_size(self, space) as i64),
            None => Err(status::NO_SUCH_SPACE),
        }
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        for (i, b) in out.iter_mut().enumerate() {
            *b = space_byte(self, space, (address as i32).wrapping_add(i as i32));
        }
        Ok(())
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        let window = window(space);
        if window.is_none() && space as usize >= SPACES.len() {
            return Err(status::NO_SUCH_SPACE);
        }
        if space == 1 {
            return Err(status::READ_ONLY);
        }
        for (i, &b) in data.iter().enumerate() {
            let at = (address as i32).wrapping_add(i as i32);
            match window {
                Some((base, length)) if (0..length).contains(&at) => self.write_space(CPU_BUS, base + at, b),
                Some(_) => {}
                None => self.write_space(space, at, b),
            }
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
        self.bus.ppu.begin_raw(!*self.frame_open);
        let stopped = self.run_frame_debug(flags).map_err(fault_status);
        if stopped == Ok(emusen_native::debug::stop::FRAME) {
            self.bus.ppu.end_raw();
        }
        stopped
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        (processor == 0).then_some(self.cpu.pc as u64)
    }

    /// The PPU, the APU, its channels, the board and the cartridge, by the ids from 256 (EmuSen_CoreAPI.md §6.33).
    fn units(&self) -> Vec<v1::Unit> {
        super::units::units(self)
    }

    /// `REGISTERS`' values, read from the CPU itself: what the shim's target read out of a saved state (EmuSen_DianaSh.md Q28).
    fn debug_registers(&self, processor: u32) -> Result<Vec<i64>, i32> {
        if processor != 0 {
            return super::units::registers(self, processor).ok_or(status::NOT_SUPPORTED);
        }
        let c = &self.cpu;
        let mut values = vec![c.a as i64, c.x as i64, c.y as i64, c.s as i64, c.pc as i64, c.p as i64];
        values.extend([7, 6, 3, 2, 1, 0].map(|bit| ((c.p >> bit) & 1) as i64));
        values.push(c.cycles);
        Ok(values)
    }

    /// The shared 6502 table over the bytes a host would read, each instruction's opcode read and then its bytes, as the
    /// shim's target read them (EmuSen_Backend.md §19.3.3, D10).
    fn debug_disassemble(&mut self, processor: u32, space: u32, address: u32, count: u32) -> Result<Vec<Instruction>, i32> {
        if processor != 0 {
            return Err(status::NOT_SUPPORTED);
        }
        <Machine as v1::Core>::space_size(self, space)?;
        let mut list = Vec::new();
        let mut pc = address as i32;
        for _ in 0..count.min(4096) {
            let opcode = space_byte(self, space, pc);
            let length = emusen_6502::length_of(opcode);
            let bytes: Vec<u8> = (0..length as i32).map(|b| space_byte(self, space, pc.wrapping_add(b))).collect();
            let (low, high) = (bytes.get(1).copied().unwrap_or(0), bytes.get(2).copied().unwrap_or(0));
            let operand = match length {
                2 => low as i32,
                3 => low as i32 | (high as i32) << 8,
                _ => -1,
            };
            let reference = if operand < 0 { None } else { emusen_6502::classify(bytes[0], operand) };
            list.push(Instruction {
                address: pc as u32,
                bytes,
                mnemonic: emusen_6502::mnemonic_of(opcode).to_owned(),
                operands: emusen_6502::format_operand(opcode, pc, low, high),
                reference: reference.map(|(kind, target)| {
                    let kind = match kind {
                        emusen_6502::Reference::Call => v1::Reference::Call,
                        emusen_6502::Reference::Read => v1::Reference::Read,
                        emusen_6502::Reference::Write => v1::Reference::Write,
                    };
                    (kind, target as u32)
                }),
            });
            pc = pc.wrapping_add(length as i32);
        }
        crate::take_fault();
        Ok(list)
    }

    fn end_call() {
        crate::take_fault();
    }
}

emusen_native::core_exports!(Machine; reset, mutes, rom_patches, debug, debug_stack, raw_picture, debug_registers, debug_disassemble);

/// `moonrt_rom_patch` for a machine of the core ABI v1: the byte a CPU read of `address` returns for `original` under
/// the current patches, or -1 for none, the exhaustive patch test's view through the v1 loader (EmuSen_CoreAPI.md §26).
///
/// # Safety
/// `machine` must be null or a live handle `emusen_core_create` returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moonrt_core_rom_patch(machine: *const v1::sys::Machine, address: u32, original: u32) -> i32 {
    unsafe { v1::exports::core_of::<Machine>(machine) }.map_or(STATUS_NULL, |m| rom_patch(m, address, original))
}

/// `moonrt_step` for a machine of the core ABI v1: one instruction with the frame loop's DMA charge, NMI edge and stolen
/// cycles, the cycles or a fault status; a test extension, as the step goldens drive it (EmuSen_Backend.md §19.3.5, M15).
///
/// # Safety
/// `machine` must be null or a live handle `emusen_core_create` returned, with no other call on it in flight.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moonrt_core_step(machine: *mut v1::sys::Machine) -> i32 {
    unsafe { v1::exports::core_of_mut::<Machine>(machine) }.map_or(STATUS_NULL, |m| {
        crate::take_fault();
        let cycles = m.step();
        crate::take_fault().map_or(cycles, fault_status)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppu::RAW_BYTES;
    use emusen_native::abi::Settings;
    use emusen_native::core::{Core, sys};

    /// An NROM image of one PRG bank: `reset` at $C000 and `nmi` at $C100, every other byte a NOP, CHR counting up.
    fn image(reset: &[u8], nmi: &[u8]) -> Vec<u8> {
        let mut image = vec![0x4E, 0x45, 0x53, 0x1A, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut prg = vec![0xEAu8; 0x4000];
        prg[..reset.len()].copy_from_slice(reset);
        prg[0x100..0x100 + nmi.len()].copy_from_slice(nmi);
        prg[0x3FFA..].copy_from_slice(&[0x00, 0xC1, 0x00, 0xC0, 0x00, 0xC0]);
        image.extend(prg);
        image.extend((0..0x2000).map(|i| i as u8));
        image
    }

    fn machine(image: &[u8]) -> Machine {
        let r = Create { image, settings: Settings::default(), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        <Machine as Core>::create(&r).unwrap()
    }

    /// Two vblanks waited, palette entry 0 set to `colour`, and `mask` and `control` written.
    fn boot(colour: u8, mask: u8, control: u8) -> Vec<u8> {
        let mut code = vec![
            0x78, 0xD8, 0xA2, 0xFF, 0x9A, // SEI, CLD, LDX #$FF, TXS
            0x2C, 0x02, 0x20, 0x10, 0xFB, // BIT $2002, BPL -5
            0x2C, 0x02, 0x20, 0x10, 0xFB, // BIT $2002, BPL -5
            0xA9, 0x3F, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20, // $3F00 into $2006
            0xA9, colour, 0x8D, 0x07, 0x20, // LDA #colour, STA $2007
            0xA9, 0x00, 0x8D, 0x06, 0x20, 0x8D, 0x06, 0x20, 0x8D, 0x05, 0x20, 0x8D, 0x05, 0x20, // $2006 and $2005 cleared
            0xA9, mask, 0x8D, 0x01, 0x20, // LDA #mask, STA $2001
            0xA9, control, 0x8D, 0x00, 0x20, // LDA #control, STA $2000
        ];
        let here = 0xC000 + code.len() as u16;
        code.extend([0x4C, here as u8, (here >> 8) as u8]); // JMP to itself
        code
    }

    fn raw(m: &Machine) -> Vec<u16> {
        let (f, bytes) = m.raw_frame(0).unwrap().expect("the frame's raw picture");
        assert_eq!((f.element, f.width, f.height, f.stride, bytes.len()), (2, 256, 240, 512, RAW_BYTES));
        bytes.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect()
    }

    /// Every value through the palette machine info declares is the RGB frame's pixel.
    fn drawn_through_the_palette(m: &Machine) {
        let palette = &m.raw_encodings()[0].palette;
        for (i, (&v, p)) in raw(m).iter().zip(m.frame().chunks_exact(4)).enumerate() {
            assert_eq!(palette[v as usize], (p[0] as u32) << 16 | (p[1] as u32) << 8 | p[2] as u32, "pixel {i}: value {v:#x}");
        }
    }

    // §6.30's nes.ppu: an emphasis write in the middle of the frame, timed from the NMI, is in the raw picture from its
    // line on, every line one value throughout; the RGB frame, which MoonRT's palette draws without emphasis, cannot
    // show it, and is still the raw picture through the palette.
    #[test]
    fn an_emphasis_write_in_the_middle_of_the_frame_is_in_the_raw_picture_from_its_line() {
        let nmi = [
            0x48, 0x8A, 0x48, 0x98, 0x48, // PHA, TXA, PHA, TYA, PHA
            0xA9, 0x00, 0x8D, 0x01, 0x20, // emphasis off at the frame's top
            0xA0, 0x0B, 0xA2, 0xFF, 0xCA, 0xD0, 0xFD, 0x88, 0xD0, 0xF8, // eleven times 255 DEX: about 124 lines
            0xA9, 0xE0, 0x8D, 0x01, 0x20, // all three emphasis bits
            0x68, 0xA8, 0x68, 0xAA, 0x68, 0x40, // PLA, TAY, PLA, TAX, PLA, RTI
        ];
        let mut m = machine(&image(&boot(0x16, 0x00, 0x80), &nmi));
        m.raw_select(1).unwrap();
        let mut detail = 0;
        let mut switch = None;
        for frame in 0..8 {
            m.advance(&mut detail).unwrap();
            if frame < 4 {
                continue;
            }
            let values = raw(&m);
            let rows: Vec<u16> = (0..240).map(|y| {
                let row = &values[y * 256..(y + 1) * 256];
                assert!(row.iter().all(|&v| v == row[0]), "frame {frame}, line {y}: one value throughout");
                row[0]
            }).collect();
            assert!(rows.iter().all(|&v| v & 0x3F == 0x16), "frame {frame}: colour $16 everywhere");
            let first = rows.iter().position(|&v| v >> 6 == 7).expect("emphasis switched on in the frame");
            assert!((10..230).contains(&first), "frame {frame}: emphasis from line {first}");
            assert!(rows[..first].iter().all(|&v| v >> 6 == 0) && rows[first..].iter().all(|&v| v >> 6 == 7), "frame {frame}: off above line {first}, on from it");
            assert_eq!(*switch.get_or_insert(first), first, "frame {frame}: the same line every frame");
            drawn_through_the_palette(&m);
        }
        println!("emphasis from line {} in frames 4 to 7", switch.expect("frames 4 to 7 checked"));
    }

    // A picture drawn from tiles, colours and sprites: every pixel of the raw picture through the palette is the frame's,
    // and a machine keeping it runs as one keeping none, state and picture.
    #[test]
    fn a_rendered_picture_is_the_raw_picture_through_the_palette_and_keeping_it_changes_nothing() {
        let mut reset = boot(0x0F, 0x1E, 0x00);
        let palette: Vec<u8> = (0..32).map(|i| (i * 7 + 1) as u8 & 0x3F).collect();
        let mut fill = vec![0xA9, 0x3F, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20];
        for &c in &palette {
            fill.extend([0xA9, c, 0x8D, 0x07, 0x20]);
        }
        fill.extend([0xA9, 0x00, 0x8D, 0x06, 0x20, 0x8D, 0x06, 0x20]);
        reset.splice(15..30, fill);
        let end = reset.len() - 3;
        let jmp = 0xC000 + end as u16;
        reset.splice(end.., [0x4C, jmp as u8, (jmp >> 8) as u8]);
        let img = image(&reset, &[0x40]);
        let (mut kept, mut plain) = (machine(&img), machine(&img));
        kept.raw_select(1).unwrap();
        let mut detail = 0;
        for _ in 0..30 {
            kept.advance(&mut detail).unwrap();
            plain.advance(&mut detail).unwrap();
            drawn_through_the_palette(&kept);
            assert_eq!(kept.frame(), plain.frame());
        }
        assert!(raw(&kept).iter().any(|&v| v & 0x3F != palette[0] as u16), "the tiles draw more than the backdrop");
        let save = |m: &mut Machine| {
            let mut out = vec![0u8; m.state_size()];
            m.save_state(&mut out).unwrap();
            out
        };
        assert_eq!(save(&mut kept), save(&mut plain));
    }

    // The region setting chooses the console at create, `auto` the header's; machine info reports its standard, its rate
    // and its state version, and a state of another console is refused with words of its own.
    #[test]
    fn the_region_setting_chooses_the_console_and_machine_info_reports_it() {
        assert!(Machine::settings_schema().iter().all(|s| s.check().is_ok() && matches!(s.scope, Scope::Create)));
        let made = |img: &[u8], region: &str| {
            let r = Create { image: img, settings: Settings::from_pairs(vec![(REGION_KEY.into(), region.into())]), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
            <Machine as Core>::create(&r).unwrap()
        };
        let ines = image(&boot(0x0F, 0, 0), &[0x40]);
        let mut pal_header = ines.clone();
        (pal_header[7], pal_header[12]) = (0x08, 0x01);
        for (img, region, model) in [
            (&ines, "auto", Model::Ntsc),
            (&ines, "nes.pal", Model::Pal),
            (&ines, "nes.dendy", Model::Dendy),
            (&pal_header, "auto", Model::Pal),
            (&pal_header, "nes.ntsc", Model::Ntsc),
        ] {
            let m = made(img, region);
            let info = m.machine_info();
            assert_eq!(*m.model, model, "{region}");
            assert_eq!(info.region, Some(if model == Model::Ntsc { Region::Ntsc } else { Region::Pal }), "{region}");
            assert_eq!(info.frame_rate, model.frame_rate(), "{region}");
            assert_eq!(info.state.version, if model == Model::Ntsc { 6 } else { 7 }, "{region}");
            assert_eq!(info.state.loads_from, (3..=7).collect::<Vec<i64>>());
        }
        assert_eq!(made(&ines, "auto").machine_info().frame_rate, (21_477_272, 262 * 341 * 4));
        assert!(<Machine as Core>::status_text(STATUS_OTHER_MODEL).is_some());
        assert_eq!(Machine::info().systems[0].regions, vec![Region::Ntsc, Region::Pal]);
    }

    fn with_battery_file(img: &[u8], data: &[u8]) -> Machine {
        let r = Create { image: img, settings: Settings::default(), files: vec![emusen_native::abi::File { which: 0, data }], pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        <Machine as Core>::create(&r).unwrap()
    }

    // C1 (EmuSen_Backend.md §19.6): file 0 reaches PRG RAM only where the header claims a battery, and machine info lists
    // the file only then.
    #[test]
    fn a_battery_file_is_taken_only_by_a_cartridge_with_a_battery() {
        let plain = image(&boot(0x0F, 0, 0), &[0x40]);
        let mut battery = plain.clone();
        battery[6] |= 0x02;
        let saved = vec![0xA5u8; 0x2000];
        let (mut without, mut with) = (with_battery_file(&plain, &saved), with_battery_file(&battery, &saved));
        assert_eq!(without.read_space(BATTERY_SPACE, 0x10), 0);
        assert_eq!(with.read_space(BATTERY_SPACE, 0x10), 0xA5);
        assert!(without.machine_info().battery.is_empty());
        assert_eq!(with.machine_info().battery.len(), 1);
    }

    // Q28 (EmuSen_DianaSh.md): the thirteen registers machine info names, read from the CPU, the flags P's bits.
    #[test]
    fn the_registers_are_the_cpus_own_in_machine_infos_order() {
        let mut m = machine(&image(&boot(0x0F, 0, 0), &[0x40]));
        let mut detail = 0;
        for _ in 0..3 {
            m.advance(&mut detail).unwrap();
        }
        let names: Vec<String> = m.machine_info().processors[0].registers.iter().map(|r| r.0.clone()).collect();
        assert_eq!(names, REGISTERS.iter().map(|r| r.0.to_owned()).collect::<Vec<_>>());
        let v = m.debug_registers(0).unwrap();
        assert_eq!(v.len(), 13);
        assert_eq!((v[0], v[1], v[2], v[3], v[4], v[5]), (m.cpu.a as i64, m.cpu.x as i64, m.cpu.y as i64, m.cpu.s as i64, m.cpu.pc as i64, m.cpu.p as i64));
        assert_eq!(v[6..12].iter().zip([7, 6, 3, 2, 1, 0]).map(|(f, bit)| *f << bit).sum::<i64>(), (m.cpu.p & 0xCF) as i64);
        assert_eq!(v[12], m.cpu.cycles);
        assert_eq!(m.debug_registers(1), Err(status::NOT_SUPPORTED));
    }

    // The boot program as the shared table writes it, from the reset vector through the CPU's bus, branches resolved.
    #[test]
    fn the_disassembly_is_the_shared_tables_over_the_bus() {
        let mut m = machine(&image(&boot(0x0F, 0, 0), &[0x40]));
        let list = m.debug_disassemble(0, CPU_BUS, 0xC000, 7).unwrap();
        let text: Vec<String> = list.iter().map(|i| format!("{:04X} {} {}", i.address, i.mnemonic, i.operands)).collect();
        assert_eq!(text, ["C000 SEI ", "C001 CLD ", "C002 LDX #$FF", "C004 TXS ", "C005 BIT $2002", "C008 BPL $C005", "C00A BIT $2002"]);
        assert_eq!(list[4].reference, Some((v1::Reference::Read, 0x2002)));
        assert_eq!(list[4].bytes, vec![0x2C, 0x02, 0x20]);
        assert_eq!(m.debug_disassemble(1, CPU_BUS, 0xC000, 1).err(), Some(status::NOT_SUPPORTED));
        assert_eq!(m.debug_disassemble(0, 10, 0, 1).err(), Some(status::NO_SUCH_SPACE));
    }

    // D1 and D15 (EmuSen_Backend.md §19.3.3): PRG ROM read-only, and the two register windows the core logs stores under,
    // read and written through the bus, nothing past their ends.
    #[test]
    fn prg_rom_is_read_only_and_the_register_windows_go_through_the_bus() {
        let mut m = machine(&image(&boot(0x0F, 0, 0), &[0x40]));
        let info = m.machine_info();
        let space = |id: u32| info.spaces.iter().find(|s| s.id == id).unwrap().clone();
        assert!(space(1).read_only && !space(0).read_only);
        assert_eq!(m.space_write(1, 0, &[0x55]), Err(status::READ_ONLY));
        for (id, name, length) in [(8, "PPUREG", 8), (9, "APUREG", 0x20)] {
            let s = space(id);
            assert_eq!((s.name.as_str(), s.side_effects, s.reports_stores, s.read_only, s.view), (name, true, true, false, false));
            assert_eq!(<Machine as Core>::space_size(&m, id), Ok(length));
        }
        m.space_write(8, 6, &[0x3F]).unwrap();
        m.space_write(8, 6, &[0x00]).unwrap();
        m.space_write(8, 7, &[0x21]).unwrap();
        assert_eq!(m.bus.ppu.palette_ram[0], 0x21);
        m.space_write(9, 0x15, &[0x01]).unwrap();
        let mut b = [0xFFu8; 2];
        m.space_read(9, 0x15, &mut b[..1]).unwrap();
        m.space_read(9, 0x20, &mut b[1..]).unwrap();
        assert_eq!(b[1], 0);
        assert_eq!(<Machine as Core>::space_size(&m, 10), Err(status::NO_SUCH_SPACE));
    }

    // §6.33's units: nine, ids from 256, each answered with as many values as machine info names, the board's after
    // its mapper number and an MMC3's eighteen after that; another id refused.
    #[test]
    fn the_units_are_answered_as_machine_info_names_them() {
        let nrom = machine(&image(&boot(0x0F, 0, 0), &[0x40]));
        let units = nrom.units();
        assert_eq!(units.iter().map(|u| (u.id, u.kind.as_str())).collect::<Vec<_>>(), [
            (256, "video"), (257, "audio"), (258, "channel"), (259, "channel"), (260, "channel"), (261, "channel"), (262, "channel"), (263, "cartridge"), (264, "other"),
        ]);
        for u in &units {
            assert_eq!(nrom.debug_registers(u.id).unwrap().len(), u.registers.len(), "{}", u.name);
        }
        assert_eq!(units[1].registers.len(), 22);
        assert_eq!(nrom.debug_registers(263).unwrap(), vec![0]);
        assert_eq!(nrom.debug_registers(264).unwrap(), vec![0, 0, 0]);
        assert_eq!(nrom.debug_registers(265), Err(status::NOT_SUPPORTED));
        let mut mmc3 = image(&boot(0x0F, 0, 0), &[0x40]);
        (mmc3[4], mmc3[6]) = (2, 0x40);
        mmc3.splice(16..16, mmc3[16..16 + 0x4000].to_vec());
        let m = machine(&mmc3);
        let board = m.units().into_iter().find(|u| u.id == 263).unwrap();
        assert_eq!(board.registers.len(), 19);
        assert_eq!((board.registers[0].0.as_str(), board.registers[18].0.as_str()), ("Mapper", "WramEnabled"));
        assert_eq!(m.debug_registers(263).unwrap()[0], 4);
    }
}
