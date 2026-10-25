//! The project's S27 material (Sapphire_Native.md §34): MPEG-2 and MPEG-1 streams of the project's own making
//! (`streams/`, encoded by ffmpeg 8.1.3 run as a black box from its test pattern with noise; the commands are in
//! §34.9), with ffmpeg's own decoding of each beside it as a witness; `standard`, the decoding H.262 defines, written
//! apart from `src/ipu/`; `manual`, the EE User's Manual's post-processing; `driver`, the IPU driven through a stream
//! by its commands; `encode`, streams written symbol by symbol for the cases no encoder makes.

#![allow(dead_code)]

pub mod driver;
pub mod encode;
pub mod manual;
pub mod program;
pub mod registers;
pub mod standard;

/// A stream of the corpus: its name, its bytes, ffmpeg's decoding (yuv420p frames in display order).
pub struct Stream {
    pub name: &'static str,
    pub bytes: &'static [u8],
    pub ffmpeg: &'static [u8],
}

macro_rules! stream {
    ($name:literal, $ext:literal) => {
        Stream {
            name: $name,
            bytes: include_bytes!(concat!("streams/", $name, ".", $ext)),
            ffmpeg: include_bytes!(concat!("streams/", $name, ".yuv")),
        }
    };
}

impl Stream {
    /// The stream with a sequence_end_code after it when it has none (ffmpeg's elementary streams end without one), so
    /// that the IPU finds a start code after the last macroblock (EE User's Manual §8.4, IDEC's notes).
    pub fn complete(&self) -> Vec<u8> {
        let mut v = self.bytes.to_vec();
        if !v.ends_with(&[0, 0, 1, 0xB7]) {
            v.extend([0, 0, 1, 0xB7]);
        }
        v
    }
}

pub fn corpus() -> Vec<Stream> {
    vec![
        stream!("intra-q2", "m2v"),
        stream!("intra-tools", "m2v"),
        stream!("intra-matrix", "m2v"),
        stream!("intra-field", "m2v"),
        stream!("intra-alt", "m2v"),
        stream!("intra-nonlinear", "m2v"),
        stream!("intra-ivf", "m2v"),
        stream!("intra-dc11", "m2v"),
        stream!("pb", "m2v"),
        stream!("pb-rate", "m2v"),
        stream!("mpeg1-intra", "m1v"),
        stream!("mpeg1-pb", "m1v"),
    ]
}

/// The ways IDEC's output is asked for on `intra-matrix`: RGB16 truncated and dithered, SGN, the thresholds, and SGN
/// with the dither, which §8.6.2 says does not clip correctly.
pub const IDEC_VARIANTS: [(&str, driver::Options); 5] = [
    ("rgb16", driver::Options { idec: true, rgb16: true, dither: false, sgn: false, th: (0, 0) }),
    ("dither", driver::Options { idec: true, rgb16: true, dither: true, sgn: false, th: (0, 0) }),
    ("sgn", driver::Options { idec: true, rgb16: false, dither: false, sgn: true, th: (0, 0) }),
    ("th", driver::Options { idec: true, rgb16: false, dither: false, sgn: false, th: (60, 140) }),
    ("sgn-dither", driver::Options { idec: true, rgb16: true, dither: true, sgn: true, th: (60, 140) }),
];

/// Every program of the stage: BDEC over each stream, IDEC over each stream with I-pictures alone, the variants, and
/// the post-processing commands over raw data.
pub fn programs() -> Vec<program::Program> {
    let mut v = Vec::new();
    let (image, log) = post().unwrap_or_else(|e| panic!("post: {e}"));
    v.push(program::build("s27-post", &image, &log));
    for s in corpus() {
        let bytes = s.complete();
        let d = standard::decode(&bytes);
        let image = driver::image(&bytes);
        let log = driver::drive(&bytes, &d, driver::Options::BDEC).unwrap_or_else(|e| panic!("{}: {e}", s.name));
        v.push(program::build(&format!("s27-bdec-{}", s.name), &image, &log));
        if s.name.contains("intra") {
            let log = driver::drive(&bytes, &d, driver::Options::IDEC32).unwrap_or_else(|e| panic!("{}: {e}", s.name));
            v.push(program::build(&format!("s27-idec-{}", s.name), &image, &log));
        }
        if s.name == "intra-matrix" {
            for (n, o) in IDEC_VARIANTS {
                let log = driver::drive(&bytes, &d, o).unwrap_or_else(|e| panic!("{}: {e}", s.name));
                v.push(program::build(&format!("s27-idec-{n}"), &image, &log));
            }
        }
    }
    v
}

/// A small generator of test data, the same on every run.
pub struct Lcg(pub u64);

impl Lcg {
    pub fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) as u32
    }
}

