//! Diamond on the core ABI v1: `Core` over the stage 1 machine, and `core_exports!`. The descriptors are the console's
//! facts of Diamond_Plan.md §4; Diamond_Native.md §2 records what the stub answers.

use emusen_native::abi::Region;
use emusen_native::core::*;

use crate::bus::BIOS_BYTES;
use crate::kernel::{self, Kernel};
use crate::card::CARD_BYTES;
use crate::debugger::{R3000, R3000BUS_ID, R3000BUS_SIZE};
use crate::machine::{AUDIO_RATE, Config, Gpu, MAX_HEIGHT, MAX_WIDTH, Machine, RAM_ID, SCRATCH_ID, SPACES, WIDTH};
use crate::media::{Market, Media, Refusal};
use crate::state::STATE_VERSION;

/// An image that is neither a PS-EXE nor a PlayStation disc.
pub const STATUS_NOT_PLAYSTATION: i32 = -9;
/// A PS-EXE shorter than its header says.
pub const STATUS_EXE_TRUNCATED: i32 = -10;
/// A cue sheet whose track files were not all passed beside it.
pub const STATUS_TRACKS_MISSING: i32 = -11;
/// A sheet or a disc image that emusen-disc cannot read.
pub const STATUS_BAD_DISC: i32 = -12;

/// The first companion file of an image set, `EMUSEN_FILE_MEDIA` of EmuSen_CoreAPI_Extensions.md §2.4: file
/// `MEDIA_FILE + n` is the sheet's nth file, then an `.sbi` found beside the image.
pub const MEDIA_FILE: u32 = 65_536;

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

pub use crate::pad::{DUALSHOCK, PAD};

fn controllers() -> Vec<Controller> {
    let buttons = |n: usize| BUTTONS[..n].iter().map(|&(bit, c, l)| Button { bit, control: Some(c), label: l.into() }).collect();
    let stick = |axis, control, label: &str| AxisControl { axis, control: Some(control), trigger: false, label: label.into() };
    vec![
        Controller { id: PAD.into(), label: "Digital Controller".into(), ports: vec![0, 1], buttons: buttons(14), axes: Vec::new() },
        Controller {
            id: DUALSHOCK.into(),
            label: "DualShock".into(),
            ports: vec![0, 1],
            buttons: { let mut b: Vec<Button> = buttons(16); b.push(Button { bit: 16, control: None, label: "Analog".into() }); b },
            axes: vec![stick(0, Axis::LeftX, "Left Stick"), stick(1, Axis::LeftY, "Left Stick"), stick(2, Axis::RightX, "Right Stick"), stick(3, Axis::RightY, "Right Stick")],
        },
    ]
}

/// File numbers of the BIOS by market (Diamond_Plan.md §5.9); 0 is the battery.
pub const BIOS_JAPAN: u32 = 1;
pub const BIOS_AMERICAS: u32 = 2;
pub const BIOS_EUROPE: u32 = 3;

const BIOS_COST: &str = "Most games start and save as on a console. A game that relies on an undocumented kernel call, reads the kernel's code, or depends on its exact timing may behave differently, and the start-up screen is EmuSen's own. Until the rest of the kernel exists, only programs loaded directly run without your BIOS; discs and memory cards need your own.";

pub fn bios(market: Market) -> Firmware {
    let (which, name, label) = match market {
        Market::Japan => (BIOS_JAPAN, "scph5500.bin", "The Japanese PlayStation's BIOS"),
        Market::Americas => (BIOS_AMERICAS, "scph5501.bin", "The American PlayStation's BIOS"),
        Market::Europe => (BIOS_EUROPE, "scph5502.bin", "The European PlayStation's BIOS"),
    };
    Firmware { which, name: name.into(), label: label.into(), size: BIOS_BYTES as u64, required: false, parts: vec![vec![name.into()]], replacement: Some(Replacement::Accuracy { cost: BIOS_COST.into() }) }
}

