//! The file layer of psx-spx's "BIOS File Functions": sixteen FCBs and ten DCBs at fixed places in the kernel's
//! variables, a name's device found by its DCB, each call passed to the device's function. Phase 1 installs the TTY
//! only; the CD and the memory cards are phase 2, so their names are unknown devices. Diamond_OpenKernel.md §4.9.

use crate::layout::{DCB_SIZE, DCBS, FCB_SIZE, FCBS, TABLES, table};
use crate::mem::{call, rd, rd8, wr};
use crate::{printf, string, tty};

pub mod fcb {
    pub const STATUS: u32 = 0x00;
    pub const PORT: u32 = 0x04;
    pub const ADDRESS: u32 = 0x08;
    pub const LENGTH: u32 = 0x0C;
    pub const POSITION: u32 = 0x10;
    pub const FLAGS: u32 = 0x14;
    pub const ERROR: u32 = 0x18;
    pub const DCB: u32 = 0x1C;
    pub const NUMBER: u32 = 0x28;
}

pub mod dcb {
    pub const NAME: u32 = 0x00;
    pub const FLAGS: u32 = 0x04;
    pub const SECTOR: u32 = 0x08;
    pub const LONG_NAME: u32 = 0x0C;
    pub const INIT: u32 = 0x10;
    pub const OPEN: u32 = 0x14;
    pub const IN_OUT: u32 = 0x18;
    pub const CLOSE: u32 = 0x1C;
    pub const IOCTL: u32 = 0x20;
    pub const READ: u32 = 0x24;
    pub const WRITE: u32 = 0x28;
    pub const ERASE: u32 = 0x2C;
    pub const UNDELETE: u32 = 0x30;
    pub const FIRSTFILE: u32 = 0x34;
    pub const NEXTFILE: u32 = 0x38;
    pub const FORMAT: u32 = 0x3C;
    pub const CD: u32 = 0x40;
    pub const RENAME: u32 = 0x44;
    pub const TESTDEVICE: u32 = 0x4C;
}

/// The error numbers of psx-spx's "File Error Numbers".
pub mod error {
    pub const BAD_PORT: u32 = 0x06;
    pub const BAD_HANDLE: u32 = 0x09;
    pub const OTHER_DEVICE: u32 = 0x12;
    pub const UNKNOWN_DEVICE: u32 = 0x13;
    pub const INVALID: u32 = 0x16;
    pub const NO_HANDLES: u32 = 0x18;
}

const FAIL: u32 = u32::MAX;

static mut FCB_BLOCKS: [u32; (FCBS * FCB_SIZE / 4) as usize] = [0; (FCBS * FCB_SIZE / 4) as usize];
static mut DCB_BLOCKS: [u32; (DCBS * DCB_SIZE / 4) as usize] = [0; (DCBS * DCB_SIZE / 4) as usize];
static mut ERRNO: u32 = 0;
static mut SEARCH: u32 = 0;

fn fcb_at(fd: u32) -> u32 {
    rd(TABLES + table::FCB) + fd * FCB_SIZE
}

fn dcb_at(i: u32) -> u32 {
    rd(TABLES + table::DCB) + i * DCB_SIZE
}

fn fail(e: u32) -> u32 {
    unsafe { ERRNO = e };
    FAIL
}

fn open_fcb(fd: u32) -> Option<u32> {
    if fd < FCBS && rd(fcb_at(fd) + fcb::STATUS) != 0 {
        Some(fcb_at(fd))
    } else {
        unsafe { ERRNO = error::BAD_HANDLE };
        None
    }
}

fn free_fcb() -> Option<u32> {
    (0..FCBS).find(|fd| rd(fcb_at(*fd) + fcb::STATUS) == 0)
}

