//! The port driver and command oracle for the NEC DSPs (VenusRT_DspHle.md §4, VenusRT_Native.md §37): a chip spoken
//! to only through DR, SR, the ST01x's RAM and its clock, as the S-CPU speaks to it, with every transfer's value, SR
//! and latency recorded. Expected values are never stored: they come from running the low-level path at test time.

use super::necdsp::{NecDsp, Port, Transfer};

/// A chip's edge on DR: it took what DR held and asks for the next write (`write` None), or it wrote DR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub write: Option<u16>,
    pub place: Place,
}

/// Where in its program the chip made an edge, told apart by the program counter alone: where it waits for a
/// command, where it has just taken a command byte, or elsewhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Idle,
    Command,
    Other,
}

/// What the driver needs of an engine: a clock, the ports, and the edge each cycle made.
pub trait Chip {
    /// One chip cycle, and the last DR edge it made.
    fn tick(&mut self) -> Option<Edge>;
    fn read(&mut self, port: Port) -> u8;
    fn write(&mut self, port: Port, value: u8);
    /// SR's high byte, as the S-CPU reads it.
    fn status(&self) -> u8;
    /// The ST01x's RAM as words; empty on a DSP-n.
    fn ram(&self) -> &[u16];
}

/// Learns a chip's idle place, its first edge after power-on, and its command place, the first read edge after the
/// S-CPU's first write at idle.
#[derive(Clone, Debug, Default)]
pub struct Places {
    idle: Option<u16>,
    command: Option<u16>,
    at_idle: bool,
    wrote: bool,
    learned: bool,
}

impl Places {
    pub fn edge(&mut self, read: bool, pc: u16) -> Place {
        let place = if *self.idle.get_or_insert(pc) == pc {
            Place::Idle
        } else if self.command == Some(pc) {
            Place::Command
        } else if !self.learned && self.at_idle && self.wrote && read {
            self.command = Some(pc);
            Place::Command
        } else {
            Place::Other
        };
        if self.at_idle && self.wrote {
            self.at_idle = false;
            self.learned = true;
        }
        if place == Place::Idle {
            self.at_idle = true;
            self.wrote = false;
        }
        place
    }

    pub fn host_wrote(&mut self) {
        self.wrote = true;
    }
}

/// The low-level path: the firmware's program run by `NecDsp`.
#[derive(Clone)]
pub struct Lle {
    pub dsp: NecDsp,
    pub places: Places,
}

impl Lle {
    pub fn new(image: &[u8]) -> Option<Lle> {
        let mut dsp = NecDsp::from_firmware(image)?;
        dsp.transfers = Some(Vec::new());
        Some(Lle { dsp, places: Places::default() })
    }
}

impl Chip for Lle {
    fn tick(&mut self) -> Option<Edge> {
        self.dsp.step();
        self.dsp.cycles += 1;
        let log = self.dsp.transfers.as_mut()?;
        let mut edge = None;
        for &(_, t) in log.iter() {
            let (write, pc) = match t {
                Transfer::ChipRead { pc } => (None, pc),
                Transfer::ChipWrite { value, pc } => (Some(value), pc),
                _ => continue,
            };
            edge = Some(Edge { write, place: self.places.edge(write.is_none(), pc) });
        }
        log.clear();
        edge
    }

    fn read(&mut self, port: Port) -> u8 {
        self.dsp.host_read(port, true)
    }

    fn write(&mut self, port: Port, value: u8) {
        if port == Port::Dr {
            self.places.host_wrote();
        }
        self.dsp.host_write(port, value);
    }

    fn status(&self) -> u8 {
        (self.dsp.sr >> 8) as u8
    }

    fn ram(&self) -> &[u16] {
        if self.dsp.st { &self.dsp.ram } else { &[] }
    }
}

/// The replacement, graded through the same ports; its places are its own stages.
#[derive(Clone)]
pub struct Hle {
    pub dsp: super::dsphle::DspHle,
    rqm: bool,
}

impl Hle {
    pub fn new(program: super::dsphle::Program) -> Hle {
        Hle { dsp: super::dsphle::DspHle::new(program), rqm: false }
    }
}

