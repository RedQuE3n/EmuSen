//! `emusen_scrape_get`: one read, `{"what": …}`, as `emusen_scrape.h` lists them. See EmuSen_Backend.md §30.3.

use crate::json::{self, float_or_null, number_or_null, quoted, strings_into, text_or_null};
use crate::store::{Record, Stored};
use crate::{Error, Scraper, kinds, layout};
use emusen_dotnet::json::Value;

pub(crate) fn record_json(out: &mut String, r: &Record) {
    out.push_str("{\"md5\":");
    out.push_str(&quoted(&r.md5));
    out.push_str(&format!(",\"bytes\":{},\"state\":\"{}\",\"game_id\":", r.bytes, r.state.word()));
    number_or_null(out, r.game_id);
    out.push_str(",\"rom_id\":");
    number_or_null(out, r.rom_id);
    out.push_str(",\"system_id\":");
    number_or_null(out, r.system_id.map(i64::from));
    for (name, value) in [
        ("name", &r.name),
        ("description", &r.description),
        ("developer", &r.developer),
        ("publisher", &r.publisher),
        ("genre", &r.genre),
        ("players", &r.players),
    ] {
        out.push_str(&format!(",\"{name}\":"));
        text_or_null(out, value.as_deref());
    }
    out.push_str(",\"rating\":");
    float_or_null(out, r.rating);
    for (name, value) in [
        ("release_date", &r.release_date),
        ("region", &r.region),
        ("language", &r.language),
        ("matched_by", &r.matched_by),
        ("offered", &r.offered),
        ("detail", &r.detail),
    ] {
        out.push_str(&format!(",\"{name}\":"));
        text_or_null(out, value.as_deref());
    }
    out.push_str(",\"fetched_at\":");
    out.push_str(&quoted(&crate::store::stamp(r.fetched_at)));
    out.push_str(",\"kinds_known\":");
    text_or_null(out, r.kinds_known.as_deref());
    out.push('}');
}

fn stored_json(out: &mut String, media: &[Stored]) {
    out.push('[');
    for (i, m) in media.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"type\":");
        out.push_str(&quoted(&m.ty));
        out.push_str(",\"path\":");
        out.push_str(&quoted(&m.relative_path));
        out.push_str(",\"region\":");
        text_or_null(out, m.region.as_deref());
        out.push_str(",\"sha1\":");
        text_or_null(out, m.sha1.as_deref());
        out.push('}');
    }
    out.push(']');
}

/// The games a request names, `[{"path", "system"}]`.
pub(crate) fn games(r: &Value) -> Result<Vec<(String, String)>, Error> {
    match r.get("games") {
        Some(Value::Array(items)) => items.iter().map(|g| Ok((json::text(g, "path")?, json::text(g, "system")?))).collect(),
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(_) => Err(Error::BadRequest("The request's games is not a list.".into())),
    }
}

pub(crate) fn get(s: &Scraper, r: &Value) -> Result<String, Error> {
    let what = json::text(r, "what")?;
    let mut out = String::new();
    match what.as_str() {
        "kinds" => out.push_str(&kinds::to_json()),
        "record" => match s.store().found_for(&json::text(r, "path")?)? {
            Some(rec) => record_json(&mut out, &rec),
            None => out.push_str("null"),
        },
        "found" => {
            out.push('{');
            for (i, (path, rec)) in s.store().found_by_path()?.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&quoted(path));
                out.push(':');
                record_json(&mut out, rec);
            }
            out.push('}');
        }
        "outcomes" => {
            out.push('{');
            for (i, (path, state, cover)) in s.store().outcomes_by_path()?.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!("{}:{{\"state\":\"{}\",\"has_cover\":{cover}}}", quoted(path), state.word()));
            }
            out.push('}');
        }
        "stored" => stored_json(&mut out, &s.store().media(&json::text(r, "md5")?, json::integer(r, "bytes")?)?),
        "game" => {
            let (md5, bytes) = (json::text(r, "md5")?, json::integer(r, "bytes")?);
            match s.store().game(&md5, bytes)? {
                Some(rec) => record_json(&mut out, &rec),
                None => out.push_str("null"),
            }
        }
        "queue" => {
            out.push('[');
            for (i, q) in s.store().queue()?.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!("{{\"path\":{},\"system\":{},\"priority\":\"{}\",\"attempts\":{}}}", quoted(&q.path), quoted(&q.system), q.priority.word(), q.attempts));
            }
            out.push(']');
        }
        "moves" => {
            out.push('[');
            for (i, (from, to, how)) in s.store().moves()?.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!("{{\"from\":{},\"to\":{},\"how\":\"{how}\"}}", quoted(from), quoted(to)));
            }
            out.push(']');
        }
        "orphans" => {
            let settings = s.settings();
            let roms_there = settings.roms.as_deref().is_some_and(emusen_galaxia::dotnet_files::directory_exists);
            let found = if roms_there {
                let names = layout::names_by_system(&games(r)?, &|p| settings.folder_of(p));
                layout::orphans(&s.store(), &names)
            } else {
                Vec::new()
            };
            strings_into(&mut out, found);
        }
        "quota" => quota_json(s, &mut out)?,
        "account" => {
            let m = crate::credentials::Member::load();
            out.push_str(&format!("{{\"signed_in\":{},\"name\":", m.is_set()));
            text_or_null(&mut out, m.is_set().then_some(m.user.as_str()));
            out.push_str(",\"verified\":");
            text_or_null(&mut out, m.verified.map(crate::credentials::round_trip_utc).as_deref());
            out.push('}');
        }
        "developer" => {
            out.push_str("{\"origin\":");
            text_or_null(&mut out, s.developer().map(|d| d.origin.word()));
            out.push('}');
        }
        "folder" => out.push_str(&quoted(&s.settings().folder_of(&json::text(r, "path")?))),
        _ => return Err(Error::BadRequest(format!("The request's what, {what}, is not one the scraper answers."))),
    }
    Ok(out)
}

pub(crate) fn quota_json(s: &Scraper, out: &mut String) -> Result<(), Error> {
    let q = {
        let store = s.store();
        s.quota.snapshot(&store, &s.clock)?
    };
    out.push_str(&format!("{{\"day\":{},\"requests\":{},\"max_per_day\":", quoted(&q.day), q.requests));
    number_or_null(out, q.max_per_day.map(i64::from));
    out.push_str(&format!(",\"ko\":{},\"max_ko_per_day\":", q.ko));
    number_or_null(out, q.max_ko_per_day.map(i64::from));
    out.push_str(&format!(",\"threads\":{},\"per_minute\":", q.threads));
    number_or_null(out, q.per_minute.map(i64::from));
    out.push_str(&format!(",\"interval_ticks\":{},\"gate\":\"{}\",\"until\":", q.interval, q.gate.word()));
    text_or_null(out, q.until.map(crate::store::stamp).as_deref());
    out.push_str(",\"reason\":");
    text_or_null(out, q.reason.map(|w| crate::redact::redact(&w)).as_deref());
    out.push_str(&format!(",\"from_server\":{},\"max_kbps\":", q.from_server));
    number_or_null(out, q.max_kbps.map(i64::from));
    out.push('}');
    Ok(())
}
