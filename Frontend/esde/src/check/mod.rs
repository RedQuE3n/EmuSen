//! `check`: a folder of ES-DE themes measured as EmuSen_Qt.md §4.4 asks, as TSV for the emusen.dev command esde-check:
//! one row per theme, its diagnostics, and its property map of set, unknown and unsupported pairs. Each theme is loaded
//! for every selectable variant and each system asked for, as pass 3's survey did (EmuSen_BigPicture.md §25.8).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::loader::{self, capabilities, Choices, Code, Severity, ThemeSystem};
use crate::scene::mapping;

/// EmuSen's five systems, which the 2026-09-26 survey measured.
pub const SYSTEMS: [&str; 5] = ["nes", "snes", "gb", "gbc", "n64"];

/// One theme measured over its variants and the systems.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Measure {
    pub theme: String,
    pub root: PathBuf,
    pub variants: usize,
    pub loads: usize,
    pub themed_systems: usize,
    pub systems: usize,
    pub errors: usize,
    pub warnings: usize,
    /// Element kinds used, drawn or not.
    pub kinds: BTreeSet<String>,
    /// type.property pairs set, each with its state: set (drawn), unsupported, or unknown.
    pub properties: BTreeMap<String, &'static str>,
    /// Element kinds the scene does not draw, and wheel carousels, which it draws only in part.
    pub undrawn: BTreeSet<String>,
    /// Each distinct diagnostic at warning or above: severity, code, file relative to the theme, line, message.
    pub diagnostics: BTreeSet<(Severity, &'static str, String, u32, String)>,
}

impl Measure {
    /// Themed for every system asked for.
    pub fn loads(&self) -> bool {
        self.themed_systems == self.systems
    }

    pub fn count(&self, state: &str) -> usize {
        self.properties.values().filter(|s| **s == state).count()
    }

    /// Drawn whole: it loads, sets nothing unsupported, accepted or unknown, and uses no undrawn kind.
    pub fn whole(&self) -> bool {
        self.loads() && self.undrawn.is_empty() && self.properties.values().all(|s| *s == "set")
    }
}

/// The theme folder that holds capabilities.xml: the folder itself, else its first subfolder that does.
pub fn theme_root(folder: &Path) -> PathBuf {
    if folder.join(capabilities::FILE_NAME).is_file() {
        return folder.to_path_buf();
    }
    let mut subfolders: Vec<PathBuf> =
        std::fs::read_dir(folder).map(|r| r.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect()).unwrap_or_default();
    subfolders.sort();
    subfolders.into_iter().find(|d| d.join(capabilities::FILE_NAME).is_file()).unwrap_or_else(|| folder.to_path_buf())
}

/// A folder is a corpus when it holds no capabilities.xml of its own but its subfolders are themes; each subfolder is then measured.
pub fn themes_in(folder: &Path) -> Vec<PathBuf> {
    if folder.join(capabilities::FILE_NAME).is_file() {
        return vec![folder.to_path_buf()];
    }
    let mut themes: Vec<PathBuf> =
        std::fs::read_dir(folder).map(|r| r.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect()).unwrap_or_default();
    themes.sort();
    if themes.iter().any(|t| t.join(capabilities::FILE_NAME).is_file())
        || themes.iter().any(|t| theme_root(t).join(capabilities::FILE_NAME).is_file())
    {
        themes
    } else {
        vec![folder.to_path_buf()]
    }
}