/// A name's device, its port (hex digits after the letters) and its path after the colon.
fn device(name: u32) -> Option<(u32, u32, u32)> {
    if name == 0 {
        return None;
    }
    let mut n = 0;
    while (b'a' as u32..=b'z' as u32).contains(&rd8(name + n)) {
        n += 1;
    }
    let mut port = 0;
    let mut p = name + n;
    while let Some(d) = (rd8(p) as u8 as char).to_digit(16) {
        port = port << 4 | d;
        p += 1;
    }
    if n == 0 || rd8(p) != b':' as u32 {
        return None;
    }
    (0..DCBS)
        .map(dcb_at)
        .find(|d| {
            let short = rd(*d + dcb::NAME);
            short != 0 && string::length(short) == n && (0..n).all(|i| rd8(short + i) == rd8(name + i))
        })
        .map(|d| (d, port, p + 1))
}

fn setup(fd: u32, mode: u32, dcb: u32, port: u32) -> u32 {
    let f = fcb_at(fd);
    for o in (0..FCB_SIZE).step_by(4) {
        wr(f + o, 0);
    }
    wr(f + fcb::STATUS, mode);
    wr(f + fcb::PORT, port);
    wr(f + fcb::FLAGS, rd(dcb + dcb::FLAGS));
    wr(f + fcb::DCB, dcb);
    wr(f + fcb::NUMBER, fd);
    f
}

/// A(00h) and B(32h).
pub extern "C" fn open(name: u32, mode: u32) -> u32 {
    let Some((d, port, path)) = device(name) else { return fail(error::UNKNOWN_DEVICE) };
    let Some(fd) = free_fcb() else { return fail(error::NO_HANDLES) };
    if mode == 0 {
        return fail(error::INVALID);
    }
    let f = setup(fd, mode, d, port);
    let open = rd(d + dcb::OPEN);
    let e = if open != 0 { call(open, f, path, mode, 0) } else { 0 };
    if e != 0 {
        wr(f + fcb::STATUS, 0);
        return fail(e);
    }
    fd
}

/// A(04h) and B(36h): the handle.
pub extern "C" fn close(fd: u32) -> u32 {
    let Some(f) = open_fcb(fd) else { return FAIL };
    let close = rd(rd(f + fcb::DCB) + dcb::CLOSE);
    if close != 0 {
        call(close, f, 0, 0, 0);
    }
    wr(f + fcb::STATUS, 0);
    fd
}

fn transfer(fd: u32, address: u32, length: u32, write: bool) -> u32 {
    let Some(f) = open_fcb(fd) else { return FAIL };
    if length == 0 {
        wr(f + fcb::ERROR, error::INVALID);
        return fail(error::INVALID);
    }
    let d = rd(f + fcb::DCB);
    let direct = rd(d + if write { dcb::WRITE } else { dcb::READ });
    let n = if direct != 0 {
        call(direct, f, address, length, 0)
    } else if rd(d + dcb::IN_OUT) != 0 {
        wr(f + fcb::ADDRESS, address);
        wr(f + fcb::LENGTH, length);
        call(rd(d + dcb::IN_OUT), f, if write { 2 } else { 1 }, 0, 0)
    } else {
        wr(f + fcb::ERROR, error::INVALID);
        return fail(error::INVALID);
    };
    if n != FAIL {
        wr(f + fcb::POSITION, rd(f + fcb::POSITION).wrapping_add(n));
    }
    n
}

/// A(02h) and B(34h).
pub extern "C" fn read(fd: u32, dst: u32, length: u32) -> u32 {
    transfer(fd, dst, length, false)
}

/// A(03h) and B(35h).
pub extern "C" fn write(fd: u32, src: u32, length: u32) -> u32 {
    transfer(fd, src, length, true)
}

/// A(01h) and B(33h): from the start or the current position; seeking from the end is refused.
pub extern "C" fn lseek(fd: u32, offset: u32, kind: u32) -> u32 {
    let Some(f) = open_fcb(fd) else { return FAIL };
    let position = match kind {
        0 => offset,
        1 => rd(f + fcb::POSITION).wrapping_add(offset),
        _ => {
            wr(f + fcb::ERROR, error::INVALID);
            return fail(error::INVALID);
        }
    };
    wr(f + fcb::POSITION, position);
    position
}

