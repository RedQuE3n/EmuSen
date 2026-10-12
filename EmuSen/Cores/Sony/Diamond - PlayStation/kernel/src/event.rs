//! The events of psx-spx's "BIOS Event Functions": EvCBs in the table of tables, a handle F1000000h plus the index.
//! Diamond_OpenKernel.md §4.6.

use crate::blocks;
use crate::layout::{EVCB_SIZE, TABLES, table};
use crate::mem::{call, rd, wr};

pub const FREE: u32 = 0;
pub const DISABLED: u32 = 0x1000;
pub const BUSY: u32 = 0x2000;
pub const READY: u32 = 0x4000;
pub const MODE_CALLBACK: u32 = 0x1000;
pub const MODE_READY: u32 = 0x2000;

const CLASS: u32 = 0x00;
const STATUS: u32 = 0x04;
const SPEC: u32 = 0x08;
const MODE: u32 = 0x0C;
const FUNC: u32 = 0x10;

#[inline(always)]
fn evcb(index: u32) -> u32 {
    rd(TABLES + table::EVCB) + index * EVCB_SIZE
}

fn of(handle: u32) -> Option<u32> {
    let index = handle & 0xFFFF;
    (index < blocks::events()).then(|| evcb(index))
}

/// C(04h): the first free EvCB's index, or FFFFFFFFh.
pub extern "C" fn get_free_evcb_slot() -> u32 {
    (0..blocks::events()).find(|i| rd(evcb(*i) + STATUS) == FREE).unwrap_or(u32::MAX)
}

/// B(08h): opened disabled.
pub extern "C" fn open_event(class: u32, spec: u32, mode: u32, func: u32) -> u32 {
    let index = get_free_evcb_slot();
    if index == u32::MAX {
        return u32::MAX;
    }
    let e = evcb(index);
    wr(e + CLASS, class);
    wr(e + STATUS, DISABLED);
    wr(e + SPEC, spec);
    wr(e + MODE, mode);
    wr(e + FUNC, func);
    0xF100_0000 | index
}

/// B(09h).
pub extern "C" fn close_event(handle: u32) -> u32 {
    if let Some(e) = of(handle) {
        wr(e + STATUS, FREE);
    }
    1
}

/// B(0Ch): a disabled event becomes busy; a ready one stays ready.
pub extern "C" fn enable_event(handle: u32) -> u32 {
    if let Some(e) = of(handle)
        && rd(e + STATUS) == DISABLED
    {
        wr(e + STATUS, BUSY);
    }
    1
}

/// B(0Dh).
pub extern "C" fn disable_event(handle: u32) -> u32 {
    if let Some(e) = of(handle)
        && rd(e + STATUS) != FREE
    {
        wr(e + STATUS, DISABLED);
    }
    1
}

/// B(0Ah): 0 if disabled, else waits for ready, returns 1 and leaves it busy.
pub extern "C" fn wait_event(handle: u32) -> u32 {
    let Some(e) = of(handle) else { return 0 };
    loop {
        match rd(e + STATUS) {
            READY => {
                wr(e + STATUS, BUSY);
                return 1;
            }
            BUSY => {}
            _ => return 0,
        }
    }
}

/// B(0Bh).
pub extern "C" fn test_event(handle: u32) -> u32 {
    match of(handle) {
        Some(e) if rd(e + STATUS) == READY => {
            wr(e + STATUS, BUSY);
            1
        }
        _ => 0,
    }
}

/// B(07h) for the exception path, in the kernel's RAM (Diamond_OpenKernel_Phase2.md §25.3).
#[unsafe(link_section = ".ramtext")]
pub extern "C" fn deliver_event_ram(class: u32, spec: u32) -> u32 {
    deliver(class, spec)
}

/// B(07h), the table's entry, for the kernel's code in the ROM.
pub extern "C" fn deliver_event(class: u32, spec: u32) -> u32 {
    deliver(class, spec)
}

/// Every busy event of that class and spec, by its mode, made ready or its function called.
#[inline(always)]
fn deliver(class: u32, spec: u32) -> u32 {
    for i in 0..blocks::events() {
        let e = evcb(i);
        if rd(e + STATUS) != BUSY || rd(e + CLASS) != class || rd(e + SPEC) != spec {
            continue;
        }
        match rd(e + MODE) {
            MODE_READY => wr(e + STATUS, READY),
            MODE_CALLBACK => {
                let f = rd(e + FUNC);
                if f != 0 {
                    call(f, class, spec, 0, 0);
                }
            }
            _ => {}
        }
    }
    0
}

/// B(20h).
pub extern "C" fn undeliver_event(class: u32, spec: u32) -> u32 {
    for i in 0..blocks::events() {
        let e = evcb(i);
        if rd(e + STATUS) == READY && rd(e + MODE) == MODE_READY && rd(e + CLASS) == class && rd(e + SPEC) == spec {
            wr(e + STATUS, BUSY);
        }
    }
    0
}
