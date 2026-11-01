//! Where the test core keeps what a test reads back: offsets in its system RAM, and its sizes.

pub const RAM_SIZE: usize = 0x10000;
pub const VRAM_SIZE: usize = 0x1000;
pub const SRAM_SIZE: usize = 0x2000;
pub const RTC_SIZE: usize = 8;

/// The fixed part of a state: magic, frame, samples, then every memory.
pub const STATE_FIXED: usize = 4 + 8 + 8 + RAM_SIZE + VRAM_SIZE + SRAM_SIZE + RTC_SIZE;

pub const BASE_WIDTH: u32 = 320;
pub const BASE_HEIGHT: u32 = 240;
pub const MAX_WIDTH: u32 = 640;
pub const MAX_HEIGHT: u32 = 480;

/// The addresses the memory maps put the two regions at.
pub const MAP_RAM_START: usize = 0x0000_0000;
pub const MAP_VRAM_START: usize = 0x0010_0000;

/// The analog subclass of the pad the core offers beside the RetroPad.
pub const TEST_PAD_DEVICE: u32 = emusen_libretro_sys::DEVICE_ANALOG | (1 << emusen_libretro_sys::DEVICE_TYPE_SHIFT);

pub const LIVES: usize = 0x0010;
pub const LEVEL: usize = 0x0011;
pub const FRAME_COUNTER: usize = 0x0020;
/// The pad's mask as `input_state` gave it, u16.
pub const PAD: usize = 0x0030;
/// Left X, left Y, right X, right Y, i16 each.
pub const AXES: usize = 0x0034;
/// L2 and R2 as analog buttons, i16 each.
pub const TRIGGERS: usize = 0x003C;
/// The disc's index, whether the tray is open, and how many images.
pub const DISC: usize = 0x0040;
/// The device each of ports 0 and 1 was set to, a byte each.
pub const PORT_DEVICE: usize = 0x0044;
/// The sum of the frame times the host reported, i64.
pub const FRAME_TIME: usize = 0x0048;
/// How many times an option update was seen, u32.
pub const OPTION_UPDATES: usize = 0x0050;
/// 1 when the frame was drawn into the host's framebuffer.
pub const DREW_IN_FRAMEBUFFER: usize = 0x0054;
/// How many codes `retro_cheat_set` took, u32, and the last one's text.
pub const CHEAT_COUNT: usize = 0x0058;
pub const CHEAT_TEXT: usize = 0x0060;
pub const CHEAT_TEXT_LEN: usize = 32;
/// One byte per probe entry: 1 answered, 0 refused, `PROBE_NOT_RUN` before the probe.
/// What `tc_fault` found: 1 the fault handled or the write done, 2 refused by the system.
pub const FAULT: usize = 0x005C;
pub const PROBE: usize = 0x0100;
pub const PROBE_LEN: usize = 0x100;
pub const PROBE_NOT_RUN: u8 = 2;
