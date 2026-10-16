//! MarsRT on the core ABI v1: `Core` over the machine and its scan-out, the console's facts as descriptors, the settings C#
//! Mars offers as the schema, and the `marsrt_core_*` extensions the shim and WiseMan's tests call on a v1 handle. See
//! Mars_Native.md §7 and EmuSen_CoreAPI.md §18.6.

use std::sync::Arc;

use emusen_native::core::{
    self as v1, Accuracy, Audio, Axis, AxisControl, Battery, Button, Choice, Control, Controller, Create, Effect, FrameInfo, Info, MachineInfo, Port, Processor,
    Region, Scope, Setting, SettingKind, Settings, Space, StateFormat, System, Video, caps, event, flags, status,
};
use emusen_native::debug::Hooks;
use emusen_native::ffi::StateMachine;

use super::{Core, PHASES, STATUS_NOT_A_ROM, STATUS_NULL, space};
use crate::machine::{Machine, SNAPSHOT_VERSION, STATE_VERSION};
use crate::memory::bus::StateShape;
use crate::rom::RomImage;
use crate::state::{State, StateError, StateWriter};
use crate::vi::scan::{RASTER_WIDTH, Scanout};

/// `MarsDebugSpaces`' names, numbered as `ffi::space` numbers them.
pub const SPACES: [&str; 6] = ["RDRAM", "DMEM", "IMEM", "PIFRAM", "ROM", "CPU"];

/// `MarsCore.StickReach` and `CButtonThreshold`: a stick's full reach, and how far the right stick goes before a C button is held.
const STICK_REACH: f64 = 127.0;
const C_THRESHOLD: f64 = 0.5;

/// The joybus's button bits and the frontend's buttons, in `MarsCore.PadButtons`' order; the C buttons have no frontend button and
/// arrive on the right stick (Mars_Core.md §5).
const PAD: [(u32, Option<Control>, &str); 14] = [
    (11, Some(Control::Up), "Up"),
    (10, Some(Control::Down), "Down"),
    (9, Some(Control::Left), "Left"),
    (8, Some(Control::Right), "Right"),
    (15, Some(Control::A), "A"),
    (14, Some(Control::B), "B"),
    (12, Some(Control::Start), "Start"),
    (5, Some(Control::L), "L"),
    (4, Some(Control::R), "R"),
    (13, Some(Control::L2), "Z"),
    (3, None, "C Up"),
    (2, None, "C Down"),
    (1, None, "C Left"),
    (0, None, "C Right"),
];

const C_UP: u16 = 1 << 3;
const C_DOWN: u16 = 1 << 2;
const C_LEFT: u16 = 1 << 1;
const C_RIGHT: u16 = 1;

/// The file numbers: the cartridge's save, and the Controller Paks of ports 1 to 4, files 1 to 4.
pub const FILE_SAVE: u32 = 0;
pub const FILE_PAK: u32 = 1;
pub const PAKS: u32 = 4;
/// The battery files a machine can name.
pub const FILES: usize = 1 + PAKS as usize;

/// The battery file's extension and the paks', `MarsCore.PakExtension` for the first port, the names C# Mars and MarsRT have
/// always written, and `.2.mpk` to `.4.mpk` for the others (EmuSen_SaveMedia.md §7.2).
const SAVE_SUFFIX: &str = ".srm";
const PAK_SUFFIXES: [&str; PAKS as usize] = [".mpk", ".2.mpk", ".3.mpk", ".4.mpk"];
const PAK_LABELS: [&str; PAKS as usize] = ["Controller 1", "Controller 2", "Controller 3", "Controller 4"];

/// The pak a file number names, by its port.
fn pak_port(which: u32) -> Option<usize> {
    (FILE_PAK..FILE_PAK + PAKS).contains(&which).then(|| (which - FILE_PAK) as usize)
}

/// The run-scope settings as one value, so a group is applied only when it changed (Mars_Native.md §6.4.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Values {
    pub idle_skip: bool,
    pub rsp_whole: bool,
    pub threaded: bool,
    pub deferred: bool,
    pub skip_repeated: bool,
    pub verify_rdp: bool,
    pub workers: u32,
    pub recompiler: bool,
    pub verify_blocks: bool,
    pub tier: u32,
    pub scale: i32,
    pub antialiasing: i32,
    pub gpu: bool,
}

impl Values {
    /// These, with the keys `settings` names replaced; every value has been checked against the schema.
    fn with(mut self, s: &Settings) -> Values {
        let switch = |key: &str, was: bool| s.get(key).map_or(was, |v| v == "true");
        let count = |key: &str, was: i64| s.get(key).and_then(|v| v.parse::<i64>().ok()).unwrap_or(was);
        self.idle_skip = switch(IDLE_SKIP, self.idle_skip);
        self.rsp_whole = switch(RSP_WHOLE, self.rsp_whole);
        self.threaded = switch(THREADED_RDP, self.threaded);
        self.deferred = switch(DEFERRED, self.deferred);
        self.skip_repeated = switch(SKIP_REPEATED, self.skip_repeated);
        self.verify_rdp = switch(VERIFY_RDP, self.verify_rdp);
        self.workers = count(RDP_WORKERS, self.workers as i64) as u32;
        self.recompiler = switch(RECOMPILER, self.recompiler);
        self.verify_blocks = switch(VERIFY_BLOCKS, self.verify_blocks);
        self.tier = count(BLOCK_TIER, self.tier as i64) as u32;
        self.scale = count(RENDER_SCALE, self.scale as i64) as i32;
        if let Some(v) = s.get(ANTIALIASING) {
            self.antialiasing = if v == "Off" { 1 } else { v[..1].parse().unwrap_or(1) };
        }
        self.gpu = switch(GPU, self.gpu);
        self
    }
}

pub const EXPANSION_PAK: &str = "ExpansionPak";
pub const BOOT: &str = "Boot";
pub const THREADED_RDP: &str = "ThreadedRdp";
pub const RDP_WORKERS: &str = "RdpWorkers";
pub const DEFERRED: &str = "DeferredPresentation";
pub const SKIP_REPEATED: &str = "SkipRepeatedScans";
pub const RENDER_SCALE: &str = "RenderScale";
pub const ANTIALIASING: &str = "Antialiasing";
pub const GPU: &str = "Gpu";
pub const RECOMPILER: &str = "Recompiler";
pub const IDLE_SKIP: &str = "IdleSkip";
pub const RSP_WHOLE: &str = "RspWhole";
pub const VERIFY_BLOCKS: &str = "VerifyBlocks";
pub const BLOCK_TIER: &str = "BlockTier";
pub const VERIFY_RDP: &str = "VerifyRdp";

