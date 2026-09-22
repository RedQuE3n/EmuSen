//! The WiseMan fixtures the ported tests stand on: C#'s `new MemoryBus()`, `MipsAssembler` and `SyntheticN64Rom`.

use std::sync::Arc;

use crate::bus::{MemoryBus, RDRAM_SIZE, RDRAM_SIZE_EXPANDED};
use crate::cpu::{Cpu, Fault};
use crate::machine::Machine;
use crate::rom::{self, RomImage};
use crate::save::save_type;

/// `MipsAssembler.LoadAddress` and `EntryPoint`: the program at physical zero, fetched through KSEG0.
pub const LOAD_ADDRESS: u32 = 0;
pub const ENTRY_POINT: u64 = 0xFFFF_FFFF_8000_0000;

/// A fault code no instruction raises, standing for C#'s null `LastException`.
const NO_FAULT: u32 = u32::MAX;

/// `new MemoryBus()`: C#'s constructors leave the RSP halted and port one's controller present, which Rust leaves to `boot`.
pub fn new_bus() -> MemoryBus {
    built(RDRAM_SIZE)
}

/// `new MemoryBus(expansionPak: true)`.
pub fn new_bus_expanded() -> MemoryBus {
    built(RDRAM_SIZE_EXPANDED)
}

fn built(size: usize) -> MemoryBus {
    let mut bus = MemoryBus::new(size);
    bus.sp.processor.halted = true;
    bus.si.controllers[0].present = true;
    bus
}

/// `new MemoryBus { Cart = ... }`.
pub fn bus_with_cart(image: &[u8]) -> MemoryBus {
    let mut bus = new_bus();
    *bus.cart = Some(rom_image(image));
    bus
}

/// A C# `Cpu` and the bus it was built over, which Rust keeps apart.
pub struct Rig {
    pub cpu: Cpu,
    pub bus: MemoryBus,
}

impl Rig {
    /// `new Cpu(bus)`, with no exception recorded yet.
    pub fn over(bus: MemoryBus) -> Rig {
        let mut cpu = Cpu::power_on(&bus);
        cpu.run.fault.code = NO_FAULT;
        Rig { cpu, bus }
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    pub fn run(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// `LastException`.
    pub fn last(&self) -> Option<Fault> {
        (self.cpu.run.fault.code != NO_FAULT).then_some(self.cpu.run.fault)
    }

    pub fn last_code(&self) -> Option<u32> {
        self.last().map(|f| f.code)
    }

    pub fn cop0_written(&mut self) {
        self.cpu.cop0_written(&self.bus);
    }

    /// The program counter back on the assembled program.
    pub fn at_entry(&mut self) {
        self.cpu.pc = ENTRY_POINT;
        self.cpu.next_pc = ENTRY_POINT + 4;
    }
}

/// `MipsAssembler`: MIPS III words, so a test reads as assembly.
#[derive(Clone, Default)]
pub struct Asm {
    words: Vec<u32>,
}

impl Asm {
    pub fn new() -> Asm {
        Asm::default()
    }

    pub fn word(mut self, word: u32) -> Asm {
        self.words.push(word);
        self
    }

    fn i(self, op: u32, rs: u32, rt: u32, immediate: i16) -> Asm {
        self.word((op << 26) | (rs << 21) | (rt << 16) | immediate as u16 as u32)
    }

    fn u(self, op: u32, rs: u32, rt: u32, immediate: u16) -> Asm {
        self.word((op << 26) | (rs << 21) | (rt << 16) | immediate as u32)
    }

    fn r(self, rs: u32, rt: u32, rd: u32, sa: u32, funct: u32) -> Asm {
        self.word((rs << 21) | (rt << 16) | (rd << 11) | (sa << 6) | funct)
    }

    pub fn nop(self) -> Asm {
        self.word(0)
    }

    pub fn addiu(self, rt: u32, rs: u32, immediate: i16) -> Asm {
        self.i(0x09, rs, rt, immediate)
    }

    pub fn ori(self, rt: u32, rs: u32, immediate: u16) -> Asm {
        self.u(0x0D, rs, rt, immediate)
    }

    pub fn lui(self, rt: u32, immediate: u16) -> Asm {
        self.u(0x0F, 0, rt, immediate)
    }

    pub fn sra(self, rd: u32, rt: u32, sa: u32) -> Asm {
        self.r(0, rt, rd, sa, 0x03)
    }

    pub fn srav(self, rd: u32, rt: u32, rs: u32) -> Asm {
        self.r(rs, rt, rd, 0, 0x07)
    }

    pub fn mult(self, rs: u32, rt: u32) -> Asm {
        self.r(rs, rt, 0, 0, 0x18)
    }

    pub fn multu(self, rs: u32, rt: u32) -> Asm {
        self.r(rs, rt, 0, 0, 0x19)
    }

    pub fn beq(self, rs: u32, rt: u32, offset: i16) -> Asm {
        self.i(0x04, rs, rt, offset)
    }

    pub fn bne(self, rs: u32, rt: u32, offset: i16) -> Asm {
        self.i(0x05, rs, rt, offset)
    }

    pub fn lw(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x23, rs, rt, offset)
    }

    pub fn ld(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x37, rs, rt, offset)
    }

