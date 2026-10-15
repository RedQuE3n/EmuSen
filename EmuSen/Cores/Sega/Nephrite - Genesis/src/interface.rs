//! The surface between the Genesis and an attachment, the Sega CD on its expansion port or the 32X in its cartridge
//! slot: the only module of the crate an attachment imports. The Genesis's buses and frame are generic over the
//! attachment, so each configuration is compiled apart and a Genesis with nothing attached (`Bare`) pays nothing for
//! the surface. Nephrite_Attachments.md §3 designs it and §11 records it as built.

pub use emusen_native::steps::Steps;
pub use emusen_native::{StateReader, StateWriter, Truncated};

/// A time on the Genesis's master clock: master clocks since power-on.
pub type Master = u64;

/// What the console tells an attachment as it is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    /// The master clock's rate, a fraction in Hz.
    pub master_hz: (u64, u64),
    pub pal: bool,
    /// The master clock of the 68000's first cycle, which power-on is.
    pub start: Master,
    /// The mix's output rate, which an attachment's stream is synthesised at.
    pub rate: u32,
}

/// An answer to a read in the 68000's space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer<T> {
    /// Not decoded here: the Genesis's own map answers.
    Pass,
    /// Decoded here with nothing driving the bus: the Genesis's open bus answers (Nephrite_Disputes.md D-29).
    Open,
    Value(T),
}

/// The bus side: asked first for every 68000 access, at `at`, the end of the 68000's bus cycle; times never decrease.
pub trait Attachment {
    /// An expansion unit on the side port: the version register's bit 5 reads 0.
    const EXPANSION: bool = false;

    /// A byte of the 68000's space, `a` masked to 24 bits.
    fn read8(&mut self, a: u32, at: Master) -> Answer<u8>;
    /// A word of the 68000's space, `a` masked to 24 bits and even.
    fn read16(&mut self, a: u32, at: Master) -> Answer<u16>;
    /// A word the VDP's transfer reads from the 68000's bus, which an attachment may answer apart from the 68000's.
    fn transfer16(&mut self, a: u32, at: Master) -> Answer<u16> {
        self.read16(a, at)
    }
    /// A write: `mask` is $FF00, $00FF or $FFFF as the 68000 drives the lanes, a byte on both halves of `v`; false where
    /// it is not decoded here.
    fn write(&mut self, a: u32, v: u16, mask: u16, at: Master) -> bool;
    /// Brought up to `to`, the 68000's present after each of its instructions.
    fn run_to(&mut self, to: Master);
    /// Its sound brought up to `to`, the stream the mix adds after the Genesis's output circuit; None for no sound.
    fn sound(&mut self, to: Master) -> Option<&mut Steps>;
}

/// Why a state read whole may not be applied to this attachment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A state made over another medium than the one loaded: the two fingerprints.
    WrongMedia { saved: u64, here: u64 },
}

/// The attachment's block of a state, written after the Genesis's (Nephrite_Attachments.md §3.5).
pub trait AttachmentState {
    type Saved;
    fn write_state(&self, w: &mut StateWriter);
    /// The block read whole before anything is applied; `version` is the state's.
    fn read_state(&self, r: &mut StateReader, version: i32) -> Result<Self::Saved, Truncated>;
    fn check(&self, saved: &Self::Saved) -> Result<(), Refusal>;
    fn apply_state(&mut self, saved: Self::Saved);
}

/// One space as the ABI lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpaceRef {
    pub id: u32,
    pub name: &'static str,
    pub read_only: bool,
}

/// Each owner's range of space ids, so that an id never moves between systems (Nephrite_Plan.md §4.3).
pub const GENESIS_SPACES: std::ops::Range<u32> = 0..16;
pub const MCD_SPACES: std::ops::Range<u32> = 16..32;
pub const S32X_SPACES: std::ops::Range<u32> = 32..48;

/// Each owner's range of processor ids, fixed whatever is fitted (decided 2026-10-15, Nephrite_Attachments.md Q7).
pub const GENESIS_PROCESSORS: std::ops::Range<u32> = 0..2;
pub const MCD_PROCESSORS: std::ops::Range<u32> = 2..3;
pub const S32X_PROCESSORS: std::ops::Range<u32> = 3..5;

/// The attachment's memories as the debugger and the state see them, from its own range of ids.
pub trait AttachmentView {
    /// Its spaces in id order.
    fn spaces(&self, out: &mut Vec<SpaceRef>);
    fn bytes(&self, id: u32) -> Option<&[u8]>;
    fn bytes_mut(&mut self, id: u32) -> Option<&mut [u8]>;
}

/// Nothing attached: every call answers that nothing is there, inlined so that each arm folds away.
#[derive(Clone, Copy, Debug, Default)]
pub struct Bare;

impl Attachment for Bare {
    #[inline(always)]
    fn read8(&mut self, _: u32, _: Master) -> Answer<u8> {
        Answer::Pass
    }
    #[inline(always)]
    fn read16(&mut self, _: u32, _: Master) -> Answer<u16> {
        Answer::Pass
    }
    #[inline(always)]
    fn write(&mut self, _: u32, _: u16, _: u16, _: Master) -> bool {
        false
    }
    #[inline(always)]
    fn run_to(&mut self, _: Master) {}
    #[inline(always)]
    fn sound(&mut self, _: Master) -> Option<&mut Steps> {
        None
    }
}

impl AttachmentState for Bare {
    type Saved = ();
    fn write_state(&self, _: &mut StateWriter) {}
    fn read_state(&self, _: &mut StateReader, _: i32) -> Result<(), Truncated> {
        Ok(())
    }
    fn check(&self, _: &()) -> Result<(), Refusal> {
        Ok(())
    }
    fn apply_state(&mut self, _: ()) {}
}

impl AttachmentView for Bare {
    fn spaces(&self, _: &mut Vec<SpaceRef>) {}
    fn bytes(&self, _: u32) -> Option<&[u8]> {
        None
    }
    fn bytes_mut(&mut self, _: u32) -> Option<&mut [u8]> {
        None
    }
}
