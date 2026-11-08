//! `theme-audit`, theme-check's loaded step (EmuSen_Qt.md §3.7, man theme-check): a theme package's screens placed offscreen
//! through the theme host on a synthetic library and held to the kit's fit and reach audits; one `file:line kind words` a finding.

mod audit;
mod crowd;
mod glue;
mod world;

use std::path::PathBuf;

// The two QML modules the scene and every theme resolve, linked in by name.
extern crate emusen_theme_host;
extern crate emusen_theme_qt;

const USAGE: &str = "Usage: theme-audit [--state DIR] <folder>\n  Exit 1 for a finding, 0 for none, 2 when it could not run. See man theme-check.";

/// The folder and the recordings' folder, or the words for the usage.
fn options(args: &[String]) -> Result<(String, Option<PathBuf>), String> {
    let (mut folder, mut state) = (None, None);
    let mut words = args.iter().skip(1);
    while let Some(word) = words.next() {
        match word.as_str() {
            "--state" => state = Some(PathBuf::from(words.next().ok_or(format!("theme-audit: --state needs a folder\n{USAGE}"))?)),
            other if other.starts_with("--") => return Err(format!("theme-audit: {other} is not an option\n{USAGE}")),
            other if folder.is_none() => folder = Some(other.to_owned()),
            _ => return Err(format!("theme-audit: one folder at a time\n{USAGE}")),
        }
    }
    Ok((folder.ok_or(format!("theme-audit: a folder is needed\n{USAGE}"))?, state))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let status = match options(&args).and_then(|(folder, state)| audit::run(&PathBuf::from(&folder), &folder, state.as_deref())) {
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
