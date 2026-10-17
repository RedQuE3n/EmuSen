//! The shell's look, EmuSen's own (Diamond_Shell.md §3.2): LunaP's menu palette reduced to 15 bits, the 5x8 font, and
//! the layout of a 320x240 screen. A text colour is a blend factor, 80h a channel's unity over the font's white.

use diamond_guest::gpu::{fill, rgb, text, ADVANCE};

/// Fills, as GP0 takes them.
pub const BACKGROUND: u32 = rgb(0x0E, 0x0E, 0x10);
pub const PANEL: u32 = rgb(0x1B, 0x1B, 0x1E);
pub const RULE: u32 = rgb(0x55, 0x55, 0x5A);
pub const CHOSEN: u32 = rgb(0x24, 0x4A, 0x72);
pub const SHEET: u32 = rgb(0x26, 0x26, 0x2A);
pub const BAR: u32 = rgb(0x4C, 0x9A, 0xE8);

/// Text, as blend factors: LunaP's text, its bright text, muted, accent and warning.
pub const TEXT: u32 = rgb(0x4E, 0x4E, 0x50);
pub const BRIGHT: u32 = rgb(0x77, 0x77, 0x78);
pub const MUTED: u32 = rgb(0x3A, 0x3A, 0x3C);
pub const ACCENT: u32 = rgb(0x26, 0x4D, 0x74);
pub const WARN: u32 = rgb(0x68, 0x43, 0x38);

pub const WIDTH: u32 = 320;
pub const HEADER: u32 = 18;
pub const LIST_Y: u32 = 46;
pub const ROW: u32 = 20;
pub const ROWS_SHOWN: usize = 8;
pub const STATUS_Y: u32 = 212;
pub const HINT_Y: u32 = 226;

/// A string's width in pixels, to its first 0 or `max` characters.
pub fn width(s: &[u8], max: usize) -> u32 {
    s.iter().take(max).take_while(|&&c| c != 0).count() as u32 * ADVANCE
}

/// Text ending at `right`.
pub fn right(right: u32, y: u32, s: &[u8], max: usize, colour: u32) {
    text(right.saturating_sub(width(s, max)), y, s, max, colour);
}

/// A panel with a one-pixel rule round it.
pub fn panel(x: u32, y: u32, w: u32, h: u32, inside: u32) {
    fill(x, y, w, h, RULE);
    fill(x + 1, y + 1, w - 2, h - 2, inside);
}

/// A number's decimal digits.
pub fn number(mut v: u32, out: &mut [u8; 4]) -> &[u8] {
    let mut i = 4;
    loop {
        i -= 1;
        out[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 || i == 0 {
            break;
        }
    }
    &out[i..]
}

/// The screen's frame: the background, the header with EmuSen's name and the drive's state at its right.
pub fn frame(drive: &[u8]) {
    fill(0, 0, WIDTH, 240, BACKGROUND);
    fill(0, 0, WIDTH, HEADER, PANEL);
    fill(0, HEADER, WIDTH, 1, RULE);
    text(8, 5, b"EmuSen", 6, ACCENT);
    right(WIDTH - 8, 5, drive, 24, MUTED);
}

/// The status line and the hints under it.
pub fn footer(status: &[u8], hints: &[u8]) {
    fill(0, STATUS_Y - 4, WIDTH, 1, RULE);
    text(8, STATUS_Y, status, 51, WARN);
    text(8, HINT_Y, hints, 51, MUTED);
}
