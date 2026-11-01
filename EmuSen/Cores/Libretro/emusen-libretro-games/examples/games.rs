// `games <home> install <runner> <stem>...`, `approve <stem>...`, `tool <core request JSON>`, `run <console> <game> <boot>
// <frames> [[!]key=value]...` (! stores a value the schema does not list), EMUSEN_GAMES_LOG=N printing N log events: real libretro cores in a sandbox player home through the Rust runtime (EmuSen_LibretroHost.md §10m).

use std::path::{Path, PathBuf};
use std::time::Instant;

use emusen_core_entry::Entry;
use emusen_crystal::abi::{EVENT_LOG, OPEN_NO_BATTERY, SCOPE_CONSOLE, emusen_crystal_frame_info};
use emusen_crystal::open::Asked;
use emusen_crystal::runtime::{Config, Runtime};
use emusen_crystal::session::{OpenedWith, Session};
use emusen_libretro_games::{Home, info_value, install_libraries, run_job, timing, tool_context};

fn fail(words: String) -> ! {
    println!("error: {words}");
    std::process::exit(1);
}

fn built() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    exe.parent().and_then(Path::parent).unwrap().join("deps")
}

fn runtime(home: &Home) -> Runtime {
    home.point_tree();
    unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", home.installation.join("emusen-core-runner")) };
    let dir = |p: &Path| vec![p.to_string_lossy().into_owned()];
    Runtime::new(&Config { cores: Some(dir(&home.installation)), player_cores: Some(dir(&home.cores)), development: Some(false), sqlite: None, lender: None, fetch: None })
}

