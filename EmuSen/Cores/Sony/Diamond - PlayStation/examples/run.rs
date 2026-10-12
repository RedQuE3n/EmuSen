//! Runs a PS-EXE on Diamond for a number of frames and prints its TTY: `run <exe> [frames] [--bios FILE | --bare] [--pal]
//! [--dump ADDRESS WORDS] [--ram FILE] [--vram FILE] [--picture FILE]`: RAM's words from a hex address, or RAM, VRAM or
//! the RGBA picture (its size printed) to a file.
//! `--disasm ADDRESS N` disassembles N words of a game's RAM after the run. `--armed` runs each frame through the debugger's loop with the kit's C15 tables armed (Diamond_Debugger.md), and `--armed-reads` the read watch and read data breakpoint of 1.2 beside them, with nothing to hit. `--wav FILE` writes the SPU's output, 16-bit stereo at 44,100 Hz. `--boot` boots the disc through the kernel's LoadExec from a loader, without the BIOS's intro (Diamond_CDROM.md §9).
//! `--heading TEXT` prints the TTY from TEXT on, leaving out what a BIOS printed before the program; `--trace` then
//! traces the controller's commands and responses, as `--trace-game` does from the entry of a game booted by `--boot`, once the BIOS has handed over. `--disc PATH` puts a disc in the drive (a cue sheet with its tracks beside it, a CHD, an `.iso` or a lone `.bin`;
//! the image itself when PATH is no PS-EXE), `--analog` presses port 1's Analog button at frame 30; `--empty` leaves the drive empty; `--pictures DIR N` writes the picture every N frames as `fFRAME_WxH.rgba`; `--press F:MASK[:N]` holds the buttons of a hex mask of the pad report's bits for N frames, `--stick F:AXIS:VALUE[:N]` a stick (0-1 the left x and y, 2-3 the right, -1 to 1); `--swap` opens and closes the lid a second after the TTY asks, as ps1-tests'
//! `cdrom/disc-swap` does; `--trace-at FRAME` traces the controller from a frame on; `--calls` counts the kernel calls a program makes by table and number, as the program
//! enters the A0h, B0h and C0h vectors, a BIOS's own calls among them. `--game` observes only the game, from its entry on, on either kernel: the TTY from the entry,
//! the text it passes to write(1) and putchar, its calls counted, and with `--game-log FILE` each call's arguments and result and its own loads and stores at
//! the CD, SIO0, DMA and interrupt registers, `--log-frames A:B` limiting the accesses (Diamond_OpenKernel_Phase2.md §22). A development tool for the test programs (Diamond_Native.md §11.11, Diamond_CDROM.md §9):
//! on the open kernel, a player's own BIOS, or none.

use diamond::kernel::{self, Kernel};
use diamond::machine::{Config, Gpu, Machine, TTY_ID};
use diamond::media::{Market, Media};

