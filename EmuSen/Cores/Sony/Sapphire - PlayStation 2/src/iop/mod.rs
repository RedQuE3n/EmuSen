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
}

impl Iop {
    pub fn new(blank: Timing) -> Iop {
        let mut cpu = R3000::new();
        cpu.prid = PRID;
        cpu.processor = 1;
        Iop { cpu, bus: IopBus::new(blank), held: true, exceptions: [0; 32] }
    }

    /// Releases the IOP at `pc` in kernel mode with SR clear, so that its exceptions use the vectors in RAM [C].
    pub fn enter(&mut self, pc: u32) {
        self.cpu.regs.pc = pc;
        self.cpu.regs.sr = 0;
        self.held = false;
    }

    /// The processor's reset (Sapphire_Disputes.md D-320): at the reset vector BFC00000h with SR's BEV set, as Diamond's
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
            if let (_, blackmoon_r3000::Step::Exception(code)) = self.cpu.run(&mut self.bus, u32::MAX) {
                self.exceptions[code as usize & 31] += 1;
            }
        }
    }
}
