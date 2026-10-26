//! Sapphire on the core ABI v1: `Core` over the machine, and `core_exports!`. The descriptors are the console's facts
//! of Sapphire_Plan.md §4; Sapphire_Native.md §2 records the S1 answers and §35 the machine S24 assembled: the
//! PCRTC's picture and the SPU2's sound handed on, the pads and cards on SIO2, the EEPROM as the medium `ps2.nvm`.

use emusen_native::abi::Region;
use emusen_native::core::*;

use crate::bios;
use crate::debugger;
use crate::cdvd::nvm;
use crate::machine::{AUDIO_RATE, BIOS_BYTES, MEMORIES, Machine, Market, RAM_ID};
use crate::pcrtc::{MAX_HEIGHT, MAX_WIDTH};
use crate::media::{DiscMarket, Media, Refusal};
use crate::sio2::card::{CARD_BYTES, MemoryCard};
use crate::state::{OLDEST_STATE_VERSION, STATE_VERSION, STATUS_OTHER_BIOS, STATUS_QUEUE_PAST_STATE};

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
/// A program put in the drive, which takes only discs.
pub const STATUS_NOT_A_DISC: i32 = -16;
/// A disc image whose sheet, tracks or container do not open, or one cut short of its volume.
pub const STATUS_BAD_DISC: i32 = -17;

/// The DVD drive, the console's one, in machine info's `drives` (G10).
pub const DRIVE: u32 = 0;

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

/// The memory cards' battery files, file 0 the card in slot 1 and file 1 in slot 2, their medium and the slots' words
/// in core info (Sapphire_Plan.md §4.4; EmuSen_SaveMedia.md §3.2; Sapphire_Native.md §30.7).
pub const CARD_FILES: [u32; 2] = [0, 1];
pub const CARD_SUFFIXES: [&str; 2] = [".ps2", ".2.ps2"];
pub const CARD_MEDIUM: &str = "ps2.card";
pub const CARD_LABELS: [&str; 2] = ["Slot 1", "Slot 2"];

/// The console's EEPROM, the medium `ps2.nvm` (Sapphire_Plan.md §4.4), a console file kept per console model: files 2,
/// 3 and 4 for the Japanese, American and European consoles (Sapphire_Native.md §35.5).
pub const NVM_FILES: [u32; 3] = [2, 3, 4];
pub const NVM_SUFFIXES: [&str; 3] = [".jp.nvm", ".us.nvm", ".eu.nvm"];
pub const NVM_MEDIUM: &str = "ps2.nvm";
pub const NVM_LABEL: &str = "Inside the console";

fn market_index(m: Market) -> usize {
    match m {
        Market::Japan => 0,
        Market::Americas => 1,
        Market::Europe => 2,
    }
}

fn nvm_battery(n: usize) -> Battery {
    Battery::console(NVM_FILES[n], NVM_SUFFIXES[n]).medium(NVM_MEDIUM).label(NVM_LABEL)
}

/// File numbers of the BIOS by market, above the sixteen kept for the cards and the console's own memory
/// (Sapphire_Native.md §2.5).
pub const BIOS_JAPAN: u32 = 16;
pub const BIOS_AMERICAS: u32 = 17;
pub const BIOS_EUROPE: u32 = 18;

/// The words a host shows when the BIOS is absent, and refuses an empty start with (Sapphire_Native.md §30.15).
pub const BIOS_COST: &str = "Without a BIOS of the console's market the PlayStation 2 runs nothing: it starts to its menu with the player's own BIOS in the firmware folder. Sapphire's own open BIOS, which is to take its place, is not written yet.";

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

/// The extensions the core opens, the system pack's checked subset, `.m3u` being the host's (Sapphire_Native.md §49.2).
pub const EXTENSIONS: [&str; 5] = [".elf", ".iso", ".chd", ".cue", ".bin"];

/// What a player reads of the core: what runs, what does not, and the gates of Sapphire_Plan.md §8 met and not yet met
/// when it was offered (decided 2026-10-26 by the tester, Sapphire_Native.md §49.1).
/// The gates as the accuracy record's notes, one line each (Sapphire_Native.md §49.3).
pub const GATES: &str = "G1 met: ps2autotests' cpu programs equal their texts but D-28's line; the IOP's crate passes the r3000 suite.\nG2 met: 75 of the 78 ps2autotests print their texts whole on the player's BIOS, the three disputes D-28, D-363 and S13's efu.\nG3 not yet: the GS programs equal PCSX2's pictures or are disputes, a second referee lineage owed.\nG4 met: the kit on Jak and Daxter, the others owed to the kit's copies of an image; the fit audit; the CRT profile (Sapphire_Native.md §49).\nG5 not yet: Jak and Daxter and Kingdom Hearts at four anchors each; Dark Cloud 2 at one, its memory card message.\nG6 not yet: 0.4 to 5 frames a second on the desktop against the console's 59.94.\nG7 met: each debugger claim a test (Sapphire_Native.md §41).\nG8 not yet: the clone check and the disputes read by another, at S48.\nG9 not yet: the open BIOS is not written; on the player's BIOS Dark Cloud 2 runs since D-465's fix, past its memory card message unmeasured.\nG10 met: the console starts to its menu with an empty tray.\nG11 met: the memory card manager's PlayStation 2 tab.";

pub const DESCRIPTION: &str = "The Sony PlayStation 2 in Rust, written from hardware documents and graded by test programs. Offered to players on 2026-10-26 before it is fast: it needs the player's own BIOS, runs games at about 0.4 to 5 frames a second on a desktop, and a game on the player's BIOS sees no memory card. Gates met: G1 and G2 (the test programs, their failures disputes), G4 (the frontend), G7 (the debugger), G10 (the start with no disc) and G11 (the memory card manager). Not yet met: G3 (pictures graded against one referee), G5 (the goldens: Dark Cloud 2 at one anchor of three), G6 (speed), G8 (the clean-room check, at S48) and G9 (the open BIOS not written).";

