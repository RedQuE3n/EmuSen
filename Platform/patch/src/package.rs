//! A package's patch list as its manifest records it (EmuSen_GamePackages.md §3.3, §5.4): each file read from the
//! package's folder and held to the SHA-1 recorded when it was added, applied in order, and the result held to the
//! `patched` hashes where they were taken for this same list. The one composition the two runtimes and the library
//! tool share; the files are Galaxia's to place, the formats this crate's.

use crate::{apply_list, Format, Step, MAX_IMAGE};
use emusen_galaxia::claims;
use emusen_galaxia::json::Value;
use emusen_galaxia::rom_hash::{self, Hasher};

/// The largest patch file read: a BPS of a whole new 64 MiB image, with room for its numbers.
pub const MAX_PATCH: u64 = MAX_IMAGE + MAX_IMAGE / 8;

/// One patch of a package's list, read.
#[derive(Clone, Debug)]
pub struct Listed {
    pub file: String,
    pub format: Format,
    pub bytes: Vec<u8>,
    pub sha1: String,
    pub source_sha1: Option<String>,
}

fn text<'a>(entry: &'a Value, name: &str) -> Option<&'a str> {
    entry.get(name).and_then(Value::as_str)
}

/// The list's entries, in order, as the manifest has them; empty where it has none.
pub fn entries(doc: &Value) -> &[Value] {
    match doc.get("patches") {
        Some(Value::Array(items)) => items,
        _ => &[],
    }
}

/// Whether a list entry's file names a place inside the package: relative, with no `..`.
pub fn inside(file: &str) -> bool {
    let b = file.as_bytes();
    !file.is_empty() && !file.starts_with(['/', '\\']) && !(b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':') && !file.split(['/', '\\']).any(|p| p == "..")
}

/// Each patch of the list read from the package's folder and checked against its recorded SHA-1, with words naming the
/// file that is missing, unreadable or not the one added.
pub fn read_list(folder: &str, doc: &Value) -> Result<Vec<Listed>, String> {
    let mut list = Vec::new();
    for (n, entry) in entries(doc).iter().enumerate() {
        let file = text(entry, "file").unwrap_or("");
        let format = text(entry, "format").and_then(Format::named).ok_or_else(|| format!("patch {n} of the package names no format this build applies"))?;
        if !inside(file) {
            return Err(format!("patch {n} names {file:?}, which is not a file inside the package"));
        }
        let at = claims::under(folder, file);
        let size = std::fs::metadata(&at).map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { format!("{file}: the patch is missing from the package") } else { format!("{file}: {e}") })?.len();
        if size > MAX_PATCH {
            return Err(format!("{file}: the patch is {size} bytes, more than the {MAX_PATCH} a patch may be"));
        }
        let bytes = std::fs::read(&at).map_err(|e| format!("{file}: {e}"))?;
        let sha1 = rom_hash::sha1_of(&bytes);
        if let Some(want) = text(entry, "sha1")
            && !want.eq_ignore_ascii_case(&sha1)
        {
            return Err(format!("{file} is not the patch that was added: its SHA-1 is {sha1}, and the package recorded {}", want.to_ascii_lowercase()));
        }
        list.push(Listed { file: file.to_string(), format, bytes, sha1, source_sha1: text(entry, "source_sha1").map(str::to_string) });
    }
    Ok(list)
}

/// `patched.patches_sha1`: the SHA-1 of one line a patch, in order, of its format, a space and its file's SHA-1, each
/// line ended by a line feed (EmuSen_GamePackages.md §15.2). It changes when the list or any of its files does.
pub fn key(list: &[(Format, &str)]) -> String {
    let mut lines = String::new();
    for (format, sha1) in list {
        lines.push_str(format.name());
        lines.push(' ');
        lines.push_str(&sha1.to_ascii_lowercase());
        lines.push('\n');
    }
    rom_hash::sha1_of(lines.as_bytes())
}

fn key_of(list: &[Listed]) -> String {
    key(&list.iter().map(|l| (l.format, l.sha1.as_str())).collect::<Vec<_>>())
}

/// The patched image's hashes as the manifest records them, `patched`, with its size.
pub fn patched_json(list: &[Listed], image: &[u8]) -> Value {
    let mut h = Hasher::new(true);
    h.update(image);
    let h = h.finish();
    Value::Object(vec![
        (emusen_galaxia::json::Text::new("patches_sha1"), Value::string(&key_of(list))),
        (emusen_galaxia::json::Text::new("size"), Value::number(h.size)),
        (emusen_galaxia::json::Text::new("crc32"), Value::string(&h.crc32)),
        (emusen_galaxia::json::Text::new("md5"), Value::string(&h.md5)),
        (emusen_galaxia::json::Text::new("sha1"), Value::string(h.sha1.as_deref().unwrap_or(""))),
    ])
}

/// The list applied to `source`, every file read and checked first; none where the list is empty. Where `patched`
/// was taken for this same list, the result's SHA-1 must be its, which gives an IPS list a target check.
pub fn apply_listed(doc: &Value, list: &[Listed], source: &[u8]) -> Result<Vec<u8>, String> {
    let steps: Vec<Step> = list.iter().map(|l| Step { name: &l.file, format: l.format, bytes: &l.bytes, source_sha1: if l.format == Format::Ips { l.source_sha1.as_deref() } else { None } }).collect();
    let image = apply_list(source, &steps)?;
    let recorded = doc.get("patched").filter(|p| text(p, "patches_sha1").is_some_and(|k| k.eq_ignore_ascii_case(&key_of(list))));
    if let Some(want) = recorded.and_then(|p| text(p, "sha1")) {
        let found = rom_hash::sha1_of(&image);
        if !found.eq_ignore_ascii_case(want) {
            return Err(format!("the patched image has SHA-1 {found}, and the package recorded {} when its patches were added", want.to_ascii_lowercase()));
        }
    }
    Ok(image)
}

/// The image a packaged game's core is given: `source` with the manifest's patches applied, or none where it lists
/// none (EmuSen_GamePackages.md §5.4).
pub fn image(folder: &str, doc: &Value, source: &[u8]) -> Result<Option<Vec<u8>>, String> {
    if entries(doc).is_empty() {
        return Ok(None);
    }
    let list = read_list(folder, doc)?;
    apply_listed(doc, &list, source).map(Some)
}

/// The image a package at `folder` gives its core: its manifest read, and `image`'s answer.
pub fn image_at(folder: &str, source: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let manifest = emusen_galaxia::dotnet_files::join(folder, emusen_galaxia::package::FILE_NAME);
    let doc = match emusen_galaxia::package::read(&manifest) {
        Ok(Ok(doc)) => doc,
        Ok(Err(refusal)) => return Err(refusal.words().to_string()),
        Err(e) => return Err(format!("{manifest}: {e}")),
    };
    image(folder, &doc, source)
}

/// What a state saved on a patched package's game records as its ROM: the patched image's MD5 and size, where the
/// list is not empty and `patched` holds them (EmuSen_GamePackages.md §5.5).
pub fn patched_identity(doc: &Value) -> Option<(String, u64)> {
    emusen_galaxia::package::patched(doc).map(|p| (p.md5, p.size))
}
