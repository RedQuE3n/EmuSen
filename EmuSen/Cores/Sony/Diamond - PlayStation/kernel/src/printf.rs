//! A(3Fh) printf: the conversions and prefixes psx-spx lists, each character through putchar. "h" sign-extends the
//! low halfword for every conversion, as documented for the unsigned ones too. Diamond_OpenKernel.md §4.10.

use crate::mem::{rd, rd8, wr, wr16};
use crate::tty::putchar;

struct Args<'a> {
    at: u32,
    list: Option<&'a [u32]>,
    index: usize,
}

impl Args<'_> {
    fn next(&mut self) -> u32 {
        let i = self.index;
        self.index += 1;
        match self.list {
            Some(l) => l.get(i).copied().unwrap_or(0),
            None => rd(self.at + 4 * i as u32),
        }
    }
}

#[derive(Default, Clone, Copy)]
struct Spec {
    left: bool,
    plus: bool,
    space: bool,
    alternate: bool,
    zero: bool,
    width: u32,
    precision: Option<u32>,
    half: bool,
}

fn pad(n: u32, c: u32, count: &mut u32) {
    for _ in 0..n {
        putchar(c);
        *count += 1;
    }
}

fn field(spec: &Spec, prefix: &[u8], digits: &[u8], count: &mut u32) {
    let zeros = spec.precision.map_or(0, |p| p.saturating_sub(digits.len() as u32));
    let body = prefix.len() as u32 + zeros + digits.len() as u32;
    let fill = spec.width.saturating_sub(body);
    let zero_fill = spec.zero && !spec.left && spec.precision.is_none();
    if !spec.left && !zero_fill {
        pad(fill, b' ' as u32, count);
    }
    for c in prefix {
        putchar(*c as u32);
        *count += 1;
    }
    if zero_fill {
        pad(fill, b'0' as u32, count);
    }
    pad(zeros, b'0' as u32, count);
    for c in digits {
        putchar(*c as u32);
        *count += 1;
    }
    if spec.left {
        pad(fill, b' ' as u32, count);
    }
}

fn digits(mut v: u32, base: u32, upper: bool, buffer: &mut [u8; 12]) -> usize {
    let set: &[u8; 16] = if upper { b"0123456789ABCDEF" } else { b"0123456789abcdef" };
    let mut n = 0;
    loop {
        buffer[11 - n] = set[(v % base) as usize];
        n += 1;
        v /= base;
        if v == 0 {
            return n;
        }
    }
}

fn format(fmt: u32, args: &mut Args) -> u32 {
    let mut count = 0;
    let mut i = fmt;
    loop {
        let c = rd8(i);
        i += 1;
        if c == 0 {
            return count;
        }
        if c != b'%' as u32 {
            putchar(c);
            count += 1;
            continue;
        }
        let mut spec = Spec::default();
        loop {
            match rd8(i) as u8 {
                b'-' => spec.left = true,
                b'+' => spec.plus = true,
                b' ' => spec.space = true,
                b'#' => spec.alternate = true,
                b'0' => spec.zero = true,
                _ => break,
            }
            i += 1;
        }
        if rd8(i) == b'*' as u32 {
            let w = args.next() as i32;
            if w < 0 {
                spec.left = true;
            }
            spec.width = w.unsigned_abs();
            i += 1;
        } else {
            while (b'0'..=b'9').contains(&(rd8(i) as u8)) {
                spec.width = spec.width * 10 + rd8(i) - b'0' as u32;
                i += 1;
            }
        }
        if rd8(i) == b'.' as u32 {
            i += 1;
            let mut p = 0;
            if rd8(i) == b'*' as u32 {
                p = args.next();
                i += 1;
            } else {
                while (b'0'..=b'9').contains(&(rd8(i) as u8)) {
                    p = p * 10 + rd8(i) - b'0' as u32;
                    i += 1;
                }
            }
            spec.precision = Some(p);
        }
        loop {
            match rd8(i) as u8 {
                b'h' => spec.half = true,
                b'l' => spec.half = false,
                b'L' => {}
                _ => break,
            }
            i += 1;
        }
        let conversion = rd8(i) as u8;
        if conversion == 0 {
            return count;
        }
        i += 1;
        let mut buffer = [0u8; 12];
        let value = |args: &mut Args, half: bool| {
            let v = args.next();
            if half { v as u16 as i16 as i32 as u32 } else { v }
        };
        match conversion {
            b'c' => {
                buffer[11] = args.next() as u8;
                field(&Spec { zero: false, precision: None, ..spec }, b"", &buffer[11..], &mut count);
            }
            b's' => {
                let a = args.next();
                let s = if a == 0 { b"<NULL>\0".as_ptr() as u32 } else { a };
                let mut n = crate::string::length(s);
                if let Some(p) = spec.precision {
                    n = n.min(p);
                }
                let fill = spec.width.saturating_sub(n);
                if !spec.left {
                    pad(fill, b' ' as u32, &mut count);
                }
                for k in 0..n {
                    putchar(rd8(s + k));
                    count += 1;
                }
                if spec.left {
                    pad(fill, b' ' as u32, &mut count);
                }
            }
            b'd' | b'i' | b'D' => {
                let v = value(args, spec.half && conversion != b'D') as i32;
                let n = digits(v.unsigned_abs(), 10, false, &mut buffer);
                let sign: &[u8] = if v < 0 {
                    b"-"
                } else if spec.plus {
                    b"+"
                } else if spec.space {
                    b" "
                } else {
                    b""
                };
                field(&spec, sign, &buffer[12 - n..], &mut count);
            }
            b'u' | b'U' => {
                let n = digits(value(args, spec.half && conversion == b'u'), 10, false, &mut buffer);
                field(&spec, b"", &buffer[12 - n..], &mut count);
            }
            b'o' | b'O' => {
                let v = value(args, spec.half && conversion == b'o');
                let n = digits(v, 8, false, &mut buffer);
                let prefix: &[u8] = if spec.alternate && v != 0 { b"0" } else { b"" };
                field(&spec, prefix, &buffer[12 - n..], &mut count);
            }
            b'x' | b'X' | b'p' => {
                let v = value(args, spec.half && conversion != b'p');
                let upper = conversion == b'X';
                let n = digits(v, 16, upper, &mut buffer);
                let prefix: &[u8] = match (spec.alternate && v != 0, upper) {
                    (false, _) => b"",
                    (true, false) => b"0x",
                    (true, true) => b"0X",
                };
                field(&spec, prefix, &buffer[12 - n..], &mut count);
            }
            b'n' => {
                let a = args.next();
                if spec.half { wr16(a, count) } else { wr(a, count) }
            }
            other => {
                putchar(other as u32);
                count += 1;
            }
        }
    }
}

/// Reached from A(3Fh) with the arguments in a row from `args`.
#[unsafe(no_mangle)]
extern "C" fn printf_args(fmt: u32, args: u32) -> u32 {
    format(fmt, &mut Args { at: args, list: None, index: 0 })
}

/// The kernel's own printing.
pub fn print(fmt: u32, args: &[u32]) -> u32 {
    format(fmt, &mut Args { at: 0, list: Some(args), index: 0 })
}
