//! The tailored code's memory, behind the `native` feature: a code memory a dispatcher places blocks in one at a time. See EmuSen_Jit.md §9.

use emusen_assembler::Code;

/// The reservation: 1 GiB, so that every rel32 between the arenas and the stubs reaches.
const RESERVED: usize = 1 << 30;
const COLD: usize = RESERVED / 2;
/// The data arena: writable and never executable, so that a block's links are written without touching code pages.
const DATA: usize = COLD + RESERVED / 4;

/// A code memory, owned by the one thread that places in it and runs its code.
pub struct Memory {
    base: *mut u8,
    page: usize,
    hot: usize,
    cold: usize,
    data: usize,
    stubs: Vec<u64>,
}

// SAFETY: the memory has one owner at a time; the pointer moves with it.
unsafe impl Send for Memory {}

fn up(n: usize, to: usize) -> usize {
    n.div_ceil(to) * to
}

impl Memory {
    /// A memory with `stubs` placed first, each reached by its index; a stub reaches the stubs before it.
    pub fn new(stubs: Vec<Code>) -> Memory {
        let (base, page) = os::reserve(RESERVED);
        // SAFETY: the data arena, inside the reservation, from which nothing is ever run.
        unsafe { os::protect(base.add(DATA), RESERVED - DATA, false) };
        let mut m = Memory { base, page, hot: 0, cold: COLD, data: DATA, stubs: Vec::new() };
        for s in stubs {
            assert!(s.cold.is_empty() && s.data.is_empty(), "a stub is hot code alone");
            let e = m.place(s).expect("the stubs fit an empty memory");
            m.stubs.push(e);
        }
        m
    }

    /// Stub `id`'s entry.
    pub fn stub(&self, id: u32) -> u64 {
        self.stubs[id as usize]
    }

    /// Bytes held.
    pub fn held(&self) -> u64 {
        (self.hot + (self.cold - COLD) + (self.data - DATA)) as u64
    }

    /// The code placed after what is there, its entry; none when it does not fit or cannot be linked. No code of this memory may be running: its pages are writable, never executable, while they are written.
    pub fn place(&mut self, mut code: Code) -> Option<u64> {
        let (hot, cold, data) = (self.hot, self.cold, up(self.data, 8));
        if hot + code.hot.len() + 16 >= COLD || cold + code.cold.len() + 16 >= DATA || data + code.data.len() >= RESERVED {
            return None;
        }
        let b = self.base as u64;
        let stubs = &self.stubs;
        if !code.link(b + hot as u64, b + cold as u64, b + data as u64, &|id| stubs[id as usize]) {
            return None;
        }
        let entry = if code.entry_cold { b + cold as u64 } else { b + hot as u64 } + u64::from(code.entry);
        self.write(hot, &code.hot);
        self.write(cold, &code.cold);
        // SAFETY: inside the data arena, always writable.
        unsafe { std::ptr::copy_nonoverlapping(code.data.as_ptr(), self.base.add(data), code.data.len()) };
        self.hot = up(hot + code.hot.len(), 16);
        self.cold = up(cold + code.cold.len(), 16);
        self.data = data + code.data.len();
        Some(entry)
    }

    /// `bytes` at `at`, the pages they fall on writable for the copy and executable after: W^X with the block laid beside the last, so that a block costs its bytes, not two pages.
    fn write(&mut self, at: usize, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        let from = at / self.page * self.page;
        let to = up(at + bytes.len(), self.page);
        // SAFETY: inside the reservation; the owner runs nothing from these pages until the second call.
        unsafe {
            os::protect(self.base.add(from), to - from, false);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.base.add(at), bytes.len());
            os::protect(self.base.add(from), to - from, true);
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        // SAFETY: the reservation made in `new`; the owner has forgotten every address in it.
        unsafe { os::release(self.base, RESERVED) };
    }
}

/// The code memory's reservation and protection: Unix's calls; elsewhere the tailored code is never placed, and asking panics.
#[cfg(unix)]
mod os {
    pub(super) fn reserve(bytes: usize) -> (*mut u8, usize) {
        // SAFETY: an anonymous reservation, no access until a batch is written.
        let base = unsafe { libc::mmap(std::ptr::null_mut(), bytes, libc::PROT_NONE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE, -1, 0) };
        assert!(base != libc::MAP_FAILED, "the code reservation");
        // SAFETY: sysconf has no preconditions.
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
        (base.cast(), page)
    }

    /// # Safety
    /// `at..at + bytes` is inside a reservation and pages no code runs from while they are writable.
    pub(super) unsafe fn protect(at: *mut u8, bytes: usize, exec: bool) {
        let p = if exec { libc::PROT_READ | libc::PROT_EXEC } else { libc::PROT_READ | libc::PROT_WRITE };
        // SAFETY: the caller's.
        assert_eq!(unsafe { libc::mprotect(at.cast(), bytes, p) }, 0, "the code's protection");
    }

    /// # Safety
    /// A reservation of `bytes` from `reserve`, whose addresses nothing holds.
    pub(super) unsafe fn release(base: *mut u8, bytes: usize) {
        // SAFETY: the caller's.
        unsafe { libc::munmap(base.cast(), bytes) };
    }
}

#[cfg(not(unix))]
mod os {
    pub(super) fn reserve(_: usize) -> (*mut u8, usize) {
        panic!("the tailored code memory is made on Unix hosts only")
    }
    pub(super) unsafe fn protect(_: *mut u8, _: usize, _: bool) {}
    pub(super) unsafe fn release(_: *mut u8, _: usize) {}
}
