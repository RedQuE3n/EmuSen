//! VenusRT on the stable core ABI, version 1: `Core` over the machine and `core_exports!`, beside the pre-stable
//! `native_exports!` of `ffi` until the host's adapter is VenusRT's only loader. See EmuSen_CoreAPI.md §18.6 and
//! VenusRT_Native.md §31.

use emusen_native::abi::NativeCore;
use emusen_native::core::*;

use crate::ffi::STATUS_IMAGE_TOO_SHORT;
use crate::machine::{DSP_RATE, Machine};
use crate::state::STATE_VERSION;

/// The space names, in id order; `CpuBus` and `WRAM` are the SNES system pack's, which its cheat codecs target.
pub const SPACES: [&str; 8] = ["CpuBus", "IO", "WRAM", "VRAM", "CGRAM", "OAM", "SRAM", "APURAM"];

/// The chips' spaces, ids 8 to 14, listed for a cartridge that has the chip (`ffi::GSURAM` and on).
pub const CHIP_SPACES: [&str; 7] = ["GSURAM", "GSUBUS", "SA1IRAM", "BWRAM", "SA1BUS", "DSPRAM", "DSPPRG"];

/// The pad's bits as `set_buttons` takes them: bit n is `PadButton` n.
const BUTTONS: [(Control, &str); 12] = [
    (Control::B, "B"),
    (Control::Y, "Y"),
    (Control::Select, "Select"),
    (Control::Start, "Start"),
    (Control::Up, "Up"),
    (Control::Down, "Down"),
    (Control::Left, "Left"),
    (Control::Right, "Right"),
    (Control::A, "A"),
    (Control::X, "X"),
    (Control::L, "L"),
    (Control::R, "R"),
];

/// Master clocks a second over a frame's: NTSC 236,250,000/11 Hz over 357,366 (262 lines of 1,364 with the short
/// line every other frame), PAL 21,281,370 Hz over 425,568 (312 lines).
const NTSC_FRAME: (u64, u64) = (236_250_000, 11 * 357_366);
const PAL_FRAME: (u64, u64) = (21_281_370, 425_568);

/// A NEC DSP cartridge's firmware as file 2, whole or as its program and data in files 2 and 3; `stem` and `size` as
/// `Cartridge::nec_firmware` names them, or the open entry core info lists for any of them. Never required: without
/// it VenusRT runs its replacement, or the game without its chip (VenusRT_DspHle.md §7.2).
fn dsp(named: Option<(&str, u64)>) -> Firmware {
    let (name, size, parts) = match named {
        Some((stem, size)) => (format!("{stem}.rom"), size, vec![vec![format!("{stem}.rom")], vec![format!("{stem}.program.rom"), format!("{stem}.data.rom")]]),
        None => ("dsp.rom".to_owned(), 0, Vec::new()),
    };
    let label = "The cartridge's NEC DSP program and data: DSP-1 to DSP-4 (8,192 bytes) or ST010/ST011 (53,248)".into();
    Firmware { which: 2, name, label, size, required: false, parts, replacement: Some(replacement(named.map(|(stem, _)| stem))) }
}

/// The SPC700's boot program as file 1, never required: without it VenusRT runs its own (D-38, VenusRT_Native.md §35).
fn boot() -> Firmware {
    Firmware {
        which: 1,
        name: "spc700.rom".into(),
        label: "The SPC700's boot program (64 bytes)".into(),
        size: 64,
        required: false,
        parts: vec![vec!["spc700.rom".into()]],
        replacement: Some(Replacement::Accuracy {
            cost: "Without the image, VenusRT's own boot program runs: a program reading $FFC0-$FFFF with the boot ROM mapped sees other bytes than a console's, and the upload handshake can take some cycles more (VenusRT_Native.md §35.3).".into(),
        }),
    }
}

/// What running without the image costs, chip by chip (VenusRT_DspHle.md §5.5), in words a player reads.
fn replacement(stem: Option<&str>) -> Replacement {
    let without = |chip: &str| Replacement::None { cost: format!("VenusRT has no replacement for the {chip} yet: without the image the game runs without its chip.") };
    match stem {
        Some("dsp1") | Some("dsp1b") => Replacement::Accuracy {
            cost: "Without the image, VenusRT's open replacement for the DSP-1 runs: its ports as the chip's, Multiply, Radius, Range, the memory test and the ROM version exact, and every other command a game gives within a few units, so Super Mario Kart and the racing and baseball games play as with the image; a view tilted past about 80 degrees takes the chip's own branch, so Pilotwings' level flight draws its ground as with the image, but Lock On draws no ground (VenusRT_Native.md §52).".into(),
        },
        Some("dsp2") => Replacement::Accuracy {
            cost: "Without the image, VenusRT's open replacement for the DSP-2 runs: every command Dungeon Master gives answers as the chip does, to the cycle, and the game runs as with the image; the data ROM transfer gives zeros, the scaling command's timing is estimated, and counts beyond the chip's buffers are not reproduced (VenusRT_Native.md §42).".into(),
        },
        Some("st010") => Replacement::Accuracy {
            cost: "Without the image, VenusRT's open replacement for the ST010 runs: its mailbox to the cycle, the sort, scale, distance and multiply commands exact, the rotation command within one unit, the raster command without its perspective, and the driver simulation not computed yet, so the opponents' cars do not move as with the image; the battery file is the same on both (VenusRT_Native.md §43).".into(),
        },
        Some("dsp3") => without("DSP-3"),
        Some("dsp4") => Replacement::Accuracy {
            cost: "Without the image, VenusRT's open replacement for the DSP-4 runs its protocol and multiply command, but not the road and scenery commands, so Top Gear 3000 runs its menus as with the image but its race screen stays black (VenusRT_Native.md §48.3).".into(),
        },
        Some(_) => without("ST011"),
        None => Replacement::Accuracy {
            cost: "Without the image, VenusRT runs its open replacement for the DSP-1, DSP-2, DSP-4 or ST010, whose commands are not all computed yet, and a DSP-3 or ST011 game runs without its chip (VenusRT_DspHle.md §5.5).".into(),
        },
    }
}