/// A frame through the debugger's loop, its full logs drained and continued as a host does.
fn armed_frame(m: &mut Machine) {
    while m.run_frame_debug(emusen_native::debug::run::CONTINUE) != 0 {
        m.debug.hooks.calls_log.clear();
        m.debug.hooks.writes_log.clear();
        m.debug.hooks.reads_log.clear();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut image = std::fs::read(&args[0]).expect("the program");
    let boot = args.iter().any(|a| a == "--boot");
    let mut entry = 0u32;
    let disc_path = args.iter().position(|a| a == "--disc").map(|i| args[i + 1].clone()).or_else(|| (!image.starts_with(b"PS-X EXE")).then(|| args[0].clone()));
    let frames: i64 = args.get(1).and_then(|f| f.parse().ok()).unwrap_or(300);
    let bios = args.iter().position(|a| a == "--bios").map(|i| std::fs::read(&args[i + 1]).expect("the BIOS"));
    let market = if args.iter().any(|a| a == "--pal") { Market::Europe } else { Market::Americas };
    let load = std::time::Instant::now();
    let mut disc = disc_path.map(|p| open_disc(&p));
    let loaded = load.elapsed();
    let game = args.iter().any(|a| a == "--game");
    if boot {
        let (d, _) = disc.as_mut().expect("a disc to boot");
        let (l, e) = loader(d);
        image = l;
        entry = e;
    } else if game {
        entry = boot_program(disc.as_mut().map(|d| &mut d.0).expect("a disc for --game")).2;
    }
    let media = match &disc {
        Some((_, media)) if !image.starts_with(b"PS-X EXE") || boot => if boot { Media { kind: Media::read(&image).unwrap().kind, market: media.market } } else { *media },
        _ => Media::read(&image).expect("a PS-EXE"),
    };
    let market = if args.iter().any(|a| a == "--pal") { Market::Europe } else { media.market.unwrap_or(market) };
    let mut m = Machine::new(media, Config { market, gpu: if args.iter().any(|a| a == "--gpu160") { Gpu::Pin160 } else { Gpu::Pin208 } }, [Some("psx.dualshock"), if args.iter().any(|a| a == "--two-pads") { Some("psx.dualshock") } else { None }], &image, rom(&bios, args.iter().any(|a| a == "--bare")));
    if let Some((d, _)) = disc
        && !args.iter().any(|a| a == "--empty")
    {
        m.insert_disc(d);
        eprintln!("[disc opened in {:.0} ms]", loaded.as_secs_f64() * 1e3);
    }
    let card_files = ["--card", "--card2"].map(|flag| args.iter().position(|a| a == flag).map(|i| args[i + 1].clone()));
    for (n, f) in card_files.iter().enumerate() {
        if let Some(f) = f {
            let data = std::fs::read(f).ok();
            m.insert_card(n, data.as_deref());
        }
    }
    let presses: Vec<(i64, u32, i64)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--press")
        .map(|(i, _)| {
            let p: Vec<&str> = args[i + 1].split(':').collect();
            (p[0].parse().expect("a frame"), u32::from_str_radix(p[1], 16).expect("a hex mask"), p.get(2).map_or(6, |n| n.parse().expect("frames")))
        })
        .collect();
    let sticks: Vec<(i64, u32, f64, i64)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--stick")
        .map(|(i, _)| {
            let p: Vec<&str> = args[i + 1].split(':').collect();
            (p[0].parse().expect("a frame"), p[1].parse().expect("an axis"), p[2].parse().expect("a value"), p.get(3).map_or(6, |n| n.parse().expect("frames")))
        })
        .collect();
    if args.iter().any(|a| a == "--cards") {
        m.insert_card(0, None);
        m.insert_card(1, None);
    }
    if args.iter().any(|a| a == "--lid-open") {
        m.set_lid(true, None);
    }
    let swap = args.iter().any(|a| a == "--swap");
    let reads = args.iter().any(|a| a == "--armed-reads");
    let armed = reads || args.iter().any(|a| a == "--armed");
    if armed {
        use emusen_native::debug::{flag, range};
        m.debug.hooks.configure(flag::CALLS | flag::WRITES | flag::PROFILING | 1 << flag::COVERAGE | if reads { flag::READS } else { 0 }, i32::MIN, -1);
        m.debug.hooks.set_breakpoints(&[-2, -2]);
        m.debug.hooks.set_ranges(range::WATCH, &[diamond::machine::RAM_ID, 0x1F_FFFC, 0x1F_FFFF]);
        if reads {
            m.debug.hooks.set_ranges(range::READ_WATCH, &[diamond::machine::RAM_ID, 0x1F_FFFC, 0x1F_FFFF]);
            m.debug.hooks.set_ranges(range::READ_BREAK, &[diamond::machine::RAM_ID, 0x1F_FFFC, 0x1F_FFFF]);
        }
    }
    let heading = args.iter().position(|a| a == "--heading").map(|i| args[i + 1].clone());
    let trace = args.iter().any(|a| a == "--trace");
    let (mut last_ask, mut asked_at) = ((0, 0), 0);
    let analog = args.iter().any(|a| a == "--analog");
    let trace_game = args.iter().any(|a| a == "--trace-game");
    let lid_at = args.iter().position(|a| a == "--open-at").map(|i| args[i + 1].parse::<i64>().expect("a frame"));
    let every = args.iter().position(|a| a == "--pictures").map(|i| (args[i + 1].clone(), args[i + 2].parse::<i64>().expect("a frame count")));
    let calls = args.iter().any(|a| a == "--calls");
    let trace_at = args.iter().position(|a| a == "--trace-at").map(|i| args[i + 1].parse::<i64>().expect("a frame"));
    let mut seen: Vec<(char, u32, u64, i64)> = Vec::new();
    let wav = args.iter().position(|a| a == "--wav").map(|i| args[i + 1].clone());
    let mut audio: Vec<i16> = Vec::new();
    let mut chunk = vec![0i16; 8192];
    let mut watch = game.then(|| Watch::new(entry, args.iter().position(|a| a == "--game-log").map(|i| args[i + 1].clone()), args.iter().position(|a| a == "--log-frames").map(|i| args[i + 1].clone())));
    let start = std::time::Instant::now();
    for _ in 0..frames {
        if wav.is_some() {
            let n = m.board.spu.out.drain(&mut chunk, usize::MAX);
            audio.extend_from_slice(&chunk[..n]);
        }
        if trace_at == Some(m.frames) {
            m.board.cdrom.trace = true;
        }
        if !presses.is_empty() {
            let mask = presses.iter().filter(|(at, _, n)| (*at..*at + *n).contains(&m.frames)).fold(0, |a, p| a | p.1);
            m.board.sio0.set_buttons(0, mask);
        }
        for axis in 0..4 {
            if sticks.iter().any(|s| s.1 == axis) {
                let v = sticks.iter().rev().find(|(at, a, _, n)| *a == axis && (*at..*at + *n).contains(&m.frames)).map_or(0.0, |s| s.2);
                if let Some(c) = m.board.sio0.controller(0) {
                    c.set_axis(axis, v);
                }
            }
        }
        if calls {
            m.board.field_done = false;
            while !m.board.field_done {
                let (pc, n) = (m.cpu.regs.pc & 0x1FFF_FFFF, m.cpu.regs.r[9]);
                if matches!(pc, 0xA0 | 0xB0 | 0xC0) && !m.cpu.regs.branch.slot {
                    let t = (b'A' + ((pc - 0xA0) >> 4) as u8) as char;
                    match seen.iter_mut().find(|e| e.0 == t && e.1 == n) {
                        Some(e) => e.2 += 1,
                        None => seen.push((t, n, 1, m.frames)),
                    }
                }
                m.step();
            }
            m.board.sync_spu();
            m.frames += 1;
            if let Some((dir, n)) = &every
                && m.frames % n == 0
            {
                let d = &m.board.gpu.display;
                std::fs::write(format!("{dir}/f{:06}_{}x{}.rgba", m.frames, d.width, d.height), d.frame()).expect("written");
            }
            continue;
        }
        if trace_game && !m.board.cdrom.trace {
            m.board.field_done = false;
            while !m.board.field_done {
                if m.cpu.regs.pc == entry && !m.board.cdrom.trace {
                    m.board.cdrom.trace = true;
                    eprintln!("[the game starts at frame {}]", m.frames);
                }
                m.step();
            }
            m.frames += 1;
            continue;
        }
        if analog && (30..40).contains(&m.frames) {
            m.board.sio0.set_buttons(0, if m.frames < 35 { 1 << 16 } else { 0 });
        }
        match &mut watch {
            Some(w) => w.frame(&mut m),
            None if armed => armed_frame(&mut m),
            None => m.advance(),
        }
        if let Some((dir, n)) = &every
            && m.frames % n == 0
        {
            let d = &m.board.gpu.display;
            std::fs::write(format!("{dir}/f{:06}_{}x{}.rgba", m.frames, d.width, d.height), d.frame()).expect("written");
        }
        if lid_at.is_some_and(|f| f == m.frames) {
            m.set_lid(true, None);
        }

        if trace && !m.board.cdrom.trace && heading.as_ref().is_some_and(|h| String::from_utf8_lossy(m.space(TTY_ID).unwrap()).contains(h.as_str())) {
            m.board.cdrom.trace = true;
        }
        if swap {
            let tty = String::from_utf8_lossy(m.space(TTY_ID).unwrap()).into_owned();
            let asked = (tty.matches("Open the shell now").count(), tty.matches("Close the shell now").count());
            if asked != last_ask {
                last_ask = asked;
                asked_at = m.frames;
            }
            let due = m.frames >= asked_at + 60;
            if asked.0 > asked.1 && !m.board.cdrom.lid_open && due {
                m.set_lid(true, None);
            } else if asked.1 == asked.0 && asked.1 > 0 && m.board.cdrom.lid_open && due {
                m.set_lid(false, None);
            }
        }
    }
    if let Some(path) = &wav {
        let n = m.board.spu.out.drain(&mut chunk, usize::MAX);
        audio.extend_from_slice(&chunk[..n]);
        write_wav(path, &audio);
    }
    for (n, f) in card_files.iter().enumerate() {
        if let (Some(f), Some(c)) = (f, m.card_in(n))
            && c.changed
        {
            std::fs::write(f, &c.data).expect("the card written");
            eprintln!("[card written to {f}]");
        }
    }
    for (t, n, count, first) in &seen {
        eprintln!("[call {t}({n:02X}h) {count} times, first at frame {first}]");
    }
    if let Some(w) = &mut watch {
        w.finish();
    }
    let tty = m.space(TTY_ID).unwrap();
    let end = tty.iter().position(|&b| b == 0).unwrap_or(tty.len());
    let from = watch.as_ref().map_or(0, |w| w.tty_from.unwrap_or(end));
    let text = String::from_utf8_lossy(&tty[from.min(end)..end]);
    match &heading {
        Some(h) => print!("{}", text.find(h.as_str()).map_or("", |at| &text[at..])),
        None => print!("{text}"),
    }
    if let Some(i) = args.iter().position(|a| a == "--dump") {
        let at = u32::from_str_radix(args[i + 1].trim_start_matches("0x"), 16).expect("a hex address");
        let n: u32 = args[i + 2].parse().expect("a count");
        for k in 0..n {
            print!("{:08X}{}", m.board.ram_word(at + 4 * k), if k % 8 == 7 { "\n" } else { " " });
        }
        println!();
    }
    if let Some(i) = args.iter().position(|a| a == "--disasm") {
        let at = u32::from_str_radix(args[i + 1].trim_start_matches("0x"), 16).expect("a hex address");
        let n: u32 = args[i + 2].parse().expect("a count");
        let dialect = blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR));
        for k in 0..n {
            let a = at + 4 * k;
            println!("{a:08X}  {}", emusen_mips::disassemble(m.board.ram_word(a), a, &dialect));
        }
    }
    if let Some(i) = args.iter().position(|a| a == "--ram") {
        std::fs::write(&args[i + 1], &m.board.ram).expect("written");
    }
    if let Some(i) = args.iter().position(|a| a == "--vram") {
        std::fs::write(&args[i + 1], &m.board.gpu.vram.bytes).expect("written");
    }
    if let Some(i) = args.iter().position(|a| a == "--picture") {
        let d = &m.board.gpu.display;
        std::fs::write(&args[i + 1], d.frame()).expect("written");
        eprintln!("picture {}x{}", d.width, d.height);
    }
    eprintln!("\n[{} frames, {} cycles, pc {:08X}, {:.1} ms]", m.frames, m.board.now, m.cpu.regs.pc, start.elapsed().as_secs_f64() * 1e3);
}

