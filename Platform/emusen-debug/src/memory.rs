//! The memory spaces: `spaces`, `mem`, `write`, `memfind`, `dump` and `load`, as their C# classes worded them.

use crate::{core_name, session, Run};
use dianash_module::{Answer, Call};
use emusen_session_service::words::{encode_bytes, hex_digits, parse_hex, parse_value_bits, split_common_flags};
pub(crate) use emusen_session_service::space::{all, find, read, Space};
use serde_json::{json, Value};

/// The words every bulk read refuses a space with live hardware behind it in.
pub(crate) fn side_effects(space: &Space, what: &str) -> String {
    format!("{} can have real side effects on read (live hardware registers) - refusing a bulk {what} there. Try WRAM (or another plain-memory space) instead.", space.name)
}

pub fn spaces(call: &Call) -> Run {
    let session = session(call)?;
    let mut out = format!("{} memory spaces:\n", core_name(&session)?);
    for s in all(&session)? {
        out += &format!("  {} {} bytes  {}\n", format!("{:<10}", s.name), format!("{:>8}", s.size.to_string()), if s.writable { "R/W" } else { "R/O" });
    }
    Ok(Answer::ok(out.trim_end()))
}

pub fn mem(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 3 {
        return Ok(Answer::ok("Usage: mem <space> <addr> [<len>]"));
    }
    let (pos, flags) = split_common_flags(&parts[1..])?;
    if pos.len() < 2 {
        return Ok(Answer::ok("Usage: mem <space> <addr> [<len>]"));
    }
    let session = session(call)?;
    let space = find(&session, &pos[0])?;
    let addr = parse_hex(&pos[1])?;
    let len = if pos.len() >= 3 { parse_hex(&pos[2])? } else { 16 };
    let bytes = read(&session, &space, addr, len)?;
    if flags.json {
        let digits = hex_digits(space.size);
        let mut rows = Vec::new();
        let mut row = 0i32;
        while row < len {
            let row_addr = addr.wrapping_add(row) as u32;
            let mut row_bytes: Vec<Option<u8>> = Vec::with_capacity(16);
            let mut ascii = String::new();
            for col in 0..16 {
                let idx = row + col;
                if idx < len && (idx as usize) < bytes.len() {
                    let b = bytes[idx as usize];
                    row_bytes.push(Some(b));
                    ascii.push(if (0x20..0x7F).contains(&b) { b as char } else { '.' });
                } else {
                    row_bytes.push(None);
                }
            }
            rows.push(json!({"address": row_addr, "addr_hex": format!("{row_addr:0digits$X}"), "bytes": row_bytes, "ascii": ascii}));
            row += 16;
        }
        return Ok(Answer::ok(json!({"space": space.name, "address": addr as u32, "rows": rows}).to_string()));
    }
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
    Ok(Answer::ok(out.trim_end()))
}

pub fn write(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 4 {
        return Ok(Answer::ok("Usage: write <space> <addr> <value> [-w 1|2|4] [--le|--be] [--dec|--float] [--json]"));
    }
    let (pos, flags) = split_common_flags(&parts[1..])?;
    if pos.len() < 3 {
        return Ok(Answer::ok("Usage: write <space> <addr> <value> [-w 1|2|4] [--le|--be] [--dec|--float] [--json]"));
    }
    let session = session(call)?;
    let space = find(&session, &pos[0])?;
    if !space.writable {
        return Ok(Answer::ok(format!("{} is read-only.", space.name)));
    }
    let addr = parse_hex(&pos[1])?;
    let default_w = if flags.float { 4 } else { 1 };
    let width = flags.width.unwrap_or(default_w);
    let decimal = flags.decimal.unwrap_or(flags.float);
    let big_endian = flags.endian.unwrap_or(space.byte_order).is_big();
    let raw_val = if width == 1 && !flags.float && !decimal && flags.width.is_none() {
        u32::from(parse_hex(&pos[2])? as u8)
    } else {
        parse_value_bits(&pos[2], width, decimal, flags.float)?
    };
    let encoded = encode_bytes(raw_val, width, big_endian);
    session.space_write(&space.name, addr as u32, &encoded)?;
    if flags.json {
        return Ok(Answer::ok(json!({
            "space": space.name,
            "address": addr as u32,
            "value": raw_val,
            "width": width,
            "big_endian": big_endian,
            "bytes": encoded,
        }).to_string()));
    }
    if width == 1 && !flags.float && flags.width.is_none() {
        Ok(Answer::ok(format!("{}[0x{addr:X}] = 0x{:02X}", space.name, encoded[0])))
    } else {
        let digits = (width as usize) * 2;
        let order = if big_endian { "BE" } else { "LE" };
        Ok(Answer::ok(format!("{}[0x{addr:X}] = 0x{raw_val:0digits$X} ({width}-byte {order})", space.name)))
    }
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
    let (pos, flags) = split_common_flags(&parts[1..])?;
    if pos.len() < 2 {
        return Ok(Answer::ok("Usage: memfind <space> <bytes> [<max>]"));
    }
    let session = session(call)?;
    let space = find(&session, &pos[0])?;
    if space.side_effects {
        return Ok(Answer::ok(side_effects(&space, "scan")));
    }
    let mut max = 20;
    let mut words: Vec<String> = pos[1..].to_vec();
    if words.len() >= 2 {
        if let Some(parsed) = words.last().unwrap().trim().parse::<i32>().ok().filter(|&p| p > 0) {
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
    let truncated = hits.len() as i64 > i64::from(max);
    if flags.json {
        let digits = hex_digits(space.size);
        let shown: Vec<Value> = hits
            .iter()
            .take(max as usize)
            .map(|&a| json!({"address": a as u32, "addr_hex": format!("{a:0digits$X}"), "bytes": &bytes[a..a + pattern.len()]}))
            .collect();
        return Ok(Answer::ok(json!({
            "space": space.name,
            "count": shown.len(),
            "truncated": truncated,
            "hits": shown,
        }).to_string()));
    }
    if hits.is_empty() {
        return Ok(Answer::ok(format!("No match for {}-byte pattern in {}.", pattern.len(), space.name)));
    }
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
    std::fs::write(&path, &data).map_err(|e| format!("{}: {e}", path))?;
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
    let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path))?;
    session.space_write(&space.name, addr as u32, &data)?;
    Ok(Answer::ok(format!("Loaded {} bytes from {path} into {} starting at 0x{addr:X}.", data.len(), space.name)))
}