impl Machine {
    /// What follows a frame: the cheat pokes, then the battery RAM compared with the copy the host last saved.
    fn frame_end(&mut self) {
        for &[space, address, value, compare] in &self.pokes {
            let now = match space {
                0 => self.sys.read_value(address & 0xFF_FFFF, false),
                _ => self.sys.wram.get(address as usize & 0x1_FFFF).copied(),
            };
            if compare != flags::NO_COMPARE && now != Some(compare as u8) {
                continue;
            }
            match space {
                0 => self.sys.poke(address & 0xFF_FFFF, value as u8),
                _ => self.sys.wram[address as usize & 0x1_FFFF] = value as u8,
            }
        }
        self.refresh_st_battery();
        if self.sys.cart.header.battery() && !self.battery_changed && *self.battery_bytes() != *self.battery_copy {
            self.battery_changed = true;
            emit(event::BATTERY, 0, 0);
        }
    }

    /// The debugger's processors, named as its commands' scope words name them: the S-CPU, the SPC700 and the
    /// cartridge's own, if it has one.
    fn processors(&self) -> Vec<Processor> {
        use crate::debugger::{CPU_REGISTERS, Chip, DSP_REGISTERS, GSU_REGISTERS, SPC_REGISTERS};
        let named = |r: &[(&str, u32)]| r.iter().map(|&(n, b)| (n.to_owned(), b)).collect::<Vec<_>>();
        let mut list = vec![
            Processor { id: 0, name: "CPU".into(), pc_bits: 24, registers: named(&CPU_REGISTERS), code_space: Some(0) },
            Processor { id: 1, name: "SPC".into(), pc_bits: 16, registers: named(&SPC_REGISTERS), code_space: Some(7) },
        ];
        match self.chip() {
            Some(Chip::Sa1) => list.push(Processor { id: 2, name: "SA1".into(), pc_bits: 24, registers: named(&CPU_REGISTERS), code_space: Some(crate::ffi::SA1BUS) }),
            Some(Chip::Gsu) => list.push(Processor { id: 2, name: "GSU".into(), pc_bits: 24, registers: named(&GSU_REGISTERS), code_space: Some(crate::ffi::GSUBUS) }),
            Some(Chip::Dsp) => list.push(Processor { id: 2, name: "DSP".into(), pc_bits: 16, registers: named(&DSP_REGISTERS), code_space: Some(crate::ffi::DSPPRG) }),
            None => {}
        }
        list
    }

    /// An ST010 or ST011's battery-backed RAM, the cartridge's save (fullsnes: "680000h-6FFFFFh ST010/ST011 On-chip
    /// Battery-backed RAM"), read out byte by byte as the S-CPU sees it; empty for any other cartridge.
    fn refresh_st_battery(&mut self) {
        if let Some((dsp, _)) = self.sys.cart.dsp.as_mut().filter(|(d, _)| d.st()) {
            let n = 2 * dsp.ram().len();
            self.st_battery.resize(n, 0);
            for (i, b) in self.st_battery.iter_mut().enumerate() {
                *b = dsp.host_read(crate::chips::necdsp::Port::Ram(i), false);
            }
        }
    }

    /// Which path the SPC700's boot program and the cartridge's NEC DSP, if it has one, run on, for machine info.
    fn firmware_sources(&self) -> Vec<(u32, FirmwareSource)> {
        let boot = (1, if self.sys.apu.boot_file { FirmwareSource::File } else { FirmwareSource::Replacement });
        if !self.sys.cart.wants_dsp() && self.sys.cart.dsp.is_none() {
            return vec![boot];
        }
        let source = match self.sys.cart.dsp.as_ref().map(|(d, _)| d.tag()) {
            Some(0) => FirmwareSource::File,
            Some(_) => FirmwareSource::Replacement,
            None => FirmwareSource::Absent,
        };
        vec![boot, (2, source)]
    }

    /// The battery file's bytes: the ST01x's RAM, or the cartridge's RAM.
    fn battery_bytes(&self) -> &[u8] {
        if self.st_battery.is_empty() { &self.sys.cart.sram } else { &self.st_battery }
    }
}

impl Core for Machine {
    const CAPABILITIES: u64 = caps::RESET | caps::SNAPSHOT | caps::BATTERY_DIRTY | caps::ROM_PATCHES | caps::CHEAT_POKES | caps::DEBUG | caps::DEBUG_STACK | caps::DEBUG_REGISTERS | caps::DEBUG_DISASSEMBLE;