pub const REGION_KEY: &str = "region";
/// The EE's tier: the interpreter, or the recompiler of S36 (Sapphire_Native.md §44).
pub const EE_TIER_KEY: &str = "ee_tier";
pub const TIERS: [(&str, &str); 2] = [("interpreter", "Interpreter"), ("recompiler", "Recompiler")];
/// VU1's tier (Sapphire_Native.md §45.6): the interpreter, or the recompiler, which is exact.
pub const VU1_KEY: &str = "vu1";
/// Idle skipping (S33, Sapphire_Native.md §47): on or off.
pub const IDLE_KEY: &str = "idle_skip";
pub const IDLE_CHOICES: [(&str, &str); 2] = [("on", "On"), ("off", "Off")];
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
        Setting {
            effect: Effect::Exact,
            ..setting(EE_TIER_KEY, "EE processor", "How the main processor's code runs: the interpreter, one instruction at a time, or the recompiler, which compiles its integer code and branches to the host's machine code in blocks on a thread of its own. Exact: frame for frame the interpreter, which is the other choice, slower. Takes effect when a game is next loaded.", TIERS.iter().map(|&(v, l)| choice(v, l)).collect(), "recompiler", "Performance")
        },
        Setting {
            effect: Effect::Exact,
            ..setting(VU1_KEY, "VU1", "How the vector unit that transforms most of a game's geometry runs its microprograms: interpreted, or compiled whole to the host's machine code on a thread of its own, each checked against the unit's memory before it runs. Exact: the recompiler leaves the console as the interpreter does at every point the rest of the machine can see. Takes effect when a game is next loaded.", TIERS.iter().map(|&(v, l)| choice(v, l)).collect(), "recompiler", "Performance")
        },
        Setting {
            effect: Effect::Exact,
            ..setting(IDLE_KEY, "Idle skipping", "Passes the time the processors spend waiting in a loop for something to happen at once, up to the moment it happens, instead of running each turn of the loop. Exact: every picture, sound and saved state is the one the console gives with it off, which is the other choice, slower. Takes effect when a game is next loaded.", IDLE_CHOICES.iter().map(|&(v, l)| choice(v, l)).collect(), "on", "Performance")
        },
    ]
}

/// Whether the settings, or `SAPPHIRE_VU1` for measurement, choose VU1's recompiler, the default since S40's gate was
/// met (decided 2026-10-26 by the tester, Sapphire_Native.md §45.6).
pub fn vu1_recompiler_chosen(setting: Option<&str>) -> bool {
    std::env::var("SAPPHIRE_VU1").ok().as_deref().or(setting) != Some("interpreter")
}

/// Whether the settings, or `EMUSEN_SAPPHIRE_IDLE` for measurement, turn idle skipping on, the default once S33's gate
/// was met (Sapphire_Native.md §47).
pub fn idle_chosen(setting: Option<&str>) -> bool {
    std::env::var("EMUSEN_SAPPHIRE_IDLE").ok().as_deref().or(setting) != Some("off")
}

/// Whether the settings, or `EMUSEN_SAPPHIRE_TIER` for measurement, choose the recompiler, the default since S36's gate
/// was met (decided 2026-10-26 by the tester, Sapphire_Native.md §44.6).
pub fn recompiler_chosen(setting: Option<&str>) -> bool {
    std::env::var("EMUSEN_SAPPHIRE_TIER").ok().as_deref().or(setting) != Some("interpreter")
}

/// The market of the console: the setting's when it names one; on Automatic, the disc's own market where its serial or
/// VMODE names one, since a console boots only its market's discs (Sapphire_Native.md §49.2); else that of the BIOS
/// entry the player's file came in, the Americas' first, so that the BIOS's own market decides the region
/// (Sapphire_Plan.md §5.9); with none, the Americas'.
pub fn console_market(setting: Option<&str>, disc: Option<Market>, files: &[u32]) -> Market {
    if let Some(r) = REGIONS.iter().find(|r| Some(r.0) == setting) {
        return r.2;
    }
    disc.or_else(|| [Market::Americas, Market::Europe, Market::Japan].into_iter().find(|&m| files.contains(&bios(m).which))).unwrap_or(Market::Americas)
}

impl Core for Machine {
    const CAPABILITIES: u64 = caps::AXES | caps::MEDIA | caps::BORROW | caps::DEBUG | caps::DEBUG_STACK | caps::DEBUG_REGISTERS | caps::DEBUG_DISASSEMBLE | caps::DEBUG_READS;

