//! A clip a test writes itself, shared by the program's tests and the ES-DE painter's: one colour, magenta, 64 × 48 at 30
//! frames a second, as a YUV4MPEG2 stream, the simplest a test can write by hand; FFmpeg finds the format by its content.

use std::path::Path;

/// Frames a second of every clip written here.
pub const RATE: u32 = 30;

/// A magenta clip `seconds` long at `path`, its folders made.
pub fn write(path: &Path, seconds: u32) {
    let (w, h) = (64usize, 48usize);
    let mut bytes = format!("YUV4MPEG2 W{w} H{h} F{RATE}:1 Ip A1:1 C420jpeg\n").into_bytes();
    for _ in 0..seconds * RATE {
        bytes.extend_from_slice(b"FRAME\n");
        bytes.extend(std::iter::repeat_n(107u8, w * h));
        bytes.extend(std::iter::repeat_n(202u8, w * h / 4));
        bytes.extend(std::iter::repeat_n(222u8, w * h / 4));
    }
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}