    pub fn lwl(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x22, rs, rt, offset)
    }

    pub fn lwr(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x26, rs, rt, offset)
    }

    pub fn ldl(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x1A, rs, rt, offset)
    }

    pub fn ldr(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x1B, rs, rt, offset)
    }

    pub fn swl(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x2A, rs, rt, offset)
    }

    pub fn swr(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x2E, rs, rt, offset)
    }

    pub fn sdl(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x2C, rs, rt, offset)
    }

    pub fn sw(self, rt: u32, rs: u32, offset: i16) -> Asm {
        self.i(0x2B, rs, rt, offset)
    }

    fn cop0_move(self, rs: u32, rt: u32, rd: u32) -> Asm {
        self.word((0x10 << 26) | (rs << 21) | (rt << 16) | (rd << 11))
    }

    pub fn mfc0(self, rt: u32, rd: usize) -> Asm {
        self.cop0_move(0, rt, rd as u32)
    }

    pub fn mtc0(self, rt: u32, rd: usize) -> Asm {
        self.cop0_move(4, rt, rd as u32)
    }

    /// `Cop1`: the sub-opcode field decides the whole of a coprocessor-1 move.
    pub fn cop1(self, rs: u32, rt: u32, fs: u32) -> Asm {
        self.word((0x11 << 26) | (rs << 21) | (rt << 16) | (fs << 11))
    }

    pub fn mfc1(self, rt: u32, fs: u32) -> Asm {
        self.cop1(0x00, rt, fs)
    }

    pub fn dmfc1(self, rt: u32, fs: u32) -> Asm {
        self.cop1(0x01, rt, fs)
    }

    pub fn cfc1(self, rt: u32, fs: u32) -> Asm {
        self.cop1(0x02, rt, fs)
    }

    pub fn mtc1(self, rt: u32, fs: u32) -> Asm {
        self.cop1(0x04, rt, fs)
    }

    pub fn dmtc1(self, rt: u32, fs: u32) -> Asm {
        self.cop1(0x05, rt, fs)
    }

    pub fn ctc1(self, rt: u32, fs: u32) -> Asm {
        self.cop1(0x06, rt, fs)
    }

    /// `Cop1Format`: format, function, and the three register fields.
    pub fn cop1_format(self, format: u32, funct: u32, fd: u32, fs: u32, ft: u32) -> Asm {
        self.word((0x11 << 26) | (format << 21) | (ft << 16) | (fs << 11) | (fd << 6) | funct)
    }

    fn cop0_function(self, funct: u32) -> Asm {
        self.word((0x10 << 26) | (0x10 << 21) | funct)
    }

    pub fn tlbr(self) -> Asm {
        self.cop0_function(0x01)
    }

    pub fn tlbwi(self) -> Asm {
        self.cop0_function(0x02)
    }

    pub fn tlbwr(self) -> Asm {
        self.cop0_function(0x06)
    }

    pub fn tlbp(self) -> Asm {
        self.cop0_function(0x08)
    }

    pub fn eret(self) -> Asm {
        self.cop0_function(0x18)
    }

    pub fn tge(self, rs: u32, rt: u32) -> Asm {
        self.r(rs, rt, 0, 0, 0x30)
    }

    pub fn tgeu(self, rs: u32, rt: u32) -> Asm {
        self.r(rs, rt, 0, 0, 0x31)
    }

    pub fn tlt(self, rs: u32, rt: u32) -> Asm {
        self.r(rs, rt, 0, 0, 0x32)
    }

    pub fn tltu(self, rs: u32, rt: u32) -> Asm {
        self.r(rs, rt, 0, 0, 0x33)
    }

    pub fn teq(self, rs: u32, rt: u32) -> Asm {
        self.r(rs, rt, 0, 0, 0x34)
    }

    pub fn tne(self, rs: u32, rt: u32) -> Asm {
        self.r(rs, rt, 0, 0, 0x36)
    }

    pub fn tgei(self, rs: u32, immediate: i16) -> Asm {
        self.i(0x01, rs, 0x08, immediate)
    }

    pub fn tgeiu(self, rs: u32, immediate: i16) -> Asm {
        self.i(0x01, rs, 0x09, immediate)
    }

    pub fn tlti(self, rs: u32, immediate: i16) -> Asm {
        self.i(0x01, rs, 0x0A, immediate)
    }

    pub fn tltiu(self, rs: u32, immediate: i16) -> Asm {
        self.i(0x01, rs, 0x0B, immediate)
    }

    pub fn teqi(self, rs: u32, immediate: i16) -> Asm {
        self.i(0x01, rs, 0x0C, immediate)
    }

    pub fn tnei(self, rs: u32, immediate: i16) -> Asm {
        self.i(0x01, rs, 0x0E, immediate)
    }

    pub fn syscall(self) -> Asm {
        self.r(0, 0, 0, 0, 0x0C)
    }

    pub fn words(&self) -> &[u32] {
        &self.words
    }

    /// `LoadInto`: through the bus, as C# writes it.
    pub fn load_into(&self, bus: &mut MemoryBus) {
        for (i, &word) in self.words.iter().enumerate() {
            bus.write32(LOAD_ADDRESS + i as u32 * 4, word);
        }
    }

    /// `Build`: the program in memory and the program counter already on it.
    pub fn build(self, mut bus: MemoryBus) -> Rig {
        self.load_into(&mut bus);
        let mut rig = Rig::over(bus);
        rig.at_entry();
        rig
    }

    pub fn build_new(self) -> Rig {
        self.build(new_bus())
    }

    pub fn run(self, steps: usize, bus: MemoryBus) -> Rig {
        let mut rig = self.build(bus);
        rig.run(steps);
        rig
    }

    pub fn run_new(self, steps: usize) -> Rig {
        self.run(steps, new_bus())
    }
}

