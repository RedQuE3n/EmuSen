//! The PlayStation's GameShark (with Game Buster and Action Replay, the same Datel format) and Xploder codes, as RAM
//! writes and tests; the formats, the choices and the refusals are EmuSen_Cheats.md §10.

use crate::cheats::{Decoded, Write, WriteType, format_error, hex, is_hex};
use emusen_dotnet::text::is_letter_or_digit;

const RAM: &str = "RAM";

/// A code's line: the command word, whose top byte is the type, and the value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    pub command: u32,
    pub value: u16,
}

impl Line {
    fn kind(self) -> u8 {
        (self.command >> 24) as u8
    }

    fn address(self) -> i32 {
        (self.command & 0x1F_FFFF) as i32
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08X} {:04X}", self.command, self.value)
    }
}

/// Every hex digit of the code, in twelves; none where there are none, a remainder, or a letter or digit that is not hex.
pub fn read_lines(code: &str) -> Option<Vec<Line>> {
    let digits: String = code.chars().filter(|&c| is_hex(c)).collect();
    if digits.is_empty() || digits.len() % 12 != 0 || code.chars().any(|c| is_letter_or_digit(c) && !is_hex(c)) {
        return None;
    }
    Some((0..digits.len()).step_by(12).map(|at| Line { command: hex(&digits[at..at + 8]), value: hex(&digits[at + 8..at + 12]) as u16 }).collect())
}

/// The first digits psx-spx gives the Datel format, 3 and 8 shared with the Xploder, 0 for its dummy line and 5 for its slide.
fn game_shark_line(l: Line) -> bool {
    matches!(l.kind() >> 4, 0x1 | 0x2 | 0x3 | 0x5 | 0x8 | 0xC | 0xD | 0xE) || (l.command == 0 && l.value == 0)
}

pub fn is_game_shark(code: &str) -> bool {
    read_lines(code).is_some_and(|lines| lines.iter().all(|&l| game_shark_line(l)))
}

/// A write's value is the line's low byte for an 8-bit code; every address is main RAM's.
fn write(l: Line, width: i32, kind: WriteType) -> Write {
    let mut w = Write::poke(RAM, l.address(), if width == 1 { (l.value & 0xFF) as u32 } else { l.value as u32 }, width);
    w.kind = kind;
    w
}

pub fn game_shark_writes(code: &str) -> Decoded<Vec<Write>> {
    let lines = match read_lines(code) {
        Some(lines) if lines.iter().all(|&l| game_shark_line(l)) => lines,
        _ => {
            return format_error(format!(
                "'{code}' is not a PlayStation GameShark code: lines of eight and four hex digits (TTAAAAAA VVVV) whose types are the Datel device's."
            ));
        }
    };
    let mut writes = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        use WriteType::*;
        match l.kind() {
            0x30 => writes.push(write(l, 1, Set)),
            0x80 => writes.push(write(l, 2, Set)),
            0x10 => writes.push(write(l, 2, Increase)),
            0x11 => writes.push(write(l, 2, Decrease)),
            0x20 => writes.push(write(l, 1, Increase)),
            0x21 => writes.push(write(l, 1, Decrease)),
            0xD0 => writes.push(write(l, 2, IfEqual)),
            0xD1 => writes.push(write(l, 2, IfNotEqual)),
            0xD2 => writes.push(write(l, 2, IfGreater)),
            0xD3 => writes.push(write(l, 2, IfLess)),
            0xE0 => writes.push(write(l, 1, IfEqual)),
            0xE1 => writes.push(write(l, 1, IfNotEqual)),
            0xE2 => writes.push(write(l, 1, IfGreater)),
            0xE3 => writes.push(write(l, 1, IfLess)),
            0x00 if l.command == 0 && l.value == 0 => {}
            0x50 => {
                if i + 1 >= lines.len() {
                    return format_error(format!("'{l}' is a slide with no line after it to repeat."));
                }
                i += 1;
                writes.push(slide(l, lines[i])?);
            }
            _ => return format_error(game_shark_refusal(l)),
        }
        i += 1;
    }
    guarded(writes, &lines, code)
}

