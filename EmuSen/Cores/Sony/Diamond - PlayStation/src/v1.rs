//! Diamond on the core ABI v1: `Core` over the stage 1 machine, and `core_exports!`. The descriptors are the console's
//! facts of Diamond_Plan.md §4; Diamond_Native.md §2 records what the stub answers.

use emusen_native::abi::Region;
use emusen_native::core::*;

use crate::machine::{AUDIO_RATE, BIOS_BYTES, BIOS_ID, Config, Gpu, MAX_HEIGHT, MAX_WIDTH, Machine, RAM_ID, WIDTH};
use crate::media::{Market, Media, Refusal};
use crate::state::STATE_VERSION;

/// An image that is neither a PS-EXE nor a PlayStation disc.
pub const STATUS_NOT_PLAYSTATION: i32 = -9;
/// A PS-EXE shorter than its header says.
pub const STATUS_EXE_TRUNCATED: i32 = -10;

/// The pad report's bits (psx-spx "Standard Controllers"), pressed as 1, in the order a bindings screen lists them,
/// each with its canonical control (Diamond_Plan.md §4.2); L3 and R3 are the DualShock's alone.
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

pub const PAD: &str = "psx.pad";
pub const DUALSHOCK: &str = "psx.dualshock";

fn controllers() -> Vec<Controller> {
    let buttons = |n: usize| BUTTONS[..n].iter().map(|&(bit, c, l)| Button { bit, control: Some(c), label: l.into() }).collect();
    let stick = |axis, control, label: &str| AxisControl { axis, control: Some(control), trigger: false, label: label.into() };
    vec![
        Controller { id: PAD.into(), label: "Digital Controller".into(), ports: vec![0, 1], buttons: buttons(14), axes: Vec::new() },
        Controller {
            id: DUALSHOCK.into(),
            label: "DualShock".into(),
            ports: vec![0, 1],
            buttons: buttons(16),
            axes: vec![stick(0, Axis::LeftX, "Left Stick"), stick(1, Axis::LeftY, "Left Stick"), stick(2, Axis::RightX, "Right Stick"), stick(3, Axis::RightY, "Right Stick")],
        },
    ]
}

/// File numbers of the BIOS by market (Diamond_Plan.md §5.9); 0 is the battery.
pub const BIOS_JAPAN: u32 = 1;
pub const BIOS_AMERICAS: u32 = 2;
pub const BIOS_EUROPE: u32 = 3;

const BIOS_COST: &str = "No PlayStation program runs yet, with this file or without it. Diamond's own open BIOS, written from the documents, takes its place when it exists: its kernel at stage 3 and the disc boot, files and memory cards at stage 10 (Diamond_Plan.md §5.9). A player's own BIOS stays the exact path.";

pub fn bios(market: Market) -> Firmware {
    let (which, name, label) = match market {
        Market::Japan => (BIOS_JAPAN, "scph5500.bin", "The Japanese PlayStation's BIOS"),
        Market::Americas => (BIOS_AMERICAS, "scph5501.bin", "The American PlayStation's BIOS"),
        Market::Europe => (BIOS_EUROPE, "scph5502.bin", "The European PlayStation's BIOS"),
    };
    Firmware { which, name: name.into(), label: label.into(), size: BIOS_BYTES as u64, required: false, parts: vec![vec![name.into()]], replacement: Some(Replacement::None { cost: BIOS_COST.into() }) }
}

/// The extensions the core opens at stage 1, the system pack's checked subset (Diamond_Native.md §2.4).
pub const EXTENSIONS: [&str; 2] = [".psexe", ".psx"];

/// The settings, read at create; none changes what the stub computes but the region's rate and the ports.
pub const REGION_KEY: &str = "region";
pub const GPU_KEY: &str = "gpu";
pub const PAD_KEYS: [&str; 2] = ["pad1", "pad2"];