    fn info() -> Info {
        Info {
            id: "venusrt".into(),
            name: "VenusRT".into(),
            display_name: Some("VenusRT (Rust)".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("The SNES in Rust, written from hardware documents and graded by test ROMs. Its SPC700 boot program is its own, so no firmware file is needed, and the player's spc700.rom is used in its place when present. With its own program, a program reading $FFC0-$FFFF with the boot ROM mapped sees other bytes than a console's, and the boot handshake's timing can differ by some cycles.".into()),
            systems: vec![System {
                id: "snes".into(),
                name: "Super Nintendo Entertainment System".into(),
                extensions: vec![".smc".into(), ".sfc".into()],
                regions: vec![Region::Ntsc, Region::Pal],
                controllers: vec![Controller {
                    id: "snes.pad".into(),
                    label: "SNES Controller".into(),
                    ports: vec![0, 1],
                    buttons: BUTTONS.iter().enumerate().map(|(bit, &(c, label))| Button { bit: bit as u32, control: Some(c), label: label.into() }).collect(),
                    axes: Vec::new(),
                }],
                firmware: vec![boot(), dsp(None)],
            }],
            deterministic: true,
            ..Info::default()
        }
    }

    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        // The headers' scores alone first: only an image with a NEC DSP's chipset among its candidates is loaded whole.
        let rom = if image.len() % 1024 == 512 { &image[512..] } else { image };
        if !crate::cart::candidates(rom).iter().any(|h| matches!(h.chipset, 0x03..=0x05 | 0xF6)) {
            return vec![boot()];
        }
        match crate::cart::Cartridge::new(image).and_then(|c| c.nec_firmware()) {
            Some(named) => vec![boot(), dsp(Some(named))],
            None => vec![boot()],
        }
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_IMAGE_TOO_SHORT => Some("the image is shorter than one 32 KiB bank after any copier header".into()),
            crate::state::STATUS_OTHER_DSP_ENGINE => Some("the state was saved with the other NEC DSP engine: the player's image against VenusRT's replacement".into()),
            _ => None,
        }
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        let mut m = <Machine as NativeCore>::create(request.image, &Settings::default(), &request.files)?;
        if let (Some((dsp, _)), Some(file)) = (m.sys.cart.dsp.as_mut().filter(|(d, _)| d.st()), request.files.iter().find(|f| f.which == 0)) {
            for (i, &b) in file.data.iter().enumerate().take(2 * dsp.ram().len()) {
                dsp.host_write(crate::chips::necdsp::Port::Ram(i), b);
            }
        }
        m.refresh_st_battery();
        m.battery_copy = m.battery_bytes().to_vec();
        Ok(m)
    }

    fn machine_info(&self) -> MachineInfo {
        let pal = self.sys.timing.pal;
        MachineInfo {
            system: "snes".into(),
            region: Some(if pal { Region::Pal } else { Region::Ntsc }),
            frame_rate: if pal { PAL_FRAME } else { NTSC_FRAME },
            video: Video { base_width: 256, base_height: 224, max_width: 512, max_height: 448, aspect: (4, 3), formats: vec![pixel::RGBA8888] },
            audio: Audio { rate: DSP_RATE, channels: Vec::new() },
            ports: vec![Port { port: 0, controller: Some("snes.pad".into()) }, Port { port: 1, controller: Some("snes.pad".into()) }],
            spaces: SPACES
                .iter()
                .chain(&CHIP_SPACES)
                .enumerate()
                .filter(|&(id, _)| id < SPACES.len() || self.chip_space_size(id as u32).is_some())
                .map(|(id, &name)| Space {
                    read_only: matches!(name, "IO" | "GSUBUS" | "SA1BUS" | "DSPPRG"),
                    cheats: matches!(name, "CpuBus" | "WRAM"),
                    reports_stores: matches!(name, "CpuBus" | "WRAM" | "SRAM" | "APURAM"),
                    ..Space::new(id as u32, name)
                })
                .collect(),
            processors: self.processors(),
            battery: vec![Battery { which: 0, suffix: ".srm".into() }],
            state: StateFormat { format: "VNRT".into(), version: STATE_VERSION as i64, loads_from: vec![17, 18, 19, 20, 21, STATE_VERSION as i64] },
            phases: Vec::new(),
            patches: Some((0, 0xFF_FFFF)),
            skip_rendering_state_neutral: true,
            firmware: self.firmware_sources(),
        }
    }

    fn reset(&mut self) -> Result<(), i32> {
        Machine::reset(self);
        Ok(())
    }

    /// The frame, with nothing of the debugger's fitted, then what follows every frame.
    fn advance(&mut self, detail: &mut u64) -> Result<(), i32> {
        self.disarm();
        NativeCore::advance(self, detail)?;
        self.frame_end();
        Ok(())
    }

    fn set_options(&mut self, flags: u32) {
        NativeCore::set_options(self, flags)
    }

    fn frame_count(&self) -> i64 {
        NativeCore::frame_count(self)
    }

    fn frame_info(&self) -> FrameInfo {
        FrameInfo { aspect_num: 4, aspect_den: 3, ..FrameInfo::rgba(self.sys.ppu.frame_width as i32, self.sys.ppu.frame_height as i32) }
    }

    fn frame(&self) -> &[u8] {
        NativeCore::frame(self)
    }

    fn audio_rate(&self) -> i32 {
        NativeCore::audio_rate(self)
    }

    fn audio_buffered(&self) -> usize {
        NativeCore::audio_buffered(self)
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        NativeCore::drain_audio(self, out, max_frames)
    }

    fn set_audio_limit(&mut self, samples: usize) {
        NativeCore::set_audio_limit(self, samples)
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        if port > 1 {
            return Err(status::NO_SUCH_PORT);
        }
        NativeCore::set_buttons(self, port, mask, changed)
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        NativeCore::space_size(self, space)
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        NativeCore::space_read(self, space, address, out)
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        NativeCore::space_write(self, space, address, data)
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        let (bytes, _) = NativeCore::battery(self, which)?;
        let bytes = if self.st_battery.is_empty() { bytes } else { &self.st_battery[..] };
        Ok((bytes, flags::BATTERY_TRACKED | if self.battery_changed { flags::BATTERY_CHANGED } else { 0 }))
    }

    fn battery_saved(&mut self, which: u32) -> Result<(), i32> {
        if which != 0 {
            return Err(status::BAD_FILE);
        }
        self.battery_copy = self.battery_bytes().to_vec();
        self.battery_changed = false;
        Ok(())
    }

    /// The generated snapshot is the full state, which `state_load` reads for either kind.
    fn snapshot_size(&self) -> usize {
        self.state_size()
    }

    fn save_snapshot(&mut self, out: &mut [u8]) -> Result<usize, i32> {
        self.save_state(out).map_err(|e| emusen_native::ffi::Status::status(&e))
    }

    fn snapshot_layout(&self) -> String {
        self.layout()
    }

    fn set_rom_patches(&mut self, triples: &[u32]) -> Result<(), i32> {
        self.sys.patches = triples.chunks_exact(3).map(|t| (t[0] & 0xFF_FFFF, t[1] as u8, t[2])).collect();
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut Hooks> {
        Some(&mut self.hooks)
    }

    /// Processor 0's breakpoints live in the hooks; the SPC700's and the cartridge processor's go to their probes.
    fn debug_breakpoints(&mut self, processor: u32, pairs: &[i32]) -> Result<(), i32> {
        match processor {
            0 => self.hooks.set_breakpoints(pairs),
            1 | 2 if processor == 1 || self.chip().is_some() => self.debug_breakpoints[processor as usize - 1] = pairs.chunks_exact(2).map(|p| (p[0], p[1])).collect(),
            _ => return Err(status::NOT_SUPPORTED),
        }
        Ok(())
    }

    fn debug_run_frame(&mut self, flags: u32, _detail: &mut u64) -> Result<u32, i32> {
        let why = self.run_frame_debug(flags);
        if why == emusen_native::debug::stop::FRAME {
            self.frame_end();
        }
        Ok(why)
    }

    fn debug_stopped(&self) -> u32 {
        self.debug_stopped
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        Machine::debug_pc(self, processor)
    }

    fn debug_registers(&self, processor: u32) -> Result<Vec<i64>, i32> {
        Machine::debug_registers(self, processor).ok_or(status::NOT_SUPPORTED)
    }

    /// The instruction set is the space's where the space is a processor's own (APURAM the SPC700's, GSUBUS and
    /// GSURAM the GSU's, SA1BUS, SA1IRAM and BWRAM the SA-1's, DSPPRG the DSP's), else `processor`'s.
    fn debug_disassemble(&mut self, processor: u32, space: u32, address: u32, count: u32) -> Result<Vec<Instruction>, i32> {
        use crate::debugger::Chip;
        use crate::disasm::{gsu, spc700, upd77c25, w65816};
        use crate::ffi::{BWRAM, DSPPRG, GSUBUS, GSURAM, SA1BUS, SA1IRAM};
        #[derive(Clone, Copy, PartialEq)]
        enum Isa {
            Cpu,
            Sa1,
            Spc,
            Gsu,
            Dsp,
        }
        NativeCore::space_size(self, space)?;
        let chip = self.chip();
        let isa = match (space, processor, chip) {
            (7, _, _) => Isa::Spc,
            (GSURAM | GSUBUS, _, _) => Isa::Gsu,
            (SA1BUS | SA1IRAM | BWRAM, _, _) => Isa::Sa1,
            (DSPPRG, _, _) => Isa::Dsp,
            (_, 1, _) => Isa::Spc,
            (_, 2, Some(Chip::Sa1)) => Isa::Sa1,
            (_, 2, Some(Chip::Gsu)) => Isa::Gsu,
            (_, 2, Some(Chip::Dsp)) => Isa::Dsp,
            (_, 0, _) => Isa::Cpu,
            _ => return Err(status::NOT_SUPPORTED),
        };
        let cpu = match (isa, self.sys.cart.sa1.as_ref()) {
            (Isa::Sa1, Some(s)) => s.cpu,
            _ => self.cpu,
        };
        let mut widths = w65816::Widths { m: cpu.e || cpu.p & 0x20 != 0, x: cpu.e || cpu.p & 0x10 != 0, e: cpu.e };
        let mut prefix = match self.sys.cart.gsu.as_ref() {
            Some(g) if Machine::debug_pc(self, 2) == Some(address as u64) => gsu::Prefix { alt: g.alt, b: g.b },
            _ => gsu::Prefix::default(),
        };
        let st = self.sys.cart.dsp.as_ref().is_some_and(|(d, _)| d.st());
        // The SPC700's code at $FFC0-$FFFF is the boot program while CONTROL maps it, whatever the RAM beneath holds.
        let boot = space == 7 && isa == Isa::Spc && self.sys.apu.control & 0x80 != 0;
        let machine = std::cell::RefCell::new(self);
        let read = |a: u32| {
            if boot && (0xFFC0..=0xFFFF).contains(&a) {
                return machine.borrow().sys.apu.boot[(a - 0xFFC0) as usize];
            }
            let mut byte = [0u8];
            let _ = NativeCore::space_read(&mut **machine.borrow_mut(), space, a, &mut byte);
            byte[0]
        };
        let mut list = Vec::new();
        let mut at = address;
        for _ in 0..count.min(4096) {
            let i = match isa {
                Isa::Cpu | Isa::Sa1 => w65816::decode(&read, at, &mut widths, cpu.dbr),
                Isa::Spc => spc700::decode(&read, at),
                Isa::Gsu => gsu::decode(&read, at, &mut prefix),
                Isa::Dsp => upd77c25::decode(&read, at, st),
            };
            let n = i.bytes.len() as u32;
            at = match isa {
                Isa::Spc | Isa::Dsp => i.address.wrapping_add(n),
                _ => (at & 0xFF_0000) | (at.wrapping_add(n) & 0xFFFF),
            };
            list.push(i);
        }
        Ok(list)
    }

    /// Pokes in CpuBus (space 0) and WRAM (space 2), the spaces machine info marks for cheats.
    fn set_cheat_pokes(&mut self, quads: &[u32]) -> Result<(), i32> {
        if quads.chunks_exact(4).any(|q| q[0] != 0 && q[0] != 2) {
            detail("cheat pokes reach CpuBus and WRAM only");
            return Err(status::NO_SUCH_SPACE);
        }
        self.pokes = quads.chunks_exact(4).map(|q| [q[0], q[1], q[2], q[3]]).collect();
        Ok(())
    }
}

