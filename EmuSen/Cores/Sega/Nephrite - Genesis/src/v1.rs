//! Nephrite on the core ABI v1: `Core` over the stage 0 machine, and `core_exports!`. The descriptors are the
//! console's facts of Nephrite_Plan.md §4; Nephrite_Native.md §2 records what the stub answers.

use emusen_native::abi::Region;
use emusen_native::core::*;

use crate::genesis::Model;
use crate::machine::{HEIGHT, Machine, WIDTH, default_model};
use crate::media::{Media, System, cartridge_bytes};
use crate::debugger::{M68KBUS_ID, M68KBUS_SIZE, Z80BUS_ID, Z80BUS_SIZE};
use crate::state::{OLDEST_STATE_VERSION, STATE_VERSION, STATUS_OTHER_SYSTEM};

/// An image too short for a cartridge's vector table and header.
pub const STATUS_IMAGE_TOO_SHORT: i32 = -9;
pub const MIN_IMAGE: usize = 0x200;

/// The output rate, the step synthesisers' (`sound.rs`).
pub const AUDIO_RATE: i32 = crate::sound::RATE as i32;

/// Master clocks a second over a frame's (argued, Nephrite_Native.md §2.3): NTSC 15 times the colour subcarrier,
/// 4,725,000,000/88 Hz, over 262 lines of 3,420; PAL 12 times 4.43361875 MHz over 313 lines.
pub const NTSC_FRAME: (u64, u64) = (4_725_000_000, 88 * 262 * 3420);
pub const PAL_FRAME: (u64, u64) = (53_203_425, 313 * 3420);

/// The pad's bits in the order the hardware's two reads deliver them, then the six-button pad's four; each with its
/// canonical control (Nephrite_Plan.md §4.2).
const BUTTONS: [(u32, Control, &str); 12] = [
    (0, Control::Up, "Up"),
    (1, Control::Down, "Down"),
    (2, Control::Left, "Left"),
    (3, Control::Right, "Right"),
    (6, Control::Y, "A"),
    (4, Control::B, "B"),
    (5, Control::A, "C"),
    (10, Control::L, "X"),
    (9, Control::X, "Y"),
    (8, Control::R, "Z"),
    (7, Control::Start, "Start"),
    (11, Control::Select, "Mode"),
];

fn pad(id: &str, label: &str, six: bool) -> Controller {
    Controller {
        id: id.into(),
        label: label.into(),
        ports: vec![0, 1],
        buttons: BUTTONS.iter().filter(|b| six || b.0 < 8).map(|&(bit, c, l)| Button { bit, control: Some(c), label: l.into() }).collect(),
        axes: Vec::new(),
    }
}

/// The settings that choose each port's pad, read at create and between frames.
pub const PAD_KEYS: [&str; 2] = ["pad1", "pad2"];

fn pad_setting(port: usize) -> Setting {
    Setting {
        key: PAD_KEYS[port].into(),
        label: format!("Port {} controller", port + 1),
        help: "The pad plugged into the port: Sega's three-button Control Pad, or the six-button Arcade Pad, which a game reads through the same port and which some games use for X, Y, Z and Mode.".into(),
        kind: SettingKind::Choice(vec![
            Choice { value: "md.pad3".into(), label: "3-Button Control Pad".into(), help: None },
            Choice { value: "md.pad6".into(), label: "6-Button Arcade Pad".into(), help: None },
        ]),
        default: "md.pad3".into(),
        scope: Scope::Run,
        category: Some("Controllers".into()),
        effect: Effect::None,
        advanced: false,
        hidden: false,
        restart: false,
    }
}

/// The setting that chooses the console's region, read at create: the cartridge's header, or a console of one market.
pub const REGION_KEY: &str = "region";

/// Each region's version register: overseas (bit 7) and PAL (bit 6).
pub const REGIONS: [(&str, &str, bool, bool); 4] = [
    ("md.us", "Americas (NTSC)", true, false),
    ("md.eu", "Europe (PAL)", true, true),
    ("md.jp", "Japan (NTSC)", false, false),
    ("md.asia", "Asia (PAL)", false, true),
];