fn install(home: &Home, runner: &str, stems: &[String]) {
    let manifest = install_libraries(home, &built(), Path::new(runner)).unwrap_or_else(|e| fail(e));
    print!("installation {}:\n{manifest}", home.installation.display());
    let mut show = |l: &str| {
        if !l.contains("\"event\":\"progress\"") {
            println!("{l}");
        }
    };
    run_job(r#"{"op":"core-fetch"}"#, tool_context(home), &mut show).unwrap_or_else(|e| fail(e));
    let names: Vec<String> = stems.iter().map(|s| format!("\"{s}\"")).collect();
    run_job(&format!(r#"{{"op":"core-get","cores":[{}]}}"#, names.join(",")), tool_context(home), &mut show).unwrap_or_else(|e| fail(e));
}

fn approve(home: &Home, stems: &[String]) {
    let rt = runtime(home);
    for stem in stems {
        let entry = Entry::new(&home.cores, stem).unwrap_or_else(|e| fail(e.0));
        let notice = entry.read_notice().unwrap_or_else(|e| fail(e.0));
        let info = std::fs::read_to_string(entry.info_path()).unwrap_or_default();
        let version = info_value(&info, "display_version").unwrap_or_default();
        match rt.approve(&notice.sha256) {
            Ok(()) => println!("approved {stem} {version} build {} sha256 {}", notice.build, notice.sha256),
            Err(e) => println!("approve {stem}: {}", e.1),
        }
    }
    let d = rt.discovery();
    for c in d.found().iter().filter(|c| c.entry.is_some()) {
        println!("engine {} {:?}", c.engine_name(), c.standing(&rt.trust()));
    }
    for w in d.waiting() {
        println!("waiting {}", w.sha256);
    }
}

fn frame(s: &Session) -> emusen_crystal_frame_info {
    let mut info = emusen_crystal_frame_info { size: size_of::<emusen_crystal_frame_info>() as u32, ..Default::default() };
    s.frame(Some(&mut info)).unwrap_or_else(|e| fail(e.1));
    info
}

fn picture(s: &Session) -> Vec<u8> {
    let mut out = vec![0u8; 64 << 20];
    let n = s.copy_picture(&mut out).unwrap_or(0);
    out.truncate(n);
    out
}

/// A picture's FNV-1a digest and how many of its bytes are not zero, to tell a drawn frame from a blank one.
fn digest(p: &[u8]) -> (u64, usize) {
    (p.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3)), p.iter().filter(|&&b| b != 0).count())
}

fn run(home: &Home, console: &str, game: &str, boot: usize, frames: usize, settings: &[String]) {
    let rt = runtime(home);
    for kv in settings {
        let (k, v) = kv.split_once('=').unwrap_or_else(|| fail(format!("{kv} is not key=value")));
        if let Some(k) = k.strip_prefix('!') {
            let tree = emusen_galaxia::tree::tree();
            let mut stored = emusen_galaxia::consoles::load(&tree);
            stored.set_value(console, k, v);
            stored.save(&tree);
            println!("stored {k}={v} for the {console} as its settings file holds it, past the schema's choices");
            continue;
        }
        rt.settings_set(0, SCOPE_CONSOLE, Some(console), k, Some(v)).unwrap_or_else(|e| fail(format!("{k}: {}", e.1)));
    }
    let log = rt.events.subscribe(0, EVENT_LOG);
    let opened = Instant::now();
    let id = rt.open(&Asked { path: Some(game), console: None, engine: None, opened: OpenedWith { flags: OPEN_NO_BATTERY, ..OpenedWith::default() } }).unwrap_or_else(|e| fail(e.1));
    let s = rt.sessions.get(id).unwrap_or_else(|e| fail(e.1));
    println!("opened in {:.2} s: {}", opened.elapsed().as_secs_f64(), s.info_json());
    let start = Instant::now();
    for _ in 0..boot {
        frame(&s);
    }
    println!("boot: {boot} frames in {:.2} s", start.elapsed().as_secs_f64());
    let mut times = Vec::with_capacity(frames);
    let mut last = frame(&s);
    for _ in 0..frames {
        let t = Instant::now();
        last = frame(&s);
        times.push(t.elapsed().as_secs_f64());
    }
    let t = timing(&times).unwrap_or_else(|| fail("no frames timed".into()));
    let (hash, lit) = digest(&picture(&s));
    println!("timed: {} frames, {:.2} fps, median {:.3} ms, worst {:.3} ms", t.frames, t.fps, t.median_ms, t.worst_ms);
    println!("picture: {} x {}, aspect {:.4}, rate {:.3} Hz, rendered {}, serial {}, digest {hash:016x}, {lit} bytes lit", last.width, last.height, last.display_aspect, last.frame_rate_hz, last.rendered, last.picture_serial);
    if let Ok(png) = std::env::var("EMUSEN_GAMES_PNG") {
        let p = picture(&s);
        if last.width > 0 && p.len() == (last.width * last.height * 4) as usize {
            match emusen_crystal::png::write(&png, &p, last.width as u32, last.height as u32) {
                Ok(()) => println!("png: {png}"),
                Err(e) => println!("png: {e}"),
            }
        } else {
            println!("png: none, {} bytes for {} x {}", p.len(), last.width, last.height);
        }
    }

    let state = home.root.join("probe.state").to_string_lossy().into_owned();
    match s.save_state(&state, None, false) {
        Err(e) => println!("state: save refused: {}", e.1),
        Ok(()) => {
            let size = std::fs::metadata(&state).map(|m| m.len()).unwrap_or(0);
            let saved = (frame(&s), digest(&picture(&s)));
            for _ in 0..60 {
                frame(&s);
            }
            let moved = digest(&picture(&s));
            match rt.with_ctx(|ctx| s.load_state(&state, None, ctx)) {
                Err(e) => println!("state: {size} bytes saved, load refused: {}", e.1),
                Ok(()) => {
                    let loaded = (frame(&s), digest(&picture(&s)));
                    let later: Vec<String> = (0..4).map(|_| format!("{:016x}", (frame(&s), digest(&picture(&s))).1.0)).collect();
                    println!(
                        "state: {size} bytes; frame after save {:016x}, 60 frames on {:016x}, after load {:016x} ({}), then {}",
                        saved.1.0,
                        moved.0,
                        loaded.1.0,
                        if loaded.1.0 == saved.1.0 { "the same picture" } else { "a different picture" },
                        later.join(" ")
                    );
                    if let Ok(png) = std::env::var("EMUSEN_GAMES_PNG_LOADED") {
                        let (p, f) = (picture(&s), frame(&s));
                        if f.width > 0 && p.len() == (f.width * f.height * 4) as usize {
                            let _ = emusen_crystal::png::write(&png, &p, f.width as u32, f.height as u32);
                        }
                    }
                }
            }
        }
    }
    drop(s);
    rt.close(id).unwrap_or_else(|e| fail(e.1));
    println!("closed");
    let mut said = vec![0u8; log.drain(None)];
    log.drain(Some(&mut said));
    let lines = std::env::var("EMUSEN_GAMES_LOG").ok().and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
    for e in String::from_utf8_lossy(&said).split("},{").take(lines) {
        println!("log: {e}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        fail("usage: games <home> install <runner> <stem>... | approve <stem>... | tool <request> | run <console> <game> <boot> <frames> [key=value]...".into());
    }
    let home = Home::at(Path::new(&args[0]));
    match (args[1].as_str(), &args[2..]) {
        ("install", [runner, stems @ ..]) if !stems.is_empty() => install(&home, runner, stems),
        ("approve", stems) if !stems.is_empty() => approve(&home, stems),
        ("tool", [request]) => run_job(request, tool_context(&home), &mut |l| println!("{l}")).unwrap_or_else(|e| fail(e)),
        ("run", [console, game, boot, frames, settings @ ..]) => {
            let n = |s: &str| s.parse::<usize>().unwrap_or_else(|_| fail(format!("{s} is not a count")));
            run(&home, console, game, n(boot), n(frames), settings)
        }
        _ => fail(format!("{} is not install, approve, tool or run with its arguments", args[1])),
    }
}
