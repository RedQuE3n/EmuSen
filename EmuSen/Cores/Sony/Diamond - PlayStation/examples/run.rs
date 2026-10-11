//! Runs a PS-EXE on Diamond for a number of frames and prints its TTY: `run <exe> [frames] [--bios FILE | --bare] [--pal]
//! [--dump ADDRESS WORDS] [--ram FILE] [--vram FILE] [--picture FILE]`: RAM's words from a hex address, or RAM, VRAM or
//! the RGBA picture (its size printed) to a file.
//! `--disasm ADDRESS N` disassembles N words of a game's RAM after the run. `--boot` boots the disc through the kernel's LoadExec from a loader, without the BIOS's intro (Diamond_CDROM.md §9).
//! `--heading TEXT` prints the TTY from TEXT on, leaving out what a BIOS printed before the program; `--trace` then
//! traces the controller's commands and responses, as `--trace-game` does from the entry of a game booted by `--boot`, once the BIOS has handed over. `--disc PATH` puts a disc in the drive (a cue sheet with its tracks beside it, a CHD, an `.iso` or a lone `.bin`;
//! the image itself when PATH is no PS-EXE), `--empty` leaves the drive empty; `--pictures DIR N` writes the picture every N frames as `fFRAME_WxH.rgba`; `--swap` opens and closes the lid a second after the TTY asks, as ps1-tests'
//! `cdrom/disc-swap` does. A development tool for the test programs (Diamond_Native.md §11.11, Diamond_CDROM.md §9):
//! on the open kernel, a player's own BIOS, or none.

use diamond::kernel::{self, Kernel};
use diamond::machine::{Config, Gpu, Machine, TTY_ID};
use diamond::media::{Market, Media};

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
    if boot {
        let (d, _) = disc.as_mut().expect("a disc to boot");
        let (l, e) = loader(d);
        image = l;
        entry = e;
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
    if args.iter().any(|a| a == "--cards") {
        m.insert_card(0, None);
        m.insert_card(1, None);
    }
    if args.iter().any(|a| a == "--lid-open") {
        m.set_lid(true, None);
    }
    let swap = args.iter().any(|a| a == "--swap");
    let heading = args.iter().position(|a| a == "--heading").map(|i| args[i + 1].clone());
    let trace = args.iter().any(|a| a == "--trace");
    let (mut last_ask, mut asked_at) = ((0, 0), 0);
    let trace_game = args.iter().any(|a| a == "--trace-game");
    let lid_at = args.iter().position(|a| a == "--open-at").map(|i| args[i + 1].parse::<i64>().expect("a frame"));
    let every = args.iter().position(|a| a == "--pictures").map(|i| (args[i + 1].clone(), args[i + 2].parse::<i64>().expect("a frame count")));
    let start = std::time::Instant::now();
    for _ in 0..frames {
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
        m.advance();
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
    let tty = m.space(TTY_ID).unwrap();
    let end = tty.iter().position(|&b| b == 0).unwrap_or(tty.len());
    let text = String::from_utf8_lossy(&tty[..end]);
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
    eprintln!("[booting {name} with the stack at {stack:08X}, its entry {entry:08X}]");
    let mut src = String::from("j start\nnop\nstart: lui a3, 0x8010\n");
    for (i, c) in name.bytes().chain([0]).enumerate() {
        src += &format!("ori t0, zero, {c}\nsb t0, {}(a3)\n", 0x100 + i);
    }
    src += &format!("ori t1, zero, 0x71\nori t2, zero, 0xA0\njalr t2\nnop\nlui a0, 0x8010\nori a0, a0, 0x100\nlui a1, {}\nori a1, a1, {}\nori a2, zero, 0\nori t1, zero, 0x51\nori t2, zero, 0xA0\njalr t2\nnop\ndone: j done\nnop\n", stack >> 16, stack & 0xFFFF);
    let words = emusen_mips::assemble(&src, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    (diamond::exe::build(&words), entry)
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
