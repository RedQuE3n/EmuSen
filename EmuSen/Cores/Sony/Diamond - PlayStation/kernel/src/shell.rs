//! What runs at 80030000h once the kernel is set up: the CD boot of psx-spx's "CDROM File Playstation EXE and
//! SYSTEM.CNF" (BOOT, TCB, EVENT and STACK, PSX.EXE without the file, the argument at 180h, _96_init, the program
//! loaded and run by Exec with SYSTEM.CNF's stack), the warm boot of A(A0h), and the shell program started when no
//! disc boots, under hand-over v1. Diamond_OpenKernel_Phase2.md §15, Diamond_Shell.md §1.3.

use crate::layout::{self, A_TABLE, SCRATCH};
use crate::mem::{call, rd, rd8, wr, wr8};
use crate::{blocks, cdrom, entry, exec, file};

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

/// What the drive holds, as the shell's hand-over names it in Exec's second parameter (Diamond_Shell.md §1.3).
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Disc {
    Empty = 0,
    Audio = 1,
    Refused = 2,
    Data = 3,
}

/// The drive asked by GetID until it answers, through the spin-up, its second response's flags read by bit as psx-spx's
/// GetID table gives them: missing (40h), audio (10h), denied (80h) with the ID error; a licensed data disc boots.
pub fn disc() -> Disc {
    for _ in 0..40 {
        match cdrom::get_id() {
            Ok(_) => return Disc::Data,
            Err([s, 0x80]) if s & 0x10 == 0 => pause(4),
            Err([_, f]) if f & 0x40 != 0 => return Disc::Empty,
            Err([_, f]) if f & 0x10 != 0 => return Disc::Audio,
            Err([s, 0x80]) if s & 0x08 != 0 => return Disc::Refused,
            Err(_) => return Disc::Empty,
        }
    }
    Disc::Empty
}

/// `frames` vblanks waited on I_STAT's bit, vblank being masked while the kernel's shell code runs.
fn pause(frames: u32) {
    for _ in 0..frames {
        wr(layout::io::I_STAT, !1);
        while rd(layout::io::I_STAT) & 1 == 0 {}
    }
}

/// The shell program (`kernel-shell`), a PS-EXE carried in the ROM at 1FC60000h in a section of its own, so that its
/// size moves nothing else of the image; Diamond_Shell.md §1.3, §11.4.
const SHELL_LEN: usize = include_bytes!("../../kernel-shell/diamond-shell.exe").len();
#[unsafe(link_section = ".shell")]
static SHELL: [u8; SHELL_LEN] = *include_bytes!("../../kernel-shell/diamond-shell.exe");
static mut SHELL_HEADER: [u32; 15] = [0; 15];

/// Hand-over v1: the shell copied to the RAM its header names and started by Exec(header, 1, reason); it returns 0 when
/// the drive holds a disc to boot, having stopped its pads and cards and masked vblank.
fn run_shell(reason: Disc) {
    let header = &raw mut SHELL_HEADER as u32;
    let rom = SHELL.as_ptr() as u32;
    for i in 0..0x3C {
        wr8(header + i, rd8(rom + 0x10 + i));
    }
    let (address, size) = (rd(header + 0x08), rd(header + 0x0C));
    for i in (0..size.min(SHELL.len() as u32 - 0x800)).step_by(4) {
        wr(address + i, rd(rom + 0x800 + i));
    }
    unsafe { entry::flush_cache() };
    wr(layout::io::I_MASK, 0);
    wr(layout::io::I_STAT, 0);
    unsafe { entry::exec(header, 1, reason as u32) };
    wr(layout::io::I_MASK, 0);
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
        match disc() {
            Disc::Data => start(settings()),
            other => run_shell(other),
        }
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