pub const REGIONS: [(&str, &str, Market); 3] = [("psx.jp", "Japan (NTSC)", Market::Japan), ("psx.us", "Americas (NTSC)", Market::Americas), ("psx.eu", "Europe (PAL)", Market::Europe)];
pub const GPUS: [(&str, &str, Gpu); 2] = [("psx.gpu208", "208-pin GPU (later consoles)", Gpu::Pin208), ("psx.gpu160", "160-pin GPU (early consoles)", Gpu::Pin160)];
pub const PLUGS: [(&str, &str); 3] = [(DUALSHOCK, "DualShock"), (PAD, "Digital Controller"), ("none", "Nothing")];

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
    let mut regions = vec![choice("auto", "From the disc or program")];
    regions.extend(REGIONS.iter().map(|&(v, l, _)| choice(v, l)));
    let pads = |port: usize| PLUGS.iter().filter(|p| port == 1 || p.0 != "none").map(|&(v, l)| choice(v, l)).collect();
    vec![
        setting(REGION_KEY, "Region", "The console's market, which sets the picture's rate, 60 frames a second for NTSC and 50 for PAL, and which BIOS the console has. From the disc, a game runs as a console of the market its licence text names; a program that names none runs as an American console. Takes effect when a game is next loaded.", regions, "auto", "Console"),
        setting(GPU_KEY, "Graphics chip", "Which of the two GPU families the console has: the 208-pin chip of the later consoles, or the 160-pin chip of the first ones, which differ in a few drawing details that some programs can tell apart. Takes effect when a game is next loaded.", GPUS.iter().map(|&(v, l, _)| choice(v, l)).collect(), GPUS[0].0, "Console"),
        setting(PAD_KEYS[0], "Port 1 controller", "What is plugged into the port: a DualShock, which starts in digital mode and answers as the digital pad until it is switched, so every game accepts it; or Sony's first, digital controller. Takes effect when a game is next loaded.", pads(0), DUALSHOCK, "Controllers"),
        setting(PAD_KEYS[1], "Port 2 controller", "What is plugged into the port: a DualShock, Sony's digital controller, or nothing. Takes effect when a game is next loaded.", pads(1), DUALSHOCK, "Controllers"),
    ]
}

fn market_of(media: &Media, setting: Option<&str>) -> Market {
    REGIONS.iter().find(|r| Some(r.0) == setting).map_or(media.market.unwrap_or(Market::Americas), |r| r.2)
}

impl Core for Machine {
    const CAPABILITIES: u64 = 0;

