//! Every fixed address the kernel uses, each from psx-spx: the kernel's RAM ("BIOS Memory Map"), the table of tables
//! ("BIOS Control Blocks"), and the I/O ports ("I/O Map" and the chapters it indexes). Diamond_OpenKernel.md §2.

/// The first 64 KiB of RAM, the kernel's.
pub const KERNEL_RAM: u32 = 0x8000_0000;
pub const KERNEL_RAM_SIZE: u32 = 0x1_0000;
/// 00000000h-0000000Fh: a copy of the exception vector, read by games through null pointers.
pub const GARBAGE: u32 = 0x8000_0000;
pub const RAM_SIZE_MB: u32 = 0x8000_0060;
pub const RAM_WORD_64: u32 = 0x8000_0064;
pub const RAM_WORD_68: u32 = 0x8000_0068;
pub const VECTOR_EXCEPTION: u32 = 0x8000_0080;
pub const VECTOR_A: u32 = 0x8000_00A0;
pub const VECTOR_B: u32 = 0x8000_00B0;
pub const VECTOR_C: u32 = 0x8000_00C0;
pub const TABLES: u32 = 0x8000_0100;
pub const A_TABLE: u32 = 0x8000_0200;
pub const A_ENTRIES: usize = 0xC0;
pub const B_ENTRIES: usize = 0x100;
pub const C_ENTRIES: usize = 0x80;
/// B(00h)'s control-block memory.
pub const KERNEL_MEMORY: u32 = 0xA000_E000;
pub const KERNEL_MEMORY_SIZE: u32 = 0x2000;
/// SYSTEM.CNF's defaults: EVENT = 10, TCB = 4, STACK = 801FFF00.
pub const DEFAULT_EVENTS: u32 = 0x10;
pub const DEFAULT_THREADS: u32 = 4;
pub const DEFAULT_STACKTOP: u32 = 0x801F_FF00;
/// Where the kernel starts the code that follows its own initialisation ("Mid-Boot Hook").
pub const SHELL_ENTRY: u32 = 0x8003_0000;
/// The scratchpad, which carries A(A0h)'s settings through the reset.
pub const SCRATCH: u32 = 0x1F80_0000;

/// The table of tables at 100h: each entry the address and the size in bytes of its control blocks.
pub mod table {
    pub const EXCB: u32 = 0x00;
    pub const PCB: u32 = 0x08;
    pub const TCB: u32 = 0x10;
    pub const EVCB: u32 = 0x20;
    pub const FCB: u32 = 0x40;
    pub const DCB: u32 = 0x50;
}

pub const EXCB_SIZE: u32 = 4 * 0x08;
pub const PCB_SIZE: u32 = 0x04;
pub const TCB_SIZE: u32 = 0xC0;
pub const EVCB_SIZE: u32 = 0x1C;
pub const FCB_SIZE: u32 = 0x2C;
pub const FCBS: u32 = 0x10;
pub const DCB_SIZE: u32 = 0x50;
pub const DCBS: u32 = 0x0A;

/// I/O, through KSEG1.
pub mod io {
    pub const DEV0_BASE: u32 = 0xBF80_1000;
    pub const DEV8_BASE: u32 = 0xBF80_1004;
    pub const DEV0_DELAY: u32 = 0xBF80_1008;
    pub const DEV1_DELAY: u32 = 0xBF80_100C;
    pub const BIOS_DELAY: u32 = 0xBF80_1010;
    pub const SPU_DELAY: u32 = 0xBF80_1014;
    pub const CDROM_DELAY: u32 = 0xBF80_1018;
    pub const DEV8_DELAY: u32 = 0xBF80_101C;
    pub const COM_DELAY: u32 = 0xBF80_1020;
    pub const RAM_SIZE: u32 = 0xBF80_1060;
    pub const I_STAT: u32 = 0xBF80_1070;
    pub const I_MASK: u32 = 0xBF80_1074;
    pub const D2_MADR: u32 = 0xBF80_10A0;
    pub const D2_BCR: u32 = 0xBF80_10A4;
    pub const D2_CHCR: u32 = 0xBF80_10A8;
    pub const DPCR: u32 = 0xBF80_10F0;
    pub const DICR: u32 = 0xBF80_10F4;
    pub const TIMER: u32 = 0xBF80_1100;
    pub const GP0: u32 = 0xBF80_1810;
    pub const GP1: u32 = 0xBF80_1814;
    /// The SCN2681 DUART on the expansion port: channel A's transmit holding register.
    pub const DUART_THRA: u32 = 0xBF80_2023;
    pub const BCC: u32 = 0xFFFE_0130;
}

/// The values psx-spx's "Memory Control" gives as usual, and BCC's bits it marks as usually set (RAM, DS, IBLKSZ=1,
/// IS1, RDPRI, NOPAD, BGNT, LDSCH).
pub mod setting {
    pub const DEV0_BASE: u32 = 0x1F00_0000;
    pub const DEV8_BASE: u32 = 0x1F80_2000;
    pub const DEV0_DELAY: u32 = 0x0013_243F;
    pub const DEV1_DELAY: u32 = 0x0000_3022;
    pub const BIOS_DELAY: u32 = 0x0013_243F;
    pub const SPU_DELAY: u32 = 0x2009_31E1;
    pub const CDROM_DELAY: u32 = 0x0002_0843;
    pub const DEV8_DELAY: u32 = 0x0007_0777;
    pub const COM_DELAY: u32 = 0x0003_1125;
    pub const BCC: u32 = 1 << 3 | 1 << 7 | 1 << 8 | 1 << 11 | 1 << 13 | 1 << 14 | 1 << 15 | 1 << 16;
}

/// The kernel's identity in the ROM header and GetSystemInfo: its BCD date and its maker string.
pub const KERNEL_DATE: u32 = 0x2026_1011;
pub const CONSOLE_TYPE: u32 = 3;
