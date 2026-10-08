//! Every error a frontend shows or swallows, one plain-text file a day: the C# `ErrorLog`. See
//! EmuSen_Settings_Reference.md §4.70.
//!
//! The caller supplies the time as text it formatted itself, so that a line reads the same whatever calendar or
//! separators the machine uses, and this owns the rest: the entry's shape, the file's name, the fourteen days kept
//! and the eight megabytes a day may hold. A log that cannot be written is never a second fault.

use crate::cheats::{ends_as, files_in};
use crate::config;
use crate::dotnet_path::{self as path, Style, is_blank};
use crate::dotnet_text::{replace_line_endings, utf16_len};
use crate::models::{APP_SETTINGS_FILE, APP_SETTINGS_ID, MODELS};
use crate::tree::{Directory, Tree, file_exists};
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

pub const PREFIX: &str = "emusen_";
pub const KEEP_DAYS: u64 = 14;
pub const MAX_BYTES: u64 = 8 << 20;

static GATE: Mutex<()> = Mutex::new(());
static PRUNED: AtomicBool = AtomicBool::new(false);

/// `ErrorLog.DefaultRoot`: the folder used when the setting names none this machine can make.
pub fn default_root(tree: &Tree) -> String {
    tree.directory(Directory::LogDefault).unwrap_or_default()
}

/// `ErrorLog.Usable`: a folder that is named and can be made; made by the asking.
pub fn usable(directory: Option<&str>) -> bool {
    match directory {
        Some(directory) if !is_blank(Some(directory)) => std::fs::create_dir_all(directory).is_ok(),
        _ => false,
    }
}

/// `ErrorLog.Root`: the folder given, else the `LogDirectory` setting when it can be made, else the default; with what reading the settings had to report.
pub fn root(tree: &Tree, directory_override: Option<&str>) -> (String, Option<String>) {
    if let Some(directory) = directory_override {
        return (directory.to_string(), None);
    }
    let (settings, said) = config::load_or_new(tree, &MODELS[APP_SETTINGS_ID as usize], None, APP_SETTINGS_FILE);
    let configured = settings.get("LogDirectory").and_then(|value| value.as_str());
    (if usable(configured) { configured.unwrap_or_default().to_string() } else { default_root(tree) }, said)
}

/// `ErrorLog.PathFor`: a day's file in a root, the day as the caller's `yyyyMMdd`.
pub fn path_for(root: &str, day: &str) -> String {
    path::combine(Style::HOST, root, &format!("{PREFIX}{day}.log"))
}

fn one_line(text: &str) -> String {
    replace_line_endings(text, " \u{23CE} ")
}

/// One entry: its line, its context on a second, and a fault's text indented under them.
pub fn format_entry(stamp: &str, level: &str, area: &str, message: &str, context: Option<&str>, fault: Option<&str>) -> String {
    let mut entry = format!("{stamp} {level} [{area}] {}\n", one_line(message));
    if let Some(context) = context.filter(|context| !is_blank(Some(context))) {
        entry.push_str(&format!("    context: {}\n", one_line(context)));
    }
    if let Some(fault) = fault {
        for line in replace_line_endings(fault, "\n").split('\n') {
            entry.push_str(&format!("    {line}\n"));
        }
    }
    entry
}

/// Removes the day files last written more than fourteen days before `now`, once in a process's life.
fn prune(root: &str, now: SystemTime) -> std::io::Result<()> {
    if PRUNED.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    let cutoff = now.checked_sub(Duration::from_secs(KEEP_DAYS * 24 * 60 * 60));
    for name in files_in(root)? {
        if !name.starts_with(PREFIX) || !ends_as(&name[PREFIX.len()..], ".log") {
            continue;
        }
        let file = path::combine(Style::HOST, root, &name);
        // The time is the entry's own, a link's and not its target's, which is the one .NET reads; a time that cannot be read is as old as files get.
        let old = match (std::fs::symlink_metadata(&file).and_then(|m| m.modified()), cutoff) {
            (Ok(written), Some(cutoff)) => written < cutoff,
            (Err(_), _) => true,
            (Ok(_), None) => false,
        };
        if old {
            let _ = std::fs::remove_file(&file);
        }
    }
    Ok(())
}

