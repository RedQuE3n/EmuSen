//! `emusen_scrape_change`: one change, `{"op": …}`, as `emusen_scrape.h` lists them. See EmuSen_Backend.md §30.3.

use crate::json::{self, quoted, strings_into};
use crate::{Error, Scraper, credentials, layout};
use emusen_dotnet::json::Value;

/// The instant a request names in `at` (a round-trip text with its offset) with that offset, else now on the handle's
/// clock with the machine's: the UTC ticks and the offset in seconds.
fn given_now(s: &Scraper, r: &Value) -> Result<(i64, i64), Error> {
    if let Some(at) = json::text_or_none(r, "at")? {
        let utc = emusen_dotnet::date::instant(&at).ok_or_else(|| Error::BadRequest("The request's at is not a time.".into()))?;
        return Ok((utc, offset_written(&at)));
    }
    let utc = s.clock.now();
    Ok((utc, local_offset(utc / emusen_dotnet::date::TICKS_PER_SECOND)))
}

fn offset_written(text: &str) -> i64 {
    let tail = &text[text.len().saturating_sub(6)..];
    let b = tail.as_bytes();
    if b.len() == 6 && (b[0] == b'+' || b[0] == b'-') && b[3] == b':' {
        let minutes = tail[1..3].parse::<i64>().unwrap_or(0) * 60 + tail[4..6].parse::<i64>().unwrap_or(0);
        return if b[0] == b'-' { -minutes * 60 } else { minutes * 60 };
    }
    0
}

#[cfg(unix)]
pub(crate) fn local_offset(seconds: i64) -> i64 {
    let t = seconds as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&t, &mut tm) }.is_null() { 0 } else { tm.tm_gmtoff as i64 }
}

#[cfg(not(unix))]
pub(crate) fn local_offset(_: i64) -> i64 {
    0
}

pub(crate) fn change(s: &Scraper, r: &Value) -> Result<String, Error> {
    let op = json::text(r, "op")?;
    let mut out = String::new();
    match op.as_str() {
        "settings" => {
            s.apply_settings(r)?;
            out.push_str("true");
        }
        "forget" => {
            let rom = json::text(r, "path")?;
            let system = json::text_or_none(r, "system")?;
            let folder = match json::text_or_none(r, "folder")? {
                Some(f) => f,
                None => s.settings().folder_of(&rom),
            };
            let deleted = layout::forget(&s.store(), &rom, system.as_deref(), &folder)?;
            strings_into(&mut out, deleted);
        }
        "forget_unfound" => {
            s.store().forget_unfound(&json::text(r, "path")?)?;
            out.push_str("true");
        }
        "clean_up" => {
            let relatives = json::strings(r, "relatives")?;
            let (utc, offset) = given_now(s, r)?;
            let tick = emusen_dotnet::date::TICKS_PER_SECOND;
            let machine = utc + local_offset(utc.div_euclid(tick)) * tick;
            let (moved, bytes, folder) = layout::clean_up(&s.store(), &relatives, machine, utc + offset * tick, offset)?;
            out.push_str(&format!("{{\"moved\":{moved},\"bytes\":{bytes},\"folder\":{}}}", quoted(&folder)));
        }
        "sign_out" => {
            let had = credentials::Member::delete().map_err(|e| Error::Io(emusen_dotnet::file::io_message(&e, &credentials::Member::path())))?;
            s.set_run_member(None);
            out.push_str(&format!("{{\"had\":{had}}}"));
        }
        "sign_in" => {
            let user = json::text(r, "user")?;
            out.push_str(&s.sign_in(credentials::Member::new(user.trim_matches(char::is_whitespace), &json::text(r, "password")?))?);
        }
        "check" => out.push_str(&s.sign_in(credentials::Member::load())?),
        "search" => out.push_str(&s.search(r)?),
        "art_added" => {
            s.add_art(json::text_or_none(r, "console")?, &json::text(r, "stem")?, &json::text(r, "path")?);
            out.push_str("true");
        }
        "openvgdb_download" => out.push_str(&s.openvgdb_download()?),
        "openvgdb_remove" => out.push_str(&s.openvgdb_remove()?),
        _ => return Err(Error::BadRequest(format!("The request's op, {op}, is not one the scraper makes."))),
    }
    Ok(out)
}