impl Chip for Hle {
    fn tick(&mut self) -> Option<Edge> {
        let clock = ((self.dsp.cycles + 1) as u128 * self.dsp.ratio.1 as u128).div_ceil(self.dsp.ratio.0 as u128) as u64;
        let dr = self.dsp.dr;
        self.dsp.run_to(clock);
        let rose = self.dsp.sr & 0x8000 != 0 && (!self.rqm || self.dsp.dr != dr);
        self.rqm = self.dsp.sr & 0x8000 != 0;
        if !rose {
            return None;
        }
        let place = if self.dsp.idle() { Place::Idle } else if self.dsp.reading_command() { Place::Command } else { Place::Other };
        Some(Edge { write: (self.dsp.offering() || place == Place::Idle).then_some(self.dsp.dr), place })
    }

    fn read(&mut self, port: Port) -> u8 {
        let v = self.dsp.host_read(port, true);
        self.rqm = self.dsp.sr & 0x8000 != 0;
        v
    }

    fn write(&mut self, port: Port, value: u8) {
        self.dsp.host_write(port, value);
        self.rqm = self.dsp.sr & 0x8000 != 0;
    }

    fn status(&self) -> u8 {
        (self.dsp.sr >> 8) as u8
    }

    fn ram(&self) -> &[u16] {
        &self.dsp.ram
    }
}

/// PCG-XSH-RR 64/32 (O'Neill 2014), the oracle's seeded inputs.
#[derive(Clone, Debug)]
pub struct Pcg(u64);

impl Pcg {
    pub fn new(seed: u64) -> Pcg {
        let mut p = Pcg(0);
        p.next();
        p.0 = p.0.wrapping_add(seed);
        p.next();
        p
    }

    pub fn next(&mut self) -> u32 {
        let old = self.0;
        self.0 = old.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let x = (((old >> 18) ^ old) >> 27) as u32;
        x.rotate_right((old >> 59) as u32)
    }

    pub fn word(&mut self) -> u16 {
        (self.next() >> 16) as u16
    }

    /// Uniform in `lo..=hi`.
    pub fn within(&mut self, lo: u32, hi: u32) -> u32 {
        lo + self.next() % (hi - lo + 1)
    }
}

/// The quickest answer the driver models, in chip cycles to the first byte and between a word's bytes: about one
/// S-CPU bus cycle each (VenusRT_Native.md §37).
pub const FASTEST: (u32, u32) = (4, 3);

/// How the S-CPU side behaves: chip cycles from RQM rising to its first byte, between a word's two bytes, and the
/// limits after which a transaction is given up.
#[derive(Clone, Debug)]
pub struct Host {
    pub respond: (u32, u32),
    pub gap: (u32, u32),
    pub rng: Pcg,
    /// Chip cycles a phase may take before the chip counts as stalled.
    pub cap: u64,
    pub max_steps: usize,
    /// The word written when the chip asks for more inputs than the case gave.
    pub filler: u16,
}

impl Host {
    /// A poll loop's response, fixed: 8 cycles to the first byte, 3 between a word's bytes.
    pub fn steady() -> Host {
        Host { respond: (8, 8), gap: (3, 3), rng: Pcg::new(0), cap: 4_000_000, max_steps: 4096, filler: 0 }
    }

    /// A response drawn anew at each transfer, for the phase check.
    pub fn jittered(seed: u64) -> Host {
        Host { respond: (FASTEST.0, 29), gap: (FASTEST.1, 6), rng: Pcg::new(seed), ..Host::steady() }
    }

