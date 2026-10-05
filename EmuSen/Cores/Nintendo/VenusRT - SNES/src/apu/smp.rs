//! The S-SMP around the SPC700: 64 KiB of RAM, VenusRT's own boot program (D-38), the I/O page with its four ports both ways, the three
//! timers and the S-DSP, which takes one of its steps in each SPC700 cycle; and its clock, run behind the S-CPU's and
//! caught up at a port access and at the frame's end (fullsnes, "SNES APU"; VenusRT_Plan.md §5.3). See
//! VenusRT_Native.md §21 and §22.

use super::boot::BOOT;
use super::dsp::Dsp;
use super::spc700::{self, Spc700};

/// A peeked instruction: its length in DSP cycles, and up to four port writes with how many there were.
type Peeked = (u64, [(u64, u8, u8); 4], usize);

/// The writes a port queue holds at most; the SPC700 is never more than an instruction behind, so a few at most.
const QUEUE: usize = 8;
/// D-6: the SPC700 leaves reset 150 master clocks before the 65C816, and spends 8 cycles in its reset sequence.
const LEAD: u64 = 150;
const RESET_CYCLES: u32 = 8;
pub const PORT_BYTES: usize = 2 * (1 + QUEUE * 10);

/// SPC700 cycles to master clocks, exactly: 1.024 MHz against 236,250,000/11 Hz (NTSC) or 21,281,370 Hz (PAL).
pub const NTSC_RATIO: (u64, u64) = (5_632, 118_125);
pub const PAL_RATIO: (u64, u64) = (102_400, 2_128_137);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Timer {
    /// TnDIV, 0 meaning 256.
    pub divider: u8,
    /// The count of source ticks toward the divider.
    pub stage: u8,
    /// TnOUT's four bits.
    pub out: u8,
}

#[derive(Clone, Debug)]
pub struct Smp {
    pub cpu: Spc700,
    pub ram: Box<[u8]>,
    pub test: u8,
    pub control: u8,
    pub dsp_address: u8,
    pub dsp: Dsp,
    /// The DSP's stereo samples since the machine last took them.
    pub out: Vec<i16>,
    /// $2140-$2143 as the S-CPU wrote them, and $F4-$F7 as the SPC700 wrote them.
    pub to_apu: [u8; 4],
    pub to_cpu: [u8; 4],
    /// D-37: writes not yet landed on the other side, (quarter of a DSP cycle, port, value): the S-CPU's, landing
    /// at its bus cycle's end, and the SPC700's, landing in the fourth quarter of its cycle.
    pub to_apu_q: Vec<(u64, u8, u8)>,
    pub to_cpu_q: Vec<(u64, u8, u8)>,
    /// The last `peek`, kept while the SPC700 and the input queue stand where they were.
    peeked: Option<(u64, Peeked)>,
    /// The debugger's seam, fitted for an observed frame only; not in the state.
    pub probe: Option<Box<crate::probe::Probe>>,
    /// AUXIO4 and AUXIO5, latches read back as written, not the RAM beneath (D-31).
    pub aux: [u8; 2],
    pub timers: [Timer; 3],
    /// The DSP's cycles (1.024 MHz) since power-on; an SPC700 cycle is one of them at TEST's default.
    pub cycles: u64,
    /// The timers' first stages, one for timers 0 and 1 and one for timer 2, counting toward 384 and 48 (D-27).
    pub prescale: [u16; 2],
    /// TEST as the timers' last cycle saw it, which with `prescale` gives each first stage's gated level then (D-26); at
    /// an instruction's boundary it is TEST itself.
    pub gate_test: u8,
    pub ratio: (u64, u64),
    /// The boot program mapped at $FFC0-$FFFF: VenusRT's own (D-38), or the player's 64-byte image, `boot_file`.
    pub boot: [u8; 64],
    pub boot_file: bool,
}

impl Smp {
    pub fn new(pal: bool) -> Smp {
        Smp::with_boot(pal, None)
    }

