//! The memory spaces: `spaces`, `mem`, `write`, `memfind`, `dump` and `load`, as their C# classes worded them.

use crate::{core_name, session, Run};
use dianash_module::{Answer, Call};
use emusen_dotnet::file::{read_all_bytes, write_all_bytes};
use emusen_dotnet::number::parse_i32;
use emusen_dotnet::text::{pad_left, pad_right, trim_end};
use emusen_session_service::words::parse_hex;
pub(crate) use emusen_session_service::space::{all, find, read, Space};

/// The words every bulk read refuses a space with live hardware behind it in.
pub(crate) fn side_effects(space: &Space, what: &str) -> String {
    format!("{} can have real side effects on read (live hardware registers) - refusing a bulk {what} there. Try WRAM (or another plain-memory space) instead.", space.name)
}

pub fn spaces(call: &Call) -> Run {
    let session = session(call)?;
    let mut out = format!("{} memory spaces:\n", core_name(&session)?);
    for s in all(&session)? {
        out += &format!("  {} {} bytes  {}\n", pad_right(&s.name, 10), pad_left(&s.size.to_string(), 8), if s.writable { "R/W" } else { "R/O" });
    }
    Ok(Answer::ok(trim_end(&out)))
}

pub fn mem(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 3 {
        return Ok(Answer::ok("Usage: mem <space> <addr> [<len>]"));
    }
    let session = session(call)?;
    let space = find(&session, &parts[1])?;
    let addr = parse_hex(&parts[2])?;
    let len = if parts.len() >= 4 { parse_hex(&parts[3])? } else { 16 };
    let bytes = read(&session, &space, addr, len)?;
    let mut out = format!("{} @ 0x{addr:X} ({len} bytes):\n", space.name);
    let mut row = 0i32;
    while row < len {
        out += &format!("  {:06X}: ", addr.wrapping_add(row));
        let mut ascii = String::new();
        for col in 0..16 {
            if row + col < len {
                let b = bytes[(row + col) as usize];
                out += &format!("{b:02X} ");
                ascii.push(if (0x20..0x7F).contains(&b) { b as char } else { '.' });
            } else {
                out += "   ";
            }
        }
        out.push(' ');
        out += &ascii;
        out.push('\n');
        row += 16;
    }
    Ok(Answer::ok(trim_end(&out)))
}

pub fn write(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 4 {
        return Ok(Answer::ok("Usage: write <space> <addr> <value>"));
    }
    let session = session(call)?;
    let space = find(&session, &parts[1])?;
    if !space.writable {
        return Ok(Answer::ok(format!("{} is read-only.", space.name)));
    }
    let addr = parse_hex(&parts[2])?;
    let value = parse_hex(&parts[3])? as u8;
    session.space_write(&space.name, addr as u32, &[value])?;
    Ok(Answer::ok(format!("{}[0x{addr:X}] = 0x{value:02X}", space.name)))
}

/// `ParsePattern`: one entry a byte, `None` for `??`; the separators `,` `:` `-` `_` dropped first.
fn pattern(spec: &str) -> Option<Vec<Option<u8>>> {
    let cleaned: Vec<char> = spec.chars().filter(|c| !matches!(c, ',' | ':' | '-' | '_')).collect();
    if cleaned.len() % 2 != 0 {
        return None;
    }
    cleaned
        .chunks(2)
        .map(|pair| {
            let pair: String = pair.iter().collect();
            if pair == "??" {
                return Some(None);
            }
            // byte.TryParse with NumberStyles.HexNumber: white space around the digits is allowed.
            let digits = pair.trim_matches(|c: char| matches!(c, '\t'..='\r' | ' '));
            if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }
            u8::from_str_radix(digits, 16).ok().map(Some)
        })
        .collect()
}