fn region_setting() -> Setting {
    let mut choices = vec![Choice { value: "auto".into(), label: "From the cartridge".into(), help: None }];
    choices.extend(REGIONS.iter().map(|&(value, label, _, _)| Choice { value: value.into(), label: label.into(), help: None }));
    Setting {
        key: REGION_KEY.into(),
        label: "Region".into(),
        help: "The console's market, which a game reads from its version register and which sets the picture's rate: 60 frames a second for NTSC, 50 for PAL. From the cartridge, a game made for the Americas or several markets runs as an American console, one for Europe alone as a European, one for Japan alone as a Japanese. A game that checks its market may refuse the others. Takes effect when a game is next loaded.".into(),
        kind: SettingKind::Choice(choices),
        default: "auto".into(),
        scope: Scope::Create,
        category: Some("Console".into()),
        effect: Effect::None,
        advanced: false,
        hidden: false,
        restart: true,
    }
}

/// The model's market as the region setting asks for it, the header's where it says `auto` or nothing.
pub fn region_model(model: Model, setting: Option<&str>) -> Model {
    match REGIONS.iter().find(|r| Some(r.0) == setting) {
        Some(&(_, _, overseas, pal)) => Model { overseas, pal, ..model },
        None => model,
    }
}

/// The setting that chooses the console's model, read at create.
pub const MODEL_KEY: &str = "model";

fn model_setting() -> Setting {
    Setting {
        key: MODEL_KEY.into(),
        label: "Console model".into(),
        help: "The console the game runs on. A model 1 has the discrete YM2612, whose converter gives quiet sounds their grain, and a soft output filter; a model 2 has the YM3438 inside its main chip, clean at low levels, and a brighter, steeper filter. Takes effect when a game is next loaded.".into(),
        kind: SettingKind::Choice(vec![
            Choice { value: "md.model1".into(), label: "Model 1".into(), help: None },
            Choice { value: "md.model2".into(), label: "Model 2".into(), help: None },
        ]),
        default: "md.model1".into(),
        scope: Scope::Create,
        category: Some("Console".into()),
        effect: Effect::None,
        advanced: false,
        hidden: false,
        restart: true,
    }
}

impl Machine {
    fn apply_pads(&mut self, settings: &Settings) {
        for (port, key) in PAD_KEYS.iter().enumerate() {
            if let Some(v) = settings.get(key) {
                self.six_button[port] = v == "md.pad6";
            }
        }
    }
}

fn controllers() -> Vec<Controller> {
    vec![pad("md.pad3", "3-Button Control Pad", false), pad("md.pad6", "6-Button Arcade Pad", true)]
}

/// File numbers of the firmware (Nephrite_Plan.md §5.6); 0 is the battery.
pub const TMSS: u32 = 1;
pub const CD_BIOS_U: u32 = 2;
pub const CD_BIOS_E: u32 = 3;
pub const CD_BIOS_J: u32 = 4;
pub const S32X_68K: u32 = 5;
pub const S32X_MASTER: u32 = 6;
pub const S32X_SLAVE: u32 = 7;
/// Sonic & Knuckles' lock-on: the player's cartridge on top, and the cartridge's second, 256 KiB ROM.
pub const LOCK_ON: u32 = 8;
pub const SK_PATCH: u32 = 9;

fn firmware(which: u32, name: &str, label: &str, size: u64, cost: &str) -> Firmware {
    Firmware { which, name: name.into(), label: label.into(), size, required: false, parts: vec![vec![name.into()]], replacement: Some(Replacement::None { cost: cost.into() }) }
}

fn tmss() -> Firmware {
    firmware(TMSS, "bios_MD.bin", "The Genesis's TMSS boot ROM (2,048 bytes)", 2048,
        "Without the image Nephrite runs as a console without TMSS, as the first Genesis and Mega Drive models were: licensed games start the same, without the licence screen.")
}

const CD_COST: &str = "Nephrite's open replacement for the Sega CD BIOS is not written yet (Nephrite_Plan.md §5.6): at this stage no disc runs, with the image or without it.";
const S32X_COST: &str = "Nephrite's open replacements for the 32X's boot ROMs are not written yet (Nephrite_Plan.md §5.6): at this stage no 32X game runs, with the images or without them.";

fn cd_bios(region: char) -> Firmware {
    let which = match region {
        'J' => CD_BIOS_J,
        'E' => CD_BIOS_E,
        _ => CD_BIOS_U,
    };
    let market = match region {
        'J' => "Japanese Mega-CD",
        'E' => "European Mega-CD",
        _ => "American Sega CD",
    };
    firmware(which, &format!("bios_CD_{region}.bin"), &format!("The {market}'s BIOS (131,072 bytes)"), 131_072, CD_COST)
}