/// `MarsCore`'s default for `RdpWorkers`: one per three processors, one to four.
fn default_workers() -> i64 {
    (std::thread::available_parallelism().map_or(1, |n| n.get()) as i64 / 3).clamp(1, 4)
}

fn switch(key: &str, label: &str, help: &str, default: bool, scope: Scope, effect: Effect) -> Setting {
    Setting { key: key.into(), label: label.into(), help: help.into(), kind: SettingKind::Switch, default: default.to_string(), scope, category: None, effect, advanced: false, hidden: false, restart: scope == Scope::Create }
}

fn hidden(mut s: Setting) -> Setting {
    s.hidden = true;
    s.advanced = true;
    s.category = Some("Tests".into());
    s
}

fn choices(values: &[(&str, &str)]) -> SettingKind {
    SettingKind::Choice(values.iter().map(|&(value, label)| Choice { value: value.into(), label: label.into(), help: None }).collect())
}

/// The settings, in `MarsCore.VideoSettings`' order with MarsRT's words where they differ, then MarsRT's own (Mars_Native.md §7.2).
pub fn schema() -> Vec<Setting> {
    let exact = || Effect::Exact;
    let mut list = vec![
        switch(THREADED_RDP, "Draw on a separate thread", "The display processor runs its lists on a thread of its own, behind marks on the memory it reaches. Exact: frame for frame the machine on one thread.", true, Scope::Run, exact()),
        Setting {
            key: RDP_WORKERS.into(),
            label: "Rasteriser threads".into(),
            help: "How many processors share each list when it runs on a thread of its own, each shading every Nth row. Exact at any count: frame for frame the machine on one thread. One per three cores is the default, as on Mars.".into(),
            kind: SettingKind::Count { min: 1, max: 8, step: 1 },
            default: default_workers().to_string(),
            scope: Scope::Run,
            category: None,
            effect: exact(),
            advanced: false,
            hidden: false,
            restart: false,
        },
        switch(DEFERRED, "Scan out while the next frame runs", "The picture is finished on another thread while the machine runs the next frame, so it reaches the screen one frame late, exactly the picture it would have been.", true, Scope::Run, Effect::Latency { cost: "The picture reaches the screen one frame later.".into() }),
        switch(SKIP_REPEATED, "Skip a scan that repeats the last", "A scan whose registers and bytes match the last walk is not walked again. Exact; the picture is the same either way.", true, Scope::Run, exact()),
        Setting {
            key: RENDER_SCALE.into(),
            label: "Internal resolution".into(),
            help: "The picture drawn at a multiple of the console's, beside the exact drawing games read back. Each step costs its square in drawing: 2x is four times the pixels, 4x sixteen. Threads help; 2x is what most machines can hold at full speed.".into(),
            kind: choices(&[("1", "1"), ("2", "2"), ("3", "3"), ("4", "4")]),
            default: "1".into(),
            scope: Scope::Run,
            category: None,
            effect: Effect::Enhancement { cost: "Each step costs its square in drawing.".into(), hardware: "1".into() },
            advanced: false,
            hidden: false,
            restart: false,
        },
        Setting {
            key: ANTIALIASING.into(),
            label: "Antialiasing".into(),
            help: "Each pixel of the picture averaged from a drawing that many times finer each way, which smooths edges and shimmering textures. It multiplies the drawing's cost like the internal resolution, and the two together are held to four: at 2x resolution the most is 2x.".into(),
            kind: choices(&[("Off", "Off"), ("2x", "2x"), ("3x", "3x"), ("4x", "4x")]),
            default: "Off".into(),
            scope: Scope::Run,
            category: None,
            effect: Effect::Enhancement { cost: "It multiplies the drawing's cost like the internal resolution.".into(), hardware: "Off".into() },
            advanced: false,
            hidden: false,
            restart: false,
        },
        switch(GPU, "Draw the multiple on the graphics card", "The picture at an internal resolution above one is shaded by the graphics card rather than by the processor's own threads, which is where nearly all of that setting's cost is. It needs Vulkan; without it the setting does nothing and the processor draws as before. It changes nothing at 1x, and the drawing the game itself reads back is never the card's.", false, Scope::Run, exact()),
        switch(EXPANSION_PAK, "Expansion Pak", "The memory accessory that doubles the console's 4MB. A few games refuse to start without it and more use it when it is there. Takes effect when a game is next loaded; a save state resumes with the memory it was made with.", true, Scope::Create, Effect::None),
        switch(RECOMPILER, "Compile the processor's code", "The processor's code is compiled to the host's machine code in blocks, each compared with memory before it runs, on a thread of its own. Exact: frame for frame the interpreter, which is what off leaves running, slower.", true, Scope::Run, exact()),
        hidden(switch(IDLE_SKIP, "Pass the idle loop whole", "The idle loop passed in one piece, as C#'s Cpu.SkipIdle. Exact either way.", true, Scope::Run, exact())),
        hidden(switch(RSP_WHOLE, "Run the signal processor whole inside the idle loop", "A running signal processor run to its next event inside the idle loop, as C#'s RunBlocks. Exact either way.", true, Scope::Run, exact())),
        hidden(switch(VERIFY_BLOCKS, "Check the compiled blocks", "The interpreter run beside the blocks and compared after every instruction; for tests.", false, Scope::Run, exact())),
        hidden(Setting {
            key: BLOCK_TIER.into(),
            label: "Recompiler tier".into(),
            help: "Which step of the recompiler runs: 0 the furthest built, 1 decoded blocks, 2 compiled, 3 compiled with registers held; for tests.".into(),
            kind: SettingKind::Count { min: 0, max: 15, step: 1 },
            default: "0".into(),
            scope: Scope::Run,
            category: None,
            effect: exact(),
            advanced: true,
            hidden: true,
            restart: false,
        }),
        hidden(switch(VERIFY_RDP, "Check the drain's bytes", "Every byte the display processor's thread touches checked against the marks, as Mars's EMUSEN_MARS_VERIFY_RDP; for tests.", false, Scope::Run, exact())),
        hidden(switch(BOOT, "Boot as the corpus boots", "The boot code handed off and nothing loaded beside it, neither the save chip nor the pak, as the test corpus boots a machine; for tests.", false, Scope::Create, Effect::None)),
    ];
    list.retain(|s| s.check().is_ok());
    list
}

