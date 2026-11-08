//! A game package's manifest, `package.json`, version 1: its schema checked, read, written atomically, every field this
//! build does not know kept where it was, and a newer major refused with words. What the fields mean to a launch, a
//! conversion or a patch is their callers'; this owns the file. See EmuSen_GamePackages.md §3.3 and EmuSen_Galaxia.md
//! §3.10.

use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

/// The manifest's name inside a package's folder.
pub const FILE_NAME: &str = "package.json";
/// The value of `format` that names the schema.
pub const FORMAT: &str = "emusen-package";
/// The major version this build reads and writes; any minor of it is read.
pub const MAJOR: u32 = 1;
/// The minor version this build writes in a new manifest.
pub const MINOR: u32 = 0;
/// A package's own places for a game's battery saves, its states and its cheat list (EmuSen_GamePackages.md §3.2).
pub const SAVES_FOLDER: &str = "saves";
pub const STATES_FOLDER: &str = "states";
pub const CHEATS_FILE: &str = "cheats.json";
/// A packaged game's own settings, its crop among them, `GamePackage.SettingsFile` (EmuSen_GamePackages.md §18.2).
pub const SETTINGS_FILE: &str = "settings.json";
/// A packaged game's art, under ES-DE's type folders, `GamePackage.MediaFolder` (EmuSen_GamePackages.md §18.3).
pub const MEDIA_FOLDER: &str = "media";

/// A packaged game's art of one ES-DE type folder, `GamePackage.MediaPathFor`: `<package>/media/<type folder>/<package's
/// name><extension>` (EmuSen_GamePackages.md §18.3).
pub fn media_path(root: impl AsRef<std::path::Path>, type_folder: &str, extension: &str) -> std::path::PathBuf {
    let root = root.as_ref();
    root.join(MEDIA_FOLDER).join(type_folder).join(format!("{}{extension}", crate::saves::package_stem(root)))
}

/// Why a text is not a manifest this build takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Not JSON as a file a person edits is read (`config_json`): comments and a trailing comma allowed.
    NotJson(String),
    /// JSON that the schema does not allow.
    Invalid(String),
    /// A newer major version, which may mean anything.
    Newer(String),
}

impl Refusal {
    pub fn words(&self) -> &str {
        match self {
            Refusal::NotJson(w) | Refusal::Invalid(w) | Refusal::Newer(w) => w,
        }
    }
}

/// Reads a manifest's text: parsed, checked, and returned whole, unknown fields in their places.
pub fn parse(text: &str) -> Result<Value, Refusal> {
    let read: Document = crate::config_json::parse(text).map_err(|e| Refusal::NotJson(format!("the manifest is not JSON: {e}")))?;
    if let Some(at) = read.twice {
        return Err(invalid(format!("{at} is written twice")));
    }
    check(&read.value)?;
    Ok(read.value)
}

/// A manifest as parsed: the document, and the path of its first member written twice, which a map alone would hide.
struct Document {
    value: Value,
    twice: Option<String>,
}

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Document, D::Error> {
        deserializer.deserialize_any(DocumentVisitor)
    }
}

struct DocumentVisitor;

impl<'de> Visitor<'de> for DocumentVisitor {
    type Value = Document;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a JSON value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Document, E> {
        Ok(Document { value: Value::Bool(v), twice: None })
    }

    fn visit_i64<E>(self, v: i64) -> Result<Document, E> {
        Ok(Document { value: Value::from(v), twice: None })
    }

    fn visit_u64<E>(self, v: u64) -> Result<Document, E> {
        Ok(Document { value: Value::from(v), twice: None })
    }

    fn visit_f64<E>(self, v: f64) -> Result<Document, E> {
        Ok(Document { value: Value::from(v), twice: None })
    }

    fn visit_str<E>(self, v: &str) -> Result<Document, E> {
        Ok(Document { value: Value::from(v), twice: None })
    }

    fn visit_string<E>(self, v: String) -> Result<Document, E> {
        Ok(Document { value: Value::String(v), twice: None })
    }

    fn visit_unit<E>(self) -> Result<Document, E> {
        Ok(Document { value: Value::Null, twice: None })
    }

