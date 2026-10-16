//! `man`'s pages: one file each, `<name>.<section>`, in the shipped modules' `man/` beside the program, read without
//! running anything of a module's. The C# `ManPages`. See EmuSen_DianaSh.md §5.5 and §13.5.

use crate::dotnet;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree;
use std::collections::HashMap;
use std::sync::OnceLock;

/// `<program>/dianash/man`, outside the sandbox as the program's own files are.
pub fn directory() -> String {
    path::combine(Style::HOST, &tree::shell_modules_directory(), "man")
}

/// The pages, by name ignoring case, indexed once per process; the first of two names in ordinal order stands.
fn index() -> &'static HashMap<String, String> {
    static INDEX: OnceLock<HashMap<String, String>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut pages = HashMap::new();
        let Ok(entries) = std::fs::read_dir(directory()) else { return pages };
        let mut files: Vec<String> = entries.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).map(|e| e.path().to_string_lossy().into_owned()).collect();
        files.sort();
        for file in files {
            let name = path::file_name(Style::HOST, &file);
            let Some((stem, section)) = name.rsplit_once('.') else { continue };
            if section.len() == 1 && matches!(section.as_bytes()[0], b'1'..=b'9') {
                pages.entry(dotnet::fold(stem)).or_insert(file.clone());
            }
        }
        pages
    })
}

/// A page's text: the file's, less the newline a text file ends in.
pub fn lookup(name: &str) -> Option<String> {
    let file = index().get(&dotnet::fold(name))?;
    let mut text = dotnet::read_all_text(file).ok()?;
    if text.ends_with('\n') {
        text.pop();
    }
    Some(text)
}

/// Every page's name, for a test that each has a case.
pub fn names() -> Vec<String> {
    index().values().map(|file| path::file_name_without_extension(Style::HOST, file).to_string()).collect()
}
