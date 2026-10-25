//! Sapphire on the core ABI v1: `Core` over the stage S1 machine, and `core_exports!`. The descriptors are the
//! console's facts of Sapphire_Plan.md §4; Sapphire_Native.md §2 records what the stub answers.

use emusen_native::abi::Region;
use emusen_native::core::*;

use crate::bios;
use crate::machine::{AUDIO_RATE, BIOS_BYTES, BIOS_ID, Machine, Market, PAL_HEIGHT, RAM_ID, WIDTH};
use crate::media::{Media, Refusal};
use crate::state::STATE_VERSION;

/// An image that is neither an ELF nor an ISO 9660 volume.
pub const STATUS_NOT_PS2: i32 = -9;
/// An ELF that is not a program of the EE.
pub const STATUS_ELF_NOT_EE: i32 = -10;
/// An ELF shorter than its headers and segments say.
pub const STATUS_ELF_TRUNCATED: i32 = -11;
/// A PlayStation disc, which Diamond runs.
pub const STATUS_PLAYSTATION_DISC: i32 = -12;
/// An ISO 9660 volume whose root names no BOOT2 program.
pub const STATUS_NO_BOOT2: i32 = -13;

/// The pad report's bits in the PlayStation's order, which the DualShock 2's digital report keeps [A, read at S25], each
/// with Diamond's canonical control so that bindings carry between the two (Sapphire_Plan.md §4.2).
const BUTTONS: [(u32, Control, &str); 16] = [
    (4, Control::Up, "Up"),
    (6, Control::Down, "Down"),
    (7, Control::Left, "Left"),
    (5, Control::Right, "Right"),
    (12, Control::X, "Triangle"),
    (13, Control::A, "Circle"),
    (14, Control::B, "Cross"),
    (15, Control::Y, "Square"),
    (10, Control::L, "L1"),
    (11, Control::R, "R1"),
    (8, Control::L2, "L2"),
    (9, Control::R2, "R2"),
    (0, Control::Select, "Select"),
    (3, Control::Start, "Start"),
    (1, Control::L3, "L3"),
    (2, Control::R3, "R3"),
];

pub const DUALSHOCK2: &str = "ps2.dualshock2";
pub const DIGITAL: &str = "ps2.digital";

fn controllers() -> Vec<Controller> {
    let buttons = |n: usize| BUTTONS[..n].iter().map(|&(bit, c, l)| Button { bit, control: Some(c), label: l.into() }).collect();
    let stick = |axis, control, label: &str| AxisControl { axis, control: Some(control), trigger: false, label: label.into() };
    vec![
        Controller {
            id: DUALSHOCK2.into(),
            label: "DualShock 2".into(),
            ports: vec![0, 1],
            buttons: buttons(16),
            axes: vec![stick(0, Axis::LeftX, "Left Stick"), stick(1, Axis::LeftY, "Left Stick"), stick(2, Axis::RightX, "Right Stick"), stick(3, Axis::RightY, "Right Stick")],
        },
        Controller { id: DIGITAL.into(), label: "Digital Controller".into(), ports: vec![0, 1], buttons: buttons(14), axes: Vec::new() },
    ]
}

/// File numbers of the BIOS by market, above the sixteen kept for the cards and the console's own memory
/// (Sapphire_Native.md §2.5).
pub const BIOS_JAPAN: u32 = 16;
pub const BIOS_AMERICAS: u32 = 17;
pub const BIOS_EUROPE: u32 = 18;

const BIOS_COST: &str = "No PlayStation 2 program runs yet, with this file or without it. A player's own BIOS is the first path, from the stage that boots it (S24); Sapphire's own open BIOS takes its place later, as a separate track (Sapphire_Plan.md §5.9).";