emusen_native::core_exports!(Machine; reset, rom_patches, cheat_pokes, debug, debug_stack, debug_registers, debug_disassemble);

#[cfg(test)]
mod tests {
    use super::*;

    fn machine() -> Machine {
        let mut image = crate::machine::tests::rom(&[0x80, 0xFE]);
        (image[0x7FD6], image[0x7FD8]) = (0x02, 0x03);
        let request = Create { image: &image, settings: Settings::default(), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION };
        <Machine as Core>::create(&request).unwrap()
    }

    fn state(m: &Machine) -> Vec<u8> {
        let mut v = vec![0; m.state_size()];
        m.save_state(&mut v).unwrap();
        v
    }

    fn named(title: &[u8], chipset: u8) -> Vec<String> {
        let mut image = crate::machine::tests::rom(&[]);
        image[0x7FC0..0x7FD5].fill(b' ');
        image[0x7FC0..0x7FC0 + title.len()].copy_from_slice(title);
        image[0x7FD6] = chipset;
        <Machine as Core>::firmware_for(&image).iter().map(|f| format!("{} {} {}", f.which, f.name, f.size)).collect()
    }

    // The NEC DSP named from the header and title by fullsnes's list of games; nothing for a game without one (D-38).
    #[test]
    fn the_firmware_is_named_from_the_header_and_title() {
        assert_eq!(named(b"SUPER MARIO WORLD", 0x02), ["1 spc700.rom 64"]);
        assert_eq!(named(b"PILOTWINGS", 0x03), ["1 spc700.rom 64", "2 dsp1.rom 8192"]);
        assert_eq!(named(b"SUPER MARIO KART", 0x05)[1], "2 dsp1b.rom 8192");
        assert_eq!(named(b"DUNGEON MASTER", 0x03)[1], "2 dsp2.rom 8192");
        assert_eq!(named(b"TOP GEAR 3000", 0x03)[1], "2 dsp4.rom 8192");
        assert_eq!(named(b"F1 ROC II", 0xF6)[1], "2 st010.rom 53248");
        assert_eq!(named(b"MORITA SHOGI", 0xF6)[1], "2 st011.rom 53248");
        assert!(<Machine as Core>::info().systems[0].firmware.iter().all(|f| f.which >= 1));
    }

