//! Reads and writes of a plain memory space from outside: past its end a read gives 0 and a write is dropped.

/// `out` filled from `mem` at `address`, 0 past its end.
pub fn read_bytes(mem: &[u8], address: u32, out: &mut [u8]) {
    for (i, b) in out.iter_mut().enumerate() {
        *b = mem.get(address as usize + i).copied().unwrap_or(0);
    }
}

/// `data` put into `mem` at `address`, dropped past its end.
pub fn write_bytes(mem: &mut [u8], address: u32, data: &[u8]) {
    for (i, &b) in data.iter().enumerate() {
        if let Some(d) = mem.get_mut(address as usize + i) {
            *d = b;
        }
    }
}