/// The three markets' BIOS by their Redump names, each the market's newest retail console version, with every retail
/// version's MD5 so that a player's file is found by its content (Sapphire_Native.md §12.3).
pub fn bios(market: Market) -> Firmware {
    let (which, name, label, md5): (_, _, _, &[&str]) = match market {
        Market::Japan => (BIOS_JAPAN, "ps2-0230j-20080220.bin", "PlayStation 2 BIOS (Japan)", &bios::JAPAN),
        Market::Americas => (BIOS_AMERICAS, "ps2-0230a-20080220.bin", "PlayStation 2 BIOS (Americas)", &bios::AMERICAS),
        Market::Europe => (BIOS_EUROPE, "ps2-0230e-20080220.bin", "PlayStation 2 BIOS (Europe)", &bios::EUROPE),
    };
    Firmware {
        which,
        name: name.into(),
        label: label.into(),
        size: BIOS_BYTES as u64,
        required: false,
        parts: vec![vec![name.into()]],
        replacement: Some(Replacement::None { cost: BIOS_COST.into() }),
        md5: md5.iter().map(|&m| m.into()).collect(),
    }
}

/// The extensions the core opens at S1, the system pack's checked subset (Sapphire_Plan.md §11 Q16).
pub const EXTENSIONS: [&str; 1] = [".elf"];

pub const REGION_KEY: &str = "region";
pub const PAD_KEYS: [&str; 2] = ["pad1", "pad2"];
pub const REGIONS: [(&str, &str, Market); 3] = [("ps2.jp", "Japan (NTSC)", Market::Japan), ("ps2.us", "Americas (NTSC)", Market::Americas), ("ps2.eu", "Europe (PAL)", Market::Europe)];
pub const PLUGS: [(&str, &str); 3] = [(DUALSHOCK2, "DualShock 2"), (DIGITAL, "Digital Controller"), ("none", "Nothing")];

fn choice(value: &str, label: &str) -> Choice {
    Choice { value: value.into(), label: label.into(), help: None }
}

fn setting(key: &str, label: &str, help: &str, choices: Vec<Choice>, default: &str, category: &str) -> Setting {
    Setting {
        key: key.into(),
        label: label.into(),
        help: help.into(),
        kind: SettingKind::Choice(choices),
        default: default.into(),
        scope: Scope::Create,
        category: Some(category.into()),
        effect: Effect::None,
        advanced: false,
        hidden: false,
        restart: true,
    }
}

fn schema() -> Vec<Setting> {
    let mut regions = vec![choice("auto", "Automatic")];
    regions.extend(REGIONS.iter().map(|&(v, l, _)| choice(v, l)));
    let pads = |port: usize| PLUGS.iter().filter(|p| port == 1 || p.0 != "none").map(|&(v, l)| choice(v, l)).collect();
    vec![
        setting(REGION_KEY, "Region", "The console's market, which sets the picture's rate, 60 frames a second for NTSC and 50 for PAL, and which BIOS the console has. Automatic runs an American console until the disc's own market is read. Takes effect when a game is next loaded.", regions, "auto", "Console"),
        setting(PAD_KEYS[0], "Port 1 controller", "What is plugged into the port: a DualShock 2, or a controller with buttons alone. Takes effect when a game is next loaded.", pads(0), DUALSHOCK2, "Controllers"),
        setting(PAD_KEYS[1], "Port 2 controller", "What is plugged into the port: a DualShock 2, a controller with buttons alone, or nothing. Takes effect when a game is next loaded.", pads(1), DUALSHOCK2, "Controllers"),
    ]
}

/// The market a machine runs as: the setting's, else the Americas' until the disc's own is read at S29.
pub fn market_of(setting: Option<&str>) -> Market {
    REGIONS.iter().find(|r| Some(r.0) == setting).map_or(Market::Americas, |r| r.2)
}

impl Core for Machine {
    const CAPABILITIES: u64 = 0;