/// A sector's 2,048 bytes of user data.
fn user(d: &mut emusen_disc::Disc, lba: u32) -> Vec<u8> {
    let mut raw = [0u8; 2352];
    d.read(150 + lba, &mut raw);
    raw[24..2072].to_vec()
}

/// A loader for `--boot`: SYSTEM.CNF's BOOT and STACK read from the disc's root directory (ECMA-119), then the kernel's
/// A(71h) _96_init and A(51h) LoadExec, so that the player's BIOS loads and starts the game with its own functions.
fn loader(d: &mut emusen_disc::Disc) -> (Vec<u8>, u32) {
    let (name, stack, entry) = boot_program(d);
    eprintln!("[booting {name} with the stack at {stack:08X}, its entry {entry:08X}]");
    let mut src = String::from("j start\nnop\nstart: lui a3, 0x8010\n");
    for (i, c) in name.bytes().chain([0]).enumerate() {
        src += &format!("ori t0, zero, {c}\nsb t0, {}(a3)\n", 0x100 + i);
    }
    src += &format!("ori t1, zero, 0x71\nori t2, zero, 0xA0\njalr t2\nnop\nlui a0, 0x8010\nori a0, a0, 0x100\nlui a1, {}\nori a1, a1, {}\nori a2, zero, 0\nori t1, zero, 0x51\nori t2, zero, 0xA0\njalr t2\nnop\ndone: j done\nnop\n", stack >> 16, stack & 0xFFFF);
    let words = emusen_mips::assemble(&src, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    (diamond::exe::build(&words), entry)
}

/// SYSTEM.CNF's BOOT name and STACK, and the boot program's entry from its header.
fn boot_program(d: &mut emusen_disc::Disc) -> (String, u32, u32) {
    let pvd = user(d, 16);
    let root = u32::from_le_bytes([pvd[158], pvd[159], pvd[160], pvd[161]]);
    let dir = user(d, root);
    let mut at = 0;
    let mut cnf = None;
    while at < dir.len() && dir[at] != 0 {
        let len = dir[at] as usize;
        let name = &dir[at + 33..at + 33 + dir[at + 32] as usize];
        if name.starts_with(b"SYSTEM.CNF") {
            cnf = Some(u32::from_le_bytes([dir[at + 2], dir[at + 3], dir[at + 4], dir[at + 5]]));
        }
        at += len;
    }
    let text = String::from_utf8_lossy(&user(d, cnf.expect("SYSTEM.CNF in the root"))).into_owned();
    let field = |key: &str| text.lines().find_map(|l| l.split_once('=').filter(|(k, _)| k.trim() == key).map(|(_, v)| v.trim().trim_end_matches('\0').to_owned()));
    let name = field("BOOT").expect("a BOOT line");
    let entry = find(d, root, &name).map_or(0, |lba| {
        let h = user(d, lba);
        u32::from_le_bytes([h[0x10], h[0x11], h[0x12], h[0x13]])
    });
    let stack = field("STACK").and_then(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16).ok()).unwrap_or(0x801F_FFF0);
    (name, stack, entry)
}