/// A(05h) and B(37h).
pub extern "C" fn ioctl(fd: u32, cmd: u32, arg: u32) -> u32 {
    let Some(f) = open_fcb(fd) else { return FAIL };
    let ioctl = rd(rd(f + fcb::DCB) + dcb::IOCTL);
    if ioctl == 0 {
        return fail(error::INVALID);
    }
    call(ioctl, f, cmd, arg, 0)
}

/// A(07h) and B(39h): bit 1 of the device's flags.
pub extern "C" fn isatty(fd: u32) -> u32 {
    open_fcb(fd).map_or(0, |f| rd(f + fcb::FLAGS) >> 1 & 1)
}

static mut BYTE: u32 = 0;

/// A(08h) and B(3Ah): sign-extended, as documented.
pub extern "C" fn getc(fd: u32) -> u32 {
    let b = &raw mut BYTE as u32;
    wr(b, 0);
    if read(fd, b, 1) != 1 {
        return FAIL;
    }
    rd8(b) as u8 as i8 as i32 as u32
}

/// A(09h) and B(3Bh): the handle second.
pub extern "C" fn putc(c: u32, fd: u32) -> u32 {
    let b = &raw mut BYTE as u32;
    wr(b, c & 0xFF);
    write(fd, b, 1)
}

/// B(54h).
pub extern "C" fn get_errno() -> u32 {
    unsafe { ERRNO }
}

/// B(55h).
pub extern "C" fn get_error(fd: u32) -> u32 {
    if fd < FCBS && rd(fcb_at(fd)) != 0 { rd(fcb_at(fd) + fcb::ERROR) } else { FAIL }
}

/// A name's device function called with a scratch FCB, as cd, format, erase and the rest do; 0 when it fails.
fn by_name(name: u32, function: u32, a: u32, b: u32) -> u32 {
    let Some((d, port, path)) = device(name) else {
        unsafe { ERRNO = error::UNKNOWN_DEVICE };
        return 0;
    };
    let Some(fd) = free_fcb() else {
        unsafe { ERRNO = error::NO_HANDLES };
        return 0;
    };
    let f = setup(fd, 0, d, port);
    let fun = rd(d + function);
    if fun == 0 {
        unsafe { ERRNO = error::INVALID };
        return 0;
    }
    call(fun, f, path, a, b)
}

/// B(40h).
pub extern "C" fn cd(name: u32) -> u32 {
    by_name(name, dcb::CD, 0, 0)
}

/// B(41h).
pub extern "C" fn format(name: u32) -> u32 {
    by_name(name, dcb::FORMAT, 0, 0)
}

/// B(45h).
pub extern "C" fn erase(name: u32) -> u32 {
    by_name(name, dcb::ERASE, 0, 0)
}

/// B(46h).
pub extern "C" fn undelete(name: u32) -> u32 {
    by_name(name, dcb::UNDELETE, 0, 0)
}

/// B(59h).
pub extern "C" fn testdevice(name: u32) -> u32 {
    by_name(name, dcb::TESTDEVICE, 0, 0)
}

/// B(42h): the FCB it used is kept for nextfile.
pub extern "C" fn firstfile2(name: u32, direntry: u32) -> u32 {
    unsafe { SEARCH = 0 };
    let Some((d, port, path)) = device(name) else {
        unsafe { ERRNO = error::UNKNOWN_DEVICE };
        return 0;
    };
    let Some(fd) = free_fcb() else { return 0 };
    let f = setup(fd, 0, d, port);
    let fun = rd(d + dcb::FIRSTFILE);
    if fun == 0 {
        return 0;
    }
    unsafe { SEARCH = f };
    call(fun, f, path, direntry, 0)
}

