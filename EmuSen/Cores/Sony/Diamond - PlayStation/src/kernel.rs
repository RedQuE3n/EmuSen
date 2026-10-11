//! The kernel's seam (Diamond_Plan.md §5.9, Diamond_Native.md §11.10): which kernel image fills the BIOS region, and
//! the two points where the machine meets a running kernel without reading it: the shell's entry, where a PS-EXE is
//! side-loaded once the kernel has set itself up, and the documented A0h/B0h call entries, where a player's BIOS's
//! character output is taken for the TTY.

/// Where the kernel enters its shell once its tables are installed (psx-spx "Mid-Boot Hook": 80030000h), as a
/// physical address.
pub const SHELL_ENTRY: u32 = 0x0003_0000;

/// The A0h and B0h call entries and their putchar numbers (psx-spx "BIOS Function Summary": A(3Ch), B(3Dh)).
pub const A_CALLS: u32 = 0xA0;
pub const B_CALLS: u32 = 0xB0;
pub const A_PUTCHAR: u32 = 0x3C;
pub const B_PUTCHAR: u32 = 0x3D;

/// The kernel a machine runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kernel {
    /// A player's own BIOS from the firmware path: reset at BFC00000h, TTY by its putchar calls.
    Player,
    /// Diamond's open kernel: reset at BFC00000h, TTY by the DEV8 DUART's channel A.
    Open,
    /// None: a program is started directly with Exec's registers, and only the DUART carries TTY.
    None,
}

/// The open kernel's committed image (Diamond_OpenKernel.md §2.1, its hash beside it), which fills the BIOS region
/// when no player's BIOS is given. It is written apart, under the plan's §1.3 rules for replacement writers, and used
/// here through its §3.3 interface only.
static OPEN_KERNEL: &[u8] = include_bytes!("../kernel/diamond-kernel.bin");

pub fn open_kernel() -> Option<&'static [u8]> {
    Some(OPEN_KERNEL)
}

impl Kernel {
    /// Whether the machine resets into the BIOS region rather than starting a program directly.
    pub fn boots(self) -> bool {
        self != Kernel::None
    }

    /// Whether character output is taken from putchar calls at the A0h and B0h entries.
    pub fn taps_putchar(self) -> bool {
        self == Kernel::Player
    }
}

/// The character a putchar call at `pc` passes, if the call is one: the physical address an entry, R9 its number.
pub fn putchar(pc: u32, r9: u32, r4: u32) -> Option<u8> {
    match (pc & 0x1FFF_FFFF, r9) {
        (A_CALLS, A_PUTCHAR) | (B_CALLS, B_PUTCHAR) => Some(r4 as u8),
        _ => None,
    }
}
