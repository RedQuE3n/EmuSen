//! Runs a PS-EXE on Diamond for a number of frames and prints its TTY: `run <exe> [frames] [--bios FILE] [--pal]
//! [--dump ADDRESS WORDS] [--ram FILE]`: RAM's words from a hex address, or the whole of RAM to a file.
//! A development tool for the test programs (Diamond_Native.md §11.11); the BIOS, when given, is a player's own.

use diamond::kernel::Kernel;
use diamond::machine::{Config, Gpu, Machine, TTY_ID};
use diamond::media::{Market, Media};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let image = std::fs::read(&args[0]).expect("the program");
    let frames: i64 = args.get(1).and_then(|f| f.parse().ok()).unwrap_or(300);
    let bios = args.iter().position(|a| a == "--bios").map(|i| std::fs::read(&args[i + 1]).expect("the BIOS"));
    let market = if args.iter().any(|a| a == "--pal") { Market::Europe } else { Market::Americas };
    let media = Media::read(&image).expect("a PS-EXE");
    let mut m = Machine::new(media, Config { market, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, bios.as_deref().map(|b| (b, Kernel::Player)));
    let start = std::time::Instant::now();
    for _ in 0..frames {
        m.advance();
    }
    let tty = m.space(TTY_ID).unwrap();
    let end = tty.iter().position(|&b| b == 0).unwrap_or(tty.len());
    print!("{}", String::from_utf8_lossy(&tty[..end]));
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
    eprintln!("\n[{} frames, {} cycles, pc {:08X}, {:.1} ms]", m.frames, m.board.now, m.cpu.regs.pc, start.elapsed().as_secs_f64() * 1e3);
}
