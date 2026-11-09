//! `theme-audit`, theme-check's loaded step (EmuSen_Qt.md §3.7, man theme-check): a theme package's screens placed offscreen
//! through the theme host on a synthetic library and held to the kit's fit and reach audits; one `file:line kind words` a finding.

mod audit;
mod catalogue;
mod crowd;
mod world;

use std::path::{Path, PathBuf};

// The two QML modules the scene and every theme resolve, linked in by name.
extern crate emusen_theme_host;
extern crate emusen_theme_qt;

const USAGE: &str = "Usage: theme-audit [--state DIR] [--catalogue FILE] <folder>\n  Exit 1 for a finding, 0 for none, 2 when it could not run. See man theme-check.";

/// The folder, the recordings' folder and the catalogue's file, or the words for the usage.
fn options(args: &[String]) -> Result<(String, Option<PathBuf>, Option<PathBuf>), String> {
    let (mut folder, mut state, mut catalogue) = (None, None, None);
    let mut words = args.iter().skip(1);
    while let Some(word) = words.next() {
        match word.as_str() {
            "--state" => state = Some(PathBuf::from(words.next().ok_or(format!("theme-audit: --state needs a folder\n{USAGE}"))?)),
            "--catalogue" => catalogue = Some(PathBuf::from(words.next().ok_or(format!("theme-audit: --catalogue needs a file\n{USAGE}"))?)),
            other if other.starts_with("--") => return Err(format!("theme-audit: {other} is not an option\n{USAGE}")),
            other if folder.is_none() => folder = Some(other.to_owned()),
            _ => return Err(format!("theme-audit: one folder at a time\n{USAGE}")),
        }
    }
    Ok((folder.ok_or(format!("theme-audit: a folder is needed\n{USAGE}"))?, state, catalogue))
}

/// The catalogue's name (its file's stem, which labels its states) and its entries.
fn catalogue(file: &Path) -> Result<(String, Vec<catalogue::Entry>), String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("theme-audit: {}: {e}", file.display()))?;
    let entries = catalogue::parse(&text).map_err(|e| format!("theme-audit: {}: {e}", file.display()))?;
    Ok((file.file_stem().map_or(String::new(), |s| s.to_string_lossy().into_owned()), entries))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let run = |(folder, state, listed): (String, Option<PathBuf>, Option<PathBuf>)| {
        let listed = listed.as_deref().map(catalogue).transpose()?;
        audit::run(&PathBuf::from(&folder), &folder, state.as_deref(), listed.as_ref().map(|(name, e)| (name.as_str(), e.as_slice())))
    };
    let status = match options(&args).and_then(run) {
        Ok(lines) => {
            lines.iter().for_each(|l| println!("{l}"));
            i32::from(!lines.is_empty())
        }
        Err(words) => {
            eprintln!("{words}");
            2
        }
    };
    std::process::exit(status);
}
