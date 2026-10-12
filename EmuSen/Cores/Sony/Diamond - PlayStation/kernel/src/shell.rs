//! What runs at 80030000h once the kernel is set up: the CD boot of psx-spx's "CDROM File Playstation EXE and
//! SYSTEM.CNF" (BOOT, TCB, EVENT and STACK, PSX.EXE without the file, the argument at 180h, _96_init, the program
//! loaded and run by Exec with SYSTEM.CNF's stack), the warm boot of A(A0h), and the screen when no disc boots.
//! Diamond_OpenKernel.md §15.

use crate::layout::{self, A_TABLE, SCRATCH};
use crate::mem::{call, rd, rd8, wr, wr8};
use crate::{blocks, cdrom, entry, exec, file, screen};

/// The shell's stack, 1800h bytes in the kernel's RAM, so that a program loaded high cannot overwrite it.
#[unsafe(no_mangle)]
pub static mut SHELL_STACK: [u32; 0x600] = [0; 0x600];

/// k1 at the reset entry when A(A0h) asked for a warm boot.
const WARM: u32 = 0x5741_524D;

const NAME_MAX: u32 = 64;
const ARGUMENT: u32 = 0x8000_0180;
const ARGUMENT_MAX: u32 = 0x80;

/// The boot's settings: the program's name, then TCB, EVENT and STACK.
#[derive(Clone, Copy)]
struct Settings {
    name: [u8; NAME_MAX as usize],
    threads: u32,
    events: u32,
    stack: u32,
}

static mut SETTINGS: Settings = Settings { name: [0; NAME_MAX as usize], threads: 0, events: 0, stack: 0 };
static mut TEXT: [u32; 0x200] = [0; 0x200];
static mut HEADER: [u32; 15] = [0; 15];

#[derive(PartialEq, Eq)]
pub enum Disc {
    Data,
    Other,
}

/// The drive asked by GetID until it answers, through the spin-up: a licensed data disc, or anything else.
pub fn disc() -> Disc {
    for _ in 0..40 {
        match cdrom::get_id() {
            Ok(_) => return Disc::Data,
            Err([s, 0x80]) if s & 0x10 == 0 => screen::pause(4),
            Err(_) => return Disc::Other,
        }
    }
    Disc::Other
}

fn upper(c: u32) -> u32 {
    crate::convert::toupper(c) & 0xFF
}

fn hex(text: u32, mut at: u32) -> u32 {
    while matches!(rd8(text + at), 0x20 | 0x09) {
        at += 1;
    }
    let mut v = 0;
    while let Some(d) = (rd8(text + at) as u8 as char).to_digit(16) {
        v = v << 4 | d;
        at += 1;
    }
    v
}

/// SYSTEM.CNF's lines, "KEY = value": BOOT's name and the argument after it, and the hexadecimal TCB, EVENT and
/// STACK; the defaults for what it does not give.
fn parse(text: u32, length: u32) -> Settings {
    let mut s = Settings {
        name: [0; NAME_MAX as usize],
        threads: layout::DEFAULT_THREADS,
        events: layout::DEFAULT_EVENTS,
        stack: layout::DEFAULT_STACKTOP,
    };
    s.name[..15].copy_from_slice(b"cdrom:PSX.EXE;1");
    let mut line = 0;
    while line < length {
        let mut end = line;
        while end < length && !matches!(rd8(text + end), 0x0A | 0x0D | 0) {
            end += 1;
        }
        let mut key = [0u8; 8];
        let mut k = 0;
        let mut at = line;
        while at < end && rd8(text + at) != b'=' as u32 {
            let c = rd8(text + at);
            if c > 0x20 && k < 8 {
                key[k] = upper(c) as u8;
                k += 1;
            }
            at += 1;
        }
        if at < end {
            let value = at + 1;
            match &key[..k] {
                b"BOOT" => boot_line(&mut s, text, value, end),
                b"TCB" => s.threads = hex(text, value),
                b"EVENT" => s.events = hex(text, value),
                b"STACK" => s.stack = hex(text, value),
                _ => {}
            }
        }
        line = end + 1;
    }
    s
}

/// BOOT's name to the first blank, and what follows it copied to 180h as the program's argument.
fn boot_line(s: &mut Settings, text: u32, mut at: u32, end: u32) {
    while at < end && matches!(rd8(text + at), 0x20 | 0x09) {
        at += 1;
    }
    let mut n = 0;
    while at < end && !matches!(rd8(text + at), 0x20 | 0x09) {
        if n + 1 < NAME_MAX {
            s.name[n as usize] = rd8(text + at) as u8;
            n += 1;
        }
        at += 1;
    }
    s.name[n as usize..].fill(0);
    while at < end && matches!(rd8(text + at), 0x20 | 0x09) {
        at += 1;
    }
    for i in 0..ARGUMENT_MAX {
        wr8(ARGUMENT + i, if at + i < end && i + 1 < ARGUMENT_MAX { rd8(text + at + i) } else { 0 });
    }
}

/// SYSTEM.CNF read from the disc's root, or the defaults with PSX.EXE when it has none.
fn settings() -> Settings {
    let text = &raw mut TEXT as u32;
    for i in 0..0x200 {
        wr(text + 4 * i, 0);
    }
    let fd = file::open(b"cdrom:\\SYSTEM.CNF;1\0".as_ptr() as u32, 1);
    let mut n = 0;
    if fd != u32::MAX {
        n = file::read(fd, text, 0x800);
        file::close(fd);
        if n == u32::MAX {
            n = 0;
        }
    }
    parse(text, n)
}

/// The program loaded and run as the console boots it; returns only when it cannot be loaded.
fn start(s: Settings) {
    unsafe { SETTINGS = s };
    blocks::set_conf(s.events, s.threads, s.stack);
    cdrom::init_96();
    let header = &raw mut HEADER as u32;
    let name = unsafe { &raw const SETTINGS.name } as u32;
    if exec::load(name, header) == 0 {
        return;
    }
    wr(header + 0x20, s.stack);
    wr(header + 0x24, 0);
    wr(layout::io::I_MASK, 0);
    wr(layout::io::I_STAT, 0);
    unsafe { entry::exec(header, 1, 0) };
    call(rd(A_TABLE + 4 * 0xA1), b'B' as u32, 0x38C, 0, 0);
}

#[unsafe(no_mangle)]
extern "C" fn shell_main(k1: u32) -> ! {
    if k1 == WARM {
        let s = restore();
        cdrom::init_96();
        start(s);
    }
    loop {
        cdrom::init_96();
        if disc() == Disc::Data {
            start(settings());
        }
        screen::no_disc();
    }
}

/// A(A0h): the kernel set up again from its reset and the program SYSTEM.CNF named started again, without reading
/// SYSTEM.CNF; its settings carried through the reset in the scratchpad.
pub extern "C" fn boot() -> u32 {
    let s = unsafe { SETTINGS };
    let bytes = &raw const s as u32;
    for i in 0..core::mem::size_of::<Settings>() as u32 {
        wr8(SCRATCH + i, rd8(bytes + i));
    }
    unsafe { entry::reboot(WARM) }
}

fn restore() -> Settings {
    let mut s = Settings { name: [0; NAME_MAX as usize], threads: 0, events: 0, stack: 0 };
    let bytes = &raw mut s as u32;
    for i in 0..core::mem::size_of::<Settings>() as u32 {
        wr8(bytes + i, rd8(SCRATCH + i));
    }
    s
}
