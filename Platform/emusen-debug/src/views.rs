//! The chips' own views: `tile`, `tilemap`, `sprites`, `pal`, `channels` and `mute`. The core decodes; the command
//! formats (EmuSen_Debugging_Tools_Reference_v5.md §3.1a).

use crate::memory::find;
use crate::{core_name, session, Run};
use dianash_module::{Answer, Call};
use emusen_dotnet::number::parse_i32;
use emusen_dotnet::text::{equals_ignore_case, pad_left, pad_right, trim_end};
use emusen_session_service::words::parse_hex;
use emusen_session_service::Record;

/// A .NET `bool` as text.
fn boolean(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

pub fn tile(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 4 {
        return Ok(Answer::ok("Usage: tile <space> <addr> <bpp>"));
    }
    let session = session(call)?;
    let space = find(&session, &parts[1])?;
    let addr = parse_hex(&parts[2])?;
    let bpp = parse_hex(&parts[3])?;
    let pixels = session.tile_pixels(&space.name, addr as u32, bpp as u32)?;
    if pixels.len() < 64 {
        return Ok(Answer::fail(format!("{} decodes no tiles, so there is nothing to draw - see `man tile`.", core_name(&session)?)));
    }
    let mut out = format!("{} @ 0x{addr:X}, {bpp}bpp tile:\n", space.name);
    for row in 0..8 {
        out += "  ";
        for col in 0..8 {
            let value = pixels[row * 8 + col];
            if value == 0 {
                out += if bpp == 8 { ". " } else { "." };
            } else if bpp == 8 {
                out += &format!("{value:02X}");
            } else {
                out += &format!("{value:X}");
            }
            out.push(' ');
        }
        out.push('\n');
    }
    Ok(Answer::ok(trim_end(&out)))
}

pub fn tilemap(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    if parts.len() < 5 {
        return Ok(Answer::ok(crate::COMMANDS.iter().find(|(n, _, _)| *n == "tilemap").map_or("", |c| c.2)));
    }
    let space = find(&session, &parts[1])?;
    let addr = parse_hex(&parts[2])?;
    let cols = parse_hex(&parts[3])?;
    let rows = parse_hex(&parts[4])?;
    let stride = session.info()?.int("tilemap_entry_stride") as i32;
    if rows > 0 && cols < 0 {
        return Err("Specified argument was out of the range of valid values. (Parameter 'count')".to_string());
    }
    let mut out = String::new();
    for row in 0..rows.max(0) {
        let mut cells = Vec::new();
        for col in 0..cols.max(0) {
            let at = addr.wrapping_add(row.wrapping_mul(cols).wrapping_add(col).wrapping_mul(stride));
            cells.push(session.tilemap_entry(&space.name, at as u32)?.text("text"));
        }
        out += &cells.join(" ");
        out.push('\n');
    }
    Ok(Answer::ok(out.trim_end_matches('\n')))
}

pub fn sprites(call: &Call) -> Run {
    let session = session(call)?;
    let sprites = session.sprites()?;
    let list = sprites.list("sprites");
    let mut out = format!("{} active sprite(s):\n", list.len());
    for s in list {
        out += &format!(
            "  #{}: x={} y={} {}x{} tile=0x{:03X} pal={} pri={} flipX={} flipY={}\n",
            s.int("index"),
            s.int("x"),
            s.int("y"),
            s.int("width"),
            s.int("height"),
            s.int("tile") as i32,
            s.int("palette"),
            s.int("priority"),
            boolean(s.flag("flip_x")),
            boolean(s.flag("flip_y"))
        );
    }
    Ok(Answer::ok(trim_end(&out)))
}

pub fn pal(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    let palettes = session.palettes()?;
    let wanted = if parts.len() >= 2 { Some(i64::from(parse_hex(&parts[1])?)) } else { None };
    let mut out = String::new();
    for p in palettes.items().iter().filter(|p| wanted.is_none_or(|w| p.int("index") == w)) {
        let colors: Vec<String> = p
            .list("colors")
            .iter()
            .map(|c| {
                let rgb: Vec<i64> = c.items().iter().filter_map(emusen_session_service::number).collect();
                format!("{:02X}{:02X}{:02X}", rgb.first().copied().unwrap_or(0), rgb.get(1).copied().unwrap_or(0), rgb.get(2).copied().unwrap_or(0))
            })
            .collect();
        out += &format!("pal {}: {}\n", pad_left(&p.int("index").to_string(), 2), colors.join(" "));
    }
    Ok(Answer::ok(trim_end(&out)))
}

pub fn channels(call: &Call) -> Run {
    let session = session(call)?;
    let channels = session.audio_channels()?;
    if channels.items().is_empty() {
        return Ok(Answer::ok("(this core reports no audio channels)"));
    }
    let mut out = String::new();
    for c in channels.items() {
        out += &format!(
            "[{}] {} active={} level={} muted={} {}\n",
            c.int("index"),
            pad_right(&c.text("name"), 8),
            pad_right(boolean(c.flag("active")), 5),
            pad_left(&c.int("level").to_string(), 3),
            pad_right(boolean(c.flag("muted")), 5),
            c.text("info")
        );
    }
    Ok(Answer::ok(out.trim_end_matches('\n')))
}

pub fn mute(call: &Call, parts: &[String]) -> Run {
    let session = session(call)?;
    let usage = crate::COMMANDS.iter().find(|(n, _, _)| *n == "mute").map_or("", |c| c.2);
    if parts.len() < 3 {
        return Ok(Answer::ok(usage));
    }
    let Some(index) = parse_i32(&parts[1]) else { return Ok(Answer::ok(usage)) };
    let on = equals_ignore_case(&parts[2], "on");
    session.set_channel_muted(index as u32, on)?;
    Ok(Answer::ok(format!("Channel {index} {}.", if on { "muted" } else { "unmuted" })))
}
