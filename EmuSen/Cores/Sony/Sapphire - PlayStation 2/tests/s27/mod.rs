//! The project's S27 material (Sapphire_Native.md §34): MPEG-2 and MPEG-1 streams of the project's own making
//! (`streams/`, encoded by ffmpeg 8.1.3 run as a black box from its test pattern with noise; the commands are in
//! §34.9), with ffmpeg's own decoding of each beside it as a witness; `standard`, the decoding H.262 defines, written
//! apart from `src/ipu/`; `manual`, the EE User's Manual's post-processing; `driver`, the IPU driven through a stream
//! by its commands; `encode`, streams written symbol by symbol for the cases no encoder makes.

#![allow(dead_code)]

pub mod driver;
pub mod encode;
pub mod manual;
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
        stream!("pb", "m2v"),
        stream!("pb-rate", "m2v"),
        stream!("mpeg1-intra", "m1v"),
        stream!("mpeg1-pb", "m1v"),
    ]
}
