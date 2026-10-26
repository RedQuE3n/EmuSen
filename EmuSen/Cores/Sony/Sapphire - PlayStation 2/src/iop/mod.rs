//! The IOP (stage S21, Sapphire_Native.md §22): `blackmoon-r3000`'s R3000A on Sapphire's own IOP bus (`bus`), which
//! holds the memory map, the I-cache (`icache`) and scratchpad, the wait states, the INTC (`intc`), the DMAC (`dma`)
//! and the six timers (`timer`). The IOP runs at an eighth of the EE's clock (`clock::IOP`) and is a unit of the board
//! with its own next event: the board brings it to the EE's time after each EE instruction. With no firmware there is
//! nothing at its reset vector, so it is held until a program is entered; with a BIOS it starts at its reset vector,
//! and again whenever the EE resets it through the SIF (S24, Sapphire_Native.md §35.2).

pub mod bus;
pub mod dma;
pub mod icache;
pub mod intc;
pub mod timer;
pub mod tty;

use blackmoon_r3000::R3000;

use crate::pcrtc::timing::Timing;
use crate::clock::IOP;
use bus::IopBus;

/// The IOP's PRId: 1Fh, inside the range ps2tek's two statements leave (at least 0Fh, since "RESET will jump to TBIN
/// if COP0.PRid is less than 0Fh", and below 59h, the EE's threshold) and the value PCSX2 reads (Sapphire_Disputes.md
/// D-60).
pub const PRID: u32 = 0x1F;

/// SR's BEV, the exception vectors in the boot ROM (psx-spx "COP0 Status Register").
pub const SR_BEV: u32 = 1 << 22;

/// The reset vector, in the boot ROM (psx-spx "Exception Vectors"; ps2tek "Memory Map").
pub const RESET: u32 = 0xBFC0_0000;

pub struct Iop {
    pub cpu: R3000,
    pub bus: IopBus,
    /// Held in reset: nothing runs until `enter`.
    pub held: bool,
    /// Exceptions and interrupts taken, by Cause's code, for the hardware's reports (S24).
    pub exceptions: [u64; 32],
    /// The console caught at the module interface on the player's BIOS (S10).
    pub tty: tty::Tty,
}

impl Iop {
    pub fn new(blank: Timing) -> Iop {
        let mut cpu = R3000::new();
        cpu.prid = PRID;
        cpu.processor = 1;
        Iop { cpu, bus: IopBus::new(blank), held: true, exceptions: [0; 32], tty: tty::Tty::default() }
    }

    /// Releases the IOP at `pc` in kernel mode with SR clear, so that its exceptions use the vectors in RAM [C].
    pub fn enter(&mut self, pc: u32) {
        self.cpu.regs.pc = pc;
        self.cpu.regs.sr = 0;
        self.held = false;
    }

    /// The processor's reset (Sapphire_Disputes.md D-340): at the reset vector BFC00000h with SR's BEV set, as Diamond's
    /// R3000A starts, its own units reset with it; held while the boot ROM is empty, and by the board while the EE holds
    /// the reset line through the SIF's CTRL bit 19, released when it clears the bit.
    pub fn reset(&mut self) {
        let mut cpu = R3000::new();
        cpu.prid = PRID;
        cpu.processor = 1;
        cpu.regs.pc = RESET;
        cpu.regs.sr = SR_BEV;
        cpu.hilo_ready = self.bus.now;
        self.cpu = cpu;
        self.bus.reset_units();
        self.held = self.bus.rom.is_empty();
    }

    /// The EE cycle of the IOP's next instruction, or never while it is held.
    pub fn next_event(&self) -> u64 {
        if self.held { u64::MAX } else { self.bus.now * IOP }
    }

    /// Runs the IOP until its clock reaches EE cycle `t`; the units are brought along at their events.
    pub fn run_to(&mut self, t: u64) {
        if self.held {
            self.bus.now = self.bus.now.max(t / IOP);
            self.bus.run_units(t);
            return;
        }
        while self.bus.now * IOP < t {
            self.bus.service();
            if self.bus.now * IOP >= t {
                break;
            }
            self.bus.set_horizon(t);
            let step = if self.tty.armed() { self.run_watched() } else { self.cpu.run(&mut self.bus, u32::MAX).1 };
            if let blackmoon_r3000::Step::Exception(code) = step {
                self.exceptions[code as usize & 31] += 1;
            }
        }
    }

    /// The console's hook kept, or found again through the export tables, at a frame's end (S10).
    pub fn resolve_tty(&mut self) {
        self.tty.resolve(&self.bus.ram, &self.bus.export_tables);
    }

    /// The run one instruction at a time while the console's hook is armed, each boundary shown to it first (S10).
    fn run_watched(&mut self) -> blackmoon_r3000::Step {
        use blackmoon_r3000::{Bus, Step};
        loop {
            let r = &self.cpu.regs.r;
            let a = [r[4], r[5], r[6]];
            if let Some(v) = self.tty.answer(self.cpu.regs.pc, a, &self.bus.ram) {
                let regs = &mut self.cpu.regs;
                regs.r[2] = v;
                regs.pc = regs.r[31];
                regs.branch = Default::default();
                continue;
            }
            self.tty.at(self.cpu.regs.pc, a);
            let step = self.cpu.run(&mut self.bus, 1).1;
            self.tty.stepped(step == Step::Instruction, &self.bus.ram);
            if step != Step::Instruction || !self.bus.running() {
                return step;
            }
        }
    }
}

impl Iop {
    /// The IOP's block of `SPH2` (Sapphire_Native.md §38.2): the R3000A's registers (its crate's block), whether it is
    /// held in reset, and its bus's own part; since version 3 the console's capture (S10, §36.3): the device, the hook
    /// and the export tables the bus noted, the text being an output, not state. The exception counts are the hardware's
    /// reports.
    pub fn write_state(&self, w: &mut emusen_native::StateWriter) {
        use emusen_native::State;
        w.group("Cpu", |w| self.cpu.write_state(w));
        w.bool("Held", self.held);
        w.group("Bus", |w| self.bus.write_own(w));
        w.group("Tty", |w| {
            self.tty.write_state(w);
            w.u32("Tables", self.bus.export_tables.len() as u32);
            w.u32s("TableSlots", &crate::state::slots::<u32, { tty::TABLES }>(&self.bus.export_tables.iter().copied().collect()));
        });
    }

    pub fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        use emusen_native::State;
        self.cpu.read_state(r)?;
        self.held = r.bool()?;
        self.bus.read_own(r)?;
        self.tty.read_state(r)?;
        let n = r.u32()? as usize;
        let mut slots = [0u32; tty::TABLES];
        r.u32s(&mut slots)?;
        self.bus.export_tables = slots[..n.min(tty::TABLES)].to_vec();
        Ok(())
    }
}