    // A DSP's firmware whole as file 2, or as its program in file 2 and its data in file 3; the data alone is refused.
    #[test]
    fn a_dsp_firmware_comes_whole_or_in_two_parts() {
        let mut image = crate::machine::tests::rom(&[0x80, 0xFE]);
        image[0x7FD6] = 0x03;
        let (program, data) = (vec![0u8; 6_144], vec![0u8; 2_048]);
        let make = |files: Vec<File<'_>>| <Machine as Core>::create(&Create { image: &image, settings: Settings::default(), files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION });
        let whole = [program.as_slice(), data.as_slice()].concat();
        assert!(make(vec![File { which: 2, data: &whole }]).unwrap().sys.cart.dsp.is_some());
        assert!(make(vec![File { which: 2, data: &program }, File { which: 3, data: &data }]).unwrap().sys.cart.dsp.is_some());
        assert_eq!(make(vec![File { which: 3, data: &data }]).err(), Some(status::BAD_FILE));
    }

    fn dsp_cartridge(title: &[u8], chipset: u8) -> Vec<u8> {
        let mut image = crate::machine::tests::rom(&[0x80, 0xFE]);
        image[0x7FC0..0x7FD5].fill(b' ');
        image[0x7FC0..0x7FC0 + title.len()].copy_from_slice(title);
        image[0x7FD6] = chipset;
        image
    }

    fn create(image: &[u8], files: Vec<File<'_>>) -> Result<Machine, i32> {
        <Machine as Core>::create(&Create { image, settings: Settings::default(), files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION })
    }

