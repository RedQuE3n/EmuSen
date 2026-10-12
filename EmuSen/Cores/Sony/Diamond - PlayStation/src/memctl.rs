//! Memory control (psx-spx "Memory Control"): the expansion bases, each device's delay and size register, the common
//! delays and the RAM size register, and the access times they give (Diamond_Native.md §11.3, D-4).

/// 1F801000h..1F801020h: DEV0 base, DEV8 base, the delay/size registers of DEV0, DEV1, DEV2 (BIOS), DEV4 (SPU), DEV5
/// (CD-ROM) and DEV8, and COM_DELAY.
pub const DEV0_BASE: usize = 0;
pub const DEV8_BASE: usize = 1;
pub const DEV0: usize = 2;
pub const DEV1: usize = 3;
pub const DEV2: usize = 4;
pub const DEV4: usize = 5;
pub const DEV5: usize = 6;
pub const DEV8: usize = 7;
pub const COM: usize = 8;

/// The values the kernel leaves (psx-spx "I/O Map"), which a program started with no kernel finds.
pub const KERNEL_VALUES: [u32; 9] = [0x1F00_0000, 0x1F80_2000, 0x0013_243F, 0x0000_3022, 0x0013_243F, 0x2009_31E1, 0x0002_0843, 0x0007_0777, 0x0003_1125];
pub const KERNEL_RAM_SIZE: u32 = 0x0000_0B88;

/// One device's access, in CPU cycles: the first bus cycle and each further one, and whether the bus is 16 bits wide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    pub first: u32,
    pub seq: u32,
    pub wide: bool,
    /// Whether the address advances between the bus cycles of one access; without it every cycle meets one address.
    pub increment: bool,
}

impl Timing {
    /// The cycles of an access of `bytes`, from its first bus cycle to its last.
    pub fn cycles(self, bytes: u32) -> u32 {
        let units = if self.wide { bytes.div_ceil(2) } else { bytes };
        self.first + (units.max(1) - 1) * self.seq
    }
}

#[derive(Clone, Debug)]
pub struct MemControl {
    pub regs: [u32; 9],
    ram_size: u32,
    /// Both banks' bytes as `ram_banks` gives them, kept with the register since every access asks.
    ram_window: u32,
    reads: [Timing; 9],
    writes: [Timing; 9],
}

impl MemControl {
    pub fn new(regs: [u32; 9], ram_size: u32) -> MemControl {
        let mut m = MemControl { regs, ram_size: 0, ram_window: 0, reads: [Timing { first: 0, seq: 0, wide: false, increment: false }; 9], writes: [Timing { first: 0, seq: 0, wide: false, increment: false }; 9] };
        m.set_ram_size(ram_size);
        m.retime();
        m
    }

    pub fn ram_size(&self) -> u32 {
        self.ram_size
    }

    pub fn set_ram_size(&mut self, value: u32) {
        self.ram_size = value;
        let (ras0, ras1) = self.ram_banks();
        self.ram_window = ras0 + ras1;
    }

    /// The bytes of RAM both banks map.
    #[inline(always)]
    pub fn ram_window(&self) -> u32 {
        self.ram_window
    }

    pub fn read(&self, index: usize) -> u32 {
        self.regs[index]
    }

    pub fn write(&mut self, index: usize, value: u32) {
        self.regs[index] = match index {
            DEV0_BASE | DEV8_BASE => 0x1F00_0000 | value & 0x00FF_FFFF,
            COM => value & 0xFFFF,
            _ => value & 0xAF1F_FFFF,
        };
        self.retime();
    }

    fn retime(&mut self) {
        let com = self.regs[COM];
        for d in DEV0..=DEV8 {
            let r = self.regs[d];
            self.reads[d] = timing(r, r >> 4 & 0xF, com);
            self.writes[d] = timing(r, r & 0xF, com);
        }
    }

    pub fn read_timing(&self, device: usize) -> Timing {
        self.reads[device]
    }

    pub fn write_timing(&self, device: usize) -> Timing {
        self.writes[device]
    }

    /// The window a device decodes: 1 << N bytes (bits 16-20).
    pub fn window(&self, device: usize) -> u32 {
        1u32 << (self.regs[device] >> 16 & 0x1F).min(23)
    }

    /// The RAM banks the RAM size register maps (psx-spx "1F801060h"): /RAS0's size, and /RAS1's when enabled.
    pub fn ram_banks(&self) -> (u32, u32) {
        let r = self.ram_size;
        let bit0 = r >> 11 & 1;
        let ras0 = 0x10_0000 << ((r >> 9 & 1) << 1 | bit0);
        let ras1 = if r & 1 << 10 != 0 { 0x10_0000 << ((r >> 8 & 1) << 1 | bit0) } else { 0 };
        (ras0, ras1)
    }
}

/// A delay register's first and later bus cycles (Diamond_Disputes.md D-4): psx-spx's formula is not followed where the
/// console's `access-time` log contradicts it.
fn timing(r: u32, delay: u32, com: u32) -> Timing {
    let recovery = if r & 1 << 8 != 0 { com & 0xF } else { 0 };
    let floating = if r & 1 << 10 != 0 { com >> 8 & 0xF } else { 0 };
    let at = delay + 1;
    Timing { first: at + 3, seq: at + 1 + recovery + floating, wide: r & 1 << 12 != 0, increment: r & 1 << 13 != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The kernel's registers give the console's `access-time` figures (ps1-tests' psx.log, D-4), as whole cycles.
    #[test]
    fn the_kernel_values_give_the_consoles_access_times() {
        let m = MemControl::new(KERNEL_VALUES, KERNEL_RAM_SIZE);
        let t = |d: usize| [1, 2, 4].map(|b| m.read_timing(d).cycles(b));
        assert_eq!(t(DEV2), [7, 13, 25], "BIOS");
        assert_eq!(t(DEV0), [7, 13, 25], "DEV0");
        assert_eq!(t(DEV8), [11, 26, 56], "DEV8");
        assert_eq!(t(DEV1), [6, 6, 10], "DEV1");
        assert_eq!(t(DEV5), [8, 14, 26], "CD-ROM");
        assert_eq!(t(DEV4), [18, 18, 39], "SPU");
        assert_eq!(m.window(DEV2), 0x8_0000);
        assert_eq!(m.window(DEV8), 0x80);
        assert_eq!(m.ram_banks(), (0x80_0000, 0));
        let mut two = m.clone();
        two.set_ram_size(0x888);
        assert_eq!(two.ram_banks(), (0x20_0000, 0));
        assert_eq!((m.ram_window(), two.ram_window()), (0x80_0000, 0x20_0000), "the window kept with the register");
    }
}