/// Loads a theme for each selectable variant (or none, when none is offered) and each system, and gathers what was set and said.
pub fn measure(folder: &Path, systems: &[&str]) -> Measure {
    let root = theme_root(folder);
    let directory = root.to_string_lossy().into_owned();
    let caps = capabilities::read(&directory);
    let mut variants: Vec<Option<String>> = caps.variants.iter().filter(|v| v.selectable).map(|v| Some(v.name.clone())).collect();
    if variants.is_empty() {
        variants.push(None);
    }
    let name = folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut m = Measure { theme: name, root: root.clone(), variants: variants.len(), systems: systems.len(), ..Measure::default() };
    let relative =
        |file: &str| Path::new(file).strip_prefix(&root).map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| file.to_owned());
    for system in systems {
        let mut themed = false;
        for variant in &variants {
            let choices = Choices { variant: variant.clone(), ..Choices::default() };
            let theme = loader::load_with(&caps, &ThemeSystem::new(system, system, system), &choices, None);
            m.loads += 1;
            themed |= theme.is_themed();
            for d in &theme.diagnostics {
                match d.severity {
                    Severity::Error => m.errors += 1,
                    Severity::Warning => m.warnings += 1,
                    Severity::Debug => continue,
                }
                m.diagnostics.insert((d.severity, d.code.name(), relative(&d.file), d.line, d.message.clone()));
                if d.code == Code::UnknownProperty || d.code == Code::UnknownElement {
                    m.properties.insert(d.subject.clone(), "unknown");
                }
            }
            for e in theme.system_view.elements.iter().chain(&theme.gamelist_view.elements).chain(&theme.sound_view.elements) {
                m.kinds.insert(e.kind().to_owned());
                if e.kind() == "carousel" {
                    if let Some(wheel) = e.string("type").filter(|t| t.ends_with("Wheel")) {
                        m.undrawn.insert(format!("carousel:{wheel}"));
                    }
                }
                if e.kind() != "sound" && !mapping::is_drawn(e.kind()) {
                    m.undrawn.insert(e.kind().to_owned());
                }
                for property in e.explicit.keys() {
                    let pair = format!("{}.{property}", e.kind());
                    let state = match mapping::status(e.kind(), property) {
                        _ if e.kind() == "sound" => "set",
                        Some(mapping::Status::Drawn) => "set",
                        Some(mapping::Status::Accepted) => "accepted",
                        _ => "unsupported",
                    };
                    if m.properties.get(&pair).is_none_or(|s| *s == "set") {
                        m.properties.insert(pair, state);
                    }
                }
            }
        }
        m.themed_systems += usize::from(themed);
    }
    m
}

pub const SUMMARY_HEADER: &str =
    "theme\tloads\tsystems\tvariants\truns\terrors\twarnings\tset\tunknown\tunsupported\taccepted\twhole\tkinds\tundrawn";

/// The theme's row of the corpus measure.
pub fn summary_row(m: &Measure) -> String {
    let join = |set: &BTreeSet<String>| if set.is_empty() { "-".to_owned() } else { set.iter().cloned().collect::<Vec<_>>().join(",") };
    format!(
        "{}\t{}\t{}/{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        m.theme,
        if m.loads() { "yes" } else { "no" },
        m.themed_systems,
        m.systems,
        m.variants,
        m.loads,
        m.errors,
        m.warnings,
        m.properties.len(),
        m.count("unknown"),
        m.count("unsupported"),
        m.count("accepted"),
        if m.whole() { "yes" } else { "no" },
        join(&m.kinds),
        join(&m.undrawn)
    )
}

pub const DIAGNOSTICS_HEADER: &str = "theme\tseverity\tcode\tfile\tline\tmessage";

pub fn diagnostic_rows(m: &Measure) -> Vec<String> {
    m.diagnostics
        .iter()
        .map(|(severity, code, file, line, message)| {
            format!("{}\t{severity:?}\t{code}\t{file}\t{line}\t{}", m.theme, message.replace(['\t', '\n'], " "))
        })
        .collect()
}

pub const PROPERTIES_HEADER: &str = "theme\tpair\tstate";

pub fn property_rows(m: &Measure) -> Vec<String> {
    m.properties.iter().map(|(pair, state)| format!("{}\t{pair}\t{state}", m.theme)).collect()
}

/// What the command prints, and whether every theme loaded.
pub struct Report {
    pub text: String,
    pub all_load: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table {
    Summary,
    Diagnostics,
    Properties,
}

/// Measures every theme under the folders and prints one table, header first; a last line totals the summary.
pub fn report(folders: &[PathBuf], systems: &[&str], table: Table) -> Report {
    let mut lines = vec![match table {
        Table::Summary => SUMMARY_HEADER,
        Table::Diagnostics => DIAGNOSTICS_HEADER,
        Table::Properties => PROPERTIES_HEADER,
    }
    .to_owned()];
    let (mut themes, mut loading, mut whole, mut drawn_kinds) = (0, 0, 0, 0);
    for folder in folders {
        for theme in themes_in(folder) {
            let m = measure(&theme, systems);
            themes += 1;
            loading += usize::from(m.loads());
            whole += usize::from(m.whole());
            drawn_kinds += usize::from(m.undrawn.is_empty());
            match table {
                Table::Summary => lines.push(summary_row(&m)),
                Table::Diagnostics => lines.extend(diagnostic_rows(&m)),
                Table::Properties => lines.extend(property_rows(&m)),
            }
        }
    }
    if table == Table::Summary {
        lines.push(format!("# {themes} themes, {loading} load, {whole} drawn whole, {drawn_kinds} use no undrawn kind"));
    }
    Report { text: lines.join("\n") + "\n", all_load: loading == themes }
}