/// Appends an entry to a day's file, the file named by `path_for`; false when the day is full or the entry could not be written. The entry that fills the day carries one last notice.
///
/// No text is an entry its caller could not put into UTF-8: it fails where C# fails with one, at the write, after the folder is pruned and the file made.
pub fn append_to(file: &str, stamp: &str, now: SystemTime, text: Option<&str>) -> bool {
    append_entry(file, stamp, now, text).is_some()
}

/// Appends an entry to its day's file in a root and returns the file; none when it was not written.
pub fn append(root: &str, day: &str, stamp: &str, now: SystemTime, text: &str) -> Option<String> {
    let file = path_for(root, day);
    append_to(&file, stamp, now, Some(text)).then_some(file)
}

fn append_entry(file: &str, stamp: &str, now: SystemTime, text: Option<&str>) -> Option<()> {
    let _held = GATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = path::directory_name(Style::HOST, file)?;
    std::fs::create_dir_all(&root).ok()?;
    prune(&root, now).ok()?;
    let size = if file_exists(file) { std::fs::metadata(file).ok()?.len() } else { 0 };
    if size >= MAX_BYTES {
        return None;
    }
    let mut log = std::fs::OpenOptions::new().append(true).create(true).open(file).ok()?;
    let mut text = text?.to_string();
    // The length is counted as C# counts a string's, in UTF-16 code units, against a size in bytes.
    if size + utf16_len(&text) as u64 >= MAX_BYTES {
        text.push_str(&format!("{stamp} WARN [log] this file reached {} MB; nothing more is written to it today\n", MAX_BYTES >> 20));
    }
    log.write_all(text.as_bytes()).ok()
}

