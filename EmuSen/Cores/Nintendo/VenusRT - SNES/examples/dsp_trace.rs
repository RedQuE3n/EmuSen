//! The NEC DSP trace recorder (VenusRT_DspHle.md §4.2, VenusRT_Native.md §36): every command a game gives its chip,
//! with the frame, inputs, results and each transfer's latency, and the game's command histogram.
//! `dsp_trace <rom> <frames> [script]`; the image from EMUSEN_VENUSRT_FIRMWARE by the name the cartridge needs; the
//! trace to ~/.cache/emusen/probe/venusrt/dsp-hle/trace-<rom>.txt, counts on stdout. No battery file is read.
//!
//! A script holds the harness's verbs: `frames N`, `tap BTN [frames]`, `hold BTN`, `release BTN`, `shot FILE` (the
//! picture as RGBA), `tapuntil BTN wram ADDR HEXBYTES [cap] [every]`, and `tapuntil BTN chip N - [cap] [every]`,
//! which taps until N more commands below 40h, other than a test command, have reached the chip.
use std::collections::BTreeMap;
use std::fmt::Write as _;
use venusrt::chips::dsporacle::{Place, Places};
use venusrt::chips::necdsp::Transfer;
use venusrt::machine::Machine;

const BUTTONS: [&str; 12] = ["b", "y", "select", "start", "up", "down", "left", "right", "a", "x", "l", "r"];

#[derive(Clone, Debug, Default)]
struct Txn {
    frame: u64,
    command: u8,
    /// Each transfer: 'i' or 'o', its value, its byte count, the chip cycles from the S-CPU's last access to RQM's rise.
    steps: Vec<(char, u16, u8, u64)>,
    /// Bytes the S-CPU moved against the chip's direction (a read where it asked for a write, or the reverse), and
    /// bytes beyond a word.
    against: u32,
    over: u32,
    command_bytes: u8,
    end: char,
}

#[derive(Default)]
struct Recorder {
    places: Places,
    open: Option<Txn>,
    /// The cycle of the S-CPU's last access and of the last rise.
    last_host: u64,
    done: Vec<Txn>,
    idle_reads: u64,
    /// ST010: a mailbox command waiting for the chip to clear its busy bit, from the cycle it was set.
    mail: Option<(u8, u64, Vec<u16>)>,
    mail_done: Vec<(u64, u8, u64, usize)>,
}

impl Recorder {
    fn close(&mut self, end: char) {
        if let Some(mut t) = self.open.take() {
            t.end = end;
            self.done.push(t);
        }
    }

    fn feed(&mut self, frame: u64, at: u64, t: Transfer) {
        match t {
            Transfer::ChipRead { pc } | Transfer::ChipWrite { pc, .. } => {
                let read = matches!(t, Transfer::ChipRead { .. });
                match self.places.edge(read, pc) {
                    Place::Idle => {
                        self.close('.');
                        return;
                    }
                    Place::Command if self.open.as_ref().is_some_and(|o| !o.steps.is_empty()) => {
                        let mut o = self.open.take().unwrap();
                        let next = o.steps.pop().unwrap().1 as u8;
                        o.end = '-';
                        self.done.push(o);
                        self.open = Some(Txn { frame, command: next, command_bytes: 1, ..Txn::default() });
                    }
                    _ => {}
                }
                if let Some(o) = self.open.as_mut() {
                    o.steps.push((if read { 'i' } else { 'o' }, 0, 0, (at + 1).saturating_sub(self.last_host)));
                }
            }
            Transfer::HostWrite(v) | Transfer::HostRead(v) => {
                let write = matches!(t, Transfer::HostWrite(_));
                if write {
                    self.places.host_wrote();
                }
                self.last_host = at;
                match self.open.as_mut() {
                    None if write => self.open = Some(Txn { frame, command: v, command_bytes: 1, ..Txn::default() }),
                    None => self.idle_reads += 1,
                    Some(o) => match o.steps.last_mut() {
                        None if write && o.command_bytes == 1 => o.command_bytes = 2,
                        None => o.over += 1,
                        Some(s) => {
                            if (s.0 == 'i') != write {
                                o.against += 1;
                            }
                            if s.2 >= 2 {
                                o.over += 1;
                            } else {
                                s.1 |= (v as u16) << (8 * s.2);
                                s.2 += 1;
                            }
                        }
                    },
                }
            }
            Transfer::HostRam(..) => {}
        }
    }
}

fn shape(t: &Txn) -> String {
    let mut s = String::new();
    let b: Vec<char> = t.steps.iter().map(|x| x.0).collect();
    let mut i = 0;
    while i < b.len() {
        let mut j = i;
        while j + 1 < b.len() && b[j + 1] == b[i] {
            j += 1;
        }
        let _ = if j > i { write!(s, "{}{}", b[i], j - i + 1) } else { write!(s, "{}", b[i]) };
        i = j + 1;
    }
    s.push(t.end);
    s
}

