//! Replays GS register writes kept by `examples/s28.rs`'s `gstrace` (a game's own command stream, Sapphire_Native.md
//! §39) into a GS made fresh, and writes a buffer of its local memory as a picture after each file, so that a drawing
//! defect is narrowed to the writes that make it before a program of the project's own is written to prove it.
//!
//! `cargo run --release --example gsreplay -- <out-dir> <bp> <bw> <psm> <width> <height> <trace>...`
//!
//! `bp` is the buffer's base in blocks of 256 bytes (FRAME's FBP × 32), `bw` its width in units of 64 pixels, `psm` its
//! format's code. A trace line `-- <name>` in a file writes the picture there too, named after it.

use sapphire::gs::local::Local;
use sapphire::gs::psm::Psm;
use sapphire::gs::{Gs, HostInterface};

fn picture(local: &Local, psm: Psm, bp: u32, bw: u32, w: u32, h: u32, path: &std::path::Path) {
    let mut v = format!("P6\n{w} {h}\n255\n").into_bytes();
    for y in 0..h {
        for x in 0..w {
            let p = local.read(psm, bp, bw, x, y);
            let rgb = match psm {
                Psm::Ct16 | Psm::Ct16s => [((p & 31) << 3) as u8, ((p >> 5 & 31) << 3) as u8, ((p >> 10 & 31) << 3) as u8],
                _ => [p as u8, (p >> 8) as u8, (p >> 16) as u8],
            };
            v.extend_from_slice(&rgb);
        }
    }
    std::fs::write(path, v).unwrap();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = std::path::PathBuf::from(&a[1]);
    std::fs::create_dir_all(&out).unwrap();
    let n = |i: usize| u32::from_str_radix(a[i].trim_start_matches("0x"), 16).unwrap();
    let (bp, bw, psm, w, h) = (n(2), n(3), Psm::from_code(n(4)).expect("a format"), n(5), n(6));
    let mut gs = Gs::new();
    for file in &a[7..] {
        let text = std::fs::read_to_string(file).unwrap();
        for line in text.lines() {
            let mut f = line.split_whitespace();
            match (f.next(), f.next()) {
                (Some("--"), Some(name)) => picture(gs.local(), psm, bp, bw, w, h, &out.join(format!("{name}.ppm"))),
                (Some(r), Some(d)) if r.len() == 2 => gs.write(u8::from_str_radix(r, 16).unwrap(), u64::from_str_radix(d, 16).unwrap()),
                _ => {}
            }
        }
        let name = std::path::Path::new(file).file_stem().unwrap().to_string_lossy().into_owned();
        picture(gs.local(), psm, bp, bw, w, h, &out.join(format!("{name}.ppm")));
    }
}
