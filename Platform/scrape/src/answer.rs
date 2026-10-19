//! Every reading of ScreenScraper's JSON, `ScreenScraperJson` ported, because API v2 is a beta that may change. The
//! text is read as `JsonDocument.Parse` reads it (`emusen_dotnet::json_document`), and each refusal the C# catches as
//! a malformed answer is given in the words of the exception the C# saw first, since a fifth malformed answer is kept
//! in `media.db` with them. Members are read in the C#'s order for that reason. See EmuSen_BigPicture.md §5.1, §17.

use emusen_dotnet::json::Value;
use emusen_dotnet::json_document;
use emusen_dotnet::number::parse_i64;

/// A text keyed by a region or a language, as ScreenScraper lists names, synopses, dates and genre names.
#[derive(Clone, Debug, PartialEq)]
pub struct Localised {
    pub key: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Genre {
    pub names: Vec<Localised>,
    pub main: bool,
}

/// One media file the game has, with the address ScreenScraper serves it from; the address carries the credentials.
#[derive(Clone, Debug, PartialEq)]
pub struct Media {
    pub ty: String,
    pub region: Option<String>,
    pub url: String,
    pub format: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<i64>,
}

/// The member's limits and today's counts, as every response carries them; none where a field was absent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Quota {
    pub max_threads: Option<i32>,
    pub max_download_kbps: Option<i32>,
    pub requests_today: Option<i32>,
    pub requests_ko_today: Option<i32>,
    pub max_requests_per_minute: Option<i32>,
    pub max_requests_per_day: Option<i32>,
    pub max_requests_ko_per_day: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub id: i64,
    pub rom_id: Option<i64>,
    pub system_id: Option<i32>,
    pub names: Vec<Localised>,
    pub synopses: Vec<Localised>,
    pub dates: Vec<Localised>,
    pub genres: Vec<Genre>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub players: Option<String>,
    pub note: Option<f64>,
    pub media: Vec<Media>,
}

/// The member `ssuserInfos` names as a sign-in: the login name, the level and the quota.
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub name: String,
    pub level: Option<String>,
    pub quota: Quota,
}

type Read<T> = Result<T, String>;

/// `JsonValueKind`'s name, as an `InvalidOperationException` names the kind it found.
fn kind(v: &Value) -> &'static str {
    match v {
        Value::Object(_) => "Object",
        Value::Array(_) => "Array",
        Value::String(_) => "String",
        Value::Number(_) => "Number",
        Value::Bool(true) => "True",
        Value::Bool(false) => "False",
        Value::Null => "Null",
    }
}

fn not_object(v: &Value) -> String {
    format!("The requested operation requires an element of type 'Object', but the target element has type '{}'.", kind(v))
}

/// `GetProperty`: the member, or `KeyNotFoundException`'s words; an element not an object refused.
fn get_property<'a>(v: &'a Value, name: &str) -> Read<&'a Value> {
    match v {
        Value::Object(_) => v.get(name).ok_or_else(|| "The given key was not present in the dictionary.".to_string()),
        _ => Err(not_object(v)),
    }
}

/// `TryGetProperty`: the member or none; an element not an object refused.
fn try_get_property<'a>(v: &'a Value, name: &str) -> Read<Option<&'a Value>> {
    match v {
        Value::Object(_) => Ok(v.get(name)),
        _ => Err(not_object(v)),
    }
}

/// The C#'s `Prop`: a member of an object, else nothing.
fn prop<'a>(v: Option<&'a Value>, name: &str) -> Option<&'a Value> {
    match v {
        Some(o @ Value::Object(_)) => o.get(name),
        _ => None,
    }
}

fn array(v: Option<&Value>) -> &[Value] {
    match v {
        Some(Value::Array(items)) => items,
        _ => &[],
    }
}

/// The C#'s `Text`: a string's text, a number's token, else none. A string holding an escaped lone surrogate is refused
/// as `JsonElement.GetString` refuses it.
fn text(v: Option<&Value>) -> Read<Option<String>> {
    match v {
        Some(Value::String(t)) if t.broken => Err(broken_words(t)),
        Some(Value::String(t)) => Ok(Some(t.text.clone())),
        Some(Value::Number(token)) => Ok(Some(token.clone())),
        _ => Ok(None),
    }
}

fn broken_words(t: &emusen_dotnet::json::Text) -> String {
    t.refusal.clone().unwrap_or_else(|| "Cannot read incomplete UTF-16 JSON text as string with missing low surrogate.".to_string())
}

fn long(v: Option<&Value>) -> Read<Option<i64>> {
    Ok(text(v)?.as_deref().and_then(parse_i64))
}

fn int(v: Option<&Value>) -> Read<Option<i32>> {
    Ok(text(v)?.as_deref().and_then(emusen_dotnet::number::parse_i32))
}

fn double(v: Option<&Value>) -> Read<Option<f64>> {
    Ok(text(v)?.as_deref().and_then(emusen_dotnet::number::parse_f64_invariant_float))
}

fn quota(user: &Value) -> Read<Quota> {
    let u = Some(user);
    Ok(Quota {
        max_threads: int(prop(u, "maxthreads"))?,
        max_download_kbps: int(prop(u, "maxdownloadspeed"))?,
        requests_today: int(prop(u, "requeststoday"))?,
        requests_ko_today: int(prop(u, "requestskotoday"))?,
        max_requests_per_minute: int(prop(u, "maxrequestspermin"))?,
        max_requests_per_day: int(prop(u, "maxrequestsperday"))?,
        max_requests_ko_per_day: int(prop(u, "maxrequestskoperday"))?,
    })
}

