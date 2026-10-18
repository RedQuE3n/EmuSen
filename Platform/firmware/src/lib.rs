//! The firmware store, the C# `FirmwareLibrary` and the facts of `FirmwareOverview`'s rows, knowing nothing about any
//! console (EmuSen_Firmware.md §2). The folders are the player's chosen one when its setting is usable, then
//! home/Firmware, each searched once and only read; within a folder the canonical name, then the alternates, then each
//! split form whole from that folder; size is the only test. An install writes into home/Firmware alone.

use emusen_dotnet::json::{self, Value};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::setting_folders::{self, FolderValue};
use emusen_galaxia::tree::file_exists;
use std::sync::Mutex;

/// One image a core can use: its canonical name, its size, the other names it circulates under, and its split forms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub file: String,
    pub size: i32,
    pub alternates: Vec<String>,
    pub parts: Vec<Vec<String>>,
}

/// An image found: the folder, the names read, and the bytes whole or as the parts of a split form in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub folder: String,
    pub names: Vec<String>,
    pub parts: Vec<Vec<u8>>,
}

/// Where an image is, without its bytes: what the overview and the question before a launch need.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Located {
    pub folder: String,
    pub names: Vec<String>,
    pub whole: bool,
}

/// The name a refused or absent value is reported under, `AppSettings.FirmwareDirectory`.
pub const SETTING: &str = "FirmwareDirectory";

/// The folders searched, first to last, and the setting's value after the rules (EmuSen_Firmware.md §2.3).
pub fn folders(setting: Option<&str>, home: &str, store: &str) -> (Vec<String>, FolderValue) {
    let chosen = setting_folders::checked(setting, home);
    let list = match chosen.path.as_deref() {
        Some(p) if chosen.searched() && !setting_folders::same_folder(p, store) => vec![p.to_string(), store.to_string()],
        _ => vec![store.to_string()],
    };
    (list, chosen)
}

static REPORTED: Mutex<Option<String>> = Mutex::new(None);

/// The line to report for a refused or absent value, said once and again only when the value changes: `ReportOnce`.
pub fn report_once(value: &FolderValue) -> Option<String> {
    let problem = value.problem();
    let key = problem.as_ref().map(|_| format!("{SETTING}\n{}", value.written));
    let mut reported = REPORTED.lock().unwrap_or_else(|p| p.into_inner());
    if key == *reported {
        return None;
    }
    *reported = key;
    problem.map(|p| format!("appsettings.json: {SETTING}: {p}"))
}

/// A file's length as reading it finds it, where `File.Exists`: a link's final target's, none for a link to nothing.
fn length(file: &str) -> Option<u64> {
    if !file_exists(file) {
        return None;
    }
    std::fs::metadata(file).ok().map(|m| m.len())
}

/// What a search takes of a file: its bytes, or only that it opens and its length.
trait Take {
    type Part;
    fn exact(file: &str, size: i32) -> Option<Self::Part>;
    fn any(file: &str) -> Option<Self::Part>;
    fn len(part: &Self::Part) -> u64;
}

struct Bytes;

impl Take for Bytes {
    type Part = Vec<u8>;
    fn exact(file: &str, size: i32) -> Option<Vec<u8>> {
        if length(file)? != size as u64 || size < 0 {
            return None;
        }
        std::fs::read(file).ok()
    }
    fn any(file: &str) -> Option<Vec<u8>> {
        if !file_exists(file) {
            return None;
        }
        std::fs::read(file).ok()
    }
    fn len(part: &Vec<u8>) -> u64 {
        part.len() as u64
    }
}

struct Opens;

impl Take for Opens {
    type Part = u64;
    fn exact(file: &str, size: i32) -> Option<u64> {
        let n = length(file)?;
        if n != size as u64 || size < 0 {
            return None;
        }
        std::fs::File::open(file).ok().map(|_| n)
    }
    fn any(file: &str) -> Option<u64> {
        if !file_exists(file) {
            return None;
        }
        std::fs::File::open(file).ok()?.metadata().ok().map(|m| m.len())
    }
    fn len(part: &u64) -> u64 {
        *part
    }
}

fn search<T: Take>(request: &Request, folders: &[String]) -> Option<(String, Vec<String>, Vec<T::Part>)> {
    for folder in folders {
        for name in std::iter::once(&request.file).chain(&request.alternates) {
            if let Some(part) = T::exact(&path::combine(Style::HOST, folder, name), request.size) {
                return Some((folder.clone(), vec![name.clone()], vec![part]));
            }
        }
        for form in request.parts.iter().filter(|f| f.len() >= 2) {
            let parts: Option<Vec<T::Part>> = form.iter().map(|name| T::any(&path::combine(Style::HOST, folder, name))).collect();
            let Some(parts) = parts else { continue };
            if request.size > 0 && parts.iter().map(T::len).sum::<u64>() != request.size as u64 {
                continue;
            }
            return Some((folder.clone(), form.clone(), parts));
        }
    }
    None
}

