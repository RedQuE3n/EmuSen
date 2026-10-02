//! A ROM with the boot ROM and, if given, a NEC DSP's firmware, run with presses, its WRAM and picture written out.
//! `chip_run <rom> <firmware|-> <frames> <out> [frame:button:frames ...]`, buttons as PadButton numbers (Start 3, A 8).
#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = load::machine(&a[1]);
    if a[2] != "-" {
        assert!(m.attach_dsp(&std::fs::read(&a[2]).expect("the firmware")), "firmware of 8,192 or 53,248 bytes");
    }
    let frames: i64 = a[3].parse().unwrap();
    let presses: Vec<(i64, u16, i64)> = a[5..].iter().map(|p| {
        let v: Vec<i64> = p.split(':').map(|x| x.parse().unwrap()).collect();
        (v[0], 1 << v[1], v[2])
    }).collect();
    while m.total_frames() < frames {
        let f = m.total_frames();
        m.pads[0] = presses.iter().filter(|p| f >= p.0 && f < p.0 + p.2).fold(0, |s, p| s | p.1);
        m.run_frame();
    }
    std::fs::write(&a[4], &m.sys.wram[..]).unwrap();
    std::fs::write(format!("{}.rgba", a[4]), m.sys.ppu.picture()).unwrap();
    println!("{} frames, dsp {}", frames, m.sys.cart.dsp.as_ref().map(|(d, map)| format!("{:?} pc {:04X} sr {:04X} cycles {}", map, d.pc, d.sr, d.cycles)).unwrap_or("none".into()));
}