    fn info() -> Info {
        Info {
            id: "sapphire".into(),
            name: "Sapphire".into(),
            display_name: Some("Sapphire".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some(DESCRIPTION.into()),
            systems: vec![emusen_native::core::System {
                id: "ps2".into(),
                name: "Sony PlayStation 2".into(),
                extensions: EXTENSIONS.iter().map(|&e| e.into()).collect(),
                regions: vec![Region::Ntsc, Region::Pal],
                controllers: controllers(),
                firmware: vec![bios(Market::Japan), bios(Market::Americas), bios(Market::Europe)],
                development: false,
            }],
            deterministic: true,
            accuracy: Some(Accuracy { measured_with: "defaults".into(), suite: "ps2autotests on the player's BIOS, the project's programs against the referees, and the conformance kit".into(), notes: GATES.into() }),
            ..Info::default()
        }
    }

    fn settings_schema() -> Vec<Setting> {
        schema()
    }

    /// Both cards' files, each removable, a card the player carries, a slot of the medium `ps2.card`; and the EEPROM of
    /// each console model, the console's own medium `ps2.nvm` (EmuSen_SaveMedia.md §3.2).
    fn battery_files(system: &str) -> Option<Vec<Battery>> {
        (system == "ps2").then(|| {
            let cards = (0..2).map(|n| Battery::removable(CARD_FILES[n], CARD_SUFFIXES[n]).medium(CARD_MEDIUM).label(CARD_LABELS[n]));
            cards.chain((0..3).map(nvm_battery)).collect()
        })
    }

    /// A disc of a known market asks for that market's BIOS; a program, a disc of no known market, and a sheet or CHD
    /// whose disc this call cannot open ask for every market's, the console taking its own (Sapphire_Native.md §49.2).
    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        let every = || [Market::Americas, Market::Europe, Market::Japan].map(bios).to_vec();
        match Media::read(image) {
            Ok(Media::Disc(d)) => d.market().map_or_else(every, |m| vec![bios(market_from(m))]),
            Ok(_) => every(),
            Err(_) if emusen_disc::is_sheet(image) || image.starts_with(CHD_MAGIC) => every(),
            Err(_) => Vec::new(),
        }
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_NOT_PS2 => Some("the image is neither an ELF program nor a disc image holding an ISO 9660 volume".into()),
            STATUS_ELF_NOT_EE => Some("the ELF is not a program of the PlayStation 2's EE (a 32-bit little-endian MIPS executable)".into()),
            STATUS_ELF_TRUNCATED => Some("the ELF is shorter than its headers and segments say".into()),
            STATUS_PLAYSTATION_DISC => Some("the disc is a PlayStation game, which Diamond runs, not a PlayStation 2 game".into()),
            STATUS_NO_BOOT2 => Some("the disc's SYSTEM.CNF names no PlayStation 2 program (no BOOT2 line)".into()),
            STATUS_OTHER_BIOS => Some("the state was made on a console with another BIOS than this one's, or with a BIOS where this console has none, or with none where it has one".into()),
            STATUS_QUEUE_PAST_STATE => Some("a queue inside the console holds more than the state has room for at this instant; the state can be saved a moment later".into()),
            STATUS_NOT_A_DISC => Some("a program is no disc and goes in no drive".into()),
            STATUS_BAD_DISC => Some("the disc image does not open: its sheet, its tracks or its container are not as a disc's, or it holds fewer sectors than its volume says".into()),
            _ => None,
        }
    }

    /// A program waits for the fast boot; a disc is in the drive when the console is switched on (Sapphire_Native.md §49.2).
    fn create(request: &Create<'_>) -> Result<Self, i32> {
        let (media, disc) = judge(request.image, &request.files)?;
        let identity = media.identity();
        let mut m = Machine::make(request, media)?;
        if let Some(d) = disc {
            m.disc_at_power_on(d);
            m.drive_disc = Some(identity);
        }
        Ok(m)
    }

    /// The console with its tray closed and empty powers on to its BIOS's menu (Sapphire_Plan.md §8, G10).
    fn starts_empty(system: &str) -> bool {
        system == "ps2"
    }

    /// With no disc to name the market, the player's BIOS given names it, as `console_market` reads the files. A start
    /// with no BIOS of the market makes a console that runs nothing, which a host refuses by the firmware entry's
    /// replacement, none (Sapphire_Native.md §30.15).
    fn create_empty(_system: &str, request: &Create<'_>) -> Result<Self, i32> {
        Machine::make(request, Media::Empty)
    }

    fn machine_info(&self) -> MachineInfo {
        let which = bios(self.market).which;
        let (_, width, height) = (if self.pal() { crate::clock::PAL } else { crate::clock::NTSC }).picture;
        let processor = |id: u32, name: &str, pc_bits: u32, registers, space| Processor { id, name: name.into(), pc_bits, registers, code_space: Some(space) };
        MachineInfo {
            system: "ps2".into(),
            region: Some(if self.pal() { Region::Pal } else { Region::Ntsc }),
            frame_rate: self.frame_rate(),
            video: Video { base_width: width, base_height: height, max_width: MAX_WIDTH, max_height: MAX_HEIGHT, aspect: (4, 3), formats: vec![pixel::RGBA8888] },
            audio: Audio { rate: AUDIO_RATE, channels: Vec::new() },
            ports: (0..2).map(|p| Port { port: p, controller: self.ports[p as usize].map(str::to_string) }).collect(),
            spaces: MEMORIES
                .iter()
                .map(|&(id, name, _, read_only)| {
                    let watched = id == RAM_ID || id == debugger::SCRATCH_ID;
                    Space { read_only, cheats: id == RAM_ID, reports_stores: watched, reports_loads: watched, ..Space::new(id, name) }
                })
                .collect(),
            processors: vec![
                processor(debugger::EE, "R5900", 32, debugger::ee_registers(), RAM_ID),
                processor(debugger::IOP, "IOP", 32, debugger::iop_registers(), debugger::IOPRAM_ID),
                processor(debugger::VU0, "VU0", 12, debugger::vu_registers(0), debugger::VU0MICRO_ID),
                processor(debugger::VU1, "VU1", 14, debugger::vu_registers(1), debugger::VU1MICRO_ID),
            ],
            battery: (0..2).map(|n| Battery::removable(CARD_FILES[n], CARD_SUFFIXES[n])).chain(std::iter::once(nvm_battery(market_index(self.market)))).collect(),
            state: StateFormat { format: "SPH2".into(), version: STATE_VERSION as i64, loads_from: (OLDEST_STATE_VERSION..=STATE_VERSION).map(i64::from).collect() },
            phases: Vec::new(),
            patches: None,
            skip_rendering_state_neutral: false,
            firmware: vec![(which, if self.firmware.contains(&which) { FirmwareSource::File } else { FirmwareSource::Absent })],
        }
    }

    /// A frame; when the picture's size has changed, the state's length with it, which the host is told as the
    /// picture's own size (Sapphire_Native.md §38.2).
    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        let before = self.sys.board.pcrtc.picture.rgba.len();
        self.disarm();
        Machine::advance(self);
        if self.sys.board.pcrtc.picture.rgba.len() != before {
            emit(event::STATE_SIZE, emusen_native::ffi::StateMachine::state_size(self) as i64, 0);
        }
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut emusen_native::debug::Hooks> {
        Some(&mut self.debug.hooks)
    }

    /// The EE's breakpoints are the shared hooks'; the IOP's and the VUs' go to the probes beside them.
    fn debug_breakpoints(&mut self, processor: u32, pairs: &[i32]) -> Result<(), i32> {
        if Machine::debug_breakpoints(self, processor, pairs) { Ok(()) } else { Err(status::NOT_SUPPORTED) }
    }

    /// A frame through the observed loop; at its end, the state's length told when the picture's size changed.
    fn debug_run_frame(&mut self, flags: u32, _detail: &mut u64) -> Result<u32, i32> {
        let before = self.sys.board.pcrtc.picture.rgba.len();
        let why = self.run_frame_debug(flags);
        if why == 0 && self.sys.board.pcrtc.picture.rgba.len() != before {
            emit(event::STATE_SIZE, emusen_native::ffi::StateMachine::state_size(self) as i64, 0);
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
        if processor > debugger::VU1 {
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
        self.sys.board.pcrtc.picture.frame_info()
    }

    fn frame(&self) -> &[u8] {
        &self.sys.board.pcrtc.picture.rgba
    }

    fn audio_rate(&self) -> i32 {
        AUDIO_RATE
    }

    fn audio_buffered(&self) -> usize {
        self.sys.board.iop.bus.spu2.out.len()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        self.sys.board.iop.bus.spu2.out.drain(out, max_frames)
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.sys.board.iop.bus.spu2.out.set_limit(samples);
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        let p = self.pads.get_mut(port as usize).ok_or(status::NO_SUCH_PORT)?;
        *p = (*p & !changed) | (mask & changed);
        self.sync_pad(port as usize);
        Ok(())
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        MEMORIES.iter().find(|m| m.0 == space).map(|m| m.2 as i64).ok_or(status::NO_SUCH_SPACE)
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        self.space_size(space)?;
        for (i, b) in out.iter_mut().enumerate() {
            *b = self.space_byte(space, address as usize + i).unwrap_or(0);
        }
        Ok(())
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        let m = MEMORIES.iter().find(|m| m.0 == space).ok_or(status::NO_SUCH_SPACE)?;
        if m.3 {
            return Err(status::READ_ONLY);
        }
        for (i, &b) in data.iter().enumerate() {
            self.set_space_byte(space, address as usize + i, b);
        }
        Ok(())
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        if which == NVM_FILES[market_index(self.market)] {
            let n = self.nvm();
            return Ok((&n.bytes[..], flags::BATTERY_TRACKED | if n.dirty { flags::BATTERY_CHANGED } else { 0 }));
        }
        let c = CARD_FILES.iter().position(|&w| w == which).and_then(|n| self.card(n)).ok_or(status::BAD_FILE)?;
        Ok((&c.image, flags::BATTERY_TRACKED | if c.dirty { flags::BATTERY_CHANGED } else { 0 }))
    }

    fn battery_saved(&mut self, which: u32) -> Result<(), i32> {
        if which == NVM_FILES[market_index(self.market)] {
            self.nvm_mut().dirty = false;
            return Ok(());
        }
        let n = CARD_FILES.iter().position(|&w| w == which).ok_or(status::BAD_FILE)?;
        self.card_mut(n).ok_or(status::BAD_FILE)?.dirty = false;
        Ok(())
    }

    fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), i32> {
        let sticks = self.sticks.get_mut(port as usize).ok_or(status::NO_SUCH_PORT)?;
        // The descriptor's axes 0-3 are left X, left Y, right X, right Y; the pad reports right X, right Y, left X, left Y.
        if let Some(&at) = [2usize, 3, 0, 1].get(axis as usize) {
            sticks[at] = crate::sio2::pad::stick(value);
        }
        self.sync_pad(port as usize);
        Ok(())
    }

    fn drives(&self) -> Vec<Drive> {
        let c = &self.sys.board.iop.bus.cdvd;
        vec![Drive { drive: DRIVE, label: "DVD drive".into(), kind: "dvd".into(), lid_open: c.tray_open(), loaded: c.disc().is_some() }]
    }

    fn media_lid(&mut self, drive: u32, open: bool) -> Result<(), i32> {
        if drive != DRIVE {
            return Err(status::NO_SUCH_DRIVE);
        }
        self.set_tray(open);
        Ok(())
    }

    /// A disc goes in through the open tray, judged by the recogniser as an image is at create, then opened as the
    /// drive's medium; nothing, an empty drive (EmuSen_CoreAPI.md §6.23).
    fn media_insert(&mut self, drive: u32, insert: &Insert<'_>) -> Result<(), i32> {
        if drive != DRIVE {
            return Err(status::NO_SUCH_DRIVE);
        }
        let open = self.sys.board.iop.bus.cdvd.tray_open();
        if !open && !insert.restore {
            detail("the DVD drive's tray is closed; open it before changing the disc");
            return Err(status::LID_CLOSED);
        }
        let disc = match insert.image {
            None => None,
            Some(image) => match judge(image, &insert.files)? {
                (media, Some(d)) => Some((media.identity(), d)),
                (_, None) => {
                    detail("the image is an EE program; a program is opened as a game, not put in the drive");
                    return Err(STATUS_NOT_A_DISC);
                }
            },
        };
        // A restore with the tray closed is the host setting the drive for a state, through the tray as a hand would.
        self.set_tray(true);
        self.drive_disc = disc.as_ref().map(|d| d.0);
        self.insert_disc(disc.map(|d| d.1));
        self.set_tray(open);
        Ok(())
    }
}