    fn draw(&mut self, range: (u32, u32)) -> u32 {
        if range.0 == range.1 { range.0 } else { self.rng.within(range.0, range.1) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dir {
    /// The S-CPU wrote DR.
    In,
    /// The S-CPU read DR.
    Out,
}

/// One transfer through DR as the S-CPU made it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Step {
    pub dir: Dir,
    pub value: u16,
    pub bytes: u8,
    /// Chip cycles from the S-CPU's completion of the previous transfer to RQM rising for this one, and from that
    /// rise to the S-CPU's completion of this one.
    pub latency: u32,
    pub answer: u32,
    /// SR's high byte at RQM's rise and at the transfer's first byte.
    pub sr_rise: u8,
    pub sr_access: u8,
    /// Edges the chip made after RQM rose and before the S-CPU answered.
    pub late_edges: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum End {
    /// Back where it waits for a command, with DR holding `value`.
    Idle { latency: u32, value: u16 },
    /// The command byte was passed over: the chip took the next byte as a command.
    Ignored,
    /// No edge within the host's cap.
    Stalled,
    /// More transfers than the host's limit.
    Capped,
}

/// A command and everything that passed through the ports for it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Transaction {
    pub command: u8,
    pub steps: Vec<Step>,
    pub end: End,
}

impl Transaction {
    pub fn inputs(&self) -> usize {
        self.steps.iter().filter(|s| s.dir == Dir::In).count()
    }

    pub fn outputs(&self) -> Vec<u16> {
        self.steps.iter().filter(|s| s.dir == Dir::Out).map(|s| s.value).collect()
    }

    pub fn latencies(&self) -> Vec<u32> {
        let mut l: Vec<u32> = self.steps.iter().map(|s| s.latency).collect();
        if let End::Idle { latency, .. } = self.end {
            l.push(latency);
        }
        l
    }

    /// The shape as `i` and `o` per transfer, the way the reports print it.
    pub fn shape(&self) -> String {
        let mut s: String = self.steps.iter().map(|t| if t.dir == Dir::In { 'i' } else { 'o' }).collect();
        s.push(match self.end {
            End::Idle { .. } => '.',
            End::Ignored => '-',
            End::Stalled => '!',
            End::Capped => '+',
        });
        s
    }

    /// Everything but the cycle counts.
    pub fn without_timing(&self) -> Transaction {
        let mut t = self.clone();
        for s in t.steps.iter_mut() {
            s.latency = 0;
            s.answer = 0;
            s.late_edges = 0;
            s.sr_rise = 0;
        }
        if let End::Idle { latency, .. } = &mut t.end {
            *latency = 0;
        }
        t
    }
}

/// How two records of the same case differ.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Difference {
    /// The sequence of reads and writes, or the end.
    pub shape: bool,
    pub values: bool,
    pub latency: bool,
    pub status: bool,
}

impl Difference {
    pub fn any(self) -> bool {
        self.shape || self.values || self.latency || self.status
    }
}

pub fn compare(a: &Transaction, b: &Transaction) -> Difference {
    let shape = a.shape() != b.shape();
    let values = a.steps.iter().zip(&b.steps).any(|(x, y)| x.value != y.value || x.bytes != y.bytes)
        || matches!((a.end, b.end), (End::Idle { value: x, .. }, End::Idle { value: y, .. }) if x != y);
    let latency = a.latencies() != b.latencies() || a.steps.iter().zip(&b.steps).any(|(x, y)| x.late_edges != y.late_edges);
    let status = a.steps.iter().zip(&b.steps).any(|(x, y)| x.sr_access != y.sr_access || x.sr_rise != y.sr_rise);
    Difference { shape, latency, values: values && !shape, status }
}

/// Runs until RQM rises or the cap, giving the cycles taken and the last edge; edges made with RQM already set count
/// as late.
fn wait_rise<C: Chip>(chip: &mut C, cap: u64) -> Option<(u32, Edge, u8)> {
    let mut last = None;
    for n in 1..=cap {
        let rqm = chip.status() & 0x80 != 0;
        if let Some(e) = chip.tick() {
            last = Some(e);
        }
        let sr = chip.status();
        if !rqm && sr & 0x80 != 0 {
            return Some((n as u32, last?, sr));
        }
    }
    None
}

/// Runs `n` cycles, giving the last edge made and how many.
fn run<C: Chip>(chip: &mut C, n: u32) -> (Option<Edge>, u8) {
    let (mut last, mut count) = (None, 0u8);
    for _ in 0..n {
        if let Some(e) = chip.tick() {
            last = Some(e);
            count = count.saturating_add(1);
        }
    }
    (last, count)
}

/// A word through DR, one byte in 8-bit mode (SR's DRC) or two, low first; the bytes and the cycles between them.
fn put<C: Chip>(chip: &mut C, host: &mut Host, value: u16) -> (u8, u32) {
    chip.write(Port::Dr, value as u8);
    if chip.status() & 0x14 == 0x10 {
        let gap = host.draw(host.gap);
        run(chip, gap);
        chip.write(Port::Dr, (value >> 8) as u8);
        return (2, gap);
    }
    (1, 0)
}

fn take<C: Chip>(chip: &mut C, host: &mut Host) -> (u16, u8, u32) {
    let low = chip.read(Port::Dr) as u16;
    if chip.status() & 0x14 == 0x10 {
        let gap = host.draw(host.gap);
        run(chip, gap);
        return (low | (chip.read(Port::Dr) as u16) << 8, 2, gap);
    }
    (low, 1, 0)
}

/// Power-on to the chip's first edge, the place it waits for a command; the cycles it took.
pub fn power_on<C: Chip>(chip: &mut C, host: &Host) -> Option<u32> {
    wait_rise(chip, host.cap).map(|(n, _, _)| n)
}

/// One command from idle: the command word, then a write of the next input wherever the chip asks for one and a
/// read wherever it wrote, until it is idle again.
pub fn transact<C: Chip>(chip: &mut C, host: &mut Host, command: u8, inputs: &[u16]) -> Transaction {
    let respond = host.draw(host.respond);
    run(chip, respond);
    put(chip, host, command as u16);
    let mut steps = Vec::new();
    let mut next = inputs.iter();
    loop {
        let Some((latency, edge, sr_rise)) = wait_rise(chip, host.cap) else {
            return Transaction { command, steps, end: End::Stalled };
        };
        let respond = host.draw(host.respond);
        let (late, late_edges) = run(chip, respond);
        let edge = late.unwrap_or(edge);
        if edge.place == Place::Idle {
            return Transaction { command, steps, end: End::Idle { latency, value: edge.write.unwrap_or(0) } };
        }
        if edge.place == Place::Command && !steps.is_empty() {
            return Transaction { command, steps, end: End::Ignored };
        }
        if steps.len() == host.max_steps {
            return Transaction { command, steps, end: End::Capped };
        }
        let sr_access = chip.status();
        let (dir, value, bytes, gap) = match edge.write {
            None => {
                let v = *next.next().unwrap_or(&host.filler);
                let (b, g) = put(chip, host, v);
                (Dir::In, v, b, g)
            }
            Some(_) => {
                let (v, b, g) = take(chip, host);
                (Dir::Out, v, b, g)
            }
        };
        steps.push(Step { dir, value, bytes, latency, answer: respond + gap, sr_rise, sr_access, late_edges });
    }
}

/// One phase's timing as a replacement models it: RQM rises `work` cycles after the previous rise or `notice` cycles
/// after the S-CPU completes the previous transfer, whichever is later. The first phase counts from the command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Phase {
    pub work: u32,
    pub notice: u32,
}

