//! The EE's physical memory map (EE User's Manual §2.1, its Figure; ps2tek "Memory Map"): 32 MiB of RDRAM at 0, the
//! EE's registers from 10000000h, the GS's from 12000000h, the 4 MiB boot ROM at 1FC00000h, and the bus errors of
//! §2.1.2. The registers belong to later stages and read as zero here; Sapphire_Native.md §17.4 lists each region.

use super::Bus;

pub const RDRAM: u32 = 32 << 20;
pub const BOOT_ROM: u32 = 0x1FC0_0000;
pub const BOOT_ROM_SIZE: u32 = 4 << 20;

/// What a physical address reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Rdram,
    /// Main memory's window past the installed 32 MiB, up to 10000000h.
    Unpopulated,
    /// The EE's registers, VU memories and the GS's registers: S11 to S16.
    Registers,
    Reserved,
    BootRom,
    /// §2.1.2: 20000000h to 3FFFFFF0h and 80000000h to FFFFFFF0h assert BUSERR*.
    BusError,
    /// 40000000h to 7FFFFFFFh, "Extend Main Memory" on the Figure, absent on a retail console.
    Extend,
}

pub fn region(paddr: u32) -> Region {
    match paddr {
        0..RDRAM => Region::Rdram,
        RDRAM..0x1000_0000 => Region::Unpopulated,
        0x1000_0000..0x1400_0000 => Region::Registers,
        BOOT_ROM..0x2000_0000 => Region::BootRom,
        0x1400_0000..BOOT_ROM => Region::Reserved,
        0x2000_0000..0x4000_0000 | 0x8000_0000.. => Region::BusError,
        0x4000_0000..0x8000_0000 => Region::Extend,
    }
}

/// The map with RDRAM and the boot ROM's bytes; the ROM is empty unless an image is given (none at this stage).
pub struct Map {
    pub rdram: Vec<u8>,
    pub rom: Vec<u8>,
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}

impl Map {
    pub fn new() -> Map {
        Map { rdram: vec![0; RDRAM as usize], rom: Vec::new() }
    }

    pub fn load(&mut self, at: u32, words: &[u32]) {
        for (i, w) in words.iter().enumerate() {
            let a = at as usize + i * 4;
            self.rdram[a..a + 4].copy_from_slice(&w.to_le_bytes());
        }
    }

    pub fn word(&self, at: u32) -> u32 {
        let a = at as usize;
        u32::from_le_bytes(self.rdram[a..a + 4].try_into().unwrap())
    }
}

impl Bus for Map {
    fn read(&mut self, paddr: u32, bytes: usize) -> u128 {
        let (mem, at) = match region(paddr) {
            Region::Rdram => (&self.rdram, paddr as usize),
            Region::BootRom => (&self.rom, (paddr - BOOT_ROM) as usize),
            _ => return 0,
        };
        let mut v = [0u8; 16];
        if at + bytes <= mem.len() {
            v[..bytes].copy_from_slice(&mem[at..at + bytes]);
        }
        u128::from_le_bytes(v)
    }

    fn write(&mut self, paddr: u32, bytes: usize, value: u128) {
        if region(paddr) == Region::Rdram {
            let a = paddr as usize;
            self.rdram[a..a + bytes].copy_from_slice(&value.to_le_bytes()[..bytes]);
        }
    }

    fn error(&mut self, paddr: u32) -> bool {
        region(paddr) == Region::BusError
    }
}