    // VenusRT_DspHle.md §7: no file is required, each entry names its replacement's effect, and create picks the
    // engine from whether file 2 came; machine info says which path runs.
    #[test]
    fn a_dsp_game_runs_without_its_image_on_the_replacement_or_without_its_chip() {
        for title in [&b"SUPER MARIO KART"[..], b"PILOTWINGS", b"DUNGEON MASTER", b"TOP GEAR 3000"] {
            let list = <Machine as Core>::firmware_for(&dsp_cartridge(title, 0x03));
            assert!(list.iter().all(|f| !f.required && f.replacement.is_some()));
            let doc = emusen_native::core::desc::firmware_json(&list);
            assert_eq!(emusen_native::core::schema::validate(emusen_native::core::schema::FIRMWARE, &doc), Vec::<String>::new());
        }
        assert!(matches!(<Machine as Core>::firmware_for(&dsp_cartridge(b"TOP GEAR 3000", 0x03))[1].replacement, Some(Replacement::Accuracy { .. })));
        assert!(<Machine as Core>::info().systems[0].firmware.iter().all(|f| !f.required && f.replacement.is_some()));
        let kart = create(&dsp_cartridge(b"SUPER MARIO KART", 0x05), Vec::new()).unwrap();
        assert_eq!(kart.sys.cart.dsp.as_ref().map(|(d, _)| d.tag()), Some(1));
        assert_eq!(Core::machine_info(&kart).firmware, vec![(1, FirmwareSource::Replacement), (2, FirmwareSource::Replacement)]);
        assert!(!Core::machine_info(&kart).processors.iter().any(|p| p.name == "DSP"));
        let image = vec![0u8; 8192];
        let lle = create(&dsp_cartridge(b"SUPER MARIO KART", 0x05), vec![File { which: 2, data: &image }]).unwrap();
        assert_eq!(Core::machine_info(&lle).firmware, vec![(1, FirmwareSource::Replacement), (2, FirmwareSource::File)]);
        let tg = create(&dsp_cartridge(b"TOP GEAR 3000", 0x03), Vec::new()).unwrap();
        assert_eq!(tg.sys.cart.dsp.as_ref().map(|(d, _)| d.tag()), Some(1));
        assert_eq!(Core::machine_info(&tg).firmware, vec![(1, FirmwareSource::Replacement), (2, FirmwareSource::Replacement)]);
        let gundam = create(&dsp_cartridge(b"SD GUNDAM GX", 0x03), Vec::new()).unwrap();
        assert!(gundam.sys.cart.dsp.is_none());
        assert_eq!(Core::machine_info(&gundam).firmware, vec![(1, FirmwareSource::Replacement), (2, FirmwareSource::Absent)]);
        assert_eq!(Core::machine_info(&machine()).firmware, vec![(1, FirmwareSource::Replacement)]);
    }

    // EmuSen_Firmware.md §0: a 64-byte file 1 is the player's boot image, mapped at $FFC0-$FFFF in place of VenusRT's
    // own program and kept across reset; a synthetic image here, which parks the SPC700 at its own vector.
    #[test]
    fn the_players_boot_image_replaces_the_open_program() {
        let mut image = [0u8; 64];
        image[0] = 0x2F;
        image[1] = 0xFE;
        (image[62], image[63]) = (0xC0, 0xFF);
        let rom = crate::machine::tests::rom(&[0x80, 0xFE]);
        let mut m = create(&rom, vec![File { which: 1, data: &image }]).unwrap();
        assert_eq!(Core::machine_info(&m).firmware, vec![(1, FirmwareSource::File)]);
        Core::advance(&mut m, &mut 0).unwrap();
        assert_eq!(m.sys.apu.cpu.pc, 0xFFC0, "the image's loop at its vector");
        let mut b = [0u8; 2];
        NativeCore::space_read(&mut m, 0, 0x2140, &mut b).unwrap();
        assert_ne!(b, [0xAA, 0xBB], "VenusRT's own program would have signalled ready");
        Core::reset(&mut m).unwrap();
        assert!(m.sys.apu.boot_file && m.sys.apu.boot == image);
        assert!(!create(&rom, Vec::new()).unwrap().sys.apu.boot_file);
    }

    // VenusRT_Native.md §43: the ST010's battery file is its RAM on either engine, so a file one engine wrote is the
    // RAM the other runs from; the image's engine runs only with an image (EMUSEN_VENUSRT_FIRMWARE).
    #[test]
    fn the_st010_battery_file_reads_alike_on_either_engine() {
        let cart = dsp_cartridge(b"F1 ROC II", 0xF6);
        let file: Vec<u8> = (0..4096u32).map(|i| (i.wrapping_mul(2654435761) >> 13) as u8).collect();
        let hle = create(&cart, vec![File { which: 0, data: &file }]).unwrap();
        assert_eq!(hle.sys.cart.dsp.as_ref().map(|(d, _)| d.tag()), Some(1));
        assert_eq!(hle.battery_bytes(), &file[..]);
        let Some(image) = crate::chips::dsporacle::firmware("st010") else { return };
        let lle = create(&cart, vec![File { which: 0, data: hle.battery_bytes() }, File { which: 2, data: &image }]).unwrap();
        assert_eq!(lle.sys.cart.dsp.as_ref().map(|(d, _)| d.tag()), Some(0));
        assert_eq!(lle.battery_bytes(), hle.battery_bytes());
        let again = create(&cart, vec![File { which: 0, data: lle.battery_bytes() }]).unwrap();
        assert_eq!(again.battery_bytes(), &file[..]);
    }

