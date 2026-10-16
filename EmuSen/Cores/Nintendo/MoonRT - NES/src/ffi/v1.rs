//! MoonRT on the core ABI v1, beside its pre-stable exports until the v1 adapter is its only loader. The console's
//! facts its C# shim held are its descriptors here. See EmuSen_CoreAPI.md §13.1, §22.

use emusen_native::core::{
    self as v1, Battery, Button, Control, Controller, Create, Firmware, FrameInfo, Info, MachineInfo, Port, Processor, RawEncoding, RawFrame, Region, Space, StateFormat,
    System, Video, caps, status,
};

use super::{BATTERY_SPACE, STATUS_NOT_INES, STATUS_NULL, STATUS_PRG_TRUNCATED, STATUS_UNSUPPORTED_BOARD, build_rom_patches, fault_status, rom_patch};
use crate::machine::{LoadError, Machine, OLDEST_READABLE_VERSION, STATE_VERSION};
use crate::memory::cartridge::RomError;
use crate::ppu::{FRAME_BYTES, SCREEN_HEIGHT, SCREEN_WIDTH, TOTAL_SCANLINES};
use crate::ppu::render::NES_PALETTE;

/// `MoonCore.MasterClockHz` over the master clocks of a frame, 341 dots of 4 a scanline.
const MASTER_CLOCK_HZ: u64 = 21_477_272;
const MASTER_CLOCKS_PER_SCANLINE: u64 = 341 * 4;

/// `MoonMachine.SpaceNames`, numbered as the ABI numbers them.
const SPACES: [&str; 8] = ["RAM", "PRGROM", "PRGRAM", "CHR", "CIRAM", "OAM", "PALETTE", "CPUBUS"];

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
    const CAPABILITIES: u64 = caps::RESET | caps::MUTES | caps::ROM_PATCHES | caps::DEBUG | caps::DEBUG_STACK | caps::RAW_PICTURE;

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
            battery: if battery { vec![Battery::game(0, ".srm")] } else { Vec::new() },
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

    fn end_call() {
        crate::take_fault();
    }
}

emusen_native::core_exports!(Machine; reset, mutes, rom_patches, debug, debug_stack, raw_picture);

/// `moonrt_rom_patch` for a machine of the core ABI v1: the byte a CPU read of `address` returns for `original` under
/// the current patches, or -1 for none, the exhaustive patch test's view through the v1 loader (EmuSen_CoreAPI.md §26).
///
/// # Safety
/// `machine` must be null or a live handle `emusen_core_create` returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moonrt_core_rom_patch(machine: *const v1::sys::Machine, address: u32, original: u32) -> i32 {
    unsafe { v1::exports::core_of::<Machine>(machine) }.map_or(STATUS_NULL, |m| rom_patch(m, address, original))
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
}