/// `FirmwareLibrary.Find`: the first folder holding the image whole by any of its names or as a complete split form.
pub fn find(request: &Request, folders: &[String]) -> Option<Found> {
    search::<Bytes>(request, folders).map(|(folder, names, parts)| Found { folder, names, parts })
}

/// Where `find` would take the image, each file opened and measured but not read.
pub fn locate(request: &Request, folders: &[String]) -> Option<Located> {
    search::<Opens>(request, folders).map(|(folder, names, parts)| Located { folder, names, whole: parts.len() == 1 })
}

/// The first folder holding a file of the canonical name and the wrong size, for the overview's note.
pub fn wrong_size_in(request: &Request, folders: &[String]) -> Option<String> {
    folders.iter().find(|f| length(&path::combine(Style::HOST, f, &request.file)).is_some_and(|n| n != request.size as u64)).cloned()
}

/// `FirmwareLibrary.PathFor`: where an install writes.
pub fn path_for(request: &Request, store: &str) -> String {
    path::combine(Style::HOST, store, &request.file)
}

/// `FirmwareLibrary.Install`: the picked file copied into the store under the canonical name when its size is right;
/// false without words when it is not, the words when the folder or the file could not be written.
pub fn install(request: &Request, source: &str, store: &str) -> Result<bool, String> {
    let Some(bytes) = Bytes::exact(source, request.size) else { return Ok(false) };
    std::fs::create_dir_all(store).map_err(|e| emusen_dotnet::file::io_message(&e, store))?;
    emusen_dotnet::file::write_all_bytes(&path_for(request, store), &bytes)?;
    Ok(true)
}

fn words(value: Option<&Value>) -> Result<Vec<String>, String> {
    match value {
        Some(Value::Array(items)) => items.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(|| "a list holds a value that is not text".to_string())).collect(),
        None | Some(Value::Null) => Ok(Vec::new()),
        _ => Err("a list is not a list".to_string()),
    }
}

/// A request from its JSON: `file`, `size`, `alternates` and `parts`, a list of lists.
pub fn request_from(value: &Value) -> Result<Request, String> {
    let file = value.get("file").and_then(Value::as_str).ok_or("a request has no file")?.to_string();
    let size = match value.get("size") {
        Some(Value::Number(n)) => n.parse::<i32>().map_err(|_| format!("a request's size {n} is not a whole number"))?,
        _ => return Err("a request has no size".to_string()),
    };
    let parts = match value.get("parts") {
        Some(Value::Array(forms)) => forms.iter().map(|f| words(Some(f))).collect::<Result<_, _>>()?,
        _ => Vec::new(),
    };
    Ok(Request { file, size, alternates: words(value.get("alternates"))?, parts })
}

/// A list of folders from its JSON.
pub fn folders_from(value: Option<&Value>) -> Result<Vec<String>, String> {
    words(value)
}

fn text_list(items: &[String]) -> Value {
    Value::Array(items.iter().map(|s| Value::string(s.as_str())).collect())
}

fn compact(value: &Value) -> String {
    let mut out = String::new();
    json::write_compact(&mut out, value);
    out
}

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect())
}

/// The folders and the setting's value as JSON: `folders`, `state`, `path`, `written`, `problem`.
pub fn folders_json(list: &[String], value: &FolderValue) -> String {
    compact(&object(vec![
        ("folders", text_list(list)),
        ("state", Value::string(value.state.name())),
        ("path", value.path.as_deref().map_or(Value::Null, Value::string)),
        ("written", Value::string(value.written.as_str())),
        ("problem", value.problem().map_or(Value::Null, Value::string)),
    ]))
}

/// What was found as an envelope: the JSON's length in four little-endian bytes, the JSON (`folder`, `names`,
/// `lengths`), then every part's bytes in order.
pub fn found_envelope(found: &Found) -> Vec<u8> {
    let head = compact(&object(vec![
        ("folder", Value::string(found.folder.as_str())),
        ("names", text_list(&found.names)),
        ("lengths", Value::Array(found.parts.iter().map(|p| Value::number(p.len())).collect())),
    ]));
    let mut out = Vec::with_capacity(4 + head.len() + found.parts.iter().map(Vec::len).sum::<usize>());
    out.extend_from_slice(&(head.len() as u32).to_le_bytes());
    out.extend_from_slice(head.as_bytes());
    for part in &found.parts {
        out.extend_from_slice(part);
    }
    out
}

