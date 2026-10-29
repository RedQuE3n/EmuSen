//! Version 1.1, behind the `native` feature. See EmuSen_Recompiler.md §14.

use emusen_assembler::Code;

/// A block's tailored code, and the 8-byte data word, if any, that holds the Cranelift function's address (the tier below its entry's test falls back to).
pub struct Native {
    pub code: Code,
    pub fallback: Option<u32>,
}

/// The reservation: 1 GiB, so that every rel32 between the arenas and the stubs reaches.
const RESERVED: usize = 1 << 30;
const COLD: usize = RESERVED / 2;

pub(crate) struct Memory {
    base: *mut u8,
    page: usize,
    hot: usize,
    cold: usize,
    stubs: Vec<u64>,
}

// SAFETY: the memory is owned by the compiler's thread alone; the pointer is moved with it.
unsafe impl Send for Memory {}

fn up(n: usize, to: usize) -> usize {
    n.div_ceil(to) * to
}

impl Memory {
    pub(crate) fn new() -> Memory {
        // SAFETY: an anonymous reservation, no access until a batch is written.
        let base = unsafe { libc::mmap(std::ptr::null_mut(), RESERVED, libc::PROT_NONE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE, -1, 0) };
        assert!(base != libc::MAP_FAILED, "the code reservation");
        // SAFETY: sysconf has no preconditions.
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
        Memory { base: base.cast(), page, hot: 0, cold: COLD, stubs: Vec::new() }
    }

    /// One batch: the stubs, at the first, then each block; each block's entry, or none where it could not be linked or the reservation is full.
    pub(crate) fn place(&mut self, stubs: &dyn Fn() -> Vec<Code>, blocks: Vec<(Native, u64)>) -> Vec<Option<u64>> {
        let (hot0, cold0) = (self.hot, self.cold);
        let mut writes: Vec<(usize, Vec<u8>)> = Vec::new();
        if self.stubs.is_empty() {
            for mut s in stubs() {
                let at = self.base as u64 + self.hot as u64;
                let placed = &self.stubs;
                assert!(s.link(at, at + s.hot.len() as u64, at, &|id| placed[id as usize]), "a stub reaches the stubs before it");
                assert!(s.cold.is_empty() && s.data.is_empty(), "a stub is hot code alone");
                self.stubs.push(at + u64::from(s.entry));
                writes.push((self.hot, s.hot));
                self.hot = up(self.hot + writes.last().map_or(0, |w| w.1.len()), 16);
            }
        }
        let mut entries = Vec::with_capacity(blocks.len());
        for (mut n, fallback) in blocks {
            let need_hot = n.code.hot.len() + 16;
            let need_cold = n.code.cold.len() + n.code.data.len() + 32;
            if self.hot + need_hot >= COLD || self.cold + need_cold >= RESERVED {
                entries.push(None);
                continue;
            }
            let b = self.base as u64;
            let hot = b + self.hot as u64;
            let cold = b + self.cold as u64;
            let data = up(self.cold + n.code.cold.len(), 8);
            if let Some(k) = n.fallback {
                n.code.data[k as usize..k as usize + 8].copy_from_slice(&fallback.to_le_bytes());
            }
            let stubs = &self.stubs;
            if !n.code.link(hot, cold, b + data as u64, &|id| stubs[id as usize]) {
                entries.push(None);
                continue;
            }
            entries.push(Some(if n.code.entry_cold { cold } else { hot } + u64::from(n.code.entry)));
            writes.push((self.hot, std::mem::take(&mut n.code.hot)));
            writes.push((self.cold, std::mem::take(&mut n.code.cold)));
            writes.push((data, std::mem::take(&mut n.code.data)));
            self.hot = up(self.hot + need_hot - 16, 16);
            self.cold = up(data + writes.last().map_or(0, |w| w.1.len()), 16);
        }
        // W^X: the batch's fresh pages writable, written, then readable and executable; never written again.
        debug_assert!(hot0 % self.page == 0 && cold0 % self.page == 0, "a batch starts on a fresh page");
        let ranges = [(hot0, up(self.hot, self.page)), (cold0, up(self.cold, self.page))];
        for &(from, to) in &ranges {
            if to > from {
                // SAFETY: inside the reservation; the pages are this batch's alone, which nothing has run yet.
                assert_eq!(unsafe { libc::mprotect(self.base.add(from).cast(), to - from, libc::PROT_READ | libc::PROT_WRITE) }, 0, "writable");
            }
        }
        for (at, bytes) in writes {
            // SAFETY: inside the pages made writable above.
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.base.add(at), bytes.len()) };
        }
        for &(from, to) in &ranges {
            if to > from {
                // SAFETY: as above.
                assert_eq!(unsafe { libc::mprotect(self.base.add(from).cast(), to - from, libc::PROT_READ | libc::PROT_EXEC) }, 0, "executable");
            }
        }
        // Each batch starts on a fresh page, so that a published page is never writable again.
        self.hot = up(self.hot, self.page);
        self.cold = up(self.cold, self.page);
        entries
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        // SAFETY: the reservation made in `new`; the owner has forgotten every address in it.
        unsafe { libc::munmap(self.base.cast(), RESERVED) };
    }
}
