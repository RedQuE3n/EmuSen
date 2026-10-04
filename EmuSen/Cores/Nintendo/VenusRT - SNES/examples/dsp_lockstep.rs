//! The whole-game oracle (VenusRT_DspHle.md §4.3): one ROM on two machines, the image's chip on one and VenusRT's
//! replacement on the other, frame by frame under the same pad. `dsp_lockstep <rom> <frames> [script]`, the image from
//! EMUSEN_VENUSRT_FIRMWARE; no battery file. Each frame compares the picture, WRAM, VRAM, CGRAM, OAM, the APU's RAM, the
//! master clock and the whole state with the chip's group left out; the report goes to stdout and to
//! ~/.cache/emusen/probe/venusrt/dsp-hle/lockstep-<rom>.txt.
//!
//! LOCKSTEP_AS=<stem> runs both machines as that program instead of the cartridge's.
//! LOCKSTEP_OTHER=<stem> puts another image in the replacement's place, so two programs can be run against each other;
//! LOCKSTEP_OTHER_CLOCK=<per mille> runs its clock faster or slower, so that only its timing differs.
//!
//! The script takes dsp_trace's verbs (`frames`, `tap`, `hold`, `release`, `tapuntil BTN wram ADDR HEX [cap] [every]`,
//! `tapuntil BTN chip N - [cap] [every]`), decided on the image's machine and given to both.
//!
//! The visual error: per frame, the share of pixels that differ, and for sprites alike in tile and attributes on both
//! machines, how far apart they are placed (VenusRT_Native.md §51). LOCKSTEP_SHOTS=f1,f2,... saves at those frames a PNG
//! of the image's picture, the replacement's and their differing pixels side by side, to the cache's shots/ folder.
use std::fmt::Write as _;
use std::hash::{Hash, Hasher};
use venusrt::chips::dsporacle::{Place, Places};
use venusrt::chips::necdsp::Transfer;
use venusrt::machine::Machine;

const BUTTONS: [&str; 12] = ["b", "y", "select", "start", "up", "down", "left", "right", "a", "x", "l", "r"];

fn button(name: &str) -> u16 {
    1 << BUTTONS.iter().position(|b| b.eq_ignore_ascii_case(name)).unwrap_or_else(|| panic!("no button {name}"))
}

fn hash<T: Hash + ?Sized>(v: &T) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

/// The state with the chip's group left out.
fn state_without_chip(m: &Machine) -> Vec<u8> {
    let mut c = m.clone();
    c.sys.cart.dsp = None;
    let mut out = vec![0u8; c.state_size()];
    c.save_state(&mut out).unwrap();
    out
}

struct Pair {
    lle: Machine,
    hle: Machine,
    places: Places,
    /// Commands below 40h, other than a test, that reached the image's chip; and every one, for the report.
    working: usize,
    commands: [u64; 256],
    frames: u64,
    first: Option<(u64, String)>,
    equal_pictures: u64,
    met_again: Option<u64>,
    at_idle: bool,
    dsp4: bool,
    /// Per frame, the share of pixels that differ; per sprite compared, the larger of its x and y distances.
    pixel_shares: Vec<f64>,
    sprite_errors: Vec<u32>,
    sprites_unmatched: u64,
    name: String,
    shots: Vec<u64>,
}

/// The on-screen sprites of an OAM image by index: (x, y, tile and attributes), from the low and high tables.
fn sprites(oam: &[u8]) -> Vec<Option<(i32, i32, u16)>> {
    (0..128)
        .map(|i| {
            let (lo, hi) = (&oam[i * 4..i * 4 + 4], oam[512 + i / 4] >> ((i % 4) * 2));
            let x = lo[0] as i32 - if hi & 1 != 0 { 256 } else { 0 };
            let y = lo[1] as i32;
            (y < 224 && x > -64 && x < 256).then_some((x, y, lo[2] as u16 | (lo[3] as u16) << 8))
        })
        .collect()
}