/// `SyntheticN64Rom.Build`'s parameters; never real game data.
pub struct SyntheticRom {
    pub length: usize,
    pub entry_point: u32,
    pub title: &'static str,
    pub unique_code: &'static str,
    pub category: u8,
    pub destination: u8,
    pub version: u8,
    pub patches: Vec<(usize, Vec<u8>)>,
}

impl Default for SyntheticRom {
    fn default() -> Self {
        SyntheticRom {
            length: rom::MINIMUM_LENGTH,
            entry_point: 0x8000_0400,
            title: "WISEMAN",
            unique_code: "WM",
            category: b'N',
            destination: b'E',
            version: 0,
            patches: Vec::new(),
        }
    }
}

const TITLE_OFFSET: usize = 0x20;
const TITLE_LENGTH: usize = 20;

impl SyntheticRom {
    /// A default image with `bytes` just past the header.
    pub fn patched(bytes: &[u8]) -> SyntheticRom {
        SyntheticRom { patches: vec![(0, bytes.to_vec())], ..SyntheticRom::default() }
    }

    pub fn build(&self) -> Vec<u8> {
        let mut image = vec![0u8; self.length.max(rom::MINIMUM_LENGTH)];
        put32(&mut image, 0x00, rom::MAGIC);
        put32(&mut image, 0x04, 0x0000_000F);
        put32(&mut image, 0x08, self.entry_point);
        put32(&mut image, 0x0C, 0x0000_144C);
        put32(&mut image, 0x10, 0xDEAD_BEEF);
        put32(&mut image, 0x14, 0xFEED_FACE);
        image[TITLE_OFFSET..TITLE_OFFSET + TITLE_LENGTH].fill(b' ');
        for (i, b) in self.title.bytes().take(TITLE_LENGTH).enumerate() {
            image[TITLE_OFFSET + i] = b;
        }
        let unique = self.unique_code.as_bytes();
        image[0x3B] = self.category;
        image[0x3C] = unique[0];
        image[0x3D] = unique[1];
        image[0x3E] = self.destination;
        image[0x3F] = self.version;
        for (offset, bytes) in &self.patches {
            let at = rom::HEADER_LENGTH + offset;
            image[at..at + bytes.len()].copy_from_slice(bytes);
        }
        image
    }
}

