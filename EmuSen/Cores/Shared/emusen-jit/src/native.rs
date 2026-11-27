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
    rw: *mut u8,
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
        let (base, rw, page) = os::reserve(RESERVED, DATA);
        let mut m = Memory { base, rw, page, hot: 0, cold: COLD, data: DATA, stubs: Vec::new() };
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

    /// `bytes` written at `at` through the writable alias when dual-mapped, else via page-protection toggling.
    fn write(&mut self, at: usize, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        if !self.rw.is_null() {
            // SAFETY: `at..at + bytes.len()` lies inside the shared `0..DATA` mapping; `base` remains RX.
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.rw.add(at), bytes.len());
                os::flush_icache(self.base.add(at), bytes.len());
            }
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
        unsafe { os::release(self.base, self.rw, RESERVED, DATA) };
    }
}

/// The code memory's reservation and protection: Linux dual-mapped W^X via memfd, Unix fallback elsewhere.
#[cfg(target_os = "linux")]
mod os {
    pub(super) fn reserve(bytes: usize, data_offset: usize) -> (*mut u8, *mut u8, usize) {
        let fd = unsafe { libc::memfd_create(c"emusen-jit".as_ptr(), libc::MFD_CLOEXEC) };
        assert!(fd >= 0, "the code memfd");
        assert_eq!(unsafe { libc::ftruncate(fd, bytes as libc::off_t) }, 0, "the code memfd size");
        // SAFETY: `0..data_offset` stays R-X and `data_offset..bytes` becomes RW- for the data arena.
        let rx = unsafe { libc::mmap(std::ptr::null_mut(), bytes, libc::PROT_READ | libc::PROT_EXEC, libc::MAP_SHARED, fd, 0) };
        assert!(rx != libc::MAP_FAILED, "the code rx mapping");
        assert_eq!(unsafe { libc::mprotect(rx.cast::<u8>().add(data_offset).cast(), bytes - data_offset, libc::PROT_READ | libc::PROT_WRITE) }, 0, "the data arena protection");
        let rw = unsafe { libc::mmap(std::ptr::null_mut(), data_offset, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0) };
        assert!(rw != libc::MAP_FAILED, "the code rw mapping");
        unsafe { libc::close(fd) };
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
        (rx.cast(), rw.cast(), page)
    }

    pub(super) unsafe fn protect(_at: *mut u8, _bytes: usize, _exec: bool) {}

    pub(super) unsafe fn flush_icache(at: *mut u8, bytes: usize) {
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        unsafe {
            unsafe extern "C" {
                fn __clear_cache(start: *mut libc::c_void, end: *mut libc::c_void);
            }
            __clear_cache(at.cast(), at.add(bytes).cast());
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let _ = (at, bytes);
    }

    pub(super) unsafe fn release(base: *mut u8, rw: *mut u8, bytes: usize, rw_bytes: usize) {
        unsafe {
            if !rw.is_null() {
                libc::munmap(rw.cast(), rw_bytes);
            }
            libc::munmap(base.cast(), bytes);
        }
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
mod os {
    pub(super) fn reserve(bytes: usize, data_offset: usize) -> (*mut u8, *mut u8, usize) {
        let base = unsafe { libc::mmap(std::ptr::null_mut(), bytes, libc::PROT_NONE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE, -1, 0) };
        assert!(base != libc::MAP_FAILED, "the code reservation");
        unsafe { protect(base.cast::<u8>().add(data_offset), bytes - data_offset, false) };
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
        (base.cast(), std::ptr::null_mut(), page)
    }

    pub(super) unsafe fn protect(at: *mut u8, bytes: usize, exec: bool) {
        let p = if exec { libc::PROT_READ | libc::PROT_EXEC } else { libc::PROT_READ | libc::PROT_WRITE };
        assert_eq!(unsafe { libc::mprotect(at.cast(), bytes, p) }, 0, "the code's protection");
    }

    pub(super) unsafe fn flush_icache(_at: *mut u8, _bytes: usize) {}

    pub(super) unsafe fn release(base: *mut u8, _rw: *mut u8, bytes: usize, _rw_bytes: usize) {
        unsafe { libc::munmap(base.cast(), bytes) };
    }
}

#[cfg(not(unix))]
mod os {
    pub(super) fn reserve(_: usize, _: usize) -> (*mut u8, *mut u8, usize) {
        panic!("the tailored code memory is made on Unix hosts only")
    }
    pub(super) unsafe fn protect(_: *mut u8, _: usize, _: bool) {}
    pub(super) unsafe fn flush_icache(_: *mut u8, _: usize) {}
    pub(super) unsafe fn release(_: *mut u8, _: *mut u8, _: usize, _: usize) {}
}
