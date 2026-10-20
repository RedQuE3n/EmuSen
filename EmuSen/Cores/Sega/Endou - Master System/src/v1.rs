//! Endou on the core ABI v1: `Core` over the stage 1 machine, and `core_exports!`. The descriptors are the consoles'
//! facts of Endou_Plan.md §4; Endou_Native.md §2 records what the stub answers.

use emusen_native::abi::Region;
use emusen_native::core::*;

use crate::board::{Board, Mapper, Row, TABLE, Why, identify_with};
use crate::boot::{BIOS_SIZE, Boot, open_boot};
use crate::bus::{PATCH_HIGH, Patch};
use crate::debug;
use crate::machine::{AUDIO_RATE, HEIGHT, MAX_HEIGHT, Machine, RAM_ID, WIDTH, Z80BUS_ID};
use crate::media::{Console, Market, Media, Refusal};
use crate::model::Model;
use crate::state::{STATE_VERSION, STATE_VERSION_BEFORE_FM};

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

/// The model setting (Endou_Plan.md §5.8): from the market, or a console chosen. An SG-1000 cartridge runs on the
/// SG-1000 or the Mark III, the consoles whose slot takes it; a Master System cartridge on any but the SG-1000.
pub const MODEL_KEY: &str = "model";
pub const MODELS: [(&str, &str, Model); 6] = [
    ("mark3", "Mark III", Model::MarkIII),
    ("mark3fm", "Mark III with the FM Sound Unit", Model::MarkIIIFm),
    ("jsms", "Japanese Master System", Model::JapaneseSms),
    ("sms1", "Master System", Model::Sms1),
    ("sms2", "Master System II", Model::Sms2),
    ("sg1000", "SG-1000", Model::Sg1000),
];

/// The BIOS settings, one per console that has a BIOS (Endou_OpenBoot.md §5, decided 2026-10-19): the open boot program
/// by default, the player's own image as an opt-in.
pub const BIOS_KEYS: [(&str, &str, Model); 3] =
    [("bios_sms1", "Master System BIOS", Model::Sms1), ("bios_sms2", "Master System II BIOS", Model::Sms2), ("bios_jsms", "Japanese Master System BIOS", Model::JapaneseSms)];
pub const BIOS_OPEN: &str = "open";
pub const BIOS_PLAYER: &str = "player";

/// File numbers of the BIOS by console (0 is the battery).
pub const BIOS_SMS1: u32 = 1;
pub const BIOS_SMS2: u32 = 2;
pub const BIOS_JSMS: u32 = 3;

/// The firmware page's words for running on the open boot program (Endou_OpenBoot.md §8), and the Japanese Master
/// System's, whose BIOS checks no header.
pub const BIOS_COST: &str = "Games start without Sega's logo screen. The open boot program sets the console up as Sega's manual says the BIOS does, checks an export cartridge's header as the documents describe, and hands the game over the same way; a game that reads the BIOS's code or relies on what the documents leave unsaid may behave differently, the start-up takes a different time, and the BIOS's built-in games are absent.";
pub const BIOS_JSMS_COST: &str = "Games start without Sega's logo screen. The open boot program sets the console up as Sega's manual says the BIOS does and hands the game over the same way, checking no header, as the Japanese console's BIOS does not; a game that reads the BIOS's code or relies on what the documents leave unsaid may behave differently, the start-up takes a different time, and the BIOS's built-in game is absent.";

/// The MD5 of the 8 KiB image No-Intro's DAT lists for the Japanese Master System, (Japan) v2.1 (Endou_References.md
/// §6f).
pub const BIOS_JSMS_MD5: &str = "24a519c53f67b00640d0048ef7089105";

/// The MD5s of the 8 KiB images No-Intro's DAT lists for the first Master System, (USA, Europe) v1.3 and (Europe) v2.0;
/// the Master System II's are larger, with a built-in game, and not mapped yet (Endou_Native.md §14.4).
pub const BIOS_SMS1_MD5: [&str; 2] = ["840481177270d5642a14ca71ee72844c", "bdb34f2c471e5d0c358eeec621f9d029"];