/// 5000nnbb dddd then the write it repeats: nn writes of dddd, bb apart in address and ee apart in value.
fn slide(slide: Line, next: Line) -> Decoded<Write> {
    let (count, step) = (((slide.command >> 8) & 0xFF) as i32, (slide.command & 0xFF) as i32);
    if slide.command & 0x00FF_0000 != 0 {
        return format_error(format!("'{slide}' is not a slide the chapter gives, whose second byte is 00."));
    }
    if count == 0 {
        return format_error(format!("'{slide}' slides zero times, which writes nothing."));
    }
    let width = match next.kind() {
        0x30 => 1,
        0x80 => 2,
        _ => return format_error(format!("'{slide}' must be followed by the 30 or 80 line it repeats, not '{next}'.")),
    };
    let mut w = write(next, width, WriteType::Set);
    w.value = if width == 1 { (slide.value & 0xFF) as u32 } else { slide.value as u32 };
    w.repeat_count = count;
    w.repeat_add_address = step;
    w.repeat_add_value = next.value as u32;
    Ok(w)
}

fn game_shark_refusal(l: Line) -> String {
    match l.kind() {
        0xC0 => format!("'{l}' turns every code on when its test passes; whether the device keeps them on after is not in its chapter, so EmuSen applies no such code yet."),
        0xC1 => format!("'{l}' delays the codes from the device's own start, which EmuSen does not have; leave it out and the codes apply from the first frame."),
        0xC2 => format!("'{l}' copies a block of memory, a kind of code EmuSen's cheats do not have."),
        0xC3 => format!("'{l}' is an indirect write of one homebrew loader's, not the Datel device's."),
        0xD4..=0xD6 => format!("'{l}' tests the pad's buttons as the device reads them, which a cheat in EmuSen cannot."),
        0x12 | 0x22 => format!("'{l}' is a 32-bit increment of one homebrew loader's, not the Datel device's."),
        _ => format!("'{l}' is not a GameShark code type EmuSen decodes: it takes 30, 80, 10, 11, 20, 21, D0 to D3, E0 to E3 and 50."),
    }
}

/// The six bytes AABBCCDD EEFF decrypted by the key in the type's low three bits, psx-spx's decrypt_xplorer_cheat_code.
pub fn decrypt(l: Line) -> Decoded<Line> {
    let mut x = [(l.command >> 24) as u8, (l.command >> 16) as u8, (l.command >> 8) as u8, l.command as u8, (l.value >> 8) as u8, l.value as u8];
    let key = x[0] & 7;
    x[0] ^= key;
    let b = |v: u8| v as i32;
    match key {
        0 => {}
        4 => {
            x[1] ^= 0x25;
            x[2] ^= (0xFA + (b(x[1]) & 0x11)) as u8;
            x[3] ^= (0xC0 + (b(x[2]) & 0x11) + (b(x[1]) ^ 0x12)) as u8;
            x[4] ^= (0x7E + (b(x[3]) & 0x11) + (b(x[2]) ^ 0x12) + b(x[1])) as u8;
            x[5] ^= (0x26 + (b(x[4]) & 0x11) + (b(x[3]) ^ 0x12) + b(x[2]) + b(x[1])) as u8;
        }
        5 => {
            x[1] = x[1].wrapping_add(0x57);
            x[2] = x[2].wrapping_add(0x42);
            x[3] = x[3].wrapping_add(0x31);
            x[4] = x[4].wrapping_add(0x32);
            x[5] = x[5].wrapping_add(0x33);
        }
        6 => {
            for (i, v) in x.iter_mut().enumerate().skip(1) {
                *v = ((b(*v) + 0xAB) ^ i as i32) as u8;
            }
        }
        7 => {
            x[5] = x[5].wrapping_add(0xCB);
            x[4] = (b(x[4]) + 0xCB + (b(x[5]) & 0x73)) as u8;
            x[3] = (b(x[3]) + 0x5A + (b(x[4]) & 0x73) - (b(x[5]) ^ 0x90)) as u8;
            x[2] = (b(x[2]) + 0x16 + (b(x[3]) & 0x73) - (b(x[4]) ^ 0x90) + b(x[5])) as u8;
            x[1] = (b(x[1]) + 0xF5 + (b(x[2]) & 0x73) - (b(x[3]) ^ 0x90) + b(x[4]) + b(x[5])) as u8;
        }
        _ => return format_error(format!("'{l}' names encryption key {key}, which the Xploder's chapter gives no rule for.")),
    }
    Ok(Line { command: u32::from_be_bytes([x[0], x[1], x[2], x[3]]), value: u16::from_be_bytes([x[4], x[5]]) })
}