fn s32x_boot() -> Vec<Firmware> {
    vec![
        firmware(S32X_68K, "32X_G_BIOS.BIN", "The 32X's 68000 boot ROM (256 bytes)", 256, S32X_COST),
        firmware(S32X_MASTER, "32X_M_BIOS.BIN", "The 32X's master SH-2 boot ROM (2,048 bytes)", 2048, S32X_COST),
        firmware(S32X_SLAVE, "32X_S_BIOS.BIN", "The 32X's slave SH-2 boot ROM (1,024 bytes)", 1024, S32X_COST),
    ]
}

/// The files Sonic & Knuckles takes beside its own image: the player's cartridges, not firmware; a combined image
/// carries them already.
fn lock_on_files() -> Vec<Firmware> {
    let cart = |which, name: &str, label: &str, size| Firmware {
        which,
        name: name.into(),
        label: label.into(),
        size,
        required: false,
        parts: vec![vec![name.into()]],
        replacement: Some(Replacement::None { cost: "Without it Sonic & Knuckles runs alone, as the cartridge does with nothing on top.".into() }),
    };
    vec![
        cart(LOCK_ON, "lockon.bin", "The cartridge locked on top of Sonic & Knuckles", 0x20_0000),
        cart(SK_PATCH, "sk2chip.bin", "Sonic & Knuckles' second ROM, for Sonic 2 on top (262,144 bytes)", 0x4_0000),
    ]
}

fn takes_lock_on(media: &Media, image_len: usize) -> bool {
    media.header.as_ref().is_some_and(|h| h.serial.starts_with(crate::cart::LOCK_ON_SERIAL)) && image_len <= 0x20_0000
}

/// A system whose games do not run yet is marked in development, so that no host offers it (EmuSen_CoreAPI.md §27.4).
fn system(id: &str, name: &str, extensions: &[&str], firmware: Vec<Firmware>, development: bool) -> emusen_native::core::System {
    emusen_native::core::System {
        id: id.into(),
        name: name.into(),
        extensions: extensions.iter().map(|&e| e.into()).collect(),
        regions: vec![Region::Ntsc, Region::Pal],
        controllers: controllers(),
        firmware,
        development,
    }
}

/// The extensions each system claims (Nephrite_Plan.md §4.1); a `.bin` disc image is told from a cartridge by its
/// contents.
pub const MD_EXTENSIONS: [&str; 4] = [".md", ".gen", ".bin", ".smd"];
pub const MCD_EXTENSIONS: [&str; 1] = [".iso"];
pub const S32X_EXTENSIONS: [&str; 1] = [".32x"];

impl Core for Machine {
    const CAPABILITIES: u64 = caps::ROM_PATCHES | caps::SETTINGS | caps::DEBUG | caps::DEBUG_STACK | caps::DEBUG_REGISTERS | caps::DEBUG_DISASSEMBLE;

    fn info() -> Info {
        Info {
            id: "nephrite".into(),
            name: "Nephrite".into(),
            display_name: Some("Nephrite".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("The Sega Genesis / Mega Drive in Rust, with the Sega CD and the 32X as its attachments, written from hardware documents and graded by test ROMs. It runs the Genesis's two processors, buses, cartridges, pads, picture and sound, the PSG and the YM2612 with each model's output; the Sega CD and the 32X are still to come.".into()),
            systems: vec![
                system("md", "Sega Genesis / Mega Drive", &MD_EXTENSIONS, vec![tmss()], false),
                system("mcd", "Sega CD / Mega-CD", &MCD_EXTENSIONS, vec![tmss(), cd_bios('U'), cd_bios('E'), cd_bios('J')], true),
                system("32x", "Sega 32X", &S32X_EXTENSIONS, [vec![tmss()], s32x_boot()].concat(), true),
            ],
            deterministic: true,
            ..Info::default()
        }
    }

    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        let image = &*cartridge_bytes(image);
        let media = Media::read(image);
        match media.system {
            System::Md if takes_lock_on(&media, image.len()) => lock_on_files(),
            System::Md => Vec::new(),
            System::Mcd => vec![cd_bios(media.bios_region())],
            System::S32x => s32x_boot(),
        }
    }

    fn settings_schema() -> Vec<Setting> {
        vec![model_setting(), region_setting(), pad_setting(0), pad_setting(1)]
    }