/// Each BIOS choice's words: what the open program costs in accuracy, and what the player's own image gives.
pub const BIOS_OPEN_HELP: &str = "Games start without Sega's logo screen and after a different time, the BIOS's built-in games are absent, and a game that reads the BIOS's code may behave differently.";
pub const BIOS_PLAYER_HELP: &str = "The console's own start-up, exact; used when the file is in the firmware folder, else the open boot program runs. Images with a built-in game are not mapped yet.";

/// The BIOS file a console takes, where it has one.
pub fn bios_file(model: Model) -> Option<u32> {
    match model {
        Model::Sms1 => Some(BIOS_SMS1),
        Model::Sms2 => Some(BIOS_SMS2),
        Model::JapaneseSms => Some(BIOS_JSMS),
        _ => None,
    }
}

/// The firmware entry of a console's BIOS.
pub fn bios(model: Model) -> Option<Firmware> {
    let (name, label, md5, cost): (_, _, &[&str], _) = match model {
        Model::Sms1 => ("bios_sms1.sms", "The Master System's BIOS", &BIOS_SMS1_MD5, BIOS_COST),
        Model::Sms2 => ("bios_sms2.sms", "The Master System II's BIOS", &[], BIOS_COST),
        Model::JapaneseSms => ("bios_jsms.sms", "The Japanese Master System's BIOS", &[BIOS_JSMS_MD5], BIOS_JSMS_COST),
        _ => return None,
    };
    Some(Firmware {
        which: bios_file(model)?,
        name: name.into(),
        label: label.into(),
        size: BIOS_SIZE as u64,
        required: false,
        parts: vec![vec![name.into()]],
        replacement: Some(Replacement::Accuracy { cost: cost.into() }),
        md5: md5.iter().map(|m| m.to_string()).collect(),
    })
}

fn bios_list() -> Vec<Firmware> {
    [Model::Sms1, Model::Sms2, Model::JapaneseSms].into_iter().filter_map(bios).collect()
}

