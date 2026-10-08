//! One config file: where it is, how it is read and how it is written. The C# `ConfigFile<T>`, for the models this
//! crate owns and for the files whose types belong to a frontend. See EmuSen_Config_Reference.md §2.
//!
//! Reading is best effort: a missing file is no error, and a file that will not load is reported and its defaults
//! used. Writing is the temp file and the rename of `atomic`, and its failure is a `false` and nothing more.

use crate::atomic::{self, words};
use crate::dotnet_path::{self as path, Style};
use crate::json::{self, Value};
use crate::migration::copy_file;
use crate::model;
use crate::models::Model;
use crate::tree::{Tree, file_exists};

/// `ConfigFile.Exists`.
pub fn exists(tree: &Tree, category: Option<&str>, file: &str) -> bool {
    file_exists(&tree.config_path(category, file))
}

/// `ConfigFile.Delete`: true when the file is gone afterwards, whether or not it was there; false when its folder is not, or it would not go.
pub fn delete(tree: &Tree, category: Option<&str>, file: &str) -> bool {
    let target = tree.config_path(category, file);
    match std::fs::remove_file(&target) {
        Ok(()) => true,
        // .NET's delete says nothing of a file that is not there, and throws for a folder that is not.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => path::directory_name(Style::HOST, &target).is_some_and(|directory| crate::tree::directory_exists(&directory)),
        Err(_) => false,
    }
}

/// The file to read: the config file itself, or failing that the copy made from where it sat before Galaxia, or that old file where no copy could be made.
fn source(tree: &Tree, category: Option<&str>, file: &str) -> Option<String> {
    let destination = tree.config_path(category, file);
    if file_exists(&destination) {
        return Some(destination);
    }
    // Only an uncategorised file ever sat in the old place.
    if category.is_some() {
        return None;
    }
    let legacy = tree.legacy_config_path(file).filter(|legacy| file_exists(legacy))?;
    // Copied, not moved, and never over a file: the old one stays where an older build looks for it.
    Some(if copy_file(&legacy, &destination).is_ok() { destination } else { legacy })
}

/// A config file's text as `File.ReadAllText` gives it; none when there is no file, and the system's words when it will not read.
pub fn read_text(tree: &Tree, category: Option<&str>, file: &str) -> Result<Option<String>, String> {
    let Some(source) = source(tree, category, file) else {
        return Ok(None);
    };
    std::fs::read(&source).map(|bytes| Some(json::decode(&bytes))).map_err(|error| words(&error))
}

/// The folder a file goes in, made as C#'s save makes it before anything else can fail.
fn make_folder(target: &str) -> Result<(), String> {
    match path::directory_name(Style::HOST, target).filter(|directory| !directory.is_empty()) {
        Some(directory) => std::fs::create_dir_all(directory).map_err(|error| words(&error)),
        None => Ok(()),
    }
}

/// Text written as a config file, through a temp file and a rename. False when it could not be; no text is a caller that could not produce any, which fails once the folder is made.
pub fn write_text(tree: &Tree, category: Option<&str>, file: &str, text: Option<&str>) -> bool {
    let target = tree.config_path(category, file);
    make_folder(&target).is_ok() && text.is_some_and(|text| atomic::write(&target, text.as_bytes()).is_ok())
}

/// What reading a model's file found.
#[derive(Debug, PartialEq)]
pub enum Loaded {
    /// No file.
    Missing,
    /// A file whose whole document is `null`, which .NET reads as no settings and no error.
    Null,
    Document(Value),
}

/// Text bound to a model: parsed, bound, and upgraded when asked. None for a document that is `null`.
pub fn bind_text(model: &Model, text: &str, upgrade: bool) -> Result<Option<Value>, String> {
    let document = json::parse(text).map_err(|error| error.to_string())?;
    let mut bound = model.schema.bind(&document).map_err(|error| error.to_string())?;
    if let (true, Some(upgrade), Some(bound)) = (upgrade, model.upgrade, bound.as_mut()) {
        upgrade(bound);
    }
    Ok(bound)
}

