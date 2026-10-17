//! Diamond on the core ABI v1: `Core` over the machine. The descriptors are the console's facts of Diamond_Plan.md §4;
//! Diamond_Native.md §2 records what the stub answers. The exports are `diamond-v1`'s, which holds this machine with its
//! enhancements (Diamond_Enhancements.md §24.1).

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

/// The first companion file of an image set, the header's `EMUSEN_FILE_MEDIA` since 1.1: file `MEDIA_FILE + n` is the
/// sheet's nth file, then an `.sbi` found beside the image (EmuSen_CoreAPI_Extensions.md §2.4).
pub const MEDIA_FILE: u32 = flags::FILE_MEDIA;

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

const BIOS_COST: &str = "Most games start and save as on a console. A game that relies on an undocumented kernel call, reads the kernel's code, or depends on its exact timing may behave differently, and the start-up screen is EmuSen's own. Games that use the console's own text characters show the Latin letters, digits and common punctuation in EmuSen's own font and a box for every other character (mostly Japanese ones), as EmuSen does not carry Sony's fonts.";

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
/// The cards' medium and their slots' words in core info (since 1.8).
pub const CARD_MEDIUM: &str = "psx.card";
pub const CARD_LABELS: [&str; 2] = ["Slot 1", "Slot 2"];

/// The extensions the core opens, the system pack's checked subset: the programs, a cue sheet with its tracks, an
/// `.iso`, a lone raw track and a CHD; the pack's `.m3u` the runtime expands and never passes (Diamond_Players.md §2).
pub const EXTENSIONS: [&str; 6] = [".psexe", ".psx", ".cue", ".bin", ".iso", ".chd"];

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