struct Run {
    m: Machine,
    rec: Recorder,
    st010: bool,
    stem: &'static str,
}

impl Run {
    fn frame(&mut self) {
        let f = self.m.sys.timing.frame;
        while self.m.sys.timing.frame == f {
            self.m.step();
            if self.st010 {
                self.mailbox(f);
            }
        }
        self.m.run_frame_end();
        self.drain(f);
    }

    fn drain(&mut self, frame: u64) {
        let Some((dsp, _)) = self.m.sys.cart.dsp.as_mut() else { return };
        let log = std::mem::take(dsp.transfers.as_mut().unwrap());
        for (at, t) in log {
            self.rec.feed(frame, at, t);
        }
    }

    fn mailbox(&mut self, frame: u64) {
        let Some((dsp, _)) = self.m.sys.cart.dsp.as_mut() else { return };
        let busy = dsp.ram[0x10] & 0x8000 != 0;
        match (&self.rec.mail, busy) {
            (None, true) => self.rec.mail = Some(((dsp.ram[0x10] & 0xFF) as u8, dsp.cycles, dsp.ram.to_vec())),
            (Some((c, at, before)), false) => {
                let changed = before.iter().zip(dsp.ram.iter()).enumerate().filter(|(i, (a, b))| a != b && *i != 0x10).count();
                self.rec.mail_done.push((frame, *c, dsp.cycles - at, changed));
                self.rec.mail = None;
            }
            _ => {}
        }
    }

    /// Commands that compute: below 40h, and neither a memory test, a data ROM dump nor a version (fullsnes's 0Fh,
    /// 1Fh, 2Fh; the DSP-4's 13h, 14h).
    fn working(&self) -> usize {
        let dsp4 = self.stem == "dsp4";
        let test = |c: u8| if dsp4 { matches!(c, 0x13 | 0x14) } else { matches!(c & 0x0F, 0x0F) && c < 0x30 };
        self.rec.done.iter().filter(|t| t.command < 0x40 && !test(t.command) && t.end != '-').count() + self.rec.mail_done.len()
    }
}

trait FrameEnd {
    fn run_frame_end(&mut self);
}

impl FrameEnd for Machine {
    /// The frame's end as `run_frame` takes it: every processor caught up to the S-CPU, the samples dropped.
    fn run_frame_end(&mut self) {
        let clock = self.sys.timing.clock;
        self.sys.apu.run_to(clock);
        if let Some((dsp, _)) = self.sys.cart.dsp.as_mut() {
            dsp.run_to(clock);
        }
        self.sys.catch_up_sa1();
        self.sys.apu.out.clear();
    }
}

fn button(name: &str) -> u16 {
    1 << BUTTONS.iter().position(|b| b.eq_ignore_ascii_case(name)).unwrap_or_else(|| panic!("no button {name}"))
}

