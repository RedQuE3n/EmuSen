//! The TTY device and standard I/O (psx-spx "BIOS TTY Console (std_io)"). Its output is channel A of the expansion
//! port's DUART, the transmit holding register at 1F802023h, one byte a character, with TAB expanded to the next
//! eighth column and LF to CR LF; the dummy TTY's goes nowhere. No input device exists in phase 1: reads return no
//! characters. Diamond_OpenKernel.md §4.10.

use crate::file::{self, dcb, error, fcb};
use crate::layout::io;
use crate::mem::{rd, rd8, wr, wr8};

const DUART_FLAGS: u32 = 3;
const DUMMY_FLAGS: u32 = 1;

static mut INFO: [u32; 20] = [0; 20];
static mut COLUMN: u32 = 0;

/// A DCB for the DUART TTY (flags 3) or the dummy (flags 1), for AddDrv.
pub fn device(duart: bool) -> u32 {
    let info = &raw mut INFO as u32;
    for o in (0..0x50).step_by(4) {
        wr(info + o, 0);
    }
    wr(info + dcb::NAME, b"tty\0".as_ptr() as u32);
    wr(info + dcb::FLAGS, if duart { DUART_FLAGS } else { DUMMY_FLAGS });
    wr(info + dcb::SECTOR, 1);
    wr(info + dcb::LONG_NAME, b"CONSOLE\0".as_ptr() as u32);
    wr(info + dcb::INIT, dev_init as *const () as u32);
    wr(info + dcb::OPEN, dev_open as *const () as u32);
    wr(info + dcb::IN_OUT, dev_in_out as *const () as u32);
    wr(info + dcb::IOCTL, dev_ioctl as *const () as u32);
    info
}

fn emit(duart: bool, c: u32) {
    if duart {
        wr8(io::DUART_THRA, c);
    }
}

fn output(duart: bool, c: u32) {
    unsafe {
        match c {
            0x09 => loop {
                emit(duart, b' ' as u32);
                COLUMN += 1;
                if COLUMN % 8 == 0 {
                    break;
                }
            },
            0x0A => {
                emit(duart, 0x0D);
                emit(duart, 0x0A);
                COLUMN = 0;
            }
            0x0D => {
                emit(duart, c);
                COLUMN = 0;
            }
            0x08 => {
                emit(duart, c);
                COLUMN = COLUMN.saturating_sub(1);
            }
            _ => {
                emit(duart, c);
                COLUMN += 1;
            }
        }
    }
}

/// A(5Bh).
pub extern "C" fn dev_init() -> u32 {
    unsafe { COLUMN = 0 };
    0
}

/// A(5Ch): ports 0 and 1 only.
pub extern "C" fn dev_open(f: u32, _path: u32, _mode: u32) -> u32 {
    if rd(f + fcb::PORT) >= 2 { error::BAD_PORT } else { 0 }
}

/// A(5Dh): command 2 writes FCB[0Ch] bytes from FCB[08h]; command 1 reads none.
pub extern "C" fn dev_in_out(f: u32, cmd: u32) -> u32 {
    let (address, length) = (rd(f + fcb::ADDRESS), rd(f + fcb::LENGTH));
    match cmd {
        2 => {
            let duart = rd(f + fcb::FLAGS) & 2 != 0;
            for i in 0..length {
                output(duart, rd8(address + i));
            }
            length
        }
        _ => 0,
    }
}

/// A(5Eh).
pub extern "C" fn dev_ioctl(_f: u32, _cmd: u32, _arg: u32) -> u32 {
    0
}

/// A(3Ch) and B(3Dh).
pub extern "C" fn putchar(c: u32) -> u32 {
    file::putc(c, 1);
    c
}

/// A(3Bh) and B(3Ch): masked by 7Fh; 0 when nothing was read.
pub extern "C" fn getchar() -> u32 {
    let c = file::getc(0);
    if c == u32::MAX { 0 } else { c & 0x7F }
}

/// A(3Eh) and B(3Fh): no % resolved, no newline added; a null pointer prints <NULL>.
pub extern "C" fn puts(src: u32) -> u32 {
    let s = if src == 0 { b"<NULL>\0".as_ptr() as u32 } else { src };
    let n = crate::string::length(s);
    if n != 0 {
        file::write(1, s, n);
    }
    0
}

/// A(3Dh) and B(3Eh): the line editing psx-spx describes, ended by CR or LF, or by the end of the input.
pub extern "C" fn gets(dst: u32) -> u32 {
    let mut n = 0;
    let mut literal = false;
    loop {
        let raw = file::getc(0);
        if raw == u32::MAX {
            break;
        }
        let c = raw & 0x7F;
        if literal {
            literal = false;
        } else {
            match c {
                0x09 => {}
                0x08 | 0x7F => {
                    if n > 0 {
                        n -= 1;
                        puts(b"\x08 \x08\0".as_ptr() as u32);
                    }
                    continue;
                }
                0x0D | 0x0A => {
                    putchar(0x0A);
                    break;
                }
                0x16 => {
                    literal = true;
                    continue;
                }
                0x00..=0x1F => {
                    putchar(0x07);
                    continue;
                }
                _ => {}
            }
        }
        if n >= 125 {
            putchar(0x07);
            continue;
        }
        let c = if c == 0x09 { b' ' as u32 } else { c };
        wr8(dst + n, c);
        putchar(c);
        n += 1;
    }
    wr8(dst + n, 0);
    dst
}