    fn visit_none<E>(self) -> Result<Document, E> {
        Ok(Document { value: Value::Null, twice: None })
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Document, A::Error> {
        let (mut items, mut twice) = (Vec::new(), None);
        while let Some(item) = seq.next_element::<Document>()? {
            twice = twice.or(item.twice.map(|at| if at.starts_with('[') { format!("[{}]{at}", items.len()) } else { format!("[{}].{at}", items.len()) }));
            items.push(item.value);
        }
        Ok(Document { value: Value::Array(items), twice })
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Document, A::Error> {
        let (mut members, mut twice) = (Map::new(), None);
        while let Some(name) = map.next_key::<String>()? {
            let member = map.next_value::<Document>()?;
            let deeper = member.twice.map(|at| if at.starts_with('[') { format!("{name}{at}") } else { format!("{name}.{at}") });
            if members.contains_key(&name) {
                twice = twice.or(Some(name.clone()));
            }
            twice = twice.or(deeper);
            members.insert(name, member.value);
        }
        Ok(Document { value: Value::Object(members), twice })
    }
}

/// Reads the manifest at `path`, decoded as `File.ReadAllText` decodes a file.
pub fn read(path: impl AsRef<std::path::Path>) -> Result<Result<Value, Refusal>, std::io::Error> {
    let bytes = std::fs::read(path)?;
    Ok(parse(&crate::text_file::decode(&bytes)))
}

/// Writes the manifest at `path` atomically, as `AtomicFile` writes; a document the schema refuses is never written.
/// The outer error is the refusal, the inner the diagnostic of a write that failed.
pub fn write(path: impl AsRef<std::path::Path>, doc: &Value) -> Result<Result<(), String>, Refusal> {
    check(doc)?;
    Ok(crate::atomic::write(path, format(doc).as_bytes()))
}

/// The bytes a manifest is written as: serde_json's indented layout, two spaces, and a line feed at the end.
pub fn format(doc: &Value) -> String {
    let mut out = crate::config_json::write(doc).expect("a document is JSON");
    out.push('\n');
    out
}

/// A new manifest in the schema's order, linked or self-contained, with the fields a later step fills left empty.
pub fn new(title: Option<&str>, system: &str, mode: &str, game: Value) -> Value {
    serde_json::json!({
        "format": FORMAT,
        "version": format!("{MAJOR}.{MINOR}"),
        "title": title,
        "system": system,
        "mode": mode,
        "game": game,
        "dump": null,
        "companions": [],
        "patches": [],
        "patched": null,
        "identity": "original",
        "container": null,
    })
}

/// The patched image's hashes as `patched` records them, where the list holds patches and they were taken; what a
/// state saved on it records and what the scraper sends first under `identity: patched` (EmuSen_GamePackages.md §5.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patched {
    pub md5: String,
    pub crc32: String,
    pub sha1: String,
    pub size: u64,
}

pub fn patched(doc: &Value) -> Option<Patched> {
    if !matches!(doc.get("patches"), Some(Value::Array(items)) if !items.is_empty()) {
        return None;
    }
    let p = doc.get("patched")?;
    let text = |name: &str| p.get(name).and_then(Value::as_str).map(str::to_ascii_lowercase);
    let size = p.get("size")?.as_u64()?;
    Some(Patched { md5: text("md5")?, crc32: text("crc32")?, sha1: text("sha1")?, size })
}

/// Whether the scraper sends the patched image's hashes first: a list of patches, and `identity` set to `patched`.
pub fn scrapes_as_patched(doc: &Value) -> bool {
    doc.get("identity").and_then(Value::as_str) == Some("patched") && patched(doc).is_some()
}

/// `patched.patches_sha1`'s rule: the SHA-1 of one line a patch, in order, of its format, a space and its file's SHA-1,
/// each line ended by a line feed (EmuSen_GamePackages.md §15.2).
pub fn patches_key(list: &[(&str, &str)]) -> String {
    let mut lines = String::new();
    for (format, sha1) in list {
        lines.push_str(format);
        lines.push(' ');
        lines.push_str(&sha1.to_ascii_lowercase());
        lines.push('\n');
    }
    crate::rom_hash::sha1_of(lines.as_bytes())
}

/// How many hex digits of the list's key name a patched game's saves.
pub const SAVE_KEY_DIGITS: usize = 8;

/// The key a patched game's battery saves, save media and states are named apart by: the first eight hex digits of
/// `patches_key` over the list as its entries record it, an entry without a SHA-1 (written by hand) keyed by its file's
/// name; none for an empty list, whose game keeps the original's saves (EmuSen_GamePackages.md §5.5, §16).
pub fn save_key(doc: &Value) -> Option<String> {
    let entries = match doc.get("patches") {
        Some(Value::Array(items)) if !items.is_empty() => items,
        _ => return None,
    };
    let text = |e: &Value, name: &str| e.get(name).and_then(Value::as_str).unwrap_or("").to_string();
    let lines: Vec<(String, String)> = entries.iter().map(|e| (text(e, "format"), e.get("sha1").and_then(Value::as_str).map_or_else(|| text(e, "file"), str::to_string))).collect();
    let pairs: Vec<(&str, &str)> = lines.iter().map(|(f, s)| (f.as_str(), s.as_str())).collect();
    Some(patches_key(&pairs)[..SAVE_KEY_DIGITS].to_string())
}

/// `save_key` of the manifest in the package at `folder`; none where it lists no patches or does not read.
pub fn save_key_at(folder: impl AsRef<std::path::Path>) -> Option<String> {
    let doc = read(folder.as_ref().join(FILE_NAME)).ok()?.ok()?;
    save_key(&doc)
}

/// Whether `key` is one `save_key` could give: hex digits, so that it can never name a place outside the package.
pub fn is_save_key(key: &str) -> bool {
    key.len() == SAVE_KEY_DIGITS && key.bytes().all(|b| b.is_ascii_hexdigit())
}

/// A file entry, `game`'s shape and each companion's.
pub fn file_entry(role: &str, place: &str, path: &str, size: Option<u64>) -> Value {
    let mut entry = serde_json::json!({ "role": role, "where": place, "path": path });
    if let Some(size) = size {
        entry["size"] = Value::from(size);
    }
    entry
}

// ---- The schema. Each refusal names the field by its path; the C# half words them identically. ----

fn invalid(words: String) -> Refusal {
    Refusal::Invalid(words)
}

fn missing(path: &str) -> Refusal {
    invalid(format!("{path} is missing"))
}

fn wrong(path: &str, want: &str) -> Refusal {
    invalid(format!("{path} must be {want}"))
}

fn child(path: &str, name: &str) -> String {
    if path.is_empty() { name.to_string() } else { format!("{path}.{name}") }
}

/// Checks a document against the schema: `format`, then `version` (so that a newer major is named
/// before anything it may have changed), then every known field in the schema's order.
pub fn check(doc: &Value) -> Result<(), Refusal> {
    if !matches!(doc, Value::Object(_)) {
        return Err(invalid("the manifest is not a JSON object".to_string()));
    }
    match doc.get("format") {
        None => return Err(missing("format")),
        Some(v) if v.as_str() == Some(FORMAT) => {}
        Some(_) => return Err(wrong("format", "\"emusen-package\"")),
    }
    let version = match doc.get("version") {
        None => return Err(missing("version")),
        Some(v) => v.as_str().filter(|s| two_numbers(s)).ok_or_else(|| wrong("version", "two numbers, major.minor"))?,
    };
    let major = version.split('.').next().unwrap_or("").trim_start_matches('0');
    if major.is_empty() {
        return Err(wrong("version", "1.x"));
    }
    if major != "1" {
        return Err(Refusal::Newer(format!("the package format is {version}, written by a newer EmuSen; this build reads 1.x")));
    }
    optional(doc, "title", "", |v, p| if matches!(v, Value::String(_)) { Ok(()) } else { Err(wrong(p, "a string or null")) })?;
    match doc.get("system") {
        None => return Err(missing("system")),
        Some(v) => non_empty(v, "system")?,
    }
    match doc.get("mode") {
        None => return Err(missing("mode")),
        Some(v) => one_of(v, "mode", &["linked", "self-contained"], "linked or self-contained")?,
    }
    match doc.get("game") {
        None => return Err(missing("game")),
        Some(v) => entry(v, "game")?,
    }
    optional(doc, "dump", "", dump)?;
    optional(doc, "companions", "", |v, p| array(v, p, entry))?;
    optional(doc, "patches", "", |v, p| array(v, p, patch))?;
    optional(doc, "patched", "", |v, p| {
        hashes(v, p, "an object or null")?;
        optional(v, "patches_sha1", p, |h, q| hex(h, q, 40))
    })?;
    if let Some(v) = doc.get("identity") {
        one_of(v, "identity", &["original", "patched"], "original or patched")?;
    }
    optional(doc, "container", "", |v, p| if matches!(v, Value::Object(_)) { Ok(()) } else { Err(wrong(p, "an object or null")) })
}

fn two_numbers(s: &str) -> bool {
    match s.split_once('.') {
        Some((a, b)) => !a.is_empty() && !b.is_empty() && a.bytes().all(|c| c.is_ascii_digit()) && b.bytes().all(|c| c.is_ascii_digit()),
        None => false,
    }
}

/// A field that may be absent or null, checked by `rule` otherwise.
fn optional(object: &Value, name: &str, parent: &str, rule: impl Fn(&Value, &str) -> Result<(), Refusal>) -> Result<(), Refusal> {
    match object.get(name) {
        None | Some(Value::Null) => Ok(()),
        Some(v) => rule(v, &child(parent, name)),
    }
}

fn required(object: &Value, name: &str, parent: &str, rule: impl Fn(&Value, &str) -> Result<(), Refusal>) -> Result<(), Refusal> {
    let path = child(parent, name);
    match object.get(name) {
        None => Err(missing(&path)),
        Some(v) => rule(v, &path),
    }
}

fn non_empty(v: &Value, path: &str) -> Result<(), Refusal> {
    match v.as_str() {
        Some(s) if !s.is_empty() => Ok(()),
        _ => Err(wrong(path, "a non-empty string")),
    }
}

fn one_of(v: &Value, path: &str, allowed: &[&str], want: &str) -> Result<(), Refusal> {
    if v.as_str().is_some_and(|s| allowed.contains(&s)) { Ok(()) } else { Err(wrong(path, want)) }
}

fn whole(v: &Value, path: &str, want: &str) -> Result<(), Refusal> {
    match v {
        Value::Number(number) if number.as_i64().is_some_and(|n| n >= 0) => Ok(()),
        _ => Err(wrong(path, want)),
    }
}

fn hex(v: &Value, path: &str, digits: usize) -> Result<(), Refusal> {
    match v.as_str() {
        Some(s) if s.len() == digits && s.bytes().all(|c| c.is_ascii_hexdigit()) => Ok(()),
        _ => Err(wrong(path, &format!("{digits} hexadecimal digits or null"))),
    }
}

fn array(v: &Value, path: &str, item: fn(&Value, &str) -> Result<(), Refusal>) -> Result<(), Refusal> {
    match v {
        Value::Array(items) => items.iter().enumerate().try_for_each(|(i, x)| item(x, &format!("{path}[{i}]"))),
        _ => Err(wrong(path, "an array or null")),
    }
}

/// The three hashes a dump's form carries, each optional; `want` is the object's own words.
fn hashes(v: &Value, path: &str, want: &str) -> Result<(), Refusal> {
    if !matches!(v, Value::Object(_)) {
        return Err(wrong(path, want));
    }
    optional(v, "crc32", path, |h, p| hex(h, p, 8))?;
    optional(v, "md5", path, |h, p| hex(h, p, 32))?;
    optional(v, "sha1", path, |h, p| hex(h, p, 40))
}

fn absolute(path: &str) -> bool {
    let b = path.as_bytes();
    path.starts_with('/') || path.starts_with('\\') || (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'/' || b[2] == b'\\'))
}