impl Values {
    /// The run-scope values the schema's defaults give.
    pub fn defaults() -> Values {
        defaults()
    }
}

/// The run-scope values the schema's defaults give.
fn defaults() -> Values {
    Values {
        idle_skip: true,
        rsp_whole: true,
        threaded: true,
        deferred: true,
        skip_repeated: true,
        verify_rdp: false,
        workers: default_workers() as u32,
        recompiler: true,
        verify_blocks: false,
        tier: 0,
        scale: 1,
        antialiasing: 1,
        gpu: false,
    }
}

fn pad() -> Controller {
    Controller {
        id: "n64.pad".into(),
        label: "Controller".into(),
        ports: vec![0, 1, 2, 3],
        buttons: PAD.iter().map(|&(bit, control, label)| Button { bit, control, label: label.into() }).collect(),
        axes: vec![
            AxisControl { axis: 0, control: Some(Axis::LeftX), trigger: false, label: "Stick X".into() },
            AxisControl { axis: 1, control: Some(Axis::LeftY), trigger: false, label: "Stick Y".into() },
            AxisControl { axis: 2, control: Some(Axis::RightX), trigger: false, label: "C Left and Right".into() },
            AxisControl { axis: 3, control: Some(Axis::RightY), trigger: false, label: "C Up and Down".into() },
        ],
    }
}

/// The files a machine of the system can name: the cartridge's save chip, the game's, and a Controller Pak in each port, which a
/// player carries between games (EmuSen_SaveMedia.md §7.2).
fn battery() -> Vec<Battery> {
    let paks = (0..PAKS).map(|n| Battery::removable(FILE_PAK + n, PAK_SUFFIXES[n as usize]).medium("n64.pak").label(PAK_LABELS[n as usize]).port(n));
    std::iter::once(Battery::game(FILE_SAVE, SAVE_SUFFIX)).chain(paks).collect()
}

/// The blank picture shown before the first scan, as C#'s `Blank(DefaultScreenHeight)`: 640 by 480, black and opaque.
const BLANK_HEIGHT: usize = 480;

fn blank() -> &'static [u8] {
    static BLANK: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    BLANK.get_or_init(|| [0u8, 0, 0, 0xFF].repeat(RASTER_WIDTH * BLANK_HEIGHT))
}

impl Core {
    /// The run-scope values applied, each group only when it changed or on the first call, in C#'s `ApplyOptions` order: the options,
    /// the threads, the recompiler, the multiple (Mars_Native.md §6.4.8).
    pub fn apply(&mut self, want: Values) {
        let fresh = self.applied.is_none();
        let was = self.applied.unwrap_or(want);
        if fresh || (want.idle_skip, want.rsp_whole) != (was.idle_skip, was.rsp_whole) {
            self.machine.options.idle_skip = want.idle_skip;
            self.machine.options.rsp_whole = want.rsp_whole;
        }
        if fresh || (want.threaded, want.deferred, want.skip_repeated, want.verify_rdp, want.workers) != (was.threaded, was.deferred, was.skip_repeated, was.verify_rdp, was.workers) {
            self.set_threads(want.threaded, want.deferred, !want.skip_repeated, want.verify_rdp, want.workers);
        }
        if fresh || (want.recompiler, want.verify_blocks, want.tier) != (was.recompiler, was.verify_blocks, was.tier) {
            self.set_recompiler(want.recompiler, want.verify_blocks, want.tier);
        }
        if fresh || (want.scale, want.antialiasing, want.gpu) != (was.scale, was.antialiasing, was.gpu) {
            self.set_multiple(want.scale, want.antialiasing, want.gpu);
        }
        self.applied = Some(want);
    }

    /// The signal processor's coverage armed as the hooks say, and what it recorded since the last drain moved into the hooks'
    /// bitmap for processor 1, which `debug_coverage` drains (Mars_Native.md §7.3).
    fn sync_rsp_coverage(&mut self) {
        let hooks = &mut self.machine.cpu.hooks.shared;
        let trace = &mut self.machine.bus.sp.trace;
        if let Some(Some(c)) = hooks.coverage.get_mut(1) {
            match trace.as_mut() {
                Some(t) => {
                    for (into, from) in c.bits.iter_mut().zip(t.bits.iter_mut()) {
                        *into |= *from;
                        *from = 0;
                    }
                    c.covered += t.recorded;
                    t.recorded = 0;
                }
                None => **trace = Some(Box::default()),
            }
        } else if trace.is_some() {
            **trace = None;
        }
    }

    /// `STATE_SIZE` for both kinds when a frame changed what the state's length depends on, as a game's first touch of its save chip
    /// does; the crate measures sizes only after a load (EmuSen_CoreAPI.md §6.9, §6.19). A change of the registers written alone moves
    /// both sizes by eight bytes a register, which needs no measuring: a game writing at random can add registers every frame.
    fn tell_state_size(&mut self, before: StateShape) {
        let now = self.machine.bus.state_shape();
        if now == before {
            return;
        }
        let (full, snapshot) = match self.told {
            Some((shape, full, snapshot)) if full > 0 && snapshot > 0 && StateShape { registers: now.registers, ..shape } == now => {
                let moved = 8 * (now.registers as i64 - shape.registers as i64);
                (full + moved, snapshot + moved)
            }
            _ => (self.state(false).map_or(0, |n| n as i64), self.state(true).map_or(0, |n| n as i64)),
        };
        self.told = Some((now, full, snapshot));
        v1::emit(event::STATE_SIZE, full, 0);
        v1::emit(event::STATE_SIZE, snapshot, 1);
    }

    /// The paks the host keeps apart put back as they stood before a load, their `Dirty` as it was, so the load writes no file; a
    /// port whose pak differs from the state's is told a new one was inserted (EmuSen_CoreAPI.md §6.27, EmuSen_SaveMedia.md §7.2).
    fn keep_paks(&mut self, kept: [Option<crate::memory::controller::ControllerPak>; 4]) {
        for (port, pak) in self.machine.bus.si.controllers.iter_mut().zip(kept) {
            let differs = match (&port.pak, &pak) {
                (Some(state), Some(kept)) => state.data != kept.data,
                (None, None) => false,
                _ => true,
            };
            *port.inserted |= differs && pak.is_some();
            port.pak = pak;
        }
    }

