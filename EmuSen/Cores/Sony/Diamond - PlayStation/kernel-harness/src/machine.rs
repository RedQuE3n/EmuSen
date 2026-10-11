//! blackmoon-r3000 on a bus of the harness's own: 2 MiB of RAM mirrored through 8 MiB, the scratchpad, the BIOS
//! holding the kernel's image, I_STAT and I_MASK with the interrupt line they drive, a GPU status that is always
//! ready, the DUART's transmit register captured as the TTY, and every other port a plain store that is logged.
//! It is a test bench for the kernel and models no device's timing (Diamond_OpenKernel.md §7).

use blackmoon_r3000::{Bus, R3000, Size, Step};

pub const RAM: usize = 2 << 20;
pub const RESET: u32 = 0xBFC0_0000;
pub const SHELL: u32 = 0x8003_0000;
/// A loop the harness's calls return to.
pub const SENTINEL: u32 = 0x801F_F800;
pub const STACK: u32 = 0x801F_E000;
pub const I_STAT: u32 = 0x1F80_1070;
pub const I_MASK: u32 = 0x1F80_1074;
pub const GP1: u32 = 0x1F80_1814;
pub const THRA: u32 = 0x1F80_2023;
pub const BCC: u32 = 0xFFFE_0130;

pub struct Rig {
    pub ram: Vec<u8>,
    pub bios: Vec<u8>,
    pub scratchpad: [u8; 0x400],
    pub io: Vec<u8>,
    pub i_stat: u32,
    pub i_mask: u32,
    pub bcc: u32,
    pub tty: Vec<u8>,
    /// Every store outside RAM and the scratchpad: physical address, width, value.
    pub writes: Vec<(u32, Size, u32)>,
    pub isolated_writes: usize,
    pub cop2_commands: Vec<u32>,
    pub clock: u64,
}

fn physical(address: u32) -> u32 {
    match address >> 29 {
        4 => address - 0x8000_0000,
        5 => address - 0xA000_0000,
        6 | 7 => address,
        _ => address & 0x1FFF_FFFF,
    }
}

impl Rig {
    pub fn new(bios: &[u8]) -> Rig {
        assert_eq!(bios.len(), 512 * 1024);
        Rig {
            ram: vec![0; RAM],
            bios: bios.to_vec(),
            scratchpad: [0; 0x400],
            io: vec![0; 0x2000],
            i_stat: 0,
            i_mask: 0,
            bcc: 0,
            tty: Vec::new(),
            writes: Vec::new(),
            isolated_writes: 0,
            cop2_commands: Vec::new(),
            clock: 0,
        }
    }

    fn byte(&self, p: u32) -> u8 {
        match p {
            0..0x0080_0000 => self.ram[p as usize % RAM],
            0x1F80_0000..0x1F80_0400 => self.scratchpad[(p - 0x1F80_0000) as usize],
            0x1F80_1000..0x1F80_3000 => self.io[(p - 0x1F80_1000) as usize],
            0x1FC0_0000..0x1FC8_0000 => self.bios[(p - 0x1FC0_0000) as usize],
            _ => 0,
        }
    }

    fn load(&self, p: u32, size: Size) -> u32 {
        match p & !3 {
            I_STAT => return self.i_stat >> (8 * (p & 3)),
            I_MASK => return self.i_mask >> (8 * (p & 3)),
            GP1 => return 0x1C00_0000 >> (8 * (p & 3)),
            BCC => return self.bcc,
            _ => {}
        }
        (0..size.bytes()).fold(0, |v, i| v | (self.byte(p + i) as u32) << (8 * i))
    }

    fn store(&mut self, p: u32, size: Size, value: u32) {
        let in_memory = p < 0x0080_0000 || (0x1F80_0000..0x1F80_0400).contains(&p);
        if !in_memory {
            self.writes.push((p, size, value));
        }
        match p & !3 {
            I_STAT => {
                self.i_stat &= value;
                return;
            }
            I_MASK => {
                self.i_mask = value & 0x7FF;
                return;
            }
            BCC => {
                self.bcc = value;
                return;
            }
            _ => {}
        }
        if p == THRA {
            self.tty.push(value as u8);
        }
        for i in 0..size.bytes() {
            let q = p + i;
            let b = (value >> (8 * i)) as u8;
            match q {
                0..0x0080_0000 => self.ram[q as usize % RAM] = b,
                0x1F80_0000..0x1F80_0400 => self.scratchpad[(q - 0x1F80_0000) as usize] = b,
                0x1F80_1000..0x1F80_3000 => self.io[(q - 0x1F80_1000) as usize] = b,
                _ => {}
            }
        }
    }

    pub fn word(&self, address: u32) -> u32 {
        self.load(physical(address), Size::Word)
    }