/// A new instance of a model, upgraded when asked, as C# upgrades the settings it makes when there is no file.
pub fn new_instance(model: &Model, upgrade: bool) -> Value {
    let mut new = model.schema.new_instance();
    if let (true, Some(upgrade)) = (upgrade, model.upgrade) {
        upgrade(&mut new);
    }
    new
}

/// `ConfigFile<T>.Load` for a model: the error is the words to report, after which the caller falls back to defaults.
pub fn load(tree: &Tree, model: &Model, category: Option<&str>, file: &str, upgrade: bool) -> Result<Loaded, String> {
    let Some(text) = read_text(tree, category, file)? else {
        return Ok(Loaded::Missing);
    };
    Ok(bind_text(model, &text, upgrade)?.map_or(Loaded::Null, Loaded::Document))
}

/// The diagnostic for a config file that would not load.
pub fn load_diagnostic(tree: &Tree, category: Option<&str>, file: &str, words: &str) -> String {
    format!("{}: {words} Falling back to defaults.", tree.config_path(category, file))
}

/// A model's settings as a program wants them: the file's, upgraded, or a new instance's when there is no file or it will not load, with the diagnostic for the second case.
pub fn load_or_new(tree: &Tree, model: &Model, category: Option<&str>, file: &str) -> (Value, Option<String>) {
    match load(tree, model, category, file, true) {
        Ok(Loaded::Document(document)) => (document, None),
        Ok(Loaded::Missing | Loaded::Null) => (new_instance(model, true), None),
        Err(words) => (new_instance(model, true), Some(load_diagnostic(tree, category, file, &words))),
    }
}

/// A model's document as its file's bytes. `canonical` is the document as JSON, or none where the caller could not produce it, which fails after the folder is made, as C# fails.
fn bytes_for(model: &Model, canonical: Option<&str>) -> Result<String, String> {
    let canonical = canonical.ok_or("The settings could not be serialized.")?;
    match bind_text(model, canonical, false)? {
        Some(bound) => model::format(model.schema, &bound),
        // Settings that are null are written as that word, which is what C# writes for them.
        None => Ok("null".to_string()),
    }
}

/// `ConfigFile<T>.Save` for a model: the folder made, the document formatted, and the bytes put in place by a rename.
pub fn save(tree: &Tree, model: &Model, category: Option<&str>, file: &str, canonical: Option<&str>) -> Result<(), String> {
    let target = tree.config_path(category, file);
    make_folder(&target)?;
    atomic::write(&target, bytes_for(model, canonical)?.as_bytes())
}

/// `CheatFile.LoadFrom`: a model read from any path; none for a file that is missing, unreadable or will not bind.
pub fn load_from(model: &Model, file: &str) -> Option<Value> {
    let text = json::decode(&std::fs::read(file).ok()?);
    bind_text(model, &text, false).ok().flatten()
}

