//! Endou on the core ABI v1: `Core` over the stage 1 machine, and `core_exports!`. The descriptors are the consoles'
//! facts of Endou_Plan.md §4; Endou_Native.md §2 records what the stub answers.

use emusen_native::abi::Region;
use emusen_native::core::*;

use crate::machine::{AUDIO_RATE, HEIGHT, MAX_HEIGHT, Machine, RAM_ID, WIDTH};
use crate::media::{Console, Market, Media, Refusal};
use crate::state::STATE_VERSION;

/// An image of no bytes, or larger than any board.
pub const STATUS_NO_CARTRIDGE: i32 = -9;

pub const SMS_PAD: &str = "sms.pad";
pub const SG1000_PAD: &str = "sg1000.pad";

/// The pad's bits in port `$DC`'s order, pressed as 1, then the console's Pause and Reset buttons, read on port 0
/// (Endou_Plan.md §4.2); the SG-1000 has a Pause button and no Reset.
const BUTTONS: [(u32, Control, &str); 8] = [
    (0, Control::Up, "Up"),
    (1, Control::Down, "Down"),
    (2, Control::Left, "Left"),
    (3, Control::Right, "Right"),
    (4, Control::B, "1"),
    (5, Control::A, "2"),
    (6, Control::Start, "Pause"),
    (7, Control::Select, "Reset"),
];

fn pad(id: &str, label: &str, buttons: usize) -> Controller {
    Controller {
        id: id.into(),
        label: label.into(),
        ports: vec![0, 1],
        buttons: BUTTONS[..buttons].iter().map(|&(bit, c, l)| Button { bit, control: Some(c), label: l.into() }).collect(),
        axes: Vec::new(),
    }
}

/// The region setting (Endou_Plan.md §5.8): from the header, or a market chosen; Europe alone is PAL.
pub const REGION_KEY: &str = "region";
pub const REGIONS: [(&str, &str, Market); 4] =
    [("sms.jp", "Japan (NTSC)", Market::Japan), ("sms.us", "Americas (NTSC)", Market::Americas), ("sms.eu", "Europe (PAL)", Market::Europe), ("sms.br", "Brazil (NTSC)", Market::Brazil)];

fn schema() -> Vec<Setting> {
    let mut choices = vec![Choice { value: "auto".into(), label: "From the cartridge".into(), help: None }];
    choices.extend(REGIONS.iter().map(|&(v, l, _)| Choice { value: v.into(), label: l.into(), help: None }));
    vec![Setting {
        key: REGION_KEY.into(),
        label: "Region".into(),
        help: "The console's market, which sets the picture's rate, 60 frames a second for NTSC and 50 for PAL. From the cartridge, a game whose header names an export market runs as an American console and any other as a Japanese one. Takes effect when a game is next loaded.".into(),
        kind: SettingKind::Choice(choices),
        default: "auto".into(),
        scope: Scope::Create,
        category: Some("Console".into()),
        effect: Effect::None,
        advanced: false,
        hidden: false,
        restart: true,
    }]
}

/// The market a machine runs as: the setting's, else the header's, else Japan's (Endou_Plan.md §10 Q9).
pub fn market_of(media: &Media, setting: Option<&str>) -> Market {
    REGIONS.iter().find(|r| Some(r.0) == setting).map_or(media.market().unwrap_or(Market::Japan), |r| r.2)
}

fn system(id: &str, name: &str, extension: &str, controller: Controller) -> emusen_native::core::System {
    emusen_native::core::System {
        id: id.into(),
        name: name.into(),
        extensions: vec![extension.into()],
        regions: vec![Region::Ntsc, Region::Pal],
        controllers: vec![controller],
        firmware: Vec::new(),
        development: true,
    }
}

impl Core for Machine {
    const CAPABILITIES: u64 = 0;

    fn info() -> Info {
        Info {
            id: "endou".into(),
            name: "Endou".into(),
            display_name: Some("Endou".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("The Sega Master System and the SG-1000 in Rust, written from hardware documents and graded by test programs. At this stage it reads a cartridge's header and shows a black picture; nothing is emulated yet.".into()),
            systems: vec![
                system("sms", "Sega Master System / Mark III", ".sms", pad(SMS_PAD, "Control Pad", 8)),
                system("sg1000", "Sega SG-1000", ".sg", pad(SG1000_PAD, "Joystick", 7)),
            ],
            deterministic: true,
            ..Info::default()
        }
    }

    fn settings_schema() -> Vec<Setting> {
        schema()
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_NO_CARTRIDGE => Some("the image is empty or larger than any Master System or SG-1000 board (4 MiB)".into()),
            _ => None,
        }
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        let media = match Media::read(request.image) {
            Ok(m) => m,
            Err(Refusal::Empty) => {
                detail("the image has no bytes");
                return Err(STATUS_NO_CARTRIDGE);
            }
            Err(Refusal::TooLarge(n)) => {
                detail(&format!("the image is {n} bytes; no board holds more than 4 MiB"));
                return Err(STATUS_NO_CARTRIDGE);
            }
        };
        if let Some(f) = request.files.iter().find(|f| f.which != 0) {
            detail(&format!("file {} is not one Endou names", f.which));
            return Err(status::BAD_FILE);
        }
        let market = market_of(&media, request.settings.get(REGION_KEY));
        Ok(Machine::new(media, market, request.image))
    }

