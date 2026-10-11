//! The threads of psx-spx's "BIOS Thread Functions": TCBs in the table of tables, a handle FF000000h plus the index,
//! switched by syscall 3. Diamond_OpenKernel.md §4.7.

use crate::blocks;
use crate::entry;
use crate::exception::tcb;
use crate::layout::{TABLES, TCB_SIZE, table};
use crate::mem::{rd, wr};

pub const FREE: u32 = 0x1000;
pub const USED: u32 = 0x4000;

fn tcb_at(index: u32) -> u32 {
    rd(TABLES + table::TCB) + index * TCB_SIZE
}

/// C(05h).
pub extern "C" fn get_free_tcb_slot() -> u32 {
    (0..blocks::threads()).find(|i| rd(tcb_at(*i)) == FREE).unwrap_or(u32::MAX)
}

/// B(0Eh): SR is left as the TCB held it, as documented.
pub extern "C" fn open_th(pc: u32, sp_fp: u32, gp: u32) -> u32 {
    let index = get_free_tcb_slot();
    if index == u32::MAX {
        return u32::MAX;
    }
    let t = tcb_at(index);
    wr(t, USED);
    wr(t + 4, 0x1000);
    wr(t + tcb::EPC, pc);
    wr(t + tcb::reg(29), sp_fp);
    wr(t + tcb::reg(30), sp_fp);
    wr(t + tcb::reg(28), gp);
    0xFF00_0000 | index
}

/// B(0Fh).
pub extern "C" fn close_th(handle: u32) -> u32 {
    if handle & 0xFFFF < blocks::threads() {
        wr(tcb_at(handle & 0xFFFF), FREE);
    }
    1
}

/// B(10h): the old thread gets 1 when it is changed back to.
pub extern "C" fn change_th(handle: u32) -> u32 {
    unsafe { entry::kernel_syscall(3, tcb_at(handle & 0xFFFF)) }
}