    /// The S-SMP from power-on with the player's boot image in place of VenusRT's own, when one is given.
    pub fn with_boot(pal: bool, image: Option<[u8; 64]>) -> Smp {
        let mut s = Smp {
            cpu: Spc700::default(),
            ram: vec![0; 0x10000].into(),
            // fullsnes: TEST 0Ah and CONTROL B0h on power-on.
            test: 0x0A,
            control: 0xB0,
            dsp_address: 0,
            dsp: Dsp { step: 1, ..Dsp::default() },
            out: Vec::new(),
            to_apu: [0; 4],
            to_cpu: [0; 4],
            to_apu_q: Vec::new(),
            to_cpu_q: Vec::new(),
            peeked: None,
            probe: None,
            aux: [0xFF; 2],
            timers: Default::default(),
            cycles: 0,
            prescale: [0; 2],
            gate_test: 0x0A,
            ratio: if pal { PAL_RATIO } else { NTSC_RATIO },
            boot: image.unwrap_or(BOOT),
            boot_file: image.is_some(),
        };
        s.cpu.pc = u16::from_le_bytes([s.boot[62], s.boot[63]]);
        // D-6: the reset sequence is the BRK's cycles with its writes held off; nothing it reads has an effect.
        for _ in 0..RESET_CYCLES {
            s.tick();
            s.timer_step();
        }
        s
    }

    /// A master clock in quarters of a DSP cycle, the unit the ports' writes are stamped in (D-37).
    fn quarters(&self, clock: u64) -> u64 {
        let clock = clock + LEAD;
        (clock as u128 * 4 * self.ratio.0 as u128 / self.ratio.1 as u128) as u64
    }

    fn stretch(&self) -> u64 {
        [1, 2, 5, 10][(self.test >> 6) as usize]
    }

    /// Runs the SPC700 an instruction at a time while every cycle of the instruction starts before `clock`, so that
    /// no S-CPU write still to come can be one it should have read (D-37). It stays up to an instruction behind.
    pub fn run_to(&mut self, clock: u64) {
        let q = self.quarters(clock);
        loop {
            // The longest instruction, DIV, is 12 cycles.
            if 4 * (self.cycles + 12 * self.stretch()) > q {
                let (len, _, _) = self.peek();
                if 4 * (self.cycles + len - 1) >= q {
                    break;
                }
            }
            if let Some(p) = self.probe.as_mut() {
                if p.before(self.cpu.pc as u32) {
                    break;
                }
            }
            let mut cpu = std::mem::take(&mut self.cpu);
            cpu.step(self);
            self.cpu = cpu;
        }
        Self::land(&mut self.to_cpu, &mut self.to_cpu_q, q);
    }

    /// The next instruction run on a copy that changes nothing: its length in DSP cycles and its port writes.
    fn peek(&mut self) -> Peeked {
        if let Some((at, p)) = self.peeked {
            if at == self.cycles {
                return p;
            }
        }
        let p = self.peek_now();
        self.peeked = Some((self.cycles, p));
        p
    }

    fn peek_now(&self) -> Peeked {
        let mut bus = Peek { s: self, cycles: self.cycles, stretch: self.stretch(), writes: [(0, 0, 0); 4], n: 0 };
        let mut cpu = self.cpu.clone();
        cpu.step(&mut bus);
        (bus.cycles - self.cycles, bus.writes, bus.n)
    }

    /// Lands the queued writes stamped at or before `q` in `ports`.
    fn land(ports: &mut [u8; 4], queue: &mut Vec<(u64, u8, u8)>, q: u64) {
        let n = queue.iter().take_while(|w| w.0 <= q).count();
        for (_, p, v) in queue.drain(..n) {
            ports[p as usize] = v;
        }
    }

    /// The S-CPU's read of $2140-$2143 at the end of its bus cycle at `clock`, the SPC700 caught up to it first: what
    /// the SPC700 had written by then, the instruction it stands in included.
    pub fn cpu_read(&mut self, port: usize, clock: u64) -> u8 {
        let q = self.quarters(clock);
        Self::land(&mut self.to_cpu, &mut self.to_cpu_q, q);
        let mut v = self.to_cpu[port & 3];
        let (_, writes, n) = self.peek();
        for &(at, p, w) in &writes[..n] {
            if at <= q && p as usize == port & 3 {
                v = w;
            }
        }
        v
    }

    /// The S-CPU's write of $2140-$2143, landing at the end of its bus cycle at `clock`.
    pub fn cpu_write(&mut self, port: usize, value: u8, clock: u64) {
        let q = self.quarters(clock);
        Self::land(&mut self.to_apu, &mut self.to_apu_q, 4 * self.cycles);
        if self.to_apu_q.len() == QUEUE {
            let (_, p, v) = self.to_apu_q.remove(0);
            self.to_apu[p as usize] = v;
        }
        self.to_apu_q.push((q, (port & 3) as u8, value));
        self.peeked = None;
    }