    fn machine_info(&self) -> MachineInfo {
        let pad = if self.console() == Console::Sms { SMS_PAD } else { SG1000_PAD };
        MachineInfo {
            system: self.console().id().into(),
            region: Some(if self.pal() { Region::Pal } else { Region::Ntsc }),
            frame_rate: self.frame_rate(),
            video: Video { base_width: WIDTH, base_height: HEIGHT, max_width: WIDTH, max_height: MAX_HEIGHT, aspect: (4, 3), formats: vec![pixel::RGBA8888] },
            audio: Audio { rate: AUDIO_RATE, channels: Vec::new() },
            ports: (0..2).map(|p| Port { port: p, controller: Some(pad.into()) }).collect(),
            spaces: self.memories.iter().map(|m| Space { read_only: m.read_only, cheats: m.id == RAM_ID, ..Space::new(m.id, m.name) }).collect(),
            processors: vec![Processor { id: 0, name: "Z80".into(), pc_bits: 16, registers: Vec::new(), code_space: None }],
            battery: Vec::new(),
            state: StateFormat { format: "ENDO".into(), version: STATE_VERSION as i64, loads_from: vec![STATE_VERSION as i64] },
            phases: Vec::new(),
            patches: None,
            skip_rendering_state_neutral: false,
            firmware: Vec::new(),
        }
    }

    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        Machine::advance(self);
        Ok(())
    }

    fn set_options(&mut self, flags: u32) {
        self.skip = flags & emusen_native::core::flags::OPTION_SKIP_RENDERING != 0;
    }

    fn frame_count(&self) -> i64 {
        self.frames
    }

    fn frame_info(&self) -> FrameInfo {
        FrameInfo::rgba(WIDTH as i32, HEIGHT as i32)
    }

    fn frame(&self) -> &[u8] {
        &self.picture
    }

    fn audio_rate(&self) -> i32 {
        AUDIO_RATE
    }

    fn audio_buffered(&self) -> usize {
        self.silence * 2
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        let frames = self.silence.min(max_frames).min(out.len() / 2);
        out[..frames * 2].fill(0);
        self.silence -= frames;
        frames * 2
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.audio_limit = samples;
        self.silence = self.silence.min(samples / 2);
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        let p = self.pads.get_mut(port as usize).ok_or(status::NO_SUCH_PORT)?;
        *p = (*p & !changed) | (mask & changed);
        Ok(())
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        self.memory(space).map(|m| m.bytes.len() as i64).ok_or(status::NO_SUCH_SPACE)
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        let m = self.memory(space).ok_or(status::NO_SUCH_SPACE)?;
        for (i, b) in out.iter_mut().enumerate() {
            *b = m.bytes.get(address as usize + i).copied().unwrap_or(0);
        }
        Ok(())
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        let m = self.memory_mut(space).ok_or(status::NO_SUCH_SPACE)?;
        if m.read_only {
            return Err(status::READ_ONLY);
        }
        for (i, &b) in data.iter().enumerate() {
            if let Some(d) = m.bytes.get_mut(address as usize + i) {
                *d = b;
            }
        }
        Ok(())
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        match which {
            0 => Ok((&[], 0)),
            _ => Err(status::BAD_FILE),
        }
    }
}