    fn state(&self, snapshot: bool) -> Result<usize, StateError> {
        self.machine.state_size(snapshot)
    }
}

impl StateMachine for Core {
    type Error = StateError;

    /// `MarsCore.LoadState`: the fields, everything a C# load derives from them, then the picture; a state of the other RDRAM size
    /// rebuilds the machine, and the scan-out with it.
    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        let rdram = self.machine.rdram_bytes();
        // A walk still out reads the device's pictures, which a state read empties (Mars_Gpu.md §14.3).
        if self.machine.join_presentation(&mut self.scanout) {
            self.frame_serial += 1;
        }
        let kept = self.paks_apart.then(|| self.machine.bus.si.controllers.each_ref().map(|c| c.pak.clone()));
        self.machine.restore_state(data)?;
        let was = self.unsaved;
        self.unsaved = [true; FILES];
        if let Some(kept) = kept {
            self.keep_paks(kept);
            self.unsaved[FILE_PAK as usize..].copy_from_slice(&was[FILE_PAK as usize..]);
        }
        if self.machine.rdram_bytes() != rdram {
            self.scanout = Scanout::default();
            v1::emit(event::MACHINE_INFO, 0, 0);
        }
        self.scanout.forget();
        if !self.skip_rendering {
            let presented = self.machine.present_now(&mut self.scanout);
            self.shown = presented.walked;
            self.frame_serial += 1;
        }
        Ok(())
    }

    /// A state while words are pending has no size, so it is reported as none and the save that follows gives the reason.
    fn state_size(&self) -> usize {
        self.state(false).unwrap_or(0)
    }

    fn save_state(&self, out: &mut [u8]) -> Result<usize, StateError> {
        self.machine.save_state(out, false)
    }

    fn layout(&self) -> String {
        self.machine.layout(false).unwrap_or_default()
    }
}

impl v1::Core for Core {
    const CAPABILITIES: u64 = caps::PRESENT
        | caps::PHASES
        | caps::AUDIO_PEEK
        | caps::AXES
        | caps::SETTINGS
        | caps::SNAPSHOT
        | caps::FRAME_SERIAL
        | caps::ROW_REPEAT
        | caps::BATTERY_DIRTY
        | caps::ROM_PATCHES
        | caps::DEBUG;

    fn info() -> Info {
        Info {
            id: "marsrt".into(),
            name: "MarsRT".into(),
            display_name: Some("MarsRT (Rust)".into()),
            version: env!("CARGO_PKG_VERSION").into(),
            license: "GPL-3.0-or-later".into(),
            authors: vec!["EmuSen".into()],
            description: Some("A Rust port of Mars, the project's Nintendo 64 core, exact against it in state, picture and sound, with a recompiler and the display processor on threads of its own.".into()),
            systems: vec![System {
                id: "n64".into(),
                name: "Nintendo 64".into(),
                extensions: vec![".z64".into(), ".n64".into(), ".v64".into()],
                regions: vec![Region::Ntsc, Region::Pal],
                controllers: vec![pad()],
                firmware: Vec::new(),
                development: false,
            }],
            deterministic: true,
            accuracy: Some(Accuracy { measured_with: "defaults".into(), suite: "Mars (C#), the reference it is graded against".into(), notes: String::new() }),
            ..Info::default()
        }
    }

    fn settings_schema() -> Vec<Setting> {
        schema()
    }

    fn status_text(code: i32) -> Option<String> {
        let words = match code {
            STATUS_NOT_A_ROM => "not a Nintendo 64 cartridge image",
            -12 => "an RDRAM size that is neither 4 MB nor 8 MB",
            -13 => "more pending display-processor words than a snapshot holds",
            -14 => "pending display-processor words, which only a snapshot can carry",
            _ => return None,
        };
        Some(words.to_owned())
    }

    /// `MarsCore.LoadRom`: the machine with the Expansion Pak or without, the save chip and the first port's pak from their files,
    /// then every run-scope setting applied as a fresh machine of C#'s is; with `Boot`, the corpus's hand-off and nothing beside it.
    fn create(r: &Create<'_>) -> Result<Self, i32> {
        let image = RomImage::from_image(r.image).map_err(|e| {
            v1::detail(&format!("not a Nintendo 64 image: {e:?}"));
            STATUS_NOT_A_ROM
        })?;
        let expansion = r.settings.get(EXPANSION_PAK) == Some("true");
        let mut saved = None;
        let mut paks: [Option<&[u8]>; PAKS as usize] = [None; PAKS as usize];
        for file in &r.files {
            match (file.which, pak_port(file.which)) {
                (FILE_SAVE, _) => saved = Some(file.data.to_vec()),
                (_, Some(port)) => paks[port] = Some(file.data),
                (which, None) => {
                    v1::detail(&format!("file {which} is not one MarsRT takes"));
                    return Err(status::BAD_FILE);
                }
            }
        }
        let machine = if r.settings.get(BOOT) == Some("true") {
            Machine::boot(Arc::new(image), expansion)
        } else {
            let mut m = Machine::load_rom(Arc::new(image), expansion, saved, paks[0].map(<[u8]>::to_vec));
            m.insert_paks([paks[1], paks[2], paks[3]], r.shared_removable);
            m
        };
        let mut core = Core::new(machine);
        core.paks_apart = r.shared_removable;
        core.apply(defaults().with(&r.settings));
        Ok(core)
    }