/// What an image is by the recogniser, or the status and words it is refused with.
fn recognise(image: &[u8]) -> Result<Media, i32> {
    Media::read(image).map_err(|r| refused(r, image.len()))
}

/// A refusal's status, its words given as the call's detail.
fn refused(r: Refusal, bytes: usize) -> i32 {
    match r {
        Refusal::Unknown => {
            detail(&format!("the {bytes}-byte image is neither an ELF nor an ISO 9660 disc image"));
            STATUS_NOT_PS2
        }
        Refusal::NotEe(what) => {
            detail(&format!("the image is {what}; Sapphire runs programs of the EE"));
            STATUS_ELF_NOT_EE
        }
        Refusal::Truncated { needs, has } => {
            detail(&format!("the ELF is {has} bytes; its headers and segments take {needs}"));
            STATUS_ELF_TRUNCATED
        }
        Refusal::PlayStation => {
            detail("the disc is a PlayStation game (its SYSTEM.CNF names a BOOT program, or it has no SYSTEM.CNF and carries Sony's licence text or PSX.EXE); open it on Diamond");
            STATUS_PLAYSTATION_DISC
        }
        Refusal::NoBoot2(what) => {
            detail(&format!("the disc holds an ISO 9660 volume, but {what}"));
            STATUS_NO_BOOT2
        }
        Refusal::ShortDisc { has, needs } => {
            detail(&format!("the disc image holds {has} sectors where its volume says {needs}: the image was cut short"));
            STATUS_BAD_DISC
        }
    }
}

/// A CHD's magic, the container's first eight bytes.
const CHD_MAGIC: &[u8] = b"MComprHD";

/// The market a disc names, as the console's.
fn market_from(m: DiscMarket) -> Market {
    match m {
        DiscMarket::Japan => Market::Japan,
        DiscMarket::Americas => Market::Americas,
        DiscMarket::Europe => Market::Europe,
    }
}