/// A PNG of RGBA rows, stored without compression.
fn png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let crc_table: Vec<u32> = (0..256u32).map(|n| (0..8).fold(n, |c, _| if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 })).collect();
    let crc = |data: &[u8]| !data.iter().fold(!0u32, |c, &b| crc_table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8));
    let mut raw = Vec::new();
    for row in rgba.chunks(width as usize * 4) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65535).collect();
    for (k, b) in blocks.iter().enumerate() {
        z.push((k + 1 == blocks.len()) as u8);
        z.extend((b.len() as u16).to_le_bytes());
        z.extend((!(b.len() as u16)).to_le_bytes());
        z.extend_from_slice(b);
    }
    let (a, b) = raw.iter().fold((1u32, 0u32), |(a, b), &x| ((a + x as u32) % 65521, (b + (a + x as u32) % 65521) % 65521));
    z.extend(((b << 16) | a).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut chunk = |kind: &[u8], data: &[u8]| {
        out.extend((data.len() as u32).to_be_bytes());
        let body = [kind, data].concat();
        out.extend_from_slice(&body);
        out.extend(crc(&body).to_be_bytes());
    };
    chunk(b"IHDR", &[&width.to_be_bytes()[..], &height.to_be_bytes()[..], &[8, 6, 0, 0, 0]].concat());
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    out
}

impl Pair {
    /// The first instruction after which the two S-CPUs differ, in a frame known to part (LOCKSTEP_AT).
    fn locate(&self) {
        let (mut a, mut b) = (self.lle.clone(), self.hle.clone());
        b.pads = a.pads;
        let mut recent: Vec<String> = Vec::new();
        let frame = a.sys.timing.frame;
        while a.sys.timing.frame == frame {
            a.step();
            b.step();
            let key = |m: &Machine| (m.cpu.pbr, m.cpu.pc, m.cpu.a, m.cpu.x, m.cpu.y, m.cpu.p, m.sys.timing.clock);
            recent.push(format!("{:02X}:{:04X} a={:04X} x={:04X} y={:04X} clock {}", a.cpu.pbr, a.cpu.pc, a.cpu.a, a.cpu.x, a.cpu.y, a.sys.timing.clock));
            if recent.len() > 12 {
                recent.remove(0);
            }
            if key(&a) != key(&b) {
                eprintln!("first difference in frame {frame}; the image's machine before it:\n  {}", recent.join("\n  "));
                eprintln!("  replacement: {:02X}:{:04X} a={:04X} x={:04X} y={:04X} clock {}", b.cpu.pbr, b.cpu.pc, b.cpu.a, b.cpu.x, b.cpu.y, b.sys.timing.clock);
                if let Some((venusrt::chips::dspengine::DspEngine::Hle(h), _)) = &b.sys.cart.dsp {
                    eprintln!("  replacement's chip: {}", h.d2_describe());
                }
                if let Some((venusrt::chips::dspengine::DspEngine::Lle(d), _)) = &a.sys.cart.dsp {
                    eprintln!("  image's chip: sr {:04X} dr {:04X} at {}", d.sr, d.dr, d.cycles);
                }
                return;
            }
        }
        eprintln!("no S-CPU difference in frame {frame}");
    }