    // VenusRT_Native.md §48.1: version 20 is stage 8's; a replacement's state from before 21 is refused with VERSION,
    // since its layout cannot be told from that version alone.
    #[test]
    fn a_replacement_state_before_version_21_is_refused() {
        let cart = dsp_cartridge(b"DUNGEON MASTER", 0x03);
        let mut m = create(&cart, Vec::new()).unwrap();
        Core::advance(&mut m, &mut 0).unwrap();
        let mut old = state(&m);
        old[4..8].copy_from_slice(&20i32.to_le_bytes());
        let err = m.load_state(&old).unwrap_err();
        assert_eq!(emusen_native::ffi::Status::status(&err), emusen_native::ffi::status::VERSION);
    }

    // VenusRT_Native.md §49.5: the DSP-1 replacement's state gained the projection at 22, so one of 21 is refused.
    #[test]
    fn a_dsp1_replacement_state_before_version_22_is_refused() {
        let cart = dsp_cartridge(b"SUPER MARIO KART", 0x05);
        let mut m = create(&cart, Vec::new()).unwrap();
        Core::advance(&mut m, &mut 0).unwrap();
        let mut old = state(&m);
        old[4..8].copy_from_slice(&21i32.to_le_bytes());
        let err = m.load_state(&old).unwrap_err();
        assert_eq!(emusen_native::ffi::Status::status(&err), emusen_native::ffi::status::VERSION);
    }

    // A state names the engine that wrote it, and the other engine refuses it with its own status.
    #[test]
    fn a_state_is_refused_by_the_other_dsp_engine() {
        let cart = dsp_cartridge(b"SUPER MARIO KART", 0x05);
        let mut hle = create(&cart, Vec::new()).unwrap();
        Core::advance(&mut hle, &mut 0).unwrap();
        let saved = state(&hle);
        assert!(hle.layout().contains(" Coprocessor.DspEngine\n") && hle.layout().contains(" Coprocessor.DspHle\n"), "{}", hle.layout());
        let mut again = create(&cart, Vec::new()).unwrap();
        again.load_state(&saved).unwrap();
        assert_eq!(state(&again), saved);
        let image = vec![0u8; 8192];
        let mut lle = create(&cart, vec![File { which: 2, data: &image }]).unwrap();
        let err = lle.load_state(&saved).unwrap_err();
        assert_eq!(emusen_native::ffi::Status::status(&err), crate::state::STATUS_OTHER_DSP_ENGINE);
        assert!(<Machine as Core>::status_text(crate::state::STATUS_OTHER_DSP_ENGINE).is_some());
    }

    // RESET: the CPU at its vector again, the memories and the frame count kept.
    #[test]
    fn reset_starts_the_machine_again_and_keeps_its_memories() {
        let mut m = machine();
        for _ in 0..3 {
            Core::advance(&mut m, &mut 0).unwrap();
        }
        m.sys.wram[0x100] = 0x42;
        m.sys.cart.sram[3] = 0x24;
        m.sys.apu.ram[0x400] = 0x99;
        Core::reset(&mut m).unwrap();
        assert_eq!((m.cpu.pbr, m.cpu.pc, m.cpu.e), (0, 0x8000, true));
        assert_eq!((m.sys.wram[0x100], m.sys.cart.sram[3], m.sys.apu.ram[0x400], Core::frame_count(&m)), (0x42, 0x24, 0x99, 3));
        Core::advance(&mut m, &mut 0).unwrap();
        assert_eq!(Core::frame_count(&m), 4);
    }

    // BATTERY_DIRTY: a change to the battery RAM is reported until the host says it saved it.
    #[test]
    fn a_battery_change_is_tracked_until_saved() {
        let mut m = machine();
        Core::advance(&mut m, &mut 0).unwrap();
        assert_eq!(Core::battery(&m, 0).unwrap().1, flags::BATTERY_TRACKED);
        Core::space_write(&mut m, 0, 0x70_0001, &[1]).unwrap();
        Core::advance(&mut m, &mut 0).unwrap();
        assert_eq!(Core::battery(&m, 0).unwrap().1, flags::BATTERY_TRACKED | flags::BATTERY_CHANGED);
        Core::battery_saved(&mut m, 0).unwrap();
        Core::advance(&mut m, &mut 0).unwrap();
        assert_eq!(Core::battery(&m, 0).unwrap().1, flags::BATTERY_TRACKED);
    }

    // ROM_PATCHES: a patch answers for the cartridge, with or without its compare value, never for WRAM.
    #[test]
    fn rom_patches_answer_for_the_cartridge_only() {
        let mut m = machine();
        Core::set_rom_patches(&mut m, &[0x00_8001, 0x55, flags::NO_COMPARE, 0x00_8002, 0x66, 0x12, 0x7E_0000, 0x77, flags::NO_COMPARE]).unwrap();
        assert_eq!(m.sys.read_value(0x00_8001, false), Some(0x55));
        assert_eq!(m.sys.read_value(0x80_8001, false), Some(0xFE));
        assert_eq!(m.sys.read_value(0x00_8002, false), Some(0xEA));
        assert_eq!(m.sys.read_value(0x7E_0000, false), Some(0));
        Core::set_rom_patches(&mut m, &[]).unwrap();
        assert_eq!(m.sys.read_value(0x00_8001, false), Some(0xFE));
    }

