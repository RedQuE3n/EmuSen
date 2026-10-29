//! What the EE core sees past its caches: a bus of naturally aligned accesses by physical address. `map` is the EE's
//! physical memory map; `Ram` is RDRAM alone, for the tests.

/// Naturally aligned reads and writes of 1, 2, 4, 8 or 16 bytes at a physical address, little-endian in the low bytes.
pub trait Bus {
    fn read(&mut self, paddr: u32, bytes: usize) -> u128;
    fn write(&mut self, paddr: u32, bytes: usize, value: u128);
    /// Whether an access at `paddr` asserts a bus error (EE User's Manual §2.1.2).
    fn error(&mut self, _paddr: u32) -> bool {
        false
    }
    /// The vector units COP2 reaches, where the bus has them (from S12); none halts a COP2 instruction.
    fn vpu(&mut self) -> Option<&mut crate::vu::Vpu> {
        None
    }
    /// `vpu` for an instruction that reaches VU0 alone, which need not bring the rest of the board (Sapphire_Speed.md §62).
    fn vpu0(&mut self) -> Option<&mut crate::vu::Vpu> {
        self.vpu()
    }
}

/// RDRAM alone, 32 MiB from physical 0, for the interpreter's tests and the project's programs; elsewhere reads 0.
pub struct Ram {
    pub bytes: Vec<u8>,
}

impl Ram {
    pub const SIZE: usize = 32 << 20;

    pub fn new() -> Ram {
        Ram { bytes: vec![0; Self::SIZE] }
    }

    pub fn load(&mut self, at: u32, words: &[u32]) {
        for (i, w) in words.iter().enumerate() {
            let a = at as usize + i * 4;
            self.bytes[a..a + 4].copy_from_slice(&w.to_le_bytes());
        }
    }

    pub fn word(&self, at: u32) -> u32 {
        let a = at as usize;
        u32::from_le_bytes(self.bytes[a..a + 4].try_into().unwrap())
    }
}

impl Default for Ram {
    fn default() -> Self {
        Ram::new()
    }
}

impl Bus for Ram {
    fn read(&mut self, paddr: u32, bytes: usize) -> u128 {
        let a = paddr as usize;
        if a + bytes > self.bytes.len() {
            return 0;
        }
        let mut v = [0u8; 16];
        v[..bytes].copy_from_slice(&self.bytes[a..a + bytes]);
        u128::from_le_bytes(v)
    }

    fn write(&mut self, paddr: u32, bytes: usize, value: u128) {
        let a = paddr as usize;
        if a + bytes <= self.bytes.len() {
            self.bytes[a..a + bytes].copy_from_slice(&value.to_le_bytes()[..bytes]);
        }
    }
}