/// Each phase's work and notice from two runs, the S-CPU answering as fast as `FASTEST` and answering after every
/// phase's work is done; None when the two runs take different transfers. Work no longer than the fastest answer
/// plus the notice cannot be seen, and is given as 0.
pub fn timing<C: Chip + Clone>(idle: &C, host: &Host, command: u8, inputs: &[u16]) -> Option<Vec<Phase>> {
    let quick = Host { respond: (FASTEST.0, FASTEST.0), gap: (FASTEST.1, FASTEST.1), ..host.clone() };
    let fast = transact(&mut idle.clone(), &mut quick.clone(), command, inputs);
    let longest = fast.latencies().into_iter().max().unwrap_or(0) + FASTEST.0 + FASTEST.1 + 1;
    let late = transact(&mut idle.clone(), &mut Host { respond: (longest, longest), ..quick }, command, inputs);
    if fast.without_timing() != late.without_timing() || !matches!(fast.end, End::Idle { .. } | End::Ignored) {
        return None;
    }
    let answers: Vec<u32> = std::iter::once(0).chain(fast.steps.iter().map(|s| s.answer)).collect();
    Some(
        fast.latencies()
            .into_iter()
            .zip(late.latencies())
            .zip(answers)
            .map(|((l, notice), a)| Phase { work: if l > notice { l + a } else { 0 }, notice })
            .collect(),
    )
}