    fn machine_info(&self) -> MachineInfo {
        let bus = &self.machine.bus;
        let pal = bus.cart.as_ref().is_some_and(|c| c.is_pal);
        let rom = bus.cart.as_ref().map_or(0, |c| c.rom.len() as u32);
        MachineInfo {
            system: "n64".into(),
            region: Some(if pal { Region::Pal } else { Region::Ntsc }),
            frame_rate: if pal { (50, 1) } else { (60_000, 1001) },
            video: Video { base_width: 320, base_height: 240, max_width: (RASTER_WIDTH * 4) as u32, max_height: 576 * 4, aspect: (4, 3), formats: Vec::new() },
            audio: Audio { rate: crate::memory::ai::DEFAULT_SAMPLE_RATE, channels: Vec::new() },
            ports: (0..4).map(|port| Port { port, controller: Some("n64.pad".into()) }).collect(),
            spaces: SPACES
                .iter()
                .enumerate()
                .map(|(id, &name)| {
                    let id = id as u32;
                    Space {
                        read_only: id == space::ROM,
                        cheats: matches!(id, space::RDRAM | space::DMEM | space::IMEM | space::PIF_RAM),
                        reports_stores: matches!(id, space::RDRAM | space::DMEM | space::IMEM | space::PIF_RAM),
                        view: id == space::CPU,
                        ..Space::new(id, name)
                    }
                })
                .collect(),
            processors: vec![
                Processor { id: 0, name: "CPU".into(), pc_bits: 32, registers: Vec::new(), code_space: Some(space::CPU) },
                Processor { id: 1, name: "RSP".into(), pc_bits: 12, registers: Vec::new(), code_space: Some(space::IMEM) },
            ],
            battery: battery(),
            state: StateFormat { format: "MARS".into(), version: STATE_VERSION as i64, loads_from: vec![STATE_VERSION as i64, SNAPSHOT_VERSION as i64] },
            phases: ["machine", "machine/rdp-wait", "present", "present/rdp-wait", "present/walk-join"].map(String::from).to_vec(),
            patches: (rom > 0).then(|| (0, rom - 1)),
            skip_rendering_state_neutral: false,
            firmware: Vec::new(),
        }
    }

    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        let shape = self.machine.bus.state_shape();
        Core::advance(self);
        self.tell_state_size(shape);
        Ok(())
    }

    fn present(&mut self) -> Result<(), i32> {
        self.present_profiled();
        Ok(())
    }

    fn set_options(&mut self, bits: u32) {
        self.skip_rendering = bits & flags::OPTION_SKIP_RENDERING != 0;
    }

    fn frame_count(&self) -> i64 {
        self.machine.total_frames
    }

    fn phases(&self) -> Vec<i64> {
        self.profile.to_vec()
    }

    fn frame_info(&self) -> FrameInfo {
        let (width, height, repeat) = if self.scanout.width == 0 { (RASTER_WIDTH as u32, BLANK_HEIGHT as u32, 1) } else { (self.scanout.width, self.scanout.height, self.scanout.row_repeat.max(1)) };
        FrameInfo {
            row_repeat: repeat as i32,
            flags: if self.shown { flags::FRAME_WALKED } else { 0 },
            aspect_num: 4,
            aspect_den: 3,
            serial: self.frame_serial,
            ..FrameInfo::rgba(width as i32, height as i32)
        }
    }

    fn frame(&self) -> &[u8] {
        if self.scanout.width == 0 { blank() } else { &self.scanout.frame }
    }

    fn audio_rate(&self) -> i32 {
        self.machine.bus.ai.sample_rate(self.machine.bus.vi.video_clock())
    }

    fn audio_buffered(&self) -> usize {
        self.machine.bus.ai.samples.len()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        let samples = &mut self.machine.bus.ai.samples;
        let mut n = max_frames.saturating_mul(2).min(samples.len()).min(out.len());
        n -= n & 1;
        for (to, from) in out[..n].iter_mut().zip(samples.drain(..n)) {
            *to = from;
        }
        n
    }

    fn set_audio_limit(&mut self, samples: usize) {
        *self.machine.bus.ai.limit = samples.max(2);
    }

    fn peek_audio(&self, out: &mut [i16]) -> usize {
        let samples = &self.machine.bus.ai.samples;
        for (to, &from) in out.iter_mut().zip(samples.iter()) {
            *to = from;
        }
        samples.len()
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        let c = self.machine.bus.si.controllers.get_mut(port as usize).ok_or(status::NO_SUCH_PORT)?;
        c.buttons = (c.buttons & !(changed as u16)) | (mask & changed) as u16;
        Ok(())
    }

    /// `MarsCore.SetAxis`: the stick to its reach, rounded as C# rounds, and the right stick's halves as the C buttons.
    fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), i32> {
        let c = self.machine.bus.si.controllers.get_mut(port as usize).ok_or(status::NO_SUCH_PORT)?;
        let value = value.clamp(-1.0, 1.0);
        let hold = |buttons: &mut u16, bit: u16, on: bool| *buttons = if on { *buttons | bit } else { *buttons & !bit };
        match axis {
            0 => c.stick_x = (value * STICK_REACH).round_ties_even() as i8,
            1 => c.stick_y = (-value * STICK_REACH).round_ties_even() as i8,
            2 => {
                hold(&mut c.buttons, C_LEFT, value <= -C_THRESHOLD);
                hold(&mut c.buttons, C_RIGHT, value >= C_THRESHOLD);
            }
            3 => {
                hold(&mut c.buttons, C_UP, value <= -C_THRESHOLD);
                hold(&mut c.buttons, C_DOWN, value >= C_THRESHOLD);
            }
            _ => return Err(status::NOT_SUPPORTED),
        }
        Ok(())
    }

    fn settle(&mut self) {
        self.machine.settle();
    }

    fn snapshot_size(&self) -> usize {
        self.state(true).unwrap_or(0)
    }

    fn save_snapshot(&mut self, out: &mut [u8]) -> Result<usize, i32> {
        self.machine.save_state(out, true).map_err(|e| e.status())
    }

    fn snapshot_layout(&self) -> String {
        self.machine.layout(true).unwrap_or_default()
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        if space == space::CPU {
            return Ok(1 << 32);
        }
        self.memory(space).map(|m| m.len() as i64).ok_or(status::NO_SUCH_SPACE)
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        self.read_memory(space, address, out)
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        self.write_memory(space, address, data).map(|_| ())
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        let unsaved = self.unsaved.get(which as usize).copied().unwrap_or(false);
        let changed = |dirty: bool| flags::BATTERY_TRACKED | if dirty || unsaved { flags::BATTERY_CHANGED } else { 0 };
        let bus = &self.machine.bus;
        match which {
            FILE_SAVE => Ok((bus.save.contents().unwrap_or(&[]), changed(bus.save.dirty()))),
            which => {
                let port = pak_port(which).ok_or(status::BAD_FILE)?;
                Ok(bus.si.controllers[port].pak.as_ref().map_or((&[][..], flags::BATTERY_TRACKED), |p| (&p.data[..], changed(p.dirty))))
            }
        }
    }

    fn battery_saved(&mut self, which: u32) -> Result<(), i32> {
        if let Some(unsaved) = self.unsaved.get_mut(which as usize) {
            *unsaved = false;
        }
        match which {
            FILE_SAVE => self.machine.bus.save.set_dirty(false),
            which => {
                let port = pak_port(which).ok_or(status::BAD_FILE)?;
                if let Some(pak) = &mut self.machine.bus.si.controllers[port].pak {
                    pak.dirty = false;
                }
            }
        }
        Ok(())
    }

    fn set_rom_patches(&mut self, triples: &[u32]) -> Result<(), i32> {
        Core::set_rom_patches(self, triples);
        Ok(())
    }

    fn set_settings(&mut self, settings: &Settings) -> Result<(), i32> {
        let now = self.applied.unwrap_or_else(defaults);
        self.apply(now.with(settings));
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut Hooks> {
        self.sync_rsp_coverage();
        Some(&mut self.machine.cpu.hooks.shared)
    }

    /// `RunFrame`'s observed loop, its phases kept as the plain frame keeps them; bit 0 runs the first instruction unchecked, bit 1
    /// continues the frame the last call stopped inside.
    fn debug_run_frame(&mut self, bits: u32, _detail: &mut u64) -> Result<u32, i32> {
        self.sync_rsp_coverage();
        self.machine.cpu.hooks.settle_flags();
        if bits & emusen_native::debug::run::CONTINUE == 0 {
            self.profile = [0; PHASES];
        }
        let (started, waited) = (std::time::Instant::now(), self.drain_waited());
        let shape = self.machine.bus.state_shape();
        let why = self.machine.run_frame_debug(bits & emusen_native::debug::run::UNCHECKED != 0, bits & emusen_native::debug::run::CONTINUE != 0);
        self.tell_state_size(shape);
        self.profile[0] += started.elapsed().as_nanos() as i64;
        self.profile[1] += (self.drain_waited() - waited).max(0);
        Ok(why)
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        match processor {
            0 => Some(self.machine.cpu.pc),
            1 => Some(self.machine.bus.sp.processor.pc as u64),
            _ => None,
        }
    }

    fn battery_files(system: &str) -> Option<Vec<Battery>> {
        (system == "n64").then(battery)
    }
}

