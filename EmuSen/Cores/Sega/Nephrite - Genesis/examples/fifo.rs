//! Runs Nemesis's VDPFIFOTesting with A pressed for its results and prints each test's record against its expected
//! data (Nephrite_Native.md §13). `fifo <image> [frames]`.

use nephrite::fifo_records::{press, records};
use nephrite::machine::Machine;
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = std::fs::read(&args[1]).expect("the image");
    let frames: i64 = args.get(2).map_or(1200, |f| f.parse().unwrap());
    let mut m = Machine::new(&image, Media::read(&image));
    while m.frames < frames {
        m.pads[0] = press(m.frames);
        m.advance();
    }
    let hex = |b: &[u8]| b.chunks(2).map(|w| format!("{:02X}{:02X}", w[0], w.get(1).copied().unwrap_or(0))).collect::<Vec<_>>().join(" ");
    let rs = records(&m.genesis.hw.wram);
    for (i, r) in rs.iter().enumerate() {
        if r.passed() {
            println!("{:3} pass {}", i + 1, r.name);
        } else {
            println!("{:3} FAIL {}\n      expected {}\n      actual   {}", i + 1, r.name, hex(&r.expected), hex(&r.actual));
        }
    }
    let ram = &m.genesis.hw.wram;
    println!("{} of {} pass; the program's own counts: pass {} fail {}", rs.iter().filter(|r| r.passed()).count(), rs.len(), u16::from_be_bytes([ram[0xFF08], ram[0xFF09]]), u16::from_be_bytes([ram[0xFF0A], ram[0xFF0B]]));
}
