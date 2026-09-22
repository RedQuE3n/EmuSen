//! Chroma key: how near the last combiner cycle's colour is to the key: C#'s `Rdp.ChromaKey.cs`.

use super::Rdp;

/// A channel past the key's centre counts its distance back; a low nibble of eight rounds the other way.
#[inline(always)]
fn distance(channel: i32, width: i32) -> i32 {
    let mut value = (channel << 15) >> 15;
    if value > 0 {
        value = if (value & 0xF) == 8 { 0x10 - value } else { -value };
    }
    (width << 4) + value
}

impl Rdp {
    pub(super) fn chroma_key(&self, red: i32, green: i32, blue: i32) -> i32 {
        let w = self.key_width;
        distance(red, w.r).min(distance(green, w.g).min(distance(blue, w.b))).clamp(0, 0xFF)
    }
}
