//! Times Diamond's frames on the emulation thread and digests what they made (Diamond_Speed.md §2): `bench <image>
//! [frames] [--from N] [--armed] [--plain] [--stepped] [--verify N] [--verify-runs N] [--times FILE] [--every N]
//! [--observed] [--renderer] [--precise] [--precision] [--waited] [--save FILE]`; `--observed` attaches the enhancement layer's test observer
//! (Diamond_Enhancements.md §7), `--renderer` stage E3's Vulkan renderer, whose frame end is timed with the frame as a
//! host pays it, the picture of the frame before handed (§38.1), its share printed apart; with `--waited` each frame's
//! picture is waited for instead and compared with the machine's, the frames that differ printed (§18); `--precise` stage
//! E4's precise geometry, the precision layer paired with the renderer, timed as `--renderer` is, and `--precision` the
//! precision layer alone with a drawer that draws nothing, its own share (§25); `--save FILE` writes
//! the state the run ends in, a layer's block with it,
//! `--plain` turns the exact levers off and `--stepped` the runs alone, `--verify N` checks every Nth skip against the
//! interpreter and `--verify-runs N` every Nth run against its steps. The image is a disc (a cue sheet, a CHD, an `.iso` or a
//! lone `.bin`) or a PS-EXE, run on the open kernel with no firmware; a `.s` source is assembled and run with no kernel.
//! Prints the mean, median, 95th percentile and worst milliseconds a frame from frame N (default 0), and the digests
//! of every frame's picture, of the sound and of the final state, each FNV-1a; `--every N` adds the state's digest every
//! N frames. Only `advance` is timed, and with `--renderer` the wait for its picture, never the digests.

use diamond::kernel::{self, Kernel};
use diamond::machine::{Config, Gpu, Machine};
use diamond::media::{Market, Media};
use emusen_native::ffi::StateMachine;

const BASIS: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv(h: u64, b: &[u8]) -> u64 {
    b.iter().fold(h, |h, &x| (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01B3))
}

/// The state's accurate blocks, those before an attached layer's own (Diamond_Enhancements.md §4), so that every form of
/// a run digests alike.
fn state_digest(m: &Machine) -> u64 {
    let mut s = vec![0; m.state_size()];
    let n = m.save_state(&mut s).expect("saved");
    let end = if m.layer_ref().is_some() { m.layout().lines().find(|l| l.ends_with(".Tag")).and_then(|l| l.split_whitespace().next()?.parse().ok()).unwrap_or(n) } else { n };
    fnv(BASIS, &s[..end])
}

struct Unseen;

impl blackmoon_gte::Observer for Unseen {}

impl diamond::layer::Layer for Unseen {}

impl precision::Draw for Unseen {
    fn executed_with(&mut self, _c: &diamond::layer::Command, _precise: Option<&[precision::Fraction; 4]>) {}
}

