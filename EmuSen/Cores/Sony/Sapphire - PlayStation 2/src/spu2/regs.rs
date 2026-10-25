//! The SPU2's registers by byte offset from 1F900000h. ps2tek's "I/O Maps" gives the rows marked [R]; the others are
//! the placements measured on the referee, or chosen where it shows nothing (Sapphire_Native.md §32.3,
//! Sapphire_Disputes.md D-260 to D-269).

/// Each core's block: core 0 from 000h, core 1 from 400h [R "... above addresses repeat for Core1 starting at
/// 1F900400h ..."].
pub const CORE_STRIDE: u32 = 0x400;

/// A voice's eight halfwords, 10h apart [R "Core0 Voice 0-23 registers", 180h]: VOLL, VOLR, PITCH, ADSR1, ADSR2, ENVX,
/// VOLXL, VOLXR.
pub mod voice {
    pub const VOLL: usize = 0;
    pub const VOLR: usize = 1;
    pub const PITCH: usize = 2;
    pub const ADSR1: usize = 3;
    pub const ADSR2: usize = 4;
    pub const ENVX: usize = 5;
    pub const VOLXL: usize = 6;
    pub const VOLXR: usize = 7;
}

/// The voices' three addresses, 0Ch apart from 1C0h [R "Core0 Voice 0-23 start/loop/next addresses", 120h], each a
/// pair of halfwords, high then low: SSA, LSAX, NAX.
pub const ADDRESSES: u32 = 0x1C0;
pub const ADDRESSES_END: u32 = 0x2E0;

pub const PMON: u32 = 0x180;
pub const NON: u32 = 0x184;
pub const VMIXL: u32 = 0x188;
pub const VMIXEL: u32 = 0x18C;
pub const VMIXR: u32 = 0x190;
pub const VMIXER: u32 = 0x194;
pub const MMIX: u32 = 0x198;
/// [R "Core attributes"].
pub const ATTR: u32 = 0x19A;
/// [R "Interrupt address H/L"].
pub const IRQAH: u32 = 0x19C;
pub const IRQAL: u32 = 0x19E;
pub const KON: u32 = 0x1A0;
pub const KOFF: u32 = 0x1A4;
/// [R "DMA transfer address H/L", "Internal transfer FIFO", "AutoDMA status"].
pub const TSAH: u32 = 0x1A8;
pub const TSAL: u32 = 0x1AA;
pub const DATA: u32 = 0x1AC;
pub const ADMA: u32 = 0x1B0;
/// The reverb's work area start, high and low, then its 22 offsets as pairs in psx-spx's order (dAPF1, dAPF2, mLSAME
/// .. mRAPF2), then the area's end, a high halfword alone.
pub const ESAH: u32 = 0x2E0;
pub const REVERB: u32 = 0x2E4;
pub const EEA: u32 = 0x33C;
/// [R "ENDX 0/1", "Status register"].
pub const ENDX: u32 = 0x340;
pub const STATX: u32 = 0x344;

/// Each core's volumes, 28h apart from 760h [R "Master Volume Left" at 1F900760h onward], by halfword: MVOLL, MVOLR,
/// EVOLL, EVOLR, AVOLL and AVOLR (the external input's [R "Core1 External Input Volume"]), BVOLL and BVOLR (the sound
/// data input's, by elimination [C]), MVOLXL, MVOLXR, then the reverb's ten coefficients (vIIR, vCOMB1-4, vWALL,
/// vAPF1-2, vLIN, vRIN).
pub const VOLUMES: u32 = 0x760;
pub const VOLUMES_STRIDE: u32 = 0x28;
pub mod vol {
    pub const MVOL: usize = 0;
    pub const EVOL: usize = 2;
    pub const AVOL: usize = 4;
    pub const BVOL: usize = 6;
    pub const MVOLX: usize = 8;
    pub const IIR: usize = 10;
    pub const COMB: usize = 11;
    pub const WALL: usize = 15;
    pub const APF: usize = 16;
    pub const IN: usize = 18;
}
pub const VOLUMES_END: u32 = VOLUMES + 2 * VOLUMES_STRIDE;

/// ATTR's bits the SPU2 acts on, the PlayStation SPU's places [A, psx-spx "ATTR"]: the reverb's writes and the IRQ
/// enable, with the noise's step and shift in bits 8-13; bits 15 and 14 gate nothing on the referee (D-262).
pub mod attr {
    pub const REVERB: u16 = 1 << 7;
    pub const IRQ: u16 = 1 << 6;
}

/// ADMA's bits: core 0's start on bit 0, core 1's on bit 1 [R "SPU2 AutoDMA"].
pub const fn adma_bit(core: usize) -> u16 {
    1 << core
}