    /// The two queues for a saved state: a count and `QUEUE` entries each.
    pub fn pack_ports(&self) -> Vec<u8> {
        let mut o = Vec::new();
        for queue in [&self.to_apu_q, &self.to_cpu_q] {
            o.push(queue.len() as u8);
            for k in 0..QUEUE {
                let (at, p, v) = queue.get(k).copied().unwrap_or_default();
                o.extend(at.to_le_bytes());
                o.extend([p, v]);
            }
        }
        o
    }

    pub fn unpack_ports(&mut self, b: &[u8]) {
        let one = |b: &[u8]| -> Vec<(u64, u8, u8)> {
            (0..(b[0] as usize).min(QUEUE)).map(|k| {
                let e = &b[1 + k * 10..11 + k * 10];
                (u64::from_le_bytes(e[..8].try_into().expect("eight bytes")), e[8] & 3, e[9])
            }).collect()
        };
        let half = 1 + QUEUE * 10;
        self.to_apu_q = one(&b[..half]);
        self.to_cpu_q = one(&b[half..2 * half]);
        self.peeked = None;
    }

    /// One SPC700 cycle passes: the timers' sources tick on their division of it (8 kHz is every 128th, 64 kHz every 16th).
    fn tick(&mut self) {
        // D-27: TEST bits 6-7 stretch every SPC700 cycle to 1, 2, 5 or 10 of the DSP's; bits 4-7 set the timers' step.
        let clk = (self.test >> 6) as usize;
        for _ in 0..[1, 2, 5, 10][clk] {
            self.cycles += 1;
            self.dsp.step(&mut self.ram, self.test & 0x02 != 0, &mut self.out);
        }
    }

    /// The timers' part of a cycle, after the SPC700's access in it, so that a TEST or CONTROL write counts at once (D-27).
    #[inline]
    fn timer_step(&mut self) {
        let clk = (self.test >> 6) as usize;
        let step = (1u16 << clk) + (2u16 << ((self.test >> 4) & 3));
        let before = self.prescale;
        let mut fire = [false; 3];
        self.prescale[0] += step;
        if self.prescale[0] >= 384 {
            self.prescale[0] -= 384;
            fire = [true, true, false];
        }
        self.prescale[1] += step;
        if self.prescale[1] >= 48 {
            self.prescale[1] -= 48;
            fire[2] = true;
        }
        // D-26: TEST bit 3 set and bit 0 clear gate each first stage's level, high over the period's second half, and a
        // second stage counts the gated level's falls. With TEST unchanged a fall is a wrap while the gate is open.
        if self.test == self.gate_test {
            if self.test & 0x09 != 0x08 {
                return;
            }
        } else {
            fire = self.gate_changed(before);
        }
        for (i, t) in self.timers.iter_mut().enumerate() {
            if self.control & (1 << i) == 0 || !fire[i] {
                continue;
            }
            t.stage = t.stage.wrapping_add(1);
            if t.stage == t.divider {
                t.stage = 0;
                t.out = (t.out + 1) & 0x0F;
            }
        }
    }

    /// A TEST write's cycle: which timers the gated levels' falls clock, between the last cycle's TEST and this one's.
    #[cold]
    fn gate_changed(&mut self, before: [u16; 2]) -> [bool; 3] {
        let was = Self::levels(self.gate_test, before);
        let now = Self::levels(self.test, self.prescale);
        self.gate_test = self.test;
        let fall = [was[0] && !now[0], was[1] && !now[1]];
        [fall[0], fall[0], fall[1]]
    }

    /// The two first stages' levels under a TEST value, from the stages' counts: high over the second half of each period.
    fn levels(test: u8, prescale: [u16; 2]) -> [bool; 2] {
        let run = test & 0x09 == 0x08;
        [run && prescale[0] >= 192, run && prescale[1] >= 24]
    }

    fn io_read(&mut self, a: u16) -> u8 {
        match a {
            0xF2 => self.dsp_address,
            0xF3 => self.dsp.read(self.dsp_address),
            0xF4..=0xF7 => {
                // D-37: latched in the first quarter of the read's cycle.
                let at = 4 * (self.cycles - self.stretch());
                Self::land(&mut self.to_apu, &mut self.to_apu_q, at);
                self.to_apu[(a - 0xF4) as usize]
            }
            0xF8 | 0xF9 => self.aux[(a - 0xF8) as usize],
            0xFD..=0xFF => {
                let t = &mut self.timers[(a - 0xFD) as usize];
                let v = t.out;
                t.out = 0;
                v
            }
            // TEST, CONTROL and the dividers are write-only.
            _ => 0,
        }
    }

