//! The S-SMP around the SPC700: 64 KiB of RAM, the boot ROM, the I/O page with its four ports both ways, the three
//! timers and the S-DSP, which takes one of its steps in each SPC700 cycle; and its clock, run behind the S-CPU's and
//! caught up at a port access and at the frame's end (fullsnes, "SNES APU"; VenusRT_Plan.md §5.3). See
//! VenusRT_Native.md §21 and §22.

use super::dsp::Dsp;
use super::spc700::{self, Spc700};

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
    /// The 64-byte boot ROM the frontend supplies; without it the SPC700 does not run.
    pub ipl: Option<[u8; 64]>,
    pub test: u8,
    pub control: u8,
    pub dsp_address: u8,
    pub dsp: Dsp,
    /// The DSP's stereo samples since the machine last took them.
    pub out: Vec<i16>,
    /// $2140-$2143 as the S-CPU wrote them, and $F4-$F7 as the SPC700 wrote them.
    pub to_apu: [u8; 4],
    pub to_cpu: [u8; 4],
    pub timers: [Timer; 3],
    /// The DSP's cycles (1.024 MHz) since power-on; an SPC700 cycle is one of them at TEST's default.
    pub cycles: u64,
    /// The timers' first stages, one for timers 0 and 1 and one for timer 2, counting toward 384 and 48 (D-27).
    pub prescale: [u16; 2],
    pub ratio: (u64, u64),
}

impl Smp {
    pub fn new(ipl: Option<[u8; 64]>, pal: bool) -> Smp {
        let mut s = Smp {
            cpu: Spc700::default(),
            ram: vec![0; 0x10000].into(),
            ipl,
            // fullsnes: TEST 0Ah and CONTROL B0h on power-on.
            test: 0x0A,
            control: 0xB0,
            dsp_address: 0,
            dsp: Dsp { step: 1, ..Dsp::default() },
            out: Vec::new(),
            to_apu: [0; 4],
            to_cpu: [0; 4],
            timers: Default::default(),
            cycles: 0,
            prescale: [0; 2],
            ratio: if pal { PAL_RATIO } else { NTSC_RATIO },
        };
        if let Some(rom) = s.ipl {
            s.cpu.pc = u16::from_le_bytes([rom[62], rom[63]]);
        }
        s
    }

    /// Runs the SPC700 until its cycles reach the master clock `clock` stands for, an instruction at a time.
    pub fn run_to(&mut self, clock: u64) {
        if self.ipl.is_none() {
            return;
        }
        let target = (clock as u128 * self.ratio.0 as u128 / self.ratio.1 as u128) as u64;
        while self.cycles < target {
            let mut cpu = std::mem::take(&mut self.cpu);
            cpu.step(self);
            self.cpu = cpu;
        }
    }

    /// The S-CPU's read of $2140-$2143.
    pub fn cpu_read(&self, port: usize) -> u8 {
        self.to_cpu[port & 3]
    }

    pub fn cpu_write(&mut self, port: usize, value: u8) {
        self.to_apu[port & 3] = value;
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
    fn timer_step(&mut self) {
        let clk = (self.test >> 6) as usize;
        let step = (1u16 << clk) + (2u16 << ((self.test >> 4) & 3));
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
        // TEST bit 3 lets the timers count and bit 0 stops them; the first stage runs regardless (fullsnes, TEST; D-26).
        if self.test & 0x09 != 0x08 {
            return;
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

    fn io_read(&mut self, a: u16) -> u8 {
        match a {
            0xF2 => self.dsp_address,
            0xF3 => self.dsp.read(self.dsp_address),
            0xF4..=0xF7 => self.to_apu[(a - 0xF4) as usize],
            0xF8 | 0xF9 => self.ram[a as usize],
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
            0xF4..=0xF7 => self.to_cpu[(a - 0xF4) as usize] = v,
            0xFA..=0xFC => self.timers[(a - 0xFA) as usize].divider = v,
            _ => {}
        }
    }
}

impl spc700::Bus for Smp {
    fn read(&mut self, address: u16) -> u8 {
        self.tick();
        let v = match address {
            0x00F0..=0x00FF => self.io_read(address),
            0xFFC0..=0xFFFF if self.control & 0x80 != 0 => self.ipl.map_or(0, |r| r[(address - 0xFFC0) as usize]),
            _ => self.ram[address as usize],
        };
        self.timer_step();
        v
    }

    fn write(&mut self, address: u16, value: u8) {
        self.tick();
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

    /// A stand-in boot ROM for tests: its reset jumps to a loop at $FFC0 (`2F FE`). Not the console's.
    pub fn idle_ipl() -> [u8; 64] {
        let mut r = [0u8; 64];
        r[0] = 0x2F;
        r[1] = 0xFE;
        r[62] = 0xC0;
        r[63] = 0xFF;
        r
    }

    // fullsnes: the ports cross over, CONTROL's bits 4 and 5 clear the input latches, and the boot ROM shadows RAM.
    #[test]
    fn the_ports_cross_over_and_control_clears_the_inputs() {
        let mut s = Smp::new(Some(idle_ipl()), false);
        s.cpu_write(1, 0x55);
        use spc700::Bus;
        assert_eq!(s.read(0xF5), 0x55);
        s.write(0xF6, 0x77);
        assert_eq!(s.cpu_read(2), 0x77);
        s.write(0xF1, 0x10 | 0x80);
        assert_eq!(s.read(0xF5), 0);
        s.write(0xFFC0, 0x12);
        assert_eq!((s.read(0xFFC0), s.ram[0xFFC0]), (0x2F, 0x12));
        s.write(0xF1, 0x00);
        assert_eq!(s.read(0xFFC0), 0x12);
    }

    // fullsnes: timer 2 counts at 64 kHz, every 16 SPC700 cycles, divided by T2DIV; TnOUT clears when read.
    #[test]
    fn a_timer_counts_its_source_ticks_by_its_divider() {
        let mut s = Smp::new(Some(idle_ipl()), false);
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
        let mut s = Smp::new(Some(idle_ipl()), false);
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

    // The boot ROM's reset vector starts the CPU, and run_to keeps the SPC700 at the master clock's share.
    #[test]
    fn the_spc700_runs_to_the_master_clocks_share() {
        let mut s = Smp::new(Some(idle_ipl()), false);
        assert_eq!(s.cpu.pc, 0xFFC0);
        s.run_to(21_477_273);
        assert!((1_023_990..=1_024_010).contains(&s.cycles), "{}", s.cycles);
        let mut none = Smp::new(None, false);
        none.run_to(21_477_273);
        assert_eq!(none.cycles, 0);
    }
}