    fn status_text(code: i32) -> Option<String> {
        match code {
            STATUS_IMAGE_TOO_SHORT => Some("the image is shorter than the 512 bytes of a cartridge's vector table and header".into()),
            STATUS_OTHER_SYSTEM => Some("the state was saved by Nephrite running another of its systems".into()),
            _ => None,
        }
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        if request.image.len() < MIN_IMAGE {
            detail(&format!("the image is {} bytes; a cartridge's vectors and header take 512", request.image.len()));
            return Err(STATUS_IMAGE_TOO_SHORT);
        }
        if let Some(f) = request.files.iter().find(|f| f.which > SK_PATCH) {
            detail(&format!("file {} is not one Nephrite names", f.which));
            return Err(status::BAD_FILE);
        }
        let image = cartridge_bytes(request.image);
        let media = Media::read(&image);
        let model = region_model(Model { model2: request.settings.get(MODEL_KEY) == Some("md.model2"), ..default_model(&media) }, request.settings.get(REGION_KEY));
        let mut m = Machine::with_model(&image, media, model);
        m.apply_pads(&request.settings);
        if let Some(top) = request.files.iter().find(|f| f.which == LOCK_ON) {
            let patch = request.files.iter().find(|f| f.which == SK_PATCH).map_or(Vec::new(), |f| f.data.to_vec());
            m.genesis.hw.cart.lock_on(cartridge_bytes(top.data).into_owned(), patch);
        }
        m.firmware = request.files.iter().filter(|f| f.which != 0).map(|f| f.which).collect();
        if let (Some(id), Some(file)) = (m.battery_id(), request.files.iter().find(|f| f.which == 0)) {
            if id == crate::machine::SRAM_ID {
                m.genesis.hw.cart.load_battery(file.data);
            } else {
                let bytes = m.bytes_mut(id).expect("the battery's memory");
                let n = bytes.len().min(file.data.len());
                bytes[..n].copy_from_slice(&file.data[..n]);
            }
        }
        m.refresh_battery_file();
        Ok(m)
    }