emusen_native::core_exports!(Core; present, phases, audio_peek, axes, settings, rom_patches, debug);

// ---- Extensions: what the shim reads beside the ABI, and the tests' ways into the machine -------------------------------------

/// A live handle's core, or `None` for null.
///
/// # Safety
/// `machine` must be null or a live handle `emusen_core_create` returned.
unsafe fn of<'a>(machine: *const v1::sys::Machine) -> Option<&'a Core> {
    unsafe { v1::exports::core_of::<Core>(machine) }
}

/// # Safety
/// As `of`.
unsafe fn of_mut<'a>(machine: *mut v1::sys::Machine) -> Option<&'a mut Core> {
    unsafe { v1::exports::core_of_mut::<Core>(machine) }
}

/// `values` copied up to `len` into `out` when it is not null; returns how many there are.
///
/// # Safety
/// `out` valid for `len` values, or null.
unsafe fn copy<T: Copy>(values: &[T], out: *mut T, len: usize) -> i64 {
    if !out.is_null() {
        unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), out, values.len().min(len)) };
    }
    values.len() as i64
}

/// Cycles, total frames, the last frame's cycles, instructions, and the idle loop's turns and instructions, up to `len`; returns 6.
///
/// # Safety
/// `machine` a live v1 handle or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_counters(machine: *const v1::sys::Machine, out: *mut i64, len: usize) -> i64 {
    let Some(c) = (unsafe { of(machine) }) else { return STATUS_NULL as i64 };
    let m = &c.machine;
    unsafe { copy(&[m.bus.cycles, m.total_frames, m.last_frame_cycles, m.cpu.instructions, m.cpu.run.idle_turns_passed, m.cpu.run.idle_instructions], out, len) }
}

/// The display processor's thread counters in `Core::thread_counters`' order, up to `len`; returns how many there are.
///
/// # Safety
/// As `marsrt_core_counters`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_thread_counters(machine: *const v1::sys::Machine, out: *mut i64, len: usize) -> i64 {
    unsafe { of(machine) }.map_or(STATUS_NULL as i64, |c| unsafe { copy(&c.thread_counters(), out, len) })
}

/// The recompiler's counters in `Blocks::counters`' order, up to `len`; returns how many there are.
///
/// # Safety
/// As `marsrt_core_counters`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_block_counters(machine: *const v1::sys::Machine, out: *mut i64, len: usize) -> i64 {
    unsafe { of(machine) }.map_or(STATUS_NULL as i64, |c| unsafe { copy(&c.block_counters(), out, len) })
}

/// `MarsCore.GpuReport` as UTF-8, copied up to `len` bytes; returns its whole length.
///
/// # Safety
/// `machine` a live v1 handle or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_gpu_report(machine: *const v1::sys::Machine, out: *mut u8, len: usize) -> i64 {
    unsafe { of(machine) }.map_or(STATUS_NULL as i64, |c| unsafe { copy(c.gpu_report().as_bytes(), out, len) })
}

/// The IS-Viewer's transcript, copied up to `len` bytes; returns its whole length.
///
/// # Safety
/// As `marsrt_core_gpu_report`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_is_viewer_text(machine: *const v1::sys::Machine, out: *mut u8, len: usize) -> i64 {
    unsafe { of(machine) }.map_or(STATUS_NULL as i64, |c| unsafe { copy(&c.machine.bus.is_viewer.text[..], out, len) })
}

/// The interpreter alone for `steps` instructions, as the corpus runs; through the blocks when they are on.
///
/// # Safety
/// `machine` a live v1 handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_run_steps(machine: *mut v1::sys::Machine, steps: u64) -> i32 {
    let Some(c) = (unsafe { of_mut(machine) }) else { return STATUS_NULL };
    c.machine.run_steps(steps);
    0
}