fn inside(path: &str) -> bool {
    let b = path.as_bytes();
    let drive = b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':';
    !path.starts_with('/') && !path.starts_with('\\') && !drive && !path.split(['/', '\\']).any(|part| part == "..")
}

/// A file of the set: `game`, and each companion.
fn entry(v: &Value, path: &str) -> Result<(), Refusal> {
    if !matches!(v, Value::Object(_)) {
        return Err(wrong(path, "an object"));
    }
    required(v, "role", path, non_empty)?;
    required(v, "where", path, |w, p| one_of(w, p, &["library", "package", "compressed", "absolute"], "library, package, compressed or absolute"))?;
    required(v, "path", path, non_empty)?;
    let place = v.get("where").and_then(Value::as_str).unwrap_or("");
    let file = v.get("path").and_then(Value::as_str).unwrap_or("");
    let at = child(path, "path");
    if place == "absolute" && !absolute(file) {
        return Err(wrong(&at, "an absolute path"));
    }
    if place != "absolute" && !inside(file) {
        return Err(wrong(&at, "a path inside its root, with no .. and not absolute"));
    }
    optional(v, "size", path, |s, p| whole(s, p, "a whole number of 0 or more, or null"))?;
    optional(v, "seen", path, |s, p| {
        if !matches!(s, Value::Object(_)) {
            return Err(wrong(p, "an object or null"));
        }
        required(s, "size", p, |n, q| whole(n, q, "a whole number of 0 or more"))?;
        required(s, "mtime", p, |t, q| if matches!(t, Value::String(_)) { Ok(()) } else { Err(wrong(q, "a string")) })
    })
}