    fn frame(&mut self) {
        if std::env::var("LOCKSTEP_AT").ok().and_then(|v| v.parse().ok()) == Some(self.lle.sys.timing.frame) {
            self.locate();
        }
        self.hle.pads = self.lle.pads;
        self.lle.run_frame();
        self.hle.run_frame();
        self.count();
        self.frames += 1;
        let (a, b) = (&self.lle, &self.hle);
        let picture = hash(a.sys.ppu.picture()) == hash(b.sys.ppu.picture());
        self.equal_pictures += picture as u64;
        let (pa, pb) = (a.sys.ppu.picture(), b.sys.ppu.picture());
        let differing = pa.chunks(4).zip(pb.chunks(4)).filter(|(x, y)| x != y).count();
        self.pixel_shares.push(differing as f64 / (pa.len() / 4).max(1) as f64);
        if !picture {
            for (x, y) in sprites(&a.sys.ppu.oam).into_iter().zip(sprites(&b.sys.ppu.oam)) {
                match (x, y) {
                    (Some(x), Some(y)) if x.2 == y.2 => self.sprite_errors.push((x.0 - y.0).unsigned_abs().max((x.1 - y.1).unsigned_abs())),
                    (None, None) => {}
                    _ => self.sprites_unmatched += 1,
                }
            }
        }
        if self.shots.contains(&a.sys.timing.frame) {
            let (fw, fh) = (a.sys.ppu.frame_width as usize, a.sys.ppu.frame_height as usize);
            let mut wide = vec![0u8; fw * 3 * fh * 4];
            for row in 0..fh {
                for col in 0..fw {
                    let i = (row * fw + col) * 4;
                    let mark = if pa[i..i + 4] == pb[i..i + 4] { [0, 0, 0, 255] } else { [255, 255, 255, 255] };
                    for (k, px) in [&pa[i..i + 4], &pb[i..i + 4], &mark[..]].into_iter().enumerate() {
                        let o = (row * fw * 3 + k * fw + col) * 4;
                        wide[o..o + 3].copy_from_slice(&px[..3]);
                        wide[o + 3] = 255;
                    }
                }
            }
            let dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache/emusen/probe/venusrt/dsp-hle/shots");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("{}-f{}.png", self.name, a.sys.timing.frame)), png(fw as u32 * 3, fh as u32, &wide)).unwrap();
        }
        let spaces = [
            ("picture", picture),
            ("WRAM", a.sys.wram == b.sys.wram),
            ("VRAM", a.sys.ppu.vram == b.sys.ppu.vram),
            ("CGRAM", a.sys.ppu.cgram == b.sys.ppu.cgram),
            ("OAM", a.sys.ppu.oam == b.sys.ppu.oam),
            ("ARAM", a.sys.apu.ram == b.sys.apu.ram),
            ("clock", a.sys.timing.clock == b.sys.timing.clock),
            ("DSPRAM", a.sys.cart.dsp.as_ref().is_none_or(|(d, _)| !d.st()) || a.sys.cart.dsp.as_ref().map(|(d, _)| d.ram()) == b.sys.cart.dsp.as_ref().map(|(d, _)| d.ram())),
        ];
        let differ: Vec<&str> = spaces.iter().filter(|s| !s.1).map(|s| s.0).collect();
        let state = differ.is_empty() && state_without_chip(a) == state_without_chip(b);
        if self.first.is_none() && (!differ.is_empty() || !state) {
            let what = if differ.is_empty() { "state".to_string() } else { differ.join(", ") };
            self.first = Some((self.lle.sys.timing.frame, what));
        } else if self.first.is_some() && self.met_again.is_none() && state {
            self.met_again = Some(self.lle.sys.timing.frame);
        }
    }

    /// Counts each command as the S-CPU's first write after the chip's idle edge, which every DR chip makes.
    fn count(&mut self) {
        let Some(dsp) = self.lle.sys.cart.dsp.as_mut().and_then(|(d, _)| d.lle_mut()) else { return };
        let log = std::mem::take(dsp.transfers.as_mut().unwrap());
        for (_, t) in log {
            match t {
                Transfer::HostWrite(c) => {
                    self.places.host_wrote();
                    if std::mem::take(&mut self.at_idle) {
                        self.commands[c as usize] += 1;
                        let test = if self.dsp4 { matches!(c, 0x13 | 0x14) } else { c & 0x0F == 0x0F || c & 0x0F == 0x0E };
                        if c < 0x40 && !test {
                            self.working += 1;
                        }
                    }
                }
                Transfer::ChipRead { pc } => {
                    self.at_idle = self.places.edge(true, pc) == Place::Idle;
                }
                Transfer::ChipWrite { pc, .. } => {
                    self.at_idle = self.places.edge(false, pc) == Place::Idle;
                }
                _ => {}
            }
        }
    }
}