/// A file's first sector by its path from the root, `cdrom:\\DIR\\NAME;1` (ECMA-119 directory records).
fn find(d: &mut emusen_disc::Disc, root: u32, path: &str) -> Option<u32> {
    let mut dir = root;
    let parts: Vec<&str> = path.trim_start_matches("cdrom:").trim_start_matches('\\').split('\\').collect();
    for (i, part) in parts.iter().enumerate() {
        let data = user(d, dir);
        let mut at = 0;
        let mut next = None;
        while at < data.len() && data[at] != 0 {
            let name = &data[at + 33..at + 33 + data[at + 32] as usize];
            if name.eq_ignore_ascii_case(part.as_bytes()) || (i + 1 < parts.len() && name == part.as_bytes()) {
                next = Some(u32::from_le_bytes([data[at + 2], data[at + 3], data[at + 4], data[at + 5]]));
            }
            at += data[at] as usize;
        }
        dir = next?;
    }
    Some(dir)
}

/// The disc at `path`: a sheet with its tracks read from beside it, or one image file.
fn open_disc(path: &str) -> (emusen_disc::Disc, Media) {
    let image = std::fs::read(path).expect("the disc image");
    let dir = std::path::Path::new(path).parent().unwrap_or(std::path::Path::new("."));
    let mut files = Vec::new();
    if diamond::medium::is_sheet(&image) {
        for name in emusen_disc::set::companions(&String::from_utf8_lossy(&image)).expect("a cue sheet") {
            files.push(std::fs::read(dir.join(&name)).unwrap_or_else(|_| panic!("the track file {name}")));
        }
    }
    if let Ok(sbi) = std::fs::read(emusen_disc::set::sbi_beside(path)) {
        files.push(sbi);
    }
    let refs: Vec<&[u8]> = files.iter().map(|f| f.as_slice()).collect();
    match diamond::medium::open(&image, &refs) {
        Ok(diamond::medium::Opened::Disc(d, media)) => (*d, media),
        Ok(_) => panic!("a PS-EXE, not a disc"),
        Err(e) => panic!("the disc was refused: {e:?}"),
    }
}