    pub fn put(&mut self, address: u32, value: u32) {
        let p = physical(address) as usize % RAM;
        self.ram[p..p + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn bytes(&self, address: u32, n: usize) -> Vec<u8> {
        (0..n as u32).map(|i| self.byte(physical(address + i))).collect()
    }

    pub fn put_bytes(&mut self, address: u32, data: &[u8]) {
        for (i, b) in data.iter().enumerate() {
            self.ram[(physical(address) as usize + i) % RAM] = *b;
        }
    }

    pub fn string(&self, address: u32) -> String {
        let mut s = String::new();
        let mut a = address;
        loop {
            let b = self.byte(physical(a));
            if b == 0 {
                return s;
            }
            s.push(b as char);
            a += 1;
        }
    }

    pub fn tty_text(&self) -> String {
        String::from_utf8_lossy(&self.tty).into_owned()
    }

    pub fn raise(&mut self, irq: u32) {
        self.i_stat |= 1 << irq;
    }
}

impl Bus for Rig {
    fn fetch(&mut self, address: u32) -> u32 {
        self.clock += 1;
        self.load(physical(address), Size::Word)
    }
    fn read(&mut self, address: u32, size: Size) -> u32 {
        self.clock += 1;
        self.load(physical(address), size)
    }
    fn write(&mut self, address: u32, size: Size, value: u32) {
        self.clock += 1;
        self.store(physical(address), size, value);
    }
    fn isolated_read(&mut self, _address: u32, _size: Size) -> u32 {
        self.clock += 1;
        0
    }
    fn isolated_write(&mut self, _address: u32, _size: Size, _value: u32) {
        self.clock += 1;
        self.isolated_writes += 1;
    }
    fn idle(&mut self, cycles: u32) {
        self.clock += cycles as u64;
    }
    fn cycles(&mut self) -> u64 {
        self.clock
    }
    fn interrupt(&mut self) -> bool {
        self.i_stat & self.i_mask & 0x7FF != 0
    }
    fn cop2_data(&mut self, _register: u8) -> u32 {
        0
    }
    fn set_cop2_data(&mut self, _register: u8, _value: u32) {}
    fn cop2_control(&mut self, _register: u8) -> u32 {
        0
    }
    fn set_cop2_control(&mut self, _register: u8, _value: u32) {}
    fn cop2_command(&mut self, command: u32) {
        self.cop2_commands.push(command);
    }
}

pub struct Machine {
    pub cpu: R3000,
    pub rig: Rig,
    /// The Cause codes of the exceptions taken, in order.
    pub exceptions: Vec<u8>,
}

impl Machine {
    /// The CPU at its reset vector with SR.BEV set, the kernel's image in the BIOS.
    pub fn reset(bios: &[u8]) -> Machine {
        let mut cpu = R3000::new();
        cpu.regs.pc = RESET;
        cpu.regs.sr = 1 << 22;
        Machine { cpu, rig: Rig::new(bios), exceptions: Vec::new() }
    }

    pub fn step(&mut self) {
        if let Step::Exception(code) = self.cpu.step(&mut self.rig) {
            self.exceptions.push(code);
        }
    }

    /// Steps until the PC is `pc`, at most `limit` of them; the count, or an error naming where the PC was.
    pub fn run_to(&mut self, pc: u32, limit: usize) -> Result<usize, String> {
        for n in 0..limit {
            if self.cpu.regs.pc == pc {
                return Ok(n);
            }
            self.step();
        }
        Err(format!("PC {:08X} not reached in {limit} steps; at {:08X}", pc, self.cpu.regs.pc))
    }

    /// Reset, and the kernel's boot run to the shell's entry.
    pub fn boot(bios: &[u8]) -> Machine {
        let mut m = Machine::reset(bios);
        m.run_to(SHELL, 5_000_000).expect("the boot reaches 80030000h");
        m.rig.put(SENTINEL, 0x0800_0000 | (SENTINEL >> 2 & 0x03FF_FFFF));
        m.rig.put(SENTINEL + 4, 0);
        m
    }

    /// A(n), B(n) or C(n) called through its vector with up to eight arguments, returning to the sentinel: v0.
    pub fn call(&mut self, vector: u32, number: u32, args: &[u32]) -> u32 {
        let r = &mut self.cpu.regs;
        r.r[29] = STACK;
        for (i, a) in args.iter().enumerate() {
            if i < 4 {
                r.r[4 + i] = *a;
            }
        }
        for (i, a) in args.iter().enumerate() {
            self.rig.put(STACK + 4 * i as u32, *a);
        }
        let r = &mut self.cpu.regs;
        r.r[9] = number;
        r.r[10] = vector;
        r.r[31] = SENTINEL;
        r.pc = 0x8000_0000 | vector;
        r.branch = Default::default();
        r.load = Default::default();
        self.run_to(SENTINEL, 20_000_000).unwrap_or_else(|e| panic!("{vector:X}({number:X}h): {e}"));
        self.cpu.regs.r[2]
    }

    pub fn a(&mut self, number: u32, args: &[u32]) -> u32 {
        self.call(0xA0, number, args)
    }

    pub fn b(&mut self, number: u32, args: &[u32]) -> u32 {
        self.call(0xB0, number, args)
    }

    pub fn c(&mut self, number: u32, args: &[u32]) -> u32 {
        self.call(0xC0, number, args)
    }

    /// A string placed in user RAM for a call, its address.
    pub fn text(&mut self, address: u32, s: &str) -> u32 {
        let mut b = s.as_bytes().to_vec();
        b.push(0);
        self.rig.put_bytes(address, &b);
        address
    }
}

/// Where test programs are placed: user RAM's first address, as a PS-EXE's usual destination.
pub const PROGRAM: u32 = 0x8001_0000;

impl Machine {
    /// A program side-loaded as a PS-EXE and run until it reaches its last two lines, `done: j done` and `nop`.
    pub fn run_program(&mut self, source: &str, limit: usize) {
        let exe = crate::psexe::Exe::program(source, PROGRAM);
        let done = PROGRAM + exe.body.len() as u32 - 8;
        exe.sideload(self);
        self.run_to(done, limit).unwrap_or_else(|e| panic!("program: {e}"));
    }

    /// Words assembled at `origin` and written there; their length in bytes.
    pub fn code(&mut self, origin: u32, source: &str) -> u32 {
        let words = crate::psexe::assemble(source, origin);
        for (i, w) in words.iter().enumerate() {
            self.rig.put(origin + 4 * i as u32, *w);
        }
        4 * words.len() as u32
    }
}
