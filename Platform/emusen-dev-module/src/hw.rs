//! `hwcount`'s counters: perf_event_open for one window of a running program, counting every thread and sampling one.
//! Linux only; a count of the program's own threads needs no privilege. See `man hwcount`.

use std::ffi::c_void;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};

const PERF_TYPE_HARDWARE: u32 = 0;
const CYCLES: u64 = 0;
const INSTRUCTIONS: u64 = 1;
const BRANCHES: u64 = 4;
const BRANCH_MISSES: u64 = 5;
const FORMAT_TIMES_GROUP: u64 = 1 | 2 | 8;
const SAMPLE_IP: u64 = 1;
const FLAG_DISABLED: u64 = 1 << 0;
const FLAG_EXCLUDE_KERNEL: u64 = 1 << 5;
const FLAG_EXCLUDE_HV: u64 = 1 << 6;
const FLAG_WATERMARK: u64 = 1 << 14;
const IOC_ENABLE: libc::c_ulong = 0x2400;
const RECORD_LOST: u32 = 2;
const RECORD_SAMPLE: u32 = 9;
const RING_PAGES: usize = 256;

#[repr(C)]
#[derive(Default)]
struct Attr {
    kind: u32,
    size: u32,
    config: u64,
    sample_period: u64,
    sample_type: u64,
    read_format: u64,
    flags: u64,
    wakeup: u32,
    bp_type: u32,
    rest: [u64; 10],
}

fn open(attr: &mut Attr, tid: i32, group: i32) -> io::Result<i32> {
    attr.kind = PERF_TYPE_HARDWARE;
    attr.size = std::mem::size_of::<Attr>() as u32;
    // SAFETY: the attribute is a live perf_event_attr of the size it states.
    let fd = unsafe { libc::syscall(libc::SYS_perf_event_open, attr as *mut Attr as *mut c_void, tid, -1, group, 0) };
    if fd < 0 { Err(io::Error::last_os_error()) } else { Ok(fd as i32) }
}

fn enable(fd: i32) {
    // SAFETY: an ioctl without an argument on a descriptor this module opened.
    unsafe { libc::ioctl(fd, IOC_ENABLE, 0) };
}

/// One thread's totals, scaled by the time its group ran.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Totals {
    pub cycles: f64,
    pub instr: f64,
    pub branches: f64,
    pub misses: f64,
}

/// The group of four counters on one thread.
pub struct Counter {
    pub tid: i32,
    lead: i32,
}

impl Counter {
    pub fn open(tid: i32) -> io::Result<Counter> {
        let mut lead = -1;
        for config in [CYCLES, INSTRUCTIONS, BRANCHES, BRANCH_MISSES] {
            let flags = FLAG_EXCLUDE_KERNEL | FLAG_EXCLUDE_HV | if lead < 0 { FLAG_DISABLED } else { 0 };
            let mut attr = Attr { config, read_format: FORMAT_TIMES_GROUP, flags, ..Attr::default() };
            let fd = open(&mut attr, tid, lead)?;
            if lead < 0 {
                lead = fd;
            }
        }
        Ok(Counter { tid, lead })
    }

    pub fn start(&self) {
        enable(self.lead);
    }

    pub fn read(&self) -> io::Result<Totals> {
        let mut buf = [0u64; 7];
        // SAFETY: the buffer is writable for the size given.
        let n = unsafe { libc::read(self.lead, buf.as_mut_ptr() as *mut c_void, std::mem::size_of_val(&buf)) };
        if n <= 0 {
            return Err(io::Error::last_os_error());
        }
        let scale = if buf[2] == 0 { 0.0 } else { buf[1] as f64 / buf[2] as f64 };
        Ok(Totals { cycles: buf[3] as f64 * scale, instr: buf[4] as f64 * scale, branches: buf[5] as f64 * scale, misses: buf[6] as f64 * scale })
    }
}

/// Cycles and instructions sampled on one thread into mapped rings.
pub struct Sampler {
    rings: [Ring; 2],
}

struct Ring {
    fd: i32,
    base: *mut u8,
    tail: u64,
    page: usize,
}

// SAFETY: a ring is touched by one thread at a time, and the kernel's side is reached by atomics.
unsafe impl Send for Sampler {}

impl Ring {
    fn open(config: u64, tid: i32, period: u64) -> io::Result<Ring> {
        // SAFETY: sysconf has no preconditions.
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
        let flags = FLAG_DISABLED | FLAG_EXCLUDE_KERNEL | FLAG_EXCLUDE_HV | FLAG_WATERMARK;
        let mut attr = Attr { config, sample_period: period, sample_type: SAMPLE_IP, flags, wakeup: (64 * page) as u32, ..Attr::default() };
        let fd = open(&mut attr, tid, -1)?;
        // SAFETY: a fresh shared mapping of the event's ring, one header page and a power of two of data pages.
        let base = unsafe { libc::mmap(std::ptr::null_mut(), (RING_PAGES + 1) * page, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0) };
        if base == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        Ok(Ring { fd, base: base as *mut u8, tail: 0, page })
    }

    fn word(&self, at: u64) -> u64 {
        let size = (RING_PAGES * self.page) as u64;
        let mut bytes = [0u8; 8];
        for (i, b) in bytes.iter_mut().enumerate() {
            // SAFETY: the offset is reduced to the data pages' size.
            *b = unsafe { *self.base.add(self.page + ((at + i as u64) % size) as usize) };
        }
        u64::from_le_bytes(bytes)
    }

    /// Moves the ring's records to `out`, returning how many the kernel lost.
    fn drain(&mut self, instr: bool, out: &mut Vec<(u64, bool)>) -> u64 {
        // SAFETY: data_head and data_tail stand at offsets 1024 and 1032 of the ring's header page, perf_event_open(2).
        let (head, tail) = unsafe { (&*(self.base.add(1024) as *const AtomicU64), &*(self.base.add(1032) as *const AtomicU64)) };
        let end = head.load(Ordering::Acquire);
        let mut lost = 0;
        while self.tail < end {
            let header = self.word(self.tail);
            match header as u32 {
                RECORD_SAMPLE => out.push((self.word(self.tail + 8), instr)),
                RECORD_LOST => lost += self.word(self.tail + 16),
                _ => {}
            }
            self.tail += (header >> 48).max(8);
        }
        tail.store(self.tail, Ordering::Release);
        lost
    }
}

impl Sampler {
    pub fn open(tid: i32, period: u64) -> io::Result<Sampler> {
        Ok(Sampler { rings: [Ring::open(CYCLES, tid, period)?, Ring::open(INSTRUCTIONS, tid, period)?] })
    }

    pub fn start(&self) {
        self.rings.iter().for_each(|r| enable(r.fd));
    }

    pub fn drain(&mut self, out: &mut Vec<(u64, bool)>) -> u64 {
        let [cycles, instr] = &mut self.rings;
        cycles.drain(false, out) + instr.drain(true, out)
    }
}