/// An enhancement's values: a switch, off the hardware's, or a choice whose first value is the hardware's.
pub enum Values {
    Switch,
    Choice(&'static [(&'static str, &'static str)]),
}

/// An enhancement as the schema declares it (Diamond_Enhancements.md §5): its key and words, its values and what it
/// costs in a player's words. Its setting is in the console's "Enhancements" group with `effect: enhancement`, the
/// hardware's value its default by construction, and is applied between frames.
pub struct Enhancement {
    pub key: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub cost: &'static str,
    pub values: Values,
}

pub const ENHANCEMENTS_CATEGORY: &str = "Enhancements";

impl Enhancement {
    pub fn setting(&self) -> Setting {
        let (kind, hardware) = match self.values {
            Values::Switch => (SettingKind::Switch, "false"),
            Values::Choice(values) => (SettingKind::Choice(values.iter().map(|&(v, l)| choice(v, l)).collect()), values.first().map_or("", |v| v.0)),
        };
        Setting {
            key: self.key.into(),
            label: self.label.into(),
            help: self.help.into(),
            kind,
            default: hardware.into(),
            scope: Scope::Run,
            category: Some(ENHANCEMENTS_CATEGORY.into()),
            effect: Effect::Enhancement { cost: self.cost.into(), hardware: hardware.into() },
            advanced: false,
            hidden: false,
            restart: false,
        }
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

/// The disc an image and its companions make, or the status and words of its refusal, as at create.
/// The image and its companions are kept with `keep`, which borrows where the host lends them (BORROW).
fn open_disc(image: &[u8], files: &[emusen_native::abi::File<'_>]) -> Result<(crate::medium::Opened<Kept>, Media), i32> {
    let mut companions: Vec<(u32, &[u8])> = files.iter().filter(|f| f.which >= MEDIA_FILE).map(|f| (f.which, f.data)).collect();
    companions.sort_by_key(|c| c.0);
    let companions: Vec<Kept> = companions.into_iter().map(|c| keep(c.1)).collect();
    // A program is no disc and keeps nothing here; a sheet's text is parsed and dropped, so only a lone image is kept.
    let kept = if image.starts_with(crate::media::EXE_MAGIC) || crate::medium::is_sheet(image) { Kept::from(image.to_vec()) } else { keep(image) };
    match crate::medium::open(kept, companions) {
        Ok(crate::medium::Opened::Program) => Ok((crate::medium::Opened::Program, Media::read(image).map_err(|_| STATUS_NOT_PLAYSTATION)?)),
        Ok(crate::medium::Opened::Disc(d, m)) => Ok((crate::medium::Opened::Disc(d, m.clone()), m)),
        Err(crate::medium::DiscRefusal::Media(Refusal::Truncated { needs, has })) => {
            detail(&format!("the PS-EXE is {has} bytes; its header and the size it states take {needs}"));
            Err(STATUS_EXE_TRUNCATED)
        }
        Err(crate::medium::DiscRefusal::Media(Refusal::NotPlayStation)) => {
            detail(&format!("the {}-byte image is neither a PS-EXE nor a PlayStation disc", image.len()));
            Err(STATUS_NOT_PLAYSTATION)
        }
        Err(crate::medium::DiscRefusal::Missing { needs, has }) => {
            detail(&format!("the cue sheet names {needs} track files and {has} were given"));
            Err(STATUS_TRACKS_MISSING)
        }
        Err(crate::medium::DiscRefusal::Bad(words)) => {
            detail(&words);
            Err(STATUS_BAD_DISC)
        }
    }
}

pub use crate::cdrom::DRIVE;

impl Core for Machine {
    const CAPABILITIES: u64 = caps::AXES | caps::BATTERY_DIRTY | caps::DEBUG | caps::DEBUG_STACK | caps::DEBUG_REGISTERS | caps::DEBUG_DISASSEMBLE | caps::DEBUG_READS | caps::MEDIA | caps::BORROW;

    fn info() -> Info {
        Info {
            id: "diamond".into(),
            name: "Diamond".into(),
            display_name: Some("Diamond".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("The Sony PlayStation in Rust, written from hardware documents and graded by test programs. At this stage the processor, the GTE, memory, timers, interrupts, DMA, the GPU, the CD-ROM drive, the SPU and the MDEC run a program or a disc with its picture and its sound, the controllers and memory cards answer on SIO0, and the disc in the drive changes through its lid.".into()),
            systems: vec![emusen_native::core::System {
                id: "psx".into(),
                name: "Sony PlayStation".into(),
                extensions: EXTENSIONS.iter().map(|&e| e.into()).collect(),
                regions: vec![Region::Ntsc, Region::Pal],
                controllers: controllers(),
                firmware: vec![bios(Market::Japan), bios(Market::Americas), bios(Market::Europe)],
                development: false,
            }],
            deterministic: true,
            ..Info::default()
        }
    }

    fn settings_schema() -> Vec<Setting> {
        schema()
    }

    /// Both cards' files, whatever the settings: a card's file given for an empty slot is ignored. Each is removable,
    /// a card the player carries, which a state load leaves when the host keeps it apart (Diamond_Cards.md §6); since
    /// 1.8 each is a slot of the medium `psx.card`, labelled by its slot (EmuSen_SaveMedia.md §7.1).
    fn battery_files(system: &str) -> Option<Vec<Battery>> {
        (system == "psx").then(|| (0..2).map(|n| Battery::removable(CARD_FILES[n], CARD_SUFFIXES[n]).medium(CARD_MEDIUM).label(CARD_LABELS[n])).collect())
    }

    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        if crate::medium::is_sheet(image) {
            return vec![bios(Market::Japan), bios(Market::Americas), bios(Market::Europe)];
        }
        match Media::read(image) {
            Ok(m) => vec![bios(market_of(&m, None))],
            Err(_) => match crate::medium::open(image, Vec::new()) {
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
        let (media, disc) = match open_disc(request.image, &request.files)? {
            (crate::medium::Opened::Program, m) => (m, None),
            (crate::medium::Opened::Disc(d, _), m) => (m, Some(d)),
        };
        Machine::make(request, media, disc)
    }

    /// Diamond_Shell.md §1.2: the console powers on with an empty drive and its kernel finds no disc.
    fn starts_empty(system: &str) -> bool {
        system == "psx"
    }

    /// With no disc to name the market, the player's BIOS that was given names it (the Americas', then Europe's, then
    /// Japan's), else the Americas; the Region setting overrules either (Diamond_Shell.md §1.2).
    fn create_empty(_system: &str, request: &Create<'_>) -> Result<Self, i32> {
        let given = |m: Market| request.files.iter().any(|f| f.which == bios(m).which && f.data.len() == BIOS_BYTES);
        let market = [Market::Americas, Market::Europe, Market::Japan].into_iter().find(|&m| given(m));
        Machine::make(request, Media { kind: crate::media::Kind::Empty, market }, None)
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
            spaces: std::iter::once(Space { read_only: true, reports_stores: true, reports_loads: true, view: true, ..Space::new(R3000BUS_ID, "R3000BUS") })
                .chain(SPACES.iter().map(|&(id, name, _, read_only)| Space { read_only, cheats: id == RAM_ID, reports_stores: id == RAM_ID || id == SCRATCH_ID, reports_loads: id == RAM_ID || id == SCRATCH_ID, ..Space::new(id, name) }))
                .collect(),
            processors: vec![
                Processor { id: R3000, name: "R3000".into(), pc_bits: 32, registers: registers(&blackmoon_r3000::DEBUG_REGISTERS), code_space: Some(R3000BUS_ID) },
                Processor { id: 1, name: "GTE".into(), pc_bits: 0, registers: registers(&[blackmoon_gte::DATA_REGISTERS, blackmoon_gte::CONTROL_REGISTERS].concat()), code_space: None },
            ],
            battery: (0..2).filter(|&n| self.card_in(n).is_some()).map(|n| Battery::removable(CARD_FILES[n], CARD_SUFFIXES[n])).collect(),
            state: StateFormat { format: "DMND".into(), version: STATE_VERSION as i64, loads_from: crate::state::LOADS_FROM.iter().map(|&v| v as i64).collect() },
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

    fn settle(&mut self) {
        Machine::settle(self);
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

    fn drives(&self) -> Vec<Drive> {
        let c = &self.board.cdrom;
        vec![Drive { drive: DRIVE, label: "CD-ROM drive".into(), kind: "cd".into(), lid_open: c.lid_open, loaded: c.disc.is_some() }]
    }

    fn media_lid(&mut self, drive: u32, open: bool) -> Result<(), i32> {
        if drive != DRIVE {
            return Err(status::NO_SUCH_DRIVE);
        }
        self.set_lid(open, None);
        Ok(())
    }

    fn media_insert(&mut self, drive: u32, insert: &Insert<'_>) -> Result<(), i32> {
        if drive != DRIVE {
            return Err(status::NO_SUCH_DRIVE);
        }
        if !self.board.cdrom.lid_open && !insert.restore {
            detail("the CD-ROM drive's lid is closed; open it before changing the disc");
            return Err(status::LID_CLOSED);
        }
        let disc = match insert.image {
            None => None,
            Some(image) => match open_disc(image, &insert.files)? {
                (crate::medium::Opened::Disc(d, _), _) => Some(*d),
                (crate::medium::Opened::Program, _) => {
                    detail("a PS-EXE is a program, not a disc; it goes in no drive");
                    return Err(STATUS_NOT_PLAYSTATION);
                }
            },
        };
        self.change_disc(disc);
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
    /// The machine for `media`, with `disc` in its drive or none: the files checked, the settings read, the kernel
    /// chosen (the player's BIOS of the market where given, else the open kernel), the cards and the multitaps.
    fn make(request: &Create<'_>, media: Media, disc: Option<Box<crate::medium::Disc>>) -> Result<Machine, i32> {
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
        m.cards_apart = request.shared_removable;
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{BIOS_ID, CYCLES_PER_SAMPLE, standard_rate};
    use crate::media::sample::{disc, exe};

    fn create(image: &[u8], settings: &[(&str, &str)], files: Vec<emusen_native::abi::File<'_>>) -> Result<Machine, i32> {
        let settings = Settings::from_pairs(settings.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect());
        Machine::create(&Create { image, settings, files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None })
    }

    fn create_empty(settings: &[(&str, &str)], files: Vec<emusen_native::abi::File<'_>>) -> Result<Machine, i32> {
        let settings = Settings::from_pairs(settings.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect());
        Machine::create_empty("psx", &Create { image: &[], settings, files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None })
    }

    /// Diamond_Shell.md §1.2: started empty, the drive is listed empty with its lid closed, the open kernel finds no
    /// disc and draws its screen; with no disc the player's BIOS that was given names the market, the Americas' first,
    /// and the Region setting overrules it.
    #[test]
    fn the_console_starts_empty_and_its_market_comes_from_the_bios_given() {
        assert!(Machine::starts_empty("psx") && !Machine::starts_empty("x"));
        let mut m = create_empty(&[], Vec::new()).unwrap();
        assert_eq!(m.drives(), vec![Drive { drive: DRIVE, label: "CD-ROM drive".into(), kind: "cd".into(), lid_open: false, loaded: false }]);
        let info = m.machine_info();
        assert_eq!((info.region, info.firmware), (Some(Region::Ntsc), vec![(BIOS_AMERICAS, FirmwareSource::Replacement)]));
        for _ in 0..420 {
            m.advance();
        }
        let at = |x: usize, y: usize| u16::from_le_bytes([m.board.gpu.vram.bytes[2 * (y * 1024 + x)], m.board.gpu.vram.bytes[2 * (y * 1024 + x) + 1]]);
        assert_eq!(at(2, 120), 1 | 1 << 5 | 2 << 10, "the open kernel's shell, its background");
        let image = vec![0x42u8; BIOS_BYTES];
        let file = |which| emusen_native::abi::File { which, data: &image };
        let market = |settings: &[(&str, &str)], files| {
            let m = create_empty(settings, files).unwrap();
            (m.config.market, m.machine_info().firmware)
        };
        assert_eq!(market(&[], vec![file(BIOS_JAPAN)]), (Market::Japan, vec![(BIOS_JAPAN, FirmwareSource::File)]));
        assert_eq!(market(&[], vec![file(BIOS_EUROPE), file(BIOS_JAPAN)]), (Market::Europe, vec![(BIOS_EUROPE, FirmwareSource::File)]));
        assert_eq!(market(&[], vec![file(BIOS_JAPAN), file(BIOS_AMERICAS), file(BIOS_EUROPE)]).0, Market::Americas);
        assert_eq!(market(&[(REGION_KEY, "psx.us")], vec![file(BIOS_EUROPE)]), (Market::Americas, vec![(BIOS_AMERICAS, FirmwareSource::Replacement)]));
        assert!(create_empty(&[], vec![file(9)]).is_err(), "a file Diamond does not name is refused as at any create");
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

    /// Diamond_Enhancements.md §5: an enhancement declared in either form is a setting the kit's C3 accepts, its
    /// default the hardware's, its cost given, in the console's "Enhancements" group, applied between frames; one whose
    /// cost is missing is refused by the same rules; and the machine's own schema offers none, the enhancements being
    /// `diamond-v1`'s (§24.2).
    #[test]
    fn an_enhancement_is_declared_with_the_hardwares_value_its_default_and_the_machine_offers_none() {
        assert!(!schema().iter().any(|s| matches!(s.effect, Effect::Enhancement { .. })), "the machine's schema offers no enhancement");
        let forms = [
            Enhancement { key: "e1.switch", label: "A switch", help: "Off is the console.", cost: "Twice the work.", values: Values::Switch },
            Enhancement { key: "e1.choice", label: "A choice", help: "1x is the console.", cost: "Four times the drawing at 2x.", values: Values::Choice(&[("1", "1x (the console's)"), ("2", "2x")]) },
        ];
        for e in &forms {
            let s = e.setting();
            assert!(s.check().is_ok(), "{}: {:?}", s.key, s.check());
            let Effect::Enhancement { hardware, cost } = &s.effect else { panic!("{}: an enhancement", s.key) };
            assert_eq!((hardware.as_str(), s.default.as_str(), cost.as_str(), s.category.as_deref(), s.scope), ([("e1.switch", "false"), ("e1.choice", "1")].iter().find(|k| k.0 == s.key).unwrap().1, hardware.as_str(), e.cost, Some(ENHANCEMENTS_CATEGORY), Scope::Run));
        }
        let uncosted = Enhancement { cost: " ", ..forms.into_iter().next().unwrap() }.setting();
        assert!(uncosted.check().unwrap_err().contains("cost"));
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
        assert_eq!((psx.id.as_str(), psx.extensions.len(), psx.development), ("psx", 6, false));
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

    /// Since 1.3 (EmuSen_CoreAPI.md §6.3): core info lists both cards' files, so a host passes them at the first create,
    /// and a card's file given for a slot the settings leave empty is taken and ignored rather than refused.
    #[test]
    fn core_info_lists_both_cards_and_an_empty_slots_file_is_ignored() {
        let listed: Vec<(u32, String)> = Machine::battery_files("psx").unwrap().into_iter().map(|b| (b.which, b.suffix)).collect();
        assert_eq!(listed, [(0, ".mcr".to_owned()), (4, ".2.mcr".to_owned())]);
        assert!(Machine::battery_files("psx").unwrap().iter().all(|b| b.kind == BatteryKind::Removable), "cards a player carries (Diamond_Cards.md §6)");
        let slots: Vec<(Option<String>, Option<String>, Option<u32>)> = Machine::battery_files("psx").unwrap().into_iter().map(|b| (b.medium, b.label, b.port)).collect();
        assert_eq!(slots, [(Some("psx.card".into()), Some("Slot 1".into()), None), (Some("psx.card".into()), Some("Slot 2".into()), None)], "the console's two slots of one medium (EmuSen_SaveMedia.md §7.1)");
        assert!(Machine::battery_files("snes").is_none());
        let card = vec![0x5Au8; CARD_BYTES];
        let m = create(&exe(None, 1), &[], vec![File { which: 0, data: &card }, File { which: 4, data: &card }]).unwrap();
        assert_eq!(m.machine_info().battery.iter().map(|b| (b.which, b.kind)).collect::<Vec<_>>(), [(0, BatteryKind::Removable)], "slot 2 is empty by default");
        assert_eq!(m.battery(0).unwrap().0, &card[..]);
    }

    /// `programs/card-save.s` as a game whose save is `fill` in `sector`, its image told apart by them.
    fn card_game(sector: u32, fill: u8) -> Vec<u8> {
        let mut image = crate::machine::tests::program(include_str!("../programs/card-save.s"));
        image[0x808..0x80C].copy_from_slice(&sector.to_le_bytes());
        image[0x80C..0x810].copy_from_slice(&(fill as u32).to_le_bytes());
        image
    }

    /// A machine for `image` over the card file `card`, kept apart from the game or not.
    fn over_card(image: &[u8], card: &[u8], apart: bool) -> Machine {
        let settings = Settings::from_pairs(Vec::new());
        Machine::create(&Create { image, settings, files: vec![File { which: 0, data: card }], pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: apart, shared_console: false, device: None }).unwrap()
    }

    /// Run until the game has saved and one ID after it is taken.
    fn played(mut m: Machine) -> Machine {
        for _ in 0..60 {
            Core::advance(&mut m, &mut 0).unwrap();
            if m.board.ram_word(0xFFFF0) == 0xD1A0_D00E && m.board.ram_word(0x10_0280) > 0 {
                return m;
            }
        }
        panic!("the save did not finish: {:02X?}", &m.board.ram[0x10_0100..0x10_0110]);
    }

    /// What a host does at a frame's end: the card's file written when the machine says it changed.
    fn host_saves(m: &mut Machine, file: &mut Vec<u8>) {
        let (data, f) = m.battery(0).unwrap();
        if f & flags::BATTERY_CHANGED != 0 {
            *file = data.to_vec();
            m.battery_saved(0).unwrap();
        }
    }

    fn sector(file: &[u8], n: usize) -> &[u8] {
        &file[n * 128..n * 128 + 128]
    }

    /// One card file used by two games: A saves, a state of A is made, B saves, then A's state is loaded and a frame
    /// run, the host writing the card whenever the machine says it changed; the card file at the end, and the machine.
    fn a_then_b_then_as_state(apart: bool) -> (Vec<u8>, Machine) {
        let (a, b) = (card_game(0x100, 0xA5), card_game(0x140, 0xB6));
        let mut file = vec![0u8; CARD_BYTES];
        let mut ma = played(over_card(&a, &file, apart));
        host_saves(&mut ma, &mut file);
        assert!(sector(&file, 0x100).iter().all(|&x| x == 0xA5), "A's save is on the card");
        let mut state = vec![0; ma.state_size()];
        ma.save_state(&mut state).unwrap();
        let mut mb = played(over_card(&b, &file, apart));
        host_saves(&mut mb, &mut file);
        assert!(sector(&file, 0x140).iter().all(|&x| x == 0xB6), "B's save is on the card");
        let mut ma = over_card(&a, &file, apart);
        ma.load_state(&state).unwrap();
        host_saves(&mut ma, &mut file);
        let passes = ma.board.ram_word(0x10_0280);
        while ma.board.ram_word(0x10_0280) == passes {
            Core::advance(&mut ma, &mut 0).unwrap();
            host_saves(&mut ma, &mut file);
        }
        (file, ma)
    }

    /// Diamond_Cards.md §1 and §10.1, the hazard: on the code before the card mode, and per game still, A's state puts
    /// back the card it holds and the host writes it, so B's save is gone though B was never loaded again.
    #[test]
    fn per_game_a_state_puts_its_card_back() {
        let (file, _) = a_then_b_then_as_state(false);
        assert!(sector(&file, 0x140).iter().all(|&x| x == 0), "B's save is gone");
        assert!(sector(&file, 0x100).iter().all(|&x| x == 0xA5));
    }

    /// Diamond_Cards.md §6, the fix: with the cards kept apart, B's save survives A's state, A's with it, and the game
    /// sees the card as newly inserted, its flag 08h in the next ID's reply.
    #[test]
    fn a_shared_cards_save_survives_a_state_of_another_game() {
        let (file, m) = a_then_b_then_as_state(true);
        assert!(sector(&file, 0x140).iter().all(|&x| x == 0xB6), "B's save survives A's state: {:02X?}", &sector(&file, 0x140)[..8]);
        assert!(sector(&file, 0x100).iter().all(|&x| x == 0xA5), "and A's");
        assert_eq!(m.card_in(0).unwrap().data, file, "the machine's card is the file");
        assert_eq!(m.board.ram[0x10_0201], crate::card::FLAG_NEW, "the ID after the load answers the flag of a card just inserted");
    }

    /// Diamond_Cards.md §10.3: per game, a save's states are those of the code before the card mode, byte for byte:
    /// the state after a game's save, after that state is loaded over a blank card, and five frames on.
    #[test]
    fn per_game_states_are_the_ones_before_the_card_mode() {
        fn fnv(b: &[u8]) -> u64 {
            b.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &x| (h ^ x as u64).wrapping_mul(0x100_0000_01B3))
        }
        fn digest(m: &Machine) -> u64 {
            let mut s = vec![0; m.state_size()];
            m.save_state(&mut s).unwrap();
            fnv(&s)
        }
        let game = card_game(0x100, 0xA5);
        let blank = vec![0u8; CARD_BYTES];
        let mut m = create(&game, &[], vec![File { which: 0, data: &blank }]).unwrap();
        while m.board.ram_word(0xFFFF0) != 0xD1A0_D00E || m.board.ram_word(0x10_0280) == 0 {
            Core::advance(&mut m, &mut 0).unwrap();
        }
        let mut s = vec![0; m.state_size()];
        m.save_state(&mut s).unwrap();
        let mut n = create(&game, &[], vec![File { which: 0, data: &blank }]).unwrap();
        n.load_state(&s).unwrap();
        let loaded = digest(&n);
        for _ in 0..5 {
            Core::advance(&mut n, &mut 0).unwrap();
        }
        let got = [fnv(&s), loaded, digest(&n)];
        // A state holds the kernel's RAM: the open kernel of Diamond_Shell.md S3 moved it, and the values of 2026-10-15
        // (8861_35FF_EF13_5A26 twice, 55CD_ED3F_58B6_961E) are still the earlier image's.
        assert_eq!(got, [0x9AD6_40C9_3A24_5911, 0x9AD6_40C9_3A24_5911, 0x4AC8_04C6_8D09_3A71], "re-recorded on the shell's kernel, 2026-10-17");
    }

    /// MEDIA (EmuSen_CoreAPI.md §6.23): a disc changes only through the open lid, or for a state about to load; the
    /// drive is machine info's; a state loads only over the disc it was made on.
    #[test]
    fn a_disc_changes_through_the_lid_and_a_state_keeps_to_its_disc() {
        let (us, jp) = (disc(Market::Americas, Some(2)), disc(Market::Japan, Some(2)));
        let mut m = create(&us, &[], vec![]).unwrap();
        fn insert(image: Option<&[u8]>, restore: bool) -> Insert<'_> {
            Insert { image, files: Vec::new(), restore }
        }
        assert_eq!(m.drives(), vec![Drive { drive: DRIVE, label: "CD-ROM drive".into(), kind: "cd".into(), lid_open: false, loaded: true }]);
        assert_eq!(m.media_insert(DRIVE, &insert(Some(&jp), false)), Err(status::LID_CLOSED));
        assert_eq!(m.media_lid(1, true), Err(status::NO_SUCH_DRIVE));
        let mut state = vec![0; m.state_size()];
        m.save_state(&mut state).unwrap();
        let first = m.disc_print;
        m.media_lid(DRIVE, true).unwrap();
        assert!(m.board.cdrom.lid_open && m.drives()[0].lid_open);
        assert_eq!(m.media_insert(DRIVE, &insert(Some(&exe(None, 1)), false)), Err(STATUS_NOT_PLAYSTATION), "a program is no disc");
        m.media_insert(DRIVE, &insert(Some(&jp), false)).unwrap();
        m.media_lid(DRIVE, false).unwrap();
        assert!(m.disc_print != first && m.disc_print != 0 && !m.board.cdrom.lid_open);
        assert_eq!(emusen_native::ffi::Status::status(&m.load_state(&state).unwrap_err()), status::WRONG_MEDIA);
        m.media_insert(DRIVE, &insert(Some(&us), true)).unwrap();
        assert_eq!(m.disc_print, first);
        m.load_state(&state).unwrap();
        m.media_lid(DRIVE, true).unwrap();
        m.media_insert(DRIVE, &insert(None, false)).unwrap();
        assert_eq!((m.disc_print, m.drives()[0].loaded), (0, false));
        Core::advance(&mut m, &mut 0).unwrap();
        let program = create(&exe(None, 1), &[], vec![]).unwrap();
        assert!(!program.drives()[0].loaded, "a program's console has an empty drive");
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