/// The latencies the model gives for a run whose S-CPU answered each rise after the cycles in `answers`.
pub fn predict(phases: &[Phase], answers: &[u32]) -> Vec<u32> {
    phases.iter().enumerate().map(|(k, p)| if k == 0 { p.notice } else { p.work.saturating_sub(answers[k - 1]).max(p.notice) }).collect()
}

/// The ST010's start: the word it writes to DR at power-on, read by the S-CPU, before its mailbox runs.
pub fn st_ready<C: Chip>(chip: &mut C, host: &mut Host) -> Option<u16> {
    wait_rise(chip, host.cap)?;
    let (v, _, _) = take(chip, host);
    run(chip, 64);
    Some(v)
}

/// An ST010 command through its RAM mailbox (fullsnes, "ST010 Commands"): the inputs stored, the command at byte
/// 0020h, bit 7 of byte 0021h set, then the cycles until the chip clears it, and the RAM after.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mailbox {
    pub command: u8,
    pub latency: Option<u32>,
    pub ram: Vec<u16>,
}

pub fn mailbox<C: Chip>(chip: &mut C, host: &mut Host, command: u8, inputs: &[(usize, u16)]) -> Mailbox {
    for &(word, v) in inputs {
        chip.write(Port::Ram(word * 2), v as u8);
        chip.write(Port::Ram(word * 2 + 1), (v >> 8) as u8);
    }
    chip.write(Port::Ram(0x20), command);
    let flags = chip.ram()[0x10] >> 8;
    chip.write(Port::Ram(0x21), flags as u8 | 0x80);
    let mut latency = None;
    for n in 1..=host.cap {
        chip.tick();
        if chip.ram()[0x10] & 0x8000 == 0 {
            latency = Some(n as u32);
            break;
        }
    }
    Mailbox { command, latency, ram: chip.ram().to_vec() }
}

/// The firmware images the oracle knows, by the file stem `Cartridge::nec_firmware` gives.
pub const CHIPS: [&str; 7] = ["dsp1", "dsp1b", "dsp2", "dsp3", "dsp4", "st010", "st011"];

/// A chip's image from the folder `EMUSEN_VENUSRT_FIRMWARE` names, as a program and data pair or whole, of the size
/// its processor takes; None, saying so, when there is none.
pub fn firmware(stem: &str) -> Option<Vec<u8>> {
    let Some(dir) = std::env::var_os("EMUSEN_VENUSRT_FIRMWARE") else {
        eprintln!("EMUSEN_VENUSRT_FIRMWARE unset, not run");
        return None;
    };
    let dir = std::path::PathBuf::from(dir);
    let size = if stem.starts_with("st") { 53_248 } else { 8_192 };
    let pair = std::fs::read(dir.join(format!("{stem}.program.rom")))
        .and_then(|mut p| std::fs::read(dir.join(format!("{stem}.data.rom"))).map(|d| {
            p.extend(d);
            p
        }));
    match pair.or_else(|_| std::fs::read(dir.join(format!("{stem}.rom")))) {
        Ok(image) if image.len() == size => Some(image),
        _ => {
            eprintln!("{stem}: no image of {size} bytes in {}, not run", dir.display());
            None
        }
    }
}

/// The chip at idle after power-on, the ST010, which answers through its mailbox, past its start word; then `offset`
/// cycles more before the S-CPU's first command.
pub fn idle_chip(stem: &str, image: &[u8], host: &mut Host, offset: u32) -> Option<Lle> {
    let mut chip = Lle::new(image)?;
    if stem == "st010" {
        st_ready(&mut chip, host)?;
    } else {
        power_on(&mut chip, host)?;
    }
    run(&mut chip, offset);
    Some(chip)
}

/// Every command byte from the same idle state, with the given inputs.
pub fn sweep(idle: &Lle, host: &mut Host, inputs: &[u16]) -> Vec<Transaction> {
    (0..=255u8).map(|c| transact(&mut idle.clone(), host, c, inputs)).collect()
}

/// Every command byte through the ST010's mailbox from the same idle state.
pub fn sweep_mailbox(idle: &Lle, host: &mut Host, inputs: &[(usize, u16)]) -> Vec<Mailbox> {
    (0..=255u8).map(|c| mailbox(&mut idle.clone(), host, c, inputs)).collect()
}