    fn info() -> Info {
        Info {
            id: "diamond".into(),
            name: "Diamond".into(),
            display_name: Some("Diamond".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("The Sony PlayStation in Rust, written from hardware documents and graded by test programs. At this stage it recognises a program or a disc and shows a black picture; nothing is emulated yet.".into()),
            systems: vec![emusen_native::core::System {
                id: "psx".into(),
                name: "Sony PlayStation".into(),
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
        Media::read(image).map(|m| vec![bios(market_of(&m, None))]).unwrap_or_default()
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_NOT_PLAYSTATION => Some("the image is neither a PS-EXE (\"PS-X EXE\" at its start) nor a PlayStation disc (the licence text in sector 4 of its system area)".into()),
            STATUS_EXE_TRUNCATED => Some("the PS-EXE is shorter than its header and the size it states".into()),
            _ => None,
        }
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        let media = match Media::read(request.image) {
            Ok(m) => m,
            Err(Refusal::NotPlayStation) => {
                detail(&format!("the {}-byte image is neither a PS-EXE nor a PlayStation disc", request.image.len()));
                return Err(STATUS_NOT_PLAYSTATION);
            }
            Err(Refusal::Truncated { needs, has }) => {
                detail(&format!("the PS-EXE is {has} bytes; its header and the size it states take {needs}"));
                return Err(STATUS_EXE_TRUNCATED);
            }
        };
        if let Some(f) = request.files.iter().find(|f| f.which > BIOS_EUROPE) {
            detail(&format!("file {} is not one Diamond names", f.which));
            return Err(status::BAD_FILE);
        }
        if let Some(f) = request.files.iter().find(|f| f.which != 0 && f.data.len() != BIOS_BYTES) {
            detail(&format!("file {} is {} bytes; a BIOS is {BIOS_BYTES}", f.which, f.data.len()));
            return Err(status::BAD_FILE);
        }
        let s = &request.settings;
        let config = Config { market: market_of(&media, s.get(REGION_KEY)), gpu: GPUS.iter().find(|g| Some(g.0) == s.get(GPU_KEY)).map_or(Gpu::Pin208, |g| g.2) };
        let port = |key: &str| PLUGS.iter().find(|p| p.0 == s.get(key).unwrap_or(DUALSHOCK) && p.0 != "none").map(|p| p.0);
        let mut m = Machine::new(media, config, [port(PAD_KEYS[0]), port(PAD_KEYS[1])]);
        m.firmware = request.files.iter().filter(|f| f.which != 0).map(|f| f.which).collect();
        if let Some(file) = request.files.iter().find(|f| f.which == bios(config.market).which) {
            m.memory_mut(BIOS_ID).expect("the BIOS").bytes.copy_from_slice(file.data);
        }
        Ok(m)
    }

    fn machine_info(&self) -> MachineInfo {
        let which = bios(self.config.market).which;
        let registers = |names: &[&str]| names.iter().map(|&n| (n.to_owned(), 32)).collect();
        MachineInfo {
            system: "psx".into(),
            region: Some(if self.pal() { Region::Pal } else { Region::Ntsc }),
            frame_rate: self.frame_rate(),
            video: Video { base_width: WIDTH, base_height: self.height(), max_width: MAX_WIDTH, max_height: MAX_HEIGHT, aspect: (4, 3), formats: vec![pixel::RGBA8888] },
            audio: Audio { rate: AUDIO_RATE, channels: Vec::new() },
            ports: (0..2).map(|p| Port { port: p as u32, controller: self.ports[p].map(Into::into) }).collect(),
            spaces: self.memories.iter().map(|m| Space { read_only: m.read_only, cheats: m.id == RAM_ID, ..Space::new(m.id, m.name) }).collect(),
            processors: vec![
                Processor { id: 0, name: "R3000".into(), pc_bits: 32, registers: registers(&blackmoon_r3000::DEBUG_REGISTERS), code_space: None },
                Processor { id: 1, name: "GTE".into(), pc_bits: 0, registers: registers(&[blackmoon_gte::DATA_REGISTERS, blackmoon_gte::CONTROL_REGISTERS].concat()), code_space: None },
            ],
            battery: Vec::new(),
            state: StateFormat { format: "DMND".into(), version: STATE_VERSION as i64, loads_from: vec![STATE_VERSION as i64] },
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
    use crate::machine::{NTSC_FRAME, PAL_FRAME};
    use crate::media::sample::{disc, exe};

    fn create(image: &[u8], settings: &[(&str, &str)], files: Vec<emusen_native::abi::File<'_>>) -> Result<Machine, i32> {
        let settings = Settings::from_pairs(settings.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect());
        Machine::create(&Create { image, settings, files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION })
    }

    #[test]
    fn the_stub_reports_the_image_its_market_and_a_black_picture() {
        let us = create(&exe(Some("North America"), 64), &[], vec![]).unwrap();
        let info = us.machine_info();
        assert_eq!((info.system.as_str(), info.region, info.frame_rate), ("psx", Some(Region::Ntsc), NTSC_FRAME));
        assert_eq!(info.spaces.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["RAM", "SCRATCH", "VRAM", "SPURAM", "BIOS"]);
        assert_eq!(info.processors.iter().map(|p| (p.name.as_str(), p.registers.len())).collect::<Vec<_>>(), [("R3000", 45), ("GTE", 64)]);
        assert_eq!((info.video.base_width, info.video.base_height, info.video.aspect), (320, 240, (4, 3)));
        assert_eq!(info.ports.iter().map(|p| p.controller.as_deref()).collect::<Vec<_>>(), [Some(DUALSHOCK), Some(DUALSHOCK)]);
        assert_eq!(info.firmware, vec![(BIOS_AMERICAS, FirmwareSource::Absent)]);
        assert!(us.frame().chunks(4).all(|p| p == [0, 0, 0, 255]) && us.frame().len() == 320 * 240 * 4);

        let eu = create(&disc(Market::Europe, Some(2)), &[], vec![]).unwrap();
        assert_eq!((eu.machine_info().region, eu.machine_info().frame_rate, eu.machine_info().video.base_height), (Some(Region::Pal), PAL_FRAME, 256));
        assert_eq!(Machine::firmware_for(&disc(Market::Japan, None))[0].name, "scph5500.bin");
        assert_eq!(Machine::firmware_for(&exe(None, 1))[0].name, "scph5501.bin");
        assert!(Machine::firmware_for(&[0; 100]).is_empty());
    }

    #[test]
    fn the_settings_choose_the_region_the_gpu_and_the_ports() {
        let m = create(&disc(Market::Japan, None), &[(REGION_KEY, "psx.eu"), (GPU_KEY, "psx.gpu160"), (PAD_KEYS[0], PAD), (PAD_KEYS[1], "none")], vec![]).unwrap();
        assert_eq!(m.config, Config { market: Market::Europe, gpu: Gpu::Pin160 });
        assert_eq!(m.machine_info().ports.iter().map(|p| p.controller.as_deref()).collect::<Vec<_>>(), [Some(PAD), None]);
        assert_eq!(m.machine_info().firmware, vec![(BIOS_EUROPE, FirmwareSource::Absent)]);
        assert_eq!(create(&disc(Market::Japan, None), &[(REGION_KEY, "auto")], vec![]).unwrap().config.market, Market::Japan);
        for s in schema() {
            assert!(s.check().is_ok(), "{}", s.key);
        }
    }

    #[test]
    fn a_frame_is_black_and_silent_at_the_spus_rate() {
        let mut m = create(&exe(None, 1), &[], vec![]).unwrap();
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
    fn other_images_are_refused_with_words() {
        assert_eq!(create(&[], &[], vec![]).err(), Some(STATUS_NOT_PLAYSTATION));
        assert_eq!(create(&[0xA5; 70_000], &[], vec![]).err(), Some(STATUS_NOT_PLAYSTATION));
        let e = exe(None, 0x1000);
        assert_eq!(create(&e[..e.len() / 2], &[], vec![]).err(), Some(STATUS_EXE_TRUNCATED));
        assert!(Machine::status_text(STATUS_NOT_PLAYSTATION).is_some() && Machine::status_text(STATUS_EXE_TRUNCATED).is_some());
    }

    #[test]
    fn the_players_bios_is_held_read_only_and_other_files_are_refused() {
        let image = vec![0x42u8; BIOS_BYTES];
        let mut m = create(&exe(Some("Japan"), 1), &[], vec![emusen_native::abi::File { which: BIOS_JAPAN, data: &image }]).unwrap();
        assert_eq!(m.memory(BIOS_ID).unwrap().bytes, image);
        assert_eq!(m.machine_info().firmware, vec![(BIOS_JAPAN, FirmwareSource::File)]);
        assert_eq!(m.space_write(BIOS_ID, 0, &[1]), Err(status::READ_ONLY));
        assert_eq!(create(&exe(None, 1), &[], vec![emusen_native::abi::File { which: 4, data: &image }]).err(), Some(status::BAD_FILE));
        assert_eq!(create(&exe(None, 1), &[], vec![emusen_native::abi::File { which: 1, data: &image[..100] }]).err(), Some(status::BAD_FILE));
        m.set_buttons(0, 0b1010, 0b1111).unwrap();
        m.set_buttons(0, 0b0001, 0b0001).unwrap();
        assert_eq!(m.pads[0], 0b1011);
        assert_eq!(m.set_buttons(2, 1, 1), Err(status::NO_SUCH_PORT));
    }

    #[test]
    fn the_descriptors_are_the_consoles() {
        let info = Machine::info();
        let psx = &info.systems[0];
        assert_eq!((psx.id.as_str(), psx.extensions.len(), psx.development), ("psx", 2, true));
        assert_eq!(psx.controllers.iter().map(|c| (c.buttons.len(), c.axes.len())).collect::<Vec<_>>(), [(14, 0), (16, 4)]);
        assert!(psx.firmware.iter().all(|f| !f.required && f.size == 524_288));
        let mut bits: Vec<u32> = psx.controllers[1].buttons.iter().map(|b| b.bit).collect();
        bits.sort();
        assert_eq!(bits, (0..16).collect::<Vec<_>>(), "the report's sixteen bits, each once");
    }
}