/// Where the image is and where a wrong-sized file of its name is, as JSON: `found` (`folder`, `names`, `whole`, or
/// null) and `wrong` (a folder, or null; looked for only when nothing is found).
pub fn located_json(located: Option<&Located>, wrong: Option<&str>) -> String {
    let found = located.map_or(Value::Null, |l| object(vec![("folder", Value::string(l.folder.as_str())), ("names", text_list(&l.names)), ("whole", Value::Bool(l.whole))]));
    compact(&object(vec![("found", found), ("wrong", wrong.map_or(Value::Null, Value::string))]))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dir(std::path::PathBuf);

    impl Dir {
        fn new(label: &str) -> Dir {
            let d = std::env::temp_dir().join(format!("emusen-firmware-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).unwrap();
            Dir(d)
        }
        fn sub(&self, name: &str) -> String {
            let p = self.0.join(name);
            std::fs::create_dir_all(&p).unwrap();
            p.to_string_lossy().into_owned()
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn put(folder: &str, name: &str, size: usize, fill: u8) {
        std::fs::write(format!("{folder}/{name}"), vec![fill; size]).unwrap();
    }

    fn widget() -> Request {
        Request { file: "widget.rom".into(), size: 64, alternates: vec!["w.bin".into()], parts: vec![vec!["w.lo".into(), "w.hi".into()]] }
    }

    #[test]
    fn the_chosen_folder_wins_a_tie_and_a_wrong_size_is_passed_over() {
        let d = Dir::new("tie");
        let (chosen, store) = (d.sub("chosen"), d.sub("store"));
        put(&chosen, "widget.rom", 63, 1);
        put(&store, "widget.rom", 64, 2);
        let folders = vec![chosen.clone(), store.clone()];
        let found = find(&widget(), &folders).unwrap();
        assert_eq!((found.folder.as_str(), found.parts[0][0]), (store.as_str(), 2));
        put(&chosen, "w.bin", 64, 3);
        assert_eq!(find(&widget(), &folders).unwrap().names, ["w.bin"]);
        assert_eq!(locate(&widget(), &folders).unwrap(), Located { folder: chosen.clone(), names: vec!["w.bin".into()], whole: true });
        assert_eq!(wrong_size_in(&widget(), &folders), Some(chosen));
    }

    #[test]
    fn a_split_form_is_taken_whole_from_one_folder_and_only_at_the_size() {
        let d = Dir::new("split");
        let (a, b) = (d.sub("a"), d.sub("b"));
        put(&a, "w.lo", 32, 1);
        put(&b, "w.hi", 32, 2);
        assert_eq!(find(&widget(), &[a.clone(), b.clone()]), None);
        put(&b, "w.lo", 31, 3);
        assert_eq!(find(&widget(), &[a.clone(), b.clone()]), None);
        put(&b, "w.lo", 32, 3);
        let found = find(&widget(), &[a, b.clone()]).unwrap();
        assert_eq!((found.folder, found.names.len(), found.parts[0][0], found.parts[1][0]), (b, 2, 3, 2));
    }

    #[test]
    fn install_writes_the_canonical_name_into_the_store_and_refuses_a_wrong_size() {
        let d = Dir::new("install");
        let (pick, store) = (d.sub("pick"), d.0.join("store").to_string_lossy().into_owned());
        put(&pick, "any.bin", 63, 7);
        assert_eq!(install(&widget(), &format!("{pick}/any.bin"), &store), Ok(false));
        put(&pick, "any.bin", 64, 7);
        assert_eq!(install(&widget(), &format!("{pick}/any.bin"), &store), Ok(true));
        assert_eq!(std::fs::read(format!("{store}/widget.rom")).unwrap(), vec![7; 64]);
    }

    #[test]
    fn the_store_alone_when_the_setting_names_the_store_or_nothing_usable() {
        let d = Dir::new("order");
        let store = d.sub("store");
        assert_eq!(folders(None, "/h", &store).0, [store.clone()]);
        assert_eq!(folders(Some(&format!("{store}/")), "/h", &store).0, [store.clone()]);
        assert_eq!(folders(Some("relative"), "/h", &store).0, [store.clone()]);
        let absent = format!("{}/absent", d.0.to_string_lossy());
        assert_eq!(folders(Some(&absent), "/h", &store).0, [absent, store]);
    }

    #[test]
    fn a_request_reads_from_its_json() {
        let v = json::parse(r#"{"file":"a.rom","size":8,"alternates":["b"],"parts":[["c","d"]]}"#).unwrap();
        assert_eq!(request_from(&v).unwrap(), Request { file: "a.rom".into(), size: 8, alternates: vec!["b".into()], parts: vec![vec!["c".into(), "d".into()]] });
    }
}