fn script(p: &mut Pair, text: &str, frames: u64) {
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let w: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize, d: u64| w.get(i).and_then(|v| v.parse().ok()).unwrap_or(d);
        match w[0] {
            "frames" => (0..num(1, 1)).for_each(|_| p.frame()),
            "tap" => {
                p.lle.pads[0] |= button(w[1]);
                (0..num(2, 4)).for_each(|_| p.frame());
                p.lle.pads[0] &= !button(w[1]);
            }
            "hold" => p.lle.pads[0] |= button(w[1]),
            "release" => p.lle.pads[0] &= !button(w[1]),
            "shot" => {
                let pic = p.hle.sys.ppu.picture();
                let (fw, fh) = (p.hle.sys.ppu.frame_width, p.hle.sys.ppu.frame_height);
                std::fs::write(w[1], [&(fw as u32).to_le_bytes()[..], &(fh as u32).to_le_bytes()[..], pic].concat()).unwrap();
                let pic = p.lle.sys.ppu.picture();
                std::fs::write(format!("{}.image", w[1]), [&(fw as u32).to_le_bytes()[..], &(fh as u32).to_le_bytes()[..], pic].concat()).unwrap();
            }
            "tapuntil" => {
                let (b, cap, every) = (button(w[1]), num(5, 3600), num(6, 20).max(5));
                let target: Vec<u8> = if w[2] == "wram" { (0..w[4].len() / 2).map(|i| u8::from_str_radix(&w[4][i * 2..i * 2 + 2], 16).unwrap()).collect() } else { Vec::new() };
                let addr = if w[2] == "wram" { usize::from_str_radix(w[3], 16).unwrap() } else { 0 };
                let wanted = if w[2] == "chip" { p.working + w[3].parse::<usize>().unwrap() } else { 0 };
                let reached = |p: &Pair| if w[2] == "chip" { p.working >= wanted } else { p.lle.sys.wram[addr..addr + target.len()] == target[..] };
                let mut n = 0;
                while !reached(p) && n < cap && p.lle.sys.timing.frame < frames {
                    match n % every {
                        0 => p.lle.pads[0] |= b,
                        4 => p.lle.pads[0] &= !b,
                        _ => {}
                    }
                    p.frame();
                    n += 1;
                }
                p.lle.pads[0] &= !b;
                eprintln!("tapuntil {} {}: {} at frame {}", w[1], w[2], if reached(p) { "reached" } else { "NOT reached" }, p.lle.sys.timing.frame);
            }
            v => panic!("unknown verb {v}"),
        }
        if p.lle.sys.timing.frame >= frames {
            return;
        }
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let frames: u64 = a.get(2).and_then(|f| f.parse().ok()).unwrap_or(3600);
    let image = std::fs::read(&a[1]).expect("the ROM");
    let mut lle = Machine::load_rom(&image).expect("an image");
    let mut hle = lle.clone();
    let (stem, _) = lle.sys.cart.nec_firmware().expect("a NEC DSP cartridge");
    // LOCKSTEP_AS=<stem> runs both engines as another program of the same slot, the DSP-1B's in Pilotwings for one.
    let stem: &str = &std::env::var("LOCKSTEP_AS").unwrap_or(stem.to_string());
    let fw = venusrt::chips::dsporacle::firmware(stem).unwrap_or_else(|| std::process::exit(1));
    assert!(lle.attach_dsp(&fw));
    // LOCKSTEP_OTHER=<stem> runs a second image in the replacement's place, the DSP-1 against the DSP-1B for one.
    match std::env::var("LOCKSTEP_OTHER") {
        Ok(other) => {
            assert!(hle.attach_dsp(&venusrt::chips::dsporacle::firmware(&other).expect("the other image")));
            // LOCKSTEP_OTHER_CLOCK=<per mille> runs the other image's clock that much faster: its timing alone moved.
            if let Some(pm) = std::env::var("LOCKSTEP_OTHER_CLOCK").ok().and_then(|v| v.parse::<u64>().ok()) {
                let d = hle.sys.cart.dsp.as_mut().and_then(|(d, _)| d.lle_mut()).unwrap();
                d.ratio.0 = d.ratio.0 * pm / 1000;
            }
        }
        Err(_) => assert!(hle.attach_replacement_as(venusrt::chips::dsphle::Program::for_stem(stem).expect("a program with a replacement")), "no replacement for {stem}"),
    }
    lle.sys.cart.dsp.as_mut().and_then(|(d, _)| d.lle_mut()).unwrap().transfers = Some(Vec::new());
    let name = std::path::Path::new(&a[1]).file_stem().unwrap().to_string_lossy().to_string();
    let shots = std::env::var("LOCKSTEP_SHOTS").map(|v| v.split(',').filter_map(|f| f.trim().parse().ok()).collect()).unwrap_or_default();
    let mut p = Pair {
        lle,
        hle,
        places: Places::default(),
        working: 0,
        commands: [0; 256],
        frames: 0,
        first: None,
        equal_pictures: 0,
        met_again: None,
        at_idle: false,
        dsp4: stem == "dsp4",
        pixel_shares: Vec::new(),
        sprite_errors: Vec::new(),
        sprites_unmatched: 0,
        name: name.clone(),
        shots,
    };
    if let Some(path) = a.get(3) {
        script(&mut p, &std::fs::read_to_string(path).expect("the script"), frames);
    }
    while p.lle.sys.timing.frame < frames {
        p.frame();
    }
    let mut out = format!(
        "{name} ({stem}): {} frames; first parting {}; pictures equal in {} of {} frames; {}\n",
        p.frames,
        p.first.as_ref().map_or("none".to_string(), |(f, w)| format!("at frame {f} in {w}")),
        p.equal_pictures,
        p.frames,
        match (&p.first, p.met_again) {
            (None, _) => "identical throughout".to_string(),
            (Some(_), Some(f)) => format!("states equal again at frame {f}"),
            (Some(_), None) => "states not equal again".to_string(),
        }
    );
    let mut shares = p.pixel_shares.clone();
    shares.sort_by(f64::total_cmp);
    let at = |v: &[f64], q: f64| v.get(((v.len() as f64 - 1.0) * q) as usize).copied().unwrap_or(0.0) * 100.0;
    let _ = writeln!(
        out,
        "  pixels differing per frame: median {:.2}%, 95th percentile {:.2}%, largest {:.2}%; frames under 1% {}",
        at(&shares, 0.5),
        at(&shares, 0.95),
        at(&shares, 1.0),
        shares.iter().filter(|&&s| s < 0.01).count()
    );
    let mut e = p.sprite_errors.clone();
    e.sort();
    let within = |d: u32| e.iter().filter(|&&x| x <= d).count();
    let _ = writeln!(
        out,
        "  sprites alike on both, in frames that differ: {}; placed alike {}, within 1 {}, 2 {}, 4 {}, 8 {}; median {}, 95th percentile {}, largest {}; unmatched {}",
        e.len(),
        within(0),
        within(1),
        within(2),
        within(4),
        within(8),
        e.get(e.len() / 2).copied().unwrap_or(0),
        e.get(e.len().saturating_sub(1) * 95 / 100).copied().unwrap_or(0),
        e.last().copied().unwrap_or(0),
        p.sprites_unmatched
    );
    // Where each S-CPU spends one more frame: a game held in a wait loop shows few distinct addresses.
    for (name, m) in [("image", &mut p.lle), ("replacement", &mut p.hle)] {
        let mut pcs = std::collections::BTreeSet::new();
        let f = m.sys.timing.frame;
        while m.sys.timing.frame == f {
            m.step();
            pcs.insert(((m.cpu.pbr as u32) << 16) | m.cpu.pc as u32);
        }
        let _ = writeln!(out, "  {name}'s S-CPU in its last frame: {} distinct addresses, from {:06X}", pcs.len(), pcs.first().copied().unwrap_or(0));
    }
    let used: Vec<String> = (0..256).filter(|&c| p.commands[c] > 0).map(|c| format!("{c:02X} x{}", p.commands[c])).collect();
    let _ = writeln!(out, "  commands at the image's chip: {}", used.join(", "));
    let dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache/emusen/probe/venusrt/dsp-hle");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("lockstep-{name}.txt")), &out).unwrap();
    print!("{out}");
}