    fn io_write(&mut self, a: u16, v: u8) {
        match a {
            0xF0 => self.test = v,
            0xF1 => {
                for (i, t) in self.timers.iter_mut().enumerate() {
                    // A timer enabled from disabled starts again from zero; a clear bit only stops it (D-25).
                    if v & !self.control & (1 << i) != 0 {
                        t.stage = 0;
                        t.out = 0;
                    }
                }
                let at = 4 * self.cycles - 1;
                Self::land(&mut self.to_apu, &mut self.to_apu_q, at);
                if v & 0x10 != 0 {
                    self.to_apu[0] = 0;
                    self.to_apu[1] = 0;
                }
                if v & 0x20 != 0 {
                    self.to_apu[2] = 0;
                    self.to_apu[3] = 0;
                }
                self.control = v;
            }
            0xF2 => self.dsp_address = v,
            0xF3 => self.dsp.write(self.dsp_address, v),
            0xF4..=0xF7 => {
                if self.to_cpu_q.len() == QUEUE {
                    let (_, p, w) = self.to_cpu_q.remove(0);
                    self.to_cpu[p as usize] = w;
                }
                self.to_cpu_q.push((4 * self.cycles - 1, (a - 0xF4) as u8, v));
            }
            0xF8 | 0xF9 => self.aux[(a - 0xF8) as usize] = v,
            0xFA..=0xFC => self.timers[(a - 0xFA) as usize].divider = v,
            _ => {}
        }
    }
}

/// The SPC700's view of the S-SMP for `Smp::peek`: reads without their side effects, the input ports as they will
/// stand at each cycle, and the output ports' writes recorded instead of made.
struct Peek<'a> {
    s: &'a Smp,
    cycles: u64,
    stretch: u64,
    writes: [(u64, u8, u8); 4],
    n: usize,
}

impl spc700::Bus for Peek<'_> {
    fn read(&mut self, address: u16) -> u8 {
        let latch = 4 * self.cycles;
        self.cycles += self.stretch;
        let s = self.s;
        match address {
            0x00F4..=0x00F7 => {
                let p = (address - 0xF4) as u8;
                s.to_apu_q.iter().filter(|w| w.0 <= latch && w.1 == p).last().map_or(s.to_apu[p as usize], |w| w.2)
            }
            0x00F2 => s.dsp_address,
            0x00F3 => s.dsp.read(s.dsp_address),
            0x00F8 | 0x00F9 => s.aux[(address - 0xF8) as usize],
            0x00FD..=0x00FF => s.timers[(address - 0xFD) as usize].out,
            0x00F0..=0x00FF => 0,
            0xFFC0..=0xFFFF if s.control & 0x80 != 0 => s.boot[(address - 0xFFC0) as usize],
            _ => s.ram[address as usize],
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.cycles += self.stretch;
        if (0x00F4..=0x00F7).contains(&address) && self.n < 4 {
            self.writes[self.n] = (4 * self.cycles - 1, (address - 0xF4) as u8, value);
            self.n += 1;
        }
    }

    fn idle(&mut self) {
        self.cycles += self.stretch;
    }
}

impl spc700::Bus for Smp {
    fn read(&mut self, address: u16) -> u8 {
        self.tick();
        let v = match address {
            0x00F0..=0x00FF => self.io_read(address),
            0xFFC0..=0xFFFF if self.control & 0x80 != 0 => self.boot[(address - 0xFFC0) as usize],
            _ => self.ram[address as usize],
        };
        self.timer_step();
        v
    }

    fn write(&mut self, address: u16, value: u8) {
        self.tick();
        if let Some(p) = self.probe.as_mut() {
            p.store(address as u32, value);
        }
        if (0x00F0..=0x00FF).contains(&address) {
            self.io_write(address, value);
        }
        // Writes reach RAM under the I/O page and the boot ROM as well, while TEST bit 1 allows them (fullsnes).
        if self.test & 0x02 != 0 {
            self.ram[address as usize] = value;
        }
        self.timer_step();
    }

