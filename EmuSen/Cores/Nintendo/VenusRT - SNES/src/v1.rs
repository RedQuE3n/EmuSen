//! VenusRT on the stable core ABI, version 1: `Core` over the machine and `core_exports!`, beside the pre-stable
//! `native_exports!` of `ffi` until the host's adapter is VenusRT's only loader. See EmuSen_CoreAPI.md §18.6 and
//! VenusRT_Native.md §31.

use emusen_native::abi::NativeCore;
use emusen_native::core::*;

use crate::ffi::{STATUS_IMAGE_TOO_SHORT, STATUS_NO_IPL};
use crate::machine::{DSP_RATE, Machine};
use crate::state::STATE_VERSION;

/// The space names, in id order; `CpuBus` and `WRAM` are the SNES system pack's, which its cheat codecs target.
pub const SPACES: [&str; 8] = ["CpuBus", "IO", "WRAM", "VRAM", "CGRAM", "OAM", "SRAM", "APURAM"];

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

fn ipl() -> Firmware {
    Firmware { which: 1, name: "spc700.rom".into(), label: "The sound unit's 64-byte boot ROM".into(), size: 64, required: true, parts: Vec::new() }
}

/// A NEC DSP cartridge's firmware as file 2, whole or as its program and data in files 2 and 3; `stem` and `size` as
/// `Cartridge::nec_firmware` names them, or the open entry core info lists for any of them.
fn dsp(named: Option<(&str, u64)>) -> Firmware {
    let (name, size, parts) = match named {
        Some((stem, size)) => (format!("{stem}.rom"), size, vec![vec![format!("{stem}.rom")], vec![format!("{stem}.program.rom"), format!("{stem}.data.rom")]]),
        None => ("dsp.rom".to_owned(), 0, Vec::new()),
    };
    Firmware { which: 2, name, label: "The cartridge's NEC DSP program and data: DSP-1 to DSP-4 (8,192 bytes) or ST010/ST011 (53,248)".into(), size, required: true, parts }
}

impl Core for Machine {
    const CAPABILITIES: u64 = caps::RESET | caps::SNAPSHOT | caps::BATTERY_DIRTY | caps::ROM_PATCHES | caps::CHEAT_POKES;

    fn info() -> Info {
        Info {
            id: "venusrt".into(),
            name: "VenusRT".into(),
            display_name: Some("VenusRT (Rust)".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("The SNES in Rust, written from hardware documents and graded by test ROMs.".into()),
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
                firmware: vec![ipl(), dsp(None)],
            }],
            deterministic: true,
            ..Info::default()
        }
    }

    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        match crate::cart::Cartridge::new(image).and_then(|c| c.nec_firmware()) {
            Some(named) => vec![ipl(), dsp(Some(named))],
            None => vec![ipl()],
        }
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_IMAGE_TOO_SHORT => Some("the image is shorter than one 32 KiB bank after any copier header".into()),
            STATUS_NO_IPL => Some("the sound unit's 64-byte boot ROM, file 1, was not given".into()),
            _ => None,
        }
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        let mut m = <Machine as NativeCore>::create(request.image, &Settings::default(), &request.files)?;
        m.battery_copy = m.sys.cart.sram.to_vec();
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
                .enumerate()
                .map(|(id, &name)| Space { read_only: name == "IO", cheats: matches!(name, "CpuBus" | "WRAM"), ..Space::new(id as u32, name) })
                .collect(),
            processors: Vec::new(),
            battery: vec![Battery { which: 0, suffix: ".srm".into() }],
            state: StateFormat { format: "VNRT".into(), version: STATE_VERSION as i64, loads_from: vec![STATE_VERSION as i64] },
            phases: Vec::new(),
            patches: Some((0, 0xFF_FFFF)),
            skip_rendering_state_neutral: true,
        }
    }

    fn reset(&mut self) -> Result<(), i32> {
        Machine::reset(self);
        Ok(())
    }

    /// The frame, then the cheat pokes, then the battery RAM compared with the copy the host last saved.
    fn advance(&mut self, detail: &mut u64) -> Result<(), i32> {
        NativeCore::advance(self, detail)?;
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
        if self.sys.cart.header.battery() && !self.battery_changed && *self.sys.cart.sram != *self.battery_copy {
            self.battery_changed = true;
            emit(event::BATTERY, 0, 0);
        }
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
        Ok((bytes, flags::BATTERY_TRACKED | if self.battery_changed { flags::BATTERY_CHANGED } else { 0 }))
    }

    fn battery_saved(&mut self, which: u32) -> Result<(), i32> {
        if which != 0 {
            return Err(status::BAD_FILE);
        }
        self.battery_copy = self.sys.cart.sram.to_vec();
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

emusen_native::core_exports!(Machine; reset, rom_patches, cheat_pokes);

#[cfg(test)]
mod tests {
    use super::*;

    fn machine() -> Machine {
        let mut image = crate::machine::tests::rom(&[0x80, 0xFE]);
        (image[0x7FD6], image[0x7FD8]) = (0x02, 0x03);
        let ipl = crate::apu::smp::tests::idle_ipl();
        let request = Create { image: &image, settings: Settings::default(), files: vec![File { which: 1, data: &ipl }], pixel_formats: 1, host_abi_version: sys::ABI_VERSION };
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

    // The NEC DSP named from the header and title by fullsnes's list of games; the boot ROM always.
    #[test]
    fn the_firmware_is_named_from_the_header_and_title() {
        assert_eq!(named(b"SUPER MARIO WORLD", 0x02), ["1 spc700.rom 64"]);
        assert_eq!(named(b"PILOTWINGS", 0x03), ["1 spc700.rom 64", "2 dsp1.rom 8192"]);
        assert_eq!(named(b"SUPER MARIO KART", 0x05)[1], "2 dsp1b.rom 8192");
        assert_eq!(named(b"DUNGEON MASTER", 0x03)[1], "2 dsp2.rom 8192");
        assert_eq!(named(b"TOP GEAR 3000", 0x03)[1], "2 dsp4.rom 8192");
        assert_eq!(named(b"F1 ROC II", 0xF6)[1], "2 st010.rom 53248");
        assert_eq!(named(b"MORITA SHOGI", 0xF6)[1], "2 st011.rom 53248");
    }

    // A DSP's firmware whole as file 2, or as its program in file 2 and its data in file 3; the data alone is refused.
    #[test]
    fn a_dsp_firmware_comes_whole_or_in_two_parts() {
        let mut image = crate::machine::tests::rom(&[0x80, 0xFE]);
        image[0x7FD6] = 0x03;
        let ipl = crate::apu::smp::tests::idle_ipl();
        let (program, data) = (vec![0u8; 6_144], vec![0u8; 2_048]);
        let make = |files: Vec<File<'_>>| <Machine as Core>::create(&Create { image: &image, settings: Settings::default(), files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION });
        let whole = [program.as_slice(), data.as_slice()].concat();
        assert!(make(vec![File { which: 1, data: &ipl }, File { which: 2, data: &whole }]).unwrap().sys.cart.dsp.is_some());
        assert!(make(vec![File { which: 1, data: &ipl }, File { which: 2, data: &program }, File { which: 3, data: &data }]).unwrap().sys.cart.dsp.is_some());
        assert_eq!(make(vec![File { which: 1, data: &ipl }, File { which: 3, data: &data }]).err(), Some(status::BAD_FILE));
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
