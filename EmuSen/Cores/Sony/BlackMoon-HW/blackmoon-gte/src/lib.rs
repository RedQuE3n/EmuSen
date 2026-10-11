//! BlackMoon's GTE, the PlayStation's Geometry Transformation Engine on the R3000A's coprocessor 2, written from
//! psx-spx's GTE chapter and graded by ps1-tests' `test-all` and amidog's GTE test. The interface is BlackMoon_GTE.md
//! §2; the registers §3; the commands §5; the timing §6.

mod commands;
mod divide;
#[cfg(test)]
mod oracle;
mod registers;

use emusen_mips::{Command, Coprocessor};
use emusen_native::{State, StateReader, StateWriter, Truncated};

pub use commands::CYCLES;
pub use divide::{UNR_TABLE, divide};

/// Whether the commands and the register transfers run.
pub const BUILT: bool = true;

/// The data registers, cop2r0-31 (psx-spx "GTE Data Register Summary").
pub const DATA_REGISTERS: [&str; 32] = [
    "VXY0", "VZ0", "VXY1", "VZ1", "VXY2", "VZ2", "RGBC", "OTZ", "IR0", "IR1", "IR2", "IR3", "SXY0", "SXY1", "SXY2", "SXYP", "SZ0", "SZ1",
    "SZ2", "SZ3", "RGB0", "RGB1", "RGB2", "RES1", "MAC0", "MAC1", "MAC2", "MAC3", "IRGB", "ORGB", "LZCS", "LZCR",
];

/// The control registers, cop2r32-63 (psx-spx "GTE Control Register Summary").
pub const CONTROL_REGISTERS: [&str; 32] = [
    "RT11RT12", "RT13RT21", "RT22RT23", "RT31RT32", "RT33", "TRX", "TRY", "TRZ", "L11L12", "L13L21", "L22L23", "L31L32", "L33", "RBK", "GBK",
    "BBK", "LR1LR2", "LR3LG1", "LG2LG3", "LB1LB2", "LB3", "RFC", "GFC", "BFC", "OFX", "OFY", "H", "DQA", "DQB", "ZSF3", "ZSF4", "FLAG",
];

const fn command(name: &'static str, canonical: u32) -> Command {
    Command { name, mask: 0x3F, select: canonical & 0x3F, canonical }
}

/// The GTE as the R3000A's coprocessor 2, for `emusen-mips`: its registers by the names above, and its commands by the
/// real command number in bits 0-5, each with the 25 bits psx-spx's "GTE Opcode Summary" gives it.
pub static COPROCESSOR: Coprocessor = Coprocessor {
    data: DATA_REGISTERS,
    control: CONTROL_REGISTERS,
    commands: &[
        command("rtps", 0x018_0001),
        command("nclip", 0x140_0006),
        command("op", 0x170_000C),
        command("dpcs", 0x078_0010),
        command("intpl", 0x098_0011),
        command("mvmva", 0x040_0012),
        command("ncds", 0x0E8_0413),
        command("cdp", 0x128_0414),
        command("ncdt", 0x0F8_0416),
        command("nccs", 0x108_041B),
        command("cc", 0x138_041C),
        command("ncs", 0x0C8_041E),
        command("nct", 0x0D8_0420),
        command("sqr", 0x0A0_0428),
        command("dcpl", 0x068_0029),
        command("dpct", 0x0F8_002A),
        command("avsz3", 0x158_002D),
        command("avsz4", 0x168_002E),
        command("rtpt", 0x028_0030),
        command("gpf", 0x190_003D),
        command("gpl", 0x1A0_003E),
        command("ncct", 0x118_043F),
    ],
};

/// The 64 registers as the engine holds them: each written as psx-spx says a write is kept (a 16-bit register
/// sign- or zero-extended, FLAG's summary bit made), so that most reads are the word itself; the transfers apply the
/// rest (BlackMoon_GTE.md §3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub data: [u32; 32],
    pub control: [u32; 32],
}

/// One vertex of RTPS or RTPT as it is projected, for the enhancement layer's precision side-channel
/// (Diamond_Plan.md §6.2): the screen sums before their shift, so with 16 fraction bits, and what the FIFO kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projection {
    /// V0, V1 or V2.
    pub vertex: u8,
    /// MAC0's sums for SX and SY, before SAR 16 and saturation.
    pub x: i64,
    pub y: i64,
    /// SZ3 as pushed.
    pub z: u16,
    /// The SXY2 word pushed.
    pub word: u32,
}

/// What watches the commands. Every method has an empty default and the plain command runs with `Unobserved`, so an
/// unwatched engine compiles to no call; an observer sees results and never changes them.
pub trait Observer {
    fn projected(&mut self, _p: &Projection) {}
}

