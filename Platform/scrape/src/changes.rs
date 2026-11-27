//! The service's `change`: one change, `{"op": …}`, as `emusen_scrape_service.h` lists them. See EmuSen_LibraryTools.md §8.

pub(crate) use emusen_galaxia::clock::local_offset;
use crate::json::{self, quoted, strings_into};
use crate::{Error, ScraperInner, credentials, layout};
use serde_json::Value;
use std::sync::Arc;

/// The instant a request names in `at` (a round-trip text with its offset) with that offset, else now on the handle's
/// clock with the machine's: the UTC ticks and the offset in seconds.
fn given_now(s: &ScraperInner, r: &Value) -> Result<(i64, i64), Error> {
    if let Some(at) = json::text_or_none(r, "at")? {
        let utc = emusen_galaxia::clock::instant(&at).ok_or_else(|| Error::BadRequest("The request's at is not a time.".into()))?;
        return Ok((utc, offset_written(&at)));
    }
    let utc = s.clock.now();
    Ok((utc, local_offset(utc / emusen_galaxia::clock::TICKS_PER_SECOND)))
}

fn offset_written(text: &str) -> i64 {
    let b = text.as_bytes();
    let tail = &b[b.len().saturating_sub(6)..];
    if tail.len() == 6 && (tail[0] == b'+' || tail[0] == b'-') && tail[3] == b':' {
        let parse2 = |s: &[u8]| std::str::from_utf8(s).ok().and_then(|t| t.parse::<i64>().ok()).unwrap_or(0);
        let minutes = parse2(&tail[1..3]) * 60 + parse2(&tail[4..6]);
        return if tail[0] == b'-' { -minutes * 60 } else { minutes * 60 };
    }
    0
}

pub(crate) fn change(s: &Arc<ScraperInner>, r: &Value) -> Result<String, Error> {
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
            let tick = emusen_galaxia::clock::TICKS_PER_SECOND;
            let machine = utc + local_offset(utc.div_euclid(tick)) * tick;
            let (moved, bytes, folder) = layout::clean_up(&s.store(), &relatives, machine, utc + offset * tick, offset)?;
            out.push_str(&format!("{{\"moved\":{moved},\"bytes\":{bytes},\"folder\":{}}}", quoted(&folder)));
        }
        "sign_out" => {
            let had = credentials::Member::delete().map_err(|e| Error::Io(format!("{}: {}", credentials::Member::path(), e)))?;
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
        "cover" => {
            let file = json::text(r, "path")?;
            let folder_path = match json::text_or_none(r, "folder_path")? {
                Some(f) => f,
                None => s.settings().folder_of(&file),
            };
            let game = crate::media::Game { file: &file, folder_path, folder: false };
            s.choose_cover(&json::text(r, "system")?, &game, json::text_or_none(r, "type")?.as_deref(), json::text_or_none(r, "at")?.as_deref())?;
            out.push_str("true");
        }
        "playing" => {
            s.set_playing(json::flag(r, "running", false)?);
            out.push_str("true");
        }
        "openvgdb_download" => out.push_str(&s.openvgdb_download()?),
        "openvgdb_remove" => out.push_str(&s.openvgdb_remove()?),
        _ => return Err(Error::BadRequest(format!("The request's op, {op}, is not one the scraper makes."))),
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::offset_written;

    #[test]
    fn offset_written_handles_multibyte_utf8_at_tail_boundary() {
        assert_eq!(offset_written("2026-01-01T00:00:00é"), 0);
        assert_eq!(offset_written("2026-01-01T00:00:00🦀"), 0);
        assert_eq!(offset_written("2026-01-01T00:00:00+02:30"), 9000);
        assert_eq!(offset_written("2026-01-01T00:00:00-05:00"), -18000);
    }
}
