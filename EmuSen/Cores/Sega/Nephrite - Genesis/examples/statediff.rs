//! Runs an image to a frame, saves, loads the state into a second machine, and runs both on with the same pads,
//! printing the first frame whose picture or state differs and the state's fields that differ (Nephrite_Native.md §39).
//! `statediff <image> <save-frame> <frames-after>`; NEPHRITE_KIT_PADS=1 presses pad 1's buttons in turn as the kit does.

use emusen_native::ffi::StateMachine;
use nephrite::machine::Machine;
use nephrite::media::Media;

fn save(m: &Machine) -> Vec<u8> {
    let mut s = vec![0; m.state_size()];
    let n = m.save_state(&mut s).expect("a state");
    s.truncate(n);
    s
}

fn fields(layout: &str, a: &[u8], b: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    for line in layout.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let (at, len): (usize, usize) = (w[0].parse().unwrap(), w[1].parse().unwrap());
        if a.get(at..at + len) != b.get(at..at + len) {
            let first = (at..at + len).find(|&i| a.get(i) != b.get(i)).unwrap_or(at);
            out.push(format!("{} (byte {} of {len}: {:02X} against {:02X})", w[3..].join(" "), first - at, a.get(first).copied().unwrap_or(0), b.get(first).copied().unwrap_or(0)));
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = nephrite::media::cartridge_bytes(&std::fs::read(&args[1]).expect("the image")).into_owned();
    let at: i64 = args[2].parse().unwrap();
    let after: i64 = args[3].parse().unwrap();
    let kit = std::env::var("NEPHRITE_KIT_PADS").is_ok();
    let pads = |f: i64| -> u32 {
        if !kit || f < 8 { return 0 }
        let i = (f - 8) / 16;
        if (f - 8) % 16 < 8 && i < 64 { 1 << (i % 12) } else { 0 }
    };
    let mut a = Machine::new(&image, Media::read(&image));
    while a.frames < at {
        a.pads[0] = pads(a.frames);
        a.advance();
    }
    let state = save(&a);
    let dump = std::env::var("NEPHRITE_DIFF_DUMP").ok();
    if let Some(d) = &dump {
        std::fs::write(format!("{d}/before_{}x{}.rgba", a.picture.len() / 4 / a.genesis.hw.vdp.frame.height.max(1), a.genesis.hw.vdp.frame.height), &a.picture).unwrap();
    }
    let mut b = Machine::new(&image, Media::read(&image));
    b.load_state(&state).expect("the state loads");
    let layout = a.layout();
    assert_eq!(save(&b), state, "save, load, save");
    let shape = |m: &Machine| { let v = &m.genesis.hw.vdp; (v.frame.width, v.frame.height, v.open, v.span_x, v.width(), m.genesis.hw.line) };
    println!("after the load: A {:?}, B {:?}", shape(&a), shape(&b));
    for _ in 0..after {
        let p = pads(a.frames);
        a.pads[0] = p;
        b.pads[0] = p;
        a.advance();
        b.advance();
        let (sa, sb) = (save(&a), save(&b));
        let picture = a.picture != b.picture;
        if picture || sa != sb {
            let rows: Vec<usize> = {
                let w = a.picture.len() / a.genesis.hw.vdp.frame.height.max(1);
                (0..a.picture.len() / w.max(1)).filter(|y| a.picture.get(y * w..(y + 1) * w) != b.picture.get(y * w..(y + 1) * w)).collect()
            };
            println!("frame {}: picture {} ({} rows differ, first {:?}), state {}", a.frames, if picture { "differs" } else { "equal" }, rows.len(), rows.first(), if sa == sb { "equal" } else { "differs" });
            if let Some(d) = &dump {
                let h = a.genesis.hw.vdp.frame.height.max(1);
                std::fs::write(format!("{d}/a_{}x{h}.rgba", a.picture.len() / 4 / h), &a.picture).unwrap();
                std::fs::write(format!("{d}/b_{}x{h}.rgba", b.picture.len() / 4 / h), &b.picture).unwrap();
            }
            if let Some(&y) = rows.first() {
                let w = a.picture.len() / 4 / a.genesis.hw.vdp.frame.height.max(1);
                let px = |p: &[u8], x: usize| format!("{:02X}{:02X}{:02X}", p[(y * w + x) * 4], p[(y * w + x) * 4 + 1], p[(y * w + x) * 4 + 2]);
                let xs: Vec<usize> = (0..w).filter(|&x| px(&a.picture, x) != px(&b.picture, x)).collect();
                println!("  row {y}: {} pixels differ from x {:?} to {:?}; A {} B {} at the first; size {}x{}", xs.len(), xs.first(), xs.last(), xs.first().map(|&x| px(&a.picture, x)).unwrap_or_default(), xs.first().map(|&x| px(&b.picture, x)).unwrap_or_default(), w, a.genesis.hw.vdp.frame.height);
                println!("  rows {:?}", rows);
            }
            for f in fields(&layout, &sa, &sb).iter().take(20) {
                println!("  {f}");
            }
            if sa != sb {
                return;
            }
        }
    }
    println!("no difference in {after} frames after {at}");
}
