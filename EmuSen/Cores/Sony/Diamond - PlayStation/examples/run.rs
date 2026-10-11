//! Runs a PS-EXE on Diamond for a number of frames and prints its TTY: `run <exe> [frames] [--bios FILE | --bare] [--pal]
//! [--dump ADDRESS WORDS] [--ram FILE] [--vram FILE] [--picture FILE]`: RAM's words from a hex address, or RAM, VRAM or
//! the RGBA picture (its size printed) to a file.
//! `--heading TEXT` prints the TTY from TEXT on, leaving out what a BIOS printed before the program; `--trace` then
//! traces the controller's commands and responses. `--disc PATH` puts a disc in the drive (a cue sheet with its tracks beside it, a CHD, an `.iso` or a lone `.bin`;
//! the image itself when PATH is no PS-EXE), `--swap` opens and closes the lid a second after the TTY asks, as ps1-tests'
//! `cdrom/disc-swap` does. A development tool for the test programs (Diamond_Native.md §11.11, Diamond_CDROM.md §9):
//! on the open kernel, a player's own BIOS, or none.

use diamond::kernel::{self, Kernel};
use diamond::machine::{Config, Gpu, Machine, TTY_ID};
use diamond::media::{Market, Media};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let image = std::fs::read(&args[0]).expect("the program");
    let disc_path = args.iter().position(|a| a == "--disc").map(|i| args[i + 1].clone()).or_else(|| (!image.starts_with(b"PS-X EXE")).then(|| args[0].clone()));
    let frames: i64 = args.get(1).and_then(|f| f.parse().ok()).unwrap_or(300);
    let bios = args.iter().position(|a| a == "--bios").map(|i| std::fs::read(&args[i + 1]).expect("the BIOS"));
    let market = if args.iter().any(|a| a == "--pal") { Market::Europe } else { Market::Americas };
    let load = std::time::Instant::now();
    let disc = disc_path.map(|p| open_disc(&p));
    let loaded = load.elapsed();
    let media = match &disc {
        Some((_, media)) if !image.starts_with(b"PS-X EXE") => *media,
        _ => Media::read(&image).expect("a PS-EXE"),
    };
    let market = if args.iter().any(|a| a == "--pal") { Market::Europe } else { media.market.unwrap_or(market) };
    let mut m = Machine::new(media, Config { market, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, rom(&bios, args.iter().any(|a| a == "--bare")));
    if let Some((d, _)) = disc {
        m.insert_disc(d);
        eprintln!("[disc opened in {:.0} ms]", loaded.as_secs_f64() * 1e3);
    }
    let swap = args.iter().any(|a| a == "--swap");
    let heading = args.iter().position(|a| a == "--heading").map(|i| args[i + 1].clone());
    let trace = args.iter().any(|a| a == "--trace");
    let (mut last_ask, mut asked_at) = ((0, 0), 0);
    let start = std::time::Instant::now();
    for _ in 0..frames {
        m.advance();
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