    fn info() -> Info {
        Info {
            id: "sapphire".into(),
            name: "Sapphire".into(),
            display_name: Some("Sapphire".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("The Sony PlayStation 2 in Rust, written from hardware documents and graded by test programs. At this stage it recognises a program or a disc and shows a black picture; nothing is emulated yet.".into()),
            systems: vec![emusen_native::core::System {
                id: "ps2".into(),
                name: "Sony PlayStation 2".into(),
                extensions: EXTENSIONS.iter().map(|&e| e.into()).collect(),
                regions: vec![Region::Ntsc, Region::Pal],
                controllers: controllers(),
                firmware: vec![bios(Market::Japan), bios(Market::Americas), bios(Market::Europe)],
                development: true,
            }],
            deterministic: true,
            ..Info::default()
        }
    }

    fn settings_schema() -> Vec<Setting> {
        schema()
    }

    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        Media::read(image).map(|_| vec![bios(market_of(None))]).unwrap_or_default()
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_NOT_PS2 => Some("the image is neither an ELF program nor a disc image holding an ISO 9660 volume".into()),
            STATUS_ELF_NOT_EE => Some("the ELF is not a program of the PlayStation 2's EE (a 32-bit little-endian MIPS executable)".into()),
            STATUS_ELF_TRUNCATED => Some("the ELF is shorter than its headers and segments say".into()),
            STATUS_PLAYSTATION_DISC => Some("the disc is a PlayStation game, which Diamond runs, not a PlayStation 2 game".into()),
            STATUS_NO_BOOT2 => Some("the disc's SYSTEM.CNF names no PlayStation 2 program (no BOOT2 line)".into()),
            _ => None,
        }
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        let media = match Media::read(request.image) {
            Ok(m) => m,
            Err(Refusal::Unknown) => {
                detail(&format!("the {}-byte image is neither an ELF nor an ISO 9660 disc image", request.image.len()));
                return Err(STATUS_NOT_PS2);
            }
            Err(Refusal::NotEe(what)) => {
                detail(&format!("the image is {what}; Sapphire runs programs of the EE"));
                return Err(STATUS_ELF_NOT_EE);
            }
            Err(Refusal::Truncated { needs, has }) => {
                detail(&format!("the ELF is {has} bytes; its headers and segments take {needs}"));
                return Err(STATUS_ELF_TRUNCATED);
            }
            Err(Refusal::PlayStation) => {
                detail("the disc is a PlayStation game (its SYSTEM.CNF names a BOOT program, or it has no SYSTEM.CNF and carries Sony's licence text or PSX.EXE); open it on Diamond");
                return Err(STATUS_PLAYSTATION_DISC);
            }
            Err(Refusal::NoBoot2(what)) => {
                detail(&format!("the disc holds an ISO 9660 volume, but {what}"));
                return Err(STATUS_NO_BOOT2);
            }
        };
        if let Some(f) = request.files.iter().find(|f| !(BIOS_JAPAN..=BIOS_EUROPE).contains(&f.which)) {
            detail(&format!("file {} is not one Sapphire names", f.which));
            return Err(status::BAD_FILE);
        }
        if let Some(f) = request.files.iter().find(|f| f.data.len() != BIOS_BYTES) {
            detail(&format!("file {} is {} bytes; a BIOS is {BIOS_BYTES}", f.which, f.data.len()));
            return Err(status::BAD_FILE);
        }
        let s = &request.settings;
        let market = market_of(s.get(REGION_KEY));
        let port = |key: &str| PLUGS.iter().find(|p| p.0 == s.get(key).unwrap_or(DUALSHOCK2) && p.0 != "none").map(|p| p.0);
        let mut m = Machine::new(media, market, [port(PAD_KEYS[0]), port(PAD_KEYS[1])]);
        m.firmware = request.files.iter().map(|f| f.which).collect();
        if let Some(file) = request.files.iter().find(|f| f.which == bios(market).which) {
            m.memory_mut(BIOS_ID).expect("the BIOS").bytes.copy_from_slice(file.data);
        }
        Ok(m)
    }

    fn machine_info(&self) -> MachineInfo {
        let which = bios(self.market).which;
        let processor = |id: u32, name: &str, pc_bits: u32| Processor { id, name: name.into(), pc_bits, registers: Vec::new(), code_space: None };
        MachineInfo {
            system: "ps2".into(),
            region: Some(if self.pal() { Region::Pal } else { Region::Ntsc }),
            frame_rate: self.frame_rate(),
            video: Video { base_width: WIDTH, base_height: self.height(), max_width: WIDTH, max_height: PAL_HEIGHT, aspect: (4, 3), formats: vec![pixel::RGBA8888] },
            audio: Audio { rate: AUDIO_RATE, channels: Vec::new() },
            ports: (0..2).map(|p| Port { port: p, controller: self.ports[p as usize].map(str::to_string) }).collect(),
            spaces: self.memories.iter().map(|m| Space { read_only: m.read_only, cheats: m.id == RAM_ID, ..Space::new(m.id, m.name) }).collect(),
            processors: vec![processor(0, "R5900", 32), processor(1, "IOP", 32), processor(2, "VU0", 12), processor(3, "VU1", 14)],
            battery: Vec::new(),
            state: StateFormat { format: "SPH2".into(), version: STATE_VERSION as i64, loads_from: vec![STATE_VERSION as i64] },
            phases: Vec::new(),
            patches: None,
            skip_rendering_state_neutral: false,
            firmware: vec![(which, if self.firmware.contains(&which) { FirmwareSource::File } else { FirmwareSource::Absent })],
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
        FrameInfo::rgba(WIDTH as i32, self.height() as i32)
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

    fn battery(&self, _which: u32) -> Result<(&[u8], u32), i32> {
        Err(status::BAD_FILE)
    }
}

emusen_native::core_exports!(Machine;);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{NTSC_FRAME, PAL_FRAME};
    use crate::media::sample::{cd, dvd, elf, raw, volume};

    fn create_with(image: &[u8], settings: &[(&str, &str)], files: Vec<emusen_native::abi::File<'_>>) -> Result<Machine, i32> {
        let settings = Settings::from_pairs(settings.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect());
        Machine::create(&Create { image, settings, files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None })
    }

    fn create(image: &[u8], settings: &[(&str, &str)]) -> Result<Machine, i32> {
        create_with(image, settings, Vec::new())
    }

    #[test]
    fn a_program_and_each_disc_form_make_a_black_silent_machine() {
        for image in [elf(), cd(), raw(&cd()), dvd()] {
            let mut m = create(&image, &[]).unwrap();
            let info = m.machine_info();
            assert_eq!((info.system.as_str(), info.region, info.frame_rate, info.video.base_height), ("ps2", Some(Region::Ntsc), NTSC_FRAME, 448));
            assert_eq!(info.processors.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["R5900", "IOP", "VU0", "VU1"]);
            assert_eq!(info.spaces.iter().filter(|s| s.cheats).map(|s| s.name.as_str()).collect::<Vec<_>>(), ["RAM"]);
            assert_eq!(info.firmware, [(BIOS_AMERICAS, FirmwareSource::Absent)]);
            let mut out = vec![1i16; 4000];
            let mut total = 0;
            for _ in 0..60 {
                Core::advance(&mut m, &mut 0).unwrap();
                let n = m.drain_audio(&mut out, usize::MAX);
                assert!(out[..n].iter().all(|&s| s == 0));
                total += n / 2;
            }
            assert_eq!(total as u128, AUDIO_RATE as u128 * 60 * NTSC_FRAME.1 as u128 / NTSC_FRAME.0 as u128);
            assert!(m.frame().chunks(4).all(|p| p == [0, 0, 0, 255]));
        }
    }

    #[test]
    fn the_settings_choose_the_market_and_the_ports() {
        let eu = create(&elf(), &[(REGION_KEY, "ps2.eu"), (PAD_KEYS[1], "none")]).unwrap();
        let info = eu.machine_info();
        assert_eq!((eu.market, info.region, info.frame_rate, info.video.base_height), (Market::Europe, Some(Region::Pal), PAL_FRAME, 512));
        assert_eq!(info.ports.iter().map(|p| p.controller.as_deref()).collect::<Vec<_>>(), [Some(DUALSHOCK2), None]);
        assert_eq!(info.firmware, [(BIOS_EUROPE, FirmwareSource::Absent)]);
        for s in schema() {
            assert!(s.check().is_ok(), "{}", s.key);
        }
    }

    #[test]
    fn other_images_and_files_are_refused_with_words() {
        let mut irx = elf();
        irx[16..18].copy_from_slice(&crate::media::ET_IRX.to_le_bytes());
        let e = elf();
        for (image, code) in [
            (Vec::new(), STATUS_NOT_PS2),
            (vec![0xA5; 50_000], STATUS_NOT_PS2),
            (irx, STATUS_ELF_NOT_EE),
            (e[..e.len() - 1].to_vec(), STATUS_ELF_TRUNCATED),
            (volume(20, "BOOT = cdrom:\\SLUS_007.70;1\r\n", true), STATUS_PLAYSTATION_DISC),
            (volume(20, "VMODE = NTSC\r\n", false), STATUS_NO_BOOT2),
        ] {
            assert_eq!(create(&image, &[]).err(), Some(code));
            assert!(Machine::status_text(code).is_some());
        }
        let bios = vec![7u8; BIOS_BYTES];
        let file = |which, data| emusen_native::abi::File { which, data };
        assert_eq!(create_with(&elf(), &[], vec![file(0, &bios)]).err(), Some(status::BAD_FILE));
        assert_eq!(create_with(&elf(), &[], vec![file(BIOS_AMERICAS, &bios[..1000])]).err(), Some(status::BAD_FILE));
        let m = create_with(&elf(), &[], vec![file(BIOS_AMERICAS, &bios)]).unwrap();
        assert_eq!((m.machine_info().firmware, m.memory(BIOS_ID).unwrap().bytes[0]), (vec![(BIOS_AMERICAS, FirmwareSource::File)], 7));
    }

    #[test]
    fn the_bios_is_read_only_and_the_pads_keep_their_bits() {
        let mut m = create(&elf(), &[]).unwrap();
        assert_eq!(m.space_write(BIOS_ID, 0, &[1]), Err(status::READ_ONLY));
        m.space_write(RAM_ID, (32 << 20) - 1, &[7, 8]).unwrap();
        let mut b = [0u8; 2];
        m.space_read(RAM_ID, (32 << 20) - 1, &mut b).unwrap();
        assert_eq!(b, [7, 0], "past the end reads nothing and writes nowhere");
        m.set_buttons(0, 0b1010_0000, 0b1111_0000).unwrap();
        m.set_buttons(0, 0b0001, 0b0001).unwrap();
        assert_eq!(m.pads[0], 0b1010_0001);
        assert_eq!(m.set_buttons(2, 1, 1), Err(status::NO_SUCH_PORT));
    }

    #[test]
    fn the_descriptors_are_the_consoles() {
        let info = Machine::info();
        let ps2 = &info.systems[0];
        assert_eq!((ps2.id.as_str(), ps2.extensions.as_slice(), ps2.development), ("ps2", &[".elf".to_string()][..], true));
        assert_eq!(ps2.firmware.iter().map(|f| (f.which, f.name.as_str(), f.size, f.required)).collect::<Vec<_>>(), [
            (16, "ps2-0230j-20080220.bin", 4 << 20, false),
            (17, "ps2-0230a-20080220.bin", 4 << 20, false),
            (18, "ps2-0230e-20080220.bin", 4 << 20, false),
        ]);
        assert_eq!(ps2.controllers.iter().map(|c| (c.id.as_str(), c.buttons.len(), c.axes.len())).collect::<Vec<_>>(), [(DUALSHOCK2, 16, 4), (DIGITAL, 14, 0)]);
        assert_eq!(Machine::firmware_for(&cd()).iter().map(|f| f.which).collect::<Vec<_>>(), [BIOS_AMERICAS]);
        assert!(Machine::firmware_for(&[1, 2, 3]).is_empty());
    }

    #[test]
    fn each_market_lists_its_retail_images_with_the_canonical_one_first() {
        let lists = [Market::Japan, Market::Americas, Market::Europe].map(|m| bios(m).md5);
        assert_eq!(lists.iter().map(Vec::len).collect::<Vec<_>>(), [27, 15, 16]);
        assert_eq!(lists.iter().map(|l| l[0].as_str()).collect::<Vec<_>>(), ["80ac46fa7e77b8ab4366e86948e54f83", "21038400dc633070a78ad53090c53017", "dc69f0643a3030aaa4797501b483d6c4"]);
        let all: Vec<&String> = lists.iter().flatten().collect();
        assert!(all.iter().all(|m| m.len() == 32 && m.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())));
        assert_eq!(all.iter().collect::<std::collections::HashSet<_>>().len(), all.len(), "no image in two markets or twice in one");
        assert_eq!(Machine::firmware_for(&cd())[0].md5, lists[1]);
    }
}
