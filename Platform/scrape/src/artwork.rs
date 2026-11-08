//! Box art the player keeps in a folder, matched to a game by its file name, `ArtworkIndex` ported: a folder named for a
//! console or a libretro system scopes what is under it to that console. The first of a cover's sources. See
//! EmuSen_Settings_Reference.md §4.33 and EmuSen_Backend.md §30.6.

use std::path::Path;
use crate::consoles::Core;
use emusen_galaxia::files::{self, Options};
use std::collections::HashMap;

pub const IMAGE_EXTENSIONS: [&str; 4] = [".png", ".jpg", ".jpeg", ".webp"];
const UNSAFE: [char; 11] = ['&', '*', '/', ':', '`', '<', '>', '?', '\\', '|', '"'];

#[derive(Default)]
pub struct Index {
    by_name: HashMap<(Option<String>, String), String>,
}

/// libretro-thumbnails' rule for a name that has to be a file name, trimmed and in lower case.
pub fn key(name: &str) -> String {
    let replaced: String = name.chars().map(|c| if UNSAFE.contains(&c) { '_' } else { c }).collect();
    replaced.trim_matches(char::is_whitespace).to_lowercase()
}

/// The name without its region and revision tags: `\s*[\(\[][^\)\]]*[\)\]]` taken out; the name itself if nothing is left.
pub fn untagged(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    'outer: while i < chars.len() {
        let mut q = i;
        while q < chars.len() && chars[q].is_whitespace() {
            q += 1;
        }
        if q < chars.len() && (chars[q] == '(' || chars[q] == '[') {
            if let Some(r) = (q + 1..chars.len()).find(|&r| chars[r] == ')' || chars[r] == ']') {
                i = r + 1;
                continue 'outer;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    let stripped = out.trim_matches(char::is_whitespace);
    if stripped.is_empty() { name.to_string() } else { stripped.to_string() }
}

impl Index {
    pub fn scan(directory: Option<&str>, cores: &[Core]) -> Index {
        let mut index = Index::default();
        let Some(dir) = directory.filter(|d| !d.trim().is_empty() && files::directory_exists(d)) else { return index };
        let mut console_by_folder: Vec<(String, String)> = Vec::new();
        let mut set = |name: &str, console: &str| match console_by_folder.iter_mut().find(|(n, _)| n.eq_ignore_ascii_case(&name)) {
            Some(entry) => entry.1 = console.to_string(),
            None => console_by_folder.push((name.to_string(), console.to_string())),
        };
        for core in cores {
            set(&core.console, &core.console);
            for system in &core.cheat_systems {
                set(system, &core.console);
            }
        }
        let _ = files::visit_files_with(dir, Options { skip_hidden: true, ignore_inaccessible: true }, |file, _| {
            let ext = file.extension().unwrap_or_default().to_string_lossy();
            if ext.is_empty() || !IMAGE_EXTENSIONS.iter().any(|e| e.trim_start_matches('.').eq_ignore_ascii_case(&ext)) {
                return;
            }
            let folder = file.strip_prefix(dir).ok().and_then(Path::parent).unwrap_or(Path::new(""));
            let console = folder.components().find_map(|part| console_by_folder.iter().find(|(n, _)| n.eq_ignore_ascii_case(&part.as_os_str().to_string_lossy())).map(|(_, c)| c.clone()));
            index.add(console, &file.file_stem().unwrap_or_default().to_string_lossy(), &file.to_string_lossy());
        });
        index
    }

    pub fn add(&mut self, console: Option<String>, stem: &str, file: &str) {
        self.by_name.insert((console.clone(), key(stem)), file.to_string());
        self.by_name.entry((console, key(&untagged(stem)))).or_insert_with(|| file.to_string());
    }

    /// The exact name first, then the name without its tags; a console's own folder before a shared one.
    pub fn find(&self, console: Option<&str>, rom_stem: &str) -> Option<String> {
        let scopes: Vec<Option<String>> = match console {
            None => vec![None],
            Some(c) => vec![Some(c.to_string()), None],
        };
        for k in [key(rom_stem), key(&untagged(rom_stem))] {
            for scope in &scopes {
                if let Some(file) = self.by_name.get(&(scope.clone(), k.clone())) {
                    return Some(file.clone());
                }
            }
        }
        None
    }
}