fn rom(bios: &Option<Vec<u8>>, bare: bool) -> Option<(&[u8], Kernel)> {
    match bios {
        Some(b) => Some((b, Kernel::Player)),
        None if bare => None,
        None => kernel::open_kernel().map(|k| (k, Kernel::Open)),
    }
}

/// A canonical WAV of 16-bit stereo at the SPU's 44,100 Hz.
fn write_wav(path: &str, samples: &[i16]) {
    let data = samples.len() as u32 * 2;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    for v in [16u32, 1 | 2 << 16, 44_100, 44_100 * 4, 4 | 16 << 16] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(path, out).expect("written");
}

/// What `--game` sees of the game alone: code in user RAM, from its entry on, so that a kernel's own code is never observed.
struct Watch {
    entry: u32,
    started: bool,
    tty_from: Option<usize>,
    log: Option<std::io::BufWriter<std::fs::File>>,
    window: (i64, i64),
    pending: Vec<(u32, char, u32)>,
    counts: Vec<(char, u32, u64, i64)>,
    last: Option<(u32, bool, u32, u32)>,
    repeats: u64,
    text: Vec<u8>,
}

/// The registers whose accesses `--game-log` records: interrupts, DMA, SIO0 and the CD-ROM controller.
fn watched(address: u32) -> bool {
    matches!(address, 0x1F80_1040..=0x1F80_104F | 0x1F80_1070..=0x1F80_1077 | 0x1F80_1080..=0x1F80_10FF | 0x1F80_1800..=0x1F80_1803)
}

