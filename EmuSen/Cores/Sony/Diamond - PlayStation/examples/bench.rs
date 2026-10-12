//! Times Diamond's frames on the emulation thread and digests what they made (Diamond_Speed.md §2): `bench <image>
//! [frames] [--from N] [--armed] [--plain] [--verify N] [--times FILE] [--every N]`; `--plain` turns the exact levers off, `--verify N` checks every Nth
//! skip against the interpreter. The image is a disc (a cue sheet, a CHD, an `.iso` or a
//! lone `.bin`) or a PS-EXE, run on the open kernel with no firmware; a `.s` source is assembled and run with no kernel.
//! Prints the mean, median, 95th percentile and worst milliseconds a frame from frame N (default 0), and the digests
//! of every frame's picture, of the sound and of the final state, each FNV-1a; `--every N` adds the state's digest every
//! N frames. Only `advance` is timed, never the digests.

use diamond::kernel::{self, Kernel};
use diamond::machine::{Config, Gpu, Machine};
use diamond::media::{Market, Media};
use emusen_native::ffi::StateMachine;

const BASIS: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv(h: u64, b: &[u8]) -> u64 {
    b.iter().fold(h, |h, &x| (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01B3))
}

fn state_digest(m: &Machine) -> u64 {
    let mut s = vec![0; m.state_size()];
    let n = m.save_state(&mut s).expect("saved");
    fnv(BASIS, &s[..n])
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = &args[0];
    let frames: usize = args.get(1).and_then(|f| f.parse().ok()).unwrap_or(1200);
    let flag = |name: &str| args.iter().position(|a| a == name).map(|i| args[i + 1].clone());
    let from: usize = flag("--from").map_or(0, |f| f.parse().expect("a frame"));
    let every: Option<usize> = flag("--every").map(|f| f.parse().expect("a frame count"));
    let armed = args.iter().any(|a| a == "--armed");
    let mut m = machine(path);
    m.idle.on = !args.iter().any(|a| a == "--plain");
    m.idle.verify = flag("--verify").map_or(0, |v| v.parse().expect("a count"));
    if armed {
        use emusen_native::debug::flag;
        m.debug.hooks.configure(flag::CALLS | flag::WRITES | flag::PROFILING | 1 << flag::COVERAGE, i32::MIN, -1);
        m.debug.hooks.set_breakpoints(&[-2, -2]);
        m.debug.hooks.set_ranges(0, &[u32::MAX - 1, 0, u32::MAX]);
        m.debug.hooks.set_ranges(1, &[u32::MAX - 1, 0, u32::MAX]);
    }
    let (mut picture, mut sound) = (BASIS, BASIS);
    let mut chunk = vec![0i16; 1 << 16];
    let mut times = Vec::with_capacity(frames);
    for f in 0..frames {
        let start = std::time::Instant::now();
        if armed {
            while m.run_frame_debug(emusen_native::debug::run::CONTINUE) != 0 {
                m.debug.hooks.calls_log.clear();
                m.debug.hooks.writes_log.clear();
            }
        } else {
            m.advance();
        }
        times.push(start.elapsed().as_secs_f64() * 1e3);
        picture = fnv(picture, m.board.gpu.display.frame());
        let n = m.board.spu.out.drain(&mut chunk, usize::MAX);
        sound = chunk[..n].iter().fold(sound, |h, &s| fnv(h, &s.to_le_bytes()));
        if every.is_some_and(|e| (f + 1) % e == 0) {
            eprintln!("frame {} state {:016X}", f + 1, state_digest(&m));
        }
    }
    if let Some(file) = flag("--times") {
        let text: String = times.iter().map(|t| format!("{t:.4}\n")).collect();
        std::fs::write(file, text).expect("written");
    }
    let mut window = times[from.min(times.len())..].to_vec();
    let mean = window.iter().sum::<f64>() / window.len().max(1) as f64;
    window.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let at = |q: f64| window.get(((window.len() as f64 * q) as usize).min(window.len().saturating_sub(1))).copied().unwrap_or(0.0);
    let name = std::path::Path::new(path).file_name().unwrap().to_string_lossy();
    println!(
        "{name}\tframes {from}-{frames}\tmean {mean:.3}\tmedian {:.3}\tp95 {:.3}\tmax {:.3}\tpicture {picture:016X}\tsound {sound:016X}\tstate {:016X}",
        at(0.5),
        at(0.95),
        at(1.0),
        state_digest(&m)
    );
    eprintln!("idle: {} skips, {} turns, {} cycles ({:.1}% of the clock); {} proofs, {} refused, {} windows met an event, {} unequal", m.idle.skips, m.idle.turns, m.idle.cycles, 100.0 * m.idle.cycles as f64 / m.board.now as f64, m.idle.proofs, m.idle.refused, m.idle.dropped, m.idle.unequal);
}

/// The machine for `path`: a disc or a PS-EXE on the open kernel, or a `.s` source with no kernel.
fn machine(path: &str) -> Machine {
    let config = Config { market: Market::Americas, gpu: Gpu::Pin208 };
    let ports = [Some("psx.dualshock"), None];
    if path.ends_with(".s") {
        let source = std::fs::read_to_string(path).expect("the source");
        let words = emusen_mips::assemble(&source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
        let image = diamond::exe::build(&words);
        return Machine::new(Media::read(&image).unwrap(), config, ports, &image, None);
    }
    let image = std::fs::read(path).expect("the image");
    let open = kernel::open_kernel().map(|k| (k, Kernel::Open));
    if image.starts_with(b"PS-X EXE") {
        return Machine::new(Media::read(&image).expect("a PS-EXE"), config, ports, &image, open);
    }
    let dir = std::path::Path::new(path).parent().unwrap_or(std::path::Path::new("."));
    let mut files = Vec::new();
    if diamond::medium::is_sheet(&image) {
        for name in emusen_disc::set::companions(&String::from_utf8_lossy(&image)).expect("a cue sheet") {
            files.push(std::fs::read(dir.join(&name)).unwrap_or_else(|_| panic!("the track file {name}")));
        }
    }
    let refs: Vec<&[u8]> = files.iter().map(|f| f.as_slice()).collect();
    let Ok(diamond::medium::Opened::Disc(disc, media)) = diamond::medium::open(&image, &refs) else { panic!("a disc") };
    let config = Config { market: media.market.unwrap_or(Market::Americas), ..config };
    let mut m = Machine::new(media, config, ports, &image, open);
    m.insert_disc(*disc);
    m
}