/// Battery files: the card in port 1's slot (or its multitap's slot A), and port 2's; the BIOS takes 1-3.
pub const CARD_FILES: [u32; 2] = [0, 4];
pub const CARD_SUFFIXES: [&str; 2] = [".mcr", ".2.mcr"];

/// The extensions the host routes to the core, the system pack's checked subset: the programs, and since stage 7 the
/// CHD, a disc in one file (Diamond_CDROM.md §8). A sheet, an .iso and a lone .bin open too, by the image's bytes.
pub const EXTENSIONS: [&str; 3] = [".psexe", ".psx", ".chd"];

/// The settings, read at create; none changes what the stub computes but the region's rate and the ports.
pub const REGION_KEY: &str = "region";
pub const GPU_KEY: &str = "gpu";
pub const PAD_KEYS: [&str; 2] = ["pad1", "pad2"];
pub const TAP_KEYS: [&str; 2] = ["tap1", "tap2"];
pub const CARD_KEYS: [&str; 2] = ["card1", "card2"];
pub const MULTITAP: &str = "multitap";
pub const CARD: &str = "card";

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
        setting(TAP_KEYS[0], "Port 1 multitap", "A multitap on port 1, so that four controllers of the port's kind are plugged in; only games made for it look for the others. Takes effect when a game is next loaded.", vec![choice("none", "None"), choice(MULTITAP, "Multitap")], "none", "Controllers"),
        setting(TAP_KEYS[1], "Port 2 multitap", "A multitap on port 2, which a few games need instead of port 1's. Takes effect when a game is next loaded.", vec![choice("none", "None"), choice(MULTITAP, "Multitap")], "none", "Controllers"),
        setting(CARD_KEYS[0], "Memory card in slot 1", "A memory card in slot 1, kept for this game beside its file; with none, the game finds no card and saves nothing. Takes effect when a game is next loaded.", vec![choice(CARD, "Memory card"), choice("none", "None")], CARD, "Memory cards"),
        setting(CARD_KEYS[1], "Memory card in slot 2", "A second memory card in slot 2, kept for this game beside the first. Takes effect when a game is next loaded.", vec![choice("none", "None"), choice(CARD, "Memory card")], "none", "Memory cards"),
    ]
}

fn market_of(media: &Media, setting: Option<&str>) -> Market {
    REGIONS.iter().find(|r| Some(r.0) == setting).map_or(media.market.unwrap_or(Market::Americas), |r| r.2)
}

impl Core for Machine {
    const CAPABILITIES: u64 = caps::AXES | caps::BATTERY_DIRTY | caps::DEBUG | caps::DEBUG_STACK | caps::DEBUG_REGISTERS | caps::DEBUG_DISASSEMBLE;

