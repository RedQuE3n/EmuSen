//! BlackMoon's GTE, the PlayStation's Geometry Transformation Engine on the R3000A's coprocessor 2, written from
//! psx-spx's GTE chapter and graded by ps1-tests' `test-all` and amidog's GTE test. The interface is BlackMoon_GTE.md
//! §2; the oracle §3.
//!
//! Stage 1: the registers, their names, the state block and the commands' signatures. Nothing executes until Diamond's
//! stage 5 (`BUILT`).

use emusen_mips::{Command, Coprocessor};
use emusen_native::{State, StateReader, StateWriter, Truncated};

/// Whether the commands and the register transfers run.
pub const BUILT: bool = false;

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

/// The 64 registers as the hardware holds them; what a read returns is the transfers' business (stage 5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub data: [u32; 32],
    pub control: [u32; 32],
}

/// The engine cannot run yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotBuilt;

/// The engine. It keeps no clock: a command returns the cycles until its results can be read, and the console, which
/// owns the clock, holds the processor's next access to coprocessor 2 until then.
#[derive(Clone, Debug, Default)]
pub struct Gte {
    pub regs: Registers,
}

impl Gte {
    pub fn new() -> Gte {
        Gte::default()
    }

    /// MFC2 and SWC2: data register `register`, as a read returns it.
    pub fn data(&self, _register: u8) -> Result<u32, NotBuilt> {
        Err(NotBuilt)
    }

    /// MTC2 and LWC2.
    pub fn set_data(&mut self, _register: u8, _value: u32) -> Result<(), NotBuilt> {
        Err(NotBuilt)
    }

    /// CFC2.
    pub fn control(&self, _register: u8) -> Result<u32, NotBuilt> {
        Err(NotBuilt)
    }

    /// CTC2.
    pub fn set_control(&mut self, _register: u8, _value: u32) -> Result<(), NotBuilt> {
        Err(NotBuilt)
    }

    /// A command, the COP2 instruction's low 25 bits; the cycles it takes.
    pub fn command(&mut self, _command: u32) -> Result<u32, NotBuilt> {
        Err(NotBuilt)
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
        }
        assert_eq!(emusen_mips::disassemble(0x4A14_0006, 0, &d).to_string(), "nclip 0x0140006", "the fake number differs");
        assert_eq!(emusen_mips::disassemble(0x4808_F800, 0, &d).to_string(), "mfc2 t0, LZCR");
        assert_eq!(emusen_mips::disassemble(0x4848_F800, 0, &d).to_string(), "cfc2 t0, FLAG");
        assert_eq!(emusen_mips::disassemble(0x4A00_0000, 0, &d).to_string(), "cop2 0x0000000");
    }

    #[test]
    fn the_engine_refuses_to_run_until_it_is_built() {
        let mut g = Gte::new();
        assert_eq!(g.command(0x0018_0001), Err(NotBuilt));
        assert_eq!((g.data(0), g.control(31)), (Err(NotBuilt), Err(NotBuilt)));
        assert_eq!((g.set_data(0, 1), g.set_control(31, 1)), (Err(NotBuilt), Err(NotBuilt)));
    }

    #[test]
    fn the_layout_is_pinned_and_a_block_round_trips() {
        let mut w = StateWriter::layout();
        Gte::new().write_state(&mut w);
        assert_eq!(w.into_layout(), "0 128 u32[32] Data\n128 128 u32[32] Control\n");
        let mut g = Gte::new();
        g.regs.data[15] = 0x00F0_0010;
        g.regs.control[31] = 0x8000_0000;
        let mut buf = vec![0u8; 256];
        g.write_state(&mut StateWriter::new(&mut buf));
        let mut back = Gte::new();
        back.read_state(&mut StateReader::new(&buf)).unwrap();
        assert_eq!(back.regs, g.regs);
        assert!(back.read_state(&mut StateReader::new(&buf[..255])).is_err());
        assert_eq!(back.regs, g.regs, "a short block changes nothing");
    }

    #[test]
    fn the_names_are_the_documents_and_unique() {
        let all: Vec<&str> = DATA_REGISTERS.iter().chain(&CONTROL_REGISTERS).copied().collect();
        assert_eq!(all.len(), 64);
        assert!(all.iter().all(|n| all.iter().filter(|m| *m == n).count() == 1));
        assert_eq!((DATA_REGISTERS[15], DATA_REGISTERS[31], CONTROL_REGISTERS[26], CONTROL_REGISTERS[31]), ("SXYP", "LZCR", "H", "FLAG"));
    }
}