/// B(43h).
pub extern "C" fn nextfile(direntry: u32) -> u32 {
    let f = unsafe { SEARCH };
    if f == 0 {
        return 0;
    }
    let fun = rd(rd(f + fcb::DCB) + dcb::NEXTFILE);
    if fun == 0 { 0 } else { call(fun, f, direntry, 0, 0) }
}

/// B(44h): both names on one device.
pub extern "C" fn rename(old: u32, new: u32) -> u32 {
    let (Some((d1, port, p1)), Some((d2, _, p2))) = (device(old), device(new)) else {
        unsafe { ERRNO = error::UNKNOWN_DEVICE };
        return 0;
    };
    if d1 != d2 {
        unsafe { ERRNO = error::OTHER_DEVICE };
        return 0;
    }
    let Some(fd) = free_fcb() else { return 0 };
    let f = setup(fd, 0, d1, port);
    let fun = rd(d1 + dcb::RENAME);
    if fun == 0 { 0 } else { call(fun, f, p1, f, p2) }
}

/// B(47h): a device's 50h-byte DCB copied into a free one, its init called; 1, or 0 with none free.
pub extern "C" fn add_drv(info: u32) -> u32 {
    let Some(d) = (0..DCBS).map(dcb_at).find(|d| rd(*d + dcb::NAME) == 0) else { return 0 };
    for o in (0..DCB_SIZE).step_by(4) {
        wr(d + o, rd(info + o));
    }
    let init = rd(d + dcb::INIT);
    if init != 0 {
        call(init, 0, 0, 0, 0);
    }
    1
}

/// B(48h).
pub extern "C" fn del_drv(name: u32) -> u32 {
    for d in (0..DCBS).map(dcb_at) {
        let short = rd(d + dcb::NAME);
        if short != 0 && string::strcmp(short, name) == 0 {
            for o in (0..DCB_SIZE).step_by(4) {
                wr(d + o, 0);
            }
            return 1;
        }
    }
    0
}

/// B(49h).
pub extern "C" fn print_installed_devices() -> u32 {
    for d in (0..DCBS).map(dcb_at) {
        if rd(d + dcb::NAME) != 0 {
            printf::print(b"\t%s:\t%s\n\0".as_ptr() as u32, &[rd(d + dcb::NAME), rd(d + dcb::LONG_NAME)]);
        }
    }
    0
}

/// A(98h).
pub extern "C" fn add_duart_tty_device() -> u32 {
    add_drv(tty::device(true))
}

/// A(99h).
pub extern "C" fn add_nullcon_driver() -> u32 {
    add_drv(tty::device(false))
}

/// C(13h): standard input on fd 0, standard output on fd 1.
pub extern "C" fn flush_std_in_out_put() -> u32 {
    close(0);
    close(1);
    open(b"tty:\0".as_ptr() as u32, 1);
    open(b"tty:\0".as_ptr() as u32, 2);
    0
}

/// C(1Bh): the TTY remounted as the DUART's (1) or the dummy (0); any other value leaves none.
pub extern "C" fn kernel_redirect(flag: u32) -> u32 {
    del_drv(b"tty\0".as_ptr() as u32);
    match flag {
        0 => {
            add_nullcon_driver();
        }
        1 => {
            add_duart_tty_device();
        }
        _ => {}
    }
    flush_std_in_out_put()
}

/// C(12h): the TTY (the CD and the cards join at phase 2).
pub extern "C" fn install_devices(flag: u32) -> u32 {
    if flag != 0 {
        add_duart_tty_device()
    } else {
        add_nullcon_driver()
    };
    0
}

/// At boot: the tables' FCB and DCB entries, the DUART TTY, standard I/O.
pub fn install_standard_devices() {
    wr(TABLES + table::FCB, &raw const FCB_BLOCKS as u32);
    wr(TABLES + table::FCB + 4, FCBS * FCB_SIZE);
    wr(TABLES + table::DCB, &raw const DCB_BLOCKS as u32);
    wr(TABLES + table::DCB + 4, DCBS * DCB_SIZE);
    install_devices(1);
    flush_std_in_out_put();
}