/// The next entry prunes again; for a test.
pub fn reset() {
    PRUNED.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use crate::tree::Override;

    #[test]
    fn an_entry_keeps_a_messages_breaks_on_its_line_and_indents_a_fault() {
        assert_eq!(format_entry("2026-10-08 01:02:03.004", "WARN", "config", "first\nsecond", None, None), "2026-10-08 01:02:03.004 WARN [config] first \u{23CE} second\n");
        assert_eq!(
            format_entry("T", "ERROR", "themes", "failed", Some("Artflix\r\n(Revisited)"), Some("System.IO.IOException: inner\r\n   at A()\n   at B()")),
            "T ERROR [themes] failed\n    context: Artflix \u{23CE} (Revisited)\n    System.IO.IOException: inner\n       at A()\n       at B()\n"
        );
        assert_eq!(format_entry("T", "ERROR", "a", "m", Some("  "), Some("")), "T ERROR [a] m\n    \n");
    }

    // One test, since the pruning happens once in a process and the tests of a crate share one.
    #[test]
    fn entries_append_old_days_are_pruned_once_and_a_full_day_takes_one_last_notice() {
        let _held = crate::test_support::error_log_tests();
        let temp = TempDir::new("log");
        let root = temp.join("Logs");
        std::fs::create_dir_all(&root).unwrap();
        let now = SystemTime::now();
        let days = |days: u64| now - Duration::from_secs(days * 24 * 60 * 60);
        for (name, age) in [("emusen_20000101.log", 15), ("emusen_recent.log", 3), ("crash_20000101.txt", 400), ("emusen_.log", 20), ("emusen_old.txt", 20), ("emusen_thirteen.log", 13)] {
            let file = temp.join(&format!("Logs/{name}"));
            std::fs::write(&file, "x").unwrap();
            std::fs::File::options().write(true).open(&file).unwrap().set_modified(days(age)).unwrap();
        }
        // Thirteen days and a half: inside the fourteen kept.
        std::fs::File::options().write(true).open(temp.join("Logs/emusen_thirteen.log")).unwrap().set_modified(days(13) - Duration::from_secs(12 * 60 * 60)).unwrap();
        // A new link to an old file is judged by the link, and so is one that names nothing.
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(temp.join("Logs/emusen_20000101.log"), temp.join("Logs/emusen_linked.log")).unwrap();
            std::os::unix::fs::symlink(temp.join("nowhere"), temp.join("Logs/emusen_dangling.log")).unwrap();
        }
        reset();
        let file = append(&root, "20261008", "T", now, "first\n").unwrap();
        assert_eq!(file, temp.join("Logs/emusen_20261008.log"));
        assert_eq!(append(&root, "20261008", "T", now, "second\n"), Some(file.clone()));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "first\nsecond\n");
        let left = |name: &str| file_exists(&temp.join(&format!("Logs/{name}")));
        assert!(!left("emusen_20000101.log") && !left("emusen_.log"));
        assert!(left("emusen_recent.log") && left("crash_20000101.txt") && left("emusen_old.txt") && left("emusen_thirteen.log"));
        #[cfg(unix)]
        assert!(std::fs::symlink_metadata(temp.join("Logs/emusen_linked.log")).is_ok() && std::fs::symlink_metadata(temp.join("Logs/emusen_dangling.log")).is_ok());

        // An entry with no text is one that could not be encoded: the file is made, nothing is written, and it is not a success.
        assert!(!append_to(&temp.join("Logs/emusen_20261009.log"), "T", now, None));
        assert_eq!(std::fs::metadata(temp.join("Logs/emusen_20261009.log")).unwrap().len(), 0);

        // Pruned already: a file made old now is not looked at again until a reset.
        std::fs::write(temp.join("Logs/emusen_later.log"), "x").unwrap();
        std::fs::File::options().write(true).open(temp.join("Logs/emusen_later.log")).unwrap().set_modified(days(30)).unwrap();
        append(&root, "20261008", "T", now, "third\n").unwrap();
        assert!(left("emusen_later.log"));

        std::fs::write(&file, vec![b'x'; MAX_BYTES as usize - 10]).unwrap();
        assert!(append(&root, "20261008", "STAMP", now, "tips it over\n").is_some());
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.ends_with("tips it over\nSTAMP WARN [log] this file reached 8 MB; nothing more is written to it today\n"));
        assert_eq!(append(&root, "20261008", "T", now, "refused\n"), None);
        assert_eq!(std::fs::metadata(&file).unwrap().len(), text.len() as u64);

        // The room left is judged by the entry's length as C# counts it, in UTF-16 code units, not by the bytes it takes.
        let wide = "é😀\n";
        assert_eq!((utf16_len(wide), wide.len()), (4, 7));
        std::fs::write(&file, vec![b'x'; MAX_BYTES as usize - 5]).unwrap();
        assert!(append(&root, "20261008", "STAMP", now, wide).is_some());
        assert_eq!(std::fs::metadata(&file).unwrap().len(), MAX_BYTES - 5 + 7);

        std::fs::write(temp.join("blocker"), "a file where the folder should be").unwrap();
        assert_eq!(append(&temp.join("blocker"), "20261008", "T", now, "nowhere\n"), None);
    }

    #[test]
    fn the_root_is_the_setting_when_it_can_be_made_and_the_default_when_it_cannot() {
        let temp = TempDir::new("log");
        let mut tree = Tree { root: temp.join("root"), application_data: temp.join("appdata"), ..Tree::default() };
        tree.set_override(Override::Config, Some(temp.join("etc")));
        assert_eq!(root(&tree, Some("/given")), ("/given".to_string(), None));
        assert_eq!(root(&tree, None), (temp.join("appdata/EmuSen/Logs"), None));
        std::fs::create_dir_all(temp.join("etc")).unwrap();
        let settings = |log: &str| std::fs::write(temp.join("etc/appsettings.json"), format!("{{\"LogDirectory\":{log}}}")).unwrap();
        settings(&crate::json::quoted(&temp.join("made/Logs")).replace("\\u002B", "+"));
        assert_eq!(root(&tree, None), (temp.join("made/Logs"), None));
        assert!(crate::tree::directory_exists(&temp.join("made/Logs")));
        std::fs::write(temp.join("blocker"), "x").unwrap();
        settings(&crate::json::quoted(&temp.join("blocker/Logs")));
        assert_eq!(root(&tree, None).0, temp.join("appdata/EmuSen/Logs"));
        settings("5");
        let (fallback, said) = root(&tree, None);
        assert_eq!(fallback, temp.join("appdata/EmuSen/Logs"));
        assert!(said.unwrap().ends_with("$.LogDirectory is not a string. Falling back to defaults."));
        assert!(!usable(None) && !usable(Some("")) && !usable(Some("  ")));
    }
}
