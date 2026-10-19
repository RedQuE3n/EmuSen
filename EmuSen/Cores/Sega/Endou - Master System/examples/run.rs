//! Runs an image for a number of frames and prints what it left: the frames' cost, the board, and the cartridge's RAM
//! as text where a test program writes its report there, as ZEXALL's SMS port does (Endou_Native.md §9.8).
//! `cargo run --release --example run -- <image> <frames> [--model mark3|sms1|sms2] [--until <text>]`: with `--until`,
//! the run stops at the first second whose report holds the text.

use std::time::Instant;

use emusen_native::core::{Core, Create, Settings, sys};
use endou::machine::Machine;

fn report(m: &Machine) -> String {
    let text: String = m.bus.cart.ram.iter().map(|&b| if b == 13 { '\n' } else if (0x20..0x7F).contains(&b) { b as char } else { '.' }).collect();
    text.lines().map(str::trim_end).collect::<Vec<_>>().join("\n").trim_end().to_owned()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = std::fs::read(&args[1]).expect("the image");
    let frames: u64 = args.get(2).map_or(60, |f| f.parse().expect("a frame count"));
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let settings = flag("--model").map(|m| vec![("model".to_owned(), m)]).unwrap_or_default();
    let until = flag("--until");
    let request = Create { image: &image, settings: Settings::from_pairs(settings), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
    let mut m = Machine::create(&request).unwrap_or_else(|e| panic!("refused: {e}"));
    let start = Instant::now();
    let mut run = 0;
    while run < frames {
        Core::advance(&mut m, &mut 0).unwrap();
        run += 1;
        if run % 60 == 0 && until.as_ref().is_some_and(|u| report(&m).contains(u.as_str())) {
            break;
        }
    }
    let took = start.elapsed();
    println!("board {:?}, model {:?}, {} frames in {:.3} s, {:.4} ms a frame, {} Z80 clocks", m.bus.cart.board, m.model(), run, took.as_secs_f64(), took.as_secs_f64() * 1e3 / run as f64, m.bus.clock);
    let text = report(&m);
    if !text.is_empty() {
        println!("--- cartridge RAM ---\n{text}");
    }
}