/// `CheatFile.SaveTo`: a model written straight to any path the player picked, with no temp file, as C# writes it.
pub fn save_to(model: &Model, file: &str, canonical: Option<&str>) -> bool {
    let write = || -> Result<(), String> {
        let directory = path::directory_name(Style::HOST, &path::full_path(file)).ok_or("The path has no folder.")?;
        std::fs::create_dir_all(directory).map_err(|error| words(&error))?;
        std::fs::write(file, bytes_for(model, canonical)?).map_err(|error| words(&error))
    };
    write().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{APP_SETTINGS_ID, AUDIO_CONFIG_ID, CHEAT_FILE_ID, MODELS};
    use crate::test_support::TempDir;
    use crate::tree::Override;

    fn tree(temp: &TempDir) -> Tree {
        let mut tree = Tree { root: temp.join("root"), application_data: temp.join("appdata"), ..Tree::default() };
        tree.set_override(Override::Config, Some(temp.join("etc")));
        tree.set_override(Override::Legacy, Some(temp.join("legacy")));
        tree
    }

    fn put(file: &str, contents: &[u8]) {
        std::fs::create_dir_all(path::directory_name(Style::HOST, file).unwrap()).unwrap();
        std::fs::write(file, contents).unwrap();
    }

    const AUDIO: &Model = &MODELS[AUDIO_CONFIG_ID as usize];
    const APP: &Model = &MODELS[APP_SETTINGS_ID as usize];

    #[test]
    fn a_missing_file_a_null_document_and_a_broken_one_are_three_answers() {
        let temp = TempDir::new("config");
        let tree = tree(&temp);
        assert_eq!(load(&tree, AUDIO, None, "audio.json", false), Ok(Loaded::Missing));
        put(&temp.join("etc/audio.json"), b" null ");
        assert_eq!(load(&tree, AUDIO, None, "audio.json", false), Ok(Loaded::Null));
        put(&temp.join("etc/audio.json"), b"{ \"SampleRate\": 44100, // by hand\n }");
        let Ok(Loaded::Document(config)) = load(&tree, AUDIO, None, "audio.json", false) else { panic!() };
        assert_eq!(config.get("SampleRate"), Some(&Value::number("44100")));
        put(&temp.join("etc/audio.json"), b"{ \"SampleRate\": \"fast\" }");
        assert_eq!(load(&tree, AUDIO, None, "audio.json", false), Err("$.SampleRate is not a whole number.".to_string()));
        put(&temp.join("etc/audio.json"), b"{ not json at all");
        let words = load(&tree, AUDIO, None, "audio.json", false).unwrap_err();
        assert_eq!(load_diagnostic(&tree, None, "audio.json", &words), format!("{}: Neither a name nor '}}' in an object at line 1, byte 3. Falling back to defaults.", temp.join("etc/audio.json")));
    }

    #[test]
    fn a_saved_file_is_whole_reads_back_and_leaves_no_temp() {
        let temp = TempDir::new("config");
        let tree = tree(&temp);
        save(&tree, AUDIO, None, "audio.json", Some(r#"{"SampleRate":48000,"MasterVolume":0.8}"#)).unwrap();
        let text = std::fs::read_to_string(temp.join("etc/audio.json")).unwrap();
        assert!(text.starts_with('{') && text.ends_with('}') && text.contains("\"SampleRate\": 48000,") && text.contains("\"Muted\": false,"));
        assert!(!file_exists(&temp.join("etc/audio.json.tmp")));
        assert!(exists(&tree, None, "audio.json"));
        assert!(save(&tree, AUDIO, None, "audio.json", None).is_err());
        // What cannot be written still has its folder made, and settings that are null are written as that word.
        assert!(save(&tree, AUDIO, Some("group"), "x.json", None).is_err());
        assert!(crate::tree::directory_exists(&temp.join("etc/group")) && !file_exists(&temp.join("etc/group/x.json")));
        save(&tree, AUDIO, Some("group"), "null.json", Some(" null ")).unwrap();
        assert_eq!(std::fs::read_to_string(temp.join("etc/group/null.json")).unwrap(), "null");
        assert!(!write_text(&tree, Some("made"), "x.json", None));
        assert!(crate::tree::directory_exists(&temp.join("etc/made")));
        assert!(write_text(&tree, Some("made"), "x.json", Some("{\"a\":1}")));
        assert_eq!(read_text(&tree, Some("made"), "x.json"), Ok(Some("{\"a\":1}".to_string())));
        assert!(save(&tree, AUDIO, None, "audio.json", Some(r#"{"RateControlMaxDeviation":1e999}"#)).is_err());
        assert_eq!(std::fs::read_to_string(temp.join("etc/audio.json")).unwrap(), text);
        assert!(delete(&tree, None, "audio.json"));
        assert!(delete(&tree, None, "audio.json"));
        assert!(!delete(&tree, Some("no-such-folder"), "x.json"));
        assert!(!exists(&tree, None, "audio.json"));
    }

    #[test]
    fn a_file_from_before_galaxia_is_copied_in_once_and_left_where_it_was() {
        let temp = TempDir::new("config");
        let tree = tree(&temp);
        put(&temp.join("legacy/audio.json"), b"{\"SampleRate\":22050}");
        put(&temp.join("legacy/x.json"), b"{}");
        assert_eq!(read_text(&tree, None, "audio.json"), Ok(Some("{\"SampleRate\":22050}".to_string())));
        assert_eq!(std::fs::read(temp.join("etc/audio.json")).unwrap(), b"{\"SampleRate\":22050}");
        assert!(file_exists(&temp.join("legacy/audio.json")));
        put(&temp.join("etc/audio.json"), b"{\"SampleRate\":1}");
        put(&temp.join("legacy/audio.json"), b"{\"SampleRate\":2}");
        assert_eq!(read_text(&tree, None, "audio.json"), Ok(Some("{\"SampleRate\":1}".to_string())));
        // A file of a category never sat in the old place, whatever is there under its name.
        assert_eq!(read_text(&tree, Some("cheats"), "x.json"), Ok(None));
        assert!(!file_exists(&temp.join("etc/cheats/x.json")));
        let mut alone = tree.clone();
        alone.set_override(Override::Legacy, None);
        assert_eq!(read_text(&alone, None, "appsettings.json"), Ok(None));
    }

    #[test]
    fn a_copy_that_cannot_be_made_reads_the_old_file_in_place() {
        let temp = TempDir::new("config");
        let tree = tree(&temp);
        put(&temp.join("legacy/audio.json"), b"{}");
        std::fs::create_dir_all(temp.join("etc/audio.json")).unwrap();
        assert_eq!(read_text(&tree, None, "audio.json"), Ok(Some("{}".to_string())));
    }

    #[test]
    fn settings_are_upgraded_when_loaded_for_use_and_not_when_only_read() {
        let temp = TempDir::new("config");
        let tree = tree(&temp);
        let (new, said) = load_or_new(&tree, APP, None, "appsettings.json");
        assert_eq!((new.get("SelectedCoreUpgraded"), said), (Some(&Value::Bool(true)), None));
        put(&temp.join("etc/appsettings.json"), b"{\"SelectedCore\":\"SNES (Venus)\",\"LogDirectory\":\"/logs\"}");
        let (settings, said) = load_or_new(&tree, APP, None, "appsettings.json");
        assert_eq!((settings.get("SelectedCore").and_then(Value::as_str), said), (Some("All consoles"), None));
        assert_eq!(settings.get("LogDirectory").and_then(Value::as_str), Some("/logs"));
        let Ok(Loaded::Document(read)) = load(&tree, APP, None, "appsettings.json", false) else { panic!() };
        assert_eq!(read.get("SelectedCore").and_then(Value::as_str), Some("SNES (Venus)"));
        put(&temp.join("etc/appsettings.json"), b"[]");
        let (fallback, said) = load_or_new(&tree, APP, None, "appsettings.json");
        assert_eq!(fallback.get("SelectedCore").and_then(Value::as_str), Some("All consoles"));
        assert_eq!(said, Some(format!("{}: $ is not an object. Falling back to defaults.", temp.join("etc/appsettings.json"))));
    }

    #[test]
    fn a_cheat_file_goes_to_any_path_and_comes_back() {
        let temp = TempDir::new("config");
        let cheats = &MODELS[CHEAT_FILE_ID as usize];
        let file = temp.join("exports/deep/mine.json");
        assert!(save_to(cheats, &file, Some(r#"{"Cheats":[{"Description":"é","Writes":[{"Address":"7E0019"}]}]}"#)));
        let back = load_from(cheats, &file).unwrap();
        assert_eq!(model::transport(&back), r#"{"Cheats":[{"Kind":"RamPoke","Writes":[{"Space":"","Address":"7E0019","Value":"00","Width":1,"Type":"Set","BitPosition":null,"BigEndian":false,"RepeatCount":1,"RepeatAddAddress":"0","RepeatAddValue":"0"}],"Compare":null,"Description":"é","Enabled":true,"Space":null,"Address":null,"Value":null}]}"#);
        assert!(!save_to(cheats, &temp.join("made/anyway/broken.json"), None));
        assert!(crate::tree::directory_exists(&temp.join("made/anyway")));
        assert_eq!(load_from(cheats, &temp.join("absent.json")), None);
        // Written straight to its path: a path that is a folder strands no temp file beside it.
        std::fs::create_dir_all(temp.join("taken/list.json")).unwrap();
        assert!(!save_to(cheats, &temp.join("taken/list.json"), Some("{}")));
        assert_eq!(std::fs::read_dir(temp.join("taken")).unwrap().count(), 1);
        put(&temp.join("bad.json"), b"{\"Cheats\":5}");
        assert_eq!(load_from(cheats, &temp.join("bad.json")), None);
    }
}
