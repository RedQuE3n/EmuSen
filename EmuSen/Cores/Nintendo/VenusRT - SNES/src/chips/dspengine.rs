//! The cartridge's NEC DSP slot: the low-level path running the player's image, or VenusRT's open replacement, behind
//! one port interface (VenusRT_DspHle.md §2.2). See VenusRT_Native.md §38.

use super::dsphle::{DspHle, Program};
use super::necdsp::{NecDsp, Port};

#[derive(Clone, Debug)]
pub enum DspEngine {
    Lle(NecDsp),
    Hle(DspHle),
}

impl DspEngine {
    pub fn replacement(program: Program) -> DspEngine {
        DspEngine::Hle(DspHle::new(program))
    }

    /// The µPD96050's slot (ST010, ST011): its RAM is on the S-CPU's bus.
    pub fn st(&self) -> bool {
        match self {
            DspEngine::Lle(d) => d.st,
            DspEngine::Hle(h) => h.st(),
        }
    }

    pub fn lle(&self) -> Option<&NecDsp> {
        match self {
            DspEngine::Lle(d) => Some(d),
            DspEngine::Hle(_) => None,
        }
    }

    pub fn lle_mut(&mut self) -> Option<&mut NecDsp> {
        match self {
            DspEngine::Lle(d) => Some(d),
            DspEngine::Hle(_) => None,
        }
    }

    /// The ST01x's RAM as words, the battery file's source; empty on a DSP-n replacement.
    pub fn ram(&self) -> &[u16] {
        match self {
            DspEngine::Lle(d) => &d.ram,
            DspEngine::Hle(h) => &h.ram,
        }
    }

    pub fn ram_mut(&mut self) -> &mut [u16] {
        match self {
            DspEngine::Lle(d) => &mut d.ram,
            DspEngine::Hle(h) => &mut h.ram,
        }
    }

    #[inline]
    pub fn run_to(&mut self, clock: u64) {
        match self {
            DspEngine::Lle(d) => d.run_to(clock),
            DspEngine::Hle(h) => h.run_to(clock),
        }
    }

    #[inline]
    pub fn host_read(&mut self, port: Port, side_effects: bool) -> u8 {
        match self {
            DspEngine::Lle(d) => d.host_read(port, side_effects),
            DspEngine::Hle(h) => h.host_read(port, side_effects),
        }
    }

    #[inline]
    pub fn host_write(&mut self, port: Port, value: u8) {
        match self {
            DspEngine::Lle(d) => d.host_write(port, value),
            DspEngine::Hle(h) => h.host_write(port, value),
        }
    }

    pub fn reset(&mut self) {
        match self {
            DspEngine::Lle(d) => d.reset(),
            DspEngine::Hle(h) => h.reset(),
        }
    }

    /// The state's tag for the engine that wrote a `Coprocessor` group: 0 the low-level path, 1 the replacement.
    pub fn tag(&self) -> u8 {
        match self {
            DspEngine::Lle(_) => 0,
            DspEngine::Hle(_) => 1,
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        match self {
            DspEngine::Lle(d) => d.pack(),
            DspEngine::Hle(h) => h.pack(),
        }
    }

    pub fn unpack(&mut self, data: &[u8]) {
        match self {
            DspEngine::Lle(d) => d.unpack(data),
            DspEngine::Hle(h) => h.unpack(data),
        }
    }
}