pub fn memfind(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 3 {
        return Ok(Answer::ok("Usage: memfind <space> <bytes> [<max>]"));
    }
    let session = session(call)?;
    let space = find(&session, &parts[1])?;
    if space.side_effects {
        return Ok(Answer::ok(side_effects(&space, "scan")));
    }
    let mut max = 20;
    let mut words: Vec<String> = parts[2..].to_vec();
    if words.len() >= 2 {
        if let Some(parsed) = parse_i32(words.last().unwrap()).filter(|&p| p > 0) {
            max = parsed;
            words.pop();
        }
    }
    let Some(pattern) = pattern(&words.concat()) else {
        return Ok(Answer::ok("Pattern must be whole hex byte pairs, using '??' for a wildcard - e.g. 00FF11 or '00 ?? 11'."));
    };
    if pattern.is_empty() {
        return Ok(Answer::ok("Pattern is empty."));
    }
    if pattern.len() as i64 > i64::from(space.size) {
        return Ok(Answer::ok(format!("Pattern is {} byte(s), longer than {} ({} bytes).", pattern.len(), space.name, space.size)));
    }
    let bytes = read(&session, &space, 0, space.size)?;
    let mut hits: Vec<usize> = Vec::new();
    let last = space.size as usize - pattern.len();
    for addr in 0..=last {
        if pattern.iter().enumerate().all(|(i, p)| p.is_none_or(|b| bytes[addr + i] == b)) {
            hits.push(addr);
            if hits.len() as i64 > i64::from(max) {
                break;
            }
        }
    }
    if hits.is_empty() {
        return Ok(Answer::ok(format!("No match for {}-byte pattern in {}.", pattern.len(), space.name)));
    }
    let truncated = hits.len() as i64 > i64::from(max);
    let shown: Vec<String> = hits.iter().take(max as usize).map(|a| format!("  0x{a:X}")).collect();
    let header = if truncated {
        format!("More than {max} match(es) in {} - showing the first {max}:", space.name)
    } else {
        format!("{} match(es) for the {}-byte pattern in {}:", hits.len(), pattern.len(), space.name)
    };
    Ok(Answer::ok(format!("{header}\n{}", shown.join("\n"))))
}

fn in_logs(call: &Call, core: &str, file: &str, command: &str) -> Result<String, Answer> {
    emusen_session_service::words::in_logs(|p| call.resolve(p), core, file, command).map_err(Answer::fail)
}

pub fn dump(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() < 5 {
        return Ok(Answer::ok("Usage: dump <space> <addr> <len> <file>"));
    }
    let space = find(&session, &parts[1])?;
    let addr = parse_hex(&parts[2])?;
    let len = parse_hex(&parts[3])?;
    let file = &parts[4];
    if space.side_effects {
        return Ok(Answer::ok(side_effects(&space, "dump")));
    }
    if len < 0 {
        return Err("Arithmetic operation resulted in an overflow.".to_string());
    }
    let data = read(&session, &space, addr, len)?;
    let path = match in_logs(call, &core_name(&session)?, file, "dump") {
        Ok(path) => path,
        Err(refused) => return Ok(refused),
    };
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    write_all_bytes(&path, &data)?;
    Ok(Answer::ok(format!("Dumped {} 0x{addr:X}-0x{:X} ({len} bytes) to {path}", space.name, addr.wrapping_add(len).wrapping_sub(1))))
}

pub fn load(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() < 4 {
        return Ok(Answer::ok("Usage: load <space> <addr> <file>"));
    }
    let space = find(&session, &parts[1])?;
    let addr = parse_hex(&parts[2])?;
    let file = &parts[3];
    if !space.writable {
        return Ok(Answer::ok(format!("{} is read-only - refusing to load into it.", space.name)));
    }
    let path = match in_logs(call, &core_name(&session)?, file, "load") {
        Ok(path) => path,
        Err(refused) => return Ok(refused),
    };
    if !std::path::Path::new(&path).is_file() {
        return Ok(Answer::ok(format!("No file at {path}.")));
    }
    let data = read_all_bytes(&path)?;
    session.space_write(&space.name, addr as u32, &data)?;
    Ok(Answer::ok(format!("Loaded {} bytes from {path} into {} starting at 0x{addr:X}.", data.len(), space.name)))
}