/// The CPU's fields alone, as C#'s `StateSerializer.Write(w, cpu)` writes them; returns the bytes, or a negative status.
///
/// # Safety
/// `machine` a live v1 handle or null; `out` valid for `len` bytes, or null to ask the length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_save_cpu(machine: *const v1::sys::Machine, out: *mut u8, len: usize) -> i64 {
    let Some(c) = (unsafe { of(machine) }) else { return STATUS_NULL as i64 };
    if out.is_null() {
        let mut w = StateWriter::counter();
        c.machine.cpu.write_state(&mut w);
        return w.len() as i64;
    }
    let mut w = StateWriter::new(unsafe { std::slice::from_raw_parts_mut(out, len) });
    c.machine.cpu.write_state(&mut w);
    if w.overflowed() { status::BUFFER_TOO_SMALL as i64 } else { w.len() as i64 }
}

/// One COP0 register, `Cop0[register]`; the cheat gate reads Status (12).
///
/// # Safety
/// `machine` a live v1 handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_cop0(machine: *const v1::sys::Machine, register: u32) -> u64 {
    unsafe { of(machine) }.map_or(0, |c| c.machine.cpu.cop0[(register & 31) as usize])
}

/// The CPU's registers for a debugger: PC, the 32 GPRs, HI, LO and the bus's cycles, up to `len`; returns 36.
///
/// # Safety
/// `machine` a live v1 handle or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_cpu_registers(machine: *const v1::sys::Machine, out: *mut u64, len: usize) -> i64 {
    let Some(c) = (unsafe { of(machine) }) else { return STATUS_NULL as i64 };
    let cpu = &c.machine.cpu;
    let mut values = [0u64; 36];
    values[0] = cpu.pc;
    values[1..33].copy_from_slice(&cpu.gpr);
    values[33..].copy_from_slice(&[cpu.hi, cpu.lo, c.machine.bus.cycles as u64]);
    unsafe { copy(&values, out, len) }
}

/// The signal processor's registers for a debugger: PC, halted, broke, and the 32 scalar registers, up to `len`; returns 35.
///
/// # Safety
/// As `marsrt_core_cpu_registers`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_rsp_registers(machine: *const v1::sys::Machine, out: *mut u32, len: usize) -> i64 {
    let Some(c) = (unsafe { of(machine) }) else { return STATUS_NULL as i64 };
    let rsp = &c.machine.bus.sp.processor;
    let mut values = [0u32; 35];
    values[..3].copy_from_slice(&[rsp.pc, rsp.halted as u32, rsp.broke as u32]);
    values[3..].copy_from_slice(&rsp.gpr);
    unsafe { copy(&values, out, len) }
}

/// The VI's fourteen registers as stored, up to `len`; returns 14.
///
/// # Safety
/// As `marsrt_core_cpu_registers`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_vi_registers(machine: *const v1::sys::Machine, out: *mut u32, len: usize) -> i64 {
    let Some(c) = (unsafe { of(machine) }) else { return STATUS_NULL as i64 };
    let values: Vec<u32> = (0..crate::vi::REGISTERS).map(|i| c.machine.bus.vi.read32(i * 4)).collect();
    unsafe { copy(&values, out, len) }
}

/// `MarsDebugSpaces.TryPhysical`: kernel mode's view of a 32-bit virtual address; returns 1 and the physical address, or 0 where
/// the TLB has no entry.
///
/// # Safety
/// `machine` a live v1 handle or null; `physical` valid for one value, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_physical(machine: *const v1::sys::Machine, address: u32, physical: *mut u32) -> u32 {
    let Some(c) = (unsafe { of(machine) }) else { return 0 };
    match c.physical(address) {
        Some(p) => {
            if let Some(out) = unsafe { physical.as_mut() } {
                *out = p;
            }
            1
        }
        None => 0,
    }
}

/// A test's way to a device register: `MemoryBus.Write32` at a physical address, as the WiseMan tests program the VI.
///
/// # Safety
/// `machine` a live v1 handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_bus_write32(machine: *mut v1::sys::Machine, physical: u32, value: u32) -> i32 {
    let Some(c) = (unsafe { of_mut(machine) }) else { return STATUS_NULL };
    c.machine.bus.write32(physical, value);
    0
}

/// A test's way to read a device register: `MemoryBus.Read32` at a physical address, side effects and all.
///
/// # Safety
/// `machine` a live v1 handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_bus_read32(machine: *mut v1::sys::Machine, physical: u32) -> u32 {
    unsafe { of_mut(machine) }.map_or(0, |c| c.machine.bus.read32(physical))
}

/// A test's way to a COP0 register, with everything C# derives after a write outside an instruction.
///
/// # Safety
/// `machine` a live v1 handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_set_cop0(machine: *mut v1::sys::Machine, register: u32, value: u64) -> i32 {
    let Some(c) = (unsafe { of_mut(machine) }) else { return STATUS_NULL };
    c.machine.cpu.cop0[(register & 31) as usize] = value;
    c.machine.cpu.cop0_written(&c.machine.bus);
    0
}

/// A test's way to an interrupt: the sources unmasked and raised at the MI, as `Mi.Mask` and `Mi.Raise` are set by hand.
///
/// # Safety
/// `machine` a live v1 handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_mi_raise(machine: *mut v1::sys::Machine, sources: u32) -> i32 {
    let Some(c) = (unsafe { of_mut(machine) }) else { return STATUS_NULL };
    c.machine.bus.mi.mask |= sources as i32;
    c.machine.bus.mi.raise(sources as i32);
    0
}

/// A test's way to run the signal processor by itself: `Rsp.Start` at `pc` when `start` is set, then `Rsp.Step` `steps` times, its
/// coverage recorded while the hooks arm it.
///
/// # Safety
/// `machine` a live v1 handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_rsp_step(machine: *mut v1::sys::Machine, start: u32, pc: u32, steps: u32) -> i32 {
    let Some(c) = (unsafe { of_mut(machine) }) else { return STATUS_NULL };
    c.sync_rsp_coverage();
    if start != 0 {
        c.machine.bus.sp.processor.start(pc);
    }
    for _ in 0..steps {
        if !c.machine.bus.sp.processor.halted {
            c.machine.bus.rsp_step_one();
        }
    }
    0
}