fn put32(image: &mut [u8], at: usize, value: u32) {
    image[at..at + 4].copy_from_slice(&value.to_be_bytes());
}

/// `SyntheticN64Rom.Build()` with every default.
pub fn build_rom() -> Vec<u8> {
    SyntheticRom::default().build()
}

/// `BuildHomebrew(saveType)`.
pub fn build_homebrew(kind: i32) -> Vec<u8> {
    build_homebrew_with(kind, false, false)
}

/// `BuildHomebrew`: an ED64-convention header, the only way an image states its own save type.
pub fn build_homebrew_with(kind: i32, real_time_clock: bool, region_free: bool) -> Vec<u8> {
    let nibble: u8 = match kind {
        save_type::NONE => 0,
        save_type::EEPROM_4K => 1,
        save_type::EEPROM_16K => 2,
        save_type::SRAM_256K => 3,
        save_type::SRAM_BANKED_768K => 4,
        save_type::FLASH_RAM => 5,
        save_type::SRAM_1M => 6,
        _ => 0xF,
    };
    let version = (nibble << 4) | u8::from(real_time_clock) | (u8::from(region_free) << 1);
    SyntheticRom { unique_code: "ED", version, ..SyntheticRom::default() }.build()
}

/// `ToByteSwapped`: every halfword swapped, as a Doctor V64 dump.
pub fn byte_swapped(big_endian: &[u8]) -> Vec<u8> {
    big_endian.chunks(2).flat_map(|p| p.iter().rev().copied()).collect()
}

/// `ToLittleEndian`: every word reversed.
pub fn little_endian(big_endian: &[u8]) -> Vec<u8> {
    big_endian.chunks(4).flat_map(|p| p.iter().rev().copied()).collect()
}

pub fn rom_image(image: &[u8]) -> Arc<RomImage> {
    Arc::new(RomImage::from_image(image).unwrap())
}

/// `Convert.FromHexString`.
pub fn from_hex(text: &str) -> Vec<u8> {
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap()).collect()
}

/// `Convert.ToHexString`: upper case, no separators.
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

/// `bus.WriteState(w)`: settled, then the bus's bytes, carried in a machine whose processor is the default.
pub fn bus_state(bus: &mut MemoryBus) -> Vec<u8> {
    bus.settle();
    let mut machine = Machine::new(bus.rdram.len()).unwrap();
    machine.bus = bus.clone();
    machine.save_state_vec(false).unwrap()
}

/// `bus.ReadState(r)`: the bytes, then everything a load derives.
pub fn load_bus(bus: &mut MemoryBus, state: &[u8]) {
    let mut machine = Machine::new(bus.rdram.len()).unwrap();
    std::mem::swap(&mut machine.bus, bus);
    machine.restore_state(state).unwrap();
    std::mem::swap(&mut machine.bus, bus);
}
