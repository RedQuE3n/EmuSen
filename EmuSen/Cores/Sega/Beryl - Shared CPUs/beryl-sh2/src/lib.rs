//! Beryl's Hitachi SH-2, the SH7604 with its on-chip cache and peripherals, written from Hitachi's SH-1/SH-2
//! Programming Manual and the SH7604 Hardware Manual. The 32X's two CPUs, and later the Saturn's. No single-step suite
//! for the SH-2 was found (Beryl_SH2.md §3); its oracle is built from the manuals and test programs.
//!
//! Stage 0: the bus, the registers and the step's signature. No instruction executes until the 32X's stage (`BUILT`).

pub use emusen_native::debug::{Observer, Unobserved};

/// Whether the processor executes.
pub const BUILT: bool = false;

/// The width of an external bus cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Byte,
    Word,
    Long,
}

/// What the processor's external bus meets, after its own cache and on-chip modules have answered what is theirs.
/// The bus owns the clock: each external access and each internal cycle is reported, and the bus advances its time by
/// it, the bus state controller's waits included.
pub trait Bus {
    fn read(&mut self, address: u32, size: Size) -> u32;
    fn write(&mut self, address: u32, size: Size, value: u32);
    /// Cycles spent inside the processor: pipeline stalls, multiplier and divider latency, cache hits.
    fn idle(&mut self, cycles: u32);
    /// The external interrupt request as IRL3-IRL0 present it, 0 for none.
    fn interrupt_level(&mut self) -> u8;
    /// The vector an external interrupt supplies, or `None` for the auto-vector.
    fn acknowledge(&mut self, level: u8) -> Option<u8>;
}

/// The general and control registers (SH-1/SH-2 Programming Manual §2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub r: [u32; 16],
    pub sr: u32,
    pub gbr: u32,
    pub vbr: u32,
    pub mach: u32,
    pub macl: u32,
    pub pr: u32,
    pub pc: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Instruction,
    /// An exception or interrupt was taken: its vector number.
    Exception(u8),
    /// SLEEP waits for an interrupt.
    Sleeping,
}

/// The processor cannot step yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotBuilt;

#[derive(Clone, Debug, Default)]
pub struct Sh2 {
    pub regs: Registers,
    /// Set for the slave of a pair, which the 32X's boot programs tell apart.
    pub slave: bool,
}

impl Sh2 {
    pub fn new(slave: bool) -> Sh2 {
        Sh2 { slave, ..Sh2::default() }
    }

    pub fn step<B: Bus>(&mut self, bus: &mut B) -> Result<Step, NotBuilt> {
        self.step_observed(bus, &mut Unobserved)
    }

    /// The step a debugger watches: `observer.before` at the instruction boundary, stores and calls as they happen.
    pub fn step_observed<B: Bus, O: Observer>(&mut self, _bus: &mut B, _observer: &mut O) -> Result<Step, NotBuilt> {
        Err(NotBuilt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Flat;

    impl Bus for Flat {
        fn read(&mut self, _: u32, _: Size) -> u32 {
            0
        }
        fn write(&mut self, _: u32, _: Size, _: u32) {}
        fn idle(&mut self, _: u32) {}
        fn interrupt_level(&mut self) -> u8 {
            0
        }
        fn acknowledge(&mut self, _: u8) -> Option<u8> {
            None
        }
    }

    #[test]
    fn the_processor_refuses_to_step_until_it_is_built() {
        assert!(!BUILT);
        assert_eq!(Sh2::new(true).step(&mut Flat), Err(NotBuilt));
    }
}