/// The call stack's depth, its unmatched returns, and the exceptions entered, up to `len`; returns 3.
///
/// # Safety
/// `machine` a live v1 handle or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_debug_counters(machine: *const v1::sys::Machine, out: *mut i64, len: usize) -> i64 {
    let Some(c) = (unsafe { of(machine) }) else { return STATUS_NULL as i64 };
    let hooks = &c.machine.cpu.hooks;
    unsafe { copy(&[hooks.depth() as i64, hooks.unmatched_returns, c.machine.cpu.run.exceptions], out, len) }
}

/// The version of the state last loaded: 1 a state, 2 a snapshot, 0 none.
///
/// # Safety
/// `machine` a live v1 handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_state_kind(machine: *const v1::sys::Machine) -> i32 {
    unsafe { of(machine) }.map_or(STATUS_NULL, |c| c.machine.loaded_version)
}

/// A state's fields alone, nothing derived, as the state tests compare them byte for byte; zero, or a negative status, and a
/// failed load changes nothing.
///
/// # Safety
/// `machine` a live v1 handle or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_load_fields(machine: *mut v1::sys::Machine, data: *const u8, len: usize) -> i32 {
    let Some(c) = (unsafe { of_mut(machine) }) else { return STATUS_NULL };
    match c.machine.load_state(unsafe { emusen_native::ffi::input(data, len) }) {
        Ok(()) => 0,
        Err(e) => e.status(),
    }
}

/// A controller plugged into the port, or pulled out.
///
/// # Safety
/// `machine` a live v1 handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marsrt_core_set_present(machine: *mut v1::sys::Machine, port: u32, present: u32) -> i32 {
    let Some(c) = (unsafe { of_mut(machine) }) else { return STATUS_NULL };
    if port >= 4 {
        return status::NO_SUCH_PORT;
    }
    c.machine.set_present(port as usize, present != 0);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_setting_passes_its_own_checks_and_none_is_dropped() {
        assert_eq!(schema().len(), 15);
        let defaults = defaults();
        let from_schema = Values { workers: 0, ..defaults }.with(&Settings::parse(schema().iter().map(|s| format!("{}={}\n", s.key, s.default)).collect::<String>().as_bytes()).unwrap());
        assert_eq!(from_schema, defaults);
    }

    use emusen_native::core::{BatteryKind, File};

    /// A machine of an image no state names, with these files, the paks kept apart or the game's.
    fn machine(files: &[(u32, Vec<u8>)], apart: bool) -> Core {
        let mut image = vec![0u8; 0x2000];
        image[..4].copy_from_slice(&[0x80, 0x37, 0x12, 0x40]);
        let settings = v1::exports::resolve(&schema(), b"ThreadedRdp=false\nDeferredPresentation=false\nRecompiler=false", true, true).unwrap();
        let files = files.iter().map(|(which, data)| File { which: *which, data }).collect();
        let request = Create { image: &image, settings, files, pixel_formats: 1, host_abi_version: 0x1_0009, shared_removable: apart, shared_console: false, device: None };
        <Core as v1::Core>::create(&request).unwrap_or_else(|e| panic!("refused {e}"))
    }

    fn pak(byte: u8) -> Vec<u8> {
        let mut data = crate::memory::controller::ControllerPak::default();
        crate::memory::controller::ControllerPak::format(&mut data.data[..]);
        data.data[0x500] = byte;
        data.data.to_vec()
    }

    #[test]
    fn the_paks_are_four_removable_slots_of_one_medium_labelled_by_port() {
        let files = battery();
        assert_eq!((files[0].which, files[0].suffix.as_str(), files[0].kind), (0, ".srm", BatteryKind::Game));
        for n in 0..4u32 {
            let b = &files[1 + n as usize];
            assert_eq!((b.which, b.kind, b.medium.as_deref(), b.port), (1 + n, BatteryKind::Removable, Some("n64.pak"), Some(n)));
            assert_eq!(b.label.as_deref(), Some(PAK_LABELS[n as usize]));
            assert!(b.suffix.ends_with(".mpk"));
        }
    }

    /// Per game, as C# Mars: the first port's pak always, formatted with no file, and another port's only from its file.
    #[test]
    fn per_game_the_first_port_has_a_pak_and_another_port_only_with_its_file() {
        let c = machine(&[(3, pak(7))], false);
        let ports = &c.machine.bus.si.controllers;
        assert!(ports[0].pak.is_some() && ports[1].pak.is_none() && ports[3].pak.is_none());
        assert_eq!(ports[2].pak.as_ref().unwrap().data[0x500], 7);
        assert_eq!(<Core as v1::Core>::battery(&c, 3).unwrap().0[0x500], 7);
        assert_eq!(<Core as v1::Core>::battery(&c, 2).unwrap().0.len(), 0);
    }

    /// Kept apart (EmuSen_CoreAPI.md §6.27): every port has a pak, a load leaves each as it stands and unchanged, and a port whose
    /// pak differs from the state's answers the next status as a pak newly inserted.
    #[test]
    fn kept_apart_a_load_leaves_each_pak_as_it_stands_and_tells_the_game_of_a_new_one() {
        let saver = machine(&[(2, pak(1))], true);
        assert!(saver.machine.bus.si.controllers.iter().all(|p| p.pak.is_some()));
        let mut state = vec![0u8; saver.state_size()];
        saver.save_state(&mut state).unwrap();
        let mut loader = machine(&[(1, pak(5)), (2, pak(9))], true);
        let before: Vec<(Vec<u8>, u32)> = (1..=4).map(|w| <Core as v1::Core>::battery(&loader, w).map(|(d, f)| (d.to_vec(), f)).unwrap()).collect();
        loader.load_state(&state).unwrap();
        for w in 1..=4u32 {
            assert_eq!(<Core as v1::Core>::battery(&loader, w).map(|(d, f)| (d.to_vec(), f)).unwrap(), before[w as usize - 1], "file {w}");
        }
        let ports = &loader.machine.bus.si.controllers;
        assert_eq!(ports.iter().map(|p| *p.inserted).collect::<Vec<_>>(), vec![true, true, false, false]);
    }

    #[test]
    fn the_pad_s_bits_are_the_joybus_s_and_each_is_named_once() {
        let mut bits: Vec<u32> = PAD.iter().map(|p| p.0).collect();
        bits.sort();
        assert_eq!(bits, (0..16).filter(|b| *b != 6 && *b != 7).collect::<Vec<_>>());
    }
}