    fn idle(&mut self) {
        self.tick();
        self.timer_step();
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    // fullsnes: the ports cross over, CONTROL's bits 4 and 5 clear the input latches, and the boot ROM shadows RAM.
    #[test]
    fn the_ports_cross_over_and_control_clears_the_inputs() {
        let mut s = Smp::new(false);
        s.cpu_write(1, 0x55, 0);
        use spc700::Bus;
        assert_eq!(s.read(0xF5), 0x55);
        s.write(0xF6, 0x77);
        assert_eq!(s.cpu_read(2, 100), 0x77);
        s.write(0xF1, 0x10 | 0x80);
        assert_eq!(s.read(0xF5), 0);
        s.write(0xFFC0, 0x12);
        assert_eq!((s.read(0xFFC0), s.ram[0xFFC0]), (BOOT[0], 0x12));
        s.write(0xF1, 0x00);
        assert_eq!(s.read(0xFFC0), 0x12);
    }

    // D-37: a program in RAM copying port 0 to port 1 forever; the S-CPU's write reaches it only after the write's
    // bus cycle, and the echo reaches the S-CPU only once the SPC700 has made it.
    #[test]
    fn the_ports_carry_each_write_at_its_own_time() {
        let mut s = Smp::new(false);
        s.ram[0x200..0x206].copy_from_slice(&[0xE4, 0xF4, 0xC4, 0xF5, 0x2F, 0xFA]);
        s.cpu.pc = 0x200;
        s.run_to(10_000);
        s.cpu_write(0, 0x55, 10_000);
        assert_eq!(s.cpu_read(1, 10_000), 0);
        s.run_to(10_400);
        assert_eq!(s.cpu_read(1, 10_400), 0x55);
        assert!(s.to_apu_q.is_empty() && s.cycles * 118_125 <= (10_400 + LEAD) * 5_632);
    }

    // fullsnes: timer 2 counts at 64 kHz, every 16 SPC700 cycles, divided by T2DIV; TnOUT clears when read.
    #[test]
    fn a_timer_counts_its_source_ticks_by_its_divider() {
        let mut s = Smp::new(false);
        use spc700::Bus;
        s.write(0xFC, 3);
        s.write(0xF1, 0x84);
        let start = s.cycles;
        while s.cycles - start < 16 * 3 * 5 {
            s.idle();
        }
        let out = s.read(0xFF);
        assert!((4..=5).contains(&out), "{out}");
        assert_eq!(s.read(0xFF), 0);
    }

    // D-25: clearing a timer's CONTROL bit stops it and keeps TnOUT; setting the bit again starts it from zero.
    #[test]
    fn a_timer_restarts_on_its_enable_edge() {
        let mut s = Smp::new(false);
        use spc700::Bus;
        s.write(0xFC, 1);
        s.write(0xF1, 0x84);
        while s.timers[2].out < 3 {
            s.idle();
        }
        s.write(0xF1, 0x80);
        for _ in 0..64 {
            s.idle();
        }
        assert_eq!(s.timers[2].out, 3);
        s.write(0xF1, 0x84);
        assert_eq!((s.timers[2].out, s.timers[2].stage), (0, 0));
        s.write(0xF1, 0x84);
        while s.timers[2].out == 0 {
            s.idle();
        }
        s.write(0xF1, 0x84);
        assert_eq!(s.timers[2].out, 1);
    }

    // D-26: TEST bit 0 set while timer 2's first stage is high counts once; set while it is low, or held, counts nothing.
    #[test]
    fn stopping_the_timers_while_the_first_stage_is_high_counts_once() {
        let mut s = Smp::new(false);
        use spc700::Bus;
        s.write(0xFC, 1);
        s.write(0xF1, 0x84);
        while s.timers[2].out == 0 {
            s.idle();
        }
        while s.prescale[1] < 24 {
            s.idle();
        }
        s.write(0xF0, 0x0B);
        assert_eq!(s.timers[2].out, 2);
        for _ in 0..64 {
            s.idle();
        }
        assert_eq!(s.timers[2].out, 2);
        s.write(0xF0, 0x0A);
        while s.prescale[1] >= 24 {
            s.idle();
        }
        let out = s.timers[2].out;
        s.write(0xF0, 0x0B);
        s.write(0xF0, 0x0A);
        assert_eq!(s.timers[2].out, out);
    }

    // The boot program's reset vector starts the CPU, and run_to keeps the SPC700 at the master clock's share.
    #[test]
    fn the_spc700_runs_to_the_master_clocks_share() {
        let mut s = Smp::new(false);
        assert_eq!(s.cpu.pc, 0xFFC0);
        s.run_to(21_477_273);
        assert!((1_023_990..=1_024_010).contains(&s.cycles), "{}", s.cycles);
    }
}