fn schema() -> Vec<Setting> {
    let mut choices = vec![Choice { value: "auto".into(), label: "From the cartridge".into(), help: None }];
    choices.extend(REGIONS.iter().map(|&(v, l, _)| Choice { value: v.into(), label: l.into(), help: None }));
    let mut models = vec![Choice { value: "auto".into(), label: "From the region".into(), help: None }];
    models.extend(MODELS.iter().map(|&(v, l, _)| Choice { value: v.into(), label: l.into(), help: None }));
    vec![Setting {
        key: MODEL_KEY.into(),
        label: "Model".into(),
        help: "The console a game runs on. From the region, a Japanese game runs on the Japanese Master System, with its FM sound unit, and any other on a Master System II; the Mark III has no FM unit unless the FM Sound Unit is chosen with it, and the first Master System has the Reset button the later one lacks. An SG-1000 game runs on the SG-1000, or on the Mark III, whose video chip shows the SG-1000's pictures in its own colours. Takes effect when a game is next loaded.".into(),
        kind: SettingKind::Choice(models),
        default: "auto".into(),
        scope: Scope::Create,
        category: Some("Console".into()),
        effect: Effect::None,
        advanced: false,
        hidden: false,
        restart: true,
    },
    Setting {
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
    .into_iter()
    .chain(BIOS_KEYS.iter().map(|&(key, label, model)| Setting {
        key: key.into(),
        label: label.into(),
        help: if model.checks_export_header() {
            "What the console runs before the game. Endou's open boot program sets the console up as Sega's manual describes and checks an export cartridge's header, with a screen of its own; your own BIOS, when its file is in the firmware folder, is the console's own start-up, its logo and any built-in game included. Takes effect when a game is next loaded."
        } else {
            "What the console runs before the game. Endou's open boot program sets the console up as Sega's manual describes and starts any game, checking no header as the Japanese console's BIOS does not, with a screen of its own; your own BIOS, when its file is in the firmware folder, is the console's own start-up, its logo and its built-in game included. Takes effect when a game is next loaded."
        }
        .into(),
        kind: SettingKind::Choice(vec![
            Choice { value: BIOS_OPEN.into(), label: "Endou's open boot program".into(), help: Some(BIOS_OPEN_HELP.into()) },
            Choice { value: BIOS_PLAYER.into(), label: "Your own BIOS".into(), help: Some(BIOS_PLAYER_HELP.into()) },
        ]),
        default: BIOS_OPEN.into(),
        scope: Scope::Create,
        category: Some("Console".into()),
        effect: Effect::None,
        advanced: false,
        hidden: false,
        restart: true,
    }))
    .collect()
}

/// The market a machine runs as: the setting's, else the header's, else Japan's (Endou_Plan.md §10 Q9).
pub fn market_of(media: &Media, setting: Option<&str>) -> Market {
    REGIONS.iter().find(|r| Some(r.0) == setting).map_or(media.market().unwrap_or(Market::Japan), |r| r.2)
}

/// The console a machine is: for an SG-1000 cartridge the Mark III where the setting names it, else the SG-1000; for a
/// Master System cartridge the setting's console where its slot takes one, else a Mark III for Japan and a Master
/// System II for every other market (Endou_Plan.md §5.8, §10 Q9). Since stage 9 Japan's console is the Japanese Master
/// System (decided 2026-10-19).
pub fn model_of(console: Console, market: Market, setting: Option<&str>) -> Model {
    let chosen = MODELS.iter().find(|m| Some(m.0) == setting).map(|m| m.2);
    match console {
        Console::Sg1000 => if chosen == Some(Model::MarkIII) { Model::MarkIII } else { Model::Sg1000 },
        Console::Sms => chosen.filter(|&m| m != Model::Sg1000).unwrap_or(if market == Market::Japan { Model::JapaneseSms } else { Model::Sms2 }),
    }
}

fn system(id: &str, name: &str, extension: &str, controller: Controller, firmware: Vec<Firmware>) -> emusen_native::core::System {
    emusen_native::core::System {
        id: id.into(),
        name: name.into(),
        extensions: vec![extension.into()],
        regions: vec![Region::Ntsc, Region::Pal],
        controllers: vec![controller],
        firmware,
        development: false,
    }
}

/// What a console runs before its game: none where it has no BIOS; the player's image where its setting chooses it and
/// the file was given; else the open boot program (Endou_OpenBoot.md §5).
pub fn boot_for(model: Model, settings: &Settings, files: &[emusen_native::abi::File<'_>]) -> (Boot, Vec<u8>) {
    let Some(which) = bios_file(model) else { return (Boot::None, Vec::new()) };
    let chosen = BIOS_KEYS.iter().find(|k| k.2 == model).is_some_and(|k| settings.get(k.0) == Some(BIOS_PLAYER));
    match files.iter().find(|f| f.which == which) {
        Some(f) if chosen => (Boot::Player, f.data.to_vec()),
        _ => (Boot::Open, open_boot(model).to_vec()),
    }
}

/// Every file is the battery or a BIOS of 8 KiB; any other is refused with words.
fn check_files(files: &[emusen_native::abi::File<'_>]) -> Result<(), i32> {
    for f in files {
        match f.which {
            0 => {}
            BIOS_SMS1 | BIOS_SMS2 | BIOS_JSMS if f.data.len() == BIOS_SIZE => {}
            BIOS_SMS1 | BIOS_SMS2 | BIOS_JSMS => {
                detail(&format!("file {} is {} bytes; a Master System BIOS Endou maps is 8,192 bytes, and the larger ones with a built-in game are not mapped yet", f.which, f.data.len()));
                return Err(status::BAD_FILE);
            }
            n => {
                detail(&format!("file {n} is not one Endou names"));
                return Err(status::BAD_FILE);
            }
        }
    }
    Ok(())
}

/// A machine for the request, its board from `table` and the rules (Endou_Native.md §9.2), its battery file loaded.
pub fn make(request: &Create<'_>, table: &[Row]) -> Result<Machine, i32> {
    make_as(request, table, None)
}

/// The same as the console a host routed the image to, where it named one (core ABI 1.15, Endou_Native.md §2.3).
pub fn make_as(request: &Create<'_>, table: &[Row], console: Option<Console>) -> Result<Machine, i32> {
    let media = match Media::read_as(request.image, console) {
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
    check_files(&request.files)?;
    let market = market_of(&media, request.settings.get(REGION_KEY));
    let model = model_of(media.console, market, request.settings.get(MODEL_KEY));
    let rom = request.image[media.skipped..].to_vec();
    let board = identify_with(&rom, media.console, table);
    let (boot, bios) = boot_for(model, &request.settings, &request.files);
    let mut m = Machine::with_boot(media, market, model, board, rom, boot, bios);
    // The battery file, where the board keeps one; bytes past the RAM's size are not the board's.
    if let Some(f) = request.files.iter().find(|f| f.which == 0) {
        if m.bus.cart.board.battery {
            let ram = &mut m.bus.cart.ram;
            let n = ram.len().min(f.data.len());
            ram[..n].copy_from_slice(&f.data[..n]);
        }
    }
    Ok(m)
}

/// A Master System with nothing in its slots (core ABI 1.11): the region's console, the Americas for `auto`, since an
/// empty console is started to see its BIOS, which the Mark III lacks (Endou_OpenBoot.md §4).
pub fn make_empty(request: &Create<'_>) -> Result<Machine, i32> {
    check_files(&request.files)?;
    let media = Media { console: Console::Sms, skipped: 0, header: None };
    let market = REGIONS.iter().find(|r| Some(r.0) == request.settings.get(REGION_KEY)).map_or(Market::Americas, |r| r.2);
    let model = model_of(Console::Sms, market, request.settings.get(MODEL_KEY));
    let board = Board { mapper: Mapper::Plain, ram: 0, battery: false, why: Why::Size };
    let (boot, bios) = boot_for(model, &request.settings, &request.files);
    Ok(Machine::with_boot(media, market, model, board, Vec::new(), boot, bios))
}

impl Core for Machine {
    const CAPABILITIES: u64 = caps::ROM_PATCHES | caps::DEBUG | caps::DEBUG_STACK | caps::DEBUG_REGISTERS | caps::DEBUG_DISASSEMBLE;

    fn info() -> Info {
        Info {
            id: "endou".into(),
            name: "Endou".into(),
            display_name: Some("Endou".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("The Sega Master System and the SG-1000 in Rust, written from hardware documents and graded by test programs. It runs the Mark III with or without its FM Sound Unit, the Japanese Master System with its FM unit, the Master System and the Master System II, the SG-1000, and SG-1000 games on the Mark III, with an open boot program in place of the BIOS; the light gun and the other peripherals are not yet built.".into()),
            systems: vec![
                system("sms", "Sega Master System / Mark III", ".sms", pad(SMS_PAD, "Control Pad", 8), bios_list()),
                system("sg1000", "Sega SG-1000", ".sg", pad(SG1000_PAD, "Joystick", 7), Vec::new()),
            ],
            deterministic: true,
            ..Info::default()
        }
    }

    fn settings_schema() -> Vec<Setting> {
        schema()
    }

    /// A Master System image may take either console's BIOS, chosen by the settings; an SG-1000 image none.
    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        match Media::read(image) {
            Ok(m) if m.console == Console::Sms => bios_list(),
            _ => Vec::new(),
        }
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_NO_CARTRIDGE => Some("the image is empty or larger than any Master System or SG-1000 board (4 MiB)".into()),
            _ => None,
        }
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        make(request, TABLE)
    }

    /// The Master System starts with nothing in it, into its BIOS; the SG-1000 has none to start into.
    fn starts_empty(system: &str) -> bool {
        system == Console::Sms.id()
    }

    fn create_empty(_system: &str, request: &Create<'_>) -> Result<Self, i32> {
        make_empty(request)
    }

    /// The console the host routed the image to; the export has refused every system core info does not list.
    fn create_as(system: &str, request: &Create<'_>) -> Result<Self, i32> {
        let console = Console::of_id(system).ok_or_else(|| {
            detail(&format!("{system} is not a system of Endou"));
            status::BAD_IMAGE
        })?;
        make_as(request, TABLE, Some(console))
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
            spaces: std::iter::once(Space { read_only: true, view: true, reports_stores: true, ..Space::new(Z80BUS_ID, "Z80BUS") })
                .chain(self.spaces().into_iter().map(|(id, name, read_only)| Space { read_only, cheats: id == RAM_ID, reports_stores: debug::reports_stores(id), ..Space::new(id, name) }))
                .collect(),
            processors: vec![Processor {
                id: 0,
                name: "Z80".into(),
                pc_bits: 16,
                registers: beryl_z80::REGISTERS.iter().map(|&(n, bits)| (n.to_owned(), bits)).collect(),
                code_space: Some(Z80BUS_ID),
            }],
            battery: if self.bus.cart.board.battery { vec![Battery::game(0, ".srm")] } else { Vec::new() },
            state: StateFormat {
                format: "ENDO".into(),
                version: STATE_VERSION as i64,
                loads_from: if self.model().fm().is_none() { vec![STATE_VERSION_BEFORE_FM as i64, STATE_VERSION as i64] } else { vec![STATE_VERSION as i64] },
            },
            phases: Vec::new(),
            patches: Some((0, PATCH_HIGH as u32)),
            skip_rendering_state_neutral: true,
            firmware: bios_file(self.model())
                .map(|which| (which, if self.boot == Boot::Player { FirmwareSource::File } else { FirmwareSource::Replacement }))
                .into_iter()
                .collect(),
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
        FrameInfo::rgba(WIDTH as i32, self.height as i32)
    }

    fn frame(&self) -> &[u8] {
        self.frame_bytes()
    }

    fn audio_rate(&self) -> i32 {
        AUDIO_RATE
    }

    fn audio_buffered(&self) -> usize {
        self.audio.len()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        self.audio.drain(out, max_frames)
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.audio.set_limit(samples);
    }

    fn peek_audio(&self, out: &mut [i16]) -> usize {
        self.audio.peek(out)
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        let p = self.pads.get_mut(port as usize).ok_or(status::NO_SUCH_PORT)?;
        *p = (*p & !changed) | (mask & changed);
        Ok(())
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        if space == Z80BUS_ID {
            return Ok(debug::Z80BUS_SIZE as i64);
        }
        self.space(space).map(|m| m.len() as i64).ok_or(status::NO_SUCH_SPACE)
    }

    /// The bus as the Z80 reads it, with no side effect, its addresses wrapping at 64 KiB; another space past its end 0.
    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        if space == Z80BUS_ID {
            for (i, b) in out.iter_mut().enumerate() {
                *b = self.bus.peek(address.wrapping_add(i as u32) as u16);
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
        if space == Z80BUS_ID {
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

    /// `ROM_PATCHES`: a Game Genie's patches on the cartridge's reads at $0000-$BFFF, each with its reference byte or none.
    fn set_rom_patches(&mut self, triples: &[u32]) -> Result<(), i32> {
        self.bus.patches = triples
            .chunks_exact(3)
            .filter(|t| t[0] <= PATCH_HIGH as u32)
            .map(|t| Patch { address: t[0] as u16, value: t[1] as u8, compare: u8::try_from(t[2]).ok() })
            .collect();
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut emusen_native::debug::Hooks> {
        Some(&mut self.hooks)
    }

    fn debug_run_frame(&mut self, flags: u32, _detail: &mut u64) -> Result<u32, i32> {
        Ok(self.run_frame_debug(flags))
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        (processor == 0).then_some(self.cpu.regs.pc as u64)
    }

    /// The VDP, the PSG, the board and the I/O chip, by the ids from 256 (EmuSen_CoreAPI.md §6.33, Endou_Native.md §15.4).
    fn units(&self) -> Vec<Unit> {
        debug::units(self)
    }

    /// The Z80's registers in `beryl-z80`'s list, or a unit's in its own.
    fn debug_registers(&self, processor: u32) -> Result<Vec<i64>, i32> {
        match processor {
            0 => Ok(self.cpu.regs.values().to_vec()),
            id => debug::unit_values(self, id).ok_or(status::NOT_SUPPORTED),
        }
    }

    fn debug_disassemble(&mut self, processor: u32, space: u32, address: u32, count: u32) -> Result<Vec<Instruction>, i32> {
        if processor != 0 {
            return Err(status::NOT_SUPPORTED);
        }
        Machine::debug_disassemble(self, space, address, count).ok_or(status::NO_SUCH_SPACE)
    }

    /// File 0 is the cartridge's RAM where its board keeps it with a battery, else empty.
    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        match which {
            0 if self.bus.cart.board.battery => Ok((&self.bus.cart.ram, 0)),
            0 => Ok((&[], 0)),
            _ => Err(status::BAD_FILE),
        }
    }
}

emusen_native::core_exports!(Machine; rom_patches, debug, debug_stack, debug_registers, debug_disassemble);

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
        assert_eq!(info.spaces.iter().map(|s| (s.name.as_str(), s.read_only, s.cheats)).collect::<Vec<_>>(), [("Z80BUS", true, false), ("RAM", false, true), ("VRAM", false, false), ("CRAM", false, false), ("ROM", true, false), ("BIOS", true, false)]);
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
    fn a_quiet_cartridge_is_black_and_silent_at_the_output_rate() {
        let mut m = create(&cartridge(0x8000, 4), &[]).unwrap();
        let mut out = vec![1i16; 4000];
        let mut total = 0;
        for _ in 0..60 {
            Core::advance(&mut m, &mut 0).unwrap();
            let n = m.drain_audio(&mut out, usize::MAX);
            assert!(out[..n].iter().all(|&s| s == 0));
            total += n / 2;
        }
        let want = AUDIO_RATE as f64 * 60.0 * NTSC_FRAME.1 as f64 / NTSC_FRAME.0 as f64;
        assert!((total as f64 - want).abs() < 2.0, "{total} for {want}");
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

    // Since core ABI 1.15 a host names the system it routed the image to: a 512 KiB `.sg` and an 8 KiB `.sms`, both without
    // Sega's header, run as the systems their extensions name, which the stage-1 rule gets wrong (Endou_Native.md §2.3).
    #[test]
    fn a_routed_system_is_the_machine_s_and_the_rule_stands_without_one() {
        let request = |image: &'static [u8]| Create { image, settings: Settings::default(), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        static MUSIC: [u8; 512 * 1024] = [0xC3; 512 * 1024];
        static BIOS: [u8; 8 * 1024] = [0xF3; 8 * 1024];
        let system = |m: Machine| m.machine_info().system;
        assert_eq!((system(Machine::create(&request(&MUSIC)).unwrap()), system(Machine::create(&request(&BIOS)).unwrap())), ("sms".to_owned(), "sg1000".to_owned()));
        let music = Machine::create_as("sg1000", &request(&MUSIC)).unwrap();
        assert_eq!((music.machine_info().system.as_str(), music.model(), music.machine_info().ports[0].controller.as_deref()), ("sg1000", Model::Sg1000, Some(SG1000_PAD)));
        let bios = Machine::create_as("sms", &request(&BIOS)).unwrap();
        assert_eq!((bios.machine_info().system.as_str(), bios.model()), ("sms", Model::JapaneseSms), "no header: a Japanese console");
        assert_eq!(Machine::create_as("gg", &request(&BIOS)).err(), Some(status::BAD_IMAGE));
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

    // A seeded fault that dropped the battery file at create passed every test (Endou_Native.md §9.13).
    #[test]
    fn a_battery_board_takes_its_file_at_create_and_gives_it_back() {
        let image = cartridge(0x40000, 4);
        let table = [Row { crc: crate::board::crc32(&image), mapper: crate::board::Mapper::Sega, ram: 0x2000, battery: true }];
        let save: Vec<u8> = (0..0x2000).map(|i| (i * 13) as u8).collect();
        let file = emusen_native::abi::File { which: 0, data: &save };
        let request = Create { image: &image, settings: Settings::default(), files: vec![file], pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        let m = make(&request, &table).unwrap();
        assert_eq!(m.battery(0).unwrap().0, &save[..]);
        assert_eq!(m.machine_info().battery.iter().map(|b| (b.which, b.suffix.as_str())).collect::<Vec<_>>(), [(0, ".srm")]);
        let plain = make(&request, &[]).unwrap();
        assert!(plain.battery(0).unwrap().0.is_empty() && plain.machine_info().battery.is_empty(), "a board without a battery takes no file");
    }

    // Endou_Plan.md §5.8: an SG-1000 cartridge on the SG-1000 or the Mark III, a Master System cartridge never on the SG-1000.
    #[test]
    fn the_model_setting_offers_each_cartridge_the_consoles_whose_slot_takes_it() {
        let sg = [0x00u8; 0x4000];
        let on = |image: &[u8], model: &str| create(image, &[(MODEL_KEY, model)]).unwrap().model();
        assert_eq!([on(&sg, "auto"), on(&sg, "mark3"), on(&sg, "sms2"), on(&sg, "sg1000")], [Model::Sg1000, Model::MarkIII, Model::Sg1000, Model::Sg1000]);
        let export = cartridge(0x8000, 4);
        assert_eq!([on(&export, "auto"), on(&export, "sg1000"), on(&export, "sms1"), on(&export, "mark3")], [Model::Sms2, Model::Sms2, Model::Sms1, Model::MarkIII]);
        let mark3 = create(&sg, &[(MODEL_KEY, "mark3")]).unwrap();
        let info = mark3.machine_info();
        assert_eq!((info.system.as_str(), mark3.bus.ram.len(), mark3.bus.vdp.variant, mark3.bus.sound.psg.variant), ("sg1000", 0x2000, beryl_sms_vdp::Variant::SMS1, beryl_sn76489::Variant::SEGA));
        assert!(info.spaces.iter().any(|s| s.name == "CRAM"), "the Mark III has CRAM");
    }

    // SMS Power!'s "Game Genie": a patch replaces the cartridge's byte at its address, only where the reference matches; a
    // Japanese cartridge on the Mark III, so that no BIOS is in the slot area.
    #[test]
    fn a_game_genie_patches_the_cartridge_s_reads_and_heeds_its_reference() {
        let mut image = cartridge(0x8000, 3);
        image[0x1234] = 0x11;
        image[0x2345] = 0x22;
        image[0x3456] = 0x00;
        let mut m = create(&image, &[(MODEL_KEY, "mark3")]).unwrap();
        assert_eq!(m.machine_info().patches, Some((0, 0xBFFF)));
        assert_eq!(Machine::CAPABILITIES & emusen_native::core::caps::ROM_PATCHES, emusen_native::core::caps::ROM_PATCHES);
        m.set_rom_patches(&[0x1234, 0x99, sys::flags::NO_COMPARE, 0x2345, 0x77, 0x23, 0x3456, 0x55, 0x00, 0xC000, 0x44, sys::flags::NO_COMPARE]).unwrap();
        assert_eq!((m.bus.peek(0x1234), m.bus.peek(0x2345), m.bus.peek(0x3456)), (0x99, 0x22, 0x55), "a wrong reference leaves the byte");
        assert_eq!(m.bus.patches.len(), 3, "an address past the slot area is no cartridge read");
        m.set_rom_patches(&[]).unwrap();
        assert_eq!(m.bus.peek(0x1234), 0x11);
    }

    #[test]
    fn the_descriptors_are_the_consoles() {
        let info = Machine::info();
        let ids: Vec<_> = info.systems.iter().map(|s| (s.id.as_str(), s.extensions.clone(), s.development, s.firmware.len())).collect();
        assert_eq!(ids, [("sms", vec![".sms".to_string()], false, 3), ("sg1000", vec![".sg".to_string()], false, 0)], "both offered since stage 7");
        let bios: Vec<_> = info.systems[0].firmware.iter().map(|f| (f.which, f.name.as_str(), f.size, f.required, f.md5.len())).collect();
        assert_eq!(bios, [(BIOS_SMS1, "bios_sms1.sms", 8192, false, 2), (BIOS_SMS2, "bios_sms2.sms", 8192, false, 0), (BIOS_JSMS, "bios_jsms.sms", 8192, false, 1)]);
        assert!(BIOS_SMS1_MD5.iter().chain([&BIOS_JSMS_MD5]).all(|m| m.len() == 32 && m.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())));
        let costs: Vec<_> = info.systems[0].firmware.iter().map(|f| f.replacement.clone()).collect();
        assert_eq!(costs, [BIOS_COST, BIOS_COST, BIOS_JSMS_COST].map(|c| Some(Replacement::Accuracy { cost: c.into() })));
        for cost in [BIOS_COST, BIOS_JSMS_COST] {
            assert_eq!(cost.split(". ").next(), Some("Games start without Sega's logo screen"), "the firmware page's summary is one short sentence");
        }
        assert_eq!((Machine::firmware_for(&cartridge(0x8000, 4)).len(), Machine::firmware_for(&[0u8; 0x4000]).len()), (3, 0));
        let sms = &info.systems[0].controllers[0];
        assert_eq!(sms.buttons.iter().map(|b| (b.bit, b.label.as_str())).collect::<Vec<_>>()[4..], [(4, "1"), (5, "2"), (6, "Pause"), (7, "Reset")]);
        assert_eq!(info.systems[1].controllers[0].buttons.len(), 7, "the SG-1000 has no Reset button");
    }
}
