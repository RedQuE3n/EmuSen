//! A game package's manifest, `package.json`, version 1: its schema checked, read, written atomically, every field this
//! build does not know kept where it was, and a newer major refused with words. What the fields mean to a launch, a
//! conversion or a patch is their callers'; this owns the file. See EmuSen_GamePackages.md §3.3 and EmuSen_Galaxia.md
//! §3.10.

use crate::json::{self, Value};
use crate::model::NEW_LINE;

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

/// Why a text is not a manifest this build takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Not JSON as .NET reads a file a person edits: comments and a trailing comma allowed, 64 levels deep.
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
    let doc = json::parse(text).map_err(|e| Refusal::NotJson(format!("the manifest is not JSON: {e}")))?;
    check(&doc)?;
    Ok(doc)
}

/// Reads the manifest at `path`, decoded as `File.ReadAllText` decodes a file.
pub fn read(path: &str) -> Result<Result<Value, Refusal>, std::io::Error> {
    let bytes = std::fs::read(path)?;
    Ok(parse(&emusen_dotnet::file::decode(&bytes)))
}

/// Writes the manifest at `path` atomically, as `AtomicFile` writes; a document the schema refuses is never written.
/// The outer error is the refusal, the inner the diagnostic of a write that failed.
pub fn write(path: &str, doc: &Value) -> Result<Result<(), String>, Refusal> {
    check(doc)?;
    Ok(crate::atomic::write(path, format(doc).as_bytes()))
}

/// The bytes a manifest is written as: System.Text.Json's indented layout, two spaces, its encoder's escapes, each
/// number as its token, the platform's line ending, and one at the end.
pub fn format(doc: &Value) -> String {
    let mut out = String::new();
    indented(&mut out, doc, 0);
    out.push_str(NEW_LINE);
    out
}

fn indented(out: &mut String, value: &Value, depth: usize) {
    let line = |out: &mut String, depth: usize| {
        out.push_str(NEW_LINE);
        out.push_str(&"  ".repeat(depth));
    };
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(token) => out.push_str(token),
        Value::String(text) => out.push_str(&json::quoted(&text.text)),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Object(members) if members.is_empty() => out.push_str("{}"),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                line(out, depth + 1);
                indented(out, item, depth + 1);
            }
            line(out, depth);
            out.push(']');
        }
        Value::Object(members) => {
            out.push('{');
            for (i, (name, member)) in members.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                line(out, depth + 1);
                out.push_str(&json::quoted(&name.text));
                out.push_str(": ");
                indented(out, member, depth + 1);
            }
            line(out, depth);
            out.push('}');
        }
    }
}

/// A new manifest in the schema's order, linked or self-contained, with the fields a later step fills left empty.
pub fn new(title: Option<&str>, system: &str, mode: &str, game: Value) -> Value {
    let text = |s: &str| Value::string(s);
    Value::Object(vec![
        (json::Text::new("format"), text(FORMAT)),
        (json::Text::new("version"), text(&format!("{MAJOR}.{MINOR}"))),
        (json::Text::new("title"), title.map_or(Value::Null, text)),
        (json::Text::new("system"), text(system)),
        (json::Text::new("mode"), text(mode)),
        (json::Text::new("game"), game),
        (json::Text::new("dump"), Value::Null),
        (json::Text::new("companions"), Value::Array(Vec::new())),
        (json::Text::new("patches"), Value::Array(Vec::new())),
        (json::Text::new("patched"), Value::Null),
        (json::Text::new("identity"), text("original")),
        (json::Text::new("container"), Value::Null),
    ])
}

/// A file entry, `game`'s shape and each companion's.
pub fn file_entry(role: &str, place: &str, path: &str, size: Option<u64>) -> Value {
    let mut members = vec![
        (json::Text::new("role"), Value::string(role)),
        (json::Text::new("where"), Value::string(place)),
        (json::Text::new("path"), Value::string(path)),
    ];
    if let Some(size) = size {
        members.push((json::Text::new("size"), Value::number(size)));
    }
    Value::Object(members)
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

/// No member written twice, and no text that is not valid Unicode, anywhere in the document, in document order.
fn well_formed(value: &Value, path: &str) -> Result<(), Refusal> {
    match value {
        Value::String(text) if text.broken => Err(invalid(format!("{path} is not valid Unicode"))),
        Value::Array(items) => items.iter().enumerate().try_for_each(|(i, item)| well_formed(item, &format!("{path}[{i}]"))),
        Value::Object(members) => {
            let mut seen: Vec<&str> = Vec::with_capacity(members.len());
            for (name, member) in members {
                if name.broken {
                    let at = if path.is_empty() { "the manifest" } else { path };
                    return Err(invalid(format!("{at} has a member name that is not valid Unicode")));
                }
                let at = child(path, &name.text);
                if seen.contains(&name.text.as_str()) {
                    return Err(invalid(format!("{at} is written twice")));
                }
                seen.push(&name.text);
                well_formed(member, &at)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Checks a document against the schema: well formed, then `format`, then `version` (so that a newer major is named
/// before anything it may have changed), then every known field in the schema's order.
pub fn check(doc: &Value) -> Result<(), Refusal> {
    if !matches!(doc, Value::Object(_)) {
        return Err(invalid("the manifest is not a JSON object".to_string()));
    }
    well_formed(doc, "")?;
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
        Value::Number(token) if token.parse::<i64>().is_ok_and(|n| n >= 0) => Ok(()),
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
        assert_eq!(format(&doc), SPEC.replace('\n', NEW_LINE));
    }

    #[test]
    fn unknown_fields_are_kept_where_they_were_with_their_tokens() {
        let text = r#"{"x-first": 1.50, "format": "emusen-package", "version": "1.7", "system": "nes", "mode": "linked",
            "game": {"role": "image", "where": "package", "path": "g.nes", "later": {"a": [1e2, -0, "é<"]}}, "zz": null, }"#;
        let doc = parse(text).unwrap();
        let out = format(&doc);
        assert!(out.starts_with("{\n  \"x-first\": 1.50,\n".replace('\n', NEW_LINE).as_str()), "{out}");
        assert!(out.contains("\"later\": {"), "{out}");
        assert!(out.contains("1e2") && out.contains("-0") && out.contains("\"\\u00E9\\u003C\""), "{out}");
        assert!(out.ends_with(&format!("\"zz\": null{NEW_LINE}}}{NEW_LINE}")), "{out}");
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
            (r#"{"format": "emusen-package", "version": "1.0", "x": ["\ud800"]}"#, "x[0] is not valid Unicode"),
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
        assert!(parse(&base(r#", "game2": {"where": "anything"}, "dump": {"file": {"md5": "00112233445566778899AABBCCDDEEFF"}}"#)).is_ok());
        assert!(parse(&base(r#", "companions": [{"role": "extra", "where": "absolute", "path": "D:\\msu\\a.pcm"}]"#)).is_ok());
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
