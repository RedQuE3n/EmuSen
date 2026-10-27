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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_space_reads_zero_and_drops_writes_past_its_end() {
        let mut mem = [1u8, 2, 3, 4];
        let mut out = [9u8; 4];
        read_bytes(&mem, 2, &mut out);
        assert_eq!(out, [3, 4, 0, 0]);
        write_bytes(&mut mem, 3, &[7, 8, 9]);
        assert_eq!(mem, [1, 2, 3, 7]);
        read_bytes(&mem, u32::MAX, &mut out);
        assert_eq!(out, [0; 4]);
    }
}
