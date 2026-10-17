//! The N64 GameShark: lines of an eight-digit command and a four-digit value, decoded to RDRAM writes and tests
//! (Mars_Cheats.md).

use crate::cheats::{Decoded, Write, WriteType, format_error, hex, is_hex};

const RDRAM: &str = "RDRAM";

/// A code's line: the command word, whose top byte is the type and the rest a physical address, and the value.
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
        (self.command & 0x00FF_FFFF) as i32
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08X} {:04X}", self.command, self.value)
    }
}

/// Runs of hex digits and '?', split on anything else; '?' rides in a run so the placeholder reaches its own words.
fn runs(code: &str) -> Vec<String> {
    let mut runs = Vec::new();
    let mut current = String::new();
    for c in code.chars() {
        if is_hex(c) || c == '?' {
            current.push(c);
        } else if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    runs
}

fn parse(command: &str, value: &str) -> Line {
    Line { command: hex(&command.replace('?', "0")), value: hex(&value.replace('?', "0")) as u16 }
}

/// An eight-digit run then a one-to-four-digit one is a line, and a run of twelves is lines run together.
pub fn read_lines(code: &str) -> Option<Vec<Line>> {
    let runs = runs(code);
    if runs.is_empty() {
        return None;
    }
    let mut lines = Vec::new();
    let mut i = 0;
    while i < runs.len() {
        let run = &runs[i];
        if run.len() == 8 && i + 1 < runs.len() && (1..=4).contains(&runs[i + 1].len()) {
            lines.push(parse(run, &runs[i + 1]));
            i += 2;
            continue;
        }
        if run.len() % 12 != 0 {
            return None;
        }
        for at in (0..run.len()).step_by(12) {
            lines.push(parse(&run[at..at + 8], &run[at + 8..at + 12]));
        }
        i += 1;
    }
    Some(lines)
}

pub fn can_decode(code: &str) -> bool {
    read_lines(code).is_some()
}

fn require_even(line: Line, address: i32, width: i32) -> Decoded<()> {
    if width == 2 && address & 1 != 0 {
        return format_error(format!(
            "'{line}' puts 16 bits at an odd address, which no halfword store can do; both reference emulators tear it across two words - see Mars_Cheats.md §2.4."
        ));
    }
    Ok(())
}

/// Both references take an 8-bit code's value from its low byte; the N64 is big-endian.
fn write(line: Line, width: i32) -> Decoded<Write> {
    require_even(line, line.address(), width)?;
    let mut w = Write::poke(RDRAM, line.address(), if width == 1 { (line.value & 0xFF) as u32 } else { line.value as u32 }, width);
    w.big_endian = true;
    Ok(w)
}

fn test(line: Line, width: i32, kind: WriteType) -> Decoded<Write> {
    let mut w = write(line, width)?;
    w.kind = kind;
    Ok(w)
}

/// 5000CCSS VVVV: count, address step and value step, applied to the one write after it.
fn repeat(repeater: Line, next: Line) -> Decoded<Write> {
    let (count, step) = (((repeater.command >> 8) & 0xFF) as i32, (repeater.command & 0xFF) as i32);
    if count == 0 {
        return format_error(format!("'{repeater}' repeats zero times, which writes nothing - both reference emulators skip it too."));
    }
    let width = match next.kind() {
        0x80 | 0xA0 => 1,
        0x81 | 0xA1 => 2,
        _ => return format_error(format!("'{repeater}' must be followed by the 80, 81, A0 or A1 line it repeats, not '{next}'.")),
    };
    let mut w = write(next, width)?;
    if count > 1 {
        require_even(next, next.address() + step, width)?;
    }
    w.repeat_count = count;
    w.repeat_add_address = step;
    w.repeat_add_value = repeater.value as u32;
    Ok(w)
}

fn refusal(line: Line) -> String {
    match line.kind() {
        0x88 | 0x89 | 0xA8 | 0xA9 | 0xD8..=0xDB => format!(
            "'{line}' applies only while the GameShark's own button is held, and Mars has no such button - write it as 80 or 81 to hold the value every frame."
        ),
        0xF0 | 0xF1 => format!(
            "'{line}' is a boot-time write, and Mars applies cheats only at frame boundaries, to a list handed over after the game is loaded - there is no boot to write it at."
        ),
        0xEE => format!("'{line}' hides the Expansion Pak from the game; Mars decides the Pak when the console is built, so leave this code out."),
        0xDE => format!("'{line}' has no effect on memory in either reference emulator, and nothing here documents what it asks of a GameShark - leave it out."),
        0xCC | 0xFF => format!("'{line}' is not a code type either reference emulator applies, so Mars refuses it rather than guess what it does."),
        _ => format!("'{line}' is not a code type Mars decodes - it takes 80, 81, A0, A1, D0, D1, D2, D3 and 50."),
    }
}

/// Every line a write or a test, a repeater and its line one write, anything else refused by name.
pub fn decode_writes(code: &str) -> Decoded<Vec<Write>> {
    let Some(lines) = read_lines(code) else {
        return format_error(format!("'{code}' isn't an N64 GameShark code - expected lines of 8 and 4 hex digits (TTAAAAAA VVVV), joined by '+' or spaces."));
    };
    if code.contains('?') {
        return format_error(format!("'{code}' holds '?', Project64's placeholder for a value picked from a list - put the value itself in its place."));
    }
    let mut writes = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        use WriteType::*;
        writes.push(match line.kind() {
            0x80 | 0xA0 => write(line, 1)?,
            0x81 | 0xA1 => write(line, 2)?,
            0xD0 => test(line, 1, IfEqual)?,
            0xD1 => test(line, 2, IfEqual)?,
            0xD2 => test(line, 1, IfNotEqual)?,
            0xD3 => test(line, 2, IfNotEqual)?,
            0x50 => {
                if i + 1 >= lines.len() {
                    return format_error(format!("'{line}' is a repeater with no line after it to repeat."));
                }
                i += 1;
                repeat(line, lines[i])?
            }
            _ => return format_error(refusal(line)),
        });
        i += 1;
    }
    if writes.last().is_some_and(|w| w.kind.is_test()) {
        return format_error(format!("'{}' tests memory but guards nothing - a D0-D3 line must be followed by the code it guards.", lines[lines.len() - 1]));
    }
    Ok(writes)
}

/// The single byte the older callers ask for, which only a lone 8-bit write can give honestly.
pub fn one_byte(writes: &[Write], code: &str) -> Decoded<(i32, u8)> {
    match writes {
        [only] if only.effective_width() == 1 && only.effective_repeat_count() == 1 => Ok((only.address, only.value as u8)),
        _ => format_error(format!("'{code}' is more than one byte written once - it has to be added whole, through DecodeWrites.")),
    }
}
