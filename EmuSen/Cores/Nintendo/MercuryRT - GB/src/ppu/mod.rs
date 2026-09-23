//! C#'s `Ppu/`: the LCD controller's registers, mode machine and colour palettes. See Mercury_Ppu.md and Mercury_Cgb.md §2.

pub mod render;
pub mod timing;

use crate::Skip;
use crate::state::{State, StateReader, StateResult, StateWriter};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const OAM_SCAN_CYCLES: i32 = 80;
pub const BASE_DRAWING_CYCLES: i32 = 172;
pub const PALETTE_RAM_SIZE: usize = 64;

/// What STAT bits 1-0 report; C# serializes the enum as an i32.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum PpuMode {
    #[default]
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

impl PpuMode {
    /// C#'s `Enum.ToObject`: any i32 is kept, so a value outside the four is stored as read.
    fn from_i32(v: i32) -> Result<PpuMode, i32> {
        match v {
            0 => Ok(PpuMode::HBlank),
            1 => Ok(PpuMode::VBlank),
            2 => Ok(PpuMode::OamScan),
            3 => Ok(PpuMode::Drawing),
            other => Err(other),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ppu {
    pub bg_palette_index: u8,
    pub bg_palette_ram: [u8; PALETTE_RAM_SIZE],
    pub bgp: u8,
    pub dot: i32,
    pub drawing_end: i32,
    pub frame_complete: bool,
    pub frame_count: i64,
    pub lcdc: u8,
    pub ly: u8,
    pub lyc: u8,
    pub mode: PpuMode,
    /// A mode read from a state that is none of the four, kept so it is written back as read.
    pub mode_raw: Option<i32>,
    pub obj_palette_index: u8,
    pub obj_palette_ram: [u8; PALETTE_RAM_SIZE],
    pub obp0: u8,
    pub obp1: u8,
    pub scx: u8,
    pub scy: u8,
    pub stat_enables: u8,
    pub stat_line: bool,
    pub window_line: i32,
    pub window_triggered: bool,
    pub wx: u8,
    pub wy: u8,
    pub frame_rgba: Skip<Vec<u8>>,
    pub skip_rendering: Skip<bool>,
}

impl Default for Ppu {
    fn default() -> Self {
        Ppu {
            bg_palette_index: 0,
            bg_palette_ram: [0; PALETTE_RAM_SIZE],
            bgp: 0,
            dot: 0,
            drawing_end: 0,
            frame_complete: false,
            frame_count: 0,
            lcdc: 0,
            ly: 0,
            lyc: 0,
            mode: PpuMode::HBlank,
            mode_raw: None,
            obj_palette_index: 0,
            obj_palette_ram: [0; PALETTE_RAM_SIZE],
            obp0: 0,
            obp1: 0,
            scx: 0,
            scy: 0,
            stat_enables: 0,
            stat_line: false,
            window_line: 0,
            window_triggered: false,
            wx: 0,
            wy: 0,
            frame_rgba: Skip(vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4]),
            skip_rendering: Skip(false),
        }
    }
}

impl Ppu {
    /// What the DMG boot ROM leaves behind, since Mercury starts past it - see Mercury_Cpu.md §5.
    pub fn reset(&mut self) {
        self.lcdc = 0x91;
        self.stat_enables = 0;
        self.scy = 0;
        self.scx = 0;
        self.ly = 0;
        self.lyc = 0;
        self.bgp = 0xFC;
        self.obp0 = 0xFF;
        self.obp1 = 0xFF;
        self.wy = 0;
        self.wx = 0;
        self.mode = PpuMode::OamScan;
        self.mode_raw = None;
        self.dot = 0;
        self.drawing_end = OAM_SCAN_CYCLES + BASE_DRAWING_CYCLES;
        self.window_line = 0;
        self.window_triggered = false;
        self.stat_line = false;
        self.frame_count = 0;
        self.frame_complete = false;
        self.reset_cgb_palettes();
        self.clear_screen();
    }

    /// White, not black, so a colour game that draws before uploading a palette does not flash dark.
    fn reset_cgb_palettes(&mut self) {
        self.bg_palette_index = 0;
        self.obj_palette_index = 0;
        self.bg_palette_ram = [0xFF; PALETTE_RAM_SIZE];
        self.obj_palette_ram = [0xFF; PALETTE_RAM_SIZE];
    }

    pub fn clear_screen(&mut self) {
        self.frame_rgba.fill(0xFF);
    }

    pub(crate) fn mode_i32(&self) -> i32 {
        self.mode_raw.unwrap_or(self.mode as i32)
    }
}

impl State for Ppu {
    fn write_state(&self, w: &mut StateWriter) {
        w.u8("BgPaletteIndex", self.bg_palette_index);
        w.bytes("BgPaletteRam", &self.bg_palette_ram);
        w.u8("Bgp", self.bgp);
        w.i32("Dot", self.dot);
        w.i32("DrawingEnd", self.drawing_end);
        w.bool("FrameComplete", self.frame_complete);
        w.i64("FrameCount", self.frame_count);
        w.u8("Lcdc", self.lcdc);
        w.u8("Ly", self.ly);
        w.u8("Lyc", self.lyc);
        w.i32("Mode", self.mode_i32());
        w.u8("ObjPaletteIndex", self.obj_palette_index);
        w.bytes("ObjPaletteRam", &self.obj_palette_ram);
        w.u8("Obp0", self.obp0);
        w.u8("Obp1", self.obp1);
        w.u8("Scx", self.scx);
        w.u8("Scy", self.scy);
        w.u8("StatEnables", self.stat_enables);
        w.bool("StatLine", self.stat_line);
        w.i32("WindowLine", self.window_line);
        w.bool("WindowTriggered", self.window_triggered);
        w.u8("Wx", self.wx);
        w.u8("Wy", self.wy);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.bg_palette_index = r.u8()?; // BgPaletteIndex
        r.bytes(&mut self.bg_palette_ram)?; // BgPaletteRam
        self.bgp = r.u8()?; // Bgp
        self.dot = r.i32()?; // Dot
        self.drawing_end = r.i32()?; // DrawingEnd
        self.frame_complete = r.bool()?; // FrameComplete
        self.frame_count = r.i64()?; // FrameCount
        self.lcdc = r.u8()?; // Lcdc
        self.ly = r.u8()?; // Ly
        self.lyc = r.u8()?; // Lyc
        let mode = r.i32()?;
        (self.mode, self.mode_raw) = match PpuMode::from_i32(mode) {
            Ok(m) => (m, None),
            Err(raw) => (PpuMode::HBlank, Some(raw)),
        };
        self.obj_palette_index = r.u8()?; // ObjPaletteIndex
        r.bytes(&mut self.obj_palette_ram)?; // ObjPaletteRam
        self.obp0 = r.u8()?; // Obp0
        self.obp1 = r.u8()?; // Obp1
        self.scx = r.u8()?; // Scx
        self.scy = r.u8()?; // Scy
        self.stat_enables = r.u8()?; // StatEnables
        self.stat_line = r.bool()?; // StatLine
        self.window_line = r.i32()?; // WindowLine
        self.window_triggered = r.bool()?; // WindowTriggered
        self.wx = r.u8()?; // Wx
        self.wy = r.u8()?; // Wy
        Ok(())
    }
}