/// Command bytes that behave alike over every input set given: classes of two or more, each led by its lowest byte.
pub fn mirror_classes(runs: &[Vec<Transaction>]) -> Vec<Vec<u8>> {
    let key = |c: usize| -> Vec<Transaction> {
        runs.iter()
            .map(|r| {
                let mut t = r[c].clone();
                t.command = 0;
                t
            })
            .collect()
    };
    let mut classes: Vec<(Vec<Transaction>, Vec<u8>)> = Vec::new();
    for c in 0..256 {
        let k = key(c);
        match classes.iter_mut().find(|(x, _)| *x == k) {
            Some((_, members)) => members.push(c as u8),
            None => classes.push((k, vec![c as u8])),
        }
    }
    classes.into_iter().map(|(_, m)| m).filter(|m| m.len() > 1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chip(stem: &str) -> Option<Lle> {
        let image = firmware(stem)?;
        Some(idle_chip(stem, &image, &mut Host::steady(), 0).expect("the chip reaches idle"))
    }

    fn words(seed: u64, n: usize) -> Vec<u16> {
        let mut p = Pcg::new(seed);
        (0..n).map(|_| p.word()).collect()
    }

    fn mail_words(seed: u64) -> Vec<(usize, u16)> {
        let mut p = Pcg::new(seed);
        (0..0x80).map(|w| (w, p.word())).filter(|&(w, _)| w != 0x10).collect()
    }

    fn bounded() -> Host {
        Host { cap: 400_000, max_steps: 1100, ..Host::steady() }
    }

    // The driver alone, on a synthetic program: idle writes 80h, reads the command and one input, writes the input back.
    #[test]
    fn the_driver_follows_a_synthetic_programs_handshake() {
        let ld = |id: u32, dst: u32| 0xC0_0000 | id << 6 | dst;
        let mov = |src: u32, dst: u32| src << 4 | dst;
        let jrqm = |to: u32| 0x80_0000 | 0x0BE << 13 | to << 2;
        let jmp = |to: u32| 0x80_0000 | 0x100 << 13 | to << 2;
        let program = [ld(0x0400, 7), ld(0x80, 6), jrqm(2), mov(8, 1), ld(0, 7), jrqm(5), mov(9, 2), mov(2, 6), jrqm(8), jmp(0)];
        let mut image = vec![0u8; 8192];
        for (i, op) in program.iter().enumerate() {
            image[i * 3..i * 3 + 3].copy_from_slice(&op.to_le_bytes()[..3]);
        }
        let mut idle = Lle::new(&image).unwrap();
        let mut host = Host::steady();
        assert!(power_on(&mut idle, &host).is_some());
        let t = transact(&mut idle.clone(), &mut host, 0x2A, &[0xBEEF]);
        assert_eq!(t.shape(), "io.", "{t:?}");
        assert_eq!(t.outputs(), [0xBEEF]);
        assert!(matches!(t.end, End::Idle { value: 0x80, .. }));
        assert_eq!(t.steps[1].bytes, 2);
        let again = transact(&mut idle.clone(), &mut Host::jittered(7), 0x2A, &[0xBEEF]);
        assert_eq!(again.without_timing(), t.without_timing());
    }

    // The replacement's frame against the low-level path: the ROM version's values and transfers are fullsnes's and
    // agree; the timing is not yet modelled (step 3).
    #[test]
    fn the_replacements_rom_version_agrees_with_the_image() {
        for (stem, program) in [("dsp1", super::super::dsphle::Program::Dsp1), ("dsp1b", super::super::dsphle::Program::Dsp1b)] {
            let Some(lle) = chip(stem) else { return };
            let mut hle = Hle::new(program);
            power_on(&mut hle, &Host::steady()).expect("the replacement reaches idle");
            let (x, y) = (transact(&mut lle.clone(), &mut Host::steady(), 0x2F, &[]), transact(&mut hle.clone(), &mut Host::steady(), 0x2F, &[]));
            let d = compare(&x, &y);
            assert!(!d.shape && !d.values, "{stem}: {d:?}\n{x:?}\n{y:?}");
            let (x, y) = (transact(&mut lle.clone(), &mut Host::steady(), 0x41, &[0x2F]), transact(&mut hle.clone(), &mut Host::steady(), 0x41, &[0x2F]));
            assert!(x.end == End::Ignored && y.end == End::Ignored, "{stem}");
        }
    }

    // The latency model on its own: a phase's rise is the later of its work after the last rise and its notice after
    // the S-CPU's answer.
    #[test]
    fn the_timing_model_takes_the_later_of_work_and_notice() {
        let phases = [Phase { work: 0, notice: 2 }, Phase { work: 14, notice: 2 }, Phase { work: 28, notice: 4 }];
        assert_eq!(predict(&phases, &[7, 7]), [2, 7, 21]);
        assert_eq!(predict(&phases, &[20, 40]), [2, 2, 4]);
    }

    // fullsnes's ROM versions, its DR on completion, and SNESdev's Multiply formula: 4000h * 4000h * 2^-15 = 2000h.
    #[test]
    fn the_documented_answers_come_back_through_the_ports() {
        for (stem, command, version, after) in [("dsp1", 0x2F, 0x0100, 0x80), ("dsp1b", 0x2F, 0x0101, 0x80), ("dsp3", 0x2F, 0x0300, 0x80), ("dsp4", 0x14, 0x0400, 0xFFFF)] {
            let Some(idle) = chip(stem) else { return };
            let t = transact(&mut idle.clone(), &mut Host::steady(), command, &[]);
            assert_eq!(t.outputs(), [version], "{stem}");
            assert!(matches!(t.end, End::Idle { value, .. } if value == after), "{stem}: {:?}", t.end);
        }
        let Some(idle) = chip("dsp1b") else { return };
        let t = transact(&mut idle.clone(), &mut Host::steady(), 0x00, &[0x4000, 0x4000]);
        assert_eq!((t.inputs(), t.outputs()), (2, vec![0x2000]));
    }

    // fullsnes: ST010 command 00h sets RAM[0010h] to 0, read here as the chip's word 0010h, the mailbox, which it
    // clears with nothing else changed; 10h-FFh mirror 00h-0Fh and 09h-0Fh mirror 01h-07h.
    #[test]
    fn the_st010_mailbox_answers_and_mirrors_as_documented() {
        let Some(idle) = chip("st010") else { return };
        let mut host = Host::steady();
        let inputs = mail_words(0x5710);
        let m = mailbox(&mut idle.clone(), &mut host, 0x00, &inputs);
        assert!(m.latency.is_some());
        assert_eq!(m.ram[0x10], 0);
        assert!(inputs.iter().all(|&(w, v)| m.ram[w] == v), "00h changed a word beside the mailbox");
        let base = |c: u8| if c & 0x0F >= 9 { (c & 0x0F) - 8 } else { c & 0x0F };
        for seed in 0..3 {
            let runs = sweep_mailbox(&idle, &mut host, &mail_words(seed));
            for c in 0..=255u8 {
                assert!(runs[c as usize].latency.is_some(), "command {c:02X} never finished");
                let (x, y) = (&runs[c as usize], &runs[base(c) as usize]);
                assert!(x.latency == y.latency && x.ram == y.ram, "command {c:02X} against {:02X}", base(c));
            }
        }
    }

    // fullsnes: the DSP-2's 10h-FFh mirror 00h-0Fh, for each command it names, and the DSP-4's 20h-FFh mirror
    // 10h-1Fh; the inputs alike.
    #[test]
    fn the_documented_mirrors_answer_alike() {
        let named = [0x01, 0x03, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0D, 0x0F];
        for (stem, bytes, base) in [
            ("dsp2", (0..=255u8).filter(|c| named.contains(&(c & 0x0F))).collect::<Vec<_>>(), (|c: u8| c & 0x0F) as fn(u8) -> u8),
            ("dsp4", (0x20..=0xFFu8).collect(), |c: u8| 0x10 | (c & 0x0F)),
        ] {
            let Some(idle) = chip(stem) else { return };
            for seed in 0..3 {
                let runs = sweep(&idle, &mut bounded(), &words(seed, 64));
                for &c in &bytes {
                    let (mut x, y) = (runs[c as usize].clone(), &runs[base(c) as usize]);
                    x.command = y.command;
                    assert!(&x == y, "{stem} command {c:02X} against {:02X}: {:?}", base(c), compare(&x, y));
                }
            }
        }
    }

    // VenusRT_DspHle.md §4.1: every command byte answers alike from two power-on phases, the second 10,007 cycles
    // later; the ST010's mailbox only by its poll's phase, a constant shift under three cycles.
    #[test]
    fn every_command_answers_alike_from_two_phases() {
        for stem in CHIPS {
            let Some(image) = firmware(stem) else { return };
            let a = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
            let b = idle_chip(stem, &image, &mut Host::steady(), 10_007).unwrap();
            for seed in 0..2 {
                if stem == "st010" {
                    let (x, y) = (sweep_mailbox(&a, &mut Host::steady(), &mail_words(seed)), sweep_mailbox(&b, &mut Host::steady(), &mail_words(seed)));
                    let shifts: std::collections::BTreeSet<i64> = x.iter().zip(&y).map(|(p, q)| q.latency.unwrap() as i64 - p.latency.unwrap() as i64).collect();
                    assert!(x.iter().zip(&y).all(|(p, q)| p.ram == q.ram), "{stem}: RAM differs");
                    assert!(shifts.len() == 1 && shifts.iter().all(|s| s.abs() < 3), "{stem}: {shifts:?}");
                    continue;
                }
                let (x, y) = (sweep(&a, &mut bounded(), &words(seed, 16)), sweep(&b, &mut bounded(), &words(seed, 16)));
                for c in 0..256 {
                    assert!(x[c] == y[c], "{stem} command {c:02X}: {:?}", compare(&x[c], &y[c]));
                }
            }
        }
    }

    // §6.2's model with two numbers a phase predicts every DSP-1B command's latencies under an S-CPU that answers
    // late by a different amount at every transfer.
    #[test]
    fn work_and_notice_predict_a_jittered_s_cpu() {
        let Some(idle) = chip("dsp1b") else { return };
        let mut modelled = 0;
        for c in 0..0x40u8 {
            for seed in 0..4 {
                let set = words(seed + 0x100 * c as u64, 16);
                let Some(phases) = timing(&idle, &bounded(), c, &set) else { continue };
                let j = transact(&mut idle.clone(), &mut Host { cap: 400_000, max_steps: 1100, ..Host::jittered(seed) }, c, &set);
                let answers: Vec<u32> = j.steps.iter().map(|s| s.answer).collect();
                assert_eq!(predict(&phases, &answers), j.latencies(), "command {c:02X} seed {seed}");
                modelled += 1;
            }
        }
        assert!(modelled > 200, "{modelled}");
    }

    // fullsnes: the DSP-1B fixes the DSP-1's 28h and answers 0101h to 2Fh. Every other command byte from 00h to 3Fh
    // but the data ROM dumps agrees over seeded inputs, timing included; 40h-FFh are passed over on both.
    #[test]
    fn the_dsp1_and_dsp1b_differ_where_fullsnes_says() {
        let (Some(a), Some(b)) = (chip("dsp1"), chip("dsp1b")) else { return };
        let dumps = |c: u8| matches!(c, 0x17 | 0x1F | 0x37 | 0x3F);
        let mut distance = 0;
        for c in 0..=255u8 {
            for seed in 0..8 {
                let set = words(seed + 0x1000 * c as u64, 16);
                let (x, y) = (transact(&mut a.clone(), &mut bounded(), c, &set), transact(&mut b.clone(), &mut bounded(), c, &set));
                if c >= 0x40 {
                    assert!(x.end == End::Ignored && y.end == End::Ignored, "{c:02X}");
                } else if c == 0x28 {
                    distance += compare(&x, &y).values as u32;
                } else if c & 0x0F == 0x0F && c & 0x30 == 0x20 || c == 0x27 {
                    assert!(compare(&x, &y).values, "{c:02X}");
                } else if !dumps(c) {
                    assert!(!compare(&x, &y).any(), "{c:02X}: {:?}", compare(&x, &y));
                }
            }
        }
        assert!(distance > 0);
    }
}