/// What an image is and, for a disc, the drive's medium it makes: a sheet judged by its first track's file, a CHD by the
/// volume on the disc it opens to, any other image by its own bytes; the disc opened from the kept bytes and companions
/// (Sapphire_Native.md §25.2, §49.2). A program makes no disc.
fn judge(image: &[u8], files: &[emusen_native::abi::File<'_>]) -> Result<(Media, Option<crate::cdvd::Disc>), i32> {
    let mut companions: Vec<(u32, &[u8])> = files.iter().filter(|f| f.which >= flags::FILE_MEDIA).map(|f| (f.which, f.data)).collect();
    companions.sort_by_key(|c| c.0);
    let (sheet, chd) = (emusen_disc::is_sheet(image), image.starts_with(CHD_MAGIC));
    let media = if chd {
        None
    } else {
        let media = recognise(if sheet { companions.first().map_or(&[][..], |c| c.1) } else { image })?;
        if matches!(media, Media::Elf(_)) {
            return Ok((media, None));
        }
        Some(media)
    };
    // A sheet's text is parsed and dropped, so only a lone image or a track's file is kept.
    let kept = if sheet { Kept::from(image.to_vec()) } else { keep(image) };
    let mut disc = crate::cdvd::open(kept, companions.into_iter().map(|c| keep(c.1)).collect()).map_err(|e| {
        detail(&format!("the disc does not open: {e:?}"));
        STATUS_BAD_DISC
    })?;
    let media = match media {
        Some(m) => m,
        None => {
            let count = match &disc {
                crate::cdvd::Disc::Dvd(d) => u64::from(d.sectors()),
                _ => 0,
            };
            Media::read_by(&mut |n| crate::cdvd::user_sector(&mut disc, n), count, false).map_err(|r| refused(r, image.len()))?
        }
    };
    if matches!(media, Media::Elf(_)) {
        return Ok((media, None));
    }
    Ok((media, Some(disc)))
}

impl Machine {
    /// The machine for `media`: the files checked, the settings read, the market chosen from the setting or the BIOS
    /// given, the cards, the EEPROM and the BIOS placed.
    fn make(request: &Create<'_>, media: Media) -> Result<Machine, i32> {
        if let Some(f) = request.files.iter().find(|f| !(BIOS_JAPAN..=BIOS_EUROPE).contains(&f.which) && !CARD_FILES.contains(&f.which) && !NVM_FILES.contains(&f.which) && f.which < flags::FILE_MEDIA) {
            detail(&format!("file {} is not one Sapphire names", f.which));
            return Err(status::BAD_FILE);
        }
        if let Some(f) = request.files.iter().find(|f| (BIOS_JAPAN..=BIOS_EUROPE).contains(&f.which) && f.data.len() != BIOS_BYTES) {
            detail(&format!("file {} is {} bytes; a BIOS is {BIOS_BYTES}", f.which, f.data.len()));
            return Err(status::BAD_FILE);
        }
        if let Some(f) = request.files.iter().find(|f| CARD_FILES.contains(&f.which) && !f.data.is_empty() && f.data.len() != CARD_BYTES) {
            detail(&format!("file {} is {} bytes; a PlayStation 2 memory card is {CARD_BYTES}, every page of 512 bytes followed by its 16 spare bytes", f.which, f.data.len()));
            return Err(status::BAD_FILE);
        }
        if let Some(f) = request.files.iter().find(|f| NVM_FILES.contains(&f.which) && !f.data.is_empty() && f.data.len() != nvm::BYTES) {
            detail(&format!("file {} is {} bytes; the console's EEPROM is {}", f.which, f.data.len(), nvm::BYTES));
            return Err(status::BAD_FILE);
        }
        let s = &request.settings;
        let firmware: Vec<u32> = request.files.iter().map(|f| f.which).filter(|w| (BIOS_JAPAN..=BIOS_EUROPE).contains(w)).collect();
        let disc_market = match &media {
            Media::Disc(d) => d.market().map(market_from),
            _ => None,
        };
        let market = console_market(s.get(REGION_KEY), disc_market, &firmware);
        let port = |key: &str| PLUGS.iter().find(|p| p.0 == s.get(key).unwrap_or(DUALSHOCK2) && p.0 != "none").map(|p| p.0);
        let mut m = Machine::new(media, market, [port(PAD_KEYS[0]), port(PAD_KEYS[1])]);
        m.firmware = firmware;
        if recompiler_chosen(s.get(EE_TIER_KEY)) {
            m.blocks = Some(Box::new(crate::blocks::Blocks::from_env()));
        }
        if vu1_recompiler_chosen(s.get(VU1_KEY)) {
            m.sys.board.vpu.set_tier(1, Some(crate::vu::jit::Settings::default()));
        } else {
            m.sys.board.vpu.set_tier(1, None);
        }
        m.idle.on = idle_chosen(s.get(IDLE_KEY));
        // An empty file is a card not yet saved: the slot keeps its erased card.
        for f in request.files.iter().filter(|f| CARD_FILES.contains(&f.which) && !f.data.is_empty()) {
            *m.card_mut(f.which as usize).expect("both slots hold a card") = MemoryCard::new(f.data.to_vec()).expect("the size was checked");
        }
        // An empty or absent file is a console whose EEPROM is not yet saved: it starts blank (Sapphire_Native.md §35.5).
        let nvm_file = NVM_FILES[market_index(market)];
        if let Some(f) = request.files.iter().find(|f| f.which == nvm_file && !f.data.is_empty()) {
            m.nvm_mut().bytes.copy_from_slice(f.data);
        }
        if let Some(file) = request.files.iter().find(|f| f.which == bios(market).which) {
            m.insert_bios(file.data);
            // An EE program is entered by fast boot once the BIOS is up (Sapphire_Native.md §36.2).
            if matches!(m.media, Media::Elf(_)) {
                m.program = Some(request.image.to_vec());
            }
        }
        Ok(m)
    }
}

emusen_native::core_exports!(Machine; axes, media, debug, debug_stack, debug_registers, debug_disassemble, debug_reads);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::BIOS_ID;
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
            let rate = m.frame_rate();
            assert_eq!(rate.0 * 1001, rate.1 * 60_000, "NTSC's 59.94 Hz");
            assert_eq!((info.system.as_str(), info.region, info.frame_rate, info.video.base_height), ("ps2", Some(Region::Ntsc), rate, 448));
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
            let want = AUDIO_RATE as i64 * 60 * rate.1 as i64 / rate.0 as i64;
            assert!((total as i64 - want).abs() <= 32, "{total} samples for {want}");
            assert!(m.frame().chunks(4).all(|p| p == [0, 0, 0, 255]) && m.frame().len() == 640 * 448 * 4);
        }
    }

    #[test]
    fn the_vu1_setting_chooses_its_tier_and_the_recompiler_is_the_default() {
        if std::env::var_os("SAPPHIRE_VU1").is_none() {
            let interpreted = create(&elf(), &[(VU1_KEY, "interpreter")]).unwrap();
            assert!(interpreted.sys.board.vpu.jit_counts(1).is_none());
            assert!(create(&elf(), &[]).unwrap().sys.board.vpu.jit_counts(1).is_some(), "the recompiler without a setting");
        }
        let compiled = create(&elf(), &[(VU1_KEY, "recompiler")]).unwrap();
        assert!(compiled.sys.board.vpu.jit_counts(1).is_some());
        if std::env::var_os("SAPPHIRE_VU0").is_none() {
            assert!(compiled.sys.board.vpu.jit_counts(0).is_none(), "VU0 stays on the interpreter");
        }
        let s = schema().into_iter().find(|s| s.key == VU1_KEY).unwrap();
        assert!(s.default == "recompiler" && matches!(s.effect, Effect::Exact));
    }

    #[test]
    fn the_settings_choose_the_market_and_the_ports() {
        let eu = create(&elf(), &[(REGION_KEY, "ps2.eu"), (PAD_KEYS[1], "none")]).unwrap();
        let info = eu.machine_info();
        assert_eq!((eu.market, info.region, info.frame_rate.0, info.video.base_height), (Market::Europe, Some(Region::Pal), info.frame_rate.1 * 50, 512));
        assert_eq!(info.ports.iter().map(|p| p.controller.as_deref()).collect::<Vec<_>>(), [Some(DUALSHOCK2), None]);
        assert_eq!(info.firmware, [(BIOS_EUROPE, FirmwareSource::Absent)]);
        for s in schema() {
            assert!(s.check().is_ok(), "{}", s.key);
        }
    }

    /// On Automatic a disc's serial, else its VMODE, names the console's market, and the setting overrides it
    /// (Sapphire_Native.md §49.2).
    #[test]
    fn a_discs_own_market_is_the_consoles_on_automatic() {
        let disc = |cnf: &str| volume(20, cnf, false);
        for (cnf, setting, market) in [
            ("BOOT2 = cdrom0:\\SLES_512.34;1\r\nVMODE = PAL\r\n", "auto", Market::Europe),
            ("BOOT2 = cdrom0:\\SCES_500.00;1\r\nVMODE = NTSC\r\n", "auto", Market::Europe),
            ("BOOT2 = cdrom0:\\SLPM_650.01;1\r\n", "auto", Market::Japan),
            ("BOOT2 = cdrom0:\\SCKA_200.01;1\r\n", "auto", Market::Japan),
            ("BOOT2 = cdrom0:\\SCUS_971.24;1\r\n", "auto", Market::Americas),
            ("BOOT2 = cdrom0:\\GAME.ELF;1\r\nVMODE = PAL\r\n", "auto", Market::Europe),
            ("BOOT2 = cdrom0:\\GAME.ELF;1\r\n", "auto", Market::Americas),
            ("BOOT2 = cdrom0:\\SLES_512.34;1\r\nVMODE = PAL\r\n", "ps2.us", Market::Americas),
        ] {
            assert_eq!(create(&disc(cnf), &[(REGION_KEY, setting)]).unwrap().market, market, "{cnf} on {setting}");
        }
        let bios = vec![0u8; BIOS_BYTES];
        let m = create_with(&disc("BOOT2 = cdrom0:\\SLES_512.34;1\r\n"), &[], vec![emusen_native::abi::File { which: BIOS_AMERICAS, data: &bios }]).unwrap();
        assert_eq!((m.market, m.booted, m.machine_info().firmware), (Market::Europe, false, vec![(BIOS_EUROPE, FirmwareSource::Absent)]), "a European disc wants the European BIOS, which a host refuses the game without");
    }

    /// A disc image given at create is in the drive when the console is switched on, its tray closed, as a console
    /// started with a game in it; a program leaves the drive empty (Sapphire_Native.md §49.2).
    #[test]
    fn a_disc_given_at_create_is_in_the_drive_at_power_on() {
        for image in [cd(), raw(&cd()), dvd()] {
            let mut m = create(&image, &[]).unwrap();
            let drive = m.drives()[0].clone();
            assert!(drive.loaded && !drive.lid_open);
            let kind = m.sys.board.iop.bus.cdvd.read8(crate::cdvd::reg::DISK_TYPE);
            assert!(matches!(kind, crate::cdvd::disk_type::PS2_CD | crate::cdvd::disk_type::PS2_DVD), "{kind:#04x}");
        }
        assert!(!create(&elf(), &[]).unwrap().drives()[0].loaded);
        let sheet = b"FILE \"track.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n";
        let track = raw(&cd());
        let file = emusen_native::abi::File { which: flags::FILE_MEDIA, data: &track };
        let m = create_with(sheet, &[], vec![file]).unwrap();
        assert!(matches!(&m.media, Media::Disc(d) if d.serial.as_deref() == Some("SLUS-20999")) && m.drives()[0].loaded, "a sheet, judged by its track");
    }

    /// A state names the drive's disc: a state made with the created disc loads over it, and over an emptied or another
    /// drive it is refused as another disc's until the host restores the disc (Sapphire_Native.md §49.4).
    #[test]
    fn a_state_keeps_to_the_disc_in_the_drive() {
        let mut m = create(&dvd(), &[]).unwrap();
        let identity = m.media.identity();
        assert_eq!(m.media_identity(), identity, "the created disc in the drive: the image's own name, as every earlier state");
        let mut state = vec![0u8; m.state_size()];
        m.save_state(&mut state).unwrap();
        m.media_lid(DRIVE, true).unwrap();
        insert(&mut m, None).unwrap();
        m.media_lid(DRIVE, false).unwrap();
        assert!(matches!(m.load_state(&state), Err(crate::state::StateError::WrongMedia { .. })), "an empty drive");
        m.media_lid(DRIVE, true).unwrap();
        insert(&mut m, Some(&cd())).unwrap();
        assert!(matches!(m.load_state(&state), Err(crate::state::StateError::WrongMedia { .. })), "another disc");
        m.media_insert(DRIVE, &Insert { image: Some(&dvd()), files: Vec::new(), restore: true }).unwrap();
        m.load_state(&state).unwrap();
        assert_eq!(empty(Vec::new()).media_identity(), Media::Empty.identity(), "an empty start: the name it always had");
    }

    #[test]
    fn the_ee_tier_setting_defaults_to_the_recompiler_and_chooses_the_interpreter() {
        let s = schema().into_iter().find(|s| s.key == EE_TIER_KEY).expect("the tier is a setting");
        assert!(s.default == "recompiler" && matches!(s.effect, Effect::Exact) && s.check().is_ok());
        assert!(create(&elf(), &[]).unwrap().blocks.is_some());
        assert!(create(&elf(), &[(EE_TIER_KEY, "interpreter")]).unwrap().blocks.is_none());
    }

    /// Idle skipping is exact, so it is on by default and one setting away (S33, Sapphire_Native.md §47.9).
    #[test]
    fn idle_skipping_is_on_by_default_and_a_setting_turns_it_off() {
        let s = schema().into_iter().find(|s| s.key == IDLE_KEY).expect("idle skipping is a setting");
        assert!(s.default == "on" && matches!(s.effect, Effect::Exact) && s.check().is_ok());
        assert!(create(&elf(), &[]).unwrap().idle.on);
        assert!(!create(&elf(), &[(IDLE_KEY, "off")]).unwrap().idle.on);
    }

    /// A disc image whose volume claims more sectors than it holds, a dump cut short (ECMA-119 §8.4.8).
    fn short() -> Vec<u8> {
        let mut d = dvd();
        let claimed = (d.len() / crate::media::USER + 1) as u32;
        d[16 * crate::media::USER + 80..16 * crate::media::USER + 84].copy_from_slice(&claimed.to_le_bytes());
        d
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
            (short(), STATUS_BAD_DISC),
        ] {
            assert_eq!(create(&image, &[]).err(), Some(code));
            assert!(Machine::status_text(code).is_some());
        }
        let bios = vec![7u8; BIOS_BYTES];
        let file = |which, data| emusen_native::abi::File { which, data };
        assert_eq!(create_with(&elf(), &[], vec![file(0, &bios)]).err(), Some(status::BAD_FILE));
        assert_eq!(create_with(&elf(), &[], vec![file(BIOS_AMERICAS, &bios[..1000])]).err(), Some(status::BAD_FILE));
        let m = create_with(&elf(), &[], vec![file(BIOS_AMERICAS, &bios)]).unwrap();
        assert_eq!((m.machine_info().firmware, m.space_byte(BIOS_ID, 0)), (vec![(BIOS_AMERICAS, FirmwareSource::File)], Some(7)));
        assert_eq!(m.program.as_deref(), Some(&elf()[..]), "an EE program waits for the fast boot");
        assert!(create(&elf(), &[]).unwrap().program.is_none(), "no BIOS: nothing to enter it by");
        let eu = create_with(&elf(), &[], vec![file(BIOS_EUROPE, &bios)]).unwrap();
        assert_eq!((eu.market, eu.booted), (Market::Europe, true), "on Automatic the BIOS's own market is the console's");
        let us = create_with(&elf(), &[(REGION_KEY, "ps2.us")], vec![file(BIOS_EUROPE, &bios)]).unwrap();
        assert_eq!((us.market, us.booted), (Market::Americas, false), "a market named by the setting takes only its own BIOS");
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
        assert_eq!((ps2.id.as_str(), ps2.extensions.iter().map(String::as_str).collect::<Vec<_>>(), ps2.development), ("ps2", vec![".elf", ".iso", ".chd", ".cue", ".bin"], false));
        assert!(info.description.as_deref().is_some_and(|d| d.contains("G6 (speed)") && d.contains("G9 (the open BIOS not written)")), "the gates not met are named");
        assert_eq!(info.accuracy.as_ref().map(|a| a.notes.lines().count()), Some(11), "one line for each gate, G1 to G11");
        assert_eq!(ps2.firmware.iter().map(|f| (f.which, f.name.as_str(), f.size, f.required)).collect::<Vec<_>>(), [
            (16, "ps2-0230j-20080220.bin", 4 << 20, false),
            (17, "ps2-0230a-20080220.bin", 4 << 20, false),
            (18, "ps2-0230e-20080220.bin", 4 << 20, false),
        ]);
        assert_eq!(ps2.controllers.iter().map(|c| (c.id.as_str(), c.buttons.len(), c.axes.len())).collect::<Vec<_>>(), [(DUALSHOCK2, 16, 4), (DIGITAL, 14, 0)]);
        let asked = |image: &[u8]| Machine::firmware_for(image).iter().map(|f| f.which).collect::<Vec<_>>();
        assert_eq!(asked(&cd()), [BIOS_AMERICAS], "a disc asks for its own market's BIOS");
        assert_eq!(asked(&volume(20, "BOOT2 = cdrom0:\\SLES_512.34;1\r\nVMODE = PAL\r\n", false)), [BIOS_EUROPE]);
        assert_eq!(asked(&volume(20, "BOOT2 = cdrom0:\\SLPM_650.01;1\r\n", false)), [BIOS_JAPAN]);
        let every = [BIOS_AMERICAS, BIOS_EUROPE, BIOS_JAPAN];
        assert_eq!((asked(&elf()), asked(b"FILE \"game.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n")), (every.to_vec(), every.to_vec()), "a program, and a sheet this call cannot follow, ask for every market's");
        assert!(asked(&[1, 2, 3]).is_empty());
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

    /// The frontend's buttons and sticks reach the pad in their own port on SIO2, which the BIOS polls (Sapphire_Native.md
    /// §35.5); a port with nothing plugged in has no pad.
    #[test]
    fn the_buttons_and_sticks_reach_the_pad_in_their_port() {
        let mut m = create(&elf(), &[(PAD_KEYS[1], "none")]).unwrap();
        m.set_buttons(0, 1 << 14 | 1 << 6, u32::MAX).unwrap();
        m.set_axis(0, 0, 1.0).unwrap();
        let pad = m.sys.board.iop.bus.sio2.pad_mut(0, 0).unwrap().input;
        assert_eq!((pad.buttons, pad.sticks[2]), (1 << 14 | 1 << 6, 0xFF), "Cross and Down, left stick right");
        assert!(m.sys.board.iop.bus.sio2.pad_mut(1, 0).is_none(), "nothing in port 2");
    }

    /// The console's EEPROM is the medium `ps2.nvm`, the market's own file: loaded at create, handed back, and marked
    /// changed once the drive writes it (Sapphire_Plan.md §4.4; Sapphire_Native.md §35.5).
    #[test]
    fn the_eeprom_is_the_markets_console_file() {
        let file = |which, data| emusen_native::abi::File { which, data };
        let mut nvm = vec![0u8; nvm::BYTES];
        nvm[16] = 0x5A;
        assert_eq!(create_with(&elf(), &[], vec![file(NVM_FILES[1], &nvm[..100])]).err(), Some(status::BAD_FILE));
        let mut m = create_with(&elf(), &[(REGION_KEY, "ps2.eu")], vec![file(NVM_FILES[2], &nvm[..]), file(NVM_FILES[1], &[7u8; nvm::BYTES][..])]).unwrap();
        assert_eq!(m.battery(NVM_FILES[2]).unwrap(), (&nvm[..], flags::BATTERY_TRACKED), "the European console's file, the American one ignored");
        assert_eq!(m.battery(NVM_FILES[1]).err(), Some(status::BAD_FILE));
        m.nvm_mut().open([1, 0, 1]);
        m.nvm_mut().write(&[9; 16]);
        assert_eq!(m.battery(NVM_FILES[2]).unwrap().1, flags::BATTERY_TRACKED | flags::BATTERY_CHANGED);
        m.battery_saved(NVM_FILES[2]).unwrap();
        assert_eq!(m.battery(NVM_FILES[2]).unwrap().1, flags::BATTERY_TRACKED);
        let blank = create(&elf(), &[]).unwrap();
        assert!(blank.nvm().bytes.iter().all(|&b| b == 0), "no file: blank");
    }

    #[test]
    fn the_two_cards_are_slots_of_ps2_card_and_a_card_file_of_the_wrong_size_is_refused() {
        let slots: Vec<_> = Machine::battery_files("ps2").unwrap().into_iter().map(|b| (b.which, b.kind, b.medium, b.label)).collect();
        let nvm = |w| (w, BatteryKind::Console, Some("ps2.nvm".into()), Some("Inside the console".into()));
        assert_eq!(
            slots,
            [(0, BatteryKind::Removable, Some("ps2.card".into()), Some("Slot 1".into())), (1, BatteryKind::Removable, Some("ps2.card".into()), Some("Slot 2".into())), nvm(2), nvm(3), nvm(4)],
            "Sapphire_Plan.md §4.4"
        );
        let file = |which, data| emusen_native::abi::File { which, data };
        assert_eq!(create_with(&elf(), &[], vec![file(0, &[0u8; 1000][..])]).err(), Some(status::BAD_FILE));
        assert!(create_with(&elf(), &[], vec![file(0, &[][..])]).is_ok(), "an empty file is a card not yet saved");
        let mut card = vec![0xFFu8; CARD_BYTES];
        card[0] = 0x53;
        let mut m = create_with(&elf(), &[], vec![file(1, &card[..])]).unwrap();
        assert_eq!(m.battery(1).unwrap(), (&card[..], flags::BATTERY_TRACKED));
        assert!(m.battery(0).unwrap().0.iter().all(|&b| b == 0xFF), "slot 1 holds an erased card");
        assert_eq!(m.machine_info().battery.iter().map(|b| b.which).collect::<Vec<_>>(), [0, 1, 3], "the cards and the American console's EEPROM");
        assert!(m.firmware.is_empty(), "a card is not firmware");
        m.set_axis(0, 2, 1.0).unwrap();
        m.set_axis(0, 1, -1.0).unwrap();
        assert_eq!(m.sticks[0], [0xFF, 0x80, 0x80, 0x00], "right X full right, left Y full up");
    }

    fn empty(files: Vec<emusen_native::abi::File<'_>>) -> Machine {
        let request = Create { image: &[], settings: Settings::from_pairs(Vec::new()), files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        Machine::create_empty("ps2", &request).unwrap()
    }

    fn insert(m: &mut Machine, image: Option<&[u8]>) -> Result<(), i32> {
        m.media_insert(DRIVE, &Insert { image, files: Vec::new(), restore: false })
    }

    /// G10 (Sapphire_Plan.md §8): the console started with nothing in it, its tray closed and empty and its market the
    /// BIOS's; with no BIOS it runs nothing and says so in machine info, which a host refuses by the entry's replacement.
    #[test]
    fn the_console_starts_empty_with_the_tray_closed_and_the_bios_given_naming_the_market() {
        assert!(Machine::starts_empty("ps2") && !Machine::starts_empty("psx"));
        const { assert!(Machine::CAPABILITIES & caps::MEDIA != 0) };
        let m = empty(Vec::new());
        assert_eq!((m.media.clone(), m.booted), (Media::Empty, false));
        assert_eq!(m.drives(), [Drive { drive: DRIVE, label: "DVD drive".into(), kind: "dvd".into(), lid_open: false, loaded: false }]);
        assert_eq!(m.machine_info().firmware, [(BIOS_AMERICAS, FirmwareSource::Absent)], "nothing runs");
        assert_eq!(bios(Market::Americas).replacement, Some(Replacement::None { cost: BIOS_COST.into() }), "the words a host refuses the start with");
        let bios = vec![0u8; BIOS_BYTES];
        let eu = empty(vec![emusen_native::abi::File { which: BIOS_EUROPE, data: &bios }]);
        assert_eq!((eu.market, eu.booted, eu.machine_info().firmware), (Market::Europe, true, vec![(BIOS_EUROPE, FirmwareSource::File)]));
    }

    /// G10: a disc goes in only through the open tray, judged by the recogniser, and reaches the drive, which detects it
    /// as a PlayStation 2 DVD once the tray closes; Eject empties it; a state's load keeps the drive (EmuSen_CoreAPI.md §6.23).
    #[test]
    fn a_disc_goes_in_through_the_tray_and_reaches_the_drive() {
        let mut m = empty(Vec::new());
        let disc = dvd();
        assert_eq!(insert(&mut m, Some(&disc)), Err(status::LID_CLOSED));
        m.media_lid(DRIVE, true).unwrap();
        assert!(m.drives()[0].lid_open);
        assert_eq!(insert(&mut m, Some(&elf())), Err(STATUS_NOT_A_DISC));
        assert_eq!(insert(&mut m, Some(&volume(20, "BOOT = cdrom:\\SLUS_007.70;1\r\n", true))), Err(STATUS_PLAYSTATION_DISC));
        assert_eq!(insert(&mut m, Some(&[7u8; 9000])), Err(STATUS_NOT_PS2));
        assert!(m.drives()[0].lid_open && !m.drives()[0].loaded, "a refused disc changes nothing");
        insert(&mut m, Some(&disc)).unwrap();
        m.media_lid(DRIVE, false).unwrap();
        for _ in 0..90 {
            Core::advance(&mut m, &mut 0).unwrap();
        }
        let drive = |m: &mut Machine| m.sys.board.iop.bus.cdvd.read8(crate::cdvd::reg::DISK_TYPE);
        assert_eq!((m.drives()[0].loaded, drive(&mut m)), (true, crate::cdvd::disk_type::PS2_DVD), "spun up and detected");
        let mut state = vec![0u8; m.state_size()];
        m.save_state(&mut state).unwrap();
        m.load_state(&state).unwrap();
        assert!(m.drives()[0].loaded && !m.drives()[0].lid_open, "the drive kept through the replay");
        m.media_lid(DRIVE, true).unwrap();
        insert(&mut m, None).unwrap();
        m.media_lid(DRIVE, false).unwrap();
        assert_eq!((m.drives()[0].loaded, drive(&mut m)), (false, crate::cdvd::disk_type::NONE));
        assert_eq!(m.media_lid(1, true), Err(status::NO_SUCH_DRIVE));
        m.media_insert(DRIVE, &Insert { image: Some(&disc), files: Vec::new(), restore: true }).unwrap();
        assert!(m.drives()[0].loaded && !m.drives()[0].lid_open, "a restore sets the drive with its tray closed");
    }
}
