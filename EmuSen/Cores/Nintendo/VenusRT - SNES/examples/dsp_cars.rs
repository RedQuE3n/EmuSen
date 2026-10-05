//! F1 ROC II's drivers on the image and on VenusRT's replacement, each machine run on its own from power-on under the
//! same pad script: `dsp_cars <rom> <frames> [script] [every]`. Every ST010 05h that completes is recorded with the
//! position it leaves (VenusRT_Native.md §54.1, words 62h-65h); the n-th 05h of a frame on one machine is compared with
//! the n-th on the other. Per window of `every` frames (300 by default): the drivers compared, those placed alike, and
//! the median, 95th percentile and largest distance in whole units. The report goes to stdout and to
//! ~/.cache/emusen/probe/venusrt/dsp-hle/cars-<rom>.txt.
use std::collections::BTreeMap;
use std::fmt::Write as _;
use venusrt::machine::Machine;

const BUTTONS: [&str; 12] = ["b", "y", "select", "start", "up", "down", "left", "right", "a", "x", "l", "r"];

/// The positions each 05h left, by frame, in the order they completed.
#[derive(Default)]
struct Drivers {
    busy: Option<u8>,
    by_frame: BTreeMap<u64, Vec<(f64, f64)>>,
}

impl Drivers {
    fn watch(&mut self, m: &Machine) {
        let Some((d, _)) = m.sys.cart.dsp.as_ref() else { return };
        let ram = d.ram();
        let mail = ram[0x10];
        match (self.busy, mail & 0x8000 != 0) {
            (None, true) => self.busy = Some(mail as u8),
            (Some(c), false) => {
                if c & 0x0F == 0x05 || c & 0x0F == 0x0D {
                    let at = |hi: usize| ram[hi] as f64 + ram[hi - 1] as f64 / 65536.0;
                    self.by_frame.entry(m.sys.timing.frame).or_default().push((at(0x63), at(0x65)));
                }
                self.busy = None;
            }
            _ => {}
        }
    }
}

fn run(m: &mut Machine, d: &mut Drivers, pads: &[(u64, u16)], frames: u64) {
    let mut k = 0;
    while m.sys.timing.frame < frames {
        while k < pads.len() && pads[k].0 <= m.sys.timing.frame {
            m.pads[0] = pads[k].1;
            k += 1;
        }
        m.step();
        d.watch(m);
    }
}

/// The script's `frames`, `tap`, `hold` and `release`, as the pad's value from each frame on.
fn pads(text: &str) -> Vec<(u64, u16)> {
    let (mut frame, mut pad, mut out) = (0u64, 0u16, vec![(0, 0)]);
    let button = |n: &str| 1u16 << BUTTONS.iter().position(|b| b.eq_ignore_ascii_case(n)).unwrap_or_else(|| panic!("no button {n}"));
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let w: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize, d: u64| w.get(i).and_then(|v| v.parse().ok()).unwrap_or(d);
        match w[0] {
            "frames" => frame += num(1, 1),
            "tap" => {
                out.push((frame, pad | button(w[1])));
                frame += num(2, 4);
                out.push((frame, pad));
            }
            "hold" => {
                pad |= button(w[1]);
                out.push((frame, pad));
            }
            "release" => {
                pad &= !button(w[1]);
                out.push((frame, pad));
            }
            v => panic!("dsp_cars takes frames, tap, hold and release, not {v}"),
        }
    }
    out
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let frames: u64 = a.get(2).and_then(|f| f.parse().ok()).unwrap_or(7200);
    let every: u64 = a.get(4).and_then(|f| f.parse().ok()).unwrap_or(300);
    let script = a.get(3).filter(|s| s.as_str() != "-").map(|p| pads(&std::fs::read_to_string(p).expect("the script"))).unwrap_or_else(|| vec![(0, 0)]);
    let image = std::fs::read(&a[1]).expect("the ROM");
    let mut lle = Machine::load_rom(&image).expect("an image");
    let mut hle = lle.clone();
    let (stem, _) = lle.sys.cart.nec_firmware().expect("a NEC DSP cartridge");
    let fw = venusrt::chips::dsporacle::firmware(stem).unwrap_or_else(|| std::process::exit(1));
    assert!(lle.attach_dsp(&fw));
    assert!(hle.attach_replacement_as(venusrt::chips::dsphle::Program::for_stem(stem).expect("a replacement")));
    let (mut x, mut y) = (Drivers::default(), Drivers::default());
    std::thread::scope(|s| {
        s.spawn(|| run(&mut lle, &mut x, &script, frames));
        s.spawn(|| run(&mut hle, &mut y, &script, frames));
    });
    let name = std::path::Path::new(&a[1]).file_stem().unwrap().to_string_lossy().to_string();
    let mut out = format!("{name}: drivers by 05h, image against replacement, {frames} frames, windows of {every}\n");
    let mut first = None;
    let mut window: BTreeMap<u64, (Vec<f64>, u64, u64)> = BTreeMap::new();
    for (f, a) in &x.by_frame {
        let w = window.entry(f / every * every).or_default();
        match y.by_frame.get(f) {
            Some(b) => {
                w.2 += a.len().abs_diff(b.len()) as u64;
                for (p, q) in a.iter().zip(b) {
                    let dist = (p.0 - q.0).abs().max((p.1 - q.1).abs());
                    if dist > 0.0 && first.is_none() {
                        first = Some(*f);
                    }
                    w.0.push(dist);
                }
            }
            None => w.1 += a.len() as u64,
        }
    }
    let _ = writeln!(out, "  05h on the image {}, on the replacement {}; the first driver placed differently in frame {}",
        x.by_frame.values().map(Vec::len).sum::<usize>(), y.by_frame.values().map(Vec::len).sum::<usize>(), first.map_or("none".to_string(), |f| f.to_string()));
    for (start, (mut d, missing, count_differs)) in window {
        d.sort_by(f64::total_cmp);
        let q = |v: f64| d.get(((d.len().max(1) - 1) as f64 * v) as usize).copied().unwrap_or(0.0);
        let _ = writeln!(out, "  frames {start}-{}: {} compared, {} alike, within 1 unit {}, within 16 {}; median {:.2}, 95th percentile {:.1}, largest {:.1}; frames without 05h on the replacement {missing}, counts differing by {count_differs}",
            start + every - 1, d.len(), d.iter().filter(|&&v| v == 0.0).count(), d.iter().filter(|&&v| v <= 1.0).count(), d.iter().filter(|&&v| v <= 16.0).count(), q(0.5), q(0.95), q(1.0));
    }
    let dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache/emusen/probe/venusrt/dsp-hle");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("cars-{name}.txt")), &out).unwrap();
    print!("{out}");
}