    fn info() -> Info {
        Info {
            id: "diamond".into(),
            name: "Diamond".into(),
            display_name: Some("Diamond".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("The Sony PlayStation in Rust, written from hardware documents and graded by test programs. At this stage the processor, the GTE, memory, timers, interrupts, DMA, the GPU, the CD-ROM drive, the SPU and the MDEC run a program or a disc with its picture and its sound, and the controllers and memory cards answer on SIO0.".into()),
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
        if crate::medium::is_sheet(image) {
            return vec![bios(Market::Japan), bios(Market::Americas), bios(Market::Europe)];
        }
        match Media::read(image) {
            Ok(m) => vec![bios(market_of(&m, None))],
            Err(_) => match crate::medium::open(image, &[]) {
                Ok(crate::medium::Opened::Disc(_, m)) => vec![bios(market_of(&m, None))],
                _ => Vec::new(),
            },
        }
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_NOT_PLAYSTATION => Some("the image is neither a PS-EXE (\"PS-X EXE\" at its start) nor a PlayStation disc (the licence text in sector 4 of its system area)".into()),
            STATUS_EXE_TRUNCATED => Some("the PS-EXE is shorter than its header and the size it states".into()),
            STATUS_TRACKS_MISSING => Some("the cue sheet names track files that were not given beside it".into()),
            STATUS_BAD_DISC => Some("the cue sheet or the disc image could not be read as a compact disc".into()),
            _ => None,
        }
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        let mut companions: Vec<(u32, &[u8])> = request.files.iter().filter(|f| f.which >= MEDIA_FILE).map(|f| (f.which, f.data)).collect();
        companions.sort_by_key(|c| c.0);
        let companions: Vec<&[u8]> = companions.into_iter().map(|c| c.1).collect();
        let (media, disc) = match crate::medium::open(request.image, &companions) {
            Ok(crate::medium::Opened::Program) => (Media::read(request.image).map_err(|_| STATUS_NOT_PLAYSTATION)?, None),
            Ok(crate::medium::Opened::Disc(d, m)) => (m, Some(d)),
            Err(crate::medium::DiscRefusal::Media(Refusal::Truncated { needs, has })) => {
                detail(&format!("the PS-EXE is {has} bytes; its header and the size it states take {needs}"));
                return Err(STATUS_EXE_TRUNCATED);
            }
            Err(crate::medium::DiscRefusal::Media(Refusal::NotPlayStation)) => {
                detail(&format!("the {}-byte image is neither a PS-EXE nor a PlayStation disc", request.image.len()));
                return Err(STATUS_NOT_PLAYSTATION);
            }
            Err(crate::medium::DiscRefusal::Missing { needs, has }) => {
                detail(&format!("the cue sheet names {needs} track files and {has} were given"));
                return Err(STATUS_TRACKS_MISSING);
            }
            Err(crate::medium::DiscRefusal::Bad(words)) => {
                detail(&words);
                return Err(STATUS_BAD_DISC);
            }
        };
        let request_files: Vec<_> = request.files.iter().filter(|f| f.which < MEDIA_FILE).collect();
        if let Some(f) = request_files.iter().find(|f| f.which > BIOS_EUROPE && !CARD_FILES.contains(&f.which)) {
            detail(&format!("file {} is not one Diamond names", f.which));
            return Err(status::BAD_FILE);
        }
        if let Some(f) = request_files.iter().find(|f| !CARD_FILES.contains(&f.which) && f.data.len() != BIOS_BYTES) {
            detail(&format!("file {} is {} bytes; a BIOS is {BIOS_BYTES}", f.which, f.data.len()));
            return Err(status::BAD_FILE);
        }
        if let Some(f) = request_files.iter().find(|f| CARD_FILES.contains(&f.which) && !f.data.is_empty() && f.data.len() != CARD_BYTES) {
            detail(&format!("file {} is {} bytes; a memory card is {CARD_BYTES}", f.which, f.data.len()));
            return Err(status::BAD_FILE);
        }
        let s = &request.settings;
        let config = Config { market: market_of(&media, s.get(REGION_KEY)), gpu: GPUS.iter().find(|g| Some(g.0) == s.get(GPU_KEY)).map_or(Gpu::Pin208, |g| g.2) };
        let port = |key: &str| PLUGS.iter().find(|p| p.0 == s.get(key).unwrap_or(DUALSHOCK) && p.0 != "none").map(|p| p.0);
        let players = request.files.iter().find(|f| f.which == bios(config.market).which).map(|f| (f.data, Kernel::Player));
        let rom = players.or_else(|| kernel::open_kernel().map(|k| (k, Kernel::Open)));
        let mut m = Machine::new(media, config, [port(PAD_KEYS[0]), port(PAD_KEYS[1])], request.image, rom);
        m.firmware = request_files.iter().filter(|f| !CARD_FILES.contains(&f.which)).map(|f| f.which).collect();
        if let Some(d) = disc {
            m.insert_disc(*d);
        }
        for n in 0..2 {
            if s.get(CARD_KEYS[n]).unwrap_or(if n == 0 { CARD } else { "none" }) == CARD {
                let data = request.files.iter().find(|f| f.which == CARD_FILES[n] && !f.data.is_empty()).map(|f| f.data);
                m.insert_card(n, data);
            }
            if s.get(TAP_KEYS[n]) == Some(MULTITAP) {
                m.plug_multitap(n);
            }
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
            ports: self.host_ports().into_iter().map(|(p, kind)| Port { port: p, controller: kind.map(Into::into) }).collect(),
            spaces: std::iter::once(Space { read_only: true, reports_stores: true, view: true, ..Space::new(R3000BUS_ID, "R3000BUS") })
                .chain(SPACES.iter().map(|&(id, name, _, read_only)| Space { read_only, cheats: id == RAM_ID, reports_stores: id == RAM_ID || id == SCRATCH_ID, ..Space::new(id, name) }))
                .collect(),
            processors: vec![
                Processor { id: R3000, name: "R3000".into(), pc_bits: 32, registers: registers(&blackmoon_r3000::DEBUG_REGISTERS), code_space: Some(R3000BUS_ID) },
                Processor { id: 1, name: "GTE".into(), pc_bits: 0, registers: registers(&[blackmoon_gte::DATA_REGISTERS, blackmoon_gte::CONTROL_REGISTERS].concat()), code_space: None },
            ],
            battery: (0..2).filter(|&n| self.card_in(n).is_some()).map(|n| Battery { which: CARD_FILES[n], suffix: CARD_SUFFIXES[n].into() }).collect(),
            state: StateFormat { format: "DMND".into(), version: STATE_VERSION as i64, loads_from: vec![STATE_VERSION as i64] },
            phases: Vec::new(),
            patches: None,
            skip_rendering_state_neutral: false,
            firmware: vec![(
                which,
                match self.kernel {
                    Kernel::Player => FirmwareSource::File,
                    Kernel::Open => FirmwareSource::Replacement,
                    Kernel::None => FirmwareSource::Absent,
                },
            )],
        }
    }

    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        Machine::advance(self);
        self.report_cards();
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut emusen_native::debug::Hooks> {
        Some(&mut self.debug.hooks)
    }

    /// The R3000A's breakpoints are the shared hooks'; the GTE runs no program to stop in.
    fn debug_breakpoints(&mut self, processor: u32, pairs: &[i32]) -> Result<(), i32> {
        match processor {
            R3000 => {
                self.debug.hooks.set_breakpoints(pairs);
                Ok(())
            }
            _ => Err(status::NOT_SUPPORTED),
        }
    }

    fn debug_run_frame(&mut self, flags: u32, _detail: &mut u64) -> Result<u32, i32> {
        let why = self.run_frame_debug(flags);
        if why == 0 {
            self.report_cards();
        }
        Ok(why)
    }

    fn debug_stopped(&self) -> u32 {
        self.debug.stopped
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        Machine::debug_pc(self, processor)
    }

    fn debug_registers(&self, processor: u32) -> Result<Vec<i64>, i32> {
        Machine::debug_registers(self, processor).ok_or(status::NOT_SUPPORTED)
    }

    fn debug_disassemble(&mut self, processor: u32, space: u32, address: u32, count: u32) -> Result<Vec<Instruction>, i32> {
        if processor != R3000 {
            return Err(status::NOT_SUPPORTED);
        }
        Machine::debug_disassemble(self, processor, space, address, count).ok_or(status::NO_SUCH_SPACE)
    }

    fn set_options(&mut self, flags: u32) {
        self.skip = flags & emusen_native::core::flags::OPTION_SKIP_RENDERING != 0;
    }

    fn frame_count(&self) -> i64 {
        self.frames
    }

    fn frame_info(&self) -> FrameInfo {
        let d = &self.board.gpu.display;
        FrameInfo { aspect_num: 4, aspect_den: 3, ..FrameInfo::rgba(d.width as i32, d.height as i32) }
    }

    fn frame(&self) -> &[u8] {
        self.board.gpu.display.frame()
    }

    fn audio_rate(&self) -> i32 {
        AUDIO_RATE
    }

    fn audio_buffered(&self) -> usize {
        self.board.spu.out.len()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        self.board.spu.out.drain(out, max_frames)
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.board.spu.out.set_limit(samples);
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        if port >= 8 {
            return Err(status::NO_SUCH_PORT);
        }
        if let Some(c) = self.board.sio0.controller(port as usize) {
            let held = c.buttons();
            c.set_buttons((held & !changed) | (mask & changed));
        }
        Ok(())
    }

    fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), i32> {
        if port >= 8 {
            return Err(status::NO_SUCH_PORT);
        }
        if let Some(c) = self.board.sio0.controller(port as usize) {
            c.set_axis(axis, value);
        }
        Ok(())
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        if space == R3000BUS_ID {
            return Ok(R3000BUS_SIZE);
        }
        self.space(space).map(|m| m.len() as i64).ok_or(status::NO_SUCH_SPACE)
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        if space == R3000BUS_ID {
            for (i, b) in out.iter_mut().enumerate() {
                *b = self.board.peek(address.wrapping_add(i as u32));
            }
            return Ok(());
        }
        let m = self.space(space).ok_or(status::NO_SUCH_SPACE)?;
        for (i, b) in out.iter_mut().enumerate() {
            *b = m.get(address as usize + i).copied().unwrap_or(0);
        }
        Ok(())
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        if space == R3000BUS_ID {
            return Err(status::READ_ONLY);
        }
        if self.space(space).is_none() {
            return Err(status::NO_SUCH_SPACE);
        }
        let m = self.space_mut(space).ok_or(status::READ_ONLY)?;
        for (i, &b) in data.iter().enumerate() {
            if let Some(d) = m.get_mut(address as usize + i) {
                *d = b;
            }
        }
        Ok(())
    }

