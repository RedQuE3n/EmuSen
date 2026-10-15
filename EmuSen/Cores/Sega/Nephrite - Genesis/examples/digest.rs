//! What a run gives, as digests to compare across builds: `digest <image>...`, or `digest --list <names> --from <dir>`
//! for a list as the kit reads it. Each image runs 1,200 frames with pad 1's buttons pressed in turn as the kit presses
//! them, and at frames 300, 600 and 1,200 prints FNV-1a over the state, the picture and the sound so far. A disc runs
//! with `NEPHRITE_BIOS` as its boot ROM, a cue sheet with its tracks (Nephrite_Attachments.md §7.1).

use emusen_native::core::Kept;
use emusen_native::ffi::StateMachine;
use nephrite::machine::{Machine, default_model};
use nephrite::media::Media;

const AT: [i64; 3] = [300, 600, 1200];

fn fnv(h: u64, bytes: &[u8]) -> u64 {
    bytes.iter().fold(h, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01B3))
}

const FNV: u64 = 0xCBF2_9CE4_8422_2325;

fn machine(path: &std::path::Path, raw: &[u8], bios: &Option<Vec<u8>>) -> Machine {
    if emusen_disc::is_sheet(raw) || nephrite::media::disc_offset(raw).is_some() {
        let tracks = if emusen_disc::is_sheet(raw) {
            let sheet = emusen_disc::cue::Sheet::parse(&String::from_utf8_lossy(raw)).expect("a sheet");
            sheet.files.iter().map(|f| Kept::from(std::fs::read(path.parent().unwrap().join(f)).expect("a track"))).collect()
        } else {
            Vec::new()
        };
        let mut disc = nephrite::mcd::open_disc(Kept::from(raw.to_vec()), tracks).expect("a disc");
        let media = Media::read(&nephrite::mcd::system_area(&mut disc));
        return Machine::with_disc(&[], media.clone(), default_model(&media), bios.clone(), Some(disc));
    }
    let image = nephrite::media::cartridge_bytes(raw).into_owned();
    Machine::new(&image, Media::read(&image))
}

/// The kit's presses: from frame 8, each of pad 1's twelve buttons held 8 frames in 16, in turn.
fn pads(f: i64) -> u32 {
    if f < 8 {
        return 0;
    }
    let i = (f - 8) / 16;
    if (f - 8) % 16 < 8 { 1 << (i % 12) } else { 0 }
}

fn run(path: &std::path::Path, bios: &Option<Vec<u8>>) -> String {
    let raw = std::fs::read(path).expect("the image");
    let mut m = machine(path, &raw, bios);
    let (mut audio, mut out, mut state) = (FNV, vec![0i16; 1 << 16], Vec::new());
    let mut line = path.file_name().unwrap().to_string_lossy().into_owned();
    while m.frames < AT[2] {
        m.pads[0] = pads(m.frames);
        m.advance();
        loop {
            let n = m.audio.drain(&mut out, 1 << 15);
            if n == 0 {
                break;
            }
            audio = out[..n].iter().fold(audio, |h, s| fnv(h, &s.to_le_bytes()));
        }
        if AT.contains(&m.frames) {
            state.resize(m.state_size(), 0);
            let n = m.save_state(&mut state).expect("a state");
            line += &format!("\t{}:{:016x}:{:016x}:{:016x}:{}", m.frames, fnv(FNV, &state[..n]), fnv(FNV, &m.picture), audio, n);
        }
    }
    line
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let bios = std::env::var_os("NEPHRITE_BIOS").map(|p| std::fs::read(p).expect("the boot ROM"));
    let images: Vec<std::path::PathBuf> = match args.first().map(String::as_str) {
        Some("--list") => {
            let from = std::path::Path::new(&args[3]);
            std::fs::read_to_string(&args[1])
                .expect("the list")
                .lines()
                .filter(|l| !l.starts_with('#'))
                .map(|l| l.split("  ").next().unwrap().trim())
                .filter(|l| !l.is_empty())
                .map(|n| from.join(n))
                .collect()
        }
        _ => args.iter().map(std::path::PathBuf::from).collect(),
    };
    for path in images {
        println!("{}", run(&path, &bios));
    }
}