/// The observer that watches nothing.
pub struct Unobserved;

impl Observer for Unobserved {}

/// The engine. It keeps no clock: a command returns the cycles until its results can be read, and the console, which
/// owns the clock, holds the processor's next read of coprocessor 2, or its next command, until then.
#[derive(Clone, Debug, Default)]
pub struct Gte {
    regs: Registers,
}

impl Gte {
    pub fn new() -> Gte {
        Gte::default()
    }

    /// The registers as held, for the state and a debugger's listing; reads go through `data` and `control`.
    pub fn registers(&self) -> &Registers {
        &self.regs
    }

    /// MFC2 and SWC2: data register `register`, as a read returns it.
    pub fn data(&self, register: u8) -> u32 {
        registers::read_data(&self.regs, register as usize & 31)
    }

    /// MTC2 and LWC2.
    pub fn set_data(&mut self, register: u8, value: u32) {
        registers::write_data(&mut self.regs, register as usize & 31, value)
    }

    /// CFC2.
    pub fn control(&self, register: u8) -> u32 {
        registers::read_control(&self.regs, register as usize & 31)
    }

    /// CTC2.
    pub fn set_control(&mut self, register: u8, value: u32) {
        registers::write_control(&mut self.regs, register as usize & 31, value)
    }

    /// Register 0-63 as a read returns it, the data registers first: a debugger's view.
    pub fn register(&self, n: usize) -> u32 {
        if n < 32 { self.data(n as u8) } else { self.control(n as u8) }
    }

    /// A command, the COP2 instruction's low 25 bits; the cycles until its results can be read.
    pub fn command(&mut self, command: u32) -> u32 {
        commands::run(&mut self.regs, command, &mut Unobserved)
    }

    /// The same, with `observer` told what the command computes.
    pub fn command_observed<O: Observer>(&mut self, command: u32, observer: &mut O) -> u32 {
        commands::run(&mut self.regs, command, observer)
    }
}

impl State for Gte {
    type Error = Truncated;

    fn write_state(&self, w: &mut StateWriter) {
        w.u32s("Data", &self.regs.data);
        w.u32s("Control", &self.regs.control);
    }

    fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut g = Registers::default();
        r.u32s(&mut g.data)?;
        r.u32s(&mut g.control)?;
        self.regs = g;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each command is written by its name in its canonical form and with its bits otherwise, and reads back.
    #[test]
    fn the_commands_disassemble_by_name_and_assemble_back() {
        let d = emusen_mips::Dialect { cop: [None, None, Some(&COPROCESSOR), None] };
        let mut numbers: Vec<u32> = COPROCESSOR.commands.iter().map(|c| c.select).collect();
        numbers.sort();
        numbers.dedup();
        assert_eq!(numbers.len(), 22);
        for c in COPROCESSOR.commands {
            let word = 0x4A00_0000 | c.canonical;
            assert_eq!(emusen_mips::disassemble(word, 0, &d).to_string(), c.name);
            assert_eq!(emusen_mips::assemble_line(c.name, 0, &d), Ok(word));
            assert_ne!(CYCLES[c.select as usize], 0, "{} has its cycles", c.name);
        }
        assert_eq!(CYCLES.iter().filter(|&&n| n != 0).count(), 23, "and 00h, as RTPS");
        assert_eq!(emusen_mips::disassemble(0x4A14_0006, 0, &d).to_string(), "nclip 0x0140006", "the fake number differs");
        assert_eq!(emusen_mips::disassemble(0x4808_F800, 0, &d).to_string(), "mfc2 t0, LZCR");
        assert_eq!(emusen_mips::disassemble(0x4848_F800, 0, &d).to_string(), "cfc2 t0, FLAG");
        assert_eq!(emusen_mips::disassemble(0x4A00_0000, 0, &d).to_string(), "cop2 0x0000000");
    }

