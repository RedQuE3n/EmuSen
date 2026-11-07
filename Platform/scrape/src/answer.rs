//! Every reading of ScreenScraper's JSON, because API v2 is a beta that may change: the answer read into typed
//! structures by serde, leniently, as the C# `ScreenScraperJson` read it. A member of the wrong kind is absent, a list
//! that is not one is empty, and a leaf is a string's text or a number's digits. What does not read is refused with
//! serde's words, which a fifth malformed answer keeps in `media.db`. See EmuSen_BigPicture.md §5.1, §17 and
//! EmuSen_RustPlatform.md §47.10.

use serde::Deserialize;
use serde::de::{Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor, value::MapAccessDeserializer};

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

/// Drains whatever a lenient reader passes over, so that what it holds is never decoded.
fn drain_seq<'de, A: SeqAccess<'de>>(mut seq: A) -> Result<(), A::Error> {
    while seq.next_element::<IgnoredAny>()?.is_some() {}
    Ok(())
}

fn drain_map<'de, A: MapAccess<'de>>(mut map: A) -> Result<(), A::Error> {
    while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
    Ok(())
}

/// A leaf: a string's text or a number's digits (a fraction always shown, so `77.0` is no whole number); none for any other kind.
#[derive(Default)]
struct Text(Option<String>);

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Text, D::Error> {
        struct Leaf;
        impl<'de> Visitor<'de> for Leaf {
            type Value = Text;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("any value")
            }
            fn visit_str<E>(self, v: &str) -> Result<Text, E> {
                Ok(Text(Some(v.to_string())))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Text, E> {
                Ok(Text(Some(v.to_string())))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Text, E> {
                Ok(Text(Some(v.to_string())))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Text, E> {
                Ok(Text(Some(format!("{v:?}"))))
            }
            fn visit_bool<E>(self, _: bool) -> Result<Text, E> {
                Ok(Text(None))
            }
            fn visit_unit<E>(self) -> Result<Text, E> {
                Ok(Text(None))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Text, A::Error> {
                drain_seq(seq).map(|()| Text(None))
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Text, A::Error> {
                drain_map(map).map(|()| Text(None))
            }
        }
        d.deserialize_any(Leaf)
    }
}

impl Text {
    fn long(&self) -> Option<i64> {
        self.0.as_deref().and_then(|t| t.trim().parse().ok())
    }
    fn int(&self) -> Option<i32> {
        self.0.as_deref().and_then(|t| t.trim().parse().ok())
    }
    fn double(&self) -> Option<f64> {
        self.0.as_deref().and_then(|t| t.trim().parse().ok())
    }
}

/// An object read as `T`, and none for a member of any other kind.
struct Object<T>(Option<T>);

impl<T> Default for Object<T> {
    fn default() -> Self {
        Object(None)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Object<T>, D::Error> {
        struct Only<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for Only<T> {
            type Value = Object<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("any value")
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Object<T>, A::Error> {
                T::deserialize(MapAccessDeserializer::new(map)).map(|t| Object(Some(t)))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Object<T>, A::Error> {
                drain_seq(seq).map(|()| Object(None))
            }
            fn visit_str<E>(self, _: &str) -> Result<Object<T>, E> {
                Ok(Object(None))
            }
            fn visit_i64<E>(self, _: i64) -> Result<Object<T>, E> {
                Ok(Object(None))
            }
            fn visit_u64<E>(self, _: u64) -> Result<Object<T>, E> {
                Ok(Object(None))
            }
            fn visit_f64<E>(self, _: f64) -> Result<Object<T>, E> {
                Ok(Object(None))
            }
            fn visit_bool<E>(self, _: bool) -> Result<Object<T>, E> {
                Ok(Object(None))
            }
            fn visit_unit<E>(self) -> Result<Object<T>, E> {
                Ok(Object(None))
            }
        }
        d.deserialize_any(Only(std::marker::PhantomData))
    }
}

/// A list's objects read as `T`, its other elements passed over (or, where `EVERY`, each read as an empty `T`); empty for a member that is no list.
struct Many<T, const EVERY: bool = false>(Vec<T>);

impl<T, const EVERY: bool> Default for Many<T, EVERY> {
    fn default() -> Self {
        Many(Vec::new())
    }
}

impl<'de, T: Deserialize<'de> + Default, const EVERY: bool> Deserialize<'de> for Many<T, EVERY> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Many<T, EVERY>, D::Error> {
        struct List<T, const EVERY: bool>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de> + Default, const EVERY: bool> Visitor<'de> for List<T, EVERY> {
            type Value = Many<T, EVERY>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("any value")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Many<T, EVERY>, A::Error> {
                let mut out = Vec::new();
                while let Some(Object(item)) = seq.next_element::<Object<T>>()? {
                    out.extend(if EVERY { Some(item.unwrap_or_default()) } else { item });
                }
                Ok(Many(out))
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Many<T, EVERY>, A::Error> {
                drain_map(map).map(|()| Many(Vec::new()))
            }
            fn visit_str<E>(self, _: &str) -> Result<Many<T, EVERY>, E> {
                Ok(Many(Vec::new()))
            }
            fn visit_i64<E>(self, _: i64) -> Result<Many<T, EVERY>, E> {
                Ok(Many(Vec::new()))
            }
            fn visit_u64<E>(self, _: u64) -> Result<Many<T, EVERY>, E> {
                Ok(Many(Vec::new()))
            }
            fn visit_f64<E>(self, _: f64) -> Result<Many<T, EVERY>, E> {
                Ok(Many(Vec::new()))
            }
            fn visit_bool<E>(self, _: bool) -> Result<Many<T, EVERY>, E> {
                Ok(Many(Vec::new()))
            }
            fn visit_unit<E>(self) -> Result<Many<T, EVERY>, E> {
                Ok(Many(Vec::new()))
            }
        }
        d.deserialize_any(List(std::marker::PhantomData))
    }
}

/// A member that is there, whatever it holds: `null` included, which `Option` alone would read as absent.
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

/// An answer: its `response`, which each reader reads as its own structure, so that a reader reads only what it uses.
#[derive(Deserialize)]
#[serde(bound = "R: Deserialize<'de>")]
struct Envelope<R> {
    response: Object<R>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Infos {
    #[serde(deserialize_with = "present")]
    ssuser: Option<Object<User>>,
    jeu: Object<Jeu>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Search {
    #[serde(deserialize_with = "present")]
    ssuser: Option<Object<User>>,
    jeux: Many<Jeu>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Users {
    #[serde(deserialize_with = "present")]
    ssuser: Option<Object<User>>,
}

/// A sign-in's name alone, read before anything else the member holds.
#[derive(Default, Deserialize)]
#[serde(default)]
struct Named {
    ssuser: Object<Id>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Systems {
    systemes: Many<Systeme>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct User {
    id: Text,
    niveau: Text,
    maxthreads: Text,
    maxdownloadspeed: Text,
    requeststoday: Text,
    requestskotoday: Text,
    maxrequestspermin: Text,
    maxrequestsperday: Text,
    maxrequestskoperday: Text,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Jeu {
    id: Text,
    romid: Text,
    rom: Object<Id>,
    systeme: Object<Id>,
    noms: Many<ByRegion>,
    synopsis: Many<ByLanguage>,
    dates: Many<ByRegion>,
    genres: Many<GenreEntry, true>,
    developpeur: Object<Texted>,
    editeur: Object<Texted>,
    joueurs: Object<Texted>,
    note: Object<Texted>,
    medias: Many<MediaEntry>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Id {
    id: Text,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Texted {
    text: Text,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct ByRegion {
    region: Text,
    text: Text,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct ByLanguage {
    langue: Text,
    text: Text,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct GenreEntry {
    noms: Many<ByLanguage>,
    principale: Text,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct MediaEntry {
    #[serde(rename = "type")]
    ty: Text,
    region: Text,
    url: Text,
    format: Text,
    sha1: Text,
    size: Text,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Systeme {
    id: Text,
    noms: Object<SystemNames>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct SystemNames {
    nom_eu: Text,
    nom_us: Text,
    nom_recalbox: Text,
}

fn localised(key: Text, text: Text) -> Option<Localised> {
    text.0.filter(|t| !t.trim().is_empty()).map(|text| Localised { key: key.0.unwrap_or_default(), text })
}

impl User {
    fn quota(&self) -> Quota {
        Quota {
            max_threads: self.maxthreads.int(),
            max_download_kbps: self.maxdownloadspeed.int(),
            requests_today: self.requeststoday.int(),
            requests_ko_today: self.requestskotoday.int(),
            max_requests_per_minute: self.maxrequestspermin.int(),
            max_requests_per_day: self.maxrequestsperday.int(),
            max_requests_ko_per_day: self.maxrequestskoperday.int(),
        }
    }
}

fn text_of(field: Object<Texted>) -> Option<String> {
    field.0.and_then(|t| t.text.0)
}

impl Jeu {
    fn game(self) -> Option<Game> {
        let id = self.id.long()?;
        let rom_id = self.romid.long().or_else(|| self.rom.0.and_then(|r| r.id.long()));
        let system_id = self.systeme.0.and_then(|s| s.id.int());
        let genres = self.genres.0.into_iter().map(|g| Genre { names: g.noms.0.into_iter().filter_map(|n| localised(n.langue, n.text)).collect(), main: g.principale.0.as_deref() == Some("1") }).collect();
        let media = self
            .medias
            .0
            .into_iter()
            .filter_map(|m| {
                let (ty, url) = (m.ty.0?, m.url.0?);
                Some(Media { ty, region: m.region.0, url, format: m.format.0, sha1: m.sha1.0.map(|s| s.to_lowercase()), size: m.size.long() })
            })
            .collect();
        Some(Game {
            id,
            rom_id,
            system_id,
            names: self.noms.0.into_iter().filter_map(|n| localised(n.region, n.text)).collect(),
            synopses: self.synopsis.0.into_iter().filter_map(|n| localised(n.langue, n.text)).collect(),
            dates: self.dates.0.into_iter().filter_map(|n| localised(n.region, n.text)).collect(),
            genres,
            developer: text_of(self.developpeur),
            publisher: text_of(self.editeur),
            players: text_of(self.joueurs),
            note: self.note.0.and_then(|t| t.text.double()),
            media,
        })
    }
}

/// The answer's `response` as `R`, none where it is no object; serde's words where the answer does not read.
fn response<R: for<'de> Deserialize<'de>>(json: &str) -> Read<Option<R>> {
    serde_json::from_str::<Envelope<R>>(json).map(|e| e.response.0).map_err(|e| e.to_string())
}

/// A response that must be an object, as every reader but the systems list needs.
fn object<R: for<'de> Deserialize<'de>>(json: &str) -> Read<R> {
    response(json)?.ok_or_else(|| "the response is not an object".to_string())
}

fn quota_of(user: &Option<Object<User>>) -> Option<Quota> {
    user.as_ref().map(|u| u.0.as_ref().map(User::quota).unwrap_or_default())
}

/// The game and the quota of a `jeuInfos` answer.
pub fn jeu_infos(json: &str) -> Read<(Option<Game>, Option<Quota>)> {
    let r: Infos = object(json)?;
    let quota = quota_of(&r.ssuser);
    match r.jeu.0 {
        Some(jeu) => jeu.game().map(|g| (Some(g), quota)).ok_or_else(|| "a game without an id".to_string()),
        None => Ok((None, quota)),
    }
}

/// `jeuRecherche`: up to 30 games ranked by likelihood; an entry with no id is left out.
pub fn jeu_recherche(json: &str) -> Read<(Vec<Game>, Option<Quota>)> {
    let r: Search = object(json)?;
    let quota = quota_of(&r.ssuser);
    Ok((r.jeux.0.into_iter().filter_map(Jeu::game).collect(), quota))
}

/// `ssuserInfos`: only the quota.
pub fn user(json: &str) -> Read<Option<Quota>> {
    Ok(quota_of(&object::<Users>(json)?.ssuser))
}

/// `ssuserInfos` as a sign-in: the member, or none when the answer names none.
pub fn member(json: &str) -> Read<Option<Member>> {
    let Some(name) = object::<Named>(json)?.ssuser.0.and_then(|u| u.id.0).filter(|n| !n.trim().is_empty()) else { return Ok(None) };
    let Some(Some(user)) = object::<Users>(json)?.ssuser.map(|u| u.0) else { return Ok(None) };
    Ok(Some(Member { name, level: user.niveau.0.clone(), quota: user.quota() }))
}

/// `systemesListe`: each system's id and its name, a later entry of an id replacing an earlier one.
pub fn systems(json: &str) -> Read<Vec<(i32, String)>> {
    let mut out: Vec<(i32, String)> = Vec::new();
    for s in response::<Systems>(json)?.map(|r| r.systemes.0).unwrap_or_default() {
        let Some(id) = s.id.int() else { continue };
        let names = s.noms.0.unwrap_or_default();
        let name = names.nom_eu.0.or(names.nom_us.0).or(names.nom_recalbox.0).unwrap_or_else(|| id.to_string());
        match out.iter_mut().find(|(i, _)| *i == id) {
            Some(entry) => entry.1 = name,
            None => out.push((id, name)),
        }
    }
    Ok(out)
}