emusen_native::core_exports!(Machine;);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{NTSC_FRAME, PAL_FRAME, ROM_ID};
    use crate::media::SG1000_MAX;
    use crate::media::sample::cartridge;

    fn create(image: &[u8], settings: &[(&str, &str)]) -> Result<Machine, i32> {
        let settings = Settings::from_pairs(settings.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect());
        Machine::create(&Create { image, settings, files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None })
    }

    #[test]
    fn the_stub_reports_the_console_its_market_and_a_black_picture() {
        let us = create(&cartridge(0x8000, 4), &[]).unwrap();
        let info = us.machine_info();
        assert_eq!((info.system.as_str(), info.region, info.frame_rate), ("sms", Some(Region::Ntsc), NTSC_FRAME));
        assert_eq!(info.spaces.iter().map(|s| (s.name.as_str(), s.read_only, s.cheats)).collect::<Vec<_>>(), [("RAM", false, true), ("VRAM", false, false), ("CRAM", false, false), ("ROM", true, false)]);
        assert_eq!((info.video.base_width, info.video.base_height, info.video.aspect), (256, 192, (4, 3)));
        assert_eq!(info.ports.iter().map(|p| p.controller.as_deref()).collect::<Vec<_>>(), [Some(SMS_PAD), Some(SMS_PAD)]);
        assert!(us.frame().chunks(4).all(|p| p == [0, 0, 0, 255]));
        let japan = create(&cartridge(0x8000, 3), &[]).unwrap();
        assert_eq!(japan.market, Market::Japan);
        let sg = create(&[0x00; 0x4000], &[]).unwrap();
        assert_eq!((sg.machine_info().system.as_str(), sg.market, sg.machine_info().ports[0].controller.as_deref()), ("sg1000", Market::Japan, Some(SG1000_PAD)));
    }

    #[test]
    fn the_region_setting_chooses_the_market_and_the_rate() {
        let eu = create(&cartridge(0x8000, 3), &[(REGION_KEY, "sms.eu")]).unwrap();
        assert_eq!((eu.market, eu.machine_info().region, eu.machine_info().frame_rate), (Market::Europe, Some(Region::Pal), PAL_FRAME));
        assert_eq!(create(&cartridge(0x8000, 4), &[(REGION_KEY, "auto")]).unwrap().market, Market::Americas);
        assert_eq!(create(&cartridge(0x8000, 4), &[(REGION_KEY, "sms.br")]).unwrap().machine_info().region, Some(Region::Ntsc));
        for s in schema() {
            assert!(s.check().is_ok(), "{}", s.key);
        }
    }

    #[test]
    fn a_frame_is_black_and_silent_at_the_output_rate() {
        let mut m = create(&cartridge(0x8000, 4), &[]).unwrap();
        let mut out = vec![1i16; 4000];
        let mut total = 0;
        for _ in 0..60 {
            Core::advance(&mut m, &mut 0).unwrap();
            let n = m.drain_audio(&mut out, usize::MAX);
            assert!(out[..n].iter().all(|&s| s == 0));
            total += n / 2;
        }
        assert_eq!(total as u128, AUDIO_RATE as u128 * 60 * NTSC_FRAME.1 as u128 / NTSC_FRAME.0 as u128);
        assert_eq!(m.audio_buffered(), 0);
    }

    #[test]
    fn other_images_and_files_are_refused_with_words() {
        assert_eq!(create(&[], &[]).err(), Some(STATUS_NO_CARTRIDGE));
        assert_eq!(create(&vec![0; (4 << 20) + 513], &[]).err(), Some(STATUS_NO_CARTRIDGE));
        assert!(Machine::status_text(STATUS_NO_CARTRIDGE).is_some());
        let image = cartridge(0x8000, 4);
        let file = emusen_native::abi::File { which: 1, data: &image };
        let request = Create { image: &image, settings: Settings::default(), files: vec![file], pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        assert_eq!(Machine::create(&request).err(), Some(status::BAD_FILE));
    }

    // The kit's garbage image is long; a seeded fault that refused short ones by panicking passed it (Endou_Native.md §6).
    #[test]
    fn an_image_of_any_length_up_to_the_largest_board_is_taken_and_runs() {
        for len in [1usize, 0x3FF, 0x2000, SG1000_MAX, SG1000_MAX + 1, 4 << 20] {
            let mut m = create(&vec![0xA5; len], &[]).unwrap_or_else(|e| panic!("{len} bytes refused with {e}"));
            Core::advance(&mut m, &mut 0).unwrap();
            assert_eq!((m.space_size(ROM_ID), m.frame_count()), (Ok(len as i64), 1));
        }
    }

    #[test]
    fn the_cartridge_is_read_only_and_the_pads_keep_their_bits() {
        let mut m = create(&cartridge(0x8000, 4), &[]).unwrap();
        assert_eq!(m.space_write(ROM_ID, 0, &[1]), Err(status::READ_ONLY));
        m.space_write(RAM_ID, 0x1FFF, &[7, 8]).unwrap();
        let mut b = [0u8; 2];
        m.space_read(RAM_ID, 0x1FFF, &mut b).unwrap();
        assert_eq!(b, [7, 0], "past the end reads nothing and writes nowhere");
        m.set_buttons(0, 0b1010_0000, 0b1111_0000).unwrap();
        m.set_buttons(0, 0b0001, 0b0001).unwrap();
        assert_eq!(m.pads[0], 0b1010_0001);
        assert_eq!(m.set_buttons(2, 1, 1), Err(status::NO_SUCH_PORT));
    }

    #[test]
    fn the_descriptors_are_the_consoles() {
        let info = Machine::info();
        let ids: Vec<_> = info.systems.iter().map(|s| (s.id.as_str(), s.extensions.clone(), s.development, s.firmware.len())).collect();
        assert_eq!(ids, [("sms", vec![".sms".to_string()], true, 0), ("sg1000", vec![".sg".to_string()], true, 0)]);
        let sms = &info.systems[0].controllers[0];
        assert_eq!(sms.buttons.iter().map(|b| (b.bit, b.label.as_str())).collect::<Vec<_>>()[4..], [(4, "1"), (5, "2"), (6, "Pause"), (7, "Reset")]);
        assert_eq!(info.systems[1].controllers[0].buttons.len(), 7, "the SG-1000 has no Reset button");
    }
}