fn list(items: Option<&Value>, key: &str) -> Read<Vec<Localised>> {
    let mut out = Vec::new();
    for item in array(items) {
        let key = text(prop(Some(item), key))?.unwrap_or_default();
        let text = text(prop(Some(item), "text"))?;
        if let Some(text) = text.filter(|t| !emusen_dotnet::text::is_blank(Some(t))) {
            out.push(Localised { key, text });
        }
    }
    Ok(out)
}

fn game(jeu: &Value) -> Read<Option<Game>> {
    let j = Some(jeu);
    let Some(id) = long(prop(j, "id"))? else { return Ok(None) };
    let rom_id = match long(prop(j, "romid"))? {
        Some(r) => Some(r),
        None => long(prop(prop(j, "rom"), "id"))?,
    };
    let system_id = int(prop(prop(j, "systeme"), "id"))?;
    let names = list(prop(j, "noms"), "region")?;
    let synopses = list(prop(j, "synopsis"), "langue")?;
    let dates = list(prop(j, "dates"), "region")?;
    let mut genres = Vec::new();
    for g in array(prop(j, "genres")) {
        let names = list(prop(Some(g), "noms"), "langue")?;
        let main = text(prop(Some(g), "principale"))?.as_deref() == Some("1");
        genres.push(Genre { names, main });
    }
    let developer = text(prop(prop(j, "developpeur"), "text"))?;
    let publisher = text(prop(prop(j, "editeur"), "text"))?;
    let players = text(prop(prop(j, "joueurs"), "text"))?;
    let note = double(prop(prop(j, "note"), "text"))?;
    let mut media = Vec::new();
    for m in array(prop(j, "medias")) {
        let m = Some(m);
        if text(prop(m, "type"))?.is_none() || text(prop(m, "url"))?.is_none() {
            continue;
        }
        let ty = text(prop(m, "type"))?.unwrap_or_default();
        let region = text(prop(m, "region"))?;
        let url = text(prop(m, "url"))?.unwrap_or_default();
        let format = text(prop(m, "format"))?;
        let sha1 = text(prop(m, "sha1"))?.map(|s| emusen_dotnet::text::to_lower(&s));
        let size = long(prop(m, "size"))?;
        media.push(Media { ty, region, url, format, sha1, size });
    }
    Ok(Some(Game { id, rom_id, system_id, names, synopses, dates, genres, developer, publisher, players, note, media }))
}

fn response(json: &str) -> Read<Value> {
    let root = json_document::parse(json)?;
    Ok(get_property(&root, "response")?.clone())
}

fn ssuser(response: &Value) -> Read<Option<Quota>> {
    match try_get_property(response, "ssuser")? {
        Some(user) => Ok(Some(quota(user)?)),
        None => Ok(None),
    }
}

/// The game and the quota of a `jeuInfos` answer; an error is the words of what the C# threw.
pub fn jeu_infos(json: &str) -> Read<(Option<Game>, Option<Quota>)> {
    let response = response(json)?;
    let quota = ssuser(&response)?;
    match try_get_property(&response, "jeu")? {
        Some(jeu @ Value::Object(_)) => match game(jeu)? {
            Some(g) => Ok((Some(g), quota)),
            None => Err("a game without an id".to_string()),
        },
        _ => Ok((None, quota)),
    }
}

/// `jeuRecherche`: up to 30 games ranked by likelihood; an entry with no id is left out.
pub fn jeu_recherche(json: &str) -> Read<(Vec<Game>, Option<Quota>)> {
    let response = response(json)?;
    let quota = ssuser(&response)?;
    let mut games = Vec::new();
    for j in array(prop(Some(&response), "jeux")) {
        if matches!(j, Value::Object(_)) {
            if let Some(g) = game(j)? {
                games.push(g);
            }
        }
    }
    Ok((games, quota))
}

/// `ssuserInfos`: only the quota.
pub fn user(json: &str) -> Read<Option<Quota>> {
    let response = response(json)?;
    ssuser(&response)
}

/// `ssuserInfos` as a sign-in: the member, or none when the answer names none.
pub fn member(json: &str) -> Read<Option<Member>> {
    let response = response(json)?;
    let Some(user @ Value::Object(_)) = try_get_property(&response, "ssuser")? else { return Ok(None) };
    let name = text(prop(Some(user), "id"))?;
    let Some(name) = name.filter(|n| !emusen_dotnet::text::is_blank(Some(n))) else { return Ok(None) };
    let level = text(prop(Some(user), "niveau"))?;
    Ok(Some(Member { name, level, quota: quota(user)? }))
}

/// `systemesListe`: each system's id and its name, a later entry of an id replacing an earlier one.
pub fn systems(json: &str) -> Read<Vec<(i32, String)>> {
    let response = response(json)?;
    let mut out: Vec<(i32, String)> = Vec::new();
    for s in array(prop(Some(&response), "systemes")) {
        let Some(id) = int(prop(Some(s), "id"))? else { continue };
        let names = prop(Some(s), "noms");
        let name = match text(prop(names, "nom_eu"))? {
            Some(n) => n,
            None => match text(prop(names, "nom_us"))? {
                Some(n) => n,
                None => text(prop(names, "nom_recalbox"))?.unwrap_or_else(|| id.to_string()),
            },
        };
        match out.iter_mut().find(|(i, _)| *i == id) {
            Some(entry) => entry.1 = name,
            None => out.push((id, name)),
        }
    }
    Ok(out)
}