    fn machine_info(&self) -> MachineInfo {
        let pal = self.genesis.hw.model.pal;
        let named = |which: u32| (which, if self.firmware.contains(&which) { FirmwareSource::File } else { FirmwareSource::Absent });
        MachineInfo {
            system: self.media.system.id().into(),
            region: Some(if pal { Region::Pal } else { Region::Ntsc }),
            frame_rate: if pal { PAL_FRAME } else { NTSC_FRAME },
            video: Video { base_width: WIDTH, base_height: HEIGHT, max_width: 320, max_height: 480, aspect: (4, 3), formats: vec![pixel::RGBA8888] },
            audio: Audio { rate: AUDIO_RATE, channels: Vec::new() },
            ports: (0..2).map(|p| Port { port: p as u32, controller: Some(if self.six_button[p] { "md.pad6" } else { "md.pad3" }.into()) }).collect(),
            spaces: self.spaces().iter().map(|m| Space { read_only: m.read_only, cheats: m.name == "WRAM", view: m.id <= crate::debugger::Z80BUS_ID, reports_stores: crate::debugger::reports_stores(m.id), ..Space::new(m.id, m.name) }).collect(),
            processors: self.processors(),
            battery: self.battery_id().map(|_| Battery { which: 0, suffix: if self.media.system == System::Mcd { ".brm" } else { ".srm" }.into() }).into_iter().collect(),
            state: StateFormat { format: "NPHR".into(), version: STATE_VERSION as i64, loads_from: (OLDEST_STATE_VERSION..=STATE_VERSION).map(i64::from).collect() },
            phases: Vec::new(),
            patches: (self.media.system == System::Md).then_some((0, 0x3F_FFFF)),
            skip_rendering_state_neutral: true,
            firmware: match self.media.system {
                System::Md if takes_lock_on(&self.media, self.genesis.hw.cart.rom.len()) => vec![named(LOCK_ON), named(SK_PATCH)],
                System::Md => Vec::new(),
                System::Mcd => vec![named(cd_bios(self.media.bios_region()).which)],
                System::S32x => vec![named(S32X_68K), named(S32X_MASTER), named(S32X_SLAVE)],
            },
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
        let f = &self.genesis.hw.vdp.frame;
        if self.picture.len() == f.width * f.height * 4 { FrameInfo::rgba(f.width as i32, f.height as i32) } else { FrameInfo::rgba(WIDTH as i32, HEIGHT as i32) }
    }

    fn frame(&self) -> &[u8] {
        &self.picture
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

    fn set_settings(&mut self, settings: &Settings) -> Result<(), i32> {
        self.apply_pads(settings);
        Ok(())
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        let p = self.pads.get_mut(port as usize).ok_or(status::NO_SUCH_PORT)?;
        *p = (*p & !changed) | (mask & changed);
        Ok(())
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        match space {
            M68KBUS_ID => Ok(M68KBUS_SIZE as i64),
            Z80BUS_ID => Ok(Z80BUS_SIZE as i64),
            _ => self.bytes(space).map(|m| m.len() as i64).ok_or(status::NO_SUCH_SPACE),
        }
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        if space == M68KBUS_ID || space == Z80BUS_ID {
            let size = if space == M68KBUS_ID { M68KBUS_SIZE } else { Z80BUS_SIZE };
            let n = out.len().min(size.saturating_sub(address as usize));
            self.read_bus(space, address, &mut out[..n]);
            out[n..].fill(0);
            return Ok(());
        }
        let m = self.bytes(space).ok_or(status::NO_SUCH_SPACE)?;
        for (i, b) in out.iter_mut().enumerate() {
            *b = m.get(address as usize + i).copied().unwrap_or(0);
        }
        Ok(())
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        if self.spaces().iter().any(|s| s.id == space && s.read_only) {
            return Err(status::READ_ONLY);
        }
        let m = self.bytes_mut(space).ok_or(status::NO_SUCH_SPACE)?;
        for (i, &b) in data.iter().enumerate() {
            if let Some(d) = m.get_mut(address as usize + i) {
                *d = b;
            }
        }
        self.refresh_battery_file();
        Ok(())
    }

    /// `ROM_PATCHES`: the Game Genie's patches on the cartridge port, compared with the board's byte when a compare is given.
    fn set_rom_patches(&mut self, triples: &[u32]) -> Result<(), i32> {
        self.genesis.hw.cart.patches = triples.chunks_exact(3).map(|t| (t[0] & 0x3F_FFFF, t[1] as u8, t[2])).collect();
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut emusen_native::debug::Hooks> {
        Some(&mut self.hooks)
    }

    /// The 68000's breakpoints are the shared hooks'; the Z80's the core's own.
    fn debug_breakpoints(&mut self, processor: u32, pairs: &[i32]) -> Result<(), i32> {
        match processor {
            crate::debugger::M68K => self.hooks.set_breakpoints(pairs),
            crate::debugger::Z80 => self.z80_breakpoints = pairs.chunks_exact(2).map(|p| (p[0], p[1])).collect(),
            _ => return Err(status::NOT_SUPPORTED),
        }
        Ok(())
    }

    fn debug_run_frame(&mut self, flags: u32, _detail: &mut u64) -> Result<u32, i32> {
        Ok(self.run_frame_debug(flags))
    }

    fn debug_stopped(&self) -> u32 {
        self.debug_stopped
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        Machine::debug_pc(self, processor)
    }

    fn debug_registers(&self, processor: u32) -> Result<Vec<i64>, i32> {
        Machine::debug_registers(self, processor).ok_or(status::NOT_SUPPORTED)
    }

    fn debug_disassemble(&mut self, processor: u32, space: u32, address: u32, count: u32) -> Result<Vec<Instruction>, i32> {
        if processor > crate::debugger::Z80 {
            return Err(status::NOT_SUPPORTED);
        }
        Machine::debug_disassemble(self, processor, space, address, count).ok_or(status::NO_SUCH_SPACE)
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        match (which, self.battery_id()) {
            (0, Some(_)) => Ok((self.battery_bytes().expect("the battery's memory"), 0)),
            (0, None) => Ok((&[], 0)),
            _ => Err(status::BAD_FILE),
        }
    }
}

emusen_native::core_exports!(Machine; rom_patches, settings, debug, debug_stack, debug_registers, debug_disassemble);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::cartridge;

    fn create(image: &[u8], files: Vec<emusen_native::abi::File<'_>>) -> Result<Machine, i32> {
        Machine::create(&Create { image, settings: Settings::default(), files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION })
    }

    // The debugger's two processors and their buses through the ABI: listed, sized, read without side effects, refused a write (Nephrite_Native.md §41).
    #[test]
    fn the_processors_and_their_buses_reach_the_host() {
        let mut m = create(&cartridge("SEGA GENESIS", "JUE", None), vec![]).unwrap();
        let info = m.machine_info();
        assert_eq!(info.processors.iter().map(|p| (p.name.as_str(), p.pc_bits, p.registers.len(), p.code_space)).collect::<Vec<_>>(), [("M68K", 24, 20, Some(0)), ("Z80", 16, 18, Some(1))]);
        assert_eq!(info.state.loads_from, [16, 17]);
        assert_eq!(info.spaces.iter().filter(|s| s.view).map(|s| (s.name.as_str(), s.read_only)).collect::<Vec<_>>(), [("M68KBUS", true), ("Z80BUS", true)]);
        assert_eq!((Core::space_size(&m, 0), Core::space_size(&m, 1)), (Ok(1 << 24), Ok(1 << 16)));
        let mut out = [0u8; 16];
        Core::space_read(&mut m, 0, 0x100, &mut out).unwrap();
        assert_eq!(&out, b"SEGA GENESIS    ");
        Core::space_read(&mut m, 1, 0xFFF8, &mut out).unwrap();
        assert_eq!(out[8..], [0; 8], "past the bus's end reads zero");
        assert_eq!(Core::space_write(&mut m, 0, 0xFF_0000, &[1]), Err(status::READ_ONLY));
        assert_eq!(Core::debug_registers(&m, 0).map(|r| r.len()), Ok(20));
        assert_eq!(Core::debug_disassemble(&mut m, 2, 0, 0, 1).map(|l| l.len()), Err(status::NOT_SUPPORTED));
        assert_eq!(Core::debug_disassemble(&mut m, 0, 0, 0x100, 1).map(|l| l.len()), Ok(1));
    }

    #[test]
    fn the_stub_reports_each_system_and_a_blank_picture() {
        let md = create(&cartridge("SEGA GENESIS", "JUE", None), vec![]).unwrap();
        let info = md.machine_info();
        assert_eq!((info.system.as_str(), info.region, info.frame_rate), ("md", Some(Region::Ntsc), NTSC_FRAME));
        assert_eq!(info.spaces.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["M68KBUS", "Z80BUS", "WRAM", "Z80RAM", "VRAM", "CRAM", "VSRAM", "ROM"]);
        assert!(md.frame().chunks(4).all(|p| p == [0, 0, 0, 255]) && md.frame().len() == 320 * 224 * 4);

        let pal = create(&cartridge("SEGA MEGA DRIVE", "E", None), vec![]).unwrap();
        assert_eq!((pal.machine_info().region, pal.machine_info().frame_rate), (Some(Region::Pal), PAL_FRAME));

        let s32x = create(&cartridge("SEGA 32X", "U", None), vec![]).unwrap();
        assert_eq!(s32x.machine_info().system, "32x");
        assert_eq!(Machine::firmware_for(&cartridge("SEGA 32X", "U", None)).len(), 3);
        assert!(Machine::firmware_for(&cartridge("SEGA GENESIS", "U", None)).is_empty());

        let mut iso = vec![0u8; 0x8000];
        iso[..14].copy_from_slice(crate::media::DISC_SIGNATURE);
        iso[0x1F0] = b'E';
        let mcd = create(&iso, vec![]).unwrap();
        let info = mcd.machine_info();
        assert_eq!((info.system.as_str(), info.region), ("mcd", Some(Region::Pal)));
        assert_eq!(info.battery[0].suffix, ".brm");
        assert_eq!(Machine::firmware_for(&iso)[0].name, "bios_CD_E.bin");
        assert_eq!(info.firmware, vec![(CD_BIOS_E, FirmwareSource::Absent)]);
    }

    #[test]
    fn the_rates_are_the_documented_clocks_over_a_frame() {
        let hz = |(n, d): (u64, u64)| n as f64 / d as f64;
        assert!((hz(NTSC_FRAME) - 59.9227).abs() < 1e-4, "{}", hz(NTSC_FRAME));
        assert!((hz(PAL_FRAME) - 49.7015).abs() < 1e-4, "{}", hz(PAL_FRAME));
    }

    #[test]
    fn a_short_image_is_refused_and_the_battery_file_is_read() {
        assert_eq!(create(&[0u8; 100], vec![]).err(), Some(STATUS_IMAGE_TOO_SHORT));
        let mut ra = [0u8; 12];
        ra[..4].copy_from_slice(&[b'R', b'A', 0xF8, 0x20]);
        ra[4..8].copy_from_slice(&0x20_0001u32.to_be_bytes());
        ra[8..12].copy_from_slice(&0x20_3FFFu32.to_be_bytes());
        let save = vec![0xA5u8; 8192];
        let m = create(&cartridge("SEGA GENESIS", "U", Some(ra)), vec![emusen_native::abi::File { which: 0, data: &save }]).unwrap();
        assert!(m.battery(0).unwrap().0.iter().skip(1).step_by(2).take(8192).eq(save.iter()), "the save RAM's odd lane, by address");
        assert_eq!(m.machine_info().battery[0].suffix, ".srm");
        assert_eq!(create(&cartridge("SEGA GENESIS", "U", None), vec![emusen_native::abi::File { which: 10, data: &save }]).err(), Some(status::BAD_FILE));
    }

    #[test]
    fn the_rom_is_read_only_and_the_pads_keep_unchanged_bits() {
        let mut m = create(&cartridge("SEGA GENESIS", "U", None), vec![]).unwrap();
        assert_eq!(m.space_write(crate::machine::ROM_ID, 0, &[1]), Err(status::READ_ONLY));
        m.set_buttons(0, 0b1010, 0b1111).unwrap();
        m.set_buttons(0, 0b0001, 0b0001).unwrap();
        assert_eq!(m.pads[0], 0b1011);
        assert_eq!(m.set_buttons(2, 1, 1), Err(status::NO_SUCH_PORT));
    }

    #[test]
    fn the_descriptors_are_valid_and_every_system_has_both_pads() {
        let info = Machine::info();
        assert_eq!(info.systems.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), ["md", "mcd", "32x"]);
        for s in &info.systems {
            assert_eq!(s.controllers.iter().map(|c| c.buttons.len()).collect::<Vec<_>>(), [8, 12]);
            assert!(s.firmware.iter().all(|f| !f.required));
        }
    }

    /// ROM_PATCHES: a patch answers every read of the cartridge at its address, a compare only where the board's byte is
    /// it; clearing them gives the board's bytes back. The range is the cartridge port's, on the Genesis alone.
    #[test]
    fn rom_patches_answer_for_the_cartridge_port() {
        let mut m = create(&cartridge("SEGA GENESIS", "U", None), vec![]).unwrap();
        assert_eq!(m.machine_info().patches, Some((0, 0x3F_FFFF)));
        let board = m.genesis.hw.cart.read16(0x100);
        assert_eq!(board, u16::from_be_bytes(*b"SE"));
        m.set_rom_patches(&[0x100, 0x12, sys::flags::NO_COMPARE, 0x101, 0x34, b'E' as u32, 0x102, 0x56, 0x00]).unwrap();
        assert_eq!((m.genesis.hw.cart.read16(0x100), m.genesis.hw.cart.read8(0x102)), (0x1234, b'G'));
        m.set_rom_patches(&[]).unwrap();
        assert_eq!(m.genesis.hw.cart.read16(0x100), board);
        assert_eq!(create(&cartridge("SEGA 32X", "U", None), vec![]).unwrap().machine_info().patches, None);
    }

    /// The region setting chooses the version register's market and the picture's rate at create; `auto` is the header's.
    #[test]
    fn the_region_setting_chooses_the_market_and_the_rate() {
        let image = cartridge("SEGA GENESIS", "U", None);
        let made = |region: &str| Machine::create(&Create { image: &image, settings: Settings::from_pairs(vec![(REGION_KEY.into(), region.into())]), files: vec![], pixel_formats: 1, host_abi_version: sys::ABI_VERSION }).unwrap();
        for (region, overseas, pal) in [("auto", true, false), ("md.us", true, false), ("md.eu", true, true), ("md.jp", false, false), ("md.asia", false, true)] {
            let m = made(region);
            let version = m.genesis.hw.io.read(0xA1_0001, 0);
            assert_eq!((version >> 7 & 1 != 0, version >> 6 & 1 != 0), (overseas, pal), "{region}");
            assert_eq!(m.machine_info().frame_rate, if pal { PAL_FRAME } else { NTSC_FRAME }, "{region}");
        }
        assert!(Machine::settings_schema().iter().all(|s| s.check().is_ok()));
        assert!(Machine::settings_schema().iter().filter(|s| [MODEL_KEY, REGION_KEY].contains(&s.key.as_str())).all(|s| matches!(s.scope, Scope::Create)));
    }

    /// A save RAM's battery file is Genesis Plus GX's: 64 KiB, each byte at its address less the RAM's even start, `$FF`
    /// between; it round-trips, and the one-lane form BlastEm writes, and PicoDrive's range in both lanes, are read too
    /// (Nephrite_Native.md §33).
    #[test]
    fn a_battery_file_is_genesis_plus_gxs_and_the_other_forms_are_read() {
        let mut ra = [0u8; 12];
        ra[..4].copy_from_slice(&[b'R', b'A', 0xF8, 0x20]);
        ra[4..8].copy_from_slice(&0x20_0001u32.to_be_bytes());
        ra[8..12].copy_from_slice(&0x20_03FFu32.to_be_bytes());
        let image = cartridge("SEGA GENESIS", "U", Some(ra));
        let made = |file: &[u8]| create(&image, vec![emusen_native::abi::File { which: 0, data: file }]).unwrap();
        let lane: Vec<u8> = (0..512u32).map(|i| (i * 7 + 3) as u8).collect();
        let mut gpgx = vec![0xFFu8; 0x1_0000];
        lane.iter().enumerate().for_each(|(i, &b)| gpgx[2 * i + 1] = b);
        let m = made(&lane);
        assert_eq!(m.battery(0).unwrap().0, &gpgx[..]);
        assert_eq!(made(m.battery(0).unwrap().0).battery(0).unwrap().0, &gpgx[..]);
        let mut pico = vec![0u8; 1024];
        lane.iter().enumerate().for_each(|(i, &b)| pico[2 * i + 1] = b);
        assert_eq!(made(&pico).battery(0).unwrap().0, &gpgx[..]);
        let mut m = made(&lane);
        assert_eq!(m.genesis.hw.cart.read8(0x20_0003), lane[1]);
        m.space_write(crate::machine::SRAM_ID, 1, &[0x5A]).unwrap();
        assert_eq!(m.battery(0).unwrap().0[3], 0x5A);
        let state = { let mut b = vec![0; m.state_size()]; let n = m.save_state(&mut b).unwrap(); b.truncate(n); b };
        m.space_write(crate::machine::SRAM_ID, 1, &[0x00]).unwrap();
        m.load_state(&state).unwrap();
        assert_eq!(m.battery(0).unwrap().0[3], 0x5A);
    }

    /// A save RAM on both lanes from an odd start, as Tecmo Super Bowl's header declares it, is filed by address too.
    #[test]
    fn a_two_lane_save_ram_is_filed_by_address() {
        let mut ra = [0u8; 12];
        ra[..4].copy_from_slice(&[b'R', b'A', 0xE0, 0x20]);
        ra[4..8].copy_from_slice(&0x20_0001u32.to_be_bytes());
        ra[8..12].copy_from_slice(&0x20_3FFFu32.to_be_bytes());
        let image = cartridge("SEGA GENESIS", "U", Some(ra));
        let mut m = create(&image, vec![]).unwrap();
        m.space_write(crate::machine::SRAM_ID, 0, &[0x10, 0x11]).unwrap();
        let file = m.battery(0).unwrap().0.to_vec();
        assert_eq!((file.len(), file[0], file[1], file[2], file[3]), (0x1_0000, 0xFF, 0x10, 0x11, 0xFF));
        let again = create(&image, vec![emusen_native::abi::File { which: 0, data: &file }]).unwrap();
        assert_eq!(again.battery(0).unwrap().0, &file[..]);
    }

    /// The model setting chooses the sound chip and the output circuit at create: model 1 unless model 2 is asked for.
    #[test]
    fn the_model_setting_chooses_the_sound_chip_and_circuit() {
        let image = cartridge("SEGA GENESIS", "U", None);
        let made = |settings: Vec<(String, String)>| Machine::create(&Create { image: &image, settings: Settings::from_pairs(settings), files: vec![], pixel_formats: 1, host_abi_version: sys::ABI_VERSION }).unwrap();
        let sound = |m: &Machine| (m.genesis.hw.sound.ym.discrete, m.genesis.hw.sound.circuit);
        assert_eq!(sound(&made(vec![])), (true, crate::sound::Circuit::MODEL1));
        assert_eq!(sound(&made(vec![(MODEL_KEY.into(), "md.model1".into())])), (true, crate::sound::Circuit::MODEL1));
        assert_eq!(sound(&made(vec![(MODEL_KEY.into(), "md.model2".into())])), (false, crate::sound::Circuit::MODEL2));
    }

    #[test]
    fn the_pad_settings_choose_each_ports_controller() {
        assert!(Machine::settings_schema().iter().all(|s| s.check().is_ok()));
        let image = cartridge("SEGA GENESIS", "U", None);
        let settings = Settings::from_pairs(vec![("pad2".into(), "md.pad6".into())]);
        let mut m = Machine::create(&Create { image: &image, settings, files: vec![], pixel_formats: 1, host_abi_version: sys::ABI_VERSION }).unwrap();
        let ports = |m: &Machine| m.machine_info().ports.iter().map(|p| p.controller.clone().unwrap()).collect::<Vec<_>>();
        assert_eq!(ports(&m), ["md.pad3", "md.pad6"]);
        m.set_settings(&Settings::from_pairs(vec![("pad1".into(), "md.pad6".into()), ("pad2".into(), "md.pad3".into())])).unwrap();
        assert_eq!(ports(&m), ["md.pad6", "md.pad3"]);
        m.advance();
        assert!(m.genesis.hw.io.pads[0].six && !m.genesis.hw.io.pads[1].six);
    }
}
