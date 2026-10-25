//! What the EE core sees of memory at S6: a bus of naturally aligned accesses by physical address, and the stand-in
//! for address translation until S7 builds the TLB (Sapphire_Native.md §16.3).

/// Naturally aligned reads and writes of 1, 2, 4, 8 or 16 bytes at a physical address, little-endian in the low bytes.
pub trait Bus {
    fn read(&mut self, paddr: u32, bytes: usize) -> u128;
    fn write(&mut self, paddr: u32, bytes: usize, value: u128);
}

/// kseg0 and kseg1 drop their top three bits (EE Core User's Manual §5.1.6); the mapped segments are taken the same
/// way until S7's TLB, a named stand-in.
pub fn physical(vaddr: u32) -> u32 {
    vaddr & 0x1FFF_FFFF
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