    // CHEAT_POKES: applied at each frame's end, a compare value holding a poke until the byte matches it.
    #[test]
    fn cheat_pokes_land_at_the_frames_end() {
        let mut m = machine();
        Core::set_cheat_pokes(&mut m, &[2, 0x10, 0x63, flags::NO_COMPARE, 0, 0x7E_0011, 0x64, 0x01, 0, 0x70_0002, 0x65, flags::NO_COMPARE]).unwrap();
        Core::advance(&mut m, &mut 0).unwrap();
        assert_eq!((m.sys.wram[0x10], m.sys.wram[0x11], m.sys.cart.sram[2]), (0x63, 0, 0x65));
        assert_eq!(Core::set_cheat_pokes(&mut m, &[3, 0, 0, flags::NO_COMPARE]), Err(status::NO_SUCH_SPACE));
    }

    // An ST010's battery is its on-chip RAM, read from file 0 and reported byte by byte as the S-CPU sees it at $68:0000.
    #[test]
    fn an_st01x_battery_is_its_on_chip_ram() {
        let mut image = crate::machine::tests::rom(&[0x80, 0xFE]);
        image[0x7FD6] = 0xF6;
        let firmware = vec![0u8; 53_248];
        let saved: Vec<u8> = (0..4096).map(|i| (i * 5 + 1) as u8).collect();
        let files = vec![File { which: 0, data: &saved }, File { which: 2, data: &firmware }];
        let mut m = <Machine as Core>::create(&Create { image: &image, settings: Settings::default(), files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION }).unwrap();
        assert_eq!(Core::battery(&m, 0).unwrap(), (&saved[..], flags::BATTERY_TRACKED));
        assert_eq!(m.sys.read_value(0x68_0003, false), Some(saved[3]));
        Core::space_write(&mut m, 0, 0x68_0003, &[0xEE]).unwrap();
        Core::advance(&mut m, &mut 0).unwrap();
        let (bytes, f) = Core::battery(&m, 0).unwrap();
        assert_eq!((bytes[3], f), (0xEE, flags::BATTERY_TRACKED | flags::BATTERY_CHANGED));
    }

    // DEBUG_DISASSEMBLE: the space picks the instruction set where it is a processor's own, else the processor does.
    #[test]
    fn disassembly_goes_by_the_space_and_the_processor() {
        let mut m = machine();
        let text = |m: &mut Machine, processor: u32, space: u32, at: u32| {
            Core::debug_disassemble(m, processor, space, at, 2).map(|l| l.iter().map(|i| format!("{:06X} {} {}", i.address, i.mnemonic, i.operands).trim_end().to_owned()).collect::<Vec<_>>())
        };
        assert_eq!(text(&mut m, 0, 0, 0x00_8000).unwrap(), ["008000 BRA $8000", "008002 NOP"]);
        Core::space_write(&mut m, 7, 0x0200, &[0xE8, 0x12, 0x3F, 0x00, 0x03]).unwrap();
        assert_eq!(text(&mut m, 0, 7, 0x0200).unwrap(), ["000200 MOV A,#$12", "000202 CALL !$0300"]);
        assert_eq!(text(&mut m, 1, 7, 0xFFC0).unwrap()[0], "00FFC0 MOV X,#$EF");
        Core::space_write(&mut m, 2, 0x0100, &[0xE8, 0x12, 0xEA]).unwrap();
        assert_eq!(text(&mut m, 0, 2, 0x0100).unwrap(), ["000100 INX", "000101 ORA ($EA)"]);
        assert_eq!(text(&mut m, 1, 2, 0x0100).unwrap()[0], "000100 MOV A,#$12");
        assert_eq!(text(&mut m, 2, 2, 0x0100), Err(status::NOT_SUPPORTED));
        assert_eq!(text(&mut m, 0, 99, 0), Err(status::NO_SUCH_SPACE));
    }

    // SNAPSHOT: the full state under kind 1.
    #[test]
    fn the_snapshot_is_the_state() {
        let mut m = machine();
        Core::advance(&mut m, &mut 0).unwrap();
        let mut snap = vec![0; Core::snapshot_size(&m)];
        assert_eq!(Core::save_snapshot(&mut m, &mut snap), Ok(snap.len()));
        assert_eq!(snap, state(&m));
    }

    // A cheat's CpuBus poke lands in WRAM through its mirror and in the cartridge's SRAM; one at an I/O register, the
    // APU's ports or ROM changes nothing in the state.
    #[test]
    fn cpu_bus_pokes_land_in_memory_and_nowhere_else() {
        let mut m = machine();
        Core::space_write(&mut m, 0, 0x00_1234, &[7]).unwrap();
        Core::space_write(&mut m, 0, 0x70_0010, &[9]).unwrap();
        Core::space_write(&mut m, 0, 0x7F_0001, &[5]).unwrap();
        assert_eq!((m.sys.wram[0x1234], m.sys.cart.sram[0x10], m.sys.wram[0x1_0001]), (7, 9, 5));
        let before = state(&m);
        for a in [0x00_2100u32, 0x00_2140, 0x00_4200, 0x00_420B, 0x00_4300, 0x00_8000, 0x80_FFFF] {
            Core::space_write(&mut m, 0, a, &[0xFF]).unwrap();
        }
        assert_eq!(state(&m), before);
        let info = m.machine_info();
        assert_eq!((info.spaces[0].name.as_str(), info.spaces[2].name.as_str()), ("CpuBus", "WRAM"));
        assert!(info.spaces[0].cheats && info.spaces[2].cheats && info.spaces[1].read_only);
    }
}