/// CSC's RAW8 macroblocks: Y through 0 to 255 with neutral chroma, Cb and then Cr through their range at Y 128, both
/// at once, and three of random samples.
pub fn raw8_inputs() -> Vec<[u8; 384]> {
    let mut r = Lcg(27);
    let mut v = Vec::new();
    for kind in 0..7 {
        let mut m = [128u8; 384];
        for i in 0..384 {
            m[i] = match (kind, i) {
                (0, i) if i < 256 => i as u8,
                (1, i) if (256..320).contains(&i) => (4 * (i - 256)) as u8,
                (2, i) if i >= 320 => (4 * (i - 320)) as u8,
                (3, i) if i >= 256 => (4 * (i % 64) + if i >= 320 { 2 } else { 0 }) as u8,
                (3, i) => (i * 7 % 256) as u8,
                (k, _) if k >= 4 => r.next() as u8,
                _ => 128,
            };
        }
        v.push(m);
    }
    v
}

/// PACK's RGB32 macroblocks: alpha 0, 40h, 80h and others, channels near the dither's and the truncation's edges, and
/// random pixels.
pub fn rgb32_inputs() -> Vec<[u32; 256]> {
    let mut r = Lcg(2027);
    let mut v = Vec::new();
    v.push(core::array::from_fn(|i| {
        let c = i as u32;
        c | (c ^ 0x55) << 8 | (255 - c) << 16 | [0, 0x40, 0x80, 0x7F][i % 4] << 24
    }));
    for _ in 0..3 {
        v.push(core::array::from_fn(|_| r.next()));
    }
    v
}

/// SETVQ's CLUT: 16 random RGB16 entries, the first two equal so that a tie shows which is taken.
pub fn clut() -> [u16; 16] {
    let mut r = Lcg(16);
    let mut c: [u16; 16] = core::array::from_fn(|_| r.next() as u16);
    c[1] = c[0];
    c
}

/// The post-processing commands over raw data (`Driver::over`): SETTH, CSC to RGB32 and to RGB16 truncated and
/// dithered, CSC under thresholds, SETVQ, then PACK to RGB16 truncated and dithered and to INDX4 truncated and dithered;
/// each output checked against `manual`. The log and the image, for the referees' program.
pub fn post() -> Result<(Vec<u128>, Vec<driver::Issued>), String> {
    let raw = raw8_inputs();
    let rgb = rgb32_inputs();
    let clut = clut();
    let mut bytes: Vec<u8> = Vec::new();
    let csc_runs: [(bool, bool, (u32, u32)); 4] = [(false, false, (0, 0)), (true, false, (0, 0)), (true, true, (0, 0)), (false, false, (40, 160))];
    for _ in csc_runs {
        bytes.extend(raw.iter().flatten());
    }
    bytes.extend(clut.iter().flat_map(|c| c.to_le_bytes()));
    let pack_runs: [(bool, bool); 4] = [(true, false), (true, true), (false, false), (false, true)];
    for _ in pack_runs {
        bytes.extend(rgb.iter().flatten().flat_map(|p| p.to_le_bytes()));
    }
    bytes.extend([0u8; 32]);
    let data: Vec<u128> = bytes.chunks(16).map(|c| u128::from_le_bytes(c.try_into().unwrap())).collect();
    let mut d = driver::Driver::over(data.clone());
    use sapphire::ipu::code;
    d.run(code::BCLR << 28)?;
    for (rgb16, dither, th) in csc_runs {
        d.run(code::SETTH << 28 | th.1 << 16 | th.0)?;
        let i = d.run(code::CSC << 28 | u32::from(rgb16) << 27 | u32::from(dither) << 26 | raw.len() as u32)?;
        let want: Vec<u128> = raw
            .iter()
            .flat_map(|m| {
                let px = manual::from_raw8(m, th.0, th.1);
                if rgb16 { manual::rgb16_qwords(&px, dither) } else { manual::rgb32_qwords(&px) }
            })
            .collect();
        if i.out != want {
            return Err(format!("CSC rgb16 {rgb16} dither {dither} th {th:?}: output differs at qword {:?}", i.out.iter().zip(&want).position(|(a, b)| a != b)));
        }
    }
    d.run(code::SETVQ << 28)?;
    for (rgb16, dither) in pack_runs {
        let i = d.run(code::PACK << 28 | u32::from(rgb16) << 27 | u32::from(dither) << 26 | rgb.len() as u32)?;
        let want: Vec<u128> = rgb
            .iter()
            .flat_map(|m| {
                let px: Vec<[u8; 4]> = m.iter().map(|p| p.to_le_bytes()).collect();
                if rgb16 { manual::rgb16_qwords(&px, dither) } else { manual::indx4_qwords(&px, dither, &clut) }
            })
            .collect();
        if i.out != want {
            return Err(format!("PACK rgb16 {rgb16} dither {dither}: output differs at qword {:?}", i.out.iter().zip(&want).position(|(a, b)| a != b)));
        }
    }
    Ok((data, d.log))
}