fn user_ram(address: u32) -> bool {
    (0x1_0000..0x20_0000).contains(&(address & 0x1FFF_FFFF))
}

impl Watch {
    fn new(entry: u32, log: Option<String>, window: Option<String>) -> Watch {
        let window = window.map_or((0, i64::MAX), |w| {
            let (a, b) = w.split_once(':').expect("A:B");
            (a.parse().expect("a frame"), b.parse().expect("a frame"))
        });
        let log = log.map(|p| std::io::BufWriter::new(std::fs::File::create(p).expect("the log")));
        Watch { entry, started: false, tty_from: None, log, window, pending: Vec::new(), counts: Vec::new(), last: None, repeats: 0, text: Vec::new() }
    }

    fn frame(&mut self, m: &mut Machine) {
        use std::io::Write;
        m.board.field_done = false;
        let accesses = (self.window.0..self.window.1).contains(&m.frames);
        while !m.board.field_done {
            let (pc, slot) = (m.cpu.regs.pc, m.cpu.regs.branch.slot);
            if !self.started {
                if pc == self.entry && !slot {
                    self.started = true;
                    self.tty_from = Some(m.board.tty.len);
                    eprintln!("[the game starts at frame {}]", m.frames);
                }
                m.step();
                continue;
            }
            if let Some(k) = self.pending.iter().rposition(|p| p.0 == pc) {
                let (_, t, n) = self.pending[k];
                self.pending.truncate(k);
                if let Some(l) = &mut self.log {
                    writeln!(l, "{} {} RET {t}({n:02X}h) v0={:08X}", m.frames, m.board.now, m.cpu.regs.r[2]).unwrap();
                }
            }
            let r = &m.cpu.regs.r;
            if matches!(pc & 0x1FFF_FFFF, 0xA0 | 0xB0 | 0xC0) && !slot && user_ram(r[31]) {
                let (t, n) = ((b'A' + (((pc & 0xFF) - 0xA0) >> 4) as u8) as char, r[9]);
                match self.counts.iter_mut().find(|e| e.0 == t && e.1 == n) {
                    Some(e) => e.2 += 1,
                    None => self.counts.push((t, n, 1, m.frames)),
                }
                match (t, n, r[4]) {
                    ('A', 0x03, 1) | ('B', 0x35, 1) => (0..r[6].min(0x1000)).for_each(|k| self.text.push(m.board.ram[(r[5].wrapping_add(k) & 0x1F_FFFF) as usize])),
                    ('A', 0x3C, c) | ('B', 0x3D, c) => self.text.push(c as u8),
                    _ => {}
                }
                if let Some(l) = &mut self.log {
                    writeln!(l, "{} {} CALL {t}({n:02X}h) a0={:08X} a1={:08X} a2={:08X} a3={:08X} ra={:08X}", m.frames, m.board.now, r[4], r[5], r[6], r[7], r[31]).unwrap();
                }
                if self.pending.len() == 64 {
                    self.pending.remove(0);
                }
                self.pending.push((r[31], t, n));
            }
            let mut access = None;
            if accesses && self.log.is_some() && user_ram(pc) {
                let w = m.board.ram_word(pc & 0x1FFF_FFFF);
                let op = w >> 26;
                if matches!(op, 0x20..=0x26 | 0x28..=0x2B) {
                    let (rs, rt) = ((w >> 21 & 31) as usize, (w >> 16 & 31) as usize);
                    let at = r[rs].wrapping_add(w as i16 as i32 as u32) & 0x1FFF_FFFF;
                    if watched(at) {
                        access = Some((op, at, rt, r[rt]));
                    }
                }
            }
            m.step();
            if let Some((op, at, rt, value)) = access {
                let taken = m.cpu.regs.pc & 0x1FFF_FFFF == 0x80 && m.cpu.regs.epc == pc;
                let seen = if taken {
                    None
                } else if op >= 0x28 {
                    Some((true, match op { 0x28 => value & 0xFF, 0x29 => value & 0xFFFF, _ => value }))
                } else if m.cpu.regs.load.register == Some(rt as u8) {
                    Some((false, m.cpu.regs.load.value))
                } else {
                    None
                };
                if let Some((write, v)) = seen {
                    let key = (pc, write, at, v);
                    if self.last == Some(key) {
                        self.repeats += 1;
                    } else {
                        let l = self.log.as_mut().unwrap();
                        if self.repeats > 0 {
                            writeln!(l, "{} {} REPEATED {}", m.frames, m.board.now, self.repeats).unwrap();
                        }
                        writeln!(l, "{} {} {pc:08X} {} {at:08X} {v:X}", m.frames, m.board.now, if write { 'W' } else { 'R' }).unwrap();
                        self.last = Some(key);
                        self.repeats = 0;
                    }
                }
            }
        }
        m.board.sync_spu();
        m.frames += 1;
    }

    fn finish(&mut self) {
        use std::io::Write;
        if let Some(l) = &mut self.log {
            l.flush().unwrap();
        }
        eprintln!("[the game's text by write(1) and putchar: {} bytes]\n{}[end of the game's text]", self.text.len(), String::from_utf8_lossy(&self.text));
        for (t, n, count, first) in &self.counts {
            eprintln!("[game call {t}({n:02X}h) {count} times, first at frame {first}]");
        }
    }
}