fn xploder_line(l: Line) -> bool {
    matches!(l.kind() >> 4, 0x0 | 0x3 | 0x4 | 0x5 | 0x6 | 0x7 | 0x8 | 0x9 | 0xB | 0xC | 0xD | 0xF)
}

pub fn is_xploder(code: &str) -> bool {
    read_lines(code).is_some_and(|lines| lines.iter().all(|&l| xploder_line(l)))
}

pub fn xploder_writes(code: &str) -> Decoded<Vec<Write>> {
    let raw = match read_lines(code) {
        Some(lines) if lines.iter().all(|&l| xploder_line(l)) => lines,
        _ => {
            return format_error(format!(
                "'{code}' is not a PlayStation Xploder code: lines of eight and four hex digits (TtAAAAAA VVVV) whose types are the Xploder's."
            ));
        }
    };
    let lines = raw.into_iter().map(decrypt).collect::<Decoded<Vec<Line>>>()?;
    let mut writes = Vec::new();
    for &l in &lines {
        use WriteType::*;
        match l.kind() >> 4 {
            0x3 => writes.push(write(l, 1, Set)),
            0x8 => writes.push(write(l, 2, Set)),
            0x0 => writes.push(write(l, 4, Set)),
            0x7 => writes.push(write(l, 2, IfEqual)),
            0x9 => writes.push(write(l, 2, IfNotEqual)),
            _ => return format_error(xploder_refusal(l)),
        }
    }
    guarded(writes, &lines, code)
}

fn xploder_refusal(l: Line) -> String {
    match l.kind() >> 4 {
        0x4 => format!("'{l}' slows the game down, by an amount the Xploder's chapter does not give, so EmuSen applies no such code."),
        0x5 => format!("'{l}' writes a run of bytes listed after it, which this decoder does not take yet."),
        0x6 => format!("'{l}' sets a processor breakpoint for code of the device's own, which a cheat in EmuSen cannot run."),
        0xB => format!("'{l}' is a slide whose end the Xploder's chapter leaves unclear, so EmuSen applies no such code."),
        0xF => format!("'{l}' turns on other selected codes, a rule the Xploder's chapter does not give."),
        _ => format!("'{l}' is a type the Xploder's chapter calls a mirror of another or does not give."),
    }
}

/// A code whose types are all the Datel device's is a GameShark code; any other the Xploder's.
pub fn is_either(code: &str) -> bool {
    is_game_shark(code) || is_xploder(code)
}

pub fn either_writes(code: &str) -> Decoded<Vec<Write>> {
    if is_game_shark(code) {
        game_shark_writes(code)
    } else if is_xploder(code) {
        xploder_writes(code)
    } else if code.chars().any(|c| matches!(c, '?' | 'x' | 'X' | 'y' | 'Y')) {
        format_error(format!("'{code}' holds a placeholder for a value the player picks; put the value itself in its place."))
    } else {
        format_error(format!("'{code}' is neither a PlayStation GameShark nor an Xploder code: lines of eight and four hex digits (TTAAAAAA VVVV)."))
    }
}

/// A test with nothing after it guards nothing.
fn guarded(writes: Vec<Write>, lines: &[Line], code: &str) -> Decoded<Vec<Write>> {
    match writes.last() {
        None => format_error(format!("'{code}' holds only dummy lines and writes nothing.")),
        Some(w) if w.kind.is_test() => format_error(format!("'{}' tests memory but guards nothing - a test must be followed by the code it guards.", lines[lines.len() - 1])),
        Some(_) => Ok(writes),
    }
}

/// The single byte the older callers ask for, only where the code is one byte written once.
pub fn one_byte(writes: &[Write], code: &str) -> Decoded<(i32, u8)> {
    match writes {
        [only] if only.effective_width() == 1 && only.effective_repeat_count() == 1 && only.kind == WriteType::Set => Ok((only.address, only.value as u8)),
        _ => format_error(format!("'{code}' is more than one byte written once - it has to be added whole, through DecodeWrites.")),
    }
}