    fn battery_saved(&mut self, which: u32) -> Result<(), i32> {
        let n = CARD_FILES.iter().position(|&w| w == which).ok_or(status::BAD_FILE)?;
        let c = self.board.sio0.card(n).ok_or(status::BAD_FILE)?;
        c.changed = false;
        self.card_reported[n] = false;
        Ok(())
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        if let Some(n) = CARD_FILES.iter().position(|&w| w == which)
            && let Some(c) = self.card_in(n)
        {
            return Ok((&c.data, flags::BATTERY_TRACKED | if c.changed { flags::BATTERY_CHANGED } else { 0 }));
        }
        match which {
            0 => Ok((&[], 0)),
            _ => Err(status::BAD_FILE),
        }
    }
}

impl Machine {
    /// A changed card reported to the host once until it saves it, after each whole frame.
    fn report_cards(&mut self) {
        for (n, &which) in CARD_FILES.iter().enumerate() {
            if self.card_in(n).is_some_and(|c| c.changed) && !self.card_reported[n] {
                self.card_reported[n] = true;
                emit(event::BATTERY, which as i64, 0);
            }
        }
    }
}

emusen_native::core_exports!(Machine; axes, debug, debug_stack, debug_registers, debug_disassemble);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{BIOS_ID, CYCLES_PER_SAMPLE, standard_rate};
    use crate::media::sample::{disc, exe};

    fn create(image: &[u8], settings: &[(&str, &str)], files: Vec<emusen_native::abi::File<'_>>) -> Result<Machine, i32> {
        let settings = Settings::from_pairs(settings.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect());
        Machine::create(&Create { image, settings, files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION })
    }

    #[test]
    fn the_stub_reports_the_image_its_market_and_a_black_picture() {
        let us = create(&exe(Some("North America"), 64), &[], vec![]).unwrap();
        let info = us.machine_info();
        assert_eq!((info.system.as_str(), info.region, info.frame_rate), ("psx", Some(Region::Ntsc), standard_rate(false)));
        assert_eq!(info.spaces.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["R3000BUS", "RAM", "SCRATCH", "VRAM", "SPURAM", "BIOS", "TTY"]);
        assert_eq!(info.processors.iter().map(|p| (p.name.as_str(), p.registers.len())).collect::<Vec<_>>(), [("R3000", 45), ("GTE", 64)]);
        assert_eq!((info.video.base_width, info.video.base_height, info.video.aspect), (320, 240, (4, 3)));
        assert_eq!(info.ports.iter().map(|p| p.controller.as_deref()).collect::<Vec<_>>(), [Some(DUALSHOCK), Some(DUALSHOCK)]);
        assert_eq!(info.firmware, vec![(BIOS_AMERICAS, FirmwareSource::Replacement)]);
        assert!(us.frame().chunks(4).all(|p| p == [0, 0, 0, 255]) && us.frame().len() == 256 * 240 * 4, "the display after GP1(00h), blanked");

        let eu = create(&disc(Market::Europe, Some(2)), &[], vec![]).unwrap();
        assert_eq!((eu.machine_info().region, eu.machine_info().frame_rate, eu.machine_info().video.base_height), (Some(Region::Pal), standard_rate(true), 256));
        assert_eq!(Machine::firmware_for(&disc(Market::Japan, None))[0].name, "scph5500.bin");
        assert_eq!(Machine::firmware_for(&exe(None, 1))[0].name, "scph5501.bin");
        assert!(Machine::firmware_for(&[0; 100]).is_empty());
    }

    #[test]
    fn the_settings_choose_the_region_the_gpu_and_the_ports() {
        let m = create(&disc(Market::Japan, None), &[(REGION_KEY, "psx.eu"), (GPU_KEY, "psx.gpu160"), (PAD_KEYS[0], PAD), (PAD_KEYS[1], "none")], vec![]).unwrap();
        assert_eq!(m.config, Config { market: Market::Europe, gpu: Gpu::Pin160 });
        assert_eq!(m.machine_info().ports.iter().map(|p| p.controller.as_deref()).collect::<Vec<_>>(), [Some(PAD), None]);
        assert_eq!(m.machine_info().firmware, vec![(BIOS_EUROPE, FirmwareSource::Replacement)]);
        assert_eq!(create(&disc(Market::Japan, None), &[(REGION_KEY, "auto")], vec![]).unwrap().config.market, Market::Japan);
        for s in schema() {
            assert!(s.check().is_ok(), "{}", s.key);
        }
    }

    #[test]
    fn a_frame_is_black_and_silent_at_the_spus_rate_with_the_spu_off() {
        let mut m = create(&exe(None, 1), &[], vec![]).unwrap();
        let mut out = vec![1i16; 4000];
        let mut total = 0;
        for _ in 0..60 {
            Core::advance(&mut m, &mut 0).unwrap();
            let n = m.drain_audio(&mut out, usize::MAX);
            assert!(out[..n].iter().all(|&s| s == 0));
            total += n / 2;
        }
        assert_eq!(total as u64, m.board.now / CYCLES_PER_SAMPLE);
        assert!((44_100..=44_400).contains(&total), "{total}");
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
        assert_eq!(m.space(BIOS_ID).unwrap(), &image[..]);
        assert_eq!(m.cpu.regs.pc, 0xBFC0_0000, "a player's BIOS boots from reset");
        assert_eq!(m.machine_info().firmware, vec![(BIOS_JAPAN, FirmwareSource::File)]);
        assert_eq!(m.space_write(BIOS_ID, 0, &[1]), Err(status::READ_ONLY));
        assert_eq!(create(&exe(None, 1), &[], vec![emusen_native::abi::File { which: 4, data: &image }]).err(), Some(status::BAD_FILE));
        assert_eq!(create(&exe(None, 1), &[], vec![emusen_native::abi::File { which: 1, data: &image[..100] }]).err(), Some(status::BAD_FILE));
        m.set_buttons(0, 0b1010, 0b1111).unwrap();
        m.set_buttons(0, 0b0001, 0b0001).unwrap();
        assert_eq!(m.board.sio0.controller(0).unwrap().buttons(), 0b1011);
        assert_eq!(m.set_buttons(8, 1, 1), Err(status::NO_SUCH_PORT));
    }

    #[test]
    fn the_descriptors_are_the_consoles() {
        let info = Machine::info();
        let psx = &info.systems[0];
        assert_eq!((psx.id.as_str(), psx.extensions.len(), psx.development), ("psx", 3, true));
        assert_eq!(psx.controllers.iter().map(|c| (c.buttons.len(), c.axes.len())).collect::<Vec<_>>(), [(14, 0), (17, 4)]);
        assert!(psx.firmware.iter().all(|f| !f.required && f.size == 524_288));
        let mut bits: Vec<u32> = psx.controllers[1].buttons.iter().map(|b| b.bit).collect();
        bits.sort();
        assert_eq!(bits, (0..17).collect::<Vec<_>>(), "the report's sixteen bits and the Analog button, each once");
    }

    /// Diamond_Plan.md §4.4: each card a battery file in the raw 128 KiB form, slot 1's file 0 and slot 2's file 4;
    /// slot 1 holds a card by default and slot 2 none.
    #[test]
    fn the_cards_are_battery_files_tracked_until_saved() {
        let m = create(&exe(None, 1), &[], vec![]).unwrap();
        assert_eq!(m.machine_info().battery.iter().map(|b| (b.which, b.suffix.as_str())).collect::<Vec<_>>(), [(0, ".mcr")]);
        let card = vec![0x5Au8; CARD_BYTES];
        let files = vec![emusen_native::abi::File { which: 0, data: &card }, emusen_native::abi::File { which: 4, data: &card }];
        let mut m = create(&exe(None, 1), &[(CARD_KEYS[1], CARD), (TAP_KEYS[0], MULTITAP)], files).unwrap();
        assert_eq!(m.machine_info().battery.iter().map(|b| b.which).collect::<Vec<_>>(), [0, 4]);
        assert_eq!(m.machine_info().ports.iter().map(|p| p.port).collect::<Vec<_>>(), [0, 1, 2, 3, 4], "port 1's multitap adds three");
        assert_eq!(m.battery(4).unwrap(), (&card[..], flags::BATTERY_TRACKED));
        m.board.sio0.card(0).unwrap().changed = true;
        assert_eq!(m.battery(0).unwrap().1, flags::BATTERY_TRACKED | flags::BATTERY_CHANGED);
        m.battery_saved(0).unwrap();
        assert_eq!(m.battery(0).unwrap().1, flags::BATTERY_TRACKED);
        assert_eq!(m.battery_saved(1), Err(status::BAD_FILE));
        let short = vec![0u8; 100];
        assert_eq!(create(&exe(None, 1), &[], vec![emusen_native::abi::File { which: 0, data: &short }]).err(), Some(status::BAD_FILE));
        let none = create(&exe(None, 1), &[(CARD_KEYS[0], "none")], vec![]).unwrap();
        assert!(none.machine_info().battery.is_empty());
    }

    /// The DualShock's sticks through `set_axis` and its Analog button through bit 16 of `set_buttons`.
    #[test]
    fn the_sticks_and_the_analog_button_reach_the_dualshock() {
        let mut m = create(&exe(None, 1), &[], vec![]).unwrap();
        m.set_buttons(0, 1 << 16, 1 << 16).unwrap();
        m.set_axis(0, 0, 1.0).unwrap();
        m.set_axis(0, 1, -1.0).unwrap();
        assert_eq!(m.board.sio0.controller(0).unwrap().report(), [0x73, 0x5A, 0xFF, 0xFF, 0x80, 0x80, 0xFF, 0x00]);
        assert_eq!(m.set_axis(8, 0, 0.0), Err(status::NO_SUCH_PORT));
    }
}
