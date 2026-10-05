//! Grades VenusRT's ST010 replacement against the image through its mailbox (VenusRT_Native.md §43):
//! `dsp_stgrade <command> <cases> [threads]`. Each case seeds the command's input words, waits 0 to 8 cycles, and
//! compares the cycles to the busy bit's clearing and the whole RAM. Counts and the first mismatches' words go to stdout.
use venusrt::chips::dsphle::Program;
use venusrt::chips::dsporacle::*;
use venusrt::chips::necdsp::Port;

/// The words a case sets, by command: seeded, with the count of 02h held to 0-20.
fn inputs(command: u8, p: &mut Pcg) -> Vec<(usize, u16)> {
    let w = |p: &mut Pcg| match p.next() % 4 {
        0 => p.word() & 0xFF,
        1 => (p.word() & 0xFF).wrapping_neg(),
        2 => [0, 0x7FFF, 0x8000, 0xFFFF, 1][p.next() as usize % 5],
        _ => p.word(),
    };
    match command & 15 {
        0x02 | 0x0A => {
            let n = p.next() % 21;
            let mut v = vec![(0x12, n as u16)];
            v.extend((0..16).map(|k| (0x20 + k, if p.next() % 2 == 0 { p.word() & 7 } else { p.word() })));
            v.extend((0..16).map(|k| (0x40 + k, p.word())));
            v
        }
        0x05 | 0x0D => {
            // A driver near its waypoint half the time, so that the gate and the turn are both reached (§54).
            let mut v: Vec<(usize, u16)> = (0x60..=0x70).filter(|k| !(0x67..=0x69).contains(k)).map(|k| (k, w(p))).collect();
            if p.next() % 2 == 0 {
                let near = |p: &mut Pcg, at: u16| at.wrapping_add((p.next() % 400) as u16).wrapping_sub(200);
                v[0].1 = near(p, v[3].1);
                v[1].1 = near(p, v[5].1);
            }
            if p.next() % 4 == 0 {
                v[6].1 &= 0xFF80;
            }
            v
        }
        _ => (0..3).map(|k| (k, w(p))).collect(),
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let command = u8::from_str_radix(&a[1], 16).unwrap();
    let cases: u64 = a.get(2).and_then(|c| c.parse().ok()).unwrap_or(1 << 16);
    let threads: u64 = a.get(3).and_then(|c| c.parse().ok()).unwrap_or(1);
    let image = firmware("st010").unwrap_or_else(|| std::process::exit(1));
    let handles: Vec<_> = (0..threads)
        .map(|th| {
            let image = image.clone();
            std::thread::spawn(move || {
                let mut lle = idle_chip("st010", &image, &mut Host::steady(), 0).unwrap();
                let mut hle = Hle::new(Program::St010);
                st_ready(&mut hle, &mut Host::steady()).unwrap();
                let mut p = Pcg::new(0x5701_0000 + th + 0x100 * command as u64);
                let (mut latency, mut ram, mut first, mut worst, mut hangs, mut alike) = (0u64, 0u64, Vec::new(), 0i32, 0u64, 0u64);
                for _ in 0..cases / threads {
                    let set = inputs(command, &mut p);
                    for _ in 0..p.next() % 9 {
                        lle.tick();
                        hle.tick();
                    }
                    let host = || Host { cap: 200_000, ..Host::steady() };
                    let (x, y) = (mailbox(&mut lle, &mut host(), command, &set), mailbox(&mut hle, &mut host(), command, &set));
                    latency += (x.latency != y.latency) as u64;
                    let differ: Vec<usize> = (0..x.ram.len()).filter(|&w| x.ram[w] != y.ram[w]).collect();
                    ram += !differ.is_empty() as u64;
                    worst = differ.iter().map(|&w| (x.ram[w] as i16 as i32 - y.ram[w] as i16 as i32).abs()).fold(worst, i32::max);
                    if (x.latency != y.latency || (!differ.is_empty() && std::env::var_os("STGRADE_LATENCY").is_none())) && first.len() < 6 {
                        first.push(format!("inputs {:04X?} latency {:?}/{:?} words {}", set.iter().take(6).collect::<Vec<_>>(), x.latency, y.latency,
                            differ.iter().take(6).map(|&w| format!("{w:03X}:{:04X}/{:04X}", x.ram[w], y.ram[w])).collect::<Vec<_>>().join(" ")));
                    }
                    alike += (!differ.is_empty() && x.ram[8] == y.ram[8]) as u64;
                    // The next case starts from the image's RAM on both engines.
                    for &w in differ.iter().filter(|&&w| w != 0x10) {
                        hle.write(Port::Ram(w * 2), x.ram[w] as u8);
                        hle.write(Port::Ram(w * 2 + 1), (x.ram[w] >> 8) as u8);
                    }
                    if x.latency.is_none() || y.latency.is_none() {
                        // A command that never completes (§54.1): counted, and both chips started afresh.
                        hangs += (x.latency.is_none() && y.latency.is_none()) as u64;
                        lle = idle_chip("st010", &image, &mut Host::steady(), 0).unwrap();
                        hle = Hle::new(Program::St010);
                        st_ready(&mut hle, &mut Host::steady()).unwrap();
                    }
                }
                (latency, ram, first, worst, hangs, alike)
            })
        })
        .collect();
    let (mut latency, mut ram, mut first, mut worst, mut hangs, mut alike) = (0, 0, Vec::new(), 0, 0, 0);
    for h in handles {
        let (l, r, f, w, n, b) = h.join().unwrap();
        hangs += n;
        alike += b;
        worst = worst.max(w);
        latency += l;
        ram += r;
        first.extend(f);
    }
    println!("st010 {command:02X}: {cases} cases; latency differs {latency}, RAM differs {ram}, largest word difference {worst}; never completing on both {hangs}; RAM differing with word 8 alike {alike}");
    for f in first.iter().take(8) {
        println!("  {f}");
    }
}