/// The renderer attached, alone or paired with the precision layer.
fn renderer(m: &mut Machine) -> &mut diamond_renderer::Renderer {
    let layer: &mut dyn std::any::Any = m.layer().expect("the renderer");
    if layer.is::<diamond_renderer::Renderer>() {
        return layer.downcast_mut::<diamond_renderer::Renderer>().unwrap();
    }
    &mut layer.downcast_mut::<precision::Precise<diamond_renderer::Renderer>>().expect("the renderer").drawer
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
    if args.iter().any(|a| a == "--observed") {
        m.attach(Box::new(diamond::layer::Tally::new()));
    }
    let precise = args.iter().any(|a| a == "--precise");
    let rendering = precise || args.iter().any(|a| a == "--renderer");
    let waited = args.iter().any(|a| a == "--waited");
    if rendering {
        let r = diamond_renderer::Renderer::with(None, drawing(&args)).unwrap_or_else(|e| panic!("no renderer: {e}"));
        if precise {
            m.attach(Box::new(precision::Precise::new(precision::Precision::default(), r.keeping(true))));
        } else {
            m.attach(Box::new(r));
        }
    }
    if args.iter().any(|a| a == "--precision") {
        m.attach(Box::new(precision::Precise::new(precision::Precision::default(), Unseen)));
    }
    let (mut waits, mut unequal, mut made) = (Vec::with_capacity(frames), 0, Vec::with_capacity(640 * 576 * 4));
    m.idle.on = !args.iter().any(|a| a == "--plain");
    m.idle.verify = flag("--verify").map_or(0, |v| v.parse().expect("a count"));
    m.runs.on = !args.iter().any(|a| a == "--plain" || a == "--stepped");
    m.runs.verify = flag("--verify-runs").map_or(0, |v| v.parse().expect("a count"));
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
        let advanced = start.elapsed().as_secs_f64() * 1e3;
        if rendering && waited {
            made.clear();
            made.extend_from_slice(m.board.gpu.display.frame());
            let wait = std::time::Instant::now();
            let drawn = renderer(&mut m).picture().0;
            waits.push(wait.elapsed().as_secs_f64() * 1e3);
            unequal += (drawn != &made[..]) as usize;
        } else if rendering {
            let wait = std::time::Instant::now();
            renderer(&mut m).frame_end();
            waits.push(wait.elapsed().as_secs_f64() * 1e3);
        }
        times.push(advanced + waits.last().copied().unwrap_or(0.0));
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
    if let Some(file) = flag("--save") {
        m.settle();
        let mut s = vec![0; m.state_size()];
        let n = m.save_state(&mut s).expect("saved");
        std::fs::write(file, &s[..n]).expect("written");
    }
    let digest = state_digest(&m);
    if rendering {
        let w = &waits[from.min(waits.len())..];
        let wait = w.iter().sum::<f64>() / w.len().max(1) as f64;
        let r = renderer(&mut m);
        eprintln!("renderer: {}; the picture waited for {wait:.3} ms a frame of the mean; {unequal} frames' pictures differ; {:?}", r.report(), r.counts());
        if precise {
            let layer: Box<dyn std::any::Any> = m.detach().expect("the pair");
            eprintln!("precision: {:?}", layer.downcast::<precision::Precise<diamond_renderer::Renderer>>().expect("the pair").precision.counts());
        } else {
            m.detach();
        }
    }
    if let Some(layer) = m.detach() {
        let layer = layer as Box<dyn std::any::Any>;
        if let Some(p) = layer.downcast_ref::<precision::Precise<Unseen>>() {
            eprintln!("precision: {:?}", p.precision.counts());
            return finish(&times, from, path, picture, sound, digest, &m);
        }
        let t = *layer.downcast::<diamond::layer::Tally>().expect("the test observer");
        eprintln!("observed: {} loads, {} stores, {} coprocessor 2 moves, {} copies, {} projections, {} words to GP0 from the processor, {} by DMA", t.loads, t.stores, t.cop2, t.copies, t.projections, t.from_cpu, t.from_dma);
    }
    finish(&times, from, path, picture, sound, digest, &m);
}

/// The frame times' line and the levers' counts.
fn finish(times: &[f64], from: usize, path: &str, picture: u64, sound: u64, digest: u64, m: &Machine) {
    let frames = times.len();
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
        digest
    );
    eprintln!("runs: {} of {} steps, {:.1} a run", m.runs.made, m.runs.steps, m.runs.steps as f64 / m.runs.made.max(1) as f64);
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
    let kept = files.into_iter().map(emusen_native::core::Kept::from).collect();
    let Ok(diamond::medium::Opened::Disc(disc, media)) = diamond::medium::open(emusen_native::core::Kept::from(image.clone()), kept) else { panic!("a disc") };
    let config = Config { market: media.market.unwrap_or(Market::Americas), ..config };
    let mut m = Machine::new(media, config, ports, &image, open);
    m.insert_disc(*disc);
    m
}

/// The renderer's options from `--scale N`, `--colour` and `--filter 3d|all` (Diamond_Enhancements.md §39 to §41).
fn drawing(args: &[String]) -> diamond_renderer::Options {
    use diamond_renderer::Filter;
    let flag = |name: &str| args.iter().position(|a| a == name).map(|i| args[i + 1].clone());
    let filter = match flag("--filter").as_deref() {
        None | Some("off") => Filter::Off,
        Some("3d") => Filter::Polygons,
        Some("all") => Filter::All,
        Some(other) => panic!("--filter {other}: off, 3d or all"),
    };
    diamond_renderer::Options { scale: flag("--scale").map_or(1, |s| s.parse().expect("a scale")), colour: args.iter().any(|a| a == "--colour"), filter }
}