fn dump(v: &Value, path: &str) -> Result<(), Refusal> {
    if !matches!(v, Value::Object(_)) {
        return Err(wrong(path, "an object or null"));
    }
    optional(v, "file", path, |h, p| hashes(h, p, "an object or null"))?;
    optional(v, "transformed", path, |h, p| hashes(h, p, "an object or null"))?;
    optional(v, "tracks", path, |t, p| array(t, p, track))?;
    optional(v, "sheet", path, |s, p| {
        hashes(s, p, "an object or null")?;
        required(s, "file", p, non_empty)
    })?;
    optional(v, "form", path, |f, p| one_of(f, p, &["cue-bin", "lone-track", "iso"], "cue-bin, lone-track, iso or null"))
}

fn track(v: &Value, path: &str) -> Result<(), Refusal> {
    hashes(v, path, "an object")?;
    required(v, "file", path, non_empty)?;
    optional(v, "size", path, |s, p| whole(s, p, "a whole number of 0 or more, or null"))
}

fn patch(v: &Value, path: &str) -> Result<(), Refusal> {
    if !matches!(v, Value::Object(_)) {
        return Err(wrong(path, "an object"));
    }
    required(v, "file", path, non_empty)?;
    required(v, "format", path, |f, p| one_of(f, p, &["ips", "bps", "ups"], "ips, bps or ups"))?;
    optional(v, "sha1", path, |h, p| hex(h, p, 40))?;
    optional(v, "source_sha1", path, |h, p| hex(h, p, 40))?;
    optional(v, "source_crc32", path, |h, p| hex(h, p, 8))?;
    optional(v, "target_crc32", path, |h, p| hex(h, p, 8))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    const SPEC: &str = r#"{
  "format": "emusen-package",
  "version": "1.0",
  "title": "Super Metroid (USA)",
  "system": "snes",
  "mode": "linked",
  "game": {
    "role": "image",
    "where": "library",
    "path": "SNES/Super Metroid (USA).sfc",
    "size": 3145728,
    "seen": {
      "size": 3145728,
      "mtime": "2023-08-03T10:12:00Z"
    }
  },
  "dump": {
    "file": {
      "crc32": "00112233",
      "md5": "00112233445566778899aabbccddeeff",
      "sha1": "00112233445566778899aabbccddeeff00112233"
    },
    "transformed": null
  },
  "companions": [],
  "patches": [
    {
      "file": "Super Metroid (USA).bps",
      "format": "bps",
      "sha1": "0123456789abcdef0123456789abcdef01234567",
      "source_crc32": "00112233",
      "target_crc32": "01234567"
    }
  ],
  "patched": null,
  "identity": "original",
  "container": null
}
"#;

    fn refusal(text: &str) -> Refusal {
        parse(text).unwrap_err()
    }

    #[test]
    fn the_pages_example_round_trips_to_its_own_bytes() {
        let doc = parse(SPEC).unwrap();
        assert_eq!(format(&doc), SPEC);
    }

    #[test]
    fn unknown_fields_are_kept_where_they_were() {
        let text = r#"{"x-first": 1.50, "format": "emusen-package", "version": "1.7", "system": "nes", "mode": "linked",
            "game": {"role": "image", "where": "package", "path": "g.nes", "later": {"a": [1e2, -0, "é<"]}}, "zz": null, }"#;
        let doc = parse(text).unwrap();
        let out = format(&doc);
        assert!(out.starts_with("{\n  \"x-first\": 1.5,\n"), "{out}");
        assert!(out.contains("\"later\": {"), "{out}");
        assert!(out.contains("100.0") && out.contains("\"é<\""), "{out}");
        assert!(out.ends_with("\"zz\": null\n}\n"), "{out}");
        assert_eq!(parse(&out).unwrap(), doc);
    }

    #[test]
    fn a_newer_major_is_refused_before_anything_it_may_have_changed() {
        assert_eq!(refusal(r#"{"format": "emusen-package", "version": "2.0"}"#), Refusal::Newer("the package format is 2.0, written by a newer EmuSen; this build reads 1.x".into()));
        assert!(matches!(refusal(r#"{"format": "emusen-package", "version": "0010.3", "game": 5}"#), Refusal::Newer(_)));
        assert!(matches!(refusal(r#"{"format": "emusen-package", "version": "99999999999999999999.0"}"#), Refusal::Newer(_)));
        assert_eq!(refusal(r#"{"format": "emusen-package", "version": "0.9"}"#), Refusal::Invalid("version must be 1.x".into()));
        assert!(parse(r#"{"format": "emusen-package", "version": "01.12", "system": "nes", "mode": "linked", "game": {"role": "image", "where": "package", "path": "g.nes"}}"#).is_ok());
    }

    #[test]
    fn each_rule_of_the_schema_refuses_with_the_fields_path() {
        const GAME: &str = r#"{"role": "image", "where": "library", "path": "NES/g.nes"}"#;
        let with = |mode: &str, game: &str, extra: &str| format!(r#"{{"format": "emusen-package", "version": "1.0", "system": "nes", "mode": {mode}, "game": {game}{extra}}}"#);
        let base = |extra: &str| with("\"linked\"", GAME, extra);
        let cases: &[(&str, &str)] = &[
            (r#"[]"#, "the manifest is not a JSON object"),
            (r#"{"version": "1.0"}"#, "format is missing"),
            (r#"{"format": "zip"}"#, "format must be \"emusen-package\""),
            (r#"{"format": "emusen-package"}"#, "version is missing"),
            (r#"{"format": "emusen-package", "version": "1"}"#, "version must be two numbers, major.minor"),
            (r#"{"format": "emusen-package", "version": "1.0", "format": "emusen-package"}"#, "format is written twice"),
            (r#"{"format": "emusen-package", "version": "1.0", "x": {"a": 1, "a": 2}}"#, "x.a is written twice"),
            (r#"{"format": "emusen-package", "version": "1.0", "x": [{"b": [{"c": 1, "c": 2}]}]}"#, "x[0].b[0].c is written twice"),
            (r#"{"format": "emusen-package", "version": "1.0", "mode": "linked", "game": {}}"#, "system is missing"),
        ];
        for (text, words) in cases {
            assert_eq!(refusal(text).words(), *words, "{text}");
        }
        let fields: Vec<(String, &str)> = vec![
            (base(r#", "title": 3"#), "title must be a string or null"),
            (with("\"copied\"", GAME, ""), "mode must be linked or self-contained"),
            (with("\"linked\"", r#"{"role": "", "where": "library", "path": "a"}"#, ""), "game.role must be a non-empty string"),
            (with("\"linked\"", r#"{"role": "image", "path": "a"}"#, ""), "game.where is missing"),
            (with("\"linked\"", r#"{"role": "image", "where": "cloud", "path": "a"}"#, ""), "game.where must be library, package, compressed or absolute"),
            (with("\"linked\"", r#"{"role": "image", "where": "library", "path": "../a"}"#, ""), "game.path must be a path inside its root, with no .. and not absolute"),
            (with("\"linked\"", r#"{"role": "image", "where": "package", "path": "C:a"}"#, ""), "game.path must be a path inside its root, with no .. and not absolute"),
            (with("\"linked\"", r#"{"role": "image", "where": "absolute", "path": "a/b"}"#, ""), "game.path must be an absolute path"),
            (with("\"linked\"", r#"{"role": "image", "where": "library", "path": "a", "size": -1}"#, ""), "game.size must be a whole number of 0 or more, or null"),
            (with("\"linked\"", r#"{"role": "image", "where": "library", "path": "a", "size": 1.0}"#, ""), "game.size must be a whole number of 0 or more, or null"),
            (with("\"linked\"", r#"{"role": "image", "where": "library", "path": "a", "seen": {"size": 1}}"#, ""), "game.seen.mtime is missing"),
            (base(r#", "dump": []"#), "dump must be an object or null"),
            (base(r#", "dump": {"file": {"md5": "abc"}}"#), "dump.file.md5 must be 32 hexadecimal digits or null"),
            (base(r#", "dump": {"tracks": [{"crc32": "00000000"}]}"#), "dump.tracks[0].file is missing"),
            (base(r#", "dump": {"sheet": {}}"#), "dump.sheet.file is missing"),
            (base(r#", "dump": {"form": "chd"}"#), "dump.form must be cue-bin, lone-track, iso or null"),
            (base(r#", "companions": [{"role": "extra", "where": "package", "path": "/x"}]"#), "companions[0].path must be a path inside its root, with no .. and not absolute"),
            (base(r#", "patches": {}"#), "patches must be an array or null"),
            (base(r#", "patches": [{"file": "a.ips", "format": "xdelta"}]"#), "patches[0].format must be ips, bps or ups"),
            (base(r#", "patched": {"patches_sha1": "12"}"#), "patched.patches_sha1 must be 40 hexadecimal digits or null"),
            (base(r#", "identity": null"#), "identity must be original or patched"),
            (base(r#", "container": "chd""#), "container must be an object or null"),
        ];
        for (text, words) in &fields {
            assert_eq!(parse(text).map(|_| ()).unwrap_err().words(), *words, "{text}");
        }
        assert!(matches!(refusal("{\"format\": "), Refusal::NotJson(_)));
        assert!(matches!(refusal(r#"{"format": "emusen-package", "version": "1.0", "x": ["\ud800"]}"#), Refusal::NotJson(_)));
        assert!(parse(&base(r#", "game2": {"where": "anything"}, "dump": {"file": {"md5": "00112233445566778899AABBCCDDEEFF"}}"#)).is_ok());
        assert!(parse(&base(r#", "companions": [{"role": "extra", "where": "absolute", "path": "D:\\msu\\a.pcm"}]"#)).is_ok());
    }

    #[test]
    fn a_patch_lists_save_key_follows_the_list_and_none_names_the_original() {
        let doc = parse(SPEC).unwrap();
        let key = save_key(&doc).unwrap();
        assert_eq!(key, patches_key(&[("bps", "0123456789ABCDEF0123456789abcdef01234567")])[..SAVE_KEY_DIGITS]);
        assert_eq!(patches_key(&[("ips", "ab")]), crate::rom_hash::sha1_of(b"ips ab\n"));
        assert!(is_save_key(&key) && !is_save_key("../x/abc") && !is_save_key("0123456"));
        let other = parse(&SPEC.replace("0123456789abcdef0123456789abcdef01234567", "1123456789abcdef0123456789abcdef01234567")).unwrap();
        assert_ne!(save_key(&other), Some(key.clone()), "another patch file is another save");
        let unsigned = parse(&SPEC.replace("\"sha1\": \"0123456789abcdef0123456789abcdef01234567\",\n", "")).unwrap();
        assert_eq!(save_key(&unsigned).unwrap(), patches_key(&[("bps", "Super Metroid (USA).bps")])[..SAVE_KEY_DIGITS]);
        assert_eq!(save_key(&parse(&SPEC.replace("\"patches\": [", "\"patches\": [], \"was\": [")).unwrap()), None);
        let temp = TempDir::new("package-key");
        std::fs::create_dir_all(temp.join("p.sfc")).unwrap();
        std::fs::write(temp.join(&format!("p.sfc/{FILE_NAME}")), SPEC).unwrap();
        assert_eq!(save_key_at(&temp.join("p.sfc")), Some(key));
        assert_eq!(save_key_at(&temp.join("absent")), None);
    }

    #[test]
    fn a_manifest_is_written_whole_or_not_at_all_and_read_back() {
        let temp = TempDir::new("package");
        let path = temp.join(&format!("Packages/nes/g.nes/{FILE_NAME}"));
        let doc = new(Some("G"), "nes", "linked", file_entry("image", "library", "NES/g.nes", Some(40976)));
        assert_eq!(write(&path, &doc), Ok(Ok(())));
        assert_eq!(read(&path).unwrap(), Ok(doc.clone()));
        assert!(std::fs::read_to_string(&path).unwrap().contains("\"identity\": \"original\""));
        let bad = new(None, "", "linked", file_entry("image", "library", "NES/g.nes", None));
        assert_eq!(write(&path, &bad).unwrap_err().words(), "system must be a non-empty string");
        assert_eq!(read(&path).unwrap(), Ok(doc), "the refused document was not written");
        assert_eq!(read(&temp.join("absent.json")).unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }
}