    #[test]
    fn the_registers_read_back_as_psx_spx_says() {
        let mut g = Gte::new();
        g.set_data(1, 0x1234_8001);
        g.set_data(16, 0xFFFF_8001);
        g.set_control(26, 0x0000_8000);
        g.set_control(31, 0xFFFF_FFFF);
        assert_eq!((g.data(1), g.data(16), g.control(26), g.control(31)), (0xFFFF_8001, 0x8001, 0xFFFF_8000, 0xFFFF_F000));
        g.set_control(31, 0x0040_0000);
        assert_eq!(g.control(31), 0x0040_0000, "IR3's bit is not summed");
        for (i, v) in [0x11, 0x22, 0x33].into_iter().enumerate() {
            g.set_data(12 + i as u8, v);
        }
        g.set_data(15, 0x44);
        assert_eq!([12, 13, 14, 15].map(|r| g.data(r)), [0x22, 0x33, 0x44, 0x44], "SXYP pushes");
        g.set_data(28, 0x7FFF);
        assert_eq!([9, 10, 11, 28, 29].map(|r| g.data(r)), [0xF80, 0xF80, 0xF80, 0x7FFF, 0x7FFF]);
        g.set_data(9, 0xFFFF_8000);
        g.set_data(10, 0x7FFF);
        assert_eq!(g.data(29), 0x7C00 | 0x1F << 5, "negative to 0, large to 1Fh");
        for (v, n) in [(0, 32), (1, 31), (0xFFFF_FFFF, 32), (0x8000_0000, 1), (0x7FFF_FFFF, 1), (0xFFF0_0000, 12)] {
            g.set_data(30, v);
            assert_eq!(g.data(31), n, "LZCR of {v:08X}");
        }
    }

    /// BlackMoon_GTE.md D-16: a number psx-spx lists as N/A clears FLAG and changes nothing else; 00h runs as RTPS.
    #[test]
    fn an_unlisted_number_clears_flag_and_00h_is_rtps() {
        let mut g = Gte::new();
        for r in 0..32 {
            g.set_data(r, 0x1234_5678u32.rotate_left(r as u32));
            g.set_control(r, 0x9ABC_DEF0u32.rotate_left(r as u32));
        }
        let before = *g.registers();
        assert_eq!(g.command(0x0000_0002), 0);
        let mut after = before;
        after.control[31] = 0;
        assert_eq!(*g.registers(), after);
        let (mut a, mut b) = (g.clone(), g.clone());
        assert_eq!((a.command(0x0008_0000), b.command(0x0008_0001)), (15, 15));
        assert_eq!(a.registers(), b.registers());
    }

    #[test]
    fn an_observer_sees_each_projection_and_changes_nothing() {
        struct Seen(Vec<Projection>);
        impl Observer for Seen {
            fn projected(&mut self, p: &Projection) {
                self.0.push(*p);
            }
        }
        let mut g = Gte::new();
        for (r, v) in [(0, 0x0010_0020), (1, 0x400), (2, 0xFFF0_0010), (3, 0x800), (4, 0x0005_FFF0), (5, 0x200)] {
            g.set_data(r, v);
        }
        for (r, v) in [(0, 0x1000), (2, 0x1000), (4, 0x1000), (24, 160 << 16), (25, 120 << 16), (26, 0x100)] {
            g.set_control(r, v);
        }
        let mut plain = g.clone();
        plain.command(0x028_0030);
        let mut seen = Seen(Vec::new());
        assert_eq!(g.command_observed(0x028_0030, &mut seen), 23);
        assert_eq!(g.registers(), plain.registers());
        assert_eq!(seen.0.iter().map(|p| p.vertex).collect::<Vec<_>>(), [0, 1, 2]);
        assert_eq!(seen.0.iter().map(|p| p.word).collect::<Vec<_>>(), [12, 13, 14].map(|r| g.data(r)));
        assert_eq!((seen.0[0].x >> 16, seen.0[0].z), (160 + 0x20 * 0x100 / 0x400, 0x400));
    }

    #[test]
    fn the_layout_is_pinned_and_a_block_round_trips() {
        let mut w = StateWriter::layout();
        Gte::new().write_state(&mut w);
        assert_eq!(w.into_layout(), "0 128 u32[32] Data\n128 128 u32[32] Control\n");
        let mut g = Gte::new();
        g.set_data(15, 0x00F0_0010);
        g.set_control(31, 0x8000_0000);
        g.command(0x0018_0001);
        let mut buf = vec![0u8; 256];
        g.write_state(&mut StateWriter::new(&mut buf));
        let mut back = Gte::new();
        back.read_state(&mut StateReader::new(&buf)).unwrap();
        assert_eq!(back.registers(), g.registers());
        assert!(back.read_state(&mut StateReader::new(&buf[..255])).is_err());
        assert_eq!(back.registers(), g.registers(), "a short block changes nothing");
    }

    #[test]
    fn the_names_are_the_documents_and_unique() {
        let all: Vec<&str> = DATA_REGISTERS.iter().chain(&CONTROL_REGISTERS).copied().collect();
        assert_eq!(all.len(), 64);
        assert!(all.iter().all(|n| all.iter().filter(|m| *m == n).count() == 1));
        assert_eq!((DATA_REGISTERS[15], DATA_REGISTERS[31], CONTROL_REGISTERS[26], CONTROL_REGISTERS[31]), ("SXYP", "LZCR", "H", "FLAG"));
    }
}