fn script(run: &mut Run, text: &str, frames: u64) {
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let p: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize, d: u64| p.get(i).and_then(|v| v.parse().ok()).unwrap_or(d);
        match p[0] {
            "frames" => (0..num(1, 1)).for_each(|_| run.frame()),
            "tap" => {
                run.m.pads[0] |= button(p[1]);
                (0..num(2, 4)).for_each(|_| run.frame());
                run.m.pads[0] &= !button(p[1]);
            }
            "hold" => run.m.pads[0] |= button(p[1]),
            "shot" => std::fs::write(p[1], run.m.sys.ppu.picture()).unwrap(),
            "release" => run.m.pads[0] &= !button(p[1]),
            "tapuntil" => {
                let (b, cap, every) = (button(p[1]), num(5, 3600), num(6, 20).max(5));
                let target: Vec<u8> = if p[2] == "wram" { (0..p[4].len() / 2).map(|i| u8::from_str_radix(&p[4][i * 2..i * 2 + 2], 16).unwrap()).collect() } else { Vec::new() };
                let addr = if p[2] == "wram" { usize::from_str_radix(p[3], 16).unwrap() } else { 0 };
                let wanted = if p[2] == "chip" { run.working() + p[3].parse::<usize>().unwrap() } else { 0 };
                let reached = |r: &Run| if p[2] == "chip" { r.working() >= wanted } else { r.m.sys.wram[addr..addr + target.len()] == target[..] };
                let mut n = 0;
                while !reached(run) && n < cap {
                    match n % every {
                        0 => run.m.pads[0] |= b,
                        4 => run.m.pads[0] &= !b,
                        _ => {}
                    }
                    run.frame();
                    n += 1;
                }
                run.m.pads[0] &= !b;
                eprintln!("tapuntil {} {}: {} after {n} frames (frame {})", p[1], p[2], if reached(run) { "reached" } else { "NOT reached" }, run.m.sys.timing.frame);
            }
            v => panic!("unknown verb {v}"),
        }
        if run.m.sys.timing.frame >= frames {
            return;
        }
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let frames: u64 = a.get(2).and_then(|f| f.parse().ok()).unwrap_or(3600);
    let image = std::fs::read(&a[1]).expect("the ROM");
    let mut m = Machine::load_rom(&image).expect("an image");
    let (stem, _) = m.sys.cart.nec_firmware().expect("a NEC DSP cartridge");
    let fw = venusrt::chips::dsporacle::firmware(stem).unwrap_or_else(|| std::process::exit(1));
    assert!(m.attach_dsp(&fw));
    m.sys.cart.dsp.as_mut().unwrap().0.transfers = Some(Vec::new());
    let mut run = Run { m, rec: Recorder::default(), st010: stem == "st010", stem };
    if let Some(path) = a.get(3) {
        script(&mut run, &std::fs::read_to_string(path).expect("the script"), frames);
    }
    while run.m.sys.timing.frame < frames {
        run.frame();
    }
    run.rec.close('+');
    let name = std::path::Path::new(&a[1]).file_stem().unwrap().to_string_lossy().to_string();
    let mut out = String::new();
    let _ = writeln!(out, "{name} ({stem}), {frames} frames");
    let mut hist: BTreeMap<u8, (u64, BTreeMap<String, u64>, u64, u64, u64)> = BTreeMap::new();
    let mut first = None;
    for t in &run.rec.done {
        first.get_or_insert(t.frame);
        let e = hist.entry(t.command).or_default();
        e.0 += 1;
        *e.1.entry(shape(t)).or_default() += 1;
        e.2 += t.steps.len() as u64;
        e.3 += t.against as u64;
        e.4 += t.over as u64;
        let _ = writeln!(
            out,
            "{} {:02X} {} in {:04X?} out {:04X?} lat {:?} against {} over {}",
            t.frame,
            t.command,
            shape(t),
            t.steps.iter().filter(|s| s.0 == 'i').map(|s| s.1).collect::<Vec<_>>(),
            t.steps.iter().filter(|s| s.0 == 'o').map(|s| s.1).collect::<Vec<_>>(),
            t.steps.iter().map(|s| s.3).collect::<Vec<_>>(),
            t.against,
            t.over
        );
    }
    let mut mhist: BTreeMap<u8, (u64, std::collections::BTreeSet<u64>, usize)> = BTreeMap::new();
    for &(f, c, lat, changed) in &run.rec.mail_done {
        first.get_or_insert(f);
        let e = mhist.entry(c).or_default();
        e.0 += 1;
        e.1.insert(lat);
        e.2 = e.2.max(changed);
        let _ = writeln!(out, "{f} mailbox {c:02X} cycles {lat} words {changed}");
    }
    let total: u64 = hist.values().map(|e| e.0).sum::<u64>() + mhist.values().map(|e| e.0).sum::<u64>();
    let transfers: u64 = hist.values().map(|e| e.2).sum();
    let mut summary = format!("{name} ({stem}): {total} commands over {frames} frames, the first at frame {first:?}; {transfers} DR transfers; {} reads of the idle word\n", run.rec.idle_reads);
    for (c, (n, shapes, tr, against, over)) in &hist {
        let top: Vec<String> = {
            let mut v: Vec<(&String, &u64)> = shapes.iter().collect();
            v.sort_by(|x, y| y.1.cmp(x.1));
            v.iter().take(3).map(|(s, k)| format!("{s} x{k}")).collect()
        };
        let _ = writeln!(
            summary,
            "  {c:02X}: {n} ({:.1}% of commands, {:.1}% of transfers){}{} shapes {}",
            100.0 * *n as f64 / total.max(1) as f64,
            100.0 * *tr as f64 / transfers.max(1) as f64,
            if *against > 0 { format!(", {against} bytes against the chip's direction") } else { String::new() },
            if *over > 0 { format!(", {over} bytes past a word") } else { String::new() },
            top.join(", ")
        );
    }
    for (c, (n, lats, changed)) in &mhist {
        let _ = writeln!(summary, "  mailbox {c:02X}: {n} ({:.1}%), {}..{} cycles, up to {changed} RAM words", 100.0 * *n as f64 / total.max(1) as f64, lats.first().unwrap(), lats.last().unwrap());
    }
    out.push_str(&summary);
    let dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache/emusen/probe/venusrt/dsp-hle");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("trace-{name}.txt")), out).unwrap();
    print!("{summary}");
}
