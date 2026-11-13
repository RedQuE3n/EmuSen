//! A clip a test writes itself, shared by the program's tests and the ES-DE painter's: one colour, magenta, 64 × 48 at 30
//! frames a second, as a YUV4MPEG2 stream, the simplest a test can write by hand; FFmpeg finds the format by its content.

use std::path::Path;

/// Frames a second of every clip written here.
pub const RATE: u32 = 30;

/// A magenta clip `seconds` long at `path`, its folders made.
pub fn write(path: &Path, seconds: u32) {
    write_frames(path, 64, 48, seconds * RATE, |_, _, _| MAGENTA);
}

/// Colours as YUV4MPEG2's full-range BT.601 Y, Cb and Cr.
pub const MAGENTA: [u8; 3] = [107, 202, 222];
pub const RED: [u8; 3] = [76, 85, 255];
pub const GREEN: [u8; 3] = [150, 44, 21];
pub const BLUE: [u8; 3] = [29, 255, 107];

/// A `w` × `h` clip of `frames` frames at `path`, each pixel `colour(frame, x, y)`, its folders made; a 2 × 2 block's
/// chroma is its top left pixel's, as 4:2:0 keeps one per block.
pub fn write_frames(path: &Path, w: usize, h: usize, frames: u32, colour: impl Fn(u32, usize, usize) -> [u8; 3]) {
    let mut bytes = format!("YUV4MPEG2 W{w} H{h} F{RATE}:1 Ip A1:1 C420jpeg\n").into_bytes();
    for f in 0..frames {
        bytes.extend_from_slice(b"FRAME\n");
        bytes.extend((0..w * h).map(|i| colour(f, i % w, i / w)[0]));
        for plane in [1, 2] {
            bytes.extend((0..(w / 2) * (h / 2)).map(|i| colour(f, 2 * (i % (w / 2)), 2 * (i / (w / 2)))[plane]));
        }
    }
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}
