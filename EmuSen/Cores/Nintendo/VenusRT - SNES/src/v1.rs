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

/// A NEC DSP cartridge's firmware: which of DSP-1 to DSP-4, ST010 or ST011 the header does not say, so the size is
/// left open (8,192 or 53,248 bytes).
fn dsp() -> Firmware {
    Firmware {
        which: 2,
        name: "dsp.rom".into(),
        label: "The cartridge's NEC DSP program and data: DSP-1 to DSP-4 (8,192 bytes) or ST010/ST011 (53,248)".into(),
        size: 0,
        required: true,
        parts: Vec::new(),
    }
}

impl Core for Machine {
    const CAPABILITIES: u64 = 0;

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
                firmware: vec![ipl(), dsp()],
            }],
            deterministic: true,
            ..Info::default()
        }
    }

    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        let wants_dsp = crate::cart::Cartridge::new(image).is_some_and(|c| c.wants_dsp());
        if wants_dsp { vec![ipl(), dsp()] } else { vec![ipl()] }
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_IMAGE_TOO_SHORT => Some("the image is shorter than one 32 KiB bank after any copier header".into()),
            STATUS_NO_IPL => Some("the sound unit's 64-byte boot ROM, file 1, was not given".into()),
            _ => None,
        }
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        <Machine as NativeCore>::create(request.image, &Settings::default(), &request.files)
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
            patches: None,
            skip_rendering_state_neutral: true,
        }
    }

    fn advance(&mut self, detail: &mut u64) -> Result<(), i32> {
        NativeCore::advance(self, detail)
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
        NativeCore::battery(self, which)
    }
}

emusen_native::core_exports!(Machine;);

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_native::ffi::StateMachine;

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
